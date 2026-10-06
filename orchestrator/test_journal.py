import asyncio
from concurrent.futures import ThreadPoolExecutor
from dataclasses import replace
import json
from pathlib import Path
import tempfile
import threading
import unittest

from orchestrator.journal import DecisionJournal, JournalError
from orchestrator.audit import export, read_records
from orchestrator.providers import (
    Completion, DecisionRequest, ProviderConfig, ProviderError, WindowRunner,
)


class CountingClient:
    def __init__(self, config=None, error=False):
        self.config = config or ProviderConfig('mock')
        self.calls = 0
        self.error = error

    def complete(self, messages):
        self.calls += 1
        if self.error:
            raise ProviderError('provider HTTP 429')
        return Completion(json.dumps({'begruendung': '', 'aktionen': [], 'notiz': 'private'}),
                          'mock', 'mock', 'local', 'test', {'total_tokens': 7}, 0.01)


class JournalTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir=Path(__file__).parent)
        self.path = Path(self.temp.name) / 'journal.sqlite3'
        self.journal = DecisionJournal(self.path)
        self.client = CountingClient()
        self.clients = {'verwalter': self.client}
        self.request = DecisionRequest('A', 'verwalter', 0, {'private': 'A'}, 'rules')

    def tearDown(self):
        self.temp.cleanup()

    def run_window(self, requests=None, clients=None, journal=None):
        return asyncio.run(WindowRunner(clients or self.clients).run(
            requests or [self.request], journal=journal or self.journal, run_id='epoch-1'))

    def test_restart_reuses_identical_response_without_call(self):
        first = self.run_window()
        replacement = CountingClient()
        second = self.run_window(clients={'verwalter': replacement}, journal=DecisionJournal(self.path))
        self.assertEqual(first, second)
        self.assertEqual(self.client.calls, 1)
        self.assertEqual(replacement.calls, 0)
        self.assertEqual(self.journal.summary('epoch-1')['roles']['verwalter']['reported_total_tokens'], 7)

    def test_inputs_and_prompt_versions_cannot_silently_change(self):
        self.run_window()
        for changed in [replace(self.request, observation={'private': 'other'}),
                        replace(self.request, notebook='changed'),
                        replace(self.request, rules='changed')]:
            with self.assertRaisesRegex(JournalError, 'inputs changed'):
                self.run_window([changed])
        replacement = CountingClient(ProviderConfig('another-model'))
        with self.assertRaisesRegex(JournalError, 'configuration changed'):
            self.run_window(clients={'verwalter': replacement})
        self.assertEqual(self.client.calls, 1)

    def test_incomplete_window_manifest_cannot_be_replaced(self):
        self.run_window()
        with self.assertRaisesRegex(JournalError, 'inputs changed'):
            self.run_window([self.request, replace(self.request, player_id='B')])

    def test_crash_after_reservation_blocks_ambiguous_retry(self):
        self.journal.prepare('epoch-1', [self.request], self.clients)
        self.journal.reserve('epoch-1', self.request, self.client.config, [])
        with self.assertRaisesRegex(JournalError, 'unknown outcome'):
            self.run_window(journal=DecisionJournal(self.path))
        self.assertEqual(self.client.calls, 0)
        self.assertEqual(self.journal.summary('epoch-1')['roles']['verwalter']['pending'], 1)

    def test_partial_window_resumes_only_unreserved_requests(self):
        second = replace(self.request, player_id='B')
        requests = [self.request, second]
        self.journal.prepare('epoch-1', requests, self.clients)
        self.journal.reserve('epoch-1', self.request, self.client.config, [])
        # Emulate a completed first request followed by a crash before the second starts.
        cached = asyncio.run(WindowRunner(self.clients).run([self.request]))[0]
        self.journal.finish('epoch-1', cached)
        self.run_window(requests)
        self.assertEqual(self.client.calls, 2)

    def test_persistent_budget_survives_client_restart(self):
        first = CountingClient(ProviderConfig('mock', max_requests=1))
        self.run_window(clients={'verwalter': first})
        second = CountingClient(ProviderConfig('mock', max_requests=1))
        with self.assertRaisesRegex(JournalError, 'budget exhausted'):
            self.run_window([replace(self.request, sim_time=900)], {'verwalter': second})
        self.assertEqual(second.calls, 0)

    def test_one_exhausted_role_does_not_lose_other_inflight_results(self):
        import time
        class SlowClient(CountingClient):
            def complete(self, messages):
                time.sleep(0.05)
                return super().complete(messages)
        clients = {'verwalter': CountingClient(ProviderConfig('mock', max_requests=1)),
                   'diplomat': SlowClient()}
        self.run_window(clients=clients)
        requests = [replace(self.request, sim_time=900),
                    replace(self.request, sim_time=900, role='diplomat')]
        with self.assertRaisesRegex(JournalError, 'budget exhausted'):
            self.run_window(requests, clients)
        summary = self.journal.summary('epoch-1')['roles']['diplomat']
        self.assertEqual(summary['complete'], 1)
        self.assertEqual(summary['pending'], 0)

    def test_failures_are_preserved_without_automatic_retries(self):
        first = CountingClient(error=True)
        result = self.run_window(clients={'verwalter': first})
        second = CountingClient()
        repeated = self.run_window(clients={'verwalter': second})
        self.assertEqual(repeated, result)
        self.assertEqual(second.calls, 0)
        self.assertEqual(self.journal.summary('epoch-1')['roles']['verwalter']['errors'], 1)

    def test_competing_reservations_produce_one_winner(self):
        self.journal.prepare('epoch-1', [self.request], self.clients)
        barrier = threading.Barrier(2)
        def reserve():
            journal = DecisionJournal(self.path)
            barrier.wait()
            try:
                journal.reserve('epoch-1', self.request, self.client.config, [])
                return 'reserved'
            except JournalError:
                return 'blocked'
        with ThreadPoolExecutor(max_workers=2) as pool:
            futures = [pool.submit(reserve) for _ in range(2)]
            outcomes = [future.result(timeout=10) for future in futures]
        self.assertEqual(sorted(outcomes), ['blocked', 'reserved'])

    def test_journal_contains_actual_prompt_and_no_environment_secrets(self):
        self.run_window()
        with self.journal.connection() as db:
            row = db.execute('SELECT messages_json FROM calls').fetchone()
            messages = json.loads(row['messages_json'])
            self.assertIn('Deine Rolle: verwalter', messages[0]['content'])
            self.assertEqual(json.loads(messages[1]['content'])['lage'], {'private': 'A'})

    def test_offline_export_filters_run_and_keeps_pending_records(self):
        self.run_window()
        other = replace(self.request, player_id='unrelated-secret')
        self.journal.prepare('other-run', [other], self.clients)
        self.journal.reserve('other-run', other, self.client.config, [])
        pending = replace(self.request, sim_time=900)
        self.journal.prepare('epoch-1', [pending], self.clients)
        self.journal.reserve('epoch-1', pending, self.client.config, [])
        output = Path(self.temp.name) / 'export.jsonl'
        summary = export(self.path, 'epoch-1', output)
        text = output.read_text(encoding='utf-8')
        self.assertNotIn('unrelated-secret', text)
        self.assertEqual(summary['complete_calls'], 1)
        self.assertEqual(summary['pending_calls'], 1)
        self.assertEqual(json.loads(text.splitlines()[-1])['type'], 'export_complete')
        self.assertEqual(self.client.calls, 1)

    def test_unknown_export_run_does_not_create_output(self):
        output = Path(self.temp.name) / 'missing.jsonl'
        with self.assertRaisesRegex(ValueError, 'not found'):
            export(self.path, 'missing', output)
        self.assertFalse(output.exists())

    def test_export_never_overwrites_existing_file(self):
        self.run_window()
        output = Path(self.temp.name) / 'existing.jsonl'
        output.write_text('original', encoding='utf-8')
        with self.assertRaises(FileExistsError):
            export(self.path, 'epoch-1', output)
        self.assertEqual(output.read_text(), 'original')


if __name__ == '__main__':
    unittest.main()
