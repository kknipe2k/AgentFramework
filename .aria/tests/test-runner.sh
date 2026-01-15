#!/bin/bash
# ARIA Test Runner
# Lightweight bash test framework with full traceability
# Usage: .aria/tests/test-runner.sh [test-file.sh]

# Exit on error, undefined vars, and pipeline failures
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$SCRIPT_DIR")"
SIGNALS_FILE="$ARIA_DIR/state/signals.jsonl"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

# Test state
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0
CURRENT_TEST=""
CURRENT_FILE=""
TEST_FAILURES=()

# ============================================
# TRACEABILITY - Log to signals.jsonl
# ============================================

_log_test_event() {
    local event_type="$1"
    local test_name="$2"
    local status="$3"
    local details="${4:-}"
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local event_id="test-$(date +%s%N | cut -c1-13)"

    # Ensure signals file directory exists
    mkdir -p "$(dirname "$SIGNALS_FILE")" 2>/dev/null || true
    touch "$SIGNALS_FILE" 2>/dev/null || true

    local json="{\"id\":\"${event_id}\",\"timestamp\":\"${timestamp}\",\"event\":\"${event_type}\",\"test_name\":\"${test_name}\",\"status\":\"${status}\""
    if [[ -n "$details" ]]; then
        # Escape quotes in details
        details="${details//\"/\\\"}"
        json="${json},\"details\":\"${details}\""
    fi
    if [[ -n "$CURRENT_FILE" ]]; then
        json="${json},\"test_file\":\"${CURRENT_FILE}\""
    fi
    json="${json},\"context_type\":\"test\",\"context_name\":\"test_runner\"}"

    echo "$json" >> "$SIGNALS_FILE" 2>/dev/null || true
}

# ============================================
# ASSERTION FUNCTIONS
# ============================================

# Assert two values are equal
# Usage: assert_eq "expected" "actual" "message"
assert_eq() {
    local expected="$1"
    local actual="$2"
    local message="${3:-Values should be equal}"

    if [[ "$expected" == "$actual" ]]; then
        _test_pass "$message"
    else
        _test_fail "$message" "Expected: '$expected', Got: '$actual'"
    fi
}

# Assert two values are not equal
# Usage: assert_neq "not_expected" "actual" "message"
assert_neq() {
    local not_expected="$1"
    local actual="$2"
    local message="${3:-Values should not be equal}"

    if [[ "$not_expected" != "$actual" ]]; then
        _test_pass "$message"
    else
        _test_fail "$message" "Should not equal: '$not_expected'"
    fi
}

# Assert value is true (non-empty and not "false")
# Usage: assert_true "$value" "message"
assert_true() {
    local value="$1"
    local message="${2:-Value should be true}"

    if [[ -n "$value" ]] && [[ "$value" != "false" ]] && [[ "$value" != "0" ]]; then
        _test_pass "$message"
    else
        _test_fail "$message" "Value was: '$value'"
    fi
}

# Assert value is false (empty, "false", or "0")
# Usage: assert_false "$value" "message"
assert_false() {
    local value="$1"
    local message="${2:-Value should be false}"

    if [[ -z "$value" ]] || [[ "$value" == "false" ]] || [[ "$value" == "0" ]]; then
        _test_pass "$message"
    else
        _test_fail "$message" "Value was: '$value'"
    fi
}

# Assert string contains substring
# Usage: assert_contains "haystack" "needle" "message"
assert_contains() {
    local haystack="$1"
    local needle="$2"
    local message="${3:-String should contain substring}"

    if [[ "$haystack" == *"$needle"* ]]; then
        _test_pass "$message"
    else
        _test_fail "$message" "String '$haystack' does not contain '$needle'"
    fi
}

# Assert string does not contain substring
# Usage: assert_not_contains "haystack" "needle" "message"
assert_not_contains() {
    local haystack="$1"
    local needle="$2"
    local message="${3:-String should not contain substring}"

    if [[ "$haystack" != *"$needle"* ]]; then
        _test_pass "$message"
    else
        _test_fail "$message" "String '$haystack' contains '$needle'"
    fi
}

# Assert file exists
# Usage: assert_file_exists "/path/to/file" "message"
assert_file_exists() {
    local filepath="$1"
    local message="${2:-File should exist}"

    if [[ -f "$filepath" ]]; then
        _test_pass "$message"
    else
        _test_fail "$message" "File not found: $filepath"
    fi
}

# Assert directory exists
# Usage: assert_dir_exists "/path/to/dir" "message"
assert_dir_exists() {
    local dirpath="$1"
    local message="${2:-Directory should exist}"

    if [[ -d "$dirpath" ]]; then
        _test_pass "$message"
    else
        _test_fail "$message" "Directory not found: $dirpath"
    fi
}

# Assert command succeeds (exit code 0)
# Usage: assert_success "command" "message"
assert_success() {
    local cmd="$1"
    local message="${2:-Command should succeed}"

    set +e
    eval "$cmd" >/dev/null 2>&1
    local exit_code=$?
    set -e

    if [[ $exit_code -eq 0 ]]; then
        _test_pass "$message"
    else
        _test_fail "$message" "Command failed with exit code: $exit_code"
    fi
}

# Assert command fails (non-zero exit code)
# Usage: assert_failure "command" "message"
assert_failure() {
    local cmd="$1"
    local message="${2:-Command should fail}"

    set +e
    eval "$cmd" >/dev/null 2>&1
    local exit_code=$?
    set -e

    if [[ $exit_code -ne 0 ]]; then
        _test_pass "$message"
    else
        _test_fail "$message" "Command succeeded but should have failed"
    fi
}

# Assert exit code matches expected
# Usage: assert_exit_code "command" expected_code "message"
assert_exit_code() {
    local cmd="$1"
    local expected="$2"
    local message="${3:-Exit code should match}"

    set +e
    eval "$cmd" >/dev/null 2>&1
    local actual=$?
    set -e

    if [[ $actual -eq $expected ]]; then
        _test_pass "$message"
    else
        _test_fail "$message" "Expected exit code $expected, got $actual"
    fi
}

# ============================================
# TEST LIFECYCLE
# ============================================

# Internal: Record test pass
_test_pass() {
    local message="$1"
    ((TESTS_PASSED++)) || true
    echo -e "  ${GREEN}✓${NC} $message"
    _log_test_event "assertion" "$CURRENT_TEST" "pass" "$message"
}

# Internal: Record test failure
_test_fail() {
    local message="$1"
    local details="${2:-}"
    ((TESTS_FAILED++)) || true
    echo -e "  ${RED}✗${NC} $message"
    if [[ -n "$details" ]]; then
        echo -e "    ${RED}$details${NC}"
    fi
    TEST_FAILURES+=("[$CURRENT_FILE] $CURRENT_TEST: $message - $details")
    _log_test_event "assertion" "$CURRENT_TEST" "fail" "$message: $details"
}

# Define a test function
# Usage: test "test name" <<< 'test body'
# Or: test "test name"; test_body; end_test
test_start() {
    CURRENT_TEST="$1"
    ((TESTS_RUN++)) || true
    echo -e "\n${CYAN}TEST:${NC} $CURRENT_TEST"
    _log_test_event "test_start" "$CURRENT_TEST" "running"
}

# End current test (optional, for multi-line tests)
test_end() {
    _log_test_event "test_end" "$CURRENT_TEST" "complete"
}

# Skip a test
# Usage: skip_test "reason"
skip_test() {
    local reason="${1:-No reason given}"
    echo -e "  ${YELLOW}⊘${NC} SKIPPED: $reason"
    _log_test_event "test_skip" "$CURRENT_TEST" "skipped" "$reason"
}

# ============================================
# TEST RUNNER
# ============================================

# Run a single test file
run_test_file() {
    local test_file="$1"
    CURRENT_FILE="$(basename "$test_file")"

    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}  $CURRENT_FILE${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════════${NC}"

    _log_test_event "file_start" "$CURRENT_FILE" "running"

    # Source the test file (it will call test functions)
    if source "$test_file"; then
        _log_test_event "file_end" "$CURRENT_FILE" "complete"
    else
        echo -e "${RED}Error sourcing test file: $test_file${NC}"
        _log_test_event "file_end" "$CURRENT_FILE" "error" "Failed to source"
        ((TESTS_FAILED++)) || true
    fi
}

# Run all test files
run_all_tests() {
    local test_dir="$SCRIPT_DIR"

    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}               ARIA TEST SUITE                              ${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Test directory: $test_dir"
    echo "Timestamp: $(date)"

    _log_test_event "suite_start" "all" "running"

    # Find and run all test files (test-*.sh except test-runner.sh)
    local test_files=()
    while IFS= read -r -d '' file; do
        test_files+=("$file")
    done < <(find "$test_dir" -maxdepth 1 -name "test-*.sh" ! -name "test-runner.sh" -print0 | sort -z)

    if [[ ${#test_files[@]} -eq 0 ]]; then
        echo -e "${YELLOW}No test files found.${NC}"
        echo "Create test files named 'test-*.sh' in $test_dir"
        _log_test_event "suite_end" "all" "empty" "No test files"
        return 0
    fi

    echo "Found ${#test_files[@]} test file(s)"

    for test_file in "${test_files[@]}"; do
        run_test_file "$test_file"
    done

    # Print summary
    print_summary
}

# Print test summary
print_summary() {
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}                    TEST SUMMARY                            ${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "  Tests run:    $TESTS_RUN"
    echo -e "  ${GREEN}Passed:${NC}       $TESTS_PASSED"
    echo -e "  ${RED}Failed:${NC}       $TESTS_FAILED"
    echo ""

    if [[ $TESTS_FAILED -gt 0 ]]; then
        echo -e "${RED}FAILURES:${NC}"
        for failure in "${TEST_FAILURES[@]}"; do
            echo -e "  ${RED}•${NC} $failure"
        done
        echo ""
        _log_test_event "suite_end" "all" "failed" "$TESTS_FAILED failures"
        return 1
    else
        echo -e "${GREEN}All tests passed!${NC}"
        _log_test_event "suite_end" "all" "passed" "$TESTS_PASSED passed"
        return 0
    fi
}

# ============================================
# MAIN
# ============================================

main() {
    if [[ $# -eq 0 ]]; then
        # Run all tests
        run_all_tests
    else
        # Run specific test file
        if [[ -f "$1" ]]; then
            run_test_file "$1"
            print_summary
        elif [[ -f "$SCRIPT_DIR/$1" ]]; then
            run_test_file "$SCRIPT_DIR/$1"
            print_summary
        else
            echo -e "${RED}Test file not found: $1${NC}"
            exit 1
        fi
    fi
}

# Only run main if executed directly (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
