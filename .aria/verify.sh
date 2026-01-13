#!/bin/bash
# ARIA Verification Gate
# Run after EVERY task that modifies code
# AI must stop if this fails

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

FAILURES=0
WARNINGS=0

echo -e "${YELLOW}═══════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}              ARIA VERIFICATION GATE                        ${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════════════${NC}"
echo ""

# ============================================
# CHECK 1: Secrets Detection
# ============================================
echo -n "Checking for secrets... "

# Check staged files for potential secrets
SECRET_PATTERNS="(api[_-]?key|secret[_-]?key|password|token|credential|private[_-]?key)\s*[=:]\s*['\"][A-Za-z0-9_\-]{8,}['\"]"

if git diff --cached --name-only 2>/dev/null | head -20 | xargs grep -lE "$SECRET_PATTERNS" 2>/dev/null; then
    echo -e "${RED}FAILED${NC}"
    echo "  Possible secret detected in staged files"
    FAILURES=$((FAILURES + 1))
else
    # Also check unstaged changes
    if git diff --name-only 2>/dev/null | head -20 | xargs grep -lE "$SECRET_PATTERNS" 2>/dev/null; then
        echo -e "${YELLOW}WARNING${NC}"
        echo "  Possible secret in modified files (not staged)"
        WARNINGS=$((WARNINGS + 1))
    else
        echo -e "${GREEN}PASSED${NC}"
    fi
fi

# ============================================
# CHECK 2: Tests
# ============================================
echo -n "Running tests... "

if [[ -f "$PROJECT_DIR/package.json" ]]; then
    # Node.js project
    if grep -q '"test"' "$PROJECT_DIR/package.json" 2>/dev/null; then
        if cd "$PROJECT_DIR" && npm test --silent 2>/dev/null; then
            echo -e "${GREEN}PASSED${NC}"
        else
            echo -e "${RED}FAILED${NC}"
            echo "  npm test failed"
            FAILURES=$((FAILURES + 1))
        fi
    else
        echo -e "${YELLOW}SKIPPED${NC} (no test script)"
    fi
elif [[ -f "$PROJECT_DIR/pytest.ini" ]] || [[ -f "$PROJECT_DIR/pyproject.toml" ]] || [[ -f "$PROJECT_DIR/setup.py" ]]; then
    # Python project
    if cd "$PROJECT_DIR" && python -m pytest --quiet 2>/dev/null; then
        echo -e "${GREEN}PASSED${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        echo "  pytest failed"
        FAILURES=$((FAILURES + 1))
    fi
elif [[ -f "$PROJECT_DIR/Cargo.toml" ]]; then
    # Rust project
    if cd "$PROJECT_DIR" && cargo test --quiet 2>/dev/null; then
        echo -e "${GREEN}PASSED${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        echo "  cargo test failed"
        FAILURES=$((FAILURES + 1))
    fi
elif [[ -f "$PROJECT_DIR/go.mod" ]]; then
    # Go project
    if cd "$PROJECT_DIR" && go test ./... 2>/dev/null; then
        echo -e "${GREEN}PASSED${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        echo "  go test failed"
        FAILURES=$((FAILURES + 1))
    fi
else
    echo -e "${YELLOW}SKIPPED${NC} (no test framework detected)"
fi

# ============================================
# CHECK 3: Linting
# ============================================
echo -n "Running linter... "

if [[ -f "$PROJECT_DIR/.eslintrc" ]] || [[ -f "$PROJECT_DIR/.eslintrc.js" ]] || [[ -f "$PROJECT_DIR/.eslintrc.json" ]] || [[ -f "$PROJECT_DIR/eslint.config.js" ]]; then
    # ESLint
    if cd "$PROJECT_DIR" && npx eslint . --quiet --max-warnings=0 2>/dev/null; then
        echo -e "${GREEN}PASSED${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        echo "  ESLint errors found"
        FAILURES=$((FAILURES + 1))
    fi
elif [[ -f "$PROJECT_DIR/pyproject.toml" ]] && grep -q "ruff" "$PROJECT_DIR/pyproject.toml" 2>/dev/null; then
    # Ruff (Python)
    if cd "$PROJECT_DIR" && ruff check . --quiet 2>/dev/null; then
        echo -e "${GREEN}PASSED${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        echo "  Ruff errors found"
        FAILURES=$((FAILURES + 1))
    fi
elif command -v pylint &>/dev/null && [[ -f "$PROJECT_DIR/setup.py" || -f "$PROJECT_DIR/pyproject.toml" ]]; then
    # Pylint
    if cd "$PROJECT_DIR" && pylint --errors-only **/*.py 2>/dev/null; then
        echo -e "${GREEN}PASSED${NC}"
    else
        echo -e "${YELLOW}WARNING${NC}"
        echo "  Pylint warnings"
        WARNINGS=$((WARNINGS + 1))
    fi
else
    echo -e "${YELLOW}SKIPPED${NC} (no linter detected)"
fi

# ============================================
# CHECK 4: TypeScript Compilation
# ============================================
if [[ -f "$PROJECT_DIR/tsconfig.json" ]]; then
    echo -n "Type checking... "
    if cd "$PROJECT_DIR" && npx tsc --noEmit 2>/dev/null; then
        echo -e "${GREEN}PASSED${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        echo "  TypeScript errors found"
        FAILURES=$((FAILURES + 1))
    fi
fi

# ============================================
# CHECK 5: Build (optional)
# ============================================
if [[ -f "$PROJECT_DIR/package.json" ]] && grep -q '"build"' "$PROJECT_DIR/package.json" 2>/dev/null; then
    echo -n "Build check... "
    if cd "$PROJECT_DIR" && npm run build --silent 2>/dev/null; then
        echo -e "${GREEN}PASSED${NC}"
    else
        echo -e "${YELLOW}WARNING${NC}"
        echo "  Build has issues"
        WARNINGS=$((WARNINGS + 1))
    fi
fi

# ============================================
# CHECK 6: Don't Touch Areas
# ============================================
if [[ -f "$SCRIPT_DIR/project-context.md" ]]; then
    echo -n "Checking protected areas... "

    # Extract "don't touch" files from project-context.md
    PROTECTED=$(grep -A 100 "Don't Touch" "$SCRIPT_DIR/project-context.md" 2>/dev/null | grep "^- " | sed 's/^- //' | head -20)

    if [[ -n "$PROTECTED" ]]; then
        MODIFIED=$(git diff --name-only 2>/dev/null; git diff --cached --name-only 2>/dev/null)

        VIOLATION=""
        for protected in $PROTECTED; do
            if echo "$MODIFIED" | grep -q "$protected" 2>/dev/null; then
                VIOLATION="$protected"
                break
            fi
        done

        if [[ -n "$VIOLATION" ]]; then
            echo -e "${RED}FAILED${NC}"
            echo "  Modified protected area: $VIOLATION"
            echo "  This requires explicit HITL approval"
            FAILURES=$((FAILURES + 1))
        else
            echo -e "${GREEN}PASSED${NC}"
        fi
    else
        echo -e "${YELLOW}SKIPPED${NC} (no protected areas defined)"
    fi
fi

# ============================================
# SUMMARY
# ============================================
echo ""
echo -e "${YELLOW}═══════════════════════════════════════════════════════════${NC}"

if [[ $FAILURES -gt 0 ]]; then
    echo -e "${RED}VERIFICATION FAILED: $FAILURES issue(s)${NC}"
    echo ""
    echo "AI MUST STOP. Do not proceed to next task."
    echo "Report failures and wait for guidance."
    exit 1
elif [[ $WARNINGS -gt 0 ]]; then
    echo -e "${YELLOW}VERIFICATION PASSED WITH WARNINGS: $WARNINGS warning(s)${NC}"
    echo ""
    echo "May proceed, but consider addressing warnings."
    exit 0
else
    echo -e "${GREEN}VERIFICATION PASSED${NC}"
    echo ""
    echo "Proceed to next task."
    exit 0
fi
