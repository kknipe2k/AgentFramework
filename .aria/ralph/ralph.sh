#!/bin/bash
# ARIA-RALPH: Autonomous loop with safety rails
# Combines Ralph's fresh-context loop with ARIA's hard blocks

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_DIR="$(dirname "$ARIA_DIR")"

# Configuration
MAX_ITERATIONS=${1:-25}
AGENT=${ARIA_RALPH_AGENT:-"claude"}  # claude, amp, etc.
SLEEP_BETWEEN=${ARIA_RALPH_SLEEP:-5}
MAX_CONSECUTIVE_FAILURES=${ARIA_RALPH_MAX_FAILURES:-3}  # Failures before HITL

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# Files
PRD_FILE="$SCRIPT_DIR/prd.json"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"
PROMPT_FILE="$SCRIPT_DIR/prompt.md"
LEARNINGS_FILE="$SCRIPT_DIR/learnings.md"
HITL_SCRIPT="$ARIA_DIR/hitl.sh"
GIT_OPS_SCRIPT="$ARIA_DIR/git-ops.sh"
MODEL_SELECTOR="$ARIA_DIR/model-selector.sh"

# Track consecutive failures per story
declare -A story_failures

# Track model used for current iteration (for learning)
current_iteration_model=""
current_iteration_task_type=""
current_iteration_complexity=5

# Auto-PR configuration
AUTO_PR=${ARIA_RALPH_AUTO_PR:-true}
CHECKPOINT_EACH_ITERATION=${ARIA_RALPH_CHECKPOINT:-true}

# Model selection configuration
AUTO_MODEL_SELECT=${ARIA_RALPH_AUTO_MODEL:-true}
FORCE_MODEL=${ARIA_RALPH_FORCE_MODEL:-""}  # Set to opus/sonnet/haiku to force

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}          ARIA-RALPH: Autonomous Execution Loop            ${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo "Agent:          $AGENT"
echo "Max iterations: $MAX_ITERATIONS"
echo "PRD:            $PRD_FILE"
echo "Progress:       $PROGRESS_FILE"
echo ""

# Pre-flight checks
preflight_check() {
    echo -e "${YELLOW}Running pre-flight checks...${NC}"

    # Check PRD exists
    if [[ ! -f "$PRD_FILE" ]]; then
        echo -e "${RED}ERROR: prd.json not found${NC}"
        echo "Create it first: aria-ralph init \"Feature description\""
        exit 1
    fi

    # Check prompt exists
    if [[ ! -f "$PROMPT_FILE" ]]; then
        echo -e "${RED}ERROR: prompt.md not found${NC}"
        exit 1
    fi

    # Check we're in a git repo
    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo -e "${RED}ERROR: Not in a git repository${NC}"
        exit 1
    fi

    # Check for uncommitted changes
    if [[ -n $(git status --porcelain) ]]; then
        echo -e "${YELLOW}WARNING: Uncommitted changes detected${NC}"
        echo "Stashing changes..."
        git stash push -m "aria-ralph-pre-run-$(date +%Y%m%d_%H%M%S)"
    fi

    # Check branch
    BRANCH=$(jq -r '.branchName // "aria-ralph/feature"' "$PRD_FILE")
    CURRENT=$(git branch --show-current)

    if [[ "$CURRENT" != "$BRANCH" ]]; then
        echo "Switching to branch: $BRANCH"
        git checkout "$BRANCH" 2>/dev/null || git checkout -b "$BRANCH"
    fi

    echo -e "${GREEN}Pre-flight checks passed${NC}"
    echo ""
}

# Count remaining stories
count_remaining() {
    jq '[.userStories[] | select(.passes == false)] | length' "$PRD_FILE"
}

# Get next story
get_next_story() {
    jq -r '[.userStories[] | select(.passes == false)] | sort_by(.priority) | .[0] | .id // empty' "$PRD_FILE"
}

# Run safety checks (ARIA rails + verification executor)
safety_check() {
    echo -e "${YELLOW}Running ARIA safety rails...${NC}"

    local failures=0
    EXECUTOR="$ARIA_DIR/verify-executor.sh"

    # 1. Check for secrets in staged files
    if git diff --cached --name-only 2>/dev/null | xargs grep -lE "(api[_-]?key|secret|password|token)\s*[=:]\s*['\"][A-Za-z0-9_\-]{10,}['\"]" 2>/dev/null; then
        echo -e "${RED}BLOCKED: Possible secret in staged files${NC}"
        echo "<aria-blocked>SECRET_DETECTED</aria-blocked>"
        failures=$((failures + 1))
    fi

    # 2. Use verification executor if available
    if [[ -x "$EXECUTOR" ]]; then
        echo "Running verification executor..."
        if ! "$EXECUTOR" standard; then
            echo -e "${RED}BLOCKED: Verification failed${NC}"
            echo "<aria-blocked>VERIFICATION_FAILED</aria-blocked>"
            failures=$((failures + 1))
        fi
    else
        # Fallback to basic checks
        if [[ -f "package.json" ]]; then
            if ! npm test --silent 2>/dev/null; then
                echo -e "${RED}BLOCKED: Tests failing${NC}"
                echo "<aria-blocked>TESTS_FAILING</aria-blocked>"
                failures=$((failures + 1))
            fi
        elif [[ -f "pytest.ini" ]] || [[ -f "pyproject.toml" ]]; then
            if ! pytest --quiet 2>/dev/null; then
                echo -e "${RED}BLOCKED: Tests failing${NC}"
                echo "<aria-blocked>TESTS_FAILING</aria-blocked>"
                failures=$((failures + 1))
            fi
        fi
    fi

    if [[ $failures -eq 0 ]]; then
        echo -e "${GREEN}Safety checks passed${NC}"
        return 0
    else
        echo -e "${RED}Safety checks failed ($failures issues)${NC}"
        return 1
    fi
}

# Log iteration result
log_iteration() {
    local iteration=$1
    local story_id=$2
    local status=$3
    local duration=$4

    echo "" >> "$PROGRESS_FILE"
    echo "## $(date '+%Y-%m-%d %H:%M') - Iteration $iteration - $story_id" >> "$PROGRESS_FILE"
    echo "- Status: $status" >> "$PROGRESS_FILE"
    echo "- Duration: ${duration}s" >> "$PROGRESS_FILE"
}

# ============================================
# HUMAN-IN-THE-LOOP INTEGRATION
# ============================================

# Request human help via HITL system
request_human_help() {
    local reason="$1"
    local story_id="$2"
    local context="$3"

    echo ""
    echo -e "${MAGENTA}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${MAGENTA}           🚨 HUMAN INTERVENTION REQUIRED 🚨               ${NC}"
    echo -e "${MAGENTA}═══════════════════════════════════════════════════════════${NC}"
    echo ""

    if [[ -x "$HITL_SCRIPT" ]]; then
        local full_context="Story: $story_id

$context

Recent progress:
$(tail -20 "$PROGRESS_FILE" 2>/dev/null || echo "No progress yet")"

        local response=$("$HITL_SCRIPT" help "$reason" "$full_context")

        if [[ -n "$response" ]]; then
            echo ""
            echo -e "${GREEN}Human guidance received: $response${NC}"

            # Add guidance to progress file
            cat >> "$PROGRESS_FILE" << EOF

## $(date '+%Y-%m-%d %H:%M') - Human Guidance for $story_id
- Reason: $reason
- Guidance: $response
EOF
            echo "$response"
            return 0
        fi
    else
        # Fallback: just wait for Enter key
        echo "Reason: $reason"
        echo "Story: $story_id"
        echo ""
        echo "Press Enter to continue, or Ctrl+C to stop..."
        read -r
    fi

    return 1
}

# Check if we should request human help (too many failures)
check_failure_threshold() {
    local story_id="$1"
    local current_failures=${story_failures[$story_id]:-0}

    if [[ $current_failures -ge $MAX_CONSECUTIVE_FAILURES ]]; then
        return 0  # Should request help
    fi
    return 1  # Keep trying
}

# Increment failure count for a story
increment_failures() {
    local story_id="$1"
    local current=${story_failures[$story_id]:-0}
    story_failures[$story_id]=$((current + 1))
}

# Reset failure count (on success)
reset_failures() {
    local story_id="$1"
    story_failures[$story_id]=0
}

# Record learning outcome for model selection
record_learning_outcome() {
    local story_id="$1"
    local outcome="$2"  # "success" or "fail"

    if [[ -x "$MODEL_SELECTOR" ]] && [[ -n "$current_iteration_model" ]]; then
        "$MODEL_SELECTOR" outcome \
            "$current_iteration_model" \
            "$current_iteration_task_type" \
            "$current_iteration_complexity" \
            "$outcome" \
            "$story_id" >/dev/null 2>&1 || true
    fi
}

# Main loop
run_loop() {
    local iteration=0
    local start_time=$(date +%s)

    while [[ $iteration -lt $MAX_ITERATIONS ]]; do
        iteration=$((iteration + 1))
        local iter_start=$(date +%s)

        echo ""
        echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
        echo -e "${BLUE}                    ITERATION $iteration / $MAX_ITERATIONS${NC}"
        echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"

        # Save checkpoint at start of iteration (for rollback safety)
        if [[ "$CHECKPOINT_EACH_ITERATION" == "true" ]] && [[ -x "$GIT_OPS_SCRIPT" ]]; then
            "$GIT_OPS_SCRIPT" checkpoint "iter_${iteration}" >/dev/null 2>&1 || true
        fi

        # Check remaining stories
        local remaining=$(count_remaining)
        if [[ "$remaining" == "0" ]]; then
            echo -e "${GREEN}All stories complete!${NC}"
            break
        fi
        echo "Remaining stories: $remaining"

        # Get next story
        local next_story=$(get_next_story)
        if [[ -z "$next_story" ]]; then
            echo -e "${GREEN}No more stories to process${NC}"
            break
        fi
        echo "Next story: $next_story"
        echo ""

        # Build the prompt with current context
        local full_prompt=$(cat "$PROMPT_FILE")
        full_prompt="$full_prompt

## Current PRD
\`\`\`json
$(cat "$PRD_FILE")
\`\`\`

## Progress So Far
\`\`\`
$(tail -100 "$PROGRESS_FILE" 2>/dev/null || echo "No progress yet")
\`\`\`

## Current Story: $next_story
"

        # Select model for this task
        local selected_model="sonnet"
        local model_flag=""
        local failures=${story_failures[$next_story]:-0}

        if [[ "$AUTO_MODEL_SELECT" == "true" ]] && [[ -x "$MODEL_SELECTOR" ]]; then
            local story_title=$(jq -r ".userStories[] | select(.id == \"$next_story\") | .title" "$PRD_FILE" 2>/dev/null || echo "")
            selected_model=$("$MODEL_SELECTOR" select "$story_title" "$next_story" "$FORCE_MODEL" "$failures")
            model_flag=$("$MODEL_SELECTOR" flag "$story_title" "$next_story" "$FORCE_MODEL" "$failures")

            # Capture info for learning feedback
            current_iteration_model="$selected_model"
            current_iteration_complexity=$("$MODEL_SELECTOR" complexity "$story_title" 2>/dev/null || echo "5")
            # Determine task type from story title
            if echo "$story_title" | grep -qiE "test|spec|coverage"; then current_iteration_task_type="testing"
            elif echo "$story_title" | grep -qiE "doc|readme|comment"; then current_iteration_task_type="documentation"
            elif echo "$story_title" | grep -qiE "fix|bug|error|issue"; then current_iteration_task_type="bugfix"
            elif echo "$story_title" | grep -qiE "refactor|clean|simplify"; then current_iteration_task_type="refactoring"
            elif echo "$story_title" | grep -qiE "feature|add|implement|create"; then current_iteration_task_type="feature"
            elif echo "$story_title" | grep -qiE "setup|config|init"; then current_iteration_task_type="setup"
            else current_iteration_task_type="general"
            fi

            echo -e "${BLUE}Model: $selected_model${NC} (type: $current_iteration_task_type, complexity: $current_iteration_complexity, failures: $failures)"
        elif [[ -n "$FORCE_MODEL" ]]; then
            selected_model="$FORCE_MODEL"
            current_iteration_model="$FORCE_MODEL"
            current_iteration_task_type="general"
            current_iteration_complexity=5
            echo -e "${BLUE}Model: $selected_model (forced)${NC}"
        fi

        # Run the agent
        echo -e "${YELLOW}Running agent...${NC}"
        local output=""
        local input_tokens=0
        local output_tokens=0

        case "$AGENT" in
            "claude")
                output=$(echo "$full_prompt" | claude --dangerously-skip-permissions -p $model_flag 2>&1 | tee /dev/stderr) || true
                # Estimate tokens (rough: 4 chars = 1 token)
                input_tokens=$(( ${#full_prompt} / 4 ))
                output_tokens=$(( ${#output} / 4 ))
                ;;
            "amp")
                output=$(echo "$full_prompt" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
                input_tokens=$(( ${#full_prompt} / 4 ))
                output_tokens=$(( ${#output} / 4 ))
                ;;
            *)
                echo -e "${RED}Unknown agent: $AGENT${NC}"
                exit 1
                ;;
        esac

        # Record token usage
        if [[ -x "$MODEL_SELECTOR" ]]; then
            "$MODEL_SELECTOR" record "$selected_model" "$input_tokens" "$output_tokens" "$next_story" >/dev/null 2>&1 || true
        fi

        local iter_end=$(date +%s)
        local duration=$((iter_end - iter_start))

        # Check for completion signal
        if echo "$output" | grep -q "<aria-complete>"; then
            echo -e "${GREEN}✅ All tasks complete!${NC}"
            log_iteration $iteration "ALL" "COMPLETE" $duration
            break
        fi

        # Check for help signal (agent explicitly requesting human)
        if echo "$output" | grep -q "<aria-help>"; then
            local help_reason=$(echo "$output" | grep -oP '<aria-help>\K[^<]+' || echo "Agent requested help")
            echo -e "${MAGENTA}🆘 Agent requesting human help${NC}"
            log_iteration $iteration "$next_story" "HELP_REQUESTED" $duration

            # Request human help
            request_human_help "$help_reason" "$next_story" "Agent explicitly requested assistance"
            reset_failures "$next_story"
            continue
        fi

        # Check for blocked signal (ARIA rail triggered)
        if echo "$output" | grep -q "<aria-blocked>"; then
            local block_reason=$(echo "$output" | grep -oP '<aria-blocked>\K[^<]+' || echo "Unknown")
            echo -e "${RED}🚫 ARIA rail blocked execution: $block_reason${NC}"
            log_iteration $iteration "$next_story" "BLOCKED:$block_reason" $duration

            # Record learning outcome: FAIL due to block
            record_learning_outcome "$next_story" "fail"

            # Increment failure count
            increment_failures "$next_story"

            # Check if we've hit the failure threshold
            if check_failure_threshold "$next_story"; then
                echo -e "${MAGENTA}Story $next_story has failed $MAX_CONSECUTIVE_FAILURES times${NC}"
                request_human_help "Story blocked repeatedly: $block_reason" "$next_story" "Block reason: $block_reason"
                reset_failures "$next_story"
            fi
        else
            # Check if the story is now complete (passed)
            local story_passed=$(jq -r ".userStories[] | select(.id == \"$next_story\") | .passes" "$PRD_FILE" 2>/dev/null || echo "false")
            if [[ "$story_passed" == "true" ]]; then
                # Record learning outcome: SUCCESS - story completed
                record_learning_outcome "$next_story" "success"
                echo -e "${GREEN}✅ Story $next_story marked as complete${NC}"
            fi

            # Successful iteration - reset failures
            reset_failures "$next_story"
            log_iteration $iteration "$next_story" "ATTEMPTED" $duration
        fi

        # Safety check before continuing
        if ! safety_check; then
            echo -e "${YELLOW}Safety check failed, will retry next iteration${NC}"

            # Record learning outcome: FAIL due to safety check
            record_learning_outcome "$next_story" "fail"

            increment_failures "$next_story"

            if check_failure_threshold "$next_story"; then
                request_human_help "Safety checks keep failing" "$next_story" "Verification or tests failing repeatedly"
                reset_failures "$next_story"
            fi
        fi

        # Sleep between iterations
        echo ""
        echo "Sleeping ${SLEEP_BETWEEN}s before next iteration..."
        sleep $SLEEP_BETWEEN

    done

    local end_time=$(date +%s)
    local total_duration=$((end_time - start_time))

    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}                    EXECUTION COMPLETE                      ${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Total iterations: $iteration"
    echo "Total duration:   ${total_duration}s"
    echo "Remaining stories: $(count_remaining)"
    echo ""

    # Show final status
    echo "Story status:"
    jq -r '.userStories[] | "  \(.id): \(if .passes then "✅" else "❌" end) \(.title)"' "$PRD_FILE"

    # Auto-create PR if all stories complete
    local remaining=$(count_remaining)
    if [[ "$remaining" == "0" ]] && [[ "$AUTO_PR" == "true" ]] && [[ -x "$GIT_OPS_SCRIPT" ]]; then
        echo ""
        echo -e "${GREEN}All stories complete - creating Pull Request...${NC}"

        # Save final checkpoint
        "$GIT_OPS_SCRIPT" checkpoint "complete" >/dev/null 2>&1 || true

        # Create PR
        local pr_url=$("$GIT_OPS_SCRIPT" pr create 2>&1)
        if [[ $? -eq 0 ]]; then
            echo -e "${GREEN}PR created: $pr_url${NC}"

            # Add to progress
            cat >> "$PROGRESS_FILE" << EOF

## $(date '+%Y-%m-%d %H:%M') - Feature Complete
- All stories passed
- Total iterations: $iteration
- Duration: ${total_duration}s
- PR: $pr_url
EOF
        else
            echo -e "${YELLOW}Could not auto-create PR. Create manually with: aria pr create${NC}"
        fi
    elif [[ "$remaining" != "0" ]]; then
        echo ""
        echo -e "${YELLOW}$remaining stories incomplete. No PR created.${NC}"
        echo "Resume with: ./.aria/ralph/ralph.sh run"
    fi
}

# Initialize new PRD
init_prd() {
    local description="$1"

    if [[ -z "$description" ]]; then
        echo "Usage: aria-ralph init \"Feature description\""
        exit 1
    fi

    # Create initial progress file
    cat > "$PROGRESS_FILE" << EOF
# ARIA-RALPH Progress Log
Started: $(date '+%Y-%m-%d %H:%M')
Feature: $description

## Codebase Patterns
(Learnings will be added here as Ralph discovers them)

---
EOF

    # Create PRD template
    cat > "$PRD_FILE" << EOF
{
  "feature": "$description",
  "branchName": "aria-ralph/$(echo "$description" | tr '[:upper:] ' '[:lower:]-' | tr -cd 'a-z0-9-' | head -c 30)",
  "createdAt": "$(date -Iseconds)",
  "userStories": [
    {
      "id": "US-001",
      "title": "First user story",
      "description": "Describe what needs to be done",
      "acceptanceCriteria": [
        "Criterion 1",
        "Criterion 2",
        "Tests pass",
        "No linting errors"
      ],
      "priority": 1,
      "passes": false,
      "notes": ""
    }
  ]
}
EOF

    echo -e "${GREEN}Initialized ARIA-RALPH${NC}"
    echo ""
    echo "Edit the PRD: $PRD_FILE"
    echo "Add user stories with acceptance criteria"
    echo ""
    echo "Then run: aria-ralph run"
}

# Status check
status() {
    if [[ ! -f "$PRD_FILE" ]]; then
        echo "Not initialized. Run: aria-ralph init \"description\""
        exit 1
    fi

    echo -e "${BLUE}ARIA-RALPH Status${NC}"
    echo ""
    echo "Feature: $(jq -r '.feature' "$PRD_FILE")"
    echo "Branch:  $(jq -r '.branchName' "$PRD_FILE")"
    echo ""
    echo "Stories:"
    jq -r '.userStories[] | "  [\(if .passes then "✅" else "  " end)] \(.id) (P\(.priority)): \(.title)"' "$PRD_FILE"
    echo ""
    echo "Progress: $(jq '[.userStories[] | select(.passes == true)] | length' "$PRD_FILE") / $(jq '.userStories | length' "$PRD_FILE") complete"
}

# Main command handler
case "${1:-help}" in
    "run")
        preflight_check
        run_loop
        ;;
    "init")
        init_prd "$2"
        ;;
    "status")
        status
        ;;
    "help"|*)
        echo "ARIA-RALPH: Autonomous execution with safety rails"
        echo ""
        echo "Commands:"
        echo "  aria-ralph init \"description\"  - Initialize new feature"
        echo "  aria-ralph run [max_iterations] - Run the loop"
        echo "  aria-ralph status               - Show current status"
        echo ""
        echo "Environment:"
        echo "  ARIA_RALPH_AGENT  - Agent to use (claude, amp)"
        echo "  ARIA_RALPH_SLEEP  - Seconds between iterations"
        ;;
esac
