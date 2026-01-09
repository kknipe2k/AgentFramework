#!/bin/bash
# ARIA Rails Executor
# Parses YAML rail definitions and executes them
# Supports auto-fix when rails fail

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RAILS_DIR="$SCRIPT_DIR/rails"
STATE_DIR="$SCRIPT_DIR/state"
LOGS_DIR="$SCRIPT_DIR/logs"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

mkdir -p "$STATE_DIR" "$LOGS_DIR"

# ============================================
# YAML PARSING (using yq, python, or grep)
# ============================================

# Check for YAML parser
get_yaml_parser() {
    if command -v yq >/dev/null 2>&1; then
        echo "yq"
    elif command -v python3 >/dev/null 2>&1; then
        echo "python"
    else
        echo "grep"
    fi
}

# Parse YAML value using available parser
yaml_get() {
    local file="$1"
    local path="$2"
    local parser=$(get_yaml_parser)

    case "$parser" in
        "yq")
            yq eval "$path" "$file" 2>/dev/null
            ;;
        "python")
            python3 -c "
import yaml
import sys
with open('$file') as f:
    data = yaml.safe_load(f)
path = '$path'.strip('.').split('.')
result = data
for key in path:
    if key and result:
        if key.startswith('[') and key.endswith(']'):
            idx = int(key[1:-1])
            result = result[idx] if isinstance(result, list) and len(result) > idx else None
        else:
            result = result.get(key) if isinstance(result, dict) else None
print(result if result is not None else '')
" 2>/dev/null
            ;;
        "grep")
            # Basic grep-based parsing (limited)
            grep -A1 "^[[:space:]]*${path##*.}:" "$file" 2>/dev/null | tail -1 | sed 's/.*: *//' | tr -d '"'
            ;;
    esac
}

# List all rails in a file
yaml_list_rails() {
    local file="$1"
    local parser=$(get_yaml_parser)

    case "$parser" in
        "yq")
            yq eval '.rails | keys | .[]' "$file" 2>/dev/null
            ;;
        "python")
            python3 -c "
import yaml
with open('$file') as f:
    data = yaml.safe_load(f)
if data and 'rails' in data:
    for key in data['rails'].keys():
        print(key)
" 2>/dev/null
            ;;
        "grep")
            grep -E '^  [a-z_]+:$' "$file" 2>/dev/null | sed 's/://g' | tr -d ' '
            ;;
    esac
}

# Get rail property
get_rail_prop() {
    local file="$1"
    local rail="$2"
    local prop="$3"
    yaml_get "$file" ".rails.$rail.$prop"
}

# ============================================
# PROJECT DETECTION
# ============================================

detect_project_types() {
    local types=""

    [[ -f "package.json" ]] && types="$types node"
    [[ -f "tsconfig.json" ]] && types="$types typescript"
    [[ -f "requirements.txt" ]] || [[ -f "pyproject.toml" ]] && types="$types python"
    [[ -f "Cargo.toml" ]] && types="$types rust"
    [[ -f "go.mod" ]] && types="$types go"

    # Framework detection
    if [[ -f "package.json" ]]; then
        grep -q '"react"' package.json 2>/dev/null && types="$types react"
        grep -q '"vue"' package.json 2>/dev/null && types="$types vue"
        grep -q '"next"' package.json 2>/dev/null && types="$types nextjs"
        grep -q '"express"' package.json 2>/dev/null && types="$types express"
    fi

    echo "$types"
}

# ============================================
# CONDITION CHECKING
# ============================================

check_applies_when() {
    local file="$1"
    local rail="$2"

    # Get applies_when conditions
    local file_exists=$(get_rail_prop "$file" "$rail" "applies_when.file_exists")
    local any_file=$(get_rail_prop "$file" "$rail" "applies_when.any_file_exists")
    local env_var=$(get_rail_prop "$file" "$rail" "applies_when.env_var_set")

    # If no conditions, always applies
    if [[ -z "$file_exists" ]] && [[ -z "$any_file" ]] && [[ -z "$env_var" ]]; then
        return 0
    fi

    # Check file_exists
    if [[ -n "$file_exists" ]]; then
        # Handle array or single value
        for f in $(echo "$file_exists" | tr '[],' ' '); do
            f=$(echo "$f" | tr -d '"' | tr -d "'")
            [[ -n "$f" ]] && [[ -f "$f" ]] && return 0
        done
    fi

    # Check any_file_exists
    if [[ -n "$any_file" ]]; then
        for f in $(echo "$any_file" | tr '[],' ' '); do
            f=$(echo "$f" | tr -d '"' | tr -d "'")
            [[ -n "$f" ]] && [[ -f "$f" ]] && return 0
            [[ -n "$f" ]] && [[ -d "$f" ]] && return 0
        done
    fi

    # Check env_var_set
    if [[ -n "$env_var" ]]; then
        [[ -n "${!env_var}" ]] && return 0
    fi

    return 1
}

# ============================================
# RAIL EXECUTION
# ============================================

execute_rail() {
    local file="$1"
    local rail="$2"
    local auto_fix="${3:-false}"

    local description=$(get_rail_prop "$file" "$rail" "description")
    local command=$(get_rail_prop "$file" "$rail" "check.command")
    local script=$(get_rail_prop "$file" "$rail" "check.script")
    local expect_exit=$(get_rail_prop "$file" "$rail" "check.expect_exit")
    local expect_output=$(get_rail_prop "$file" "$rail" "check.expect_output")
    local message=$(get_rail_prop "$file" "$rail" "message")
    local fix=$(get_rail_prop "$file" "$rail" "fix")
    local fix_cmd=$(get_rail_prop "$file" "$rail" "auto_fix")
    local severity=$(get_rail_prop "$file" "$rail" "severity")

    # Check if rail applies
    if ! check_applies_when "$file" "$rail"; then
        return 2  # Skip
    fi

    # Execute check
    local result=0
    local output=""

    if [[ -n "$command" ]]; then
        output=$(eval "$command" 2>&1) || result=$?
    elif [[ -n "$script" ]]; then
        output=$(bash -c "$script" 2>&1) || result=$?
    else
        echo "No check defined for $rail"
        return 2
    fi

    # Evaluate result
    local passed=false

    if [[ -n "$expect_exit" ]]; then
        [[ "$result" == "$expect_exit" ]] && passed=true
    elif [[ -n "$expect_output" ]]; then
        echo "$output" | grep -q "$expect_output" && passed=true
    else
        [[ "$result" == "0" ]] && passed=true
    fi

    if $passed; then
        echo "pass" > "$STATE_DIR/rail_$rail"
        return 0
    else
        echo "fail" > "$STATE_DIR/rail_$rail"

        # Try auto-fix if enabled
        if [[ "$auto_fix" == "true" ]] && [[ -n "$fix_cmd" ]]; then
            echo -e "${YELLOW}Attempting auto-fix: $fix_cmd${NC}"
            if eval "$fix_cmd" 2>&1; then
                # Re-run check
                if [[ -n "$command" ]]; then
                    eval "$command" >/dev/null 2>&1 && return 0
                fi
            fi
        fi

        # Return based on severity
        if [[ "$severity" == "warning" ]]; then
            return 3  # Warning
        else
            return 1  # Fail
        fi
    fi
}

# ============================================
# RUN RAIL CATEGORIES
# ============================================

run_category() {
    local category="$1"
    local auto_fix="${2:-false}"
    local yaml_file="$RAILS_DIR/${category}.yaml"

    if [[ ! -f "$yaml_file" ]]; then
        echo -e "${YELLOW}No rails defined for category: $category${NC}"
        return 0
    fi

    echo ""
    echo -e "${BLUE}Running $category rails...${NC}"
    echo ""

    local total=0
    local passed=0
    local failed=0
    local skipped=0
    local warnings=0

    for rail in $(yaml_list_rails "$yaml_file"); do
        total=$((total + 1))
        local description=$(get_rail_prop "$yaml_file" "$rail" "description")

        printf "  %-25s " "$rail"

        execute_rail "$yaml_file" "$rail" "$auto_fix"
        local result=$?

        case $result in
            0)
                echo -e "${GREEN}PASS${NC}"
                passed=$((passed + 1))
                ;;
            1)
                local msg=$(get_rail_prop "$yaml_file" "$rail" "message")
                local fix=$(get_rail_prop "$yaml_file" "$rail" "fix")
                echo -e "${RED}FAIL${NC} - $msg"
                [[ -n "$fix" ]] && echo -e "                              ${YELLOW}Fix: $fix${NC}"
                failed=$((failed + 1))
                ;;
            2)
                echo -e "${BLUE}SKIP${NC}"
                skipped=$((skipped + 1))
                ;;
            3)
                local msg=$(get_rail_prop "$yaml_file" "$rail" "message")
                echo -e "${YELLOW}WARN${NC} - $msg"
                warnings=$((warnings + 1))
                ;;
        esac
    done

    echo ""
    echo "  Total: $total | Pass: $passed | Fail: $failed | Skip: $skipped | Warn: $warnings"

    [[ $failed -gt 0 ]] && return 1
    return 0
}

# ============================================
# RUN ALL APPLICABLE RAILS
# ============================================

run_all() {
    local auto_fix="${1:-false}"
    local project_types=$(detect_project_types)

    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}              ARIA Rails Executor                           ${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Project types: $project_types"
    echo "Auto-fix: $auto_fix"

    local failed=0

    # Run each category
    for category in environment quality safety documentation; do
        if [[ -f "$RAILS_DIR/${category}.yaml" ]]; then
            run_category "$category" "$auto_fix" || failed=$((failed + 1))
        fi
    done

    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"

    if [[ $failed -gt 0 ]]; then
        echo -e "${RED}Some rails failed${NC}"
        return 1
    else
        echo -e "${GREEN}All rails passed${NC}"
        return 0
    fi
}

# ============================================
# SINGLE RAIL RUNNER
# ============================================

run_single() {
    local rail_name="$1"
    local auto_fix="${2:-false}"

    # Search for rail in all YAML files
    for yaml_file in "$RAILS_DIR"/*.yaml; do
        if yaml_list_rails "$yaml_file" | grep -q "^${rail_name}$"; then
            local description=$(get_rail_prop "$yaml_file" "$rail_name" "description")
            echo "Running: $rail_name - $description"
            echo ""

            execute_rail "$yaml_file" "$rail_name" "$auto_fix"
            local result=$?

            case $result in
                0) echo -e "${GREEN}PASS${NC}" ;;
                1)
                    local msg=$(get_rail_prop "$yaml_file" "$rail_name" "message")
                    local fix=$(get_rail_prop "$yaml_file" "$rail_name" "fix")
                    echo -e "${RED}FAIL${NC} - $msg"
                    [[ -n "$fix" ]] && echo "Fix: $fix"
                    return 1
                    ;;
                2) echo -e "${BLUE}SKIP${NC} - Not applicable" ;;
                3)
                    local msg=$(get_rail_prop "$yaml_file" "$rail_name" "message")
                    echo -e "${YELLOW}WARN${NC} - $msg"
                    ;;
            esac
            return 0
        fi
    done

    echo -e "${RED}Rail not found: $rail_name${NC}"
    return 1
}

# ============================================
# LIST AVAILABLE RAILS
# ============================================

list_rails() {
    echo ""
    echo "Available Rails:"
    echo ""

    for yaml_file in "$RAILS_DIR"/*.yaml; do
        local category=$(basename "$yaml_file" .yaml)
        echo -e "${BLUE}[$category]${NC}"

        for rail in $(yaml_list_rails "$yaml_file"); do
            local description=$(get_rail_prop "$yaml_file" "$rail" "description")
            printf "  %-25s %s\n" "$rail" "$description"
        done
        echo ""
    done
}

# ============================================
# MAIN
# ============================================

main() {
    local command="${1:-help}"
    shift || true

    case "$command" in
        "all")
            run_all "${1:-false}"
            ;;
        "category"|"cat")
            run_category "$1" "${2:-false}"
            ;;
        "run"|"check")
            run_single "$1" "${2:-false}"
            ;;
        "fix")
            # Run with auto-fix enabled
            if [[ -n "$1" ]]; then
                run_single "$1" "true"
            else
                run_all "true"
            fi
            ;;
        "list"|"ls")
            list_rails
            ;;
        "detect")
            echo "Detected project types:"
            detect_project_types
            ;;
        "help"|*)
            echo "ARIA Rails Executor - Run rails from YAML definitions"
            echo ""
            echo "Usage: $0 <command> [args]"
            echo ""
            echo "Commands:"
            echo "  all [--fix]           - Run all applicable rails"
            echo "  category <name>       - Run rails in category (environment, quality, safety, docs)"
            echo "  run <rail>            - Run a specific rail"
            echo "  fix [rail]            - Run with auto-fix enabled"
            echo "  list                  - List all available rails"
            echo "  detect                - Show detected project types"
            echo ""
            echo "Categories:"
            echo "  environment  - Node, deps, server, DB checks"
            echo "  quality      - Tests, lint, types, build"
            echo "  safety       - Secrets, destructive commands"
            echo "  documentation - README, changelog, API docs"
            ;;
    esac
}

main "$@"
