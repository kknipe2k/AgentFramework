#!/usr/bin/env python3
"""
ARIA Decision Summary Generator
Generates text-based decision summaries for reports.

Usage: python .aria/scripts/generate-summary.py [--format text|json|markdown]

Outputs decision summary including:
- Key decisions (ranked by confidence)
- HITL checkpoint count
- Session metrics
"""

import json
import os
import sys
from collections import defaultdict
from datetime import datetime
from pathlib import Path

# Configuration
ARIA_DIR = Path('.aria')
STATE_DIR = ARIA_DIR / 'state'
LOGS_DIR = ARIA_DIR / 'logs'
SIGNALS_FILE = STATE_DIR / 'signals.jsonl'
DECISIONS_FILE = STATE_DIR / 'decisions.jsonl'
USAGE_FILE = LOGS_DIR / 'token_usage.json'


def load_jsonl(filepath: Path) -> list:
    """Load JSONL file into list of dicts."""
    if not filepath.exists():
        return []

    items = []
    with open(filepath) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                items.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return items


def load_json(filepath: Path) -> dict:
    """Load JSON file."""
    if not filepath.exists():
        return {}
    try:
        with open(filepath) as f:
            return json.load(f)
    except (json.JSONDecodeError, FileNotFoundError):
        return {}


def get_hitl_count(signals: list) -> dict:
    """Count HITL interactions from signals."""
    hitl_stats = {
        'total': 0,
        'requests': 0,
        'responses': 0,
        'timeouts': 0
    }

    for signal in signals:
        event = signal.get('event', '')
        ctx_type = signal.get('context_type', '')

        if ctx_type == 'hitl' or 'hitl' in event.lower():
            hitl_stats['total'] += 1

            if 'request' in event.lower() or event == 'hitl_request_created':
                hitl_stats['requests'] += 1
            elif 'response' in event.lower() or event == 'hitl_response_received':
                hitl_stats['responses'] += 1
            elif 'timeout' in event.lower():
                hitl_stats['timeouts'] += 1

    return hitl_stats


def get_session_stats(signals: list) -> dict:
    """Get session statistics from signals."""
    stats = {
        'session_id': None,
        'duration_seconds': 0,
        'mode': 'unknown',
        'workflow': 'unknown',
        'skills_loaded': [],
        'tools_used': defaultdict(int)
    }

    for signal in signals:
        event = signal.get('event', '')

        # Session events
        if event == 'session_started':
            stats['session_id'] = signal.get('session_id')
            details = signal.get('details', '')
            if 'mode:' in details:
                mode_match = details.split('mode:')[1].split(',')[0]
                stats['mode'] = mode_match
            if 'workflow:' in details:
                workflow_match = details.split('workflow:')[1].split(',')[0]
                stats['workflow'] = workflow_match

        elif event == 'session_ended':
            metrics = signal.get('metrics', {})
            if isinstance(metrics, str):
                try:
                    metrics = json.loads(metrics)
                except:
                    metrics = {}
            stats['duration_seconds'] = metrics.get('duration_seconds', 0)

        # Skills loaded
        elif event == 'skill_loaded':
            skill = signal.get('skill_name') or signal.get('context_name')
            if skill and skill not in stats['skills_loaded']:
                stats['skills_loaded'].append(skill)

        # Tools used
        tool = signal.get('tool')
        if tool:
            stats['tools_used'][tool] += 1

    return stats


def rank_decisions(decisions: list, top_n: int = 5) -> list:
    """Rank decisions by confidence and return top N."""
    # Filter decisions with confidence scores
    scored = [d for d in decisions if d.get('confidence')]

    # Sort by confidence (descending)
    sorted_decisions = sorted(
        scored,
        key=lambda x: float(x.get('confidence', 0)),
        reverse=True
    )

    return sorted_decisions[:top_n]


def calculate_decision_stats(decisions: list) -> dict:
    """Calculate aggregate decision statistics."""
    if not decisions:
        return {
            'total': 0,
            'avg_confidence': 0,
            'verified': 0,
            'unverified': 0,
            'pending': 0,
            'high_confidence': 0,
            'medium_confidence': 0,
            'low_confidence': 0
        }

    confidences = [float(d.get('confidence', 0)) for d in decisions if d.get('confidence')]

    stats = {
        'total': len(decisions),
        'avg_confidence': round(sum(confidences) / len(confidences), 2) if confidences else 0,
        'verified': sum(1 for d in decisions if d.get('verified') == True or d.get('verified') == 1),
        'unverified': sum(1 for d in decisions if d.get('verified') == False or d.get('verified') == 0),
        'pending': sum(1 for d in decisions if d.get('verified') is None),
        'high_confidence': sum(1 for c in confidences if c >= 0.8),
        'medium_confidence': sum(1 for c in confidences if 0.5 <= c < 0.8),
        'low_confidence': sum(1 for c in confidences if c < 0.5)
    }

    return stats


def generate_summary() -> dict:
    """Generate comprehensive summary from all data sources."""
    # Load data
    signals = load_jsonl(SIGNALS_FILE)
    decisions = load_jsonl(DECISIONS_FILE)
    usage = load_json(USAGE_FILE)

    # Build summary
    summary = {
        'generated_at': datetime.utcnow().isoformat() + 'Z',
        'session': get_session_stats(signals),
        'hitl': get_hitl_count(signals),
        'decisions': {
            'stats': calculate_decision_stats(decisions),
            'key_decisions': rank_decisions(decisions, top_n=5)
        },
        'usage': {
            'total_cost': usage.get('total_cost', 0),
            'total_input_tokens': usage.get('total_input_tokens', 0),
            'total_output_tokens': usage.get('total_output_tokens', 0),
            'by_model': usage.get('by_model', {})
        },
        'signals_count': len(signals)
    }

    return summary


def format_as_text(summary: dict) -> str:
    """Format summary as human-readable text."""
    lines = []
    lines.append("=" * 60)
    lines.append("           AGENT DECISION SUMMARY")
    lines.append("=" * 60)
    lines.append("")

    # Session info
    session = summary.get('session', {})
    lines.append(f"Session: {session.get('session_id', 'N/A')}")
    lines.append(f"Mode: {session.get('mode', 'N/A')}")
    lines.append(f"Workflow: {session.get('workflow', 'N/A')}")

    duration = session.get('duration_seconds', 0)
    if duration:
        mins = duration // 60
        secs = duration % 60
        lines.append(f"Duration: {mins}m {secs}s")
    lines.append("")

    # Decision stats
    lines.append("-" * 40)
    lines.append("DECISION TRACE")
    lines.append("-" * 40)

    dec_stats = summary.get('decisions', {}).get('stats', {})
    lines.append(f"Total decisions: {dec_stats.get('total', 0)}")
    lines.append(f"Average confidence: {dec_stats.get('avg_confidence', 0):.2f}")
    lines.append(f"Verified: {dec_stats.get('verified', 0)}")
    lines.append(f"Pending verification: {dec_stats.get('pending', 0)}")
    lines.append("")
    lines.append(f"High confidence (>=0.8): {dec_stats.get('high_confidence', 0)}")
    lines.append(f"Medium confidence (0.5-0.8): {dec_stats.get('medium_confidence', 0)}")
    lines.append(f"Low confidence (<0.5): {dec_stats.get('low_confidence', 0)}")
    lines.append("")

    # HITL stats
    lines.append("-" * 40)
    lines.append("HUMAN INTERVENTION (HITL)")
    lines.append("-" * 40)

    hitl = summary.get('hitl', {})
    lines.append(f"Total HITL checkpoints: {hitl.get('total', 0)}")
    lines.append(f"Requests created: {hitl.get('requests', 0)}")
    lines.append(f"Responses received: {hitl.get('responses', 0)}")
    if hitl.get('timeouts', 0) > 0:
        lines.append(f"Timeouts: {hitl.get('timeouts', 0)}")
    lines.append("")

    # Key decisions
    lines.append("-" * 40)
    lines.append("KEY DECISIONS (by confidence)")
    lines.append("-" * 40)

    key_decisions = summary.get('decisions', {}).get('key_decisions', [])
    if key_decisions:
        for i, dec in enumerate(key_decisions, 1):
            conf = float(dec.get('confidence', 0))
            action = dec.get('action', 'Unknown action')[:60]
            lines.append(f"{i}. [{conf:.2f}] {action}")
            if dec.get('rationale'):
                rationale = dec.get('rationale', '')[:80]
                lines.append(f"   Rationale: {rationale}")
    else:
        lines.append("No decisions recorded")
    lines.append("")

    # Skills used
    lines.append("-" * 40)
    lines.append("SKILLS LOADED")
    lines.append("-" * 40)

    skills = session.get('skills_loaded', [])
    if skills:
        for skill in skills:
            lines.append(f"  - {skill}")
    else:
        lines.append("No skills recorded")
    lines.append("")

    # Token usage
    usage = summary.get('usage', {})
    if usage.get('total_cost', 0) > 0:
        lines.append("-" * 40)
        lines.append("TOKEN USAGE")
        lines.append("-" * 40)
        lines.append(f"Total cost: ${usage.get('total_cost', 0):.4f}")
        lines.append(f"Input tokens: {usage.get('total_input_tokens', 0):,}")
        lines.append(f"Output tokens: {usage.get('total_output_tokens', 0):,}")
        lines.append("")

    lines.append("=" * 60)

    return "\n".join(lines)


def format_as_markdown(summary: dict) -> str:
    """Format summary as markdown for reports."""
    lines = []

    # Header
    lines.append("## Agent Decision Summary")
    lines.append("")

    # Session info
    session = summary.get('session', {})
    lines.append(f"**Session:** {session.get('session_id', 'N/A')}")
    lines.append(f"**Mode:** {session.get('mode', 'N/A')}")
    lines.append(f"**Workflow:** {session.get('workflow', 'N/A')}")

    duration = session.get('duration_seconds', 0)
    if duration:
        mins = duration // 60
        lines.append(f"**Duration:** {mins} minutes")
    lines.append("")

    # Decision trace table
    lines.append("### Decision Trace")
    lines.append("")

    dec_stats = summary.get('decisions', {}).get('stats', {})
    lines.append("| Metric | Value |")
    lines.append("|--------|-------|")
    lines.append(f"| Total decisions | {dec_stats.get('total', 0)} |")
    lines.append(f"| Average confidence | {dec_stats.get('avg_confidence', 0):.2f} |")
    lines.append(f"| Verified | {dec_stats.get('verified', 0)} |")
    lines.append(f"| High confidence (>=0.8) | {dec_stats.get('high_confidence', 0)} |")
    lines.append("")

    # HITL
    hitl = summary.get('hitl', {})
    lines.append("### Human Intervention (HITL)")
    lines.append("")
    lines.append(f"- **Total checkpoints:** {hitl.get('total', 0)}")
    lines.append(f"- **Requests:** {hitl.get('requests', 0)}")
    lines.append(f"- **Responses:** {hitl.get('responses', 0)}")
    lines.append("")

    # Key decisions
    lines.append("### Key Decisions")
    lines.append("")

    key_decisions = summary.get('decisions', {}).get('key_decisions', [])
    if key_decisions:
        for i, dec in enumerate(key_decisions, 1):
            conf = float(dec.get('confidence', 0))
            action = dec.get('action', 'Unknown action')
            lines.append(f"{i}. **{action}** ({conf:.2f})")
            if dec.get('rationale'):
                lines.append(f"   - *{dec.get('rationale')}*")
        lines.append("")
    else:
        lines.append("*No decisions recorded*")
        lines.append("")

    # Skills
    skills = session.get('skills_loaded', [])
    if skills:
        lines.append("### Skills Loaded")
        lines.append("")
        for skill in skills:
            lines.append(f"- {skill}")
        lines.append("")

    return "\n".join(lines)


def main():
    """Main entry point."""
    import argparse
    parser = argparse.ArgumentParser(description='Generate ARIA decision summary')
    parser.add_argument('--format', choices=['text', 'json', 'markdown'],
                       default='text', help='Output format')
    parser.add_argument('--output', '-o', help='Output file (default: stdout)')
    args = parser.parse_args()

    # Generate summary
    summary = generate_summary()

    # Format output
    if args.format == 'json':
        output = json.dumps(summary, indent=2, default=str)
    elif args.format == 'markdown':
        output = format_as_markdown(summary)
    else:
        output = format_as_text(summary)

    # Write output
    if args.output:
        with open(args.output, 'w') as f:
            f.write(output)
        print(f"Summary written to {args.output}")
    else:
        print(output)


if __name__ == '__main__':
    main()
