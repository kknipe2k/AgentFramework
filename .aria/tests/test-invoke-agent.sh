#!/bin/bash
# Test: Agent Invocation (Issue #3 fix validation)
# Tests the invoke_agent function and error detection

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$SCRIPT_DIR")"

# Source test runner for assertions
source "$SCRIPT_DIR/test-runner.sh"

RALPH_SCRIPT="$ARIA_DIR/ralph/ralph.sh"

# Setup test environment
TEST_SIGNALS_FILE="/tmp/aria-test-invoke-signals-$$"

setup() {
    export SIGNALS_FILE="$TEST_SIGNALS_FILE"
    rm -f "$TEST_SIGNALS_FILE" 2>/dev/null || true

    # Define color codes (needed by extracted functions)
    RED='\033[0;31m'
    NC='\033[0m'

    # Extract invoke_agent functions from ralph.sh (lines ~144-267)
    eval "$(sed -n '144,267p' "$RALPH_SCRIPT")"
}

teardown() {
    rm -f "$TEST_SIGNALS_FILE" 2>/dev/null || true
}

# ============================================
# TESTS: Error Pattern Detection
# ============================================

test_start "_check_agent_output_for_errors detects API errors"
setup
result=$(_check_agent_output_for_errors "Error: API error occurred")
assert_eq "api_error" "$result" "Should detect API error"
teardown
test_end

test_start "_check_agent_output_for_errors detects auth failures"
setup
result=$(_check_agent_output_for_errors "Authentication failed: invalid key")
assert_eq "api_error" "$result" "Should detect auth failure as API error"
teardown
test_end

test_start "_check_agent_output_for_errors detects rate limits"
setup
result=$(_check_agent_output_for_errors "Rate limit exceeded, please retry")
assert_eq "api_error" "$result" "Should detect rate limit"
teardown
test_end

test_start "_check_agent_output_for_errors detects network errors"
setup
result=$(_check_agent_output_for_errors "Connection refused to api.anthropic.com")
assert_eq "network_error" "$result" "Should detect connection refused"
teardown
test_end

test_start "_check_agent_output_for_errors detects timeout"
setup
result=$(_check_agent_output_for_errors "Request timeout after 30s")
assert_eq "network_error" "$result" "Should detect timeout"
teardown
test_end

test_start "_check_agent_output_for_errors detects CLI errors"
setup
result=$(_check_agent_output_for_errors "bash: claude: command not found")
assert_eq "cli_error" "$result" "Should detect command not found"
teardown
test_end

test_start "_check_agent_output_for_errors detects model errors"
setup
result=$(_check_agent_output_for_errors "Model not found: claude-invalid")
assert_eq "model_error" "$result" "Should detect model not found"
teardown
test_end

test_start "_check_agent_output_for_errors returns empty for clean output"
setup
result=$(_check_agent_output_for_errors "Task completed successfully. All tests pass.")
assert_eq "" "$result" "Should return empty for clean output"
teardown
test_end

# ============================================
# TESTS: Agent Invocation Logging
# ============================================

test_start "_log_agent_invocation creates signals file"
setup
_log_agent_invocation "claude" "start" 0 "opus"
assert_file_exists "$TEST_SIGNALS_FILE" "Should create signals file"
teardown
test_end

test_start "_log_agent_invocation logs agent name"
setup
_log_agent_invocation "claude" "success" 0 "sonnet"
assert_contains "$(cat "$TEST_SIGNALS_FILE")" "claude" "Should log agent name"
teardown
test_end

test_start "_log_agent_invocation logs status"
setup
_log_agent_invocation "amp" "error" 1 "default" "api_error"
content=$(cat "$TEST_SIGNALS_FILE")
assert_contains "$content" "error" "Should log status"
assert_contains "$content" "api_error" "Should log error type"
teardown
test_end

test_start "_log_agent_invocation logs model"
setup
_log_agent_invocation "claude" "success" 0 "opus-latest"
assert_contains "$(cat "$TEST_SIGNALS_FILE")" "opus-latest" "Should log model"
teardown
test_end

# ============================================
# TESTS: invoke_agent (requires mock)
# ============================================

test_start "invoke_agent sets AGENT_OUTPUT variable"
setup
# Mock claude command to just echo
claude() { echo "Mock response"; return 0; }
export -f claude

invoke_agent "claude" "test prompt" "" "test-model" >/dev/null 2>&1 || true
assert_true "$AGENT_OUTPUT" "AGENT_OUTPUT should be set"
teardown
test_end

test_start "invoke_agent handles unknown agent"
setup
# Must capture the return value properly since set -e is disabled within invoke_agent
set +e
invoke_agent "unknown-agent" "test" "" "" >/dev/null 2>&1
exit_code=$?
set -e
assert_eq "2" "$exit_code" "Should return fatal error code for unknown agent"
teardown
test_end

echo ""
echo "Invoke agent tests complete."
