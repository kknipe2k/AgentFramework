#!/bin/bash
# Unit Tests: Deep Research Skill
# Tests the deep-research.md skill file structure and requirements

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Source test runner
source "$(dirname "$SCRIPT_DIR")/test-runner.sh"

# ============================================
# TESTS: Skill File Exists
# ============================================

test_start "deep-research.md exists"
assert_file_exists "$ARIA_DIR/skills/deep-research.md" "Deep research skill should exist"
test_end

test_start "deep-research.md has required header"
skill_file="$ARIA_DIR/skills/deep-research.md"
if [[ -f "$skill_file" ]]; then
    has_version=$(grep -c "version:" "$skill_file" || echo "0")
    has_modes=$(grep -c "modes:" "$skill_file" || echo "0")
    has_triggers=$(grep -c "triggers:" "$skill_file" || echo "0")

    if [[ "$has_version" -gt 0 ]] && [[ "$has_modes" -gt 0 ]] && [[ "$has_triggers" -gt 0 ]]; then
        assert_true "1" "Skill has required metadata"
    else
        assert_true "" "Missing metadata (version=$has_version, modes=$has_modes, triggers=$has_triggers)"
    fi
else
    assert_true "" "Skill file not found"
fi
test_end

# ============================================
# TESTS: HITL Gates Documented
# ============================================

test_start "deep-research.md documents depth selection gate"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -qi "depth.*selection\|HITL Gate 1" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should document depth selection HITL gate"
else
    assert_true "" "Missing depth selection gate documentation"
fi
test_end

test_start "deep-research.md documents strategy selection gate"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -qi "strategy.*selection\|HITL Gate 2" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should document strategy selection HITL gate"
else
    assert_true "" "Missing strategy selection gate documentation"
fi
test_end

test_start "deep-research.md documents query approval gate"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -qi "query.*approval\|HITL Gate 3" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should document query approval HITL gate"
else
    assert_true "" "Missing query approval gate documentation"
fi
test_end

test_start "deep-research.md documents mid-research checkpoint"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -qi "mid.*research\|checkpoint\|HITL Gate 4" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should document mid-research checkpoint"
else
    assert_true "" "Missing mid-research checkpoint documentation"
fi
test_end

test_start "deep-research.md documents synthesis options"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -qi "synthesis.*options\|HITL Gate 5" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should document synthesis options gate"
else
    assert_true "" "Missing synthesis options documentation"
fi
test_end

# ============================================
# TESTS: Depth Levels Defined
# ============================================

test_start "deep-research.md defines Quick depth"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -qi "quick.*5-10\|5-10.*min" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should define Quick depth level"
else
    assert_true "" "Missing Quick depth definition"
fi
test_end

test_start "deep-research.md defines Standard depth"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -qi "standard.*15-30\|15-30.*min" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should define Standard depth level"
else
    assert_true "" "Missing Standard depth definition"
fi
test_end

test_start "deep-research.md defines Deep depth"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -qi "deep.*30-60\|30-60.*min" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should define Deep depth level"
else
    assert_true "" "Missing Deep depth definition"
fi
test_end

test_start "deep-research.md defines Exhaustive depth"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -qi "exhaustive.*60\+\|60\+.*min" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should define Exhaustive depth level"
else
    assert_true "" "Missing Exhaustive depth definition"
fi
test_end

# ============================================
# TESTS: Source Quality Ratings
# ============================================

test_start "deep-research.md defines source quality ratings"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -q "Source Quality" "$skill_file" 2>/dev/null || grep -q "quality.*A.*B.*C" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should define source quality ratings"
else
    assert_true "" "Missing source quality rating system"
fi
test_end

test_start "deep-research.md defines A-tier sources"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -qi "authoritative\|official.*docs\|academic" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should define A-tier (authoritative) sources"
else
    assert_true "" "Missing A-tier source definition"
fi
test_end

# ============================================
# TESTS: Confidence Scoring
# ============================================

test_start "deep-research.md defines confidence scoring"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -qi "confidence.*scoring\|confidence.*formula" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should define confidence scoring system"
else
    assert_true "" "Missing confidence scoring documentation"
fi
test_end

test_start "deep-research.md defines confidence labels"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -qi "very high\|high\|medium\|low\|unverified" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should define confidence labels"
else
    assert_true "" "Missing confidence labels"
fi
test_end

# ============================================
# TESTS: Output Formats
# ============================================

test_start "deep-research.md specifies research-output.json format"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -q "research-output.json" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should specify research-output.json format"
else
    assert_true "" "Missing research-output.json specification"
fi
test_end

test_start "deep-research.md specifies IDEA.md output"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -q "IDEA.md" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should specify IDEA.md output"
else
    assert_true "" "Missing IDEA.md specification"
fi
test_end

# ============================================
# TESTS: Search Strategies
# ============================================

test_start "deep-research.md documents Broad Scan strategy"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -qi "broad.*scan" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should document Broad Scan strategy"
else
    assert_true "" "Missing Broad Scan strategy"
fi
test_end

test_start "deep-research.md documents Focused Drill strategy"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -qi "focused.*drill" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should document Focused Drill strategy"
else
    assert_true "" "Missing Focused Drill strategy"
fi
test_end

test_start "deep-research.md documents Comparative strategy"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -qi "comparative" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should document Comparative strategy"
else
    assert_true "" "Missing Comparative strategy"
fi
test_end

test_start "deep-research.md documents Temporal strategy"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -qi "temporal" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should document Temporal strategy"
else
    assert_true "" "Missing Temporal strategy"
fi
test_end

# ============================================
# TESTS: Mode Variations
# ============================================

test_start "deep-research.md documents mode variations"
skill_file="$ARIA_DIR/skills/deep-research.md"
has_standard=$(grep -c "STANDARD Mode" "$skill_file" || echo "0")
has_full=$(grep -c "FULL.*Mode" "$skill_file" || echo "0")

if [[ "$has_standard" -gt 0 ]] && [[ "$has_full" -gt 0 ]]; then
    assert_true "1" "Should document STANDARD and FULL mode variations"
else
    assert_true "" "Missing mode variations (standard=$has_standard, full=$has_full)"
fi
test_end

# ============================================
# TESTS: Signal Emissions
# ============================================

test_start "deep-research.md documents signal emissions"
skill_file="$ARIA_DIR/skills/deep-research.md"
if grep -q "emit_signal" "$skill_file" 2>/dev/null; then
    assert_true "1" "Should document signal emissions for traceability"
else
    assert_true "" "Missing signal emission documentation"
fi
test_end

echo ""
echo "Deep research skill tests complete."
