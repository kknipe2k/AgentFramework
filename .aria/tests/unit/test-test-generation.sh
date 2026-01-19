#!/bin/bash
# Unit Tests: Test Generation Skill
# Tests the stack detection and test generation libraries

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Source test runner
source "$(dirname "$SCRIPT_DIR")/test-runner.sh"

# ============================================
# TESTS: Stack Detection Library
# ============================================

test_start "detect-stack.py exists"
assert_file_exists "$ARIA_DIR/lib/detect-stack.py" "Stack detection script should exist"
test_end

test_start "detect-stack.py is valid Python"
if python -m py_compile "$ARIA_DIR/lib/detect-stack.py" 2>/dev/null; then
    assert_true "1" "Script should be valid Python"
else
    assert_true "" "Script has syntax errors"
fi
test_end

test_start "detect-stack.py runs without error"
output=$(python "$ARIA_DIR/lib/detect-stack.py" --json "$ARIA_DIR" 2>&1)
exit_code=$?
if [[ $exit_code -eq 0 ]]; then
    assert_true "1" "Script should run successfully"
else
    assert_true "" "Script failed with exit code $exit_code"
fi
test_end

test_start "detect-stack.py returns valid JSON"
output=$(python "$ARIA_DIR/lib/detect-stack.py" --json "$ARIA_DIR" 2>/dev/null)
if echo "$output" | python -c "import sys,json; json.load(sys.stdin)" 2>/dev/null; then
    assert_true "1" "Output should be valid JSON"
else
    assert_true "" "Output is not valid JSON"
fi
test_end

test_start "detect-stack.py detects primary_stack field"
output=$(python "$ARIA_DIR/lib/detect-stack.py" --json "$ARIA_DIR" 2>/dev/null)
if echo "$output" | python -c "import sys,json; d=json.load(sys.stdin); assert 'primary_stack' in d" 2>/dev/null; then
    assert_true "1" "Should have primary_stack field"
else
    assert_true "" "Missing primary_stack field"
fi
test_end

# ============================================
# TESTS: Test Generation Library
# ============================================

test_start "generate-tests.py exists"
assert_file_exists "$ARIA_DIR/lib/generate-tests.py" "Test generation script should exist"
test_end

test_start "generate-tests.py is valid Python"
if python -m py_compile "$ARIA_DIR/lib/generate-tests.py" 2>/dev/null; then
    assert_true "1" "Script should be valid Python"
else
    assert_true "" "Script has syntax errors"
fi
test_end

test_start "generate-tests.py has required functions"
# Check that the module can be imported and has key functions
result=$(python -c "
import sys
sys.path.insert(0, '$ARIA_DIR/lib')
import importlib.util
spec = importlib.util.spec_from_file_location('gen', '$ARIA_DIR/lib/generate-tests.py')
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
assert hasattr(mod, 'generate_tests'), 'Missing generate_tests'
assert hasattr(mod, 'get_unit_test_template'), 'Missing get_unit_test_template'
assert hasattr(mod, 'get_github_actions_workflow'), 'Missing get_github_actions_workflow'
print('OK')
" 2>&1)
if [[ "$result" == "OK" ]]; then
    assert_true "1" "Should have required functions"
else
    assert_true "" "Missing functions: $result"
fi
test_end

# ============================================
# TESTS: Integration - Generate in Temp Dir
# ============================================

test_start "generate-tests.py creates test files"
TEST_PROJECT="/tmp/aria-test-gen-$$"
mkdir -p "$TEST_PROJECT"

# Create minimal package.json for Node detection
echo '{"name":"test","dependencies":{"react":"18.0.0"}}' > "$TEST_PROJECT/package.json"
echo '{}' > "$TEST_PROJECT/tsconfig.json"

# Run generator
python "$ARIA_DIR/lib/generate-tests.py" "$TEST_PROJECT" --mode LITE --json >/dev/null 2>&1

# Check files were created
if [[ -d "$TEST_PROJECT/tests" ]]; then
    assert_true "1" "Should create tests directory"
else
    assert_true "" "tests directory not created"
fi

rm -rf "$TEST_PROJECT" 2>/dev/null
test_end

test_start "generate-tests.py creates CI workflow"
TEST_PROJECT="/tmp/aria-test-gen-$$"
mkdir -p "$TEST_PROJECT"
echo '{"name":"test"}' > "$TEST_PROJECT/package.json"

python "$ARIA_DIR/lib/generate-tests.py" "$TEST_PROJECT" --mode STANDARD --json >/dev/null 2>&1

if [[ -f "$TEST_PROJECT/.github/workflows/test.yml" ]]; then
    assert_true "1" "Should create GitHub Actions workflow"
else
    assert_true "" "Workflow not created"
fi

rm -rf "$TEST_PROJECT" 2>/dev/null
test_end

test_start "generate-tests.py creates verify.sh"
TEST_PROJECT="/tmp/aria-test-gen-$$"
mkdir -p "$TEST_PROJECT"
echo '{"name":"test"}' > "$TEST_PROJECT/package.json"

python "$ARIA_DIR/lib/generate-tests.py" "$TEST_PROJECT" --mode LITE --json >/dev/null 2>&1

if [[ -f "$TEST_PROJECT/.aria/verify.sh" ]]; then
    assert_true "1" "Should create verify.sh"
else
    assert_true "" "verify.sh not created"
fi

rm -rf "$TEST_PROJECT" 2>/dev/null
test_end

# ============================================
# TESTS: Skill File
# ============================================

test_start "test-generation.md skill exists"
assert_file_exists "$ARIA_DIR/skills/test-generation.md" "Skill file should exist"
test_end

test_start "test-generation.md has required sections"
skill_file="$ARIA_DIR/skills/test-generation.md"
if [[ -f "$skill_file" ]]; then
    has_when=$(grep -c "## When to Use" "$skill_file" || echo "0")
    has_workflow=$(grep -c "## Workflow" "$skill_file" || echo "0")
    has_modes=$(grep -c "## Mode Variations" "$skill_file" || echo "0")

    if [[ "$has_when" -gt 0 ]] && [[ "$has_workflow" -gt 0 ]] && [[ "$has_modes" -gt 0 ]]; then
        assert_true "1" "Skill has required sections"
    else
        assert_true "" "Missing sections (when=$has_when, workflow=$has_workflow, modes=$has_modes)"
    fi
else
    assert_true "" "Skill file not found"
fi
test_end

echo ""
echo "Test generation skill tests complete."
