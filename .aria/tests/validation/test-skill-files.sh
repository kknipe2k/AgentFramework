#!/bin/bash
# Validation Tests: Skill File Structure
# Tests that skill files have required sections

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Source test runner
source "$(dirname "$SCRIPT_DIR")/test-runner.sh"

SKILLS_DIR="$ARIA_DIR/skills"

# ============================================
# TESTS: Skills Directory
# ============================================

test_start "Skills directory exists"
assert_dir_exists "$SKILLS_DIR" "Skills directory should exist"
test_end

test_start "Skills directory has .md files"
if [[ -d "$SKILLS_DIR" ]]; then
    skill_count=$(find "$SKILLS_DIR" -name "*.md" -type f | wc -l)
    assert_true "$skill_count" "Should have skill files"
else
    assert_true "" "Skills directory should exist"
fi
test_end

# ============================================
# TESTS: Core Skills Exist
# ============================================

test_start "planning.md exists"
assert_file_exists "$SKILLS_DIR/planning.md" "planning.md should exist"
test_end

test_start "executing.md exists"
assert_file_exists "$SKILLS_DIR/executing.md" "executing.md should exist"
test_end

test_start "debugging.md exists"
assert_file_exists "$SKILLS_DIR/debugging.md" "debugging.md should exist"
test_end

# ============================================
# TESTS: Skill File Structure
# ============================================

test_start "Skills have markdown headers"
if [[ -d "$SKILLS_DIR" ]]; then
    skills_with_headers=0
    total_skills=0

    for skill in "$SKILLS_DIR"/*.md; do
        if [[ -f "$skill" ]]; then
            total_skills=$((total_skills + 1))
            if grep -q "^#" "$skill"; then
                skills_with_headers=$((skills_with_headers + 1))
            fi
        fi
    done

    if [[ "$skills_with_headers" -eq "$total_skills" ]]; then
        assert_true "1" "All $total_skills skills have headers"
    else
        assert_true "" "$skills_with_headers/$total_skills skills have headers"
    fi
else
    assert_true "" "Skills directory should exist"
fi
test_end

test_start "Skills are not empty"
if [[ -d "$SKILLS_DIR" ]]; then
    empty_skills=0

    for skill in "$SKILLS_DIR"/*.md; do
        if [[ -f "$skill" ]] && [[ ! -s "$skill" ]]; then
            empty_skills=$((empty_skills + 1))
        fi
    done

    if [[ "$empty_skills" -eq 0 ]]; then
        assert_true "1" "No empty skill files"
    else
        assert_true "" "$empty_skills empty skill files found"
    fi
else
    assert_true "" "Skills directory should exist"
fi
test_end

echo ""
echo "Skill file validation tests complete."
