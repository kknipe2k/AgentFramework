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

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Files
PRD_FILE="$SCRIPT_DIR/prd.json"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"
PROMPT_FILE="$SCRIPT_DIR/prompt.md"
LEARNINGS_FILE="$SCRIPT_DIR/learnings.md"

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

# Run safety checks (ARIA rails)
safety_check() {
    echo -e "${YELLOW}Running ARIA safety rails...${NC}"

    # Check for secrets in staged files
    if git diff --cached --name-only | xargs grep -lE "(api[_-]?key|secret|password|token)\s*[=:]\s*['\"][A-Za-z0-9_\-]{10,}['\"]" 2>/dev/null; then
        echo -e "${RED}BLOCKED: Possible secret in staged files${NC}"
        return 1
    fi

    # Check tests pass
    if [[ -f "package.json" ]]; then
        if ! npm test --silent 2>/dev/null; then
            echo -e "${RED}BLOCKED: Tests failing${NC}"
            return 1
        fi
    elif [[ -f "pytest.ini" ]] || [[ -f "pyproject.toml" ]]; then
        if ! pytest --quiet 2>/dev/null; then
            echo -e "${RED}BLOCKED: Tests failing${NC}"
            return 1
        fi
    fi

    echo -e "${GREEN}Safety checks passed${NC}"
    return 0
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

        # Run the agent
        echo -e "${YELLOW}Running agent...${NC}"
        local output=""

        case "$AGENT" in
            "claude")
                output=$(echo "$full_prompt" | claude --dangerously-skip-permissions -p 2>&1 | tee /dev/stderr) || true
                ;;
            "amp")
                output=$(echo "$full_prompt" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
                ;;
            *)
                echo -e "${RED}Unknown agent: $AGENT${NC}"
                exit 1
                ;;
        esac

        local iter_end=$(date +%s)
        local duration=$((iter_end - iter_start))

        # Check for completion signal
        if echo "$output" | grep -q "<aria-complete>"; then
            echo -e "${GREEN}✅ All tasks complete!${NC}"
            log_iteration $iteration "ALL" "COMPLETE" $duration
            break
        fi

        # Check for blocked signal (ARIA rail triggered)
        if echo "$output" | grep -q "<aria-blocked>"; then
            echo -e "${RED}🚫 ARIA rail blocked execution${NC}"
            log_iteration $iteration "$next_story" "BLOCKED" $duration
            # Don't exit, try next iteration (might be a different story)
        else
            log_iteration $iteration "$next_story" "ATTEMPTED" $duration
        fi

        # Safety check before continuing
        if ! safety_check; then
            echo -e "${YELLOW}Safety check failed, will retry next iteration${NC}"
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
