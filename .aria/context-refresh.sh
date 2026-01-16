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
# SESSION RESUMPTION (Issue #24)
# ============================================
# Allows continuation after crashes or context loss.
# Saves persistent session state that can be restored.

RESUME_FILE="$STATE_DIR/resume-session.json"
SESSION_ID_FILE="$STATE_DIR/.current_session_id"

# Save resumable session state
# Called periodically during workflow execution
save_resume_point() {
    local task_id="${1:-unknown}"
    local task_status="${2:-in_progress}"
    local notes="${3:-}"

    # Get current session ID
    local session_id="no-session"
    if [[ -f "$SESSION_ID_FILE" ]]; then
        session_id=$(cat "$SESSION_ID_FILE")
    fi

    # Get current mode
    local mode="STANDARD"
    if type aria_get_mode >/dev/null 2>&1; then
        mode=$(aria_get_mode)
    fi

    # Get plan info
    local plan_id=""
    local plan_title=""
    local total_tasks=0
    local completed_tasks=0
    if [[ -f "$PLAN_FILE" ]]; then
        plan_id=$(jq -r '.id // "unknown"' "$PLAN_FILE" 2>/dev/null || echo "unknown")
        plan_title=$(jq -r '.title // "unknown"' "$PLAN_FILE" 2>/dev/null || echo "unknown")
        total_tasks=$(jq '.tasks | length' "$PLAN_FILE" 2>/dev/null || echo "0")
        completed_tasks=$(jq '[.tasks[] | select(.status == "completed")] | length' "$PLAN_FILE" 2>/dev/null || echo "0")
    fi

    # Get branch info
    local branch
    branch=$(git branch --show-current 2>/dev/null || echo "unknown")

    # Build resume state
    local resume_state
    resume_state=$(cat << EOF
{
    "version": 1,
    "created_at": "$(get_timestamp)",
    "session_id": "$session_id",
    "mode": "$mode",
    "plan": {
        "id": "$plan_id",
        "title": "$plan_title",
        "file": "$PLAN_FILE"
    },
    "progress": {
        "total_tasks": $total_tasks,
        "completed_tasks": $completed_tasks,
        "current_task": "$task_id",
        "task_status": "$task_status"
    },
    "git": {
        "branch": "$branch"
    },
    "notes": "$notes",
    "state_files": {
        "plan": "$PLAN_FILE",
        "progress": "$PROGRESS_FILE",
        "decisions": "$DECISIONS_FILE",
        "checkpoint": "$CHECKPOINT_FILE"
    },
    "resumable": true
}
EOF
)

    # Save atomically
    echo "$resume_state" > "$RESUME_FILE.tmp" && mv "$RESUME_FILE.tmp" "$RESUME_FILE"

    emit_signal "resume_point_saved" "session" "resume" \
        "session_id=$session_id" \
        "task_id=$task_id" \
        "completed=$completed_tasks" \
        "total=$total_tasks"

    echo -e "${GREEN}Resume point saved: task $task_id${NC}"
}

# Check if there's an incomplete session to resume
# Returns: 0 if resumable session exists, 1 otherwise
check_resumable_session() {
    if [[ ! -f "$RESUME_FILE" ]]; then
        return 1
    fi

    # Check if marked as resumable
    local resumable
    resumable=$(jq -r '.resumable // false' "$RESUME_FILE" 2>/dev/null || echo "false")

    if [[ "$resumable" != "true" ]]; then
        return 1
    fi

    # Check if session is actually incomplete
    local task_status
    task_status=$(jq -r '.progress.task_status // "unknown"' "$RESUME_FILE" 2>/dev/null || echo "unknown")

    if [[ "$task_status" == "completed" ]]; then
        # Session was properly completed
        return 1
    fi

    return 0
}

# Show resume prompt
show_resume_prompt() {
    if ! check_resumable_session; then
        echo "No resumable session found."
        return 1
    fi

    echo ""
    echo -e "${YELLOW}════════════════════════════════════════════════════════${NC}"
    echo -e "${YELLOW}       INCOMPLETE SESSION DETECTED                       ${NC}"
    echo -e "${YELLOW}════════════════════════════════════════════════════════${NC}"
    echo ""

    # Show session details
    local created_at
    local session_id
    local mode
    local plan_title
    local completed
    local total
    local current_task
    local branch

    created_at=$(jq -r '.created_at // "unknown"' "$RESUME_FILE")
    session_id=$(jq -r '.session_id // "unknown"' "$RESUME_FILE")
    mode=$(jq -r '.mode // "STANDARD"' "$RESUME_FILE")
    plan_title=$(jq -r '.plan.title // "unknown"' "$RESUME_FILE")
    completed=$(jq -r '.progress.completed_tasks // 0' "$RESUME_FILE")
    total=$(jq -r '.progress.total_tasks // 0' "$RESUME_FILE")
    current_task=$(jq -r '.progress.current_task // "unknown"' "$RESUME_FILE")
    branch=$(jq -r '.git.branch // "unknown"' "$RESUME_FILE")

    echo "  Session:    $session_id"
    echo "  Created:    $created_at"
    echo "  Mode:       $mode"
    echo "  Plan:       $plan_title"
    echo "  Progress:   $completed/$total tasks completed"
    echo "  Current:    $current_task"
    echo "  Branch:     $branch"
    echo ""
    echo "  Options:"
    echo "    [r]esume   - Continue from last checkpoint"
    echo "    [s]tart    - Start fresh (discard incomplete session)"
    echo "    [v]iew     - View session details"
    echo ""

    return 0
}

# Interactive resume flow
do_resume() {
    if ! show_resume_prompt; then
        return 1
    fi

    # If non-interactive, default to resume
    if [[ ! -t 0 ]]; then
        echo "Non-interactive mode: Auto-resuming..."
        emit_signal "session_auto_resumed" "session" "resume" \
            "reason=non_interactive"
        return 0
    fi

    read -r -p "Choice [r/s/v]: " choice
    case "$choice" in
        r|R)
            echo ""
            echo -e "${GREEN}Resuming session...${NC}"
            emit_signal "session_resumed" "session" "resume" \
                "user_choice=resume"

            # Load the checkpoint
            load_checkpoint
            echo ""
            echo "Session resumed. Continue with the current task."
            return 0
            ;;
        s|S)
            echo ""
            echo -e "${YELLOW}Starting fresh session...${NC}"

            # Mark old session as not resumable
            if [[ -f "$RESUME_FILE" ]]; then
                jq '.resumable = false | .ended_reason = "user_discarded"' "$RESUME_FILE" > "$RESUME_FILE.tmp" \
                    && mv "$RESUME_FILE.tmp" "$RESUME_FILE"
            fi

            emit_signal "session_discarded" "session" "resume" \
                "user_choice=start_fresh"

            echo "Old session discarded. Starting fresh."
            return 0
            ;;
        v|V)
            echo ""
            jq '.' "$RESUME_FILE"
            echo ""
            # Recursive call to show prompt again
            do_resume
            ;;
        *)
            echo "Invalid choice. Please enter r, s, or v."
            do_resume
            ;;
    esac
}

# Mark session as completed (call at end of successful workflow)
mark_session_complete() {
    if [[ -f "$RESUME_FILE" ]]; then
        jq '.resumable = false | .progress.task_status = "completed" | .completed_at = "'"$(get_timestamp)"'"' \
            "$RESUME_FILE" > "$RESUME_FILE.tmp" && mv "$RESUME_FILE.tmp" "$RESUME_FILE"

        emit_signal "session_completed" "session" "resume" \
            "session_file=$RESUME_FILE"

        echo -e "${GREEN}Session marked as complete${NC}"
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
        # Session resumption commands (Issue #24)
        resume-save)
            save_resume_point "${1:-unknown}" "${2:-in_progress}" "${3:-}"
            ;;
        resume-check)
            if check_resumable_session; then
                echo "Resumable session found"
                exit 0
            else
                echo "No resumable session"
                exit 1
            fi
            ;;
        resume)
            do_resume
            ;;
        resume-complete)
            mark_session_complete
            ;;
        help|--help|-h)
            echo "ARIA Context Refresh"
            echo ""
            echo "Usage: context-refresh.sh <command> [options]"
            echo ""
            echo "Checkpoint Commands:"
            echo "  save [name]          Save current state as checkpoint"
            echo "  handoff [name]       Generate handoff summary (saves checkpoint first)"
            echo "  list                 List available checkpoints and handoffs"
            echo "  load [name]          Show checkpoint for loading"
            echo "  cleanup [n]          Keep only last N handoffs (default: 3)"
            echo ""
            echo "Session Resumption Commands (Issue #24):"
            echo "  resume-save <task> [status] [notes]"
            echo "                       Save resumable session checkpoint"
            echo "  resume-check         Check if resumable session exists (exit 0/1)"
            echo "  resume               Interactive resume prompt"
            echo "  resume-complete      Mark session as successfully completed"
            echo ""
            echo "Examples:"
            echo "  context-refresh.sh save after_phase_1"
            echo "  context-refresh.sh handoff"
            echo "  context-refresh.sh resume-save task-5 in_progress"
            echo "  context-refresh.sh resume"
            ;;
        *)
            echo "Unknown command: $cmd"
            echo "Run 'context-refresh.sh help' for usage"
            exit 1
            ;;
    esac
}

main "$@"
