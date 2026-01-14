#!/usr/bin/env python3
"""
ARIA Dashboard Server
Serves the web dashboard and provides API endpoints for session data.

Usage: python .aria/scripts/serve-dashboard.py [--port 8420]
"""

import http.server
import json
import os
import re
import sqlite3
import subprocess
import urllib.parse
from datetime import datetime
from pathlib import Path
from typing import Any

# Configuration
PORT = int(os.environ.get('ARIA_DASHBOARD_PORT', 8420))
ARIA_DIR = Path('.aria')
STATE_DIR = ARIA_DIR / 'state'
DASHBOARD_DIR = ARIA_DIR / 'dashboard'
DB_PATH = STATE_DIR / 'traces.db'
SIGNALS_FILE = STATE_DIR / 'signals.jsonl'
DECISIONS_FILE = STATE_DIR / 'decisions.jsonl'
PROGRESS_FILE = STATE_DIR / 'progress.json'


def init_db():
    """Initialize sqlite database with schema."""
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    c = conn.cursor()

    # Sessions table
    c.execute('''
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            start_time TEXT,
            end_time TEXT,
            mode TEXT,
            tokens_in INTEGER DEFAULT 0,
            tokens_out INTEGER DEFAULT 0,
            skills_used TEXT DEFAULT '[]',
            status TEXT DEFAULT 'active'
        )
    ''')

    # Events table (unified: signals, decisions, hitl, commits)
    c.execute('''
        CREATE TABLE IF NOT EXISTS events (
            id TEXT PRIMARY KEY,
            session_id TEXT,
            timestamp TEXT,
            event_type TEXT,
            tool TEXT,
            action TEXT,
            file_path TEXT,
            command TEXT,
            context TEXT,
            rationale TEXT,
            alternatives TEXT,
            confidence REAL,
            verified INTEGER,
            commit_hash TEXT,
            commit_message TEXT,
            hitl_action TEXT,
            hitl_response TEXT,
            raw_data TEXT,
            FOREIGN KEY (session_id) REFERENCES sessions(id)
        )
    ''')

    # Indexes for common queries
    c.execute('CREATE INDEX IF NOT EXISTS idx_events_session ON events(session_id)')
    c.execute('CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type)')
    c.execute('CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp)')

    conn.commit()
    return conn


def sync_jsonl_to_db(conn):
    """Sync JSONL files to sqlite database."""
    c = conn.cursor()

    # Get or create current session
    session_id = get_current_session_id()
    c.execute('INSERT OR IGNORE INTO sessions (id, start_time) VALUES (?, ?)',
              (session_id, datetime.utcnow().isoformat() + 'Z'))

    # Sync signals
    if SIGNALS_FILE.exists():
        existing_ids = set(row[0] for row in c.execute(
            "SELECT id FROM events WHERE event_type IN ('signal_pre', 'signal_post')"
        ).fetchall())

        with open(SIGNALS_FILE) as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    data = json.loads(line)
                    if data.get('id') in existing_ids:
                        continue
                    c.execute('''
                        INSERT OR IGNORE INTO events
                        (id, session_id, timestamp, event_type, tool, file_path, command, raw_data)
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    ''', (
                        data.get('id'),
                        session_id,
                        data.get('timestamp'),
                        f"signal_{data.get('event', 'unknown')}",
                        data.get('tool'),
                        data.get('file_path'),
                        data.get('command'),
                        line
                    ))
                except json.JSONDecodeError:
                    continue

    # Sync decisions
    if DECISIONS_FILE.exists():
        existing_ids = set(row[0] for row in c.execute(
            "SELECT id FROM events WHERE event_type = 'decision'"
        ).fetchall())

        with open(DECISIONS_FILE) as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    data = json.loads(line)
                    dec_id = data.get('id', f"dec-{hash(line) & 0xFFFFFFFF}")
                    if dec_id in existing_ids:
                        continue
                    c.execute('''
                        INSERT OR IGNORE INTO events
                        (id, session_id, timestamp, event_type, action, context, rationale,
                         alternatives, confidence, verified, raw_data)
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ''', (
                        dec_id,
                        session_id,
                        data.get('timestamp'),
                        'decision',
                        data.get('action'),
                        data.get('context'),
                        data.get('rationale'),
                        data.get('alternatives'),
                        float(data.get('confidence', 0)) if data.get('confidence') else None,
                        1 if data.get('verified') == True else (0 if data.get('verified') == False else None),
                        line
                    ))
                except (json.JSONDecodeError, ValueError):
                    continue

    # Sync git commits
    try:
        result = subprocess.run(
            ['git', 'log', '--oneline', '-20', '--format=%H|%aI|%s'],
            capture_output=True, text=True, timeout=5
        )
        if result.returncode == 0:
            existing_hashes = set(row[0] for row in c.execute(
                "SELECT commit_hash FROM events WHERE event_type = 'commit' AND commit_hash IS NOT NULL"
            ).fetchall())

            for line in result.stdout.strip().split('\n'):
                if '|' not in line:
                    continue
                parts = line.split('|', 2)
                if len(parts) < 3:
                    continue
                commit_hash, timestamp, message = parts
                if commit_hash in existing_hashes:
                    continue
                c.execute('''
                    INSERT OR IGNORE INTO events
                    (id, session_id, timestamp, event_type, commit_hash, commit_message)
                    VALUES (?, ?, ?, ?, ?, ?)
                ''', (
                    f"commit-{commit_hash[:8]}",
                    session_id,
                    timestamp,
                    'commit',
                    commit_hash,
                    message
                ))
    except (subprocess.TimeoutExpired, FileNotFoundError):
        pass

    conn.commit()


def get_current_session_id():
    """Get or generate current session ID based on date."""
    return f"session-{datetime.utcnow().strftime('%Y%m%d')}"


def get_session_summary(conn) -> dict:
    """Get high-level session summary."""
    c = conn.cursor()
    session_id = get_current_session_id()

    # Counts by event type
    c.execute('''
        SELECT event_type, COUNT(*) as count
        FROM events
        WHERE session_id = ?
        GROUP BY event_type
    ''', (session_id,))
    counts = {row['event_type']: row['count'] for row in c.fetchall()}

    # Time range
    c.execute('''
        SELECT MIN(timestamp) as start, MAX(timestamp) as end
        FROM events
        WHERE session_id = ?
    ''', (session_id,))
    time_range = c.fetchone()

    # Unique tools used
    c.execute('''
        SELECT DISTINCT tool FROM events
        WHERE session_id = ? AND tool IS NOT NULL AND tool != ''
    ''', (session_id,))
    tools = [row['tool'] for row in c.fetchall()]

    # Decisions with confidence
    c.execute('''
        SELECT AVG(confidence) as avg_conf,
               SUM(CASE WHEN verified = 1 THEN 1 ELSE 0 END) as verified,
               COUNT(*) as total
        FROM events
        WHERE session_id = ? AND event_type = 'decision'
    ''', (session_id,))
    decisions = c.fetchone()

    # Files touched
    c.execute('''
        SELECT DISTINCT file_path FROM events
        WHERE session_id = ? AND file_path IS NOT NULL AND file_path != ''
    ''', (session_id,))
    files = [row['file_path'] for row in c.fetchall()]

    return {
        'session_id': session_id,
        'start_time': time_range['start'],
        'end_time': time_range['end'],
        'counts': {
            'signals': counts.get('signal_pre', 0) + counts.get('signal_post', 0),
            'decisions': counts.get('decision', 0),
            'commits': counts.get('commit', 0),
            'hitl': counts.get('hitl', 0)
        },
        'tools_used': tools,
        'files_touched': files[:20],  # Limit
        'decisions': {
            'total': decisions['total'] or 0,
            'verified': decisions['verified'] or 0,
            'avg_confidence': round(decisions['avg_conf'] or 0, 2)
        }
    }


def get_timeline(conn, limit=100) -> list:
    """Get unified timeline of all events."""
    c = conn.cursor()
    session_id = get_current_session_id()

    c.execute('''
        SELECT * FROM events
        WHERE session_id = ?
        ORDER BY timestamp DESC
        LIMIT ?
    ''', (session_id, limit))

    events = []
    for row in c.fetchall():
        event = dict(row)
        # Clean up None values
        event = {k: v for k, v in event.items() if v is not None}
        events.append(event)

    return events


def get_decisions_with_signals(conn) -> list:
    """Get decisions with their supporting signals."""
    c = conn.cursor()
    session_id = get_current_session_id()

    # Get all decisions
    c.execute('''
        SELECT * FROM events
        WHERE session_id = ? AND event_type = 'decision'
        ORDER BY timestamp DESC
    ''', (session_id,))
    decisions = [dict(row) for row in c.fetchall()]

    # For each decision, find related signals (within 60s window, matching files)
    for dec in decisions:
        if not dec.get('timestamp'):
            dec['signals'] = []
            continue

        # Find signals around this decision's timestamp
        c.execute('''
            SELECT * FROM events
            WHERE session_id = ?
            AND event_type LIKE 'signal_%'
            AND timestamp BETWEEN datetime(?, '-60 seconds') AND datetime(?, '+60 seconds')
            ORDER BY timestamp
        ''', (session_id, dec['timestamp'], dec['timestamp']))
        dec['signals'] = [dict(row) for row in c.fetchall()]

    return decisions


def get_commits_with_decisions(conn) -> list:
    """Get commits with their associated decisions."""
    c = conn.cursor()
    session_id = get_current_session_id()

    # Get all commits
    c.execute('''
        SELECT * FROM events
        WHERE session_id = ? AND event_type = 'commit'
        ORDER BY timestamp DESC
    ''', (session_id,))
    commits = [dict(row) for row in c.fetchall()]

    # For each commit, find decisions that led to it
    for i, commit in enumerate(commits):
        # Get next commit timestamp (or session start) as boundary
        prev_timestamp = commits[i + 1]['timestamp'] if i + 1 < len(commits) else '1970-01-01'

        c.execute('''
            SELECT * FROM events
            WHERE session_id = ?
            AND event_type = 'decision'
            AND timestamp > ? AND timestamp <= ?
            ORDER BY timestamp
        ''', (session_id, prev_timestamp, commit['timestamp']))
        commit['decisions'] = [dict(row) for row in c.fetchall()]

        # Count signals in same window
        c.execute('''
            SELECT tool, COUNT(*) as count FROM events
            WHERE session_id = ?
            AND event_type LIKE 'signal_%'
            AND timestamp > ? AND timestamp <= ?
            GROUP BY tool
        ''', (session_id, prev_timestamp, commit['timestamp']))
        commit['tool_counts'] = {row['tool']: row['count'] for row in c.fetchall()}

    return commits


class DashboardHandler(http.server.SimpleHTTPRequestHandler):
    """HTTP handler for dashboard and API."""

    def __init__(self, *args, db_conn=None, **kwargs):
        self.db_conn = db_conn
        super().__init__(*args, directory=str(DASHBOARD_DIR), **kwargs)

    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path

        # API endpoints
        if path.startswith('/api/'):
            self.handle_api(path)
            return

        # Serve static files
        if path == '/' or path == '/index.html':
            self.path = '/index.html'

        super().do_GET()

    def handle_api(self, path):
        """Handle API requests."""
        try:
            # Sync data before every API call (lightweight)
            sync_jsonl_to_db(self.db_conn)

            if path == '/api/session':
                data = get_session_summary(self.db_conn)
            elif path == '/api/timeline':
                data = get_timeline(self.db_conn)
            elif path == '/api/decisions':
                data = get_decisions_with_signals(self.db_conn)
            elif path == '/api/commits':
                data = get_commits_with_decisions(self.db_conn)
            else:
                self.send_error(404, 'Unknown API endpoint')
                return

            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(json.dumps(data, indent=2, default=str).encode())

        except Exception as e:
            self.send_error(500, str(e))

    def log_message(self, format, *args):
        """Suppress default logging, use custom."""
        if '/api/' in args[0]:
            print(f"[API] {args[0]}")


def main():
    """Main entry point."""
    import argparse
    parser = argparse.ArgumentParser(description='ARIA Dashboard Server')
    parser.add_argument('--port', type=int, default=PORT, help='Port to serve on')
    args = parser.parse_args()

    # Ensure directories exist
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    DASHBOARD_DIR.mkdir(parents=True, exist_ok=True)

    # Initialize database
    conn = init_db()
    sync_jsonl_to_db(conn)

    # Create handler with db connection
    def handler(*args, **kwargs):
        return DashboardHandler(*args, db_conn=conn, **kwargs)

    # Start server
    with http.server.HTTPServer(('', args.port), handler) as server:
        print(f'''
╔═══════════════════════════════════════════════════════════════╗
║                    ARIA Dashboard                             ║
╠═══════════════════════════════════════════════════════════════╣
║  Server running at: http://localhost:{args.port}               ║
║                                                               ║
║  API Endpoints:                                               ║
║    /api/session   - Session summary                           ║
║    /api/timeline  - Unified event timeline                    ║
║    /api/decisions - Decisions with signals                    ║
║    /api/commits   - Commits with decisions                    ║
║                                                               ║
║  Press Ctrl+C to stop                                         ║
╚═══════════════════════════════════════════════════════════════╝
        ''')
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            print('\nShutting down...')
            conn.close()


if __name__ == '__main__':
    main()
