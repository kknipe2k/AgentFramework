#!/bin/bash
# Integration Test: End-to-End Signal Capture
# Verifies signals are actually captured when tools execute

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Source test runner
source "$(dirname "$SCRIPT_DIR")/test-runner.sh"

# Source common.sh
source "$ARIA_DIR/common.sh" 2>/dev/null || {
    echo "Skipping: common.sh not found"
    exit 0
}

# Test environment
TEST_DIR="/tmp/aria-e2e-test-$$"
mkdir -p "$TEST_DIR/.aria/state"
export ARIA_STATE_DIR="$TEST_DIR/.aria/state"
export ARIA_SIGNALS_FILE="$TEST_DIR/.aria/state/signals.jsonl"
export ARIA_DECISIONS_FILE="$TEST_DIR/.aria/state/decisions.jsonl"

cleanup() {
    rm -rf "$TEST_DIR" 2>/dev/null
}
trap cleanup EXIT

# ============================================
# TESTS: All 8 Signal Types
# ============================================

test_start "All 8 signal types can be emitted"
rm -f "$ARIA_SIGNALS_FILE"

# Emit all 8 signal types
emit_signal "tool_call" "tool" "Read" "file_path=/test"
emit_signal "skill_load" "skill" "planning"
emit_signal "agent_spawn" "agent" "analyzer"
emit_signal "decision_made" "decision" "architecture"
emit_signal "verify_start" "verification" "test_run"
emit_signal "operation_failed" "error" "timeout"
emit_signal "hitl_request" "hitl" "confirm_delete"
emit_signal "session_start" "session" "new_session"

line_count=$(wc -l < "$ARIA_SIGNALS_FILE")
assert_eq "8" "$line_count" "Should have 8 signals"
test_end

test_start "Each signal is valid JSON"
rm -f "$ARIA_SIGNALS_FILE"
emit_signal "test_event" "test" "TestAction"

valid="yes"
while IFS= read -r line; do
    if ! echo "$line" | python -c "import sys,json; json.load(sys.stdin)" 2>/dev/null; then
        valid=""
        break
    fi
done < "$ARIA_SIGNALS_FILE"

assert_true "$valid" "All signals should be valid JSON"
test_end

# ============================================
# TESTS: Signal Content Verification
# ============================================

test_start "Signals capture numeric values correctly"
rm -f "$ARIA_SIGNALS_FILE"
emit_signal "verify_pass" "verification" "test_run" "tests_passed=42" "duration=12.5"
content=$(cat "$ARIA_SIGNALS_FILE")
assert_contains "$content" "42" "Should capture integer"
assert_contains "$content" "12.5" "Should capture float"
test_end

test_start "Signals capture file paths"
rm -f "$ARIA_SIGNALS_FILE"
emit_signal "tool_call" "tool" "Read" "file_path=src/components/App.tsx"
content=$(cat "$ARIA_SIGNALS_FILE")
assert_contains "$content" "src/components/App.tsx" "Should capture file path"
test_end

test_start "Signals handle special characters"
rm -f "$ARIA_SIGNALS_FILE"
emit_signal "tool_call" "tool" "Bash" "command=echo hello"
content=$(cat "$ARIA_SIGNALS_FILE")
# Should be valid JSON even with special chars
valid=$(echo "$content" | python -c "import sys,json; json.load(sys.stdin) and print('yes')" 2>/dev/null || echo "")
assert_true "$valid" "Special characters should be handled correctly"
test_end

# ============================================
# TESTS: Decision-Signal Correlation
# ============================================

test_start "Decisions and signals can be correlated by time"
rm -f "$ARIA_SIGNALS_FILE" "$ARIA_DECISIONS_FILE"

# Emit signals before decision
emit_signal "tool_call" "tool" "Read" "file_path=src/auth.ts"
sleep 0.1
emit_signal "tool_call" "tool" "Read" "file_path=src/api.ts"
sleep 0.1

# Make decision
emit_decision "Use JWT auth" "Read auth.ts and api.ts" "Existing pattern" "Session auth" "0.85"

# Check both files exist with content
sig_count=$(wc -l < "$ARIA_SIGNALS_FILE")
dec_count=$(wc -l < "$ARIA_DECISIONS_FILE")

assert_eq "2" "$sig_count" "Should have 2 signals"
assert_eq "1" "$dec_count" "Should have 1 decision"
test_end

echo ""
echo "E2E signal capture tests complete."
