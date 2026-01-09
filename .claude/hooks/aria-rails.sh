#!/bin/bash
# ARIA RAILS - Forces discipline on Claude
# Install: chmod +x, add to .claude/settings.json hooks

HOOK_EVENT="$1"
TOOL_NAME="$2"
TOOL_INPUT="$3"

ARIA_STATE_DIR=".aria"
EDIT_COUNT_FILE="$ARIA_STATE_DIR/edit_count"
LAST_TEST_FILE="$ARIA_STATE_DIR/last_test"
LAST_COMMIT_FILE="$ARIA_STATE_DIR/last_commit"
INTENT_FILE="$ARIA_STATE_DIR/intent.md"

mkdir -p "$ARIA_STATE_DIR"

# ============================================
# RAIL 1: No edits without intent
# ============================================
check_intent() {
    if [[ ! -f "$INTENT_FILE" ]]; then
        echo '{"error": "BLOCKED: No intent defined. Create .aria/intent.md first with what you are trying to accomplish."}'
        exit 2
    fi
}

# ============================================
# RAIL 2: Max 3 edits without testing
# ============================================
check_test_cadence() {
    local count=$(cat "$EDIT_COUNT_FILE" 2>/dev/null || echo 0)
    local last_test=$(cat "$LAST_TEST_FILE" 2>/dev/null || echo 0)
    local now=$(date +%s)
    local edits_since_test=$((count - last_test))

    if [[ $edits_since_test -ge 3 ]]; then
        echo '{"error": "BLOCKED: 3 edits without testing. Run tests before continuing."}'
        exit 2
    fi
}

# ============================================
# RAIL 3: Max 5 edits without commit
# ============================================
check_commit_cadence() {
    local count=$(cat "$EDIT_COUNT_FILE" 2>/dev/null || echo 0)
    local last_commit=$(cat "$LAST_COMMIT_FILE" 2>/dev/null || echo 0)
    local edits_since_commit=$((count - last_commit))

    if [[ $edits_since_commit -ge 5 ]]; then
        echo '{"error": "BLOCKED: 5 edits without commit. Commit checkpoint before continuing."}'
        exit 2
    fi
}

# ============================================
# RAIL 4: Tests must pass before commit
# ============================================
check_tests_before_commit() {
    if [[ -f "$ARIA_STATE_DIR/tests_failed" ]]; then
        echo '{"error": "BLOCKED: Cannot commit with failing tests. Fix tests first."}'
        exit 2
    fi
}

# ============================================
# TRACK: Count edits
# ============================================
increment_edit_count() {
    local count=$(cat "$EDIT_COUNT_FILE" 2>/dev/null || echo 0)
    echo $((count + 1)) > "$EDIT_COUNT_FILE"
}

# ============================================
# TRACK: Record test run
# ============================================
record_test_run() {
    local count=$(cat "$EDIT_COUNT_FILE" 2>/dev/null || echo 0)
    echo "$count" > "$LAST_TEST_FILE"

    # Check if tests passed (from exit code in input)
    if echo "$TOOL_INPUT" | grep -q '"exit_code": 0'; then
        rm -f "$ARIA_STATE_DIR/tests_failed"
    else
        touch "$ARIA_STATE_DIR/tests_failed"
    fi
}

# ============================================
# TRACK: Record commit
# ============================================
record_commit() {
    local count=$(cat "$EDIT_COUNT_FILE" 2>/dev/null || echo 0)
    echo "$count" > "$LAST_COMMIT_FILE"
}

# ============================================
# MAIN HOOK LOGIC
# ============================================

case "$HOOK_EVENT" in
    "PreToolUse")
        case "$TOOL_NAME" in
            "Edit"|"Write"|"MultiEdit")
                check_intent
                check_test_cadence
                check_commit_cadence
                ;;
            "Bash")
                # Check if it's a commit
                if echo "$TOOL_INPUT" | grep -q "git commit"; then
                    check_tests_before_commit
                fi
                ;;
        esac
        ;;

    "PostToolUse")
        case "$TOOL_NAME" in
            "Edit"|"Write"|"MultiEdit")
                increment_edit_count
                ;;
            "Bash")
                # Check if it was a test run
                if echo "$TOOL_INPUT" | grep -qE "(npm test|pytest|jest|cargo test|go test)"; then
                    record_test_run
                fi
                # Check if it was a commit
                if echo "$TOOL_INPUT" | grep -q "git commit"; then
                    record_commit
                fi
                ;;
        esac
        ;;
esac

exit 0
