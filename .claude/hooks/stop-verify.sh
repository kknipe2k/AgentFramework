#!/bin/bash
# Stop Hook: Verify work at end of turn
# Based on Boris's pattern - verification gives 2-3x quality
# Now uses ARIA verification executor

ARIA_DIR=".aria"
STATE_DIR="$ARIA_DIR/state"
RALPH_DIR="$ARIA_DIR/ralph"
PRD_FILE="$RALPH_DIR/prd.json"
EXECUTOR="$ARIA_DIR/verify-executor.sh"

# Check if in Ralph mode
is_ralph_mode() {
    [[ -f "$PRD_FILE" ]] && [[ "${ARIA_RALPH_MODE:-0}" == "1" ]]
}

# Only run if we have an active session (interactive or Ralph)
if [[ ! -f "$ARIA_DIR/intent.md" ]] && ! is_ralph_mode; then
    exit 0
fi

# Check if significant work was done
EDIT_COUNT=$(cat "$STATE_DIR/edit_count" 2>/dev/null || echo 0)
if [[ $EDIT_COUNT -lt 1 ]]; then
    exit 0
fi

# Use executor if available
if [[ -x "$EXECUTOR" ]]; then
    # Run quick verification
    "$EXECUTOR" quick 2>/dev/null
    RESULT=$?

    # Generate JSON output for Claude to see
    echo "{"
    echo "  \"verification\": {"

    # Read state files
    for check in unit_tests types lint; do
        state_file="$STATE_DIR/$check"
        if [[ -f "$state_file" ]]; then
            status=$(cat "$state_file")
            echo "    \"$check\": \"$status\","
        fi
    done

    # Check for issues in changed files
    ISSUES=""
    for file in $(git diff --name-only HEAD 2>/dev/null | head -10); do
        if [[ -f "$file" ]]; then
            if echo "$file" | grep -qE "\.(js|ts|tsx)$" && ! echo "$file" | grep -qE "(test|spec)"; then
                if grep -q "console\.log" "$file" 2>/dev/null; then
                    ISSUES="$ISSUES console.log in $file;"
                fi
            fi
            if grep -q "TODO\|FIXME" "$file" 2>/dev/null; then
                ISSUES="$ISSUES TODO in $file;"
            fi
        fi
    done

    if [[ -n "$ISSUES" ]]; then
        echo "    \"issues\": \"$ISSUES\","
    else
        echo "    \"issues\": \"none\","
    fi

    # Recommendation
    LAST_TEST=$(cat "$STATE_DIR/last_test" 2>/dev/null || echo 0)
    EDITS_SINCE_TEST=$((EDIT_COUNT - LAST_TEST))

    if [[ $RESULT -ne 0 ]]; then
        echo "    \"recommendation\": \"Fix failing checks before continuing\""
        if is_ralph_mode; then
            echo "<aria-blocked>VERIFICATION_FAILED</aria-blocked>"
        fi
    elif [[ $EDITS_SINCE_TEST -gt 2 ]]; then
        echo "    \"recommendation\": \"Run tests - $EDITS_SINCE_TEST edits since last test\""
    elif [[ -n "$ISSUES" ]]; then
        echo "    \"recommendation\": \"Review issues before committing\""
    else
        echo "    \"recommendation\": \"Ready to commit\""
    fi

    echo "  }"
    echo "}"
else
    # Fallback to original simple verification
    echo "{"
    echo "  \"verification\": {"

    if [[ -f "package.json" ]]; then
        if npm test --silent 2>/dev/null; then
            echo "    \"tests\": \"pass\","
        else
            echo "    \"tests\": \"fail\","
        fi
    else
        echo "    \"tests\": \"skip\","
    fi

    echo "    \"recommendation\": \"Install ARIA executor for full verification\""
    echo "  }"
    echo "}"
fi

exit 0
