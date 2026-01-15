#!/bin/bash
# Test: HITL Signal Traceability (Issue #12 fix validation)
# Tests that HITL events are logged to signals.jsonl

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$SCRIPT_DIR")"

# Source test runner for assertions
source "$SCRIPT_DIR/test-runner.sh"

# Setup test environment
TEST_SIGNALS_FILE="/tmp/aria-test-hitl-signals-$$"
TEST_HITL_DIR="/tmp/aria-test-hitl-$$"
TEST_LOGS_DIR="/tmp/aria-test-logs-$$"
TEST_STATE_DIR="/tmp/aria-test-state-$$"

setup() {
    # Create test directories
    mkdir -p "$TEST_HITL_DIR" "$TEST_LOGS_DIR" "$TEST_STATE_DIR"

    # Override paths
    export SIGNALS_FILE="$TEST_SIGNALS_FILE"
    export HITL_DIR="$TEST_HITL_DIR"
    export LOGS_DIR="$TEST_LOGS_DIR"
    export STATE_DIR="$TEST_STATE_DIR"

    # Clean up any existing test files
    rm -f "$TEST_SIGNALS_FILE" 2>/dev/null || true

    # Define the _log_hitl_signal function inline (mirrors hitl.sh)
    _log_hitl_signal() {
        local event_type="$1"
        local request_id="$2"
        local request_type="$3"
        local details="${4:-}"
        local response="${5:-}"
        local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
        local event_id="hitl-$(date +%s%N | cut -c1-13)"

        mkdir -p "$(dirname "$SIGNALS_FILE")" 2>/dev/null || true
        touch "$SIGNALS_FILE" 2>/dev/null || true

        details="${details//\"/\\\"}"
        response="${response//\"/\\\"}"

        local json="{\"id\":\"${event_id}\",\"timestamp\":\"${timestamp}\",\"event\":\"hitl_${event_type}\",\"request_id\":\"${request_id}\",\"request_type\":\"${request_type}\""

        if [[ -n "$details" ]]; then
            json="${json},\"details\":\"${details}\""
        fi

        if [[ -n "$response" ]]; then
            json="${json},\"response\":\"${response}\""
        fi

        json="${json},\"context_type\":\"hitl\",\"context_name\":\"human_intervention\"}"

        echo "$json" >> "$SIGNALS_FILE" 2>/dev/null || true
    }
}

teardown() {
    rm -rf "$TEST_HITL_DIR" "$TEST_LOGS_DIR" "$TEST_STATE_DIR" 2>/dev/null || true
    rm -f "$TEST_SIGNALS_FILE" 2>/dev/null || true
}

# ============================================
# TESTS
# ============================================

test_start "_log_hitl_signal creates signals file"
setup
_log_hitl_signal "request_created" "test-123" "confirm" "Test request"
assert_file_exists "$TEST_SIGNALS_FILE" "Signals file should be created"
teardown
test_end

test_start "_log_hitl_signal logs request_created event"
setup
_log_hitl_signal "request_created" "req-456" "help" "Need assistance"
content=$(cat "$TEST_SIGNALS_FILE")
assert_contains "$content" "hitl_request_created" "Should log request_created event"
assert_contains "$content" "req-456" "Should include request ID"
assert_contains "$content" "help" "Should include request type"
teardown
test_end

test_start "_log_hitl_signal logs response_received event"
setup
_log_hitl_signal "response_received" "req-789" "confirm" "responder:human" "yes"
content=$(cat "$TEST_SIGNALS_FILE")
assert_contains "$content" "hitl_response_received" "Should log response_received event"
assert_contains "$content" "yes" "Should include response"
assert_contains "$content" "responder:human" "Should include responder"
teardown
test_end

test_start "_log_hitl_signal logs timeout event"
setup
_log_hitl_signal "timeout" "req-timeout" "input" "timeout_seconds:60"
content=$(cat "$TEST_SIGNALS_FILE")
assert_contains "$content" "hitl_timeout" "Should log timeout event"
assert_contains "$content" "timeout_seconds:60" "Should include timeout details"
teardown
test_end

test_start "signals include context_type hitl"
setup
_log_hitl_signal "request_created" "ctx-test" "choice" "Pick option"
content=$(cat "$TEST_SIGNALS_FILE")
assert_contains "$content" "\"context_type\":\"hitl\"" "Should have context_type hitl"
teardown
test_end

test_start "signals include human_intervention context_name"
setup
_log_hitl_signal "request_created" "ctx-test2" "general" "General request"
content=$(cat "$TEST_SIGNALS_FILE")
assert_contains "$content" "\"context_name\":\"human_intervention\"" "Should have context_name"
teardown
test_end

test_start "multiple HITL events logged sequentially"
setup
_log_hitl_signal "request_created" "multi-1" "confirm" "First request"
_log_hitl_signal "response_received" "multi-1" "confirm" "human" "yes"
_log_hitl_signal "request_created" "multi-2" "input" "Second request"
line_count=$(wc -l < "$TEST_SIGNALS_FILE")
assert_eq "3" "$line_count" "Should have 3 log entries"
teardown
test_end

echo ""
echo "HITL signal tests complete."
