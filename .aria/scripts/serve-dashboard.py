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
from typing import Any, Optional
import glob as glob_module

# Configuration
PORT = int(os.environ.get('ARIA_DASHBOARD_PORT', 8420))
ARIA_DIR = Path('.aria')
STATE_DIR = ARIA_DIR / 'state'
DASHBOARD_DIR = ARIA_DIR / 'dashboard'
DB_PATH = STATE_DIR / 'traces.db'
SIGNALS_FILE = STATE_DIR / 'signals.jsonl'
DECISIONS_FILE = STATE_DIR / 'decisions.jsonl'
PROGRESS_FILE = STATE_DIR / 'progress.json'

# Metrics files
LOGS_DIR = ARIA_DIR / 'logs'
TOKEN_USAGE_FILE = LOGS_DIR / 'token_usage.json'
MODEL_LEARNING_FILE = LOGS_DIR / 'model_learning.json'

# Claude Code native logs (for token/cost data)
CLAUDE_PROJECTS_DIR = Path.home() / '.claude' / 'projects'


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

    # Events table (unified: signals, decisions, hitl, commits, tasks)
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
            context_type TEXT,
            context_name TEXT,
            task_id TEXT,
            task_status TEXT,
            raw_data TEXT,
            FOREIGN KEY (session_id) REFERENCES sessions(id)
        )
    ''')

    # Indexes for common queries
    c.execute('CREATE INDEX IF NOT EXISTS idx_events_session ON events(session_id)')
    c.execute('CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type)')
    c.execute('CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp)')
    c.execute('CREATE INDEX IF NOT EXISTS idx_events_context ON events(context_type)')

    conn.commit()
    return conn


def get_claude_session_log() -> Optional[Path]:
    """Find the most recent Claude Code session log for this project."""
    try:
        # Get current working directory name for project folder
        cwd = Path.cwd()
        # Claude uses c--project-name format
        project_slug = f"c--{cwd.name}"
        project_dir = CLAUDE_PROJECTS_DIR / project_slug

        if not project_dir.exists():
            return None

        # Find most recent JSONL file
        jsonl_files = list(project_dir.glob('*.jsonl'))
        if not jsonl_files:
            return None

        # Sort by modification time, most recent first
        return max(jsonl_files, key=lambda p: p.stat().st_mtime)
    except Exception:
        return None


def parse_claude_log_for_metrics(log_path: Path) -> dict:
    """Parse Claude's native log file for token/cost metrics."""
    metrics = {
        'total_input_tokens': 0,
        'total_output_tokens': 0,
        'cache_read_tokens': 0,
        'cache_write_tokens': 0,
        'total_cost': 0.0,
        'by_model': {},
        'tool_calls': [],
        'session_start': None,
        'session_end': None
    }

    # Pricing per 1M tokens (approximate)
    pricing = {
        'claude-opus-4-5-20251101': {'input': 15.0, 'output': 75.0, 'cache_read': 1.5, 'cache_write': 18.75},
        'claude-sonnet-4-20250514': {'input': 3.0, 'output': 15.0, 'cache_read': 0.3, 'cache_write': 3.75},
        'claude-3-5-haiku-latest': {'input': 0.80, 'output': 4.0, 'cache_read': 0.08, 'cache_write': 1.0}
    }

    try:
        with open(log_path, 'r', encoding='utf-8') as f:
            for line in f:
                if not line.strip():
                    continue
                try:
                    entry = json.loads(line)

                    # Get timestamps
                    ts = entry.get('timestamp')
                    if ts:
                        if not metrics['session_start'] or ts < metrics['session_start']:
                            metrics['session_start'] = ts
                        if not metrics['session_end'] or ts > metrics['session_end']:
                            metrics['session_end'] = ts

                    # Extract token usage from assistant messages
                    msg = entry.get('message', {})
                    if entry.get('type') == 'assistant' and msg.get('usage'):
                        usage = msg['usage']
                        model = msg.get('model', 'unknown')

                        input_tokens = usage.get('input_tokens', 0)
                        output_tokens = usage.get('output_tokens', 0)
                        cache_read = usage.get('cache_read_input_tokens', 0)
                        cache_write = usage.get('cache_creation_input_tokens', 0)

                        # Also check nested cache_creation
                        if usage.get('cache_creation'):
                            cache_write = usage['cache_creation'].get('ephemeral_5m_input_tokens', 0)

                        metrics['total_input_tokens'] += input_tokens
                        metrics['total_output_tokens'] += output_tokens
                        metrics['cache_read_tokens'] += cache_read
                        metrics['cache_write_tokens'] += cache_write

                        # Track by model
                        if model not in metrics['by_model']:
                            metrics['by_model'][model] = {
                                'input_tokens': 0, 'output_tokens': 0,
                                'cache_read': 0, 'cache_write': 0,
                                'cost': 0.0, 'calls': 0
                            }

                        m = metrics['by_model'][model]
                        m['input_tokens'] += input_tokens
                        m['output_tokens'] += output_tokens
                        m['cache_read'] += cache_read
                        m['cache_write'] += cache_write
                        m['calls'] += 1

                        # Calculate cost for this call
                        rates = pricing.get(model, pricing['claude-sonnet-4-20250514'])
                        cost = (
                            (input_tokens / 1_000_000) * rates['input'] +
                            (output_tokens / 1_000_000) * rates['output'] +
                            (cache_read / 1_000_000) * rates['cache_read'] +
                            (cache_write / 1_000_000) * rates['cache_write']
                        )
                        m['cost'] += cost
                        metrics['total_cost'] += cost

                    # Extract tool calls
                    if msg.get('content'):
                        for content in msg['content']:
                            if content.get('type') == 'tool_use':
                                metrics['tool_calls'].append({
                                    'tool': content.get('name'),
                                    'timestamp': ts,
                                    'input_preview': str(content.get('input', {}))[:200]
                                })

                except json.JSONDecodeError:
                    continue

    except Exception as e:
        metrics['error'] = str(e)

    return metrics


def sync_jsonl_to_db(conn):
    """Sync JSONL files to sqlite database."""
    c = conn.cursor()

    # Get or create current session
    session_id = get_current_session_id()
    c.execute('INSERT OR IGNORE INTO sessions (id, start_time) VALUES (?, ?)',
              (session_id, datetime.utcnow().isoformat() + 'Z'))

    # Sync signals (v2 format with nested objects)
    if SIGNALS_FILE.exists():
        existing_ids = set(row[0] for row in c.execute(
            "SELECT id FROM events WHERE event_type LIKE 'signal_%' OR event_type IN ('skill', 'agent', 'error')"
        ).fetchall())

        with open(SIGNALS_FILE, encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    data = json.loads(line)
                    sig_id = data.get('id')
                    if sig_id in existing_ids:
                        continue

                    # Handle v2 nested format
                    sig_type = data.get('type', 'tool')
                    event = data.get('event', 'unknown')

                    # Extract tool data (may be nested in v2)
                    tool_data = data.get('tool', {})
                    tool_name = tool_data.get('name') if isinstance(tool_data, dict) else data.get('tool')
                    file_path = tool_data.get('file_path', '') if isinstance(tool_data, dict) else data.get('file_path', '')
                    command = tool_data.get('command', '') if isinstance(tool_data, dict) else data.get('command', '')

                    # Extract context (may be nested in v2)
                    ctx = data.get('context', {})
                    context_type = ctx.get('type', '') if isinstance(ctx, dict) else data.get('context_type', '')
                    context_name = ctx.get('name', '') if isinstance(ctx, dict) else data.get('context_name', '')
                    context_detail = ctx.get('detail', '') if isinstance(ctx, dict) else ''

                    # Determine event type
                    if sig_type == 'skill':
                        event_type = 'skill'
                    elif sig_type == 'agent':
                        event_type = 'agent'
                    elif sig_type == 'error':
                        event_type = 'error'
                    else:
                        event_type = f"signal_{event}"

                    c.execute('''
                        INSERT OR IGNORE INTO events
                        (id, session_id, timestamp, event_type, tool, file_path, command,
                         context_type, context_name, raw_data)
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ''', (
                        sig_id,
                        session_id,
                        data.get('timestamp'),
                        event_type,
                        tool_name,
                        file_path,
                        command,
                        context_type,
                        context_name or context_detail,
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

    # Sync git commits - ONLY commits made during this session
    # Get session start time to filter commits
    session_start = None
    c.execute('SELECT start_time FROM sessions WHERE id = ?', (session_id,))
    row = c.fetchone()
    if row and row[0]:
        session_start = row[0]

    try:
        # Use --since to only get commits from session start
        git_cmd = ['git', 'log', '--oneline', '-20', '--format=%H|%aI|%s']
        if session_start:
            git_cmd.extend(['--since', session_start])

        result = subprocess.run(
            git_cmd,
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


def get_lineage(conn) -> dict:
    """Get hierarchical lineage view with proper nesting.

    Hierarchy:
    SESSION
    └── SKILL (container, collapsible)
        ├── DECISION
        │   └── SIGNALS (supporting tool calls)
        ├── VERIFY (test run)
        └── HITL (checkpoint)
    └── COMMIT (with linked decisions)
    └── ORPHAN events (before any skill loaded)
    """
    c = conn.cursor()
    session_id = get_current_session_id()

    # Get all events ordered by timestamp
    c.execute('''
        SELECT * FROM events
        WHERE session_id = ?
        ORDER BY timestamp ASC
    ''', (session_id,))
    all_events = [dict(row) for row in c.fetchall()]

    # Build hierarchical structure
    lineage = {
        'session_id': session_id,
        'summary': {
            'skills': 0,
            'decisions': 0,
            'signals': 0,
            'hitl': 0,
            'commits': 0,
            'verify': 0
        },
        'tree': [],  # Nested tree structure
        'commits': [],  # Flat list of commits with linked decisions
        'flat_workflow': []  # Keep flat for timeline compatibility
    }

    # Track state as we build the tree
    current_skill = None  # Currently active skill container
    current_decision = None  # Currently active decision (for signal nesting)
    orphan_events = []  # Events before any skill loaded
    skills_seen = set()  # Track unique skills
    decision_lookup = {}  # Map decision timestamps to their data

    def create_skill_node(name, timestamp):
        return {
            'type': 'skill',
            'name': name,
            'timestamp': timestamp,
            'expanded': True,
            'children': []
        }

    def create_decision_node(event):
        return {
            'type': 'decision',
            'id': event.get('id'),
            'action': event.get('action'),
            'context': event.get('context'),
            'rationale': event.get('rationale'),
            'alternatives': event.get('alternatives'),
            'confidence': event.get('confidence'),
            'verified': event.get('verified'),
            'timestamp': event.get('timestamp'),
            'expanded': False,
            'signals': []
        }

    # Process events chronologically
    for event in all_events:
        ctx_type = event.get('context_type')
        ctx_name = event.get('context_name')
        event_type = event.get('event_type')
        timestamp = event.get('timestamp')

        # ===== SKILL LOADING =====
        if ctx_type == 'skill' and event_type == 'signal_pre':
            if ctx_name and ctx_name not in skills_seen:
                skills_seen.add(ctx_name)
                lineage['summary']['skills'] += 1

                # Create new skill container
                skill_node = create_skill_node(ctx_name, timestamp)

                # If we had orphan events, add them to root first
                if orphan_events:
                    orphan_container = {
                        'type': 'orphan',
                        'name': 'Pre-skill events',
                        'timestamp': orphan_events[0].get('timestamp'),
                        'expanded': False,
                        'children': orphan_events
                    }
                    lineage['tree'].append(orphan_container)
                    orphan_events = []

                lineage['tree'].append(skill_node)
                current_skill = skill_node
                current_decision = None  # Reset decision context

        # ===== DECISIONS =====
        elif event_type == 'decision':
            lineage['summary']['decisions'] += 1
            decision_node = create_decision_node(event)
            decision_lookup[timestamp] = decision_node

            if current_skill:
                current_skill['children'].append(decision_node)
            else:
                orphan_events.append(decision_node)

            current_decision = decision_node

        # ===== SIGNALS (nest under decisions if recent) =====
        elif event_type in ('signal_pre', 'signal_post'):
            # Skip skill reads (already tracked as containers)
            if ctx_type == 'skill':
                continue

            lineage['summary']['signals'] += 1
            signal_node = {
                'type': 'signal',
                'event': 'pre' if event_type == 'signal_pre' else 'post',
                'tool': event.get('tool'),
                'file_path': event.get('file_path'),
                'command': event.get('command'),
                'context_type': ctx_type,
                'context_name': ctx_name,
                'timestamp': timestamp
            }

            # Nest under current decision if within 30 seconds
            if current_decision and timestamp:
                try:
                    dec_time = datetime.fromisoformat(current_decision['timestamp'].replace('Z', '+00:00'))
                    sig_time = datetime.fromisoformat(timestamp.replace('Z', '+00:00'))
                    delta = abs((sig_time - dec_time).total_seconds())
                    if delta <= 30:
                        current_decision['signals'].append(signal_node)
                        continue
                except (ValueError, TypeError):
                    pass

            # Otherwise add to current skill or orphans
            if current_skill:
                current_skill['children'].append(signal_node)
            else:
                orphan_events.append(signal_node)

        # ===== HITL CHECKPOINTS =====
        elif event_type == 'hitl' or ctx_type == 'hitl':
            lineage['summary']['hitl'] += 1
            hitl_node = {
                'type': 'hitl',
                'action': event.get('hitl_action') or event.get('action'),
                'response': event.get('hitl_response'),
                'timestamp': timestamp,
                'expanded': False
            }

            if current_skill:
                current_skill['children'].append(hitl_node)
            else:
                orphan_events.append(hitl_node)

        # ===== VERIFY (test runs) =====
        elif ctx_type == 'verify':
            lineage['summary']['verify'] += 1
            verify_node = {
                'type': 'verify',
                'command': event.get('command'),
                'context_name': ctx_name,
                'timestamp': timestamp
            }

            if current_skill:
                current_skill['children'].append(verify_node)
            else:
                orphan_events.append(verify_node)

        # ===== COMMITS =====
        elif event_type == 'commit' or ctx_type == 'commit':
            lineage['summary']['commits'] += 1
            commit_hash = event.get('commit_hash', '')
            commit_node = {
                'type': 'commit',
                'hash': commit_hash[:8] if commit_hash else '',
                'full_hash': commit_hash,
                'message': event.get('commit_message', ''),
                'timestamp': timestamp,
                'linked_decisions': []  # Will be populated below
            }

            # Link decisions that led to this commit
            # (all decisions since last commit)
            lineage['commits'].append(commit_node)

            # Also add to tree structure
            if current_skill:
                current_skill['children'].append(commit_node)
            else:
                lineage['tree'].append(commit_node)

        # ===== SUBAGENTS =====
        elif ctx_type == 'subagent':
            subagent_node = {
                'type': 'subagent',
                'name': ctx_name,
                'timestamp': timestamp
            }

            if current_skill:
                current_skill['children'].append(subagent_node)
            else:
                orphan_events.append(subagent_node)

        # ===== TEMPLATES =====
        elif ctx_type == 'template':
            template_node = {
                'type': 'template',
                'name': ctx_name,
                'timestamp': timestamp
            }

            if current_skill:
                current_skill['children'].append(template_node)
            else:
                orphan_events.append(template_node)

        # Add to flat workflow for compatibility
        lineage['flat_workflow'].append({
            'type': ctx_type or event_type,
            'name': ctx_name or event.get('action'),
            'timestamp': timestamp
        })

    # Handle remaining orphans
    if orphan_events:
        if lineage['tree']:
            # Add to last skill if we have one
            lineage['tree'][-1].get('children', []).extend(orphan_events)
        else:
            # Make orphan container at root
            lineage['tree'].append({
                'type': 'orphan',
                'name': 'Session events',
                'timestamp': orphan_events[0].get('timestamp') if orphan_events else None,
                'expanded': True,
                'children': orphan_events
            })

    # Link decisions to commits (decisions between this commit and previous)
    sorted_commits = sorted(lineage['commits'], key=lambda x: x.get('timestamp', ''))
    all_decisions = sorted(decision_lookup.values(), key=lambda x: x.get('timestamp', ''))

    for i, commit in enumerate(sorted_commits):
        prev_time = sorted_commits[i-1]['timestamp'] if i > 0 else '1970-01-01'
        commit_time = commit['timestamp'] or '9999-12-31'

        commit['linked_decisions'] = [
            {'action': d['action'], 'confidence': d['confidence'], 'timestamp': d['timestamp']}
            for d in all_decisions
            if prev_time < (d.get('timestamp') or '') <= commit_time
        ]

    return lineage


def get_metrics() -> dict:
    """Get token usage and model metrics from logs.

    Priority:
    1. Claude's native JSONL logs (most accurate)
    2. ARIA token_usage.json (fallback)
    """
    metrics = {
        'token_usage': None,
        'model_learning': None,
        'session_duration': None,
        'cost_breakdown': None,
        'source': None
    }

    # Try Claude's native logs first
    claude_log = get_claude_session_log()
    if claude_log:
        try:
            claude_metrics = parse_claude_log_for_metrics(claude_log)
            if claude_metrics.get('total_input_tokens', 0) > 0:
                metrics['source'] = 'claude_native'
                metrics['token_usage'] = {
                    'total_input': claude_metrics['total_input_tokens'],
                    'total_output': claude_metrics['total_output_tokens'],
                    'cache_read': claude_metrics['cache_read_tokens'],
                    'cache_write': claude_metrics['cache_write_tokens'],
                    'total_cost': round(claude_metrics['total_cost'], 4),
                    'budget': 10.0,  # Default budget
                    'budget_remaining': 10.0 - claude_metrics['total_cost'],
                    'by_model': claude_metrics['by_model'],
                    'session_start': claude_metrics['session_start'],
                    'session_end': claude_metrics['session_end'],
                    'recent_history': claude_metrics['tool_calls'][-20:]
                }

                # Calculate session duration
                if claude_metrics.get('session_start') and claude_metrics.get('session_end'):
                    try:
                        start = datetime.fromisoformat(claude_metrics['session_start'].replace('Z', '+00:00'))
                        end = datetime.fromisoformat(claude_metrics['session_end'].replace('Z', '+00:00'))
                        duration = end - start
                        metrics['session_duration'] = {
                            'seconds': int(duration.total_seconds()),
                            'formatted': f"{int(duration.total_seconds() // 3600)}h {int((duration.total_seconds() % 3600) // 60)}m {int(duration.total_seconds() % 60)}s"
                        }
                    except:
                        pass

                # Cost breakdown
                if claude_metrics['total_cost'] > 0:
                    metrics['cost_breakdown'] = {
                        model: {
                            'cost': round(data.get('cost', 0.0), 4),
                            'percentage': round(data.get('cost', 0.0) / claude_metrics['total_cost'] * 100, 1),
                            'calls': data.get('calls', 0),
                            'tokens': data.get('input_tokens', 0) + data.get('output_tokens', 0)
                        }
                        for model, data in claude_metrics['by_model'].items()
                    }
        except Exception as e:
            metrics['claude_log_error'] = str(e)

    # Fallback to ARIA token_usage.json
    if not metrics['token_usage'] and TOKEN_USAGE_FILE.exists():
        try:
            with open(TOKEN_USAGE_FILE) as f:
                usage = json.load(f)
                metrics['source'] = 'aria_logs'
                metrics['token_usage'] = {
                    'total_input': usage.get('total_input_tokens', 0),
                    'total_output': usage.get('total_output_tokens', 0),
                    'total_cost': usage.get('total_cost', 0.0),
                    'budget': usage.get('budget', 10.0),
                    'budget_remaining': usage.get('budget', 10.0) - usage.get('total_cost', 0.0),
                    'by_model': usage.get('by_model', {}),
                    'session_start': usage.get('session_start', None),
                    'recent_history': usage.get('history', [])[-20:]
                }

                # Calculate session duration
                if usage.get('session_start'):
                    try:
                        start = datetime.fromisoformat(usage['session_start'].replace('Z', '+00:00'))
                        duration = datetime.now(start.tzinfo) - start if start.tzinfo else datetime.now() - start
                        metrics['session_duration'] = {
                            'seconds': int(duration.total_seconds()),
                            'formatted': f"{int(duration.total_seconds() // 3600)}h {int((duration.total_seconds() % 3600) // 60)}m"
                        }
                    except:
                        pass

                # Cost breakdown
                by_model = usage.get('by_model', {})
                total_cost = usage.get('total_cost', 0.0)
                if total_cost > 0:
                    metrics['cost_breakdown'] = {
                        model: {
                            'cost': data.get('cost', 0.0),
                            'percentage': round(data.get('cost', 0.0) / total_cost * 100, 1) if total_cost > 0 else 0,
                            'calls': data.get('calls', 0)
                        }
                        for model, data in by_model.items()
                    }
        except Exception as e:
            metrics['token_usage_error'] = str(e)

    # Read model learning data
    if MODEL_LEARNING_FILE.exists():
        try:
            with open(MODEL_LEARNING_FILE) as f:
                learning = json.load(f)
                metrics['model_learning'] = {
                    'total_samples': sum(
                        len(data.get('samples', []))
                        for data in learning.values()
                    ),
                    'task_types': list(learning.keys()),
                    'summary': {
                        task_type: {
                            'samples': len(data.get('samples', [])),
                            'avg_success': round(
                                sum(s.get('success', 0) for s in data.get('samples', [])) /
                                max(len(data.get('samples', [])), 1) * 100, 1
                            )
                        }
                        for task_type, data in learning.items()
                    }
                }
        except Exception as e:
            metrics['model_learning_error'] = str(e)

    return metrics


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

        # Handle favicon.ico gracefully (return empty 204)
        if path == '/favicon.ico':
            self.send_response(204)  # No Content
            self.end_headers()
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
            elif path == '/api/lineage':
                data = get_lineage(self.db_conn)
            elif path == '/api/metrics':
                data = get_metrics()
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
        # args[0] might be HTTPStatus enum on some platforms, not a string
        if args and isinstance(args[0], str) and '/api/' in args[0]:
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
+---------------------------------------------------------------+
|                    ARIA Dashboard                             |
+---------------------------------------------------------------+
|  Server running at: http://localhost:{args.port}               |
|                                                               |
|  API Endpoints:                                               |
|    /api/session   - Session summary                           |
|    /api/timeline  - Unified event timeline                    |
|    /api/decisions - Decisions with signals                    |
|    /api/commits   - Commits with decisions                    |
|    /api/lineage   - Hierarchical workflow lineage             |
|    /api/metrics   - Token usage & cost metrics                |
|                                                               |
|  Press Ctrl+C to stop                                         |
+---------------------------------------------------------------+
        ''')
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            print('\nShutting down...')
            conn.close()


if __name__ == '__main__':
    main()
