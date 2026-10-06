"""Inspect/export the inference journal without contacting any model provider."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import sqlite3


def read_records(path: str | Path, run_id: str):
    path = Path(path).resolve()
    # Read-only URI: a typo must not create a fresh empty database.
    db = sqlite3.connect(path.as_uri() + '?mode=ro', uri=True)
    db.row_factory = sqlite3.Row
    try:
        db.execute('BEGIN')
        run = db.execute('SELECT config_json FROM runs WHERE run_id=?', (run_id,)).fetchone()
        if run is None:
            raise ValueError('run_id not found')
        yield {'type': 'run', 'run_id': run_id, 'configuration': json.loads(run['config_json'])}
        for row in db.execute('''SELECT * FROM calls WHERE run_id=?
                                 ORDER BY sim_time, player_id, role''', (run_id,)):
            yield {
                'type': 'inference', 'run_id': run_id, 'status': row['status'],
                'request': json.loads(row['request_json']),
                'messages': json.loads(row['messages_json']),
                'result': json.loads(row['result_json']) if row['result_json'] else None,
                'created_at_utc': row['created_at'], 'finished_at_utc': row['finished_at'],
            }
    finally:
        db.close()


def export(path: str | Path, run_id: str, output: str | Path):
    records = read_records(path, run_id)
    try:
        first = next(records)  # Validate run before creating an output file.
        with Path(output).open('x', encoding='utf-8') as handle:
            handle.write(json.dumps(first, ensure_ascii=False, allow_nan=False) + '\n')
            complete, pending, errors = 0, 0, 0
            for record in records:
                handle.write(json.dumps(record, ensure_ascii=False, allow_nan=False) + '\n')
                if record['status'] == 'pending':
                    pending += 1
                else:
                    complete += 1
                    errors += int(record['result']['error'] is not None)
            summary = {'type': 'export_complete', 'complete_calls': complete,
                       'pending_calls': pending, 'failed_calls': errors}
            handle.write(json.dumps(summary) + '\n')
        return summary
    finally:
        records.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--journal', type=Path, required=True)
    parser.add_argument('--run-id', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    try:
        print(json.dumps(export(args.journal, args.run_id, args.output)))
    except (OSError, sqlite3.Error, ValueError) as exc:
        parser.exit(2, f'Export failed: {exc}\n')


if __name__ == '__main__':
    main()
