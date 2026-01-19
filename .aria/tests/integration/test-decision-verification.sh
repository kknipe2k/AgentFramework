#!/bin/bash
# Integration Test: Decision Verification Lifecycle
# Tests the full lifecycle of decisions:
# 1. Decision created with verified=null
# 2. Verification runs (tests)
# 3. Decision updated with verification outcome

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
TEST_DIR="/tmp/aria-decision-test-$$"
mkdir -p "$TEST_DIR/.aria/state"
export ARIA_STATE_DIR="$TEST_DIR/.aria/state"
export ARIA_SIGNALS_FILE="$TEST_DIR/.aria/state/signals.jsonl"
export ARIA_DECISIONS_FILE="$TEST_DIR/.aria/state/decisions.jsonl"

cleanup() {
    rm -rf "$TEST_DIR" 2>/dev/null
}
trap cleanup EXIT

# ============================================
# TESTS: Decision Creation
# ============================================

test_start "New decision has verified=null"
rm -f "$ARIA_DECISIONS_FILE"
emit_decision "Add retry logic" "Read utils/retry.ts" "Consistency" "Custom impl" "0.85"
content=$(cat "$ARIA_DECISIONS_FILE")
assert_contains "$content" '"verified":null' "New decision should have verified=null"
test_end

test_start "Decision has confidence score"
rm -f "$ARIA_DECISIONS_FILE"
emit_decision "action" "ctx" "rat" "alt" "0.85"
content=$(cat "$ARIA_DECISIONS_FILE")
assert_contains "$content" '"confidence":0.85' "Confidence should be 0.85"
test_end

# ============================================
# TESTS: Verification Signals
# ============================================

test_start "Verification signals capture test results"
rm -f "$ARIA_SIGNALS_FILE"
emit_signal "verify_start" "verification" "test_run" "test_type=unit"
emit_signal "verify_pass" "verification" "test_run" "duration=12.5" "tests_passed=42"

verify_count=$(grep -c "verification" "$ARIA_SIGNALS_FILE" || echo "0")
assert_eq "2" "$verify_count" "Should have 2 verification signals"
test_end

test_start "Verification failure is captured"
rm -f "$ARIA_SIGNALS_FILE"
emit_signal "verify_fail" "verification" "test_run" "tests_failed=4"
content=$(cat "$ARIA_SIGNALS_FILE")
assert_contains "$content" "verify_fail" "Should capture failure"
test_end

# ============================================
# TESTS: Decision Verified Status
# ============================================

test_start "Decision can be marked verified=true"
rm -f "$ARIA_DECISIONS_FILE"
emit_decision "action" "ctx" "rat" "alt" "0.8" "true"
content=$(cat "$ARIA_DECISIONS_FILE")
assert_contains "$content" '"verified":true' "Should accept verified=true"
test_end

test_start "Decision can be marked verified=false"
rm -f "$ARIA_DECISIONS_FILE"
emit_decision "action" "ctx" "rat" "alt" "0.8" "false"
content=$(cat "$ARIA_DECISIONS_FILE")
assert_contains "$content" '"verified":false' "Should accept verified=false"
test_end

# ============================================
# TESTS: Full Workflow
# ============================================

test_start "Full decision lifecycle: create -> verify -> record"
rm -f "$ARIA_SIGNALS_FILE" "$ARIA_DECISIONS_FILE"

# Step 1: Read files (pre-decision signals)
emit_signal "tool_call" "tool" "Read" "file_path=src/auth.ts"
emit_signal "tool_call" "tool" "Read" "file_path=src/api.ts"

# Step 2: Make decision
emit_decision "Implement JWT middleware" "Read auth.ts and api.ts" "Existing patterns" "Inline validation" "0.85"

# Step 3: Implementation signals
emit_signal "tool_call" "tool" "Edit" "file_path=src/middleware/jwt.ts"
emit_signal "tool_call" "tool" "Write" "file_path=src/middleware/jwt.test.ts"

# Step 4: Verify
emit_signal "verify_start" "verification" "test_run"
emit_signal "verify_pass" "verification" "test_run" "tests_passed=12"

# Count everything
sig_count=$(wc -l < "$ARIA_SIGNALS_FILE")
dec_count=$(wc -l < "$ARIA_DECISIONS_FILE")

assert_eq "6" "$sig_count" "Should have 6 signals"
assert_eq "1" "$dec_count" "Should have 1 decision"
test_end

echo ""
echo "Decision verification lifecycle tests complete."
