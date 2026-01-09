#!/bin/bash
# ARIA Agent Runner
# Invokes agents defined in .claude/agents/ via Claude CLI

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
AGENTS_DIR="$PROJECT_DIR/.claude/agents"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# ============================================
# AGENT DISCOVERY
# ============================================

list_agents() {
    echo ""
    echo -e "${BLUE}Available Agents:${NC}"
    echo ""

    for agent_file in "$AGENTS_DIR"/*.md; do
        if [[ -f "$agent_file" ]]; then
            local name=$(basename "$agent_file" .md)
            # Extract description from frontmatter
            local description=$(grep -A10 '^---' "$agent_file" | grep 'description:' | head -1 | sed 's/description: *//')
            printf "  %-20s %s\n" "$name" "$description"
        fi
    done
    echo ""
}

# ============================================
# AGENT PARSING
# ============================================

parse_agent() {
    local agent_file="$1"

    if [[ ! -f "$agent_file" ]]; then
        echo -e "${RED}Agent file not found: $agent_file${NC}"
        return 1
    fi

    # Extract frontmatter
    local in_frontmatter=false
    local frontmatter=""
    local content=""

    while IFS= read -r line; do
        if [[ "$line" == "---" ]]; then
            if $in_frontmatter; then
                in_frontmatter=false
                continue
            else
                in_frontmatter=true
                continue
            fi
        fi

        if $in_frontmatter; then
            frontmatter+="$line"$'\n'
        else
            content+="$line"$'\n'
        fi
    done < "$agent_file"

    echo "$content"
}

get_agent_prop() {
    local agent_file="$1"
    local prop="$2"

    grep -A20 '^---' "$agent_file" | grep "^${prop}:" | head -1 | sed "s/${prop}: *//"
}

# ============================================
# AGENT EXECUTION
# ============================================

run_agent() {
    local agent_name="$1"
    shift
    local context="$*"

    local agent_file="$AGENTS_DIR/${agent_name}.md"

    if [[ ! -f "$agent_file" ]]; then
        echo -e "${RED}Agent not found: $agent_name${NC}"
        echo "Available agents:"
        list_agents
        return 1
    fi

    local description=$(get_agent_prop "$agent_file" "description")
    local model=$(get_agent_prop "$agent_file" "model")
    local tools=$(get_agent_prop "$agent_file" "tools")

    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  Running Agent: $agent_name${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Description: $description"
    echo "Model: ${model:-default}"
    echo ""

    # Build prompt from agent file
    local prompt=$(parse_agent "$agent_file")

    # Add context if provided
    if [[ -n "$context" ]]; then
        prompt="$prompt

## Context
$context"
    fi

    # Add current state
    prompt="$prompt

## Current State
- Working directory: $(pwd)
- Git branch: $(git branch --show-current 2>/dev/null || echo 'not a git repo')
- Changed files: $(git diff --name-only 2>/dev/null | head -5 | tr '\n' ', ' || echo 'none')
"

    # Check if Claude CLI is available
    if command -v claude >/dev/null 2>&1; then
        echo "Invoking Claude..."
        echo ""
        echo "$prompt" | claude -p 2>&1
    else
        echo -e "${YELLOW}Claude CLI not found. Printing prompt instead:${NC}"
        echo ""
        echo "─────────────────────────────────────────"
        echo "$prompt"
        echo "─────────────────────────────────────────"
        echo ""
        echo "To run this agent, pipe the above to Claude:"
        echo "  echo \"\$prompt\" | claude -p"
    fi
}

# ============================================
# QUICK AGENTS (built-in)
# ============================================

run_verify_quick() {
    echo "Running quick verification..."

    # Run unit tests
    echo ""
    echo "Running tests..."
    if [[ -f "package.json" ]]; then
        npm test 2>&1 || echo "Tests failed"
    elif [[ -f "pytest.ini" ]] || [[ -f "pyproject.toml" ]]; then
        pytest 2>&1 || echo "Tests failed"
    else
        echo "No test framework detected"
    fi

    # Run type check
    if [[ -f "tsconfig.json" ]]; then
        echo ""
        echo "Running type check..."
        npx tsc --noEmit 2>&1 || echo "Type errors"
    fi

    # Run lint
    if [[ -f "package.json" ]] && grep -q '"lint"' package.json; then
        echo ""
        echo "Running lint..."
        npm run lint 2>&1 || echo "Lint errors"
    fi

    echo ""
    echo "Quick verification complete"
}

run_simplify() {
    local file="$1"

    if [[ -z "$file" ]]; then
        echo "Usage: $0 simplify <file>"
        return 1
    fi

    if [[ ! -f "$file" ]]; then
        echo "File not found: $file"
        return 1
    fi

    echo "Analyzing $file for simplification opportunities..."

    # Count lines, functions, complexity indicators
    local lines=$(wc -l < "$file")
    local functions=$(grep -cE '(function |const .* = |def |fn )' "$file" 2>/dev/null || echo 0)
    local todos=$(grep -c 'TODO\|FIXME\|HACK' "$file" 2>/dev/null || echo 0)
    local console_logs=$(grep -c 'console\.' "$file" 2>/dev/null || echo 0)

    echo ""
    echo "File: $file"
    echo "Lines: $lines"
    echo "Functions: $functions"
    echo "TODOs/FIXMEs: $todos"
    echo "Console statements: $console_logs"
    echo ""

    if [[ $console_logs -gt 0 ]]; then
        echo -e "${YELLOW}Consider removing $console_logs console statements${NC}"
    fi
    if [[ $todos -gt 0 ]]; then
        echo -e "${YELLOW}$todos TODO/FIXME comments to address${NC}"
    fi
    if [[ $lines -gt 300 ]]; then
        echo -e "${YELLOW}File is $lines lines - consider splitting${NC}"
    fi

    # If Claude available, do deeper analysis
    if command -v claude >/dev/null 2>&1; then
        echo ""
        echo "Running AI analysis..."
        cat "$file" | claude -p "Analyze this code for simplification opportunities. Suggest specific refactoring. Be concise." 2>&1
    fi
}

# ============================================
# MAIN
# ============================================

main() {
    local command="${1:-help}"
    shift || true

    case "$command" in
        "run")
            run_agent "$@"
            ;;
        "verify"|"verify-app")
            if [[ -f "$AGENTS_DIR/verify-app.md" ]]; then
                run_agent "verify-app" "$@"
            else
                run_verify_quick
            fi
            ;;
        "simplify"|"code-simplifier")
            if [[ -f "$AGENTS_DIR/code-simplifier.md" ]] && command -v claude >/dev/null 2>&1; then
                run_agent "code-simplifier" "$@"
            else
                run_simplify "$@"
            fi
            ;;
        "list"|"ls")
            list_agents
            ;;
        "show")
            local agent_file="$AGENTS_DIR/${1}.md"
            if [[ -f "$agent_file" ]]; then
                cat "$agent_file"
            else
                echo "Agent not found: $1"
            fi
            ;;
        "help"|*)
            echo "ARIA Agent Runner"
            echo ""
            echo "Usage: $0 <command> [args]"
            echo ""
            echo "Commands:"
            echo "  run <agent> [context]   - Run an agent with optional context"
            echo "  verify                  - Run verify-app agent"
            echo "  simplify <file>         - Run code-simplifier on a file"
            echo "  list                    - List available agents"
            echo "  show <agent>            - Show agent definition"
            echo ""
            echo "Examples:"
            echo "  $0 run verify-app"
            echo "  $0 verify"
            echo "  $0 simplify src/utils.ts"
            echo "  $0 list"
            ;;
    esac
}

main "$@"
