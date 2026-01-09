#!/bin/bash
# ARIA RAILS v2 - Complete rail system
# Blocks Claude from bad behavior with comprehensive checks

HOOK_EVENT="$1"
TOOL_NAME="$2"
TOOL_INPUT="$3"

ARIA_DIR=".aria"
STATE_DIR="$ARIA_DIR/state"
INTENT_FILE="$ARIA_DIR/intent.md"

# Create directories
mkdir -p "$STATE_DIR"

# ============================================
# CONFIGURATION
# ============================================
TEST_CADENCE=${ARIA_TEST_CADENCE:-3}
COMMIT_CADENCE=${ARIA_COMMIT_CADENCE:-5}

# ============================================
# STATE HELPERS
# ============================================
get_count() { cat "$STATE_DIR/edit_count" 2>/dev/null || echo 0; }
get_last_test() { cat "$STATE_DIR/last_test" 2>/dev/null || echo 0; }
get_last_commit() { cat "$STATE_DIR/last_commit" 2>/dev/null || echo 0; }
tests_failing() { [[ -f "$STATE_DIR/tests_failed" ]]; }

increment_edits() {
    local count=$(get_count)
    echo $((count + 1)) > "$STATE_DIR/edit_count"
}

# ============================================
# RAIL: Intent Required
# ============================================
check_intent() {
    if [[ ! -f "$INTENT_FILE" ]]; then
        cat << 'EOF'
{"error": "BLOCKED: No intent defined. Before making changes, define your intent:\n\nRun in terminal: ./.aria/aria-engine.sh init \"your intent here\"\n\nOr create .aria/intent.md manually with:\n- What you're building\n- Must have requirements\n- Must not requirements"}
EOF
        exit 2
    fi
}

# ============================================
# RAIL: Test Cadence
# ============================================
check_test_cadence() {
    local count=$(get_count)
    local last_test=$(get_last_test)
    local since=$((count - last_test))

    if [[ $since -ge $TEST_CADENCE ]]; then
        cat << EOF
{"error": "BLOCKED: $since edits without testing (max: $TEST_CADENCE).\n\nRun tests before continuing:\n  npm test\n  pytest\n  cargo test\n\nOr mark tests passed: ./.aria/aria-engine.sh pass"}
EOF
        exit 2
    fi
}

# ============================================
# RAIL: Commit Cadence
# ============================================
check_commit_cadence() {
    local count=$(get_count)
    local last_commit=$(get_last_commit)
    local since=$((count - last_commit))

    if [[ $since -ge $COMMIT_CADENCE ]]; then
        cat << EOF
{"error": "BLOCKED: $since edits without commit (max: $COMMIT_CADENCE).\n\nCommit a checkpoint before continuing:\n  git add -A && git commit -m \"checkpoint: description\"\n\nOr reset counter: ./.aria/aria-engine.sh reset"}
EOF
        exit 2
    fi
}

# ============================================
# RAIL: Tests Before Commit
# ============================================
check_tests_before_commit() {
    if tests_failing; then
        cat << 'EOF'
{"error": "BLOCKED: Cannot commit with failing tests.\n\nFix the tests first, then run them:\n  npm test\n\nOr if tests now pass: ./.aria/aria-engine.sh pass"}
EOF
        exit 2
    fi
}

# ============================================
# RAIL: No Secrets
# ============================================
check_no_secrets() {
    local file_path="$1"

    # Quick check for common secret patterns
    if [[ -f "$file_path" ]]; then
        if grep -qE "(api[_-]?key|secret|password|token)\s*[=:]\s*['\"][A-Za-z0-9_\-]{10,}['\"]" "$file_path" 2>/dev/null; then
            cat << EOF
{"error": "BLOCKED: Possible secret detected in $file_path.\n\nUse environment variables instead:\n  process.env.API_KEY\n  os.environ['SECRET']\n\nOr add to .gitignore if this is a config file."}
EOF
            exit 2
        fi
    fi
}

# ============================================
# RAIL: No Destructive Commands
# ============================================
check_no_destructive() {
    local cmd="$1"

    # Block dangerous patterns
    local dangerous_patterns=(
        "rm -rf /"
        "rm -rf ~"
        "rm -rf \*"
        "rm -rf \."
        "> /dev/sd"
        "mkfs\."
        "dd if=.* of=/dev"
        ":(){ :|:& };:"
        "chmod -R 777 /"
        "DROP DATABASE"
        "DROP TABLE.*;"
    )

    for pattern in "${dangerous_patterns[@]}"; do
        if echo "$cmd" | grep -qE "$pattern"; then
            cat << EOF
{"error": "BLOCKED: Destructive command detected.\n\nPattern matched: $pattern\n\nThis command could cause serious damage. If you really need to run it, do so manually outside of Claude."}
EOF
            exit 2
        fi
    done

    # Warn on force push to protected branches
    if echo "$cmd" | grep -qE "git push.*(--force|-f).*(main|master|production)"; then
        cat << 'EOF'
{"error": "BLOCKED: Force push to protected branch.\n\nNever force push to main/master/production.\n\nUse a feature branch and create a pull request instead."}
EOF
        exit 2
    fi
}

# ============================================
# RAIL: Server Running (for UI changes)
# ============================================
check_server_running() {
    # Only check if we're doing verification
    if [[ -f "$STATE_DIR/needs_server" ]]; then
        if ! curl -s -o /dev/null "http://localhost:${PORT:-3000}" 2>/dev/null; then
            cat << 'EOF'
{"warning": "Server not running at localhost:3000.\n\nStart the dev server in another terminal:\n  npm start\n  npm run dev\n\nThen UI verification can proceed."}
EOF
            # Warning only, don't block
        fi
    fi
}

# ============================================
# TRACKING: Post-action updates
# ============================================
track_edit() {
    increment_edits

    # Check if this is a UI file
    local file_path=$(echo "$TOOL_INPUT" | grep -oP '"file_path"\s*:\s*"\K[^"]+' 2>/dev/null || true)
    if [[ "$file_path" =~ \.(jsx|tsx|vue|svelte|html|css)$ ]]; then
        touch "$STATE_DIR/needs_server"
    fi
}

track_test_run() {
    local exit_code=$(echo "$TOOL_INPUT" | grep -oP '"exit_code"\s*:\s*\K\d+' 2>/dev/null || echo "0")

    local count=$(get_count)
    echo "$count" > "$STATE_DIR/last_test"

    if [[ "$exit_code" == "0" ]]; then
        rm -f "$STATE_DIR/tests_failed"
    else
        touch "$STATE_DIR/tests_failed"
    fi
}

track_commit() {
    local count=$(get_count)
    echo "$count" > "$STATE_DIR/last_commit"
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

                # Extract file path and check for secrets
                file_path=$(echo "$TOOL_INPUT" | grep -oP '"file_path"\s*:\s*"\K[^"]+' 2>/dev/null || true)
                # Don't check the new content, just existing files
                ;;

            "Bash")
                # Extract command
                cmd=$(echo "$TOOL_INPUT" | grep -oP '"command"\s*:\s*"\K[^"]+' 2>/dev/null || true)

                # Check for destructive commands
                check_no_destructive "$cmd"

                # Check for commit
                if echo "$cmd" | grep -q "git commit"; then
                    check_tests_before_commit
                fi

                # Check for test runs (inform, don't block)
                if echo "$cmd" | grep -qE "(npm test|pytest|jest|cargo test|go test|make test)"; then
                    # Will be tracked in PostToolUse
                    :
                fi
                ;;

            "Task")
                # Subagent tasks - less strict but still need intent
                check_intent
                ;;
        esac
        ;;

    "PostToolUse")
        case "$TOOL_NAME" in
            "Edit"|"Write"|"MultiEdit")
                track_edit

                # Get new content and check for secrets
                new_content=$(echo "$TOOL_INPUT" | grep -oP '"new_string"\s*:\s*"\K[^"]+' 2>/dev/null || \
                              echo "$TOOL_INPUT" | grep -oP '"content"\s*:\s*"\K[^"]+' 2>/dev/null || true)

                if echo "$new_content" | grep -qE "(api[_-]?key|secret|password)\s*[=:]\s*['\"][A-Za-z0-9_\-]{10,}['\"]"; then
                    echo '{"warning": "Possible secret in new content. Consider using environment variables."}'
                fi
                ;;

            "Bash")
                cmd=$(echo "$TOOL_INPUT" | grep -oP '"command"\s*:\s*"\K[^"]+' 2>/dev/null || true)

                # Track test runs
                if echo "$cmd" | grep -qE "(npm test|pytest|jest|cargo test|go test|make test)"; then
                    track_test_run
                fi

                # Track commits
                if echo "$cmd" | grep -q "git commit"; then
                    track_commit
                fi
                ;;
        esac
        ;;

    "Stop")
        # End of turn - show status
        if [[ -f "$INTENT_FILE" ]]; then
            local count=$(get_count)
            local last_test=$(get_last_test)
            local last_commit=$(get_last_commit)
            local since_test=$((count - last_test))
            local since_commit=$((count - last_commit))

            # Warnings
            if [[ $since_test -ge $((TEST_CADENCE - 1)) ]]; then
                echo "{\"warning\": \"$since_test edits since last test. Consider running tests.\"}"
            fi
            if [[ $since_commit -ge $((COMMIT_CADENCE - 1)) ]]; then
                echo "{\"warning\": \"$since_commit edits since last commit. Consider committing a checkpoint.\"}"
            fi
            if tests_failing; then
                echo '{"warning": "Tests are currently failing."}'
            fi
        fi
        ;;
esac

exit 0
