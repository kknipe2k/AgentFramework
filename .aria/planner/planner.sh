#!/bin/bash
# ARIA Planning Agent
# Creates and revises plans with HITL approval loop

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$SCRIPT_DIR")"

source "$ARIA_DIR/common.sh" || { echo "Failed to load common.sh"; exit 1; }
aria_check_deps jq || exit 1

# Directories
STATE_DIR="$ARIA_DIR/state"
PLAN_FILE="$STATE_DIR/current-plan.json"
REQUIREMENTS_FILE="$STATE_DIR/requirements.txt"

mkdir -p "$STATE_DIR"

# ============================================================================
# PLAN MANAGEMENT
# ============================================================================

init_plan() {
    local requirements="$1"
    echo "$requirements" > "$REQUIREMENTS_FILE"

    # Create initial plan structure
    cat > "$PLAN_FILE" << EOF
{
  "goal": "",
  "status": "draft",
  "tasks": [],
  "risks": [],
  "questions": [],
  "history": []
}
EOF
    aria_log "INFO" "Initialized new plan from requirements"
}

load_plan() {
    if [[ -f "$PLAN_FILE" ]]; then
        cat "$PLAN_FILE"
    else
        echo "{}"
    fi
}

save_plan() {
    local plan="$1"
    echo "$plan" > "$PLAN_FILE"
}

get_plan_status() {
    if [[ -f "$PLAN_FILE" ]]; then
        jq -r '.status // "none"' "$PLAN_FILE"
    else
        echo "none"
    fi
}

# ============================================================================
# HITL INTEGRATION
# ============================================================================

request_approval() {
    local plan="$1"

    echo ""
    echo "═══════════════════════════════════════════════════════════════"
    echo "                    PLAN REVIEW REQUESTED"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""

    # Display goal
    local goal=$(echo "$plan" | jq -r '.goal')
    echo "GOAL: $goal"
    echo ""

    # Display tasks
    echo "TASKS:"
    echo "$plan" | jq -r '.tasks[] | "  [\(.status)] \(.id). \(.description) (\(.complexity))"'
    echo ""

    # Display risks
    local risks=$(echo "$plan" | jq -r '.risks | length')
    if [[ "$risks" -gt 0 ]]; then
        echo "RISKS:"
        echo "$plan" | jq -r '.risks[] | "  - \(.description)"'
        echo ""
    fi

    # Display questions
    local questions=$(echo "$plan" | jq -r '.questions | length')
    if [[ "$questions" -gt 0 ]]; then
        echo "QUESTIONS (need your input):"
        echo "$plan" | jq -r '.questions[] | "  ? \(.)"'
        echo ""
    fi

    echo "═══════════════════════════════════════════════════════════════"
    echo ""
    echo "Options:"
    echo "  [a]pprove  - Start execution"
    echo "  [r]evise   - Provide feedback for revision"
    echo "  [e]dit     - Edit plan directly (opens in editor)"
    echo "  [c]ancel   - Abort planning"
    echo ""

    while true; do
        read -p "Your choice: " choice
        case "$choice" in
            a|A|approve)
                return 0  # Approved
                ;;
            r|R|revise)
                read -p "Feedback: " feedback
                echo "$feedback"
                return 1  # Needs revision
                ;;
            e|E|edit)
                ${EDITOR:-nano} "$PLAN_FILE"
                echo "EDITED"
                return 2  # Edited directly
                ;;
            c|C|cancel)
                return 3  # Cancelled
                ;;
            *)
                echo "Invalid choice. Use a/r/e/c"
                ;;
        esac
    done
}

# ============================================================================
# PLANNING LOOP
# ============================================================================

run_planning_loop() {
    local requirements="$1"
    local max_iterations=5
    local iteration=0

    init_plan "$requirements"

    echo ""
    aria_log "INFO" "Starting planning loop"
    echo "Requirements: $requirements"
    echo ""

    while [[ $iteration -lt $max_iterations ]]; do
        iteration=$((iteration + 1))
        aria_log "INFO" "Planning iteration $iteration"

        # In real implementation, this would call Claude to generate/revise plan
        # For now, we prompt the user to provide the plan or use a template

        local current_plan=$(load_plan)
        local plan_status=$(echo "$current_plan" | jq -r '.status')

        if [[ "$plan_status" == "draft" || "$plan_status" == "revision_needed" ]]; then
            echo ""
            echo "Plan needs to be created/revised."
            echo "The planning agent would generate a plan here."
            echo ""
            echo "For now, please edit the plan file or provide JSON:"
            echo "Plan file: $PLAN_FILE"
            echo ""
            read -p "Press Enter when plan is ready, or type 'skip' to use example: " input

            if [[ "$input" == "skip" ]]; then
                # Create example plan
                current_plan=$(cat << EOF
{
  "goal": "$requirements",
  "status": "pending_approval",
  "tasks": [
    {
      "id": 1,
      "description": "Analyze requirements and existing code",
      "acceptance": "Understanding documented",
      "complexity": "simple",
      "dependencies": [],
      "status": "pending"
    },
    {
      "id": 2,
      "description": "Implement core functionality",
      "acceptance": "Feature works as specified",
      "complexity": "medium",
      "dependencies": [1],
      "status": "pending"
    },
    {
      "id": 3,
      "description": "Add tests",
      "acceptance": "Tests pass",
      "complexity": "simple",
      "dependencies": [2],
      "status": "pending"
    }
  ],
  "risks": [
    {
      "description": "Requirements may be incomplete",
      "mitigation": "Ask clarifying questions"
    }
  ],
  "questions": [],
  "history": []
}
EOF
)
                save_plan "$current_plan"
            else
                current_plan=$(load_plan)
            fi
        fi

        # Update status to pending approval
        current_plan=$(echo "$current_plan" | jq '.status = "pending_approval"')
        save_plan "$current_plan"

        # Request HITL approval
        request_approval "$current_plan"
        local approval_result=$?

        case $approval_result in
            0)  # Approved
                current_plan=$(echo "$current_plan" | jq '.status = "approved"')
                save_plan "$current_plan"
                aria_log "INFO" "Plan approved!"
                echo ""
                echo "Plan approved. Ready for execution."
                echo "Run: .aria/ralph/ralph.sh start"
                return 0
                ;;
            1)  # Needs revision - feedback captured
                local feedback=$(request_approval "$current_plan" 2>&1 | tail -1)
                current_plan=$(echo "$current_plan" | jq --arg fb "$feedback" '.status = "revision_needed" | .history += [{"action": "revision_requested", "feedback": $fb}]')
                save_plan "$current_plan"
                aria_log "INFO" "Revision requested"
                ;;
            2)  # Edited directly
                aria_log "INFO" "Plan edited directly"
                ;;
            3)  # Cancelled
                aria_log "INFO" "Planning cancelled"
                return 1
                ;;
        esac
    done

    aria_log "ERROR" "Max planning iterations reached"
    return 1
}

# ============================================================================
# REPLAN (called by executor when stuck)
# ============================================================================

replan() {
    local blocker="$1"
    local context="${2:-}"

    aria_log "INFO" "Re-planning due to blocker: $blocker"

    local current_plan=$(load_plan)

    # Add blocker to history
    current_plan=$(echo "$current_plan" | jq --arg b "$blocker" --arg c "$context" \
        '.status = "revision_needed" | .history += [{"action": "escalated", "blocker": $b, "context": $c}]')
    save_plan "$current_plan"

    echo ""
    echo "═══════════════════════════════════════════════════════════════"
    echo "                    EXECUTION BLOCKED"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
    echo "BLOCKER: $blocker"
    [[ -n "$context" ]] && echo "CONTEXT: $context"
    echo ""
    echo "Current plan status:"
    echo "$current_plan" | jq -r '.tasks[] | "  [\(.status)] \(.id). \(.description)"'
    echo ""
    echo "Options:"
    echo "  [r]eplan  - Revise the plan"
    echo "  [s]kip    - Skip blocked task, continue"
    echo "  [a]bort   - Stop execution"
    echo ""

    read -p "Your choice: " choice
    case "$choice" in
        r|R|replan)
            # Mark for revision and restart planning loop
            run_planning_loop "$(cat "$REQUIREMENTS_FILE")"
            ;;
        s|S|skip)
            # Mark current task as skipped
            local current_task=$(echo "$current_plan" | jq -r '.tasks[] | select(.status == "in_progress") | .id')
            if [[ -n "$current_task" ]]; then
                current_plan=$(echo "$current_plan" | jq --arg id "$current_task" \
                    '(.tasks[] | select(.id == ($id | tonumber))) .status = "skipped"')
                current_plan=$(echo "$current_plan" | jq '.status = "approved"')
                save_plan "$current_plan"
                echo "Task $current_task skipped. Execution can continue."
            fi
            ;;
        a|A|abort)
            current_plan=$(echo "$current_plan" | jq '.status = "aborted"')
            save_plan "$current_plan"
            echo "Execution aborted."
            return 1
            ;;
    esac
}

# ============================================================================
# STATUS
# ============================================================================

show_status() {
    if [[ ! -f "$PLAN_FILE" ]]; then
        echo "No active plan."
        return
    fi

    local plan=$(load_plan)
    local status=$(echo "$plan" | jq -r '.status')
    local goal=$(echo "$plan" | jq -r '.goal')
    local total=$(echo "$plan" | jq '.tasks | length')
    local done=$(echo "$plan" | jq '[.tasks[] | select(.status == "done")] | length')
    local in_progress=$(echo "$plan" | jq '[.tasks[] | select(.status == "in_progress")] | length')
    local blocked=$(echo "$plan" | jq '[.tasks[] | select(.status == "blocked")] | length')

    echo ""
    echo "ARIA Plan Status"
    echo "════════════════"
    echo "Goal: $goal"
    echo "Status: $status"
    echo "Progress: $done/$total tasks done"
    [[ "$in_progress" -gt 0 ]] && echo "In Progress: $in_progress"
    [[ "$blocked" -gt 0 ]] && echo "Blocked: $blocked"
    echo ""

    echo "Tasks:"
    echo "$plan" | jq -r '.tasks[] | "  [\(.status | .[0:4])] \(.id). \(.description)"'
}

# ============================================================================
# MAIN
# ============================================================================

usage() {
    cat << EOF
ARIA Planning Agent

Usage: planner.sh <command> [args]

Commands:
  plan <requirements>   Start planning loop with requirements
  replan <blocker>      Re-plan due to execution blocker
  status                Show current plan status
  approve               Mark current plan as approved
  reset                 Clear current plan

Examples:
  planner.sh plan "Add user authentication"
  planner.sh replan "API endpoint doesn't exist"
  planner.sh status
EOF
}

main() {
    local cmd="${1:-}"
    shift || true

    case "$cmd" in
        plan)
            [[ -z "${1:-}" ]] && { echo "Error: requirements required"; usage; exit 1; }
            run_planning_loop "$*"
            ;;
        replan)
            [[ -z "${1:-}" ]] && { echo "Error: blocker description required"; usage; exit 1; }
            replan "$1" "${2:-}"
            ;;
        status)
            show_status
            ;;
        approve)
            local plan=$(load_plan)
            plan=$(echo "$plan" | jq '.status = "approved"')
            save_plan "$plan"
            echo "Plan approved."
            ;;
        reset)
            rm -f "$PLAN_FILE" "$REQUIREMENTS_FILE"
            echo "Plan cleared."
            ;;
        *)
            usage
            exit 1
            ;;
    esac
}

main "$@"
