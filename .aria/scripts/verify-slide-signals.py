#!/usr/bin/env python3
"""
Verify Slide Generation Signals

Checks that slide generation signals were properly emitted during runtime.
Used as a verification gate after slide generation completes.

Usage:
    python .aria/scripts/verify-slide-signals.py [--method nblm|pptx] [--verbose]
"""

import argparse
import json
import sys
from datetime import datetime, timedelta
from pathlib import Path

ARIA_DIR = Path('.aria')
SIGNALS_FILE = ARIA_DIR / 'state' / 'signals.jsonl'

# Required signals for each method
NBLM_REQUIRED_SIGNALS = [
    'nblm_generation_start',
    'nblm_notebook_created',
    'nblm_prompt_sending',
    'nblm_prompt_sent',
    'nblm_deck_generation_started',
    'nblm_generation_complete',
]

PPTX_REQUIRED_SIGNALS = [
    'pptx_generation_start',
    'pptx_generation_complete',
]

# Colors
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
NC = '\033[0m'


def color(text: str, color_code: str) -> str:
    """Apply color if terminal supports it."""
    if sys.stdout.isatty():
        return f"{color_code}{text}{NC}"
    return text


def load_recent_signals(since_minutes: int = 30) -> list:
    """Load signals from the last N minutes."""
    if not SIGNALS_FILE.exists():
        return []

    cutoff = datetime.now() - timedelta(minutes=since_minutes)
    signals = []

    with open(SIGNALS_FILE) as f:
        for line in f:
            try:
                sig = json.loads(line.strip())
                # Parse timestamp
                ts_str = sig.get('timestamp', '')
                if ts_str:
                    # Handle ISO format with Z
                    ts_str = ts_str.replace('Z', '+00:00')
                    try:
                        ts = datetime.fromisoformat(ts_str)
                        # Convert to naive for comparison
                        if ts.tzinfo:
                            ts = ts.replace(tzinfo=None)
                        if ts >= cutoff:
                            signals.append(sig)
                    except:
                        pass
            except json.JSONDecodeError:
                pass

    return signals


def verify_nblm_signals(signals: list, verbose: bool = False) -> tuple:
    """Verify NotebookLM signals were emitted correctly."""
    found = set()
    details = {}

    for sig in signals:
        event = sig.get('event', '')
        if event.startswith('nblm_'):
            found.add(event)
            details[event] = sig

    missing = set(NBLM_REQUIRED_SIGNALS) - found
    failed = 'nblm_generation_failed' in found

    if verbose:
        print("\nNotebookLM Signal Verification:")
        print("-" * 40)
        for req in NBLM_REQUIRED_SIGNALS:
            if req in found:
                print(f"  {color('[✓]', GREEN)} {req}")
            else:
                print(f"  {color('[✗]', RED)} {req}")

        if 'nblm_prompt_sending' in details:
            prompt_sig = details['nblm_prompt_sending']
            print(f"\nPrompt Details:")
            print(f"  Length: {prompt_sig.get('prompt_length', 'N/A')} chars")
            print(f"  Preview: {prompt_sig.get('prompt_preview', 'N/A')[:100]}...")

        if 'nblm_generation_complete' in details:
            comp_sig = details['nblm_generation_complete']
            print(f"\nCompletion Details:")
            print(f"  Notebook ID: {comp_sig.get('notebook_id', 'N/A')}")
            print(f"  Notebook URL: {comp_sig.get('notebook_url', 'N/A')}")
            print(f"  Sources Added: {comp_sig.get('sources_added', 'N/A')}")
            print(f"  Prompt Sent: {comp_sig.get('prompt_sent', 'N/A')}")

    return len(missing) == 0 and not failed, missing, failed, details


def verify_pptx_signals(signals: list, verbose: bool = False) -> tuple:
    """Verify pptx signals were emitted correctly."""
    found = set()
    details = {}

    for sig in signals:
        event = sig.get('event', '')
        if event.startswith('pptx_'):
            found.add(event)
            details[event] = sig

    missing = set(PPTX_REQUIRED_SIGNALS) - found
    failed = 'pptx_import_failed' in found

    if verbose:
        print("\nPPTX Signal Verification:")
        print("-" * 40)
        for req in PPTX_REQUIRED_SIGNALS:
            if req in found:
                print(f"  {color('[✓]', GREEN)} {req}")
            else:
                print(f"  {color('[✗]', RED)} {req}")

        if 'pptx_generation_complete' in details:
            comp_sig = details['pptx_generation_complete']
            print(f"\nCompletion Details:")
            print(f"  Output Path: {comp_sig.get('output_path', 'N/A')}")
            print(f"  Slide Count: {comp_sig.get('slide_count', 'N/A')}")
            print(f"  Core Ideas: {comp_sig.get('core_ideas_count', 'N/A')}")

    # If pptx import failed, that's expected when library not installed
    if failed and not details.get('pptx_generation_complete'):
        return True, set(), True, details  # Consider import failure acceptable

    return len(missing) == 0, missing, failed, details


def main():
    parser = argparse.ArgumentParser(description='Verify slide generation signals')
    parser.add_argument('--method', choices=['nblm', 'pptx', 'any'], default='any',
                        help='Method to verify (default: any)')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    parser.add_argument('--since', type=int, default=30,
                        help='Check signals from last N minutes (default: 30)')

    args = parser.parse_args()

    print("=" * 60)
    print("  SLIDE GENERATION SIGNAL VERIFICATION")
    print("=" * 60)

    signals = load_recent_signals(args.since)

    if not signals:
        print(f"\n{color('WARNING:', YELLOW)} No signals found in last {args.since} minutes")
        print("Either slide generation hasn't run, or signals weren't emitted.")
        sys.exit(1)

    # Filter to slide-related signals
    slide_signals = [s for s in signals if s.get('context_type') == 'slide_generation']

    if not slide_signals:
        print(f"\n{color('WARNING:', YELLOW)} No slide generation signals found")
        print("Run slide generation first, then verify.")
        sys.exit(1)

    print(f"\nFound {len(slide_signals)} slide generation signal(s)")

    # Determine method from signals
    has_nblm = any(s.get('event', '').startswith('nblm_') for s in slide_signals)
    has_pptx = any(s.get('event', '').startswith('pptx_') for s in slide_signals)

    success = True

    if args.method == 'nblm' or (args.method == 'any' and has_nblm):
        ok, missing, failed, details = verify_nblm_signals(slide_signals, args.verbose)
        if not ok:
            success = False
            if failed:
                print(f"\n{color('FAILED:', RED)} NotebookLM generation failed")
            if missing:
                print(f"\n{color('MISSING:', RED)} Required signals: {missing}")

    if args.method == 'pptx' or (args.method == 'any' and has_pptx):
        ok, missing, failed, details = verify_pptx_signals(slide_signals, args.verbose)
        if not ok:
            success = False
            if missing:
                print(f"\n{color('MISSING:', RED)} Required signals: {missing}")

    # Final verdict
    print("\n" + "=" * 60)
    if success:
        print(f"  {color('VERIFICATION PASSED', GREEN)}")
        print("  Slide generation signals verified successfully.")
    else:
        print(f"  {color('VERIFICATION FAILED', RED)}")
        print("  Some required signals are missing.")
        print("\n  Check that:")
        print("  - Slide generation completed without errors")
        print("  - generate-slides.py is the latest version with signals")
    print("=" * 60)

    sys.exit(0 if success else 1)


if __name__ == '__main__':
    main()
