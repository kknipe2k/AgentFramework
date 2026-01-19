#!/bin/bash
# Unit Tests: emit_signal and emit_decision functions
# Tests JSON generation, field validation, and append behavior

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
TEST_DIR="/tmp/aria-emit-test-$$"
mkdir -p "$TEST_DIR"
export ARIA_SIGNALS_FILE="$TEST_DIR/signals.jsonl"
export ARIA_DECISIONS_FILE="$TEST_DIR/decisions.jsonl"

cleanup() {
    rm -rf "$TEST_DIR" 2>/dev/null
}
trap cleanup EXIT

# ============================================
# TESTS: Signal Schema
# ============================================

test_start "Signal has required id field"
rm -f "$ARIA_SIGNALS_FILE"
emit_signal "tool_call" "tool" "Read" "file_path=/test.txt"
content=$(cat "$ARIA_SIGNALS_FILE")
assert_contains "$content" '"id":' "Signal should have id field"
test_end

test_start "Signal has required timestamp field"
rm -f "$ARIA_SIGNALS_FILE"
emit_signal "tool_call" "tool" "Read"
content=$(cat "$ARIA_SIGNALS_FILE")
assert_contains "$content" '"timestamp":' "Signal should have timestamp"
test_end

test_start "Signal has event field"
rm -f "$ARIA_SIGNALS_FILE"
emit_signal "verify_start" "verification" "test_run"
content=$(cat "$ARIA_SIGNALS_FILE")
assert_contains "$content" '"event":"verify_start"' "Signal should have event type"
test_end

test_start "Signal has context_type field"
rm -f "$ARIA_SIGNALS_FILE"
emit_signal "skill_load" "skill" "planning"
content=$(cat "$ARIA_SIGNALS_FILE")
assert_contains "$content" '"context_type":"skill"' "Signal should have context_type"
test_end

# ============================================
# TESTS: Signal Types
# ============================================

test_start "tool_call signal type"
rm -f "$ARIA_SIGNALS_FILE"
emit_signal "tool_call" "tool" "Edit" "file_path=/src/app.ts"
content=$(cat "$ARIA_SIGNALS_FILE")
assert_contains "$content" '"tool_call"' "Should emit tool_call event"
test_end

test_start "verify_pass signal type"
rm -f "$ARIA_SIGNALS_FILE"
emit_signal "verify_pass" "verification" "test_run" "tests_passed=42"
content=$(cat "$ARIA_SIGNALS_FILE")
assert_contains "$content" '"verify_pass"' "Should emit verify_pass event"
test_end

test_start "hitl_request signal type"
rm -f "$ARIA_SIGNALS_FILE"
emit_signal "hitl_request" "hitl" "confirm_delete" "response=approved"
content=$(cat "$ARIA_SIGNALS_FILE")
assert_contains "$content" '"hitl_request"' "Should emit hitl_request event"
test_end

# ============================================
# TESTS: Decision Schema
# ============================================

test_start "Decision has required action field"
rm -f "$ARIA_DECISIONS_FILE"
emit_decision "Use retry pattern" "ctx" "rat" "alt" "0.8"
content=$(cat "$ARIA_DECISIONS_FILE")
assert_contains "$content" '"action":"Use retry pattern"' "Decision should have action"
test_end

test_start "Decision has confidence field"
rm -f "$ARIA_DECISIONS_FILE"
emit_decision "action" "context" "rationale" "alternatives" "0.92"
content=$(cat "$ARIA_DECISIONS_FILE")
assert_contains "$content" '"confidence":0.92' "Decision should have confidence"
test_end

test_start "Decision has verified field (null by default)"
rm -f "$ARIA_DECISIONS_FILE"
emit_decision "action" "ctx" "rat" "alt" "0.8"
content=$(cat "$ARIA_DECISIONS_FILE")
assert_contains "$content" '"verified":null' "New decision should have verified=null"
test_end

test_start "Decision can have verified=true"
rm -f "$ARIA_DECISIONS_FILE"
emit_decision "action" "ctx" "rat" "alt" "0.8" "true"
content=$(cat "$ARIA_DECISIONS_FILE")
assert_contains "$content" '"verified":true' "Decision can be verified=true"
test_end

# ============================================
# TESTS: Append Behavior
# ============================================

test_start "Multiple signals append to file"
rm -f "$ARIA_SIGNALS_FILE"
emit_signal "event1" "type1" "action1"
emit_signal "event2" "type2" "action2"
emit_signal "event3" "type3" "action3"
line_count=$(wc -l < "$ARIA_SIGNALS_FILE")
assert_eq "3" "$line_count" "Should have 3 signal lines"
test_end

test_start "Multiple decisions append to file"
rm -f "$ARIA_DECISIONS_FILE"
emit_decision "dec1" "ctx" "rat" "alt" "0.7"
emit_decision "dec2" "ctx" "rat" "alt" "0.8"
line_count=$(wc -l < "$ARIA_DECISIONS_FILE")
assert_eq "2" "$line_count" "Should have 2 decision lines"
test_end

echo ""
echo "Emit functions tests complete."
