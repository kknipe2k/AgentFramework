#!/bin/bash
# Test: Safe State File Operations (Issue #7 fix validation)
# Tests atomic writes and file locking to prevent race conditions

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$ARIA_DIR")"

# Source test runner for assertions
source "$SCRIPT_DIR/test-runner.sh"

# Source common.sh for functions under test
source "$ARIA_DIR/common.sh"

# Setup test environment
TEST_STATE_DIR="/tmp/aria-test-safe-state-$$"

setup() {
    mkdir -p "$TEST_STATE_DIR"
}

teardown() {
    rm -rf "$TEST_STATE_DIR" 2>/dev/null || true
}

# ============================================
# TESTS
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

test_start "aria_append_jsonl creates file and appends"
setup
aria_append_jsonl "$TEST_STATE_DIR/events.jsonl" '{"event":"first"}'
assert_file_exists "$TEST_STATE_DIR/events.jsonl" "File should be created"
line_count=$(wc -l < "$TEST_STATE_DIR/events.jsonl")
assert_eq "1" "$line_count" "Should have 1 line"
teardown
test_end

test_start "aria_append_jsonl appends multiple lines"
setup
aria_append_jsonl "$TEST_STATE_DIR/events.jsonl" '{"event":"first"}'
aria_append_jsonl "$TEST_STATE_DIR/events.jsonl" '{"event":"second"}'
aria_append_jsonl "$TEST_STATE_DIR/events.jsonl" '{"event":"third"}'
line_count=$(wc -l < "$TEST_STATE_DIR/events.jsonl")
assert_eq "3" "$line_count" "Should have 3 lines"
teardown
test_end

test_start "aria_write_json creates valid JSON"
setup
echo '{"name":"test","value":42}' | aria_write_json "$TEST_STATE_DIR/data.json"
assert_file_exists "$TEST_STATE_DIR/data.json" "File should be created"
content=$(cat "$TEST_STATE_DIR/data.json")
assert_contains "$content" '"name"' "Should contain name key"
assert_contains "$content" '"test"' "Should contain test value"
teardown
test_end

test_start "aria_read_json returns content"
setup
echo '{"key":"value"}' > "$TEST_STATE_DIR/read.json"
content=$(aria_read_json "$TEST_STATE_DIR/read.json")
assert_contains "$content" '"key"' "Should return content"
teardown
test_end

test_start "aria_read_json returns empty for missing file"
setup
content=$(aria_read_json "$TEST_STATE_DIR/nonexistent.json")
assert_eq "" "$content" "Should return empty for missing file"
teardown
test_end

test_start "aria_update_json modifies existing JSON"
setup
echo '{"count":0}' > "$TEST_STATE_DIR/update.json"
aria_update_json "$TEST_STATE_DIR/update.json" '.count = 1'
new_count=$(jq '.count' "$TEST_STATE_DIR/update.json")
assert_eq "1" "$new_count" "Count should be updated to 1"
teardown
test_end

test_start "aria_update_json creates file if not exists"
setup
aria_update_json "$TEST_STATE_DIR/new.json" '.created = true'
assert_file_exists "$TEST_STATE_DIR/new.json" "File should be created"
created=$(jq '.created' "$TEST_STATE_DIR/new.json")
assert_eq "true" "$created" "Created field should be true"
teardown
test_end

test_start "lock files are created during operations"
setup
echo "content" | aria_locked_write "$TEST_STATE_DIR/locked.txt"
# Lock file should exist (may be empty)
assert_file_exists "$TEST_STATE_DIR/locked.txt.lock" "Lock file should exist"
teardown
test_end

test_start "concurrent appends preserve data integrity"
setup
# Simulate concurrent appends (not truly parallel but tests the mechanism)
for i in {1..10}; do
    aria_append_jsonl "$TEST_STATE_DIR/concurrent.jsonl" "{\"seq\":$i}" &
done
wait
# All 10 lines should be present
line_count=$(wc -l < "$TEST_STATE_DIR/concurrent.jsonl")
assert_eq "10" "$line_count" "All concurrent appends should succeed"
teardown
test_end

echo ""
echo "Safe state file tests complete."
