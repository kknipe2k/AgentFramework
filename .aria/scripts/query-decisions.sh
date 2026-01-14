#!/bin/bash
# ARIA Decision Query - Search past decisions for precedent
# Usage: .aria/scripts/query-decisions.sh <search_term> [OPTIONS]

set -e

ARIA_DIR=".aria"
STATE_DIR="$ARIA_DIR/state"
DECISIONS_FILE="$STATE_DIR/decisions.jsonl"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

# Parse args
SEARCH_TERM=""
LIMIT=10
SHOW_CONTEXT=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --limit|-n)
            LIMIT="$2"
            shift 2
            ;;
        --context|-c)
            SHOW_CONTEXT=true
            shift
            ;;
        --help|-h)
            echo "Usage: query-decisions.sh <search_term> [OPTIONS]"
            echo ""
            echo "Search past decisions for precedent and patterns."
            echo ""
            echo "Arguments:"
            echo "  search_term      Text to search for in decisions"
            echo ""
            echo "Options:"
            echo "  --limit N, -n N  Show at most N results (default: 10)"
            echo "  --context, -c    Show full context and rationale"
            echo "  --help, -h       Show this help"
            echo ""
            echo "Examples:"
            echo "  query-decisions.sh auth          # Find auth-related decisions"
            echo "  query-decisions.sh retry -c      # Show retry decisions with context"
            echo "  query-decisions.sh \"error handling\" -n 5"
            exit 0
            ;;
        *)
            if [[ -z "$SEARCH_TERM" ]]; then
                SEARCH_TERM="$1"
            fi
            shift
            ;;
    esac
done

# Validate
if [[ -z "$SEARCH_TERM" ]]; then
    echo -e "${RED}Error: Search term required${NC}"
    echo "Usage: query-decisions.sh <search_term>"
    echo "Run with --help for more options"
    exit 1
fi

if [[ ! -f "$DECISIONS_FILE" ]]; then
    echo -e "${YELLOW}No decisions recorded yet.${NC}"
    echo ""
    echo "Decisions are recorded when the agent emits <decision> blocks"
    echo "during consequential choices (STANDARD/FULL modes)."
    exit 0
fi

# Search
echo ""
echo -e "${BOLD}Searching for: ${CYAN}$SEARCH_TERM${NC}"
echo -e "${BOLD}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Case-insensitive search
RESULTS=$(grep -i "$SEARCH_TERM" "$DECISIONS_FILE" 2>/dev/null | tail -n "$LIMIT" || true)

if [[ -z "$RESULTS" ]]; then
    echo -e "${YELLOW}No decisions found matching '$SEARCH_TERM'${NC}"
    echo ""
    echo "Try:"
    echo "  - Broader search terms"
    echo "  - Check spelling"
    echo "  - Run trace-view.sh to see recent decisions"
    exit 0
fi

COUNT=$(echo "$RESULTS" | wc -l)
echo -e "Found ${GREEN}$COUNT${NC} matching decision(s):"
echo ""

# Display results
echo "$RESULTS" | while read -r line; do
    ts=$(echo "$line" | grep -oP '"timestamp"\s*:\s*"\K[^"]+' 2>/dev/null || true)
    action=$(echo "$line" | grep -oP '"action"\s*:\s*"\K[^"]+' 2>/dev/null || true)
    context=$(echo "$line" | grep -oP '"context"\s*:\s*"\K[^"]+' 2>/dev/null || true)
    rationale=$(echo "$line" | grep -oP '"rationale"\s*:\s*"\K[^"]+' 2>/dev/null || true)
    alternatives=$(echo "$line" | grep -oP '"alternatives"\s*:\s*"\K[^"]+' 2>/dev/null || true)
    confidence=$(echo "$line" | grep -oP '"confidence"\s*:\s*"\K[^"]+' 2>/dev/null || true)
    verified=$(echo "$line" | grep -oP '"verified"\s*:\s*\K[^,}]+' 2>/dev/null || true)

    # Format date
    date_fmt=$(echo "$ts" | cut -c1-10)
    time_fmt=$(echo "$ts" | sed 's/T/ /' | sed 's/Z//' | cut -c12-19)

    # Status indicator
    if [[ "$verified" == "true" ]]; then
        status="${GREEN}VERIFIED${NC}"
    elif [[ "$verified" == "false" ]]; then
        status="${RED}UNVERIFIED${NC}"
    else
        status="${YELLOW}PENDING${NC}"
    fi

    echo -e "${CYAN}┌─────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${CYAN}│${NC} ${BOLD}$date_fmt $time_fmt${NC}                              $status"
    echo -e "${CYAN}│${NC}"
    echo -e "${CYAN}│${NC} ${BOLD}Action:${NC} $action"

    if [[ "$SHOW_CONTEXT" == "true" ]]; then
        echo -e "${CYAN}│${NC} ${BOLD}Context:${NC} $context"
        echo -e "${CYAN}│${NC} ${BOLD}Rationale:${NC} $rationale"
        if [[ -n "$alternatives" ]]; then
            echo -e "${CYAN}│${NC} ${BOLD}Alternatives:${NC} $alternatives"
        fi
    fi

    if [[ -n "$confidence" ]]; then
        echo -e "${CYAN}│${NC} ${BOLD}Confidence:${NC} $confidence"
    fi
    echo -e "${CYAN}└─────────────────────────────────────────────────────────────┘${NC}"
    echo ""
done

echo -e "${BOLD}Tip:${NC} Use -c/--context to see full decision details"
echo ""
