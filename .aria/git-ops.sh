#!/bin/bash
# ARIA Git Operations
# Rollback mechanism and auto-PR creation for safe autonomous operation

# Exit on error, undefined vars, and pipeline failures
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh" || { echo "Failed to load common.sh"; exit 1; }

# Check dependencies
aria_check_deps git jq || exit 1

ARIA_DIR="$SCRIPT_DIR"
STATE_DIR="$ARIA_DIR/state"
RALPH_DIR="$ARIA_DIR/ralph"
PRD_FILE="$RALPH_DIR/prd.json"
PROGRESS_FILE="$RALPH_DIR/progress.txt"
LOGS_DIR="$ARIA_DIR/logs"
CHECKPOINTS_DIR="$ARIA_DIR/checkpoints"

# Colors from common.sh
RED="$ARIA_RED"
GREEN="$ARIA_GREEN"
YELLOW="$ARIA_YELLOW"
BLUE="$ARIA_BLUE"
NC="$ARIA_NC"

mkdir -p "$STATE_DIR" "$LOGS_DIR" "$CHECKPOINTS_DIR"

# ============================================
# CHECKPOINT MANAGEMENT
# ============================================

# Save current state as checkpoint
save_checkpoint() {
    local name="${1:-auto}"
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local checkpoint_id="${timestamp}_${name}"
    local checkpoint_dir="$CHECKPOINTS_DIR/$checkpoint_id"

    mkdir -p "$checkpoint_dir"

    # Save git state
    local current_branch=$(git branch --show-current 2>/dev/null || echo "unknown")
    local current_commit=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
    local has_changes=$(git status --porcelain 2>/dev/null | wc -l)

    cat > "$checkpoint_dir/git_state.json" << EOF
{
    "checkpoint_id": "$checkpoint_id",
    "timestamp": "$(date -Iseconds)",
    "branch": "$current_branch",
    "commit": "$current_commit",
    "uncommitted_changes": $has_changes,
    "name": "$name"
}
EOF

    # Save uncommitted changes as patch
    if [[ $has_changes -gt 0 ]]; then
        git diff > "$checkpoint_dir/uncommitted.patch" 2>/dev/null || true
        git diff --cached > "$checkpoint_dir/staged.patch" 2>/dev/null || true
    fi

    # Save ARIA state
    if [[ -d "$STATE_DIR" ]]; then
        cp -r "$STATE_DIR" "$checkpoint_dir/aria_state" 2>/dev/null || true
    fi

    # Save PRD state
    if [[ -f "$PRD_FILE" ]]; then
        cp "$PRD_FILE" "$checkpoint_dir/prd.json" 2>/dev/null || true
    fi

    # Log checkpoint
    echo "[$(date -Iseconds)] CHECKPOINT: $checkpoint_id" >> "$LOGS_DIR/checkpoints.log"

    echo "$checkpoint_id"
}

# List available checkpoints
list_checkpoints() {
    echo ""
    echo -e "${BLUE}Available Checkpoints:${NC}"
    echo ""

    if [[ ! -d "$CHECKPOINTS_DIR" ]] || [[ -z "$(ls -A "$CHECKPOINTS_DIR" 2>/dev/null)" ]]; then
        echo "  No checkpoints found"
        echo ""
        return
    fi

    printf "  %-25s %-15s %-10s %s\n" "ID" "BRANCH" "CHANGES" "COMMIT"
    echo "  ────────────────────────────────────────────────────────────────"

    for checkpoint in "$CHECKPOINTS_DIR"/*/; do
        if [[ -f "$checkpoint/git_state.json" ]]; then
            local id=$(basename "$checkpoint")
            local branch=$(grep '"branch"' "$checkpoint/git_state.json" | cut -d'"' -f4)
            local commit=$(grep '"commit"' "$checkpoint/git_state.json" | cut -d'"' -f4 | cut -c1-7)
            local changes=$(grep '"uncommitted_changes"' "$checkpoint/git_state.json" | grep -oE '[0-9]+')

            printf "  %-25s %-15s %-10s %s\n" "$id" "$branch" "$changes" "$commit"
        fi
    done
    echo ""
}

# ============================================
# ROLLBACK OPERATIONS
# ============================================

# Rollback to a specific checkpoint
rollback_to_checkpoint() {
    local checkpoint_id="$1"
    local checkpoint_dir="$CHECKPOINTS_DIR/$checkpoint_id"

    if [[ ! -d "$checkpoint_dir" ]]; then
        echo -e "${RED}Checkpoint not found: $checkpoint_id${NC}"
        list_checkpoints
        return 1
    fi

    echo -e "${YELLOW}Rolling back to checkpoint: $checkpoint_id${NC}"

    # Read checkpoint state
    local target_commit=$(grep '"commit"' "$checkpoint_dir/git_state.json" | cut -d'"' -f4)
    local target_branch=$(grep '"branch"' "$checkpoint_dir/git_state.json" | cut -d'"' -f4)

    # Save current state first (in case we need to undo the rollback)
    echo "Saving current state before rollback..."
    save_checkpoint "pre_rollback"

    # Discard uncommitted changes
    if [[ -n "$(git status --porcelain 2>/dev/null)" ]]; then
        echo "Discarding uncommitted changes..."
        git checkout -- . 2>/dev/null || true
        git clean -fd 2>/dev/null || true
    fi

    # Reset to target commit
    echo "Resetting to commit: $target_commit"
    git reset --hard "$target_commit" 2>/dev/null || {
        echo -e "${RED}Failed to reset to commit${NC}"
        return 1
    }

    # Restore ARIA state
    if [[ -d "$checkpoint_dir/aria_state" ]]; then
        echo "Restoring ARIA state..."
        rm -rf "$STATE_DIR"
        cp -r "$checkpoint_dir/aria_state" "$STATE_DIR"
    fi

    # Restore PRD if present
    if [[ -f "$checkpoint_dir/prd.json" ]]; then
        echo "Restoring PRD state..."
        cp "$checkpoint_dir/prd.json" "$PRD_FILE"
    fi

    echo -e "${GREEN}Rollback complete${NC}"
    echo ""
    echo "Current state:"
    echo "  Branch: $(git branch --show-current)"
    echo "  Commit: $(git rev-parse --short HEAD)"
}

# Rollback last N commits
rollback_commits() {
    local count="${1:-1}"

    echo -e "${YELLOW}Rolling back $count commit(s)...${NC}"

    # Save checkpoint first
    local checkpoint=$(save_checkpoint "pre_rollback_${count}")
    echo "Saved checkpoint: $checkpoint"

    # Check for uncommitted changes
    if [[ -n "$(git status --porcelain 2>/dev/null)" ]]; then
        echo -e "${YELLOW}Warning: Uncommitted changes will be preserved${NC}"
        git stash push -m "aria-rollback-$(date +%s)" 2>/dev/null || true
    fi

    # Soft reset to keep changes as uncommitted
    git reset --soft HEAD~${count} 2>/dev/null || {
        echo -e "${RED}Failed to rollback commits${NC}"
        return 1
    }

    echo -e "${GREEN}Rolled back $count commit(s)${NC}"
    echo ""
    echo "Changes are now uncommitted. Review with: git status"
    echo "To restore: aria rollback restore $checkpoint"
}

# Rollback to last successful state (before failures)
rollback_to_success() {
    echo -e "${YELLOW}Finding last successful state...${NC}"

    # Look for commits with "feat:" or "fix:" that aren't reverts
    local last_good=$(git log --oneline --grep="^feat:\|^fix:" --invert-grep="BLOCKED\|FAILED" -1 --format="%H" 2>/dev/null)

    if [[ -z "$last_good" ]]; then
        # Fall back to last commit before any ARIA activity
        last_good=$(git log --oneline -10 2>/dev/null | grep -v "aria\|ARIA" | head -1 | cut -d' ' -f1)
    fi

    if [[ -z "$last_good" ]]; then
        echo -e "${RED}Could not determine last successful state${NC}"
        return 1
    fi

    echo "Last successful commit: $last_good"
    echo "$(git log --oneline -1 $last_good)"
    echo ""

    read -p "Rollback to this commit? (yes/no): " confirm
    if [[ "$confirm" == "yes" ]]; then
        save_checkpoint "pre_rollback_success"
        git reset --hard "$last_good"
        echo -e "${GREEN}Rolled back to last successful state${NC}"
    else
        echo "Rollback cancelled"
    fi
}

# ============================================
# AUTO-PR CREATION
# ============================================

# Create PR from Ralph results
create_pr() {
    local title="${1:-}"
    local draft="${2:-false}"

    # Check if gh CLI is available
    if ! command -v gh >/dev/null 2>&1; then
        echo -e "${RED}GitHub CLI (gh) not found${NC}"
        echo "Install from: https://cli.github.com/"
        return 1
    fi

    # Check authentication
    if ! gh auth status >/dev/null 2>&1; then
        echo -e "${RED}Not authenticated with GitHub${NC}"
        echo "Run: gh auth login"
        return 1
    fi

    # Get PR info from PRD if available
    local feature_name=""
    local branch_name=""

    if [[ -f "$PRD_FILE" ]]; then
        feature_name=$(grep '"feature"' "$PRD_FILE" | cut -d'"' -f4)
        branch_name=$(grep '"branchName"' "$PRD_FILE" | cut -d'"' -f4)
    fi

    # Use provided title or generate from feature
    if [[ -z "$title" ]]; then
        title="${feature_name:-$(git branch --show-current)}"
    fi

    # Get current branch
    local current_branch=$(git branch --show-current)

    # Ensure we're not on main/master
    if [[ "$current_branch" == "main" ]] || [[ "$current_branch" == "master" ]]; then
        echo -e "${RED}Cannot create PR from main/master branch${NC}"
        return 1
    fi

    # Check if there are commits to push
    local unpushed=$(git log origin/${current_branch}..HEAD 2>/dev/null | wc -l || echo "0")
    if [[ "$unpushed" -gt 0 ]]; then
        echo "Pushing $unpushed unpushed commit(s)..."
        git push -u origin "$current_branch" || {
            echo -e "${RED}Failed to push${NC}"
            return 1
        }
    fi

    # Build PR body
    local body=""

    # Add summary from PRD
    if [[ -f "$PRD_FILE" ]]; then
        local stories=$(jq -r '.userStories[] | "- [\(if .passes then "x" else " " end)] \(.id): \(.title)"' "$PRD_FILE" 2>/dev/null)
        local completed=$(jq '[.userStories[] | select(.passes == true)] | length' "$PRD_FILE" 2>/dev/null)
        local total=$(jq '.userStories | length' "$PRD_FILE" 2>/dev/null)

        body="## Summary
$feature_name

## Stories ($completed/$total completed)
$stories

## Test Plan
- [ ] All unit tests pass
- [ ] Manual verification complete
- [ ] No regressions identified
"
    else
        # Generate from git log
        local commits=$(git log origin/main..HEAD --oneline 2>/dev/null || git log -5 --oneline)
        body="## Summary
Auto-generated PR

## Commits
$commits

## Test Plan
- [ ] All tests pass
- [ ] Code review complete
"
    fi

    # Add progress learnings if available
    if [[ -f "$PROGRESS_FILE" ]]; then
        local learnings=$(tail -30 "$PROGRESS_FILE" | grep -A5 "^## Codebase Patterns" || echo "")
        if [[ -n "$learnings" ]]; then
            body="$body
## Learnings
\`\`\`
$learnings
\`\`\`
"
        fi
    fi

    echo -e "${BLUE}Creating Pull Request...${NC}"
    echo ""
    echo "Title: $title"
    echo "Branch: $current_branch"
    echo ""

    # Create PR
    local pr_args="--title \"$title\" --body \"$(echo "$body" | sed 's/"/\\"/g')\""

    if [[ "$draft" == "true" ]]; then
        pr_args="$pr_args --draft"
    fi

    local pr_url=$(gh pr create --title "$title" --body "$body" ${draft:+--draft} 2>&1)

    if [[ $? -eq 0 ]]; then
        echo -e "${GREEN}PR created successfully${NC}"
        echo "$pr_url"

        # Log PR creation
        echo "[$(date -Iseconds)] PR_CREATED: $pr_url" >> "$LOGS_DIR/prs.log"

        # Update PRD with PR URL
        if [[ -f "$PRD_FILE" ]] && command -v jq >/dev/null 2>&1; then
            local tmp=$(mktemp)
            jq ". + {\"pullRequest\": \"$pr_url\"}" "$PRD_FILE" > "$tmp" && mv "$tmp" "$PRD_FILE"
        fi

        echo "$pr_url"
    else
        echo -e "${RED}Failed to create PR${NC}"
        echo "$pr_url"
        return 1
    fi
}

# Create PR when all stories are complete
create_pr_if_complete() {
    if [[ ! -f "$PRD_FILE" ]]; then
        echo "No PRD file found"
        return 1
    fi

    local remaining=$(jq '[.userStories[] | select(.passes == false)] | length' "$PRD_FILE" 2>/dev/null)

    if [[ "$remaining" == "0" ]]; then
        echo -e "${GREEN}All stories complete! Creating PR...${NC}"
        create_pr
    else
        echo "$remaining stories remaining"
        return 1
    fi
}

# ============================================
# STATUS & INFO
# ============================================

show_status() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}              ARIA Git Operations Status                    ${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""

    # Git status
    echo -e "${BLUE}Git Status:${NC}"
    echo "  Branch:  $(git branch --show-current 2>/dev/null || echo 'unknown')"
    echo "  Commit:  $(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
    echo "  Changes: $(git status --porcelain 2>/dev/null | wc -l) uncommitted"
    echo "  Ahead:   $(git log origin/$(git branch --show-current)..HEAD --oneline 2>/dev/null | wc -l) commits"
    echo ""

    # Checkpoints
    local checkpoint_count=$(ls -1 "$CHECKPOINTS_DIR" 2>/dev/null | wc -l)
    echo -e "${BLUE}Checkpoints:${NC} $checkpoint_count saved"
    echo ""

    # PRD status
    if [[ -f "$PRD_FILE" ]]; then
        local completed=$(jq '[.userStories[] | select(.passes == true)] | length' "$PRD_FILE" 2>/dev/null)
        local total=$(jq '.userStories | length' "$PRD_FILE" 2>/dev/null)
        local pr=$(jq -r '.pullRequest // "none"' "$PRD_FILE" 2>/dev/null)

        echo -e "${BLUE}PRD Status:${NC}"
        echo "  Stories: $completed/$total complete"
        echo "  PR:      $pr"
        echo ""

        if [[ "$completed" == "$total" ]] && [[ "$pr" == "none" ]]; then
            echo -e "${GREEN}All stories complete - ready for PR!${NC}"
            echo "  Run: aria pr create"
        fi
    fi
}

# ============================================
# CLI
# ============================================

main() {
    local command="${1:-help}"
    shift || true

    case "$command" in
        # Checkpoint commands
        "checkpoint"|"save")
            # Handle subcommands
            if [[ "$1" == "list" ]]; then
                list_checkpoints
            else
                local id=$(save_checkpoint "$1")
                echo -e "${GREEN}Checkpoint saved: $id${NC}"
            fi
            ;;
        "checkpoints"|"list-checkpoints")
            list_checkpoints
            ;;

        # Rollback commands
        "rollback")
            local subcommand="${1:-help}"
            shift || true

            case "$subcommand" in
                "commits"|"n")
                    rollback_commits "${1:-1}"
                    ;;
                "checkpoint"|"to")
                    rollback_to_checkpoint "$1"
                    ;;
                "success"|"last-good")
                    rollback_to_success
                    ;;
                "restore")
                    rollback_to_checkpoint "$1"
                    ;;
                *)
                    echo "Rollback commands:"
                    echo "  rollback commits <n>      - Rollback last N commits (soft)"
                    echo "  rollback checkpoint <id>  - Rollback to checkpoint"
                    echo "  rollback success          - Rollback to last successful state"
                    ;;
            esac
            ;;

        # PR commands
        "pr")
            local subcommand="${1:-create}"
            shift || true

            case "$subcommand" in
                "create")
                    create_pr "$1" "${2:-false}"
                    ;;
                "draft")
                    create_pr "$1" "true"
                    ;;
                "auto"|"if-complete")
                    create_pr_if_complete
                    ;;
                *)
                    echo "PR commands:"
                    echo "  pr create [title]     - Create PR"
                    echo "  pr draft [title]      - Create draft PR"
                    echo "  pr auto               - Create PR if all stories complete"
                    ;;
            esac
            ;;

        "status")
            show_status
            ;;

        "help"|*)
            echo "ARIA Git Operations"
            echo ""
            echo "Usage: $0 <command> [args]"
            echo ""
            echo "Checkpoint commands:"
            echo "  checkpoint [name]       - Save current state as checkpoint"
            echo "  checkpoints             - List all checkpoints"
            echo ""
            echo "Rollback commands:"
            echo "  rollback commits <n>    - Rollback last N commits"
            echo "  rollback checkpoint <id> - Rollback to specific checkpoint"
            echo "  rollback success        - Rollback to last successful state"
            echo ""
            echo "PR commands:"
            echo "  pr create [title]       - Create pull request"
            echo "  pr draft [title]        - Create draft PR"
            echo "  pr auto                 - Create PR if all stories complete"
            echo ""
            echo "Other:"
            echo "  status                  - Show git operations status"
            ;;
    esac
}

main "$@"
