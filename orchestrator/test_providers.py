import asyncio
import io
import json
import os
import threading
import time
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from unittest.mock import patch

from orchestrator.providers import (
    ChatClient, DecisionRequest, ProviderConfig, ProviderError,
    WindowRunner, load_clients, validate_decision,
)


DECISION = {"begruendung": "Reserven aufbauen.", "aktionen": [], "notiz": "privat", "wecker": None}


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        with self.server.lock:
            self.server.requests.append((self.path, body))
        if body['model'] == 'error':
            self.send_response(429)
            self.end_headers()
            self.wfile.write(b'private provider error body')
            return
        if body['model'] == 'redirect':
            self.send_response(307)
            self.send_header('Location', 'https://example.com/credentials')
            self.end_headers()
            return
        if body['model'] == 'slow':
            time.sleep(0.1)
        data = {'id': 'mock-1', 'model': 'resolved-model', 'usage': {'total_tokens': 42},
                'choices': [{'finish_reason': 'length' if body['model'] == 'truncated' else 'stop',
                             'message': {'content': json.dumps(DECISION)}}]}
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())


class IntegrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        cls.server.requests = []
        cls.server.lock = threading.Lock()
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()
        cls.thread.join()

    def client(self, model='test', **kwargs):
        return ChatClient(ProviderConfig(model, base_url=f'http://127.0.0.1:{self.server.server_port}/v1', **kwargs))

    def test_local_http_and_exact_path(self):
        response = self.client().complete([{'role': 'user', 'content': 'JSON bitte'}])
        self.assertEqual(validate_decision(response.content), DECISION)
        self.assertEqual(response.actual_model, 'resolved-model')
        self.assertEqual(response.usage['total_tokens'], 42)
        path, body = self.server.requests[-1]
        self.assertEqual(path, '/v1/chat/completions')
        self.assertEqual(body['max_tokens'], 1024)

    def test_window_isolation_and_barrier(self):
        runner = WindowRunner({'verwalter': self.client('slow'), 'stratege': self.client()})
        requests = [DecisionRequest('B', 'stratege', 900, {'secret': 'B-only'}, 'rules', 'B-memory'),
                    DecisionRequest('A', 'verwalter', 900, {'secret': 'A-only'}, 'rules', 'A-memory')]
        before = len(self.server.requests)
        results = asyncio.run(runner.run(requests))
        self.assertEqual([(r.player_id, r.error) for r in results], [('A', None), ('B', None)])
        sent = self.server.requests[before:]
        self.assertEqual(len(sent), 2)
        for _, body in sent:
            content = body['messages'][1]['content']
            self.assertNotEqual('A-only' in content, 'B-only' in content)
            self.assertNotEqual('A-memory' in content, 'B-memory' in content)
        self.assertEqual(requests[0].notebook, 'B-memory')

    def test_http_error_is_sanitized_and_not_retried(self):
        before = len(self.server.requests)
        with self.assertRaisesRegex(ProviderError, '^provider HTTP 429$'):
            self.client('error').complete([])
        self.assertEqual(len(self.server.requests), before + 1)

    def test_truncated_response_rejected(self):
        with self.assertRaisesRegex(ProviderError, 'truncated'):
            self.client('truncated').complete([])

    def test_redirect_does_not_forward_request(self):
        with self.assertRaisesRegex(ProviderError, 'redirect|HTTP 307'):
            self.client('redirect').complete([])

    def test_request_budget(self):
        client = self.client(max_requests=1)
        client.complete([])
        with self.assertRaisesRegex(ProviderError, 'budget exhausted'):
            client.complete([])

    def test_window_failure_remains_explicit(self):
        runner = WindowRunner({'verwalter': self.client('error')})
        result = asyncio.run(runner.run([DecisionRequest('A', 'verwalter', 0, {}, 'rules')]))[0]
        self.assertIsNone(result.decision)
        self.assertEqual(result.error, 'provider HTTP 429')

    def test_each_result_is_archived_before_barrier(self):
        records = []
        runner = WindowRunner({'v': self.client('slow'), 's': self.client()})
        requests = [DecisionRequest('A', 'v', 0, {}, ''), DecisionRequest('B', 's', 0, {}, '')]
        results = asyncio.run(runner.run(requests, on_result=records.append))
        self.assertEqual(len(records), 2)
        self.assertEqual({r.player_id for r in records}, {'A', 'B'})
        self.assertEqual([r.player_id for r in results], ['A', 'B'])

    def test_cli_archives_real_http_window_without_world_mutation(self):
        import subprocess
        import sys
        import tempfile
        from pathlib import Path
        with tempfile.TemporaryDirectory(dir=Path(__file__).parent) as directory:
            root = Path(directory)
            config = root / 'models.toml'
            window = root / 'window.json'
            output = root / 'results.jsonl'
            config.write_text(f'[roles.v]\nmodel="test"\nbase_url="http://127.0.0.1:{self.server.server_port}/v1"\n')
            window.write_text(json.dumps([{'player_id': 'A', 'role': 'v', 'sim_time': 0,
                                           'observation': {'private': 1}, 'rules': 'rules'}]))
            command = [sys.executable, '-m', 'orchestrator', '--config', str(config),
                       '--window', str(window), '--execute', '--output', str(output)]
            run = subprocess.run(command, capture_output=True, text=True, timeout=10)
            self.assertEqual(run.returncode, 0, run.stderr)
            records = [json.loads(line) for line in output.read_text().splitlines()]
            self.assertEqual([r['type'] for r in records], ['window_input', 'decision', 'window_complete'])
            self.assertEqual(records[1]['decision'], DECISION)
            original = output.read_bytes()
            repeated = subprocess.run(command, capture_output=True, text=True, timeout=10)
            self.assertNotEqual(repeated.returncode, 0)
            self.assertEqual(output.read_bytes(), original)

    def test_cli_restart_reuses_journal_without_second_http_call(self):
        import subprocess
        import sys
        import tempfile
        from pathlib import Path
        with tempfile.TemporaryDirectory(dir=Path(__file__).parent) as directory:
            root = Path(directory)
            config, window = root / 'models.toml', root / 'window.json'
            config.write_text(f'[roles.v]\nmodel="test"\nmax_requests=1\nbase_url="http://127.0.0.1:{self.server.server_port}/v1"\n')
            window.write_text(json.dumps([{'player_id': 'A', 'role': 'v', 'sim_time': 0,
                                           'observation': {}, 'rules': 'rules'}]))
            before = len(self.server.requests)
            common = [sys.executable, '-m', 'orchestrator', '--config', str(config),
                      '--window', str(window), '--execute', '--journal', str(root / 'run.sqlite3'),
                      '--run-id', 'epoch-test']
            for index in range(2):
                run = subprocess.run(common + ['--output', str(root / f'export-{index}.jsonl')],
                                     capture_output=True, text=True, timeout=10)
                self.assertEqual(run.returncode, 0, run.stderr)
            self.assertEqual(len(self.server.requests), before + 1)
            one = [json.loads(line) for line in (root / 'export-0.jsonl').read_text().splitlines()]
            two = [json.loads(line) for line in (root / 'export-1.jsonl').read_text().splitlines()]
            self.assertEqual(one, two)

    def test_duplicate_or_mixed_time_window_rejected(self):
        runner = WindowRunner({'v': self.client()})
        a = DecisionRequest('A', 'v', 0, {}, '')
        for batch in [[a, a], [a, DecisionRequest('B', 'v', 1, {}, '')]]:
            with self.assertRaises(ValueError):
                asyncio.run(runner.run(batch))

    def test_openrouter_request_contract_without_paid_call(self):
        client = ChatClient(ProviderConfig('vendor/model', provider='openrouter',
                            base_url='https://openrouter.ai/api/v1', api_key_env='TEST_KEY',
                            allow_remote=True))
        fake = io.BytesIO(json.dumps({'choices': [{'finish_reason': 'stop',
            'message': {'content': json.dumps(DECISION)}}]}).encode())
        schema = {'type': 'object', 'properties': {}, 'additionalProperties': False}
        with patch.dict(os.environ, {'TEST_KEY': 'test-not-a-real-key'}):
            with patch.object(client._opener, 'open', return_value=fake) as open_mock:
                client.complete([], schema=schema)
        request = open_mock.call_args.args[0]
        payload = json.loads(request.data)
        self.assertEqual(request.full_url, 'https://openrouter.ai/api/v1/chat/completions')
        self.assertEqual(payload['provider'], {'require_parameters': True, 'allow_fallbacks': False})
        self.assertTrue(payload['response_format']['json_schema']['strict'])
        self.assertEqual(request.get_header('Authorization'), 'Bearer test-not-a-real-key')

    def test_remote_requires_opt_in_and_https(self):
        for url in ['https://example.com/v1', 'http://example.com/v1']:
            with self.assertRaises(ValueError):
                ProviderConfig('m', base_url=url)
        with self.assertRaises(ValueError):
            ProviderConfig('m', base_url='http://example.com', allow_remote=True)

    def test_example_config_loads_offline(self):
        from pathlib import Path
        clients = load_clients(Path(__file__).with_name('models.example.toml'))
        self.assertEqual(set(clients), {'verwalter', 'feldherr', 'diplomat', 'stratege'})

    def test_untrusted_decisions_rejected(self):
        invalid = ['[]', '{"aktionen": [], "aktionen": []}', 'NaN',
                   '{"begruendung":"", "notiz":"", "aktionen":[{"typ":"x","wert":1e999}]}',
                   json.dumps({**DECISION, 'wecker': True}),
                   json.dumps({**DECISION, 'aktionen': [{}]}),
                   json.dumps({**DECISION, 'aktionen': [{'typ': 'x'}] * 11}),
                   json.dumps({**DECISION, 'admin': 'world'}),
                   json.dumps({**DECISION, 'begruendung': 'a ' * 151})]
        for raw in invalid:
            with self.subTest(raw=raw), self.assertRaises(ProviderError):
                validate_decision(raw)


if __name__ == '__main__':
    unittest.main()
