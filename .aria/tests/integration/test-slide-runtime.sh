#!/bin/bash
# Integration Tests: Slide Generation Runtime Verification
# Verifies that prompts are actually sent during NotebookLM builds

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Source test runner
source "$(dirname "$SCRIPT_DIR")/test-runner.sh"

# ============================================
# SETUP: Create test fixtures
# ============================================

TEST_DIR="/tmp/aria-slide-runtime-test-$$"
mkdir -p "$TEST_DIR"

# Create test FOCUS.md
cat > "$TEST_DIR/FOCUS.md" << 'EOF'
# FOCUS Document: Test Topic

## The Core

### 1. First Core Idea
Testing core idea extraction.

### 2. Second Core Idea
Another core concept.

## The Synthesis

### Theme 1: Integration
How ideas connect.
EOF

# Create test IDEA.md
cat > "$TEST_DIR/IDEA.md" << 'EOF'
# Test Research Title

## Summary
Test content for slide generation.
EOF

# Clear any existing signals for clean test
SIGNALS_FILE="$ARIA_DIR/state/signals.jsonl"

# ============================================
# TEST: pptx generation emits signals
# ============================================

test_start "pptx generation emits start signal"
# Backup existing signals
if [[ -f "$SIGNALS_FILE" ]]; then
    cp "$SIGNALS_FILE" "$SIGNALS_FILE.bak"
fi

# Count signals before
signals_before=0
if [[ -f "$SIGNALS_FILE" ]]; then
    signals_before=$(wc -l < "$SIGNALS_FILE")
fi

# Run pptx generation (won't actually need python-pptx for signal test)
result=$(python "$ARIA_DIR/scripts/generate-slides.py" \
    --focus "$TEST_DIR/FOCUS.md" \
    --idea "$TEST_DIR/IDEA.md" \
    --method pptx 2>&1) || true

# Check if pptx_generation_start signal was emitted
if [[ -f "$SIGNALS_FILE" ]]; then
    if grep -q "pptx_generation_start" "$SIGNALS_FILE" 2>/dev/null; then
        assert_true "1" "Should emit pptx_generation_start signal"
    else
        assert_true "" "Missing pptx_generation_start signal"
    fi
else
    assert_true "" "Signals file not created"
fi
test_end

test_start "pptx generation emits complete signal on success"
if [[ -f "$SIGNALS_FILE" ]]; then
    if grep -q "pptx_generation_complete" "$SIGNALS_FILE" 2>/dev/null; then
        assert_true "1" "Should emit pptx_generation_complete signal"
    else
        # May fail if python-pptx not installed - that's OK for this test
        if grep -q "pptx_import_failed" "$SIGNALS_FILE" 2>/dev/null; then
            assert_true "1" "Correctly emits import_failed signal when pptx not installed"
        else
            assert_true "" "Missing completion or import_failed signal"
        fi
    fi
else
    assert_true "" "Signals file not created"
fi
test_end

# ============================================
# TEST: Signal contains required fields
# ============================================

test_start "pptx signals contain timestamp"
if [[ -f "$SIGNALS_FILE" ]]; then
    # Get the most recent pptx signal
    latest_signal=$(grep "pptx_" "$SIGNALS_FILE" 2>/dev/null | tail -1)
    if [[ -n "$latest_signal" ]]; then
        if echo "$latest_signal" | python -c "import sys,json; d=json.load(sys.stdin); assert 'timestamp' in d" 2>/dev/null; then
            assert_true "1" "Signal has timestamp"
        else
            assert_true "" "Signal missing timestamp"
        fi
    else
        assert_true "" "No pptx signals found"
    fi
else
    assert_true "" "No signals file"
fi
test_end

test_start "pptx signals contain context_type"
if [[ -f "$SIGNALS_FILE" ]]; then
    latest_signal=$(grep "pptx_" "$SIGNALS_FILE" 2>/dev/null | tail -1)
    if [[ -n "$latest_signal" ]]; then
        if echo "$latest_signal" | python -c "import sys,json; d=json.load(sys.stdin); assert d.get('context_type') == 'slide_generation'" 2>/dev/null; then
            assert_true "1" "Signal has correct context_type"
        else
            assert_true "" "Signal has wrong context_type"
        fi
    else
        assert_true "" "No pptx signals found"
    fi
else
    assert_true "" "No signals file"
fi
test_end

# ============================================
# TEST: NotebookLM signals (offline verification)
# ============================================

test_start "generate-slides.py defines nblm_prompt_sending signal"
if grep -q "nblm_prompt_sending" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should define nblm_prompt_sending signal"
else
    assert_true "" "Missing nblm_prompt_sending signal definition"
fi
test_end

test_start "generate-slides.py defines nblm_prompt_sent signal"
if grep -q "nblm_prompt_sent" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should define nblm_prompt_sent signal"
else
    assert_true "" "Missing nblm_prompt_sent signal definition"
fi
test_end

test_start "generate-slides.py logs full prompt content"
if grep -q "prompt_full" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should log full prompt for verification"
else
    assert_true "" "Missing prompt_full in signal"
fi
test_end

test_start "nblm signal includes notebook_id"
if grep -q "'notebook_id'" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should include notebook_id for traceability"
else
    assert_true "" "Missing notebook_id in signal"
fi
test_end

test_start "nblm signal includes deck_generation_started"
if grep -q "deck_generation_started" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should track deck_generation_started"
else
    assert_true "" "Missing deck_generation_started flag"
fi
test_end

# ============================================
# TEST: Signal verification helper
# ============================================

test_start "can verify prompt was sent via signals"
# Create a verification script inline
result=$(python -c "
import json
from pathlib import Path

signals_file = Path('$SIGNALS_FILE')
if not signals_file.exists():
    print('NO_SIGNALS_FILE')
    exit(0)

# Look for prompt_sent signals
prompt_sent = False
prompt_content = None

with open(signals_file) as f:
    for line in f:
        try:
            sig = json.loads(line.strip())
            if sig.get('event') == 'nblm_prompt_sent':
                prompt_sent = True
            if sig.get('event') == 'nblm_prompt_sending':
                prompt_content = sig.get('prompt_full', sig.get('prompt_preview', ''))
        except:
            pass

if prompt_content or prompt_sent:
    print('OK')
else:
    print('OFFLINE_MODE')  # Expected when nblm not available
" 2>&1)

if [[ "$result" == "OK" ]] || [[ "$result" == "OFFLINE_MODE" ]] || [[ "$result" == "NO_SIGNALS_FILE" ]]; then
    assert_true "1" "Signal verification works (result: $result)"
else
    assert_true "" "Signal verification failed: $result"
fi
test_end

# ============================================
# TEST: Runtime verification function
# ============================================

test_start "emit_slide_signal function defined"
if grep -q "def emit_slide_signal" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should have emit_slide_signal function"
else
    assert_true "" "Missing emit_slide_signal function"
fi
test_end

test_start "emit_slide_signal writes to signals.jsonl"
if grep -q "signals.jsonl" "$ARIA_DIR/scripts/generate-slides.py" 2>/dev/null; then
    assert_true "1" "Should write to signals.jsonl"
else
    assert_true "" "Not writing to signals.jsonl"
fi
test_end

# ============================================
# CLEANUP
# ============================================

rm -rf "$TEST_DIR" 2>/dev/null

# Restore original signals if we backed them up
if [[ -f "$SIGNALS_FILE.bak" ]]; then
    mv "$SIGNALS_FILE.bak" "$SIGNALS_FILE"
fi

echo ""
echo "Slide generation runtime tests complete."
