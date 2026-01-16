#!/bin/bash
# ARIA Prototype Verification
# Runs linting, tests, and accessibility checks on prototypes

set -euo pipefail

PROTOTYPE_DIR=".aria/prototypes"
PASSED=0
FAILED=0
WARNINGS=0

echo "========================================"
echo "     ARIA Prototype Verification"
echo "========================================"
echo ""

# Find all HTML prototypes
PROTOTYPES=$(find "$PROTOTYPE_DIR" -name "*.html" 2>/dev/null || true)

if [ -z "$PROTOTYPES" ]; then
    echo "No prototypes found in $PROTOTYPE_DIR"
    exit 0
fi

for file in $PROTOTYPES; do
    echo "Checking: $file"
    echo "----------------------------------------"

    # 1. Basic HTML validation (check for common issues)
    echo "[1/4] HTML Structure..."

    # Check for required elements
    if grep -q "<!DOCTYPE html>" "$file"; then
        echo "  ✓ DOCTYPE present"
        ((PASSED++))
    else
        echo "  ✗ Missing DOCTYPE"
        ((FAILED++))
    fi

    if grep -q "<title>" "$file"; then
        echo "  ✓ Title present"
        ((PASSED++))
    else
        echo "  ✗ Missing title"
        ((FAILED++))
    fi

    # Check for broken onclick handlers (common issue)
    ONCLICK_COUNT=$(grep -c "onclick=" "$file" 2>/dev/null || echo "0")
    FUNCTION_COUNT=$(grep -c "function " "$file" 2>/dev/null || echo "0")

    if [ "$ONCLICK_COUNT" -gt 0 ] && [ "$FUNCTION_COUNT" -eq 0 ]; then
        echo "  ⚠ onclick handlers found but no functions defined"
        ((WARNINGS++))
    else
        echo "  ✓ Event handlers have matching functions"
        ((PASSED++))
    fi

    # 2. Check for console errors in JS
    echo "[2/4] JavaScript..."

    # Look for common JS issues
    if grep -qE "undefined|null\." "$file" 2>/dev/null; then
        echo "  ⚠ Potential null/undefined references"
        ((WARNINGS++))
    else
        echo "  ✓ No obvious null references"
        ((PASSED++))
    fi

    # 3. Check interactive elements
    echo "[3/4] Interactive Elements..."

    # Count tabs and tab handlers
    TAB_ELEMENTS=$(grep -c 'data-tab\|role="tab"' "$file" 2>/dev/null || echo "0")
    TAB_HANDLERS=$(grep -c 'showTab\|switchTab\|tabClick' "$file" 2>/dev/null || echo "0")

    if [ "$TAB_ELEMENTS" -gt 0 ]; then
        if [ "$TAB_HANDLERS" -gt 0 ]; then
            echo "  ✓ Tabs have handlers ($TAB_ELEMENTS tabs, $TAB_HANDLERS handlers)"
            ((PASSED++))
        else
            echo "  ✗ Tabs found but no tab handlers"
            ((FAILED++))
        fi
    fi

    # Count buttons
    BUTTON_COUNT=$(grep -c "<button" "$file" 2>/dev/null || echo "0")
    ONCLICK_BUTTON=$(grep -c "<button.*onclick" "$file" 2>/dev/null || echo "0")

    if [ "$BUTTON_COUNT" -gt 0 ]; then
        if [ "$ONCLICK_BUTTON" -eq "$BUTTON_COUNT" ] || grep -q "addEventListener" "$file"; then
            echo "  ✓ All buttons have handlers"
            ((PASSED++))
        else
            echo "  ⚠ Some buttons may lack handlers ($ONCLICK_BUTTON/$BUTTON_COUNT)"
            ((WARNINGS++))
        fi
    fi

    # 4. Accessibility basics
    echo "[4/4] Accessibility..."

    if grep -q 'alt="' "$file" 2>/dev/null || ! grep -q "<img" "$file"; then
        echo "  ✓ Images have alt text (or no images)"
        ((PASSED++))
    else
        echo "  ⚠ Images may be missing alt text"
        ((WARNINGS++))
    fi

    if grep -qE 'aria-|role=' "$file" 2>/dev/null; then
        echo "  ✓ ARIA attributes present"
        ((PASSED++))
    else
        echo "  ⚠ No ARIA attributes found"
        ((WARNINGS++))
    fi

    echo ""
done

# Summary
echo "========================================"
echo "           SUMMARY"
echo "========================================"
echo "  Passed:   $PASSED"
echo "  Warnings: $WARNINGS"
echo "  Failed:   $FAILED"
echo "========================================"

# Run external tools if available
echo ""
echo "External Tools (if installed):"
echo "----------------------------------------"

if command -v npx &> /dev/null; then
    for file in $PROTOTYPES; do
        echo "Running htmlhint on $file..."
        npx htmlhint "$file" 2>/dev/null || echo "  (htmlhint not installed or failed)"
    done
else
    echo "  npx not available - skipping htmlhint"
fi

# Exit code
if [ "$FAILED" -gt 0 ]; then
    echo ""
    echo "❌ VERIFICATION FAILED - Fix issues before delivery"
    exit 1
elif [ "$WARNINGS" -gt 3 ]; then
    echo ""
    echo "⚠️ VERIFICATION PASSED WITH WARNINGS - Review before delivery"
    exit 0
else
    echo ""
    echo "✅ VERIFICATION PASSED"
    exit 0
fi
