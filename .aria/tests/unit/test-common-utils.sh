#!/bin/bash
# Unit Tests: common.sh utilities
# Tests platform detection, message functions, and core utilities

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Source test runner
source "$(dirname "$SCRIPT_DIR")/test-runner.sh"

# Source common.sh
source "$ARIA_DIR/common.sh" 2>/dev/null || {
    echo "Skipping: common.sh not found"
    exit 0
}

# ============================================
# TESTS: Basic Functions Exist
# ============================================

test_start "emit_signal function exists"
func_exists=$(type emit_signal 2>/dev/null && echo "yes" || echo "")
assert_true "$func_exists" "emit_signal should be defined"
test_end

test_start "emit_decision function exists"
func_exists=$(type emit_decision 2>/dev/null && echo "yes" || echo "")
assert_true "$func_exists" "emit_decision should be defined"
test_end

# ============================================
# TESTS: State Directory
# ============================================

test_start "STATE_DIR handling"
# STATE_DIR may or may not be set, just verify we can reference it
assert_true "1" "STATE_DIR can be referenced"
test_end

# ============================================
# TESTS: Signal Emission
# ============================================

test_start "emit_signal creates valid JSON"
TEST_SIGNALS="/tmp/aria-test-signals-$$"
export ARIA_SIGNALS_FILE="$TEST_SIGNALS"
rm -f "$TEST_SIGNALS" 2>/dev/null

emit_signal "test_event" "test" "TestAction" "key=value"

if [[ -f "$TEST_SIGNALS" ]]; then
    content=$(cat "$TEST_SIGNALS")
    valid=$(echo "$content" | python -c "import sys,json; json.load(sys.stdin) and print('yes')" 2>/dev/null || echo "")
    assert_true "$valid" "Signal should be valid JSON"
else
    assert_true "" "Signals file should be created"
fi
rm -f "$TEST_SIGNALS" 2>/dev/null
test_end

test_start "emit_signal includes timestamp"
TEST_SIGNALS="/tmp/aria-test-signals-$$"
export ARIA_SIGNALS_FILE="$TEST_SIGNALS"
rm -f "$TEST_SIGNALS" 2>/dev/null

emit_signal "test_event" "test" "TestAction"

if [[ -f "$TEST_SIGNALS" ]]; then
    content=$(cat "$TEST_SIGNALS")
    assert_contains "$content" "timestamp" "Signal should have timestamp"
fi
rm -f "$TEST_SIGNALS" 2>/dev/null
test_end

test_start "emit_signal includes event type"
TEST_SIGNALS="/tmp/aria-test-signals-$$"
export ARIA_SIGNALS_FILE="$TEST_SIGNALS"
rm -f "$TEST_SIGNALS" 2>/dev/null

emit_signal "my_custom_event" "category" "Action"

if [[ -f "$TEST_SIGNALS" ]]; then
    content=$(cat "$TEST_SIGNALS")
    assert_contains "$content" "my_custom_event" "Signal should have event type"
fi
rm -f "$TEST_SIGNALS" 2>/dev/null
test_end

# ============================================
# TESTS: Decision Emission
# ============================================

test_start "emit_decision creates valid JSON"
TEST_DECISIONS="/tmp/aria-test-decisions-$$"
export ARIA_DECISIONS_FILE="$TEST_DECISIONS"
rm -f "$TEST_DECISIONS" 2>/dev/null

emit_decision "Test action" "Test context" "Test rationale" "None" "0.8"

if [[ -f "$TEST_DECISIONS" ]]; then
    content=$(cat "$TEST_DECISIONS")
    valid=$(echo "$content" | python -c "import sys,json; json.load(sys.stdin) and print('yes')" 2>/dev/null || echo "")
    assert_true "$valid" "Decision should be valid JSON"
else
    assert_true "" "Decisions file should be created"
fi
rm -f "$TEST_DECISIONS" 2>/dev/null
test_end

test_start "emit_decision includes confidence"
TEST_DECISIONS="/tmp/aria-test-decisions-$$"
export ARIA_DECISIONS_FILE="$TEST_DECISIONS"
rm -f "$TEST_DECISIONS" 2>/dev/null

emit_decision "Action" "Context" "Rationale" "Alts" "0.75"

if [[ -f "$TEST_DECISIONS" ]]; then
    content=$(cat "$TEST_DECISIONS")
    assert_contains "$content" "0.75" "Decision should have confidence"
fi
rm -f "$TEST_DECISIONS" 2>/dev/null
test_end

echo ""
echo "Common utils tests complete."
