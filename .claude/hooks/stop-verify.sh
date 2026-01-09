#!/bin/bash
# Stop Hook: Verify work at end of turn
# Based on Boris's pattern - verification 2-3x quality

ARIA_DIR=".aria"
STATE_DIR="$ARIA_DIR/state"

# Only run if we have an active session
if [[ ! -f "$ARIA_DIR/intent.md" ]]; then
    exit 0
fi

# Check if significant work was done
EDIT_COUNT=$(cat "$STATE_DIR/edit_count" 2>/dev/null || echo 0)
if [[ $EDIT_COUNT -lt 1 ]]; then
    exit 0
fi

echo "{"
echo "  \"verification\": {"

# 1. Check if tests exist and pass
if [[ -f "package.json" ]]; then
    if npm test --silent 2>/dev/null; then
        echo "    \"tests\": \"pass\","
    else
        echo "    \"tests\": \"fail\","
    fi
else
    echo "    \"tests\": \"skip\","
fi

# 2. Check for obvious issues in changed files
ISSUES=""
for file in $(git diff --name-only HEAD 2>/dev/null | head -10); do
    if [[ -f "$file" ]]; then
        # Check for console.log in production code
        if echo "$file" | grep -qE "\.(js|ts|tsx)$" && ! echo "$file" | grep -qE "(test|spec)"; then
            if grep -q "console\.log" "$file" 2>/dev/null; then
                ISSUES="$ISSUES console.log in $file;"
            fi
        fi
        # Check for TODO
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

# 3. Summary
LAST_TEST=$(cat "$STATE_DIR/last_test" 2>/dev/null || echo 0)
EDITS_SINCE_TEST=$((EDIT_COUNT - LAST_TEST))

if [[ $EDITS_SINCE_TEST -gt 2 ]]; then
    echo "    \"recommendation\": \"Run tests - $EDITS_SINCE_TEST edits since last test\""
elif [[ -n "$ISSUES" ]]; then
    echo "    \"recommendation\": \"Review issues before committing\""
else
    echo "    \"recommendation\": \"Ready to commit\""
fi

echo "  }"
echo "}"

exit 0
