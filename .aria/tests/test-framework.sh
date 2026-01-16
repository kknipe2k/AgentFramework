#!/bin/bash
# Test: ARIA Framework Structure
# Validates framework files exist and have correct structure

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$ARIA_DIR")"

# Source test runner for assertions
source "$SCRIPT_DIR/test-runner.sh"

# ============================================
# TESTS: Directory Structure
# ============================================

test_start "ARIA directory exists"
assert_dir_exists "$ARIA_DIR" ".aria directory should exist"
test_end

test_start "Skills directory exists"
assert_dir_exists "$ARIA_DIR/skills" "Skills directory should exist"
test_end

test_start "State directory exists"
assert_dir_exists "$ARIA_DIR/state" "State directory should exist"
test_end

test_start "Scripts directory exists"
assert_dir_exists "$ARIA_DIR/scripts" "Scripts directory should exist"
test_end

test_start "Ralph directory exists"
assert_dir_exists "$ARIA_DIR/ralph" "Ralph directory should exist"
test_end

test_start "Hooks directory exists"
assert_dir_exists "$ARIA_DIR/hooks" "Hooks directory should exist"
test_end

# ============================================
# TESTS: Core Scripts Exist
# ============================================

test_start "verify.sh exists and is executable"
assert_file_exists "$ARIA_DIR/verify.sh" "verify.sh should exist"
assert_success "[[ -x '$ARIA_DIR/verify.sh' ]]" "verify.sh should be executable"
test_end

test_start "ralph.sh exists and is executable"
assert_file_exists "$ARIA_DIR/ralph/ralph.sh" "ralph.sh should exist"
assert_success "[[ -x '$ARIA_DIR/ralph/ralph.sh' ]]" "ralph.sh should be executable"
test_end

test_start "model-selector.sh exists"
assert_file_exists "$ARIA_DIR/model-selector.sh" "model-selector.sh should exist"
test_end

test_start "git-ops.sh exists"
assert_file_exists "$ARIA_DIR/git-ops.sh" "git-ops.sh should exist"
test_end

test_start "hitl.sh exists"
assert_file_exists "$ARIA_DIR/hitl.sh" "hitl.sh should exist"
test_end

# ============================================
# TESTS: Script Headers (set -euo pipefail)
# ============================================

check_pipefail() {
    local script="$1"
    if grep -q "set -euo pipefail" "$script" 2>/dev/null; then
        return 0
    else
        return 1
    fi
}

test_start "verify.sh has set -euo pipefail"
assert_success "check_pipefail '$ARIA_DIR/verify.sh'" "Should have pipefail"
test_end

test_start "ralph.sh has set -euo pipefail"
assert_success "check_pipefail '$ARIA_DIR/ralph/ralph.sh'" "Should have pipefail"
test_end

test_start "model-selector.sh has set -euo pipefail"
assert_success "check_pipefail '$ARIA_DIR/model-selector.sh'" "Should have pipefail"
test_end

test_start "git-ops.sh has set -euo pipefail"
assert_success "check_pipefail '$ARIA_DIR/git-ops.sh'" "Should have pipefail"
test_end

test_start "hitl.sh has set -euo pipefail"
assert_success "check_pipefail '$ARIA_DIR/hitl.sh'" "Should have pipefail"
test_end

# ============================================
# TESTS: Skills Exist
# ============================================

test_start "planning.md skill exists"
assert_file_exists "$ARIA_DIR/skills/planning.md" "planning.md should exist"
test_end

test_start "executing.md skill exists"
assert_file_exists "$ARIA_DIR/skills/executing.md" "executing.md should exist"
test_end

test_start "debugging.md skill exists"
assert_file_exists "$ARIA_DIR/skills/debugging.md" "debugging.md should exist"
test_end

test_start "tdd.md skill exists"
assert_file_exists "$ARIA_DIR/skills/tdd.md" "tdd.md should exist"
test_end

# ============================================
# TESTS: Bash Syntax Validation
# ============================================

test_start "verify.sh has valid bash syntax"
assert_success "bash -n '$ARIA_DIR/verify.sh'" "Should pass syntax check"
test_end

test_start "ralph.sh has valid bash syntax"
assert_success "bash -n '$ARIA_DIR/ralph/ralph.sh'" "Should pass syntax check"
test_end

test_start "model-selector.sh has valid bash syntax"
assert_success "bash -n '$ARIA_DIR/model-selector.sh'" "Should pass syntax check"
test_end

test_start "git-ops.sh has valid bash syntax"
assert_success "bash -n '$ARIA_DIR/git-ops.sh'" "Should pass syntax check"
test_end

test_start "hitl.sh has valid bash syntax"
assert_success "bash -n '$ARIA_DIR/hitl.sh'" "Should pass syntax check"
test_end

test_start "aria-engine.sh has valid bash syntax"
assert_success "bash -n '$ARIA_DIR/aria-engine.sh'" "Should pass syntax check"
test_end

# ============================================
# TESTS: CLAUDE.md Structure
# ============================================

test_start "CLAUDE.md exists in project root"
assert_file_exists "$PROJECT_ROOT/CLAUDE.md" "CLAUDE.md should exist"
test_end

test_start "CLAUDE.md contains mode definitions"
assert_contains "$(cat "$PROJECT_ROOT/CLAUDE.md")" "LITE" "Should define LITE mode"
assert_contains "$(cat "$PROJECT_ROOT/CLAUDE.md")" "STANDARD" "Should define STANDARD mode"
assert_contains "$(cat "$PROJECT_ROOT/CLAUDE.md")" "FULL" "Should define FULL mode"
test_end

test_start "CLAUDE.md contains verification section"
assert_contains "$(cat "$PROJECT_ROOT/CLAUDE.md")" "Verification Gate" "Should have verification section"
test_end

echo ""
echo "Framework structure tests complete."
