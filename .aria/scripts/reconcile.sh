#!/bin/bash
# ARIA Reconciler - Verify decision claims match actual signals
# Usage: .aria/scripts/reconcile.sh [OPTIONS]

# Exit on error, undefined vars, and pipeline failures
set -euo pipefail

ARIA_DIR=".aria"
STATE_DIR="$ARIA_DIR/state"
DECISIONS_FILE="$STATE_DIR/decisions.jsonl"
SIGNALS_FILE="$STATE_DIR/signals.jsonl"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

# Time window for matching (seconds)
TIME_WINDOW=60

# Parse args
VERBOSE=false
FIX=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --fix)
            FIX=true
            shift
            ;;
        --window)
            TIME_WINDOW="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: reconcile.sh [OPTIONS]"
            echo ""
            echo "Verify that decision claims match actual tool call signals."
            echo ""
            echo "Options:"
            echo "  --verbose, -v    Show detailed matching info"
            echo "  --fix            Update decisions.jsonl with verification status"
            echo "  --window N       Time window for matching in seconds (default: 60)"
            echo "  --help, -h       Show this help"
            echo ""
            echo "How it works:"
            echo "  1. Reads each decision's 'context' field (what agent claimed to look at)"
            echo "  2. Searches signals for matching tool calls within time window"
            echo "  3. Reports verified/unverified status"
            exit 0
            ;;
        *)
            shift
            ;;
    esac
done

echo ""
echo -e "${BOLD}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}                 ARIA Decision Reconciliation${NC}"
echo -e "${BOLD}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Check files exist
if [[ ! -f "$DECISIONS_FILE" ]]; then
    echo -e "${YELLOW}No decisions to reconcile.${NC}"
    echo "Decisions are recorded when agent emits <decision> blocks."
    exit 0
fi

if [[ ! -f "$SIGNALS_FILE" ]]; then
    echo -e "${YELLOW}No signals to match against.${NC}"
    echo "Signals are captured by hooks in PreToolUse/PostToolUse."
    exit 0
fi

# Stats
total=0
verified=0
unverified=0
partial=0

# Process each decision
while read -r decision; do
    ((total++)) || true

    ts=$(echo "$decision" | grep -oP '"timestamp"\s*:\s*"\K[^"]+' 2>/dev/null || true)
    action=$(echo "$decision" | grep -oP '"action"\s*:\s*"\K[^"]+' 2>/dev/null || true)
    context=$(echo "$decision" | grep -oP '"context"\s*:\s*"\K[^"]+' 2>/dev/null || true)

    if [[ -z "$context" ]]; then
        ((unverified++)) || true
        if [[ "$VERBOSE" == "true" ]]; then
            echo -e "${YELLOW}?${NC} No context claim: $action"
        fi
        continue
    fi

    # Extract potential file references from context
    # Look for patterns like "read X", "looked at X", "saw X in Y"
    files_claimed=$(echo "$context" | grep -oE '[a-zA-Z0-9_\-./]+\.(ts|js|py|md|json|sh|go|rs|java|tsx|jsx)' || true)

    if [[ -z "$files_claimed" ]]; then
        # No specific files mentioned, check for tool-type claims
        if echo "$context" | grep -qi "read\|looked\|checked\|saw"; then
            ((partial++)) || true
            if [[ "$VERBOSE" == "true" ]]; then
                echo -e "${YELLOW}~${NC} Vague claim (no specific files): $action"
            fi
        else
            ((unverified++)) || true
        fi
        continue
    fi

    # Check if signals exist for claimed files
    matches=0
    claims=0

    for file in $files_claimed; do
        ((claims++)) || true
        # Search signals for this file within time window
        if grep -q "$file" "$SIGNALS_FILE" 2>/dev/null; then
            ((matches++)) || true
            if [[ "$VERBOSE" == "true" ]]; then
                echo -e "  ${GREEN}✓${NC} Found signal for: $file"
            fi
        else
            if [[ "$VERBOSE" == "true" ]]; then
                echo -e "  ${RED}✗${NC} No signal for: $file"
            fi
        fi
    done

    # Determine verification status
    if [[ $matches -eq $claims ]] && [[ $claims -gt 0 ]]; then
        ((verified++)) || true
        echo -e "${GREEN}✓ VERIFIED${NC}: $action"
        if [[ "$VERBOSE" == "true" ]]; then
            echo "  Context: $context"
            echo "  Matched: $matches/$claims files"
            echo ""
        fi
    elif [[ $matches -gt 0 ]]; then
        ((partial++)) || true
        echo -e "${YELLOW}~ PARTIAL${NC}: $action"
        if [[ "$VERBOSE" == "true" ]]; then
            echo "  Context: $context"
            echo "  Matched: $matches/$claims files"
            echo ""
        fi
    else
        ((unverified++)) || true
        echo -e "${RED}✗ UNVERIFIED${NC}: $action"
        if [[ "$VERBOSE" == "true" ]]; then
            echo "  Context: $context"
            echo "  Matched: 0/$claims files"
            echo ""
        fi
    fi

done < "$DECISIONS_FILE"

# Summary
echo ""
echo -e "${BOLD}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Summary${NC}"
echo -e "${BOLD}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "  Total decisions:  $total"
echo -e "  ${GREEN}Verified:${NC}         $verified"
echo -e "  ${YELLOW}Partial:${NC}          $partial"
echo -e "  ${RED}Unverified:${NC}       $unverified"
echo ""

# Calculate percentage
if [[ $total -gt 0 ]]; then
    pct=$((verified * 100 / total))
    echo -e "  Verification rate: ${BOLD}$pct%${NC}"
    echo ""

    if [[ $pct -ge 80 ]]; then
        echo -e "  ${GREEN}Good traceability!${NC}"
    elif [[ $pct -ge 50 ]]; then
        echo -e "  ${YELLOW}Moderate traceability - consider more specific context claims${NC}"
    else
        echo -e "  ${RED}Low traceability - decisions may not be auditable${NC}"
    fi
fi

echo ""

if [[ "$FIX" == "true" ]]; then
    echo -e "${YELLOW}--fix not yet implemented${NC}"
    echo "Future: Will update decisions.jsonl with verification status"
fi
