#!/bin/bash
# ARIA Design Notes - AI Transparency System
# Writes AI reasoning to design-notes.md for human review

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NOTES_FILE="$SCRIPT_DIR/design-notes.md"
HITL_SCRIPT="$SCRIPT_DIR/hitl.sh"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Ensure notes file exists
init_notes() {
    if [[ ! -f "$NOTES_FILE" ]]; then
        cat > "$NOTES_FILE" << 'EOF'
# Design Notes

This document captures AI reasoning, research, and design decisions.

---

## Current Session

EOF
    fi
}

# Write a timestamped entry
write_entry() {
    local type="$1"
    local title="$2"
    local content="$3"

    init_notes

    local timestamp=$(date '+%Y-%m-%d %H:%M')

    cat >> "$NOTES_FILE" << EOF

### [$type] $title
*$timestamp*

$content

---
EOF
}

# Write a checkpoint - pauses for review
checkpoint() {
    local title="$1"
    local content="$2"

    write_entry "CHECKPOINT" "$title" "$content"

    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}              DESIGN CHECKPOINT: $title${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "$content"
    echo ""
    echo -e "${YELLOW}Review design-notes.md for full context${NC}"
    echo ""
    echo "Options:"
    echo "  [c]ontinue  - Proceed with current approach"
    echo "  [r]evise    - Provide feedback"
    echo "  [v]iew      - Open design-notes.md"
    echo "  [s]top      - Halt execution"
    echo ""

    while true; do
        read -p "Your choice: " choice
        case "$choice" in
            c|C|continue)
                write_entry "DECISION" "Checkpoint: $title" "User approved. Proceeding."
                return 0
                ;;
            r|R|revise)
                read -p "Your feedback: " feedback
                write_entry "FEEDBACK" "User feedback on: $title" "$feedback"
                echo "$feedback"
                return 1  # Signal revision needed
                ;;
            v|V|view)
                ${EDITOR:-less} "$NOTES_FILE"
                ;;
            s|S|stop)
                write_entry "DECISION" "Checkpoint: $title" "User halted execution."
                return 2  # Signal stop
                ;;
            *)
                echo "Invalid choice. Use c/r/v/s"
                ;;
        esac
    done
}

# Write a concern - softer than blocked, flags for attention
concern() {
    local title="$1"
    local content="$2"
    local severity="${3:-medium}"  # low, medium, high

    write_entry "CONCERN" "$title (severity: $severity)" "$content"

    echo ""
    echo -e "${YELLOW}⚠️  DESIGN CONCERN: $title${NC}"
    echo -e "$content"
    echo ""

    # High severity concerns pause for review
    if [[ "$severity" == "high" ]]; then
        echo -e "${YELLOW}This is a high-severity concern. Pausing for review.${NC}"
        echo ""
        echo "Options:"
        echo "  [a]cknowledge - Note it and continue"
        echo "  [r]evise      - Provide guidance"
        echo "  [s]top        - Halt execution"
        echo ""

        while true; do
            read -p "Your choice: " choice
            case "$choice" in
                a|A|acknowledge)
                    write_entry "DECISION" "Concern acknowledged: $title" "User acknowledged and chose to continue."
                    return 0
                    ;;
                r|R|revise)
                    read -p "Your guidance: " feedback
                    write_entry "FEEDBACK" "User guidance on: $title" "$feedback"
                    echo "$feedback"
                    return 1
                    ;;
                s|S|stop)
                    return 2
                    ;;
                *)
                    echo "Invalid choice. Use a/r/s"
                    ;;
            esac
        done
    fi

    return 0  # Low/medium concerns just log and continue
}

# Write an assumption
assumption() {
    local title="$1"
    local content="$2"

    write_entry "ASSUMPTION" "$title" "$content"

    echo -e "${BLUE}📝 Assumption: $title${NC}"
}

# Write research findings
research() {
    local topic="$1"
    local findings="$2"
    local sources="${3:-}"

    local content="$findings"
    if [[ -n "$sources" ]]; then
        content="$content

**Sources:** $sources"
    fi

    write_entry "RESEARCH" "$topic" "$content"

    echo -e "${GREEN}🔍 Research: $topic${NC}"
}

# Write a decision with alternatives
decision() {
    local title="$1"
    local chosen="$2"
    local alternatives="$3"
    local reasoning="$4"

    local content="**Chosen:** $chosen

**Alternatives considered:**
$alternatives

**Reasoning:** $reasoning"

    write_entry "DECISION" "$title" "$content"

    echo -e "${MAGENTA}✓ Decision: $title → $chosen${NC}"
}

# Show recent notes
show_recent() {
    local lines="${1:-50}"

    if [[ -f "$NOTES_FILE" ]]; then
        tail -n "$lines" "$NOTES_FILE"
    else
        echo "No design notes yet."
    fi
}

# Clear notes (start fresh session)
clear_notes() {
    cat > "$NOTES_FILE" << 'EOF'
# Design Notes

This document captures AI reasoning, research, and design decisions.

---

## Current Session

EOF
    echo "Design notes cleared."
}

# Main
usage() {
    cat << EOF
ARIA Design Notes - AI Transparency System

Usage: design-notes.sh <command> [args]

Commands:
  checkpoint <title> <content>     Pause for design review
  concern <title> <content> [sev]  Flag a concern (severity: low/medium/high)
  assumption <title> <content>     Log an assumption
  research <topic> <findings>      Log research findings
  decision <title> <chosen> <alts> <reason>  Log a decision
  show [lines]                     Show recent notes
  clear                            Start fresh session

Examples:
  design-notes.sh checkpoint "Auth Design" "Using JWT. Review before proceeding."
  design-notes.sh concern "No Tests" "Found no existing tests" high
  design-notes.sh assumption "DB Schema" "Assuming PostgreSQL"
EOF
}

main() {
    local cmd="${1:-}"
    shift || true

    case "$cmd" in
        checkpoint)
            checkpoint "$1" "$2"
            ;;
        concern)
            concern "$1" "$2" "${3:-medium}"
            ;;
        assumption)
            assumption "$1" "$2"
            ;;
        research)
            research "$1" "$2" "${3:-}"
            ;;
        decision)
            decision "$1" "$2" "$3" "$4"
            ;;
        show)
            show_recent "${1:-50}"
            ;;
        clear)
            clear_notes
            ;;
        *)
            usage
            exit 1
            ;;
    esac
}

main "$@"
