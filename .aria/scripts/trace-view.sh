#!/bin/bash
# ARIA Trace Viewer - Visualize recent decisions and signals
# Usage: .aria/scripts/trace-view.sh [--last N] [--today] [--session]

set -e

ARIA_DIR=".aria"
STATE_DIR="$ARIA_DIR/state"
DECISIONS_FILE="$STATE_DIR/decisions.jsonl"
SIGNALS_FILE="$STATE_DIR/signals.jsonl"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Defaults
LIMIT=20
MODE="recent"

# Parse args
while [[ $# -gt 0 ]]; do
    case $1 in
        --last|-n)
            LIMIT="$2"
            shift 2
            ;;
        --today)
            MODE="today"
            shift
            ;;
        --session)
            MODE="session"
            shift
            ;;
        --help|-h)
            echo "Usage: trace-view.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --last N, -n N   Show last N entries (default: 20)"
            echo "  --today          Show today's entries only"
            echo "  --session        Show current session (last gap > 30min)"
            echo "  --help, -h       Show this help"
            exit 0
            ;;
        *)
            shift
            ;;
    esac
done

# Header
echo ""
echo -e "${BOLD}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}                    ARIA Decision Trace${NC}"
echo -e "${BOLD}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Check if files exist
if [[ ! -f "$SIGNALS_FILE" ]] && [[ ! -f "$DECISIONS_FILE" ]]; then
    echo -e "${YELLOW}No trace data found.${NC}"
    echo ""
    echo "Traces are captured when:"
    echo "  - Agent emits <decision> blocks (decisions.jsonl)"
    echo "  - Tools are called via hooks (signals.jsonl)"
    echo ""
    exit 0
fi

# Function to format timestamp
format_time() {
    local ts="$1"
    echo "$ts" | sed 's/T/ /' | sed 's/Z//' | cut -c12-19
}

# Show signals
if [[ -f "$SIGNALS_FILE" ]]; then
    echo -e "${CYAN}┌─ SIGNALS (Tool Calls) ────────────────────────────────────────┐${NC}"

    # Filter based on mode
    case $MODE in
        today)
            TODAY=$(date -u +%Y-%m-%d)
            ENTRIES=$(grep "$TODAY" "$SIGNALS_FILE" | tail -n "$LIMIT")
            ;;
        *)
            ENTRIES=$(tail -n "$LIMIT" "$SIGNALS_FILE")
            ;;
    esac

    if [[ -z "$ENTRIES" ]]; then
        echo -e "${YELLOW}│  No signals recorded${NC}"
    else
        echo "$ENTRIES" | while read -r line; do
            ts=$(echo "$line" | grep -oP '"timestamp"\s*:\s*"\K[^"]+' 2>/dev/null || true)
            event=$(echo "$line" | grep -oP '"event"\s*:\s*"\K[^"]+' 2>/dev/null || true)
            tool=$(echo "$line" | grep -oP '"tool"\s*:\s*"\K[^"]+' 2>/dev/null || true)
            file_path=$(echo "$line" | grep -oP '"file_path"\s*:\s*"\K[^"]+' 2>/dev/null || true)
            command=$(echo "$line" | grep -oP '"command"\s*:\s*"\K[^"]+' 2>/dev/null | head -c 50 || true)

            time_fmt=$(format_time "$ts")

            # Color based on event type
            if [[ "$event" == "pre" ]]; then
                event_color="${BLUE}→${NC}"
            else
                event_color="${GREEN}✓${NC}"
            fi

            # Show relevant detail
            detail=""
            if [[ -n "$file_path" ]]; then
                detail="$file_path"
            elif [[ -n "$command" ]]; then
                detail="$command..."
            fi

            printf "${CYAN}│${NC}  %s %s %-12s %s\n" "$time_fmt" "$event_color" "$tool" "$detail"
        done
    fi
    echo -e "${CYAN}└────────────────────────────────────────────────────────────────┘${NC}"
    echo ""
fi

# Show decisions
if [[ -f "$DECISIONS_FILE" ]]; then
    echo -e "${GREEN}┌─ DECISIONS ───────────────────────────────────────────────────┐${NC}"

    ENTRIES=$(tail -n "$LIMIT" "$DECISIONS_FILE" 2>/dev/null || true)

    if [[ -z "$ENTRIES" ]]; then
        echo -e "${YELLOW}│  No decisions recorded${NC}"
        echo -e "${YELLOW}│  Agent emits <decision> blocks for consequential choices${NC}"
    else
        echo "$ENTRIES" | while read -r line; do
            ts=$(echo "$line" | grep -oP '"timestamp"\s*:\s*"\K[^"]+' 2>/dev/null || true)
            action=$(echo "$line" | grep -oP '"action"\s*:\s*"\K[^"]+' 2>/dev/null | head -c 50 || true)
            confidence=$(echo "$line" | grep -oP '"confidence"\s*:\s*"\K[^"]+' 2>/dev/null || true)
            verified=$(echo "$line" | grep -oP '"verified"\s*:\s*\K[^,}]+' 2>/dev/null || true)

            time_fmt=$(format_time "$ts")

            # Color based on verification status
            if [[ "$verified" == "true" ]]; then
                status="${GREEN}✓${NC}"
            elif [[ "$verified" == "false" ]]; then
                status="${RED}✗${NC}"
            else
                status="${YELLOW}?${NC}"
            fi

            # Format confidence
            conf_display=""
            if [[ -n "$confidence" ]]; then
                conf_display="(${confidence})"
            fi

            printf "${GREEN}│${NC}  %s %s %s %s\n" "$time_fmt" "$status" "$action" "$conf_display"
        done
    fi
    echo -e "${GREEN}└────────────────────────────────────────────────────────────────┘${NC}"
    echo ""
fi

# Summary stats
signal_count=0
decision_count=0
verified_count=0

if [[ -f "$SIGNALS_FILE" ]]; then
    signal_count=$(wc -l < "$SIGNALS_FILE" 2>/dev/null || echo 0)
fi
if [[ -f "$DECISIONS_FILE" ]]; then
    decision_count=$(wc -l < "$DECISIONS_FILE" 2>/dev/null || echo 0)
    verified_count=$(grep -c '"verified":true' "$DECISIONS_FILE" 2>/dev/null || echo 0)
fi

echo -e "${BOLD}Summary:${NC} $signal_count signals | $decision_count decisions | $verified_count verified"
echo ""
echo "Commands:"
echo "  trace-view.sh --last 50     # Show more entries"
echo "  query-decisions.sh <term>   # Search decisions"
echo "  reconcile.sh                # Verify claims"
echo ""
