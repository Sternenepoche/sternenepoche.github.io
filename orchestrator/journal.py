"""Durable at-most-once inference journal; never retries an ambiguous call.

The journal records model decisions, not whether the game engine applied them.
SQLite transactions arbitrate reservations even between multiple processes.
"""
from __future__ import annotations

from contextlib import contextmanager
from dataclasses import asdict
import hashlib
import json
from pathlib import Path
import sqlite3

from .providers import (
    PROMPT_VERSION, ChatClient, Completion, DecisionRequest, DecisionResult, ProviderConfig,
)


class JournalError(RuntimeError):
    pass


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, allow_nan=False, separators=(',', ':'))


def digest(value):
    return hashlib.sha256(canonical(value).encode('utf-8')).hexdigest()


def decode_result(raw):
    value = json.loads(raw)
    if value['completion'] is not None:
        value['completion'] = Completion(**value['completion'])
    return DecisionResult(**value)


class DecisionJournal:
    def __init__(self, path: str | Path):
        self.path = Path(path)
        with self.connection() as db:
            db.executescript('''
                CREATE TABLE IF NOT EXISTS runs (
                    run_id TEXT PRIMARY KEY, config_hash TEXT NOT NULL,
                    config_json TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS windows (
                    run_id TEXT NOT NULL, sim_time INTEGER NOT NULL,
                    manifest_hash TEXT NOT NULL, manifest_json TEXT NOT NULL,
                    PRIMARY KEY (run_id, sim_time)
                );
                CREATE TABLE IF NOT EXISTS calls (
                    run_id TEXT NOT NULL, sim_time INTEGER NOT NULL,
                    player_id TEXT NOT NULL, role TEXT NOT NULL,
                    request_hash TEXT NOT NULL, request_json TEXT NOT NULL,
                    messages_json TEXT NOT NULL,
                    status TEXT NOT NULL CHECK (status IN ('pending', 'complete')),
                    result_json TEXT,
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    finished_at TEXT,
                    PRIMARY KEY (run_id, sim_time, player_id, role)
                );
            ''')

    @contextmanager
    def connection(self):
        db = sqlite3.connect(self.path, timeout=30)
        db.row_factory = sqlite3.Row
        try:
            # FULL sync and an explicit transaction persist reservations before I/O.
            db.execute('PRAGMA synchronous=FULL')
            yield db
        finally:
            db.close()

    def prepare(self, run_id: str, requests: list[DecisionRequest], clients: dict[str, ChatClient]):
        if not isinstance(run_id, str) or not run_id.strip():
            raise ValueError('persistent inference requires a nonempty run_id')
        configs = {'prompt_version': PROMPT_VERSION,
                   'roles': {role: asdict(client.config) for role, client in sorted(clients.items())}}
        manifest = [asdict(r) for r in sorted(requests, key=lambda r: (r.player_id, r.role))]
        if not manifest:
            return
        stamp = requests[0].sim_time
        with self.connection() as db, db:
            db.execute('BEGIN IMMEDIATE')
            known = db.execute('SELECT config_hash FROM runs WHERE run_id=?', (run_id,)).fetchone()
            if known and known['config_hash'] != digest(configs):
                raise JournalError('run configuration changed; use a distinct run_id')
            db.execute('INSERT OR IGNORE INTO runs VALUES (?, ?, ?)',
                       (run_id, digest(configs), canonical(configs)))
            known = db.execute('SELECT manifest_hash FROM windows WHERE run_id=? AND sim_time=?',
                               (run_id, stamp)).fetchone()
            if known and known['manifest_hash'] != digest(manifest):
                raise JournalError('window inputs changed; refusing to reuse a decision identity')
            db.execute('INSERT OR IGNORE INTO windows VALUES (?, ?, ?, ?)',
                       (run_id, stamp, digest(manifest), canonical(manifest)))
            pending = db.execute("SELECT COUNT(*) FROM calls WHERE run_id=? AND status='pending'",
                                 (run_id,)).fetchone()[0]
            if pending:
                raise JournalError('run has pending calls with unknown outcome; automatic retry is blocked')

    def reserve(self, run_id: str, request: DecisionRequest, config: ProviderConfig,
                messages: list[dict[str, str]]) -> DecisionResult | None:
        key = (run_id, request.sim_time, request.player_id, request.role)
        request_hash = digest(asdict(request))
        with self.connection() as db, db:
            db.execute('BEGIN IMMEDIATE')
            row = db.execute('SELECT * FROM calls WHERE run_id=? AND sim_time=? AND player_id=? AND role=?',
                             key).fetchone()
            if row:
                if row['request_hash'] != request_hash:
                    raise JournalError('request inputs changed')
                if row['status'] == 'pending':
                    raise JournalError('request already pending; no duplicate inference sent')
                return decode_result(row['result_json'])
            count = db.execute('SELECT COUNT(*) FROM calls WHERE run_id=? AND role=?',
                               (run_id, request.role)).fetchone()[0]
            if count >= config.max_requests:
                raise JournalError('persistent request budget exhausted')
            db.execute('''INSERT INTO calls
                (run_id,sim_time,player_id,role,request_hash,request_json,messages_json,status)
                VALUES (?,?,?,?,?,?,?,'pending')''',
                (*key, request_hash, canonical(asdict(request)), canonical(messages)))
        return None

    def finish(self, run_id: str, result: DecisionResult):
        with self.connection() as db, db:
            db.execute('BEGIN IMMEDIATE')
            changed = db.execute('''UPDATE calls SET status='complete', result_json=?,
                finished_at=CURRENT_TIMESTAMP
                WHERE run_id=? AND sim_time=? AND player_id=? AND role=? AND status='pending' ''',
                (canonical(asdict(result)), run_id, result.sim_time, result.player_id, result.role)).rowcount
            if changed != 1:
                raise JournalError('cannot finish an unreserved or already completed call')

    def summary(self, run_id: str) -> dict:
        with self.connection() as db:
            rows = db.execute('SELECT role,status,result_json FROM calls WHERE run_id=?', (run_id,)).fetchall()
        roles = {}
        for row in rows:
            role = roles.setdefault(row['role'], {'reserved': 0, 'pending': 0, 'complete': 0,
                                                  'errors': 0, 'reported_total_tokens': 0})
            role['reserved'] += 1
            role[row['status']] += 1
            if row['result_json']:
                result = json.loads(row['result_json'])
                role['errors'] += int(result['error'] is not None)
                completion = result.get('completion') or {}
                usage = completion.get('usage') or {}
                tokens = usage.get('total_tokens') if isinstance(usage, dict) else None
                if type(tokens) is int and tokens >= 0:
                    role['reported_total_tokens'] += tokens
        return {'run_id': run_id, 'roles': roles}
