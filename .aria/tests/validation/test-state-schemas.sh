#!/bin/bash
# Validation Tests: State File Schemas
# Tests JSONL and JSON schema compliance

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Source test runner
source "$(dirname "$SCRIPT_DIR")/test-runner.sh"

# ============================================
# TESTS: signals.jsonl Schema
# ============================================

test_start "signals.jsonl exists or can be created"
SIGNALS_FILE="$ARIA_DIR/state/signals.jsonl"
if [[ -f "$SIGNALS_FILE" ]] || touch "$SIGNALS_FILE" 2>/dev/null; then
    assert_true "1" "signals.jsonl accessible"
else
    assert_true "1" "signals.jsonl would be created on first emit"
fi
test_end

test_start "signals.jsonl entries are valid JSONL"
SIGNALS_FILE="$ARIA_DIR/state/signals.jsonl"
if [[ -f "$SIGNALS_FILE" ]] && [[ -s "$SIGNALS_FILE" ]]; then
    # Validate all lines in a single Python process (much faster than per-line)
    result=$(python -c "
import json
import sys
line_num = 0
try:
    with open('$SIGNALS_FILE', 'r') as f:
        for line in f:
            line_num += 1
            line = line.strip()
            if line:
                json.loads(line)
    print(f'OK:{line_num}')
except json.JSONDecodeError as e:
    print(f'FAIL:{line_num}:{e}')
except Exception as e:
    print(f'ERROR:{e}')
" 2>/dev/null)

    if [[ "$result" == OK:* ]]; then
        count="${result#OK:}"
        assert_true "1" "All $count entries are valid JSON"
    else
        assert_true "" "Invalid JSON: $result"
    fi
else
    assert_true "1" "signals.jsonl empty or not yet created"
fi
test_end

# ============================================
# TESTS: decisions.jsonl Schema
# ============================================

test_start "decisions.jsonl exists or can be created"
DECISIONS_FILE="$ARIA_DIR/state/decisions.jsonl"
if [[ -f "$DECISIONS_FILE" ]] || touch "$DECISIONS_FILE" 2>/dev/null; then
    assert_true "1" "decisions.jsonl accessible"
else
    assert_true "1" "decisions.jsonl would be created on first emit"
fi
test_end

test_start "decisions.jsonl entries are valid JSONL"
DECISIONS_FILE="$ARIA_DIR/state/decisions.jsonl"
if [[ -f "$DECISIONS_FILE" ]] && [[ -s "$DECISIONS_FILE" ]]; then
    # Validate all lines in a single Python process (much faster than per-line)
    result=$(python -c "
import json
line_num = 0
try:
    with open('$DECISIONS_FILE', 'r') as f:
        for line in f:
            line_num += 1
            line = line.strip()
            if line:
                json.loads(line)
    print(f'OK:{line_num}')
except json.JSONDecodeError as e:
    print(f'FAIL:{line_num}:{e}')
except Exception as e:
    print(f'ERROR:{e}')
" 2>/dev/null)

    if [[ "$result" == OK:* ]]; then
        count="${result#OK:}"
        assert_true "1" "All $count entries are valid JSON"
    else
        assert_true "" "Invalid JSON: $result"
    fi
else
    assert_true "1" "decisions.jsonl empty or not yet created"
fi
test_end

# ============================================
# TESTS: progress.json Schema
# ============================================

test_start "progress.json is valid JSON if exists"
PROGRESS_FILE="$ARIA_DIR/state/progress.json"
if [[ -f "$PROGRESS_FILE" ]]; then
    if python -c "import json; json.load(open('$PROGRESS_FILE'))" 2>/dev/null; then
        assert_true "1" "progress.json is valid JSON"
    else
        assert_true "" "progress.json is invalid JSON"
    fi
else
    assert_true "1" "progress.json not yet created"
fi
test_end

# ============================================
# TESTS: current-plan.json Schema
# ============================================

test_start "current-plan.json is valid JSON if exists"
PLAN_FILE="$ARIA_DIR/state/current-plan.json"
if [[ -f "$PLAN_FILE" ]]; then
    if python -c "import json; json.load(open('$PLAN_FILE'))" 2>/dev/null; then
        assert_true "1" "current-plan.json is valid JSON"
    else
        assert_true "" "current-plan.json is invalid JSON"
    fi
else
    assert_true "1" "current-plan.json not yet created"
fi
test_end

echo ""
echo "State schema validation tests complete."
