#!/bin/bash
# Test: Context Refresh Implementation (Issue #13 fix validation)
# Tests the context-refresh.sh script

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$ARIA_DIR")"

# Setup test environment BEFORE sourcing
TEST_STATE_DIR="/tmp/aria-test-context-refresh-$$"
mkdir -p "$TEST_STATE_DIR"
mkdir -p "$TEST_STATE_DIR/handoffs"

# Override state directory for ALL operations in this script
export ARIA_STATE_DIR="$TEST_STATE_DIR"
export ARIA_SIGNALS_FILE="$TEST_STATE_DIR/signals.jsonl"

# Source test runner for assertions
source "$SCRIPT_DIR/test-runner.sh"

# Source common.sh
source "$ARIA_DIR/common.sh"

REFRESH_SCRIPT="$ARIA_DIR/context-refresh.sh"

setup() {
    # Clear files for each test
    rm -f "$TEST_STATE_DIR/refresh-checkpoint.json" 2>/dev/null || true
    rm -f "$TEST_STATE_DIR/signals.jsonl" 2>/dev/null || true
    rm -rf "$TEST_STATE_DIR/handoffs/"* 2>/dev/null || true
    mkdir -p "$TEST_STATE_DIR/handoffs"
}

teardown() {
    :
}

# Cleanup on script exit
cleanup_all() {
    rm -rf "$TEST_STATE_DIR" 2>/dev/null || true
}
trap cleanup_all EXIT

# ============================================
# TESTS
# ============================================

test_start "context-refresh.sh exists and is executable"
assert_file_exists "$REFRESH_SCRIPT" "Script should exist"
assert_true "[[ -x '$REFRESH_SCRIPT' ]]" "Script should be executable"
test_end

test_start "context-refresh.sh help works"
setup
output=$("$REFRESH_SCRIPT" help 2>&1)
assert_contains "$output" "Usage:" "Should show usage"
assert_contains "$output" "save" "Should list save command"
assert_contains "$output" "handoff" "Should list handoff command"
teardown
test_end

test_start "save command creates checkpoint file"
setup
# Temporarily override STATE_DIR in the script's environment
STATE_DIR="$TEST_STATE_DIR" "$REFRESH_SCRIPT" save test_save 2>&1 >/dev/null
assert_file_exists "$TEST_STATE_DIR/refresh-checkpoint.json" "Checkpoint file should be created"
teardown
test_end

test_start "checkpoint contains required fields"
setup
STATE_DIR="$TEST_STATE_DIR" "$REFRESH_SCRIPT" save test_fields 2>&1 >/dev/null
content=$(cat "$TEST_STATE_DIR/refresh-checkpoint.json")
assert_contains "$content" '"refresh_point"' "Should have refresh_point"
assert_contains "$content" '"timestamp"' "Should have timestamp"
assert_contains "$content" '"progress"' "Should have progress"
assert_contains "$content" '"key_decisions"' "Should have key_decisions"
teardown
test_end

test_start "handoff command creates markdown file"
setup
STATE_DIR="$TEST_STATE_DIR" HANDOFFS_DIR="$TEST_STATE_DIR/handoffs" "$REFRESH_SCRIPT" handoff test_handoff 2>&1 >/dev/null
handoff_count=$(ls -1 "$TEST_STATE_DIR/handoffs"/handoff-*.md 2>/dev/null | wc -l)
assert_neq "0" "$handoff_count" "Should create handoff file"
teardown
test_end

test_start "handoff contains project info"
setup
STATE_DIR="$TEST_STATE_DIR" HANDOFFS_DIR="$TEST_STATE_DIR/handoffs" "$REFRESH_SCRIPT" handoff test_project 2>&1 >/dev/null
handoff_file=$(ls -t "$TEST_STATE_DIR/handoffs"/handoff-*.md 2>/dev/null | head -1)
content=$(cat "$handoff_file")
assert_contains "$content" "## Context Handoff" "Should have header"
assert_contains "$content" "### Project" "Should have project section"
assert_contains "$content" "### Progress" "Should have progress section"
teardown
test_end

test_start "list command shows checkpoints"
setup
STATE_DIR="$TEST_STATE_DIR" "$REFRESH_SCRIPT" save test_list 2>&1 >/dev/null
output=$(STATE_DIR="$TEST_STATE_DIR" "$REFRESH_SCRIPT" list 2>&1)
assert_contains "$output" "checkpoint" "Should show checkpoints"
teardown
test_end

test_start "load command shows checkpoint details"
setup
STATE_DIR="$TEST_STATE_DIR" "$REFRESH_SCRIPT" save test_load 2>&1 >/dev/null
output=$(STATE_DIR="$TEST_STATE_DIR" "$REFRESH_SCRIPT" load 2>&1)
assert_contains "$output" "test_load" "Should show checkpoint name"
assert_contains "$output" "Progress" "Should show progress"
teardown
test_end

test_start "save logs signal"
setup
STATE_DIR="$TEST_STATE_DIR" "$REFRESH_SCRIPT" save test_signal 2>&1 >/dev/null
assert_file_exists "$TEST_STATE_DIR/signals.jsonl" "Signals file should be created"
signal_content=$(cat "$TEST_STATE_DIR/signals.jsonl")
assert_contains "$signal_content" "context_checkpoint_saved" "Should log checkpoint save signal"
teardown
test_end

test_start "handoff logs signal"
setup
STATE_DIR="$TEST_STATE_DIR" HANDOFFS_DIR="$TEST_STATE_DIR/handoffs" "$REFRESH_SCRIPT" handoff test_handoff_signal 2>&1 >/dev/null
signal_content=$(cat "$TEST_STATE_DIR/signals.jsonl")
assert_contains "$signal_content" "context_handoff_created" "Should log handoff create signal"
teardown
test_end

echo ""
echo "Context refresh tests complete."
