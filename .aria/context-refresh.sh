#!/bin/bash
# ARIA Context Refresh
# Saves state and creates handoff summaries for context refresh points
#
# Usage:
#   context-refresh.sh save [checkpoint_name]    - Save current state
#   context-refresh.sh handoff [checkpoint_name] - Generate handoff summary
#   context-refresh.sh list                      - List available checkpoints
#   context-refresh.sh load [checkpoint_name]    - Show checkpoint for loading
#   context-refresh.sh cleanup                   - Keep only last 3 handoffs

# Exit on error, undefined vars, and pipeline failures
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh" || { echo "Failed to load common.sh"; exit 1; }

# Check dependencies
aria_check_deps jq git || exit 1

# Use ARIA_STATE_DIR if set (for testing), otherwise default
STATE_DIR="${ARIA_STATE_DIR:-$SCRIPT_DIR/state}"
HANDOFFS_DIR="${HANDOFFS_DIR:-$STATE_DIR/handoffs}"
CHECKPOINT_FILE="$STATE_DIR/refresh-checkpoint.json"
PLAN_FILE="$STATE_DIR/current-plan.json"
PROGRESS_FILE="$STATE_DIR/progress.json"
DECISIONS_FILE="${ARIA_DECISIONS_FILE:-$STATE_DIR/decisions.jsonl}"

# Colors from common.sh
BLUE="$ARIA_BLUE"
GREEN="$ARIA_GREEN"
YELLOW="$ARIA_YELLOW"
NC="$ARIA_NC"

mkdir -p "$HANDOFFS_DIR"

# ============================================
# HELPER FUNCTIONS
# ============================================

# Get current timestamp
get_timestamp() {
    date -u +"%Y-%m-%dT%H:%M:%SZ"
}

# Generate checkpoint ID
generate_checkpoint_id() {
    local name="${1:-checkpoint}"
    echo "${name}-$(date +%Y%m%d-%H%M%S)"
}

# Get list of recently modified files (from git)
get_modified_files() {
    local since="${1:-1 hour ago}"
    git log --since="$since" --name-only --pretty=format: 2>/dev/null | \
        sort -u | grep -v '^$' | head -20 || true
}

# Extract key decisions from decisions.jsonl (last N)
get_recent_decisions() {
    local count="${1:-5}"
    if [[ -f "$DECISIONS_FILE" ]]; then
        tail -n "$count" "$DECISIONS_FILE" | \
            jq -r '.action // "Unknown action"' 2>/dev/null || true
    fi
}

# Get progress from plan
get_plan_progress() {
    if [[ -f "$PLAN_FILE" ]]; then
        local total=$(jq '.tasks | length' "$PLAN_FILE" 2>/dev/null || echo "0")
        local completed=$(jq '[.tasks[] | select(.status == "completed")] | length' "$PLAN_FILE" 2>/dev/null || echo "0")
        local current=$(jq -r '.tasks[] | select(.status == "in_progress") | .title' "$PLAN_FILE" 2>/dev/null | head -1 || echo "none")
        echo "{\"total\": $total, \"completed\": $completed, \"current\": \"$current\"}"
    else
        echo "{\"total\": 0, \"completed\": 0, \"current\": \"none\"}"
    fi
}

# ============================================
# SAVE CHECKPOINT
# ============================================
save_checkpoint() {
    local checkpoint_name="${1:-$(generate_checkpoint_id)}"
    local timestamp=$(get_timestamp)

    echo -e "${BLUE}Saving checkpoint: $checkpoint_name${NC}"

    # Get current progress
    local progress=$(get_plan_progress)
    local total=$(echo "$progress" | jq '.total')
    local completed=$(echo "$progress" | jq '.completed')
    local current=$(echo "$progress" | jq -r '.current')

    # Get plan ID
    local plan_id="none"
    if [[ -f "$PLAN_FILE" ]]; then
        plan_id=$(jq -r '.id // "unknown"' "$PLAN_FILE")
    fi

    # Get completed task IDs
    local completed_tasks="[]"
    if [[ -f "$PLAN_FILE" ]]; then
        completed_tasks=$(jq '[.tasks[] | select(.status == "completed") | .id | tostring]' "$PLAN_FILE" 2>/dev/null || echo "[]")
    fi

    # Get remaining task IDs
    local remaining_tasks="[]"
    if [[ -f "$PLAN_FILE" ]]; then
        remaining_tasks=$(jq '[.tasks[] | select(.status == "pending") | .id | tostring]' "$PLAN_FILE" 2>/dev/null || echo "[]")
    fi

    # Get recent decisions
    local decisions=()
    while IFS= read -r line; do
        [[ -n "$line" ]] && decisions+=("$line")
    done < <(get_recent_decisions 5)

    # Get modified files
    local files=()
    while IFS= read -r line; do
        [[ -n "$line" ]] && files+=("$line")
    done < <(get_modified_files)

    # Build checkpoint JSON
    local checkpoint=$(jq -n \
        --arg name "$checkpoint_name" \
        --arg timestamp "$timestamp" \
        --arg plan_id "$plan_id" \
        --argjson completed_tasks "$completed_tasks" \
        --arg current_task "$current" \
        --argjson remaining_tasks "$remaining_tasks" \
        --argjson total "$total" \
        --argjson completed "$completed" \
        '{
            "refresh_point": $name,
            "timestamp": $timestamp,
            "plan_id": $plan_id,
            "progress": {
                "completed_tasks": $completed_tasks,
                "current_task": $current_task,
                "remaining_tasks": $remaining_tasks,
                "total": $total,
                "completed": $completed
            },
            "key_decisions": [],
            "files_modified": [],
            "blockers": [],
            "notes": ""
        }')

    # Add decisions array
    local decisions_json=$(printf '%s\n' "${decisions[@]:-}" | jq -R . | jq -s .)
    checkpoint=$(echo "$checkpoint" | jq --argjson d "$decisions_json" '.key_decisions = $d')

    # Add files array
    local files_json=$(printf '%s\n' "${files[@]:-}" | jq -R . | jq -s .)
    checkpoint=$(echo "$checkpoint" | jq --argjson f "$files_json" '.files_modified = $f')

    # Save checkpoint
    echo "$checkpoint" | aria_atomic_write "$CHECKPOINT_FILE"

    # Log signal
    emit_signal "context_checkpoint_saved" "context" "refresh" \
        "checkpoint_name=$checkpoint_name" \
        "tasks_completed=$completed" \
        "tasks_total=$total"

    echo -e "${GREEN}Checkpoint saved to $CHECKPOINT_FILE${NC}"
    echo "  Progress: $completed/$total tasks"
    echo "  Current: $current"
}

# ============================================
# GENERATE HANDOFF
# ============================================
generate_handoff() {
    local checkpoint_name="${1:-handoff}"
    local timestamp=$(date +%Y%m%d-%H%M%S)
    local handoff_file="$HANDOFFS_DIR/handoff-${timestamp}.md"

    echo -e "${BLUE}Generating handoff summary...${NC}"

    # Load checkpoint if exists
    local checkpoint="{}"
    if [[ -f "$CHECKPOINT_FILE" ]]; then
        checkpoint=$(cat "$CHECKPOINT_FILE")
    fi

    # Get progress info
    local progress=$(get_plan_progress)
    local total=$(echo "$progress" | jq '.total')
    local completed=$(echo "$progress" | jq '.completed')
    local current=$(echo "$progress" | jq -r '.current')

    # Get project info from git
    local project_name=$(basename "$(git rev-parse --show-toplevel 2>/dev/null || pwd)")
    local branch=$(git branch --show-current 2>/dev/null || echo "unknown")

    # Get don't touch areas from project-context.md
    local dont_touch=""
    if [[ -f "$SCRIPT_DIR/project-context.md" ]]; then
        dont_touch=$(grep -A 10 "## Don't Touch\|## Protected" "$SCRIPT_DIR/project-context.md" 2>/dev/null | head -10 || true)
    fi

    # Build handoff markdown
    cat > "$handoff_file" << HANDOFF
## Context Handoff

**Generated:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**Checkpoint:** $checkpoint_name

---

### Project
**Name:** $project_name
**Branch:** $branch

---

### Progress
- **Completed:** $completed/$total tasks
- **Current Task:** $current
- **Status:** $([ "$completed" -eq "$total" ] && echo "COMPLETE" || echo "IN PROGRESS")

---

### Key Files Modified
$(echo "$checkpoint" | jq -r '.files_modified[]? // empty' | while read f; do echo "- \`$f\`"; done)

---

### Key Decisions Made
$(echo "$checkpoint" | jq -r '.key_decisions[]? // empty' | nl -w2 -s'. ')

---

### Don't Touch
$dont_touch

---

### State Files
- \`.aria/state/current-plan.json\` - Current plan
- \`.aria/state/refresh-checkpoint.json\` - This checkpoint
- \`.aria/state/progress.json\` - Task progress

---

### Next Action
Continue with: **$current**

---

### Commands
\`\`\`bash
# Verify state
bash .aria/verify.sh

# View plan
cat .aria/state/current-plan.json | jq '.tasks[] | {id, title, status}'

# Resume
# Read this handoff, then continue with current task
\`\`\`
HANDOFF

    # Log signal
    emit_signal "context_handoff_created" "context" "refresh" \
        "handoff_file=$handoff_file" \
        "checkpoint_name=$checkpoint_name"

    echo -e "${GREEN}Handoff saved to $handoff_file${NC}"

    # Cleanup old handoffs
    cleanup_handoffs

    # Print handoff
    echo ""
    cat "$handoff_file"
}

# ============================================
# LIST CHECKPOINTS
# ============================================
list_checkpoints() {
    echo -e "${BLUE}Available checkpoints:${NC}"
    echo ""

    if [[ -f "$CHECKPOINT_FILE" ]]; then
        echo "Current checkpoint:"
        jq -r '"  \(.refresh_point) - \(.timestamp) - \(.progress.completed)/\(.progress.total) tasks"' "$CHECKPOINT_FILE"
        echo ""
    fi

    echo "Handoff files:"
    if ls "$HANDOFFS_DIR"/handoff-*.md 1>/dev/null 2>&1; then
        ls -lt "$HANDOFFS_DIR"/handoff-*.md | head -5 | while read line; do
            echo "  $line"
        done
    else
        echo "  (none)"
    fi
}

# ============================================
# LOAD CHECKPOINT
# ============================================
load_checkpoint() {
    local checkpoint_name="${1:-}"

    if [[ ! -f "$CHECKPOINT_FILE" ]]; then
        echo -e "${YELLOW}No checkpoint file found${NC}"
        return 1
    fi

    echo -e "${BLUE}Loading checkpoint...${NC}"
    echo ""

    # Show checkpoint summary
    jq -r '
        "Checkpoint: \(.refresh_point)",
        "Timestamp: \(.timestamp)",
        "Plan: \(.plan_id)",
        "",
        "Progress:",
        "  Completed: \(.progress.completed)/\(.progress.total) tasks",
        "  Current: \(.progress.current_task)",
        "",
        "Key Decisions:",
        (.key_decisions[]? | "  - \(.)"),
        "",
        "Files Modified:",
        (.files_modified[]? | "  - \(.)")
    ' "$CHECKPOINT_FILE"

    # Log signal
    emit_signal "context_checkpoint_loaded" "context" "refresh" \
        "checkpoint_file=$CHECKPOINT_FILE"
}

# ============================================
# CLEANUP OLD HANDOFFS
# ============================================
cleanup_handoffs() {
    local keep="${1:-3}"

    # Count handoff files
    local count=$(ls -1 "$HANDOFFS_DIR"/handoff-*.md 2>/dev/null | wc -l)

    if [[ "$count" -gt "$keep" ]]; then
        echo -e "${YELLOW}Cleaning up old handoffs (keeping last $keep)...${NC}"
        ls -t "$HANDOFFS_DIR"/handoff-*.md | tail -n +$((keep + 1)) | xargs rm -f
    fi
}

# ============================================
# MAIN
# ============================================
main() {
    local cmd="${1:-help}"
    shift || true

    case "$cmd" in
        save)
            save_checkpoint "$@"
            ;;
        handoff)
            save_checkpoint "${1:-checkpoint}"
            generate_handoff "$@"
            ;;
        list)
            list_checkpoints
            ;;
        load)
            load_checkpoint "$@"
            ;;
        cleanup)
            cleanup_handoffs "${1:-3}"
            ;;
        help|--help|-h)
            echo "ARIA Context Refresh"
            echo ""
            echo "Usage: context-refresh.sh <command> [options]"
            echo ""
            echo "Commands:"
            echo "  save [name]      Save current state as checkpoint"
            echo "  handoff [name]   Generate handoff summary (saves checkpoint first)"
            echo "  list             List available checkpoints and handoffs"
            echo "  load [name]      Show checkpoint for loading"
            echo "  cleanup [n]      Keep only last N handoffs (default: 3)"
            echo ""
            echo "Examples:"
            echo "  context-refresh.sh save after_phase_1"
            echo "  context-refresh.sh handoff"
            echo "  context-refresh.sh list"
            ;;
        *)
            echo "Unknown command: $cmd"
            echo "Run 'context-refresh.sh help' for usage"
            exit 1
            ;;
    esac
}

main "$@"
