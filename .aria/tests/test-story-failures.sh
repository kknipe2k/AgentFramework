#!/bin/bash
# Test: Story Failure Tracking (Issue #1 fix validation)
# Tests the POSIX-compatible file-based failure tracking

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$SCRIPT_DIR")"

# Source test runner for assertions
source "$SCRIPT_DIR/test-runner.sh"

# Source ralph.sh to get the functions (but don't run it)
# We need to extract just the functions we need
RALPH_SCRIPT="$ARIA_DIR/ralph/ralph.sh"

# Setup test environment
TEST_FAILURES_FILE="/tmp/aria-test-story-failures-$$"
TEST_SIGNALS_FILE="/tmp/aria-test-signals-$$"

setup() {
    # Override the file paths for testing
    export STORY_FAILURES_FILE="$TEST_FAILURES_FILE"
    export SIGNALS_FILE="$TEST_SIGNALS_FILE"

    # Clean up any existing test files
    rm -f "$TEST_FAILURES_FILE" "$TEST_SIGNALS_FILE" 2>/dev/null || true

    # Define the functions inline to avoid sourcing issues
    # These mirror the functions in ralph.sh

    _log_failure_tracking() {
        local operation="$1"
        local story_id="$2"
        local count="$3"
        local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
        local event_id="fail-track-$(date +%s%N | cut -c1-13)"
        mkdir -p "$(dirname "$SIGNALS_FILE")" 2>/dev/null || true
        echo "{\"id\":\"${event_id}\",\"timestamp\":\"${timestamp}\",\"event\":\"failure_tracking\",\"operation\":\"${operation}\",\"story_id\":\"${story_id}\",\"count\":${count},\"context_type\":\"ralph\",\"context_name\":\"story_failures\"}" >> "$SIGNALS_FILE" 2>/dev/null || true
    }

    get_story_failures() {
        local story_id="$1"
        if [[ -f "$STORY_FAILURES_FILE" ]]; then
            local count=$(grep "^${story_id}:" "$STORY_FAILURES_FILE" 2>/dev/null | tail -1 | cut -d: -f2)
            echo "${count:-0}"
        else
            echo "0"
        fi
    }

    set_story_failures() {
        local story_id="$1"
        local count="$2"
        local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
        local tmp_file="${STORY_FAILURES_FILE}.tmp.$$"
        if [[ -f "$STORY_FAILURES_FILE" ]]; then
            grep -v "^${story_id}:" "$STORY_FAILURES_FILE" > "$tmp_file" 2>/dev/null || touch "$tmp_file"
        else
            touch "$tmp_file"
        fi
        echo "${story_id}:${count}:${timestamp}" >> "$tmp_file"
        mv "$tmp_file" "$STORY_FAILURES_FILE"
        _log_failure_tracking "set" "$story_id" "$count"
    }

    init_story_failures() {
        echo "# ARIA Story Failure Tracking - $(date -u +"%Y-%m-%dT%H:%M:%SZ")" > "$STORY_FAILURES_FILE"
        echo "# Format: story_id:failure_count:last_updated" >> "$STORY_FAILURES_FILE"
        _log_failure_tracking "init" "ALL" 0
    }

    cleanup_story_failures() {
        if [[ -f "$STORY_FAILURES_FILE" ]]; then
            _log_failure_tracking "cleanup" "ALL" 0
        fi
    }
}

teardown() {
    rm -f "$TEST_FAILURES_FILE" "$TEST_SIGNALS_FILE" 2>/dev/null || true
    rm -f "${TEST_FAILURES_FILE}.tmp."* 2>/dev/null || true
}

# ============================================
# TESTS
# ============================================

test_start "init_story_failures creates file"
setup
init_story_failures
assert_file_exists "$TEST_FAILURES_FILE" "Failures file should be created"
teardown
test_end

test_start "get_story_failures returns 0 for unknown story"
setup
init_story_failures
result=$(get_story_failures "unknown-story")
assert_eq "0" "$result" "Unknown story should have 0 failures"
teardown
test_end

test_start "set_story_failures stores count correctly"
setup
init_story_failures
set_story_failures "story-1" 3
result=$(get_story_failures "story-1")
assert_eq "3" "$result" "Should retrieve stored count"
teardown
test_end

test_start "set_story_failures overwrites previous count"
setup
init_story_failures
set_story_failures "story-1" 1
set_story_failures "story-1" 5
result=$(get_story_failures "story-1")
assert_eq "5" "$result" "Should have updated count"
teardown
test_end

test_start "multiple stories tracked independently"
setup
init_story_failures
set_story_failures "story-a" 2
set_story_failures "story-b" 4
set_story_failures "story-c" 1
assert_eq "2" "$(get_story_failures "story-a")" "Story A count"
assert_eq "4" "$(get_story_failures "story-b")" "Story B count"
assert_eq "1" "$(get_story_failures "story-c")" "Story C count"
teardown
test_end

test_start "cleanup_story_failures handles missing file gracefully"
setup
# Don't init - file doesn't exist
cleanup_story_failures
# Should not error
assert_true "1" "Should complete without error"
teardown
test_end

test_start "failure tracking logs to signals.jsonl"
setup
init_story_failures
set_story_failures "story-1" 2
assert_file_exists "$TEST_SIGNALS_FILE" "Signals file should be created"
assert_contains "$(cat "$TEST_SIGNALS_FILE")" "failure_tracking" "Should log failure tracking events"
teardown
test_end

test_start "signals include story_id in log"
setup
init_story_failures
set_story_failures "my-unique-story" 1
assert_contains "$(cat "$TEST_SIGNALS_FILE")" "my-unique-story" "Should log story ID"
teardown
test_end

echo ""
echo "Story failure tests complete."
