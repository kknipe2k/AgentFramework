#!/bin/bash
# ARIA Engine - Rail loader, detector, and executor

set -e

ARIA_DIR=".aria"
RAILS_DIR="$ARIA_DIR/rails"
STATE_DIR="$ARIA_DIR/state"
SCREENSHOTS_DIR="$ARIA_DIR/screenshots"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Initialize directories
mkdir -p "$STATE_DIR" "$SCREENSHOTS_DIR"

# ============================================
# PROJECT DETECTION
# ============================================

detect_project_type() {
    local types=""

    # Node.js
    if [[ -f "package.json" ]]; then
        types="$types node"

        # TypeScript
        if [[ -f "tsconfig.json" ]]; then
            types="$types typescript"
        fi

        # React
        if grep -q '"react"' package.json 2>/dev/null; then
            types="$types react"
        fi

        # Next.js
        if [[ -f "next.config.js" ]] || [[ -f "next.config.mjs" ]]; then
            types="$types nextjs"
        fi

        # Express
        if grep -q '"express"' package.json 2>/dev/null; then
            types="$types express"
        fi

        # Database
        if grep -qE '"(prisma|sequelize|typeorm|pg|mysql)"' package.json 2>/dev/null; then
            types="$types database"
        fi
    fi

    # Python
    if [[ -f "requirements.txt" ]] || [[ -f "pyproject.toml" ]] || [[ -f "setup.py" ]]; then
        types="$types python"

        # Django
        if [[ -f "manage.py" ]]; then
            types="$types django"
        fi

        # Flask
        if grep -q "flask" requirements.txt 2>/dev/null; then
            types="$types flask"
        fi
    fi

    # Static site
    if [[ -f "index.html" ]] && [[ ! -f "package.json" ]]; then
        types="$types static"
    fi

    # Database (env var)
    if [[ -n "$DATABASE_URL" ]]; then
        types="$types database"
    fi

    echo "$types"
}

# ============================================
# RAIL EXECUTION
# ============================================

run_rail() {
    local rail_name="$1"
    local result=0

    case "$rail_name" in
        # Environment rails
        "node_installed")
            node --version >/dev/null 2>&1 || result=1
            ;;
        "deps_installed")
            [[ -d "node_modules" ]] || result=1
            ;;
        "server_running")
            curl -s -o /dev/null "http://localhost:${PORT:-3000}" 2>/dev/null || result=1
            ;;
        "python_installed")
            python3 --version >/dev/null 2>&1 || result=1
            ;;
        "venv_active")
            [[ -n "$VIRTUAL_ENV" ]] || result=1
            ;;
        "db_connected")
            if command -v pg_isready >/dev/null 2>&1; then
                pg_isready -h "${DB_HOST:-localhost}" >/dev/null 2>&1 || result=1
            else
                result=2  # Skip - can't check
            fi
            ;;

        # Quality rails
        "tests_pass")
            if [[ -f "package.json" ]]; then
                npm test 2>/dev/null || result=1
            elif [[ -f "pytest.ini" ]] || [[ -f "pyproject.toml" ]]; then
                pytest 2>/dev/null || result=1
            else
                result=2  # Skip - no test config
            fi
            ;;
        "lint_clean")
            if [[ -f ".eslintrc.json" ]] || [[ -f ".eslintrc.js" ]] || [[ -f ".eslintrc" ]]; then
                npm run lint 2>/dev/null || npx eslint . 2>/dev/null || result=1
            else
                result=2  # Skip
            fi
            ;;
        "types_valid")
            if [[ -f "tsconfig.json" ]]; then
                npx tsc --noEmit 2>/dev/null || result=1
            else
                result=2  # Skip
            fi
            ;;
        "build_succeeds")
            if grep -q '"build"' package.json 2>/dev/null; then
                npm run build 2>/dev/null || result=1
            else
                result=2  # Skip
            fi
            ;;

        # Safety rails
        "no_secrets")
            # Check for common secret patterns
            if grep -rE "(api[_-]?key|secret|password|token)\s*[=:]\s*['\"][^'\"]{10,}['\"]" \
                --include="*.js" --include="*.ts" --include="*.py" \
                --exclude-dir=node_modules --exclude-dir=.git \
                . 2>/dev/null | grep -v "example\|sample\|test\|spec" | head -1; then
                result=1
            fi
            ;;
        "no_destructive")
            # This is checked in PreToolUse, always passes here
            result=0
            ;;

        # Documentation rails
        "readme_exists")
            [[ -f "README.md" ]] || result=1
            ;;
        "readme_has_setup")
            grep -qiE "(npm install|pip install|## install|## setup|## getting started)" README.md 2>/dev/null || result=1
            ;;

        # Verification rails
        "app_loads")
            if command -v npx >/dev/null 2>&1; then
                node -e "
                    const http = require('http');
                    http.get('http://localhost:${PORT:-3000}', (res) => {
                        process.exit(res.statusCode === 200 ? 0 : 1);
                    }).on('error', () => process.exit(1));
                " 2>/dev/null || result=1
            else
                result=2
            fi
            ;;
        "screenshot")
            if command -v npx >/dev/null 2>&1 && [[ -f "node_modules/playwright/package.json" ]]; then
                npx playwright screenshot "http://localhost:${PORT:-3000}" "$SCREENSHOTS_DIR/latest.png" 2>/dev/null || result=2
            else
                result=2
            fi
            ;;

        *)
            result=2  # Unknown rail, skip
            ;;
    esac

    return $result
}

get_rail_message() {
    local rail_name="$1"
    case "$rail_name" in
        "node_installed") echo "Node.js is not installed" ;;
        "deps_installed") echo "Dependencies not installed. Run: npm install" ;;
        "server_running") echo "Server not running. Run: npm start" ;;
        "python_installed") echo "Python is not installed" ;;
        "venv_active") echo "Virtual environment not active. Run: source venv/bin/activate" ;;
        "db_connected") echo "Database not connected. Check DATABASE_URL" ;;
        "tests_pass") echo "Tests are failing" ;;
        "lint_clean") echo "Linting errors found" ;;
        "types_valid") echo "TypeScript type errors found" ;;
        "build_succeeds") echo "Build failed" ;;
        "no_secrets") echo "Possible secrets found in code" ;;
        "readme_exists") echo "No README.md found" ;;
        "readme_has_setup") echo "README missing setup instructions" ;;
        "app_loads") echo "App not loading at localhost:${PORT:-3000}" ;;
        *) echo "Rail check failed: $rail_name" ;;
    esac
}

# ============================================
# STATE MANAGEMENT
# ============================================

get_edit_count() {
    cat "$STATE_DIR/edit_count" 2>/dev/null || echo 0
}

get_last_test() {
    cat "$STATE_DIR/last_test" 2>/dev/null || echo 0
}

get_last_commit() {
    cat "$STATE_DIR/last_commit" 2>/dev/null || echo 0
}

increment_edits() {
    local count=$(get_edit_count)
    echo $((count + 1)) > "$STATE_DIR/edit_count"
}

record_test() {
    get_edit_count > "$STATE_DIR/last_test"
    rm -f "$STATE_DIR/tests_failed"
}

record_test_failure() {
    touch "$STATE_DIR/tests_failed"
}

record_commit() {
    get_edit_count > "$STATE_DIR/last_commit"
}

tests_failing() {
    [[ -f "$STATE_DIR/tests_failed" ]]
}

# ============================================
# RAIL CHECKING
# ============================================

check_cadence_rails() {
    local edits=$(get_edit_count)
    local last_test=$(get_last_test)
    local last_commit=$(get_last_commit)
    local test_cadence=${ARIA_TEST_CADENCE:-3}
    local commit_cadence=${ARIA_COMMIT_CADENCE:-5}

    local edits_since_test=$((edits - last_test))
    local edits_since_commit=$((edits - last_commit))

    # Check test cadence
    if [[ $edits_since_test -ge $test_cadence ]]; then
        echo -e "${RED}BLOCKED:${NC} $edits_since_test edits without testing (max: $test_cadence)"
        echo "Run tests before continuing."
        return 1
    fi

    # Check commit cadence
    if [[ $edits_since_commit -ge $commit_cadence ]]; then
        echo -e "${RED}BLOCKED:${NC} $edits_since_commit edits without commit (max: $commit_cadence)"
        echo "Commit a checkpoint before continuing."
        return 1
    fi

    return 0
}

check_pre_commit_rails() {
    # Tests must pass before commit
    if tests_failing; then
        echo -e "${RED}BLOCKED:${NC} Tests are failing"
        echo "Fix tests before committing."
        return 1
    fi

    # Check for secrets
    run_rail "no_secrets"
    if [[ $? -eq 1 ]]; then
        echo -e "${RED}BLOCKED:${NC} Possible secrets in code"
        echo "Remove secrets before committing."
        return 1
    fi

    return 0
}

check_intent_exists() {
    if [[ ! -f "$ARIA_DIR/intent.md" ]]; then
        echo -e "${RED}BLOCKED:${NC} No intent defined"
        echo "Run: aria init \"your intent here\""
        return 1
    fi
    return 0
}

# ============================================
# MAIN COMMANDS
# ============================================

cmd_init() {
    local intent="$1"
    if [[ -z "$intent" ]]; then
        echo "Usage: aria init \"your intent here\""
        exit 1
    fi

    mkdir -p "$STATE_DIR"
    echo "0" > "$STATE_DIR/edit_count"
    echo "0" > "$STATE_DIR/last_test"
    echo "0" > "$STATE_DIR/last_commit"
    rm -f "$STATE_DIR/tests_failed"

    cat > "$ARIA_DIR/intent.md" << EOF
# Intent: $intent

## Must Have:
-

## Must Not:
-

## Done When:
-
EOF

    # Detect project type
    local types=$(detect_project_type)
    echo "$types" > "$STATE_DIR/project_types"

    echo -e "${GREEN}ARIA initialized${NC}"
    echo "Intent: $intent"
    echo "Project types detected: $types"
    echo ""
    echo "Edit $ARIA_DIR/intent.md to add requirements"
}

cmd_status() {
    if [[ ! -f "$ARIA_DIR/intent.md" ]]; then
        echo "ARIA not initialized. Run: aria init \"intent\""
        exit 1
    fi

    local edits=$(get_edit_count)
    local last_test=$(get_last_test)
    local last_commit=$(get_last_commit)
    local test_cadence=${ARIA_TEST_CADENCE:-3}
    local commit_cadence=${ARIA_COMMIT_CADENCE:-5}
    local tests_ok="YES"
    tests_failing && tests_ok="NO"

    local types=$(cat "$STATE_DIR/project_types" 2>/dev/null || detect_project_type)

    echo "═══════════════════════════════════════════════════════════"
    echo "                      ARIA STATUS"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    echo "Intent: $(head -1 "$ARIA_DIR/intent.md" | sed 's/# Intent: //')"
    echo "Project: $types"
    echo ""
    echo "Edits total:        $edits"
    echo "Edits since test:   $((edits - last_test))/$test_cadence"
    echo "Edits since commit: $((edits - last_commit))/$commit_cadence"
    echo "Tests passing:      $tests_ok"
    echo ""
    echo "═══════════════════════════════════════════════════════════"

    # Warnings
    if [[ $((edits - last_test)) -ge $((test_cadence - 1)) ]]; then
        echo -e "${YELLOW}⚠ Run tests soon${NC}"
    fi
    if [[ $((edits - last_commit)) -ge $((commit_cadence - 1)) ]]; then
        echo -e "${YELLOW}⚠ Commit soon${NC}"
    fi
    if [[ "$tests_ok" == "NO" ]]; then
        echo -e "${RED}✗ Tests failing - fix before commit${NC}"
    fi
}

cmd_check() {
    local rail_name="$1"

    if [[ -z "$rail_name" ]]; then
        # Run all applicable rails
        local types=$(cat "$STATE_DIR/project_types" 2>/dev/null || detect_project_type)
        local failed=0

        echo "Running rail checks for: $types"
        echo ""

        # Core rails for all projects
        for rail in "readme_exists" "no_secrets"; do
            printf "  %-20s " "$rail"
            run_rail "$rail"
            case $? in
                0) echo -e "${GREEN}✓${NC}" ;;
                1) echo -e "${RED}✗${NC} - $(get_rail_message $rail)"; failed=1 ;;
                2) echo -e "${YELLOW}skip${NC}" ;;
            esac
        done

        # Node-specific rails
        if echo "$types" | grep -q "node"; then
            for rail in "node_installed" "deps_installed" "tests_pass" "lint_clean"; do
                printf "  %-20s " "$rail"
                run_rail "$rail"
                case $? in
                    0) echo -e "${GREEN}✓${NC}" ;;
                    1) echo -e "${RED}✗${NC} - $(get_rail_message $rail)"; failed=1 ;;
                    2) echo -e "${YELLOW}skip${NC}" ;;
                esac
            done
        fi

        # TypeScript rails
        if echo "$types" | grep -q "typescript"; then
            printf "  %-20s " "types_valid"
            run_rail "types_valid"
            case $? in
                0) echo -e "${GREEN}✓${NC}" ;;
                1) echo -e "${RED}✗${NC} - $(get_rail_message types_valid)"; failed=1 ;;
                2) echo -e "${YELLOW}skip${NC}" ;;
            esac
        fi

        # React/frontend rails
        if echo "$types" | grep -qE "(react|nextjs)"; then
            for rail in "server_running" "app_loads"; do
                printf "  %-20s " "$rail"
                run_rail "$rail"
                case $? in
                    0) echo -e "${GREEN}✓${NC}" ;;
                    1) echo -e "${RED}✗${NC} - $(get_rail_message $rail)"; failed=1 ;;
                    2) echo -e "${YELLOW}skip${NC}" ;;
                esac
            done
        fi

        echo ""
        if [[ $failed -eq 1 ]]; then
            echo -e "${RED}Some rails failed${NC}"
            return 1
        else
            echo -e "${GREEN}All rails passed${NC}"
            return 0
        fi
    else
        # Run specific rail
        run_rail "$rail_name"
        local result=$?
        case $result in
            0) echo -e "${GREEN}✓${NC} $rail_name passed" ;;
            1) echo -e "${RED}✗${NC} $rail_name failed: $(get_rail_message $rail_name)"; return 1 ;;
            2) echo -e "${YELLOW}skip${NC} $rail_name not applicable" ;;
        esac
    fi
}

cmd_verify() {
    local level="${1:-standard}"
    local executor="$ARIA_DIR/verify-executor.sh"

    # Use executor if available
    if [[ -x "$executor" ]]; then
        case "$level" in
            "quick")   "$executor" quick ;;
            "standard"|"commit") "$executor" standard ;;
            "full"|"deploy"|"pr") "$executor" full ;;
            *)
                echo "Usage: aria verify [quick|standard|full]"
                echo "  quick    - Tests, types, lint"
                echo "  standard - Quick + build + app loads"
                echo "  full     - Everything including E2E"
                return 1
                ;;
        esac
        return $?
    fi

    # Fallback: manual verification
    echo "═══════════════════════════════════════════════════════════"
    echo "                   USER VERIFICATION"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    echo "Please verify the implementation:"
    echo ""

    # Show changes
    echo "Files changed:"
    git diff --stat HEAD~1 2>/dev/null || echo "(unable to show git diff)"
    echo ""

    # Show intent
    echo "Original intent:"
    cat "$ARIA_DIR/intent.md"
    echo ""

    echo "═══════════════════════════════════════════════════════════"
    echo ""
    read -p "Does the implementation satisfy the intent? (yes/no): " answer

    if [[ "$answer" == "yes" || "$answer" == "y" ]]; then
        echo -e "${GREEN}✓ Verified${NC}"
        return 0
    else
        echo -e "${RED}✗ Not verified${NC}"
        read -p "What's wrong? " issue
        echo "$issue" >> "$STATE_DIR/issues.log"
        return 1
    fi
}

cmd_done() {
    echo "Running final checks..."
    echo ""

    # Run all checks
    if ! cmd_check; then
        echo ""
        echo -e "${RED}Cannot complete - fix failing rails first${NC}"
        return 1
    fi

    # User verification
    echo ""
    if ! cmd_verify; then
        echo ""
        echo -e "${RED}Cannot complete - user verification failed${NC}"
        return 1
    fi

    echo ""
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}                    ARIA COMPLETE${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"

    # Archive state
    local timestamp=$(date +%Y%m%d_%H%M%S)
    mkdir -p "$ARIA_DIR/archive"
    cp -r "$STATE_DIR" "$ARIA_DIR/archive/$timestamp"
    cp "$ARIA_DIR/intent.md" "$ARIA_DIR/archive/$timestamp/"

    # Clear current state
    rm -rf "$STATE_DIR"
    rm -f "$ARIA_DIR/intent.md"

    echo "State archived to $ARIA_DIR/archive/$timestamp"
}

cmd_rails() {
    local subcmd="${1:-all}"
    shift || true

    local executor="$ARIA_DIR/rails-executor.sh"
    if [[ ! -x "$executor" ]]; then
        echo "Rails executor not found"
        return 1
    fi

    "$executor" "$subcmd" "$@"
}

cmd_agent() {
    local subcmd="${1:-list}"
    shift || true

    local runner="$ARIA_DIR/agent-runner.sh"
    if [[ ! -x "$runner" ]]; then
        echo "Agent runner not found"
        return 1
    fi

    "$runner" "$subcmd" "$@"
}

cmd_hitl() {
    local subcmd="${1:-status}"
    shift || true

    local hitl_script="$ARIA_DIR/hitl.sh"
    if [[ ! -x "$hitl_script" ]]; then
        echo "HITL system not found"
        return 1
    fi

    "$hitl_script" "$subcmd" "$@"
}

cmd_rollback() {
    local subcmd="${1:-help}"
    shift || true

    local git_ops="$ARIA_DIR/git-ops.sh"
    if [[ ! -x "$git_ops" ]]; then
        echo "Git operations not found"
        return 1
    fi

    "$git_ops" rollback "$subcmd" "$@"
}

cmd_checkpoint() {
    local name="${1:-auto}"

    local git_ops="$ARIA_DIR/git-ops.sh"
    if [[ ! -x "$git_ops" ]]; then
        echo "Git operations not found"
        return 1
    fi

    "$git_ops" checkpoint "$name"
}

cmd_pr() {
    local subcmd="${1:-create}"
    shift || true

    local git_ops="$ARIA_DIR/git-ops.sh"
    if [[ ! -x "$git_ops" ]]; then
        echo "Git operations not found"
        return 1
    fi

    "$git_ops" pr "$subcmd" "$@"
}

cmd_help() {
    echo "ARIA - Agentic Rail-based Intent Architecture"
    echo ""
    echo "Commands:"
    echo "  aria init \"intent\"     - Start with intent"
    echo "  aria status            - Show current state"
    echo "  aria check [rail]      - Run rail checks (hardcoded)"
    echo "  aria rails [cmd]       - Run YAML-defined rails"
    echo "  aria verify [level]    - Run verification (quick|standard|full)"
    echo "  aria agent [cmd]       - Run agents"
    echo "  aria hitl [cmd]        - Human-in-the-loop system"
    echo "  aria rollback [cmd]    - Rollback to safe state"
    echo "  aria pr [cmd]          - Pull request operations"
    echo "  aria done              - Complete and archive"
    echo ""
    echo "  aria pass              - Mark tests as passed"
    echo "  aria fail              - Mark tests as failed"
    echo "  aria reset             - Reset all counters"
    echo "  aria checkpoint [name] - Save current state"
    echo ""
    echo "Rollback commands:"
    echo "  aria rollback commits <n>     - Undo last N commits"
    echo "  aria rollback checkpoint <id> - Restore to checkpoint"
    echo "  aria rollback success         - Rollback to last good state"
    echo ""
    echo "PR commands:"
    echo "  aria pr create [title] - Create pull request"
    echo "  aria pr draft [title]  - Create draft PR"
    echo "  aria pr auto           - Create PR if all stories done"
    echo ""
    echo "Rails commands:"
    echo "  aria rails all         - Run all applicable rails"
    echo "  aria rails fix         - Run rails with auto-fix"
    echo "  aria rails list        - List available rails"
    echo ""
    echo "HITL commands:"
    echo "  aria hitl status       - Show pending requests"
    echo "  aria hitl respond <id> - Respond to a request"
    echo "  aria hitl approve <id> - Quick approve"
}

# ============================================
# MAIN
# ============================================

case "${1:-help}" in
    init)       cmd_init "$2" ;;
    status)     cmd_status ;;
    check)      cmd_check "$2" ;;
    rails)      shift; cmd_rails "$@" ;;
    verify)     cmd_verify "$2" ;;
    agent)      shift; cmd_agent "$@" ;;
    hitl)       shift; cmd_hitl "$@" ;;
    rollback)   shift; cmd_rollback "$@" ;;
    checkpoint) cmd_checkpoint "$2" ;;
    pr)         shift; cmd_pr "$@" ;;
    done)       cmd_done ;;
    pass)       record_test; echo "Tests marked as passed" ;;
    fail)       record_test_failure; echo "Tests marked as failed" ;;
    reset)      echo "0" > "$STATE_DIR/edit_count"; echo "0" > "$STATE_DIR/last_test"; echo "0" > "$STATE_DIR/last_commit"; rm -f "$STATE_DIR/tests_failed"; echo "Counters reset" ;;
    help|*)     cmd_help ;;
esac
