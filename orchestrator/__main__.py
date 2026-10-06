"""Validate configuration offline or execute one engine-supplied window."""
import argparse
import asyncio
from dataclasses import asdict
import json
from pathlib import Path

from .providers import DecisionRequest, WindowRunner, load_clients, strict_json
from .journal import DecisionJournal, JournalError


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--config', type=Path, required=True)
    parser.add_argument('--window', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--execute', action='store_true', help='Send actual model requests')
    parser.add_argument('--concurrency', type=int, default=8)
    parser.add_argument('--journal', type=Path, help='SQLite journal for persistent budgets and safe resume')
    parser.add_argument('--run-id', help='Stable identity of this epoch')
    args = parser.parse_args()
    if bool(args.journal) != bool(args.run_id):
        parser.error('--journal and --run-id must be used together')
    clients = load_clients(args.config)
    runner = WindowRunner(clients, args.concurrency)
    requests = []
    if args.window:
        raw = strict_json(args.window.read_text(encoding='utf-8'))
        requests = [DecisionRequest(**item) for item in raw]
        if any(r.role not in clients for r in requests):
            parser.error('window contains an unconfigured role')
        if len({(r.player_id, r.role) for r in requests}) != len(requests):
            parser.error('duplicate player/role pair')
        if len({r.sim_time for r in requests}) > 1:
            parser.error('window contains multiple timestamps')
    if not args.execute:
        print(f'Configuration valid: {len(clients)} roles, {len(requests)} requests. No inference sent.')
        return 0
    if not requests or not args.output:
        parser.error('--execute requires a nonempty --window and a new --output file')
    # Refuse overwrite, and establish a durable input record before any inference.
    with args.output.open('x', encoding='utf-8') as archive:
        archive.write(json.dumps({'type': 'window_input', 'requests': [asdict(r) for r in requests]},
                                 ensure_ascii=False, allow_nan=False) + '\n')
        archive.flush()
        def record(result):
            archive.write(json.dumps({'type': 'decision', **asdict(result)},
                                     ensure_ascii=False, allow_nan=False) + '\n')
            archive.flush()
        journal = DecisionJournal(args.journal) if args.journal else None
        try:
            results = asyncio.run(runner.run(requests, on_result=record,
                                            journal=journal, run_id=args.run_id or ''))
        except JournalError as exc:
            archive.write(json.dumps({'type': 'window_blocked', 'error': str(exc)}) + '\n')
            print(f'Window blocked: {exc}')
            return 2
        archive.write(json.dumps({'type': 'window_complete',
                                 'errors': sum(r.error is not None for r in results)}) + '\n')
    errors = sum(r.error is not None for r in results)
    print(f'{len(results)} results archived; {errors} errors. No engine actions applied.')
    return int(errors > 0)


if __name__ == '__main__':
    raise SystemExit(main())
