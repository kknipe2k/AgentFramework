#!/bin/bash
# Test: File Ownership Model (Issue #7 fix validation)
# Tests emit_signal, emit_decision, and aria_atomic_write functions
# Validates single-writer pattern for state files

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$ARIA_DIR")"

# Setup test environment BEFORE sourcing (to isolate test runner signals)
TEST_STATE_DIR="/tmp/aria-test-ownership-$$"
mkdir -p "$TEST_STATE_DIR"

# Override state directory for ALL operations in this script
export ARIA_STATE_DIR="$TEST_STATE_DIR"
export ARIA_SIGNALS_FILE="$TEST_STATE_DIR/signals.jsonl"
export ARIA_DECISIONS_FILE="$TEST_STATE_DIR/decisions.jsonl"

# Source test runner for assertions (will now use our isolated directory)
source "$SCRIPT_DIR/test-runner.sh"

# Source common.sh for functions under test
source "$ARIA_DIR/common.sh"

setup() {
    # Clear files for each test
    rm -f "$TEST_STATE_DIR/signals.jsonl" 2>/dev/null || true
    rm -f "$TEST_STATE_DIR/decisions.jsonl" 2>/dev/null || true
    rm -f "$TEST_STATE_DIR/test.txt" 2>/dev/null || true
    rm -f "$TEST_STATE_DIR/test.json" 2>/dev/null || true
    rm -rf "$TEST_STATE_DIR/subdir" 2>/dev/null || true
}

teardown() {
    # Per-test cleanup (kept minimal)
    :
}

# Cleanup on script exit
cleanup_all() {
    rm -rf "$TEST_STATE_DIR" 2>/dev/null || true
}
trap cleanup_all EXIT

# ============================================
# EMIT_SIGNAL TESTS
# ============================================

test_start "emit_signal creates signals.jsonl"
setup
emit_signal "test_event" "test" "unit_test"
assert_file_exists "$TEST_STATE_DIR/signals.jsonl" "Signals file should be created"
teardown
test_end

test_start "emit_signal writes valid JSON"
setup
emit_signal "test_event" "test" "unit_test"
content=$(cat "$TEST_STATE_DIR/signals.jsonl")
assert_contains "$content" '"event":"test_event"' "Should contain event"
assert_contains "$content" '"context_type":"test"' "Should contain context_type"
assert_contains "$content" '"context_name":"unit_test"' "Should contain context_name"
teardown
test_end

test_start "emit_signal includes timestamp and id"
setup
emit_signal "test_event" "test" "unit_test"
content=$(cat "$TEST_STATE_DIR/signals.jsonl")
assert_contains "$content" '"timestamp":"' "Should contain timestamp"
assert_contains "$content" '"id":"sig-' "Should contain id with sig- prefix"
teardown
test_end

test_start "emit_signal handles key=value pairs"
setup
emit_signal "test_event" "test" "unit_test" "key1=value1" "key2=value2"
content=$(cat "$TEST_STATE_DIR/signals.jsonl")
assert_contains "$content" '"key1":"value1"' "Should contain key1"
assert_contains "$content" '"key2":"value2"' "Should contain key2"
teardown
test_end

test_start "emit_signal handles numeric values"
setup
emit_signal "test_event" "test" "unit_test" "count=42" "score=3.14"
content=$(cat "$TEST_STATE_DIR/signals.jsonl")
assert_contains "$content" '"count":42' "Should contain numeric count (no quotes)"
assert_contains "$content" '"score":3.14' "Should contain numeric score (no quotes)"
teardown
test_end

test_start "emit_signal escapes quotes in values"
setup
emit_signal "test_event" "test" "unit_test" 'message=hello "world"'
content=$(cat "$TEST_STATE_DIR/signals.jsonl")
assert_contains "$content" 'hello \"world\"' "Should escape quotes"
teardown
test_end

test_start "emit_signal appends multiple entries"
setup
emit_signal "event1" "test" "unit_test"
emit_signal "event2" "test" "unit_test"
emit_signal "event3" "test" "unit_test"
line_count=$(wc -l < "$TEST_STATE_DIR/signals.jsonl")
assert_eq "3" "$line_count" "Should have 3 lines"
teardown
test_end

test_start "emit_signal requires all three arguments"
setup
result=$(emit_signal "event_only" 2>&1) || true
assert_contains "$result" "requires" "Should error on missing args"
teardown
test_end

# ============================================
# EMIT_DECISION TESTS
# ============================================

test_start "emit_decision creates decisions.jsonl"
setup
emit_decision "Test action" "Test context" "Test rationale" "None" "0.85"
assert_file_exists "$TEST_STATE_DIR/decisions.jsonl" "Decisions file should be created"
teardown
test_end

test_start "emit_decision writes valid JSON"
setup
emit_decision "Test action" "Test context" "Test rationale" "Alternative A" "0.85"
content=$(cat "$TEST_STATE_DIR/decisions.jsonl")
assert_contains "$content" '"action":"Test action"' "Should contain action"
assert_contains "$content" '"context":"Test context"' "Should contain context"
assert_contains "$content" '"rationale":"Test rationale"' "Should contain rationale"
assert_contains "$content" '"alternatives":"Alternative A"' "Should contain alternatives"
assert_contains "$content" '"confidence":0.85' "Should contain confidence"
teardown
test_end

test_start "emit_decision includes timestamp and id"
setup
emit_decision "Test action" "Context" "Rationale" "Alts" "0.9"
content=$(cat "$TEST_STATE_DIR/decisions.jsonl")
assert_contains "$content" '"timestamp":"' "Should contain timestamp"
assert_contains "$content" '"id":"dec-' "Should contain id with dec- prefix"
teardown
test_end

test_start "emit_decision validates confidence range"
setup
result=$(emit_decision "Test" "Ctx" "Rat" "Alt" "1.5" 2>&1) || true
assert_contains "$result" "must be 0.0-1.0" "Should reject confidence > 1"
teardown
test_end

test_start "emit_decision handles verified=true"
setup
emit_decision "Test action" "Context" "Rationale" "Alts" "0.8" "true"
content=$(cat "$TEST_STATE_DIR/decisions.jsonl")
assert_contains "$content" '"verified":true' "Should contain verified:true"
teardown
test_end

test_start "emit_decision handles verified=false"
setup
emit_decision "Test action" "Context" "Rationale" "Alts" "0.8" "false"
content=$(cat "$TEST_STATE_DIR/decisions.jsonl")
assert_contains "$content" '"verified":false' "Should contain verified:false"
teardown
test_end

test_start "emit_decision defaults verified to null"
setup
emit_decision "Test action" "Context" "Rationale" "Alts" "0.8"
content=$(cat "$TEST_STATE_DIR/decisions.jsonl")
assert_contains "$content" '"verified":null' "Should default to verified:null"
teardown
test_end

# ============================================
# ARIA_ATOMIC_WRITE TESTS
# ============================================

test_start "aria_atomic_write creates file"
setup
echo "test content" | aria_atomic_write "$TEST_STATE_DIR/test.txt"
assert_file_exists "$TEST_STATE_DIR/test.txt" "File should be created"
assert_eq "test content" "$(cat "$TEST_STATE_DIR/test.txt")" "Content should match"
teardown
test_end

test_start "aria_atomic_write preserves content on success"
setup
echo '{"key":"value"}' | aria_atomic_write "$TEST_STATE_DIR/test.json"
content=$(cat "$TEST_STATE_DIR/test.json")
assert_contains "$content" '"key"' "Should contain key"
assert_contains "$content" '"value"' "Should contain value"
teardown
test_end

test_start "aria_atomic_write removes temp file on success"
setup
echo "content" | aria_atomic_write "$TEST_STATE_DIR/test.txt"
tmp_count=$(find "$TEST_STATE_DIR" -name ".tmp.*" 2>/dev/null | wc -l)
assert_eq "0" "$tmp_count" "No temp files should remain"
teardown
test_end

test_start "aria_atomic_write creates directory if needed"
setup
echo "nested" | aria_atomic_write "$TEST_STATE_DIR/subdir/nested.txt"
assert_file_exists "$TEST_STATE_DIR/subdir/nested.txt" "File in subdir should be created"
teardown
test_end

# ============================================
# CONCURRENT WRITE TESTS (Ownership Model)
# ============================================

test_start "concurrent emit_signal appends preserve all entries"
setup
# Simulate concurrent appends (not truly parallel but tests sequential safety)
for i in {1..10}; do
    emit_signal "concurrent_test" "test" "unit_test" "seq=$i" &
done
wait
# All 10 lines should be present
line_count=$(wc -l < "$TEST_STATE_DIR/signals.jsonl")
assert_eq "10" "$line_count" "All concurrent signals should be logged"
teardown
test_end

echo ""
echo "File ownership model tests complete."
