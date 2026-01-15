#!/bin/bash
# ARIA Verification Gate
# Run after EVERY task that modifies code
# AI must stop if this fails

# Exit on error, undefined vars, and pipeline failures
# -e: Exit immediately if a command exits with non-zero status
# -u: Treat unset variables as an error
# -o pipefail: Return exit code of first failing command in pipeline
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Source common.sh for emit_signal and other utilities
source "$SCRIPT_DIR/common.sh" 2>/dev/null || true

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

FAILURES=0
WARNINGS=0

# ============================================
# ROLLBACK SUPPORT (Issue #11)
# ============================================
# Creates checkpoint before verification for potential rollback.
# On failure, offers HITL choice to rollback changes.

CHECKPOINT_STASH=""
CHECKPOINT_ENABLED="${ARIA_VERIFY_CHECKPOINT:-true}"

# Create a checkpoint (git stash) before verification
create_checkpoint() {
    if [[ "$CHECKPOINT_ENABLED" != "true" ]]; then
        return 0
    fi

    # Only create checkpoint if there are changes
    if ! git diff --quiet 2>/dev/null || ! git diff --cached --quiet 2>/dev/null; then
        local stash_msg="aria-verify-checkpoint-$(date +%Y%m%d-%H%M%S)"

        # Stash all changes (including untracked)
        if git stash push -u -m "$stash_msg" 2>/dev/null; then
            CHECKPOINT_STASH="$stash_msg"

            # Emit signal for traceability
            if type emit_signal >/dev/null 2>&1; then
                emit_signal "verify_checkpoint_created" "verify" "rollback" \
                    "stash_message=$stash_msg"
            fi
            return 0
        fi
    fi
    return 0
}

# Restore checkpoint (rollback changes)
restore_checkpoint() {
    if [[ -z "$CHECKPOINT_STASH" ]]; then
        echo -e "${YELLOW}No checkpoint to restore${NC}"
        return 1
    fi

    echo -e "${YELLOW}Restoring checkpoint...${NC}"

    # Find the stash by message
    local stash_ref
    stash_ref=$(git stash list | grep "$CHECKPOINT_STASH" | head -1 | cut -d: -f1)

    if [[ -n "$stash_ref" ]]; then
        if git stash pop "$stash_ref" 2>/dev/null; then
            echo -e "${GREEN}Checkpoint restored successfully${NC}"

            # Emit signal
            if type emit_signal >/dev/null 2>&1; then
                emit_signal "verify_checkpoint_restored" "verify" "rollback" \
                    "stash_message=$CHECKPOINT_STASH" \
                    "action=pop"
            fi
            return 0
        fi
    fi

    echo -e "${RED}Failed to restore checkpoint${NC}"
    return 1
}

# Discard checkpoint (keep current state)
discard_checkpoint() {
    if [[ -z "$CHECKPOINT_STASH" ]]; then
        return 0
    fi

    # Find and drop the stash
    local stash_ref
    stash_ref=$(git stash list | grep "$CHECKPOINT_STASH" | head -1 | cut -d: -f1)

    if [[ -n "$stash_ref" ]]; then
        git stash drop "$stash_ref" 2>/dev/null || true

        # Emit signal
        if type emit_signal >/dev/null 2>&1; then
            emit_signal "verify_checkpoint_discarded" "verify" "rollback" \
                "stash_message=$CHECKPOINT_STASH" \
                "action=drop"
        fi
    fi

    CHECKPOINT_STASH=""
}

# HITL prompt on verification failure
handle_verification_failure() {
    local failure_count="$1"

    echo ""
    echo -e "${RED}════════════════════════════════════════════════════════${NC}"
    echo -e "${RED}  VERIFICATION FAILED - HITL CHECKPOINT${NC}"
    echo -e "${RED}════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "  ${failure_count} verification issue(s) found."
    echo ""

    if [[ -n "$CHECKPOINT_STASH" ]]; then
        echo -e "  Options:"
        echo -e "    [r]ollback  - Restore to pre-change state"
        echo -e "    [f]ix       - Keep changes, fix issues manually"
        echo -e "    [c]ontinue  - Proceed anyway (not recommended)"
        echo ""

        # Non-interactive: default to fix (stop and report)
        if [[ ! -t 0 ]]; then
            echo -e "${YELLOW}Non-interactive mode: Stopping for manual review${NC}"

            if type emit_signal >/dev/null 2>&1; then
                emit_signal "verify_failure_noninteractive" "verify" "failure" \
                    "failure_count=$failure_count" \
                    "checkpoint_available=true"
            fi
            return 1
        fi

        read -r -p "Choice [r/f/c]: " choice
        case "$choice" in
            r|R)
                if type emit_signal >/dev/null 2>&1; then
                    emit_signal "verify_rollback_requested" "verify" "rollback" \
                        "failure_count=$failure_count" \
                        "user_choice=rollback"
                fi
                restore_checkpoint
                echo ""
                echo -e "${YELLOW}Changes rolled back. Please review and try again.${NC}"
                return 2  # Special exit: rollback performed
                ;;
            c|C)
                if type emit_signal >/dev/null 2>&1; then
                    emit_signal "verify_failure_override" "verify" "failure" \
                        "failure_count=$failure_count" \
                        "user_choice=continue"
                fi
                discard_checkpoint
                echo ""
                echo -e "${YELLOW}Proceeding despite verification failures.${NC}"
                return 0  # User chose to continue
                ;;
            *)
                if type emit_signal >/dev/null 2>&1; then
                    emit_signal "verify_failure_fix" "verify" "failure" \
                        "failure_count=$failure_count" \
                        "user_choice=fix"
                fi
                discard_checkpoint
                echo ""
                echo -e "${YELLOW}Please fix the issues and run verification again.${NC}"
                return 1
                ;;
        esac
    else
        echo -e "  No checkpoint available for rollback."
        echo -e "  Please fix the issues and run verification again."

        if type emit_signal >/dev/null 2>&1; then
            emit_signal "verify_failure_no_checkpoint" "verify" "failure" \
                "failure_count=$failure_count"
        fi
        return 1
    fi
}

# Create checkpoint before verification starts
create_checkpoint

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
# CHECK 6: ARIA Framework Tests
# ============================================
if [[ -x "$SCRIPT_DIR/tests/test-runner.sh" ]]; then
    echo -n "Running ARIA framework tests... "
    if "$SCRIPT_DIR/tests/test-runner.sh" >/dev/null 2>&1; then
        echo -e "${GREEN}PASSED${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        echo "  ARIA framework tests failed"
        echo "  Run: .aria/tests/test-runner.sh for details"
        FAILURES=$((FAILURES + 1))
    fi
fi

# ============================================
# CHECK 7: Don't Touch Areas
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
    # Handle failure with HITL and rollback option (Issue #11)
    handle_verification_failure "$FAILURES"
    exit_code=$?

    case $exit_code in
        0)
            # User chose to continue despite failures
            echo -e "${RED}VERIFICATION FAILED: $FAILURES issue(s) (override approved)${NC}"
            discard_checkpoint
            exit 0
            ;;
        2)
            # Rollback was performed
            echo -e "${YELLOW}VERIFICATION ABORTED: Rollback performed${NC}"
            exit 2
            ;;
        *)
            # Standard failure
            echo -e "${RED}VERIFICATION FAILED: $FAILURES issue(s)${NC}"
            echo ""
            echo "AI MUST STOP. Do not proceed to next task."
            echo "Report failures and wait for guidance."
            discard_checkpoint
            exit 1
            ;;
    esac
elif [[ $WARNINGS -gt 0 ]]; then
    # Success with warnings - discard checkpoint
    discard_checkpoint
    echo -e "${YELLOW}VERIFICATION PASSED WITH WARNINGS: $WARNINGS warning(s)${NC}"
    echo ""
    echo "May proceed, but consider addressing warnings."

    # Emit success signal
    if type emit_signal >/dev/null 2>&1; then
        emit_signal "verify_passed_with_warnings" "verify" "result" \
            "warnings=$WARNINGS"
    fi
    exit 0
else
    # Full success - discard checkpoint
    discard_checkpoint
    echo -e "${GREEN}VERIFICATION PASSED${NC}"
    echo ""
    echo "Proceed to next task."

    # Emit success signal
    if type emit_signal >/dev/null 2>&1; then
        emit_signal "verify_passed" "verify" "result" \
            "failures=0" \
            "warnings=0"
    fi
    exit 0
fi
