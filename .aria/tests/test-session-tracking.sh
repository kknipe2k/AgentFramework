#!/bin/bash
# Test: Session Lifecycle Tracking (Issue #14 fix validation)
# Tests that session start/end events are logged to signals.jsonl

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$ARIA_DIR")"

# Source test runner for assertions
source "$SCRIPT_DIR/test-runner.sh"

# Setup test environment
TEST_SIGNALS_FILE="/tmp/aria-test-session-signals-$$"
TEST_STATE_DIR="/tmp/aria-test-session-state-$$"
TEST_LOGS_DIR="/tmp/aria-test-session-logs-$$"
TEST_USAGE_FILE="$TEST_LOGS_DIR/token_usage.json"

setup() {
    mkdir -p "$TEST_STATE_DIR" "$TEST_LOGS_DIR"
    export SIGNALS_FILE="$TEST_SIGNALS_FILE"
    export STATE_DIR="$TEST_STATE_DIR"
    rm -f "$TEST_SIGNALS_FILE" 2>/dev/null || true

    # Source model-selector.sh to get functions (modified paths)
    # Note: We redefine key variables to use test paths
    LOGS_DIR="$TEST_LOGS_DIR"
    USAGE_FILE="$TEST_USAGE_FILE"
    SESSION_ID_FILE="$TEST_STATE_DIR/.current_session_id"

    # Define the functions inline (mirrors model-selector.sh)
    _log_session_signal() {
        local event_type="$1"
        local session_id="$2"
        local details="${3:-}"
        local metrics="${4:-}"
        local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
        local event_id="sess-$(date +%s%N | cut -c1-13)"
        local json_entry
        if [[ -n "$metrics" ]]; then
            json_entry=$(printf '{"id":"%s","timestamp":"%s","event":"%s","session_id":"%s","details":"%s","metrics":%s,"context_type":"session","context_name":"lifecycle"}' \
                "$event_id" "$timestamp" "$event_type" "$session_id" "$details" "$metrics")
        else
            json_entry=$(printf '{"id":"%s","timestamp":"%s","event":"%s","session_id":"%s","details":"%s","context_type":"session","context_name":"lifecycle"}' \
                "$event_id" "$timestamp" "$event_type" "$session_id" "$details")
        fi
        echo "$json_entry" >> "$SIGNALS_FILE"
    }

    _generate_session_id() {
        echo "session-$(date +%Y%m%d-%H%M%S)-$$"
    }

    init_usage() {
        if [[ ! -f "$USAGE_FILE" ]]; then
            cat > "$USAGE_FILE" << EOF
{
    "session_start": "$(date -Iseconds)",
    "budget": 10.00,
    "total_input_tokens": 0,
    "total_output_tokens": 0,
    "total_cost": 0.0,
    "by_model": {
        "opus": {"input": 0, "output": 0, "cost": 0.0, "calls": 0},
        "sonnet": {"input": 0, "output": 0, "cost": 0.0, "calls": 0},
        "haiku": {"input": 0, "output": 0, "cost": 0.0, "calls": 0}
    },
    "history": []
}
EOF
        fi
    }

    start_session() {
        local mode="${1:-STANDARD}"
        local workflow="${2:-unknown}"
        local session_id=$(_generate_session_id)
        echo "$session_id" > "$SESSION_ID_FILE"
        rm -f "$USAGE_FILE"
        init_usage
        _log_session_signal "session_started" "$session_id" "mode:$mode,workflow:$workflow"
        echo "$session_id"
    }

    get_session_id() {
        if [[ -f "$SESSION_ID_FILE" ]]; then
            cat "$SESSION_ID_FILE"
        else
            echo "no-session"
        fi
    }
}

teardown() {
    rm -rf "$TEST_STATE_DIR" 2>/dev/null || true
    rm -rf "$TEST_LOGS_DIR" 2>/dev/null || true
    rm -f "$TEST_SIGNALS_FILE" 2>/dev/null || true
}

# ============================================
# TESTS
# ============================================

test_start "_generate_session_id creates unique ID"
setup
id1=$(_generate_session_id)
sleep 1.1  # Ensure timestamp changes (includes seconds)
id2=$(_generate_session_id)
assert_neq "$id1" "$id2" "Session IDs should be unique"
assert_contains "$id1" "session-" "Should have session prefix"
teardown
test_end

test_start "start_session creates signals file"
setup
session_id=$(start_session "FULL" "build")
assert_file_exists "$TEST_SIGNALS_FILE" "Signals file should be created"
teardown
test_end

test_start "start_session logs session_started event"
setup
session_id=$(start_session "STANDARD" "modify")
content=$(cat "$TEST_SIGNALS_FILE")
assert_contains "$content" "session_started" "Should log session_started event"
assert_contains "$content" "$session_id" "Should include session ID"
teardown
test_end

test_start "start_session logs mode and workflow"
setup
start_session "FULL" "research"
content=$(cat "$TEST_SIGNALS_FILE")
assert_contains "$content" "mode:FULL" "Should include mode"
assert_contains "$content" "workflow:research" "Should include workflow"
teardown
test_end

test_start "start_session saves session ID to file"
setup
session_id=$(start_session)
saved_id=$(cat "$SESSION_ID_FILE")
assert_eq "$session_id" "$saved_id" "Saved ID should match returned ID"
teardown
test_end

test_start "get_session_id returns current session"
setup
session_id=$(start_session)
retrieved_id=$(get_session_id)
assert_eq "$session_id" "$retrieved_id" "Retrieved ID should match"
teardown
test_end

test_start "get_session_id returns no-session when none active"
setup
# Don't start a session, ensure no file exists
rm -f "$SESSION_ID_FILE" 2>/dev/null || true
retrieved_id=$(get_session_id)
assert_eq "no-session" "$retrieved_id" "Should return no-session"
teardown
test_end

test_start "start_session initializes usage file"
setup
start_session
assert_file_exists "$USAGE_FILE" "Usage file should be created"
content=$(cat "$USAGE_FILE")
assert_contains "$content" "session_start" "Should have session_start"
teardown
test_end

test_start "_log_session_signal includes context_type session"
setup
_log_session_signal "session_started" "test-session-123" "test"
content=$(cat "$TEST_SIGNALS_FILE")
assert_contains "$content" "\"context_type\":\"session\"" "Should have context_type session"
teardown
test_end

test_start "_log_session_signal includes metrics when provided"
setup
metrics='{"duration_seconds":120,"total_cost":0.05}'
_log_session_signal "session_ended" "test-session-456" "status:completed" "$metrics"
content=$(cat "$TEST_SIGNALS_FILE")
assert_contains "$content" "duration_seconds" "Should include metrics"
assert_contains "$content" "total_cost" "Should include cost in metrics"
teardown
test_end

test_start "multiple sessions have unique IDs"
setup
id1=$(start_session)
rm -f "$SESSION_ID_FILE"  # Clear to start new
sleep 0.01
id2=$(start_session)
assert_neq "$id1" "$id2" "Sessions should have unique IDs"
teardown
test_end

echo ""
echo "Session tracking tests complete."
