#!/bin/bash
# Unit Tests: Slide Generation (NotebookLM + PPTX)
# Tests the generate-slides.py script and slide-generation.md skill

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Source test runner
source "$(dirname "$SCRIPT_DIR")/test-runner.sh"

# ============================================
# TESTS: Script Exists
# ============================================

test_start "generate-slides.py exists"
assert_file_exists "$ARIA_DIR/scripts/generate-slides.py" "Slide generation script should exist"
test_end

test_start "generate-slides.py is valid Python"
if python -m py_compile "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Script should be valid Python"
else
    assert_true "" "Script has syntax errors"
fi
test_end

# ============================================
# TESTS: Skill File Exists
# ============================================

test_start "slide-generation.md exists"
assert_file_exists "$ARIA_DIR/skills/slide-generation.md" "Slide generation skill should exist"
test_end

test_start "slide-generation.md has required sections"
skill_file="$ARIA_DIR/skills/slide-generation.md"
if [[ -f "$skill_file" ]]; then
    has_when=$(grep -c "## When to Use" "$skill_file" || echo "0")
    has_workflow=$(grep -c "## Workflow\|## Path\|## Method" "$skill_file" || echo "0")

    if [[ "$has_when" -gt 0 ]] || [[ "$has_workflow" -gt 0 ]]; then
        assert_true "1" "Skill has required sections"
    else
        assert_true "" "Missing sections"
    fi
else
    assert_true "" "Skill file not found"
fi
test_end

# ============================================
# TESTS: Script Has Required Functions
# ============================================

test_start "generate-slides.py has generate_nblm function"
result=$(python -c "
import sys
sys.path.insert(0, '$ARIA_DIR/scripts')
import importlib.util
spec = importlib.util.spec_from_file_location('slides', '$ARIA_DIR/scripts/generate-slides.py')
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
assert hasattr(mod, 'generate_nblm'), 'Missing generate_nblm'
print('OK')
" 2>&1)
if [[ "$result" == "OK" ]]; then
    assert_true "1" "Should have generate_nblm function"
else
    assert_true "" "Missing generate_nblm: $result"
fi
test_end

test_start "generate-slides.py has generate_pptx function"
result=$(python -c "
import sys
sys.path.insert(0, '$ARIA_DIR/scripts')
import importlib.util
spec = importlib.util.spec_from_file_location('slides', '$ARIA_DIR/scripts/generate-slides.py')
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
assert hasattr(mod, 'generate_pptx'), 'Missing generate_pptx'
print('OK')
" 2>&1)
if [[ "$result" == "OK" ]]; then
    assert_true "1" "Should have generate_pptx function"
else
    assert_true "" "Missing generate_pptx: $result"
fi
test_end

test_start "generate-slides.py has parse_focus_doc function"
result=$(python -c "
import sys
sys.path.insert(0, '$ARIA_DIR/scripts')
import importlib.util
spec = importlib.util.spec_from_file_location('slides', '$ARIA_DIR/scripts/generate-slides.py')
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
assert hasattr(mod, 'parse_focus_doc'), 'Missing parse_focus_doc'
print('OK')
" 2>&1)
if [[ "$result" == "OK" ]]; then
    assert_true "1" "Should have parse_focus_doc function"
else
    assert_true "" "Missing parse_focus_doc: $result"
fi
test_end

test_start "generate-slides.py has extract_title_from_idea function"
result=$(python -c "
import sys
sys.path.insert(0, '$ARIA_DIR/scripts')
import importlib.util
spec = importlib.util.spec_from_file_location('slides', '$ARIA_DIR/scripts/generate-slides.py')
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
assert hasattr(mod, 'extract_title_from_idea'), 'Missing extract_title_from_idea'
print('OK')
" 2>&1)
if [[ "$result" == "OK" ]]; then
    assert_true "1" "Should have extract_title_from_idea function"
else
    assert_true "" "Missing extract_title_from_idea: $result"
fi
test_end

# ============================================
# TESTS: Focus Document Parsing (Offline)
# ============================================

test_start "parse_focus_doc extracts core ideas"
result=$(python -c "
import sys, tempfile
sys.path.insert(0, '$ARIA_DIR/scripts')
import importlib.util
spec = importlib.util.spec_from_file_location('slides', '$ARIA_DIR/scripts/generate-slides.py')
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)

# Create test FOCUS.md
focus_content = '''# FOCUS Document

## The Core

1. First core idea
2. Second core idea
3. Third core idea

## The Synthesis

1. First synthesis theme
2. Second synthesis theme
'''

with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
    f.write(focus_content)
    temp_path = f.name

from pathlib import Path
result = mod.parse_focus_doc(Path(temp_path))

import os
os.unlink(temp_path)

assert 'core_ideas' in result, 'Missing core_ideas'
assert len(result['core_ideas']) >= 1, f\"Expected core ideas, got {len(result['core_ideas'])}\"
print('OK')
" 2>&1)

if [[ "$result" == "OK" ]]; then
    assert_true "1" "Should extract core ideas from FOCUS.md"
else
    assert_true "" "Failed to parse: $result"
fi
test_end

test_start "extract_title_from_idea extracts title"
result=$(python -c "
import sys, tempfile
sys.path.insert(0, '$ARIA_DIR/scripts')
import importlib.util
spec = importlib.util.spec_from_file_location('slides', '$ARIA_DIR/scripts/generate-slides.py')
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)

# Create test IDEA.md
idea_content = '''# Test Research Title

## Summary
This is a test.
'''

with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
    f.write(idea_content)
    temp_path = f.name

from pathlib import Path
title = mod.extract_title_from_idea(Path(temp_path))

import os
os.unlink(temp_path)

assert 'Test Research Title' in title, f\"Expected title, got: {title}\"
print('OK')
" 2>&1)

if [[ "$result" == "OK" ]]; then
    assert_true "1" "Should extract title from IDEA.md"
else
    assert_true "" "Failed to extract: $result"
fi
test_end

# ============================================
# TESTS: Prompts Defined
# ============================================

test_start "generate-slides.py defines FOCUS_PROMPT"
if grep -q "FOCUS_PROMPT" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should define FOCUS_PROMPT"
else
    assert_true "" "Missing FOCUS_PROMPT"
fi
test_end

test_start "generate-slides.py defines SLIDES_PROMPT"
if grep -q "SLIDES_PROMPT" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should define SLIDES_PROMPT"
else
    assert_true "" "Missing SLIDES_PROMPT"
fi
test_end

# ============================================
# TESTS: CLI Arguments
# ============================================

test_start "generate-slides.py accepts --focus argument"
if grep -q "\-\-focus" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should accept --focus argument"
else
    assert_true "" "Missing --focus argument"
fi
test_end

test_start "generate-slides.py accepts --idea argument"
if grep -q "\-\-idea" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should accept --idea argument"
else
    assert_true "" "Missing --idea argument"
fi
test_end

test_start "generate-slides.py accepts --method argument"
if grep -q "\-\-method" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should accept --method argument"
else
    assert_true "" "Missing --method argument"
fi
test_end

test_start "generate-slides.py supports nblm method"
if grep -q "nblm" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should support nblm (NotebookLM) method"
else
    assert_true "" "Missing nblm method support"
fi
test_end

test_start "generate-slides.py supports pptx method"
if grep -q "pptx" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should support pptx method"
else
    assert_true "" "Missing pptx method support"
fi
test_end

# ============================================
# TESTS: Error Handling
# ============================================

test_start "generate-slides.py handles missing notebooklm-py gracefully"
# The script should catch ImportError and suggest installation
if grep -qi "notebooklm.*not installed\|ImportError\|pip install" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should handle missing notebooklm-py gracefully"
else
    assert_true "" "Missing notebooklm-py error handling"
fi
test_end

test_start "generate-slides.py handles missing python-pptx gracefully"
if grep -qi "python-pptx.*not installed\|ImportError" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should handle missing python-pptx gracefully"
else
    assert_true "" "Missing python-pptx error handling"
fi
test_end

# ============================================
# TESTS: Outputs Directory
# ============================================

test_start "generate-slides.py uses outputs directory"
if grep -q "outputs" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should use outputs directory for slides"
else
    assert_true "" "Missing outputs directory reference"
fi
test_end

echo ""
echo "Slide generation tests complete."
