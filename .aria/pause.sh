#!/bin/bash
# ARIA Pause/Resume Control
# Allows user to pause execution and resume later

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STATE_DIR="$SCRIPT_DIR/state"
PAUSE_FILE="$STATE_DIR/paused"
STATUS_FILE="$STATE_DIR/execution-status.json"

mkdir -p "$STATE_DIR"

# Colors
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

pause_execution() {
    touch "$PAUSE_FILE"
    echo -e "${YELLOW}⏸️  Pause signal sent.${NC}"
    echo "Execution will pause at the next safe point."
    echo ""
    echo "To resume: .aria/pause.sh resume"
    echo "To check:  .aria/pause.sh status"
}

resume_execution() {
    if [[ -f "$PAUSE_FILE" ]]; then
        rm "$PAUSE_FILE"
        echo -e "${GREEN}▶️  Resume signal sent.${NC}"
        echo "Execution will continue."
    else
        echo "Not currently paused."
    fi
}

check_status() {
    echo -e "${CYAN}ARIA Execution Status${NC}"
    echo "═══════════════════════"

    if [[ -f "$PAUSE_FILE" ]]; then
        echo "State: PAUSED"
    else
        echo "State: Running (or not started)"
    fi

    if [[ -f "$STATUS_FILE" ]]; then
        echo ""
        echo "Last known position:"
        jq -r '"  Task: \(.current_task // "none")\n  Story: \(.current_story // "none")\n  Iteration: \(.iteration // 0)\n  Updated: \(.timestamp // "unknown")"' "$STATUS_FILE" 2>/dev/null || echo "  (status file unreadable)"
    fi

    if [[ -f "$SCRIPT_DIR/ralph/progress.txt" ]]; then
        echo ""
        echo "Recent progress:"
        tail -5 "$SCRIPT_DIR/ralph/progress.txt" 2>/dev/null | sed 's/^/  /'
    fi
}

save_status() {
    local task="$1"
    local story="$2"
    local iteration="$3"

    cat > "$STATUS_FILE" << EOF
{
    "current_task": "$task",
    "current_story": "$story",
    "iteration": $iteration,
    "timestamp": "$(date -Iseconds)",
    "paused": $([ -f "$PAUSE_FILE" ] && echo "true" || echo "false")
}
EOF
}

is_paused() {
    [[ -f "$PAUSE_FILE" ]]
}

wait_if_paused() {
    if is_paused; then
        echo ""
        echo -e "${YELLOW}⏸️  PAUSED - Waiting for resume signal...${NC}"
        echo "Run: .aria/pause.sh resume"
        echo "Or:  .aria/pause.sh status"
        echo ""

        while is_paused; do
            sleep 2
        done

        echo -e "${GREEN}▶️  Resumed${NC}"
    fi
}

usage() {
    cat << EOF
ARIA Pause/Resume Control

Usage: pause.sh <command>

Commands:
  pause     Send pause signal (execution pauses at next safe point)
  resume    Resume paused execution
  status    Show current execution status

The pause is "cooperative" - execution pauses at the next safe point
(between tasks), not immediately mid-operation.

Examples:
  .aria/pause.sh pause    # Pause execution
  .aria/pause.sh status   # Where are we?
  .aria/pause.sh resume   # Continue
EOF
}

case "${1:-}" in
    pause)
        pause_execution
        ;;
    resume)
        resume_execution
        ;;
    status)
        check_status
        ;;
    is_paused)
        is_paused && exit 0 || exit 1
        ;;
    wait)
        wait_if_paused
        ;;
    save)
        save_status "$2" "$3" "$4"
        ;;
    *)
        usage
        ;;
esac
