#!/bin/bash
# ARIA Rails Executor
# Parses JSON rail definitions and executes them
# Supports auto-fix when rails fail

# Exit on error, undefined vars, and pipeline failures
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh" || { echo "Failed to load common.sh"; exit 1; }

# Check dependencies
aria_check_deps jq || exit 1

RAILS_DIR="$SCRIPT_DIR/rails"
STATE_DIR="$SCRIPT_DIR/state"
LOGS_DIR="$SCRIPT_DIR/logs"
SIGNALS_FILE="$STATE_DIR/signals.jsonl"

# Colors from common.sh
RED="$ARIA_RED"
GREEN="$ARIA_GREEN"
YELLOW="$ARIA_YELLOW"
BLUE="$ARIA_BLUE"
NC="$ARIA_NC"

mkdir -p "$STATE_DIR" "$LOGS_DIR"

# ============================================
# SECURITY: Command Validation (Issue #9 fix)
# ============================================

# Dangerous patterns that should never be in rail commands
DANGEROUS_PATTERNS=(
    'rm -rf /'
    'rm -rf ~'
    'rm -rf \*'
    '> /dev/sd'
    'mkfs\.'
    'dd if=.* of=/dev'
    ':(){ :|:& };:'
    'chmod -R 777 /'
    '\$\(.*\)'        # Command substitution
    '\`.*\`'          # Backtick execution
    'curl.*\|.*sh'    # Pipe curl to shell
    'wget.*\|.*sh'    # Pipe wget to shell
    'eval '           # Nested eval
    'exec '           # Process replacement
)

# Validate command is safe to execute
validate_command() {
    local cmd="$1"
    local context="$2"

    for pattern in "${DANGEROUS_PATTERNS[@]}"; do
        if echo "$cmd" | grep -qE "$pattern"; then
            echo -e "${RED}SECURITY: Dangerous pattern detected in $context${NC}"
            echo -e "${RED}Pattern: $pattern${NC}"
            echo -e "${RED}Command blocked for safety${NC}"
            _log_rail_signal "blocked" "$context" "dangerous_pattern" "$pattern"
            return 1
        fi
    done

    return 0
}

# Log rail execution to signals.jsonl for traceability
_log_rail_signal() {
    local event="$1"      # check_pass, check_fail, autofix_attempt, blocked
    local rail_id="$2"
    local status="$3"
    local details="${4:-}"
    local timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    local event_id="rail-$(date +%s%N | cut -c1-13)"

    mkdir -p "$(dirname "$SIGNALS_FILE")" 2>/dev/null || true

    # Escape quotes
    details="${details//\"/\\\"}"

    printf '{"id":"%s","timestamp":"%s","event":"rail_%s","rail_id":"%s","status":"%s","details":"%s","context_type":"rail","context_name":"executor"}\n' \
        "$event_id" "$timestamp" "$event" "$rail_id" "$status" "$details" \
        >> "$SIGNALS_FILE" 2>/dev/null || true
}

# Safe command execution (replaces eval)
safe_execute() {
    local cmd="$1"
    local context="$2"

    # Validate first
    if ! validate_command "$cmd" "$context"; then
        return 1
    fi

    # Execute in subshell with bash -c (safer than eval)
    # This runs in a child process, limiting damage potential
    bash -c "$cmd" >/dev/null 2>&1
}

# ============================================
# RAIL EXECUTION
# ============================================

# Execute a single rail
execute_rail() {
    local rail_json="$1"

    local id=$(echo "$rail_json" | jq -r '.id')
    local description=$(echo "$rail_json" | jq -r '.description')
    local type=$(echo "$rail_json" | jq -r '.type // "soft"')
    local check=$(echo "$rail_json" | jq -r '.check')
    local message=$(echo "$rail_json" | jq -r '.message // "Rail failed"')
    local auto_fix=$(echo "$rail_json" | jq -r '.auto_fix // ""')

    echo -n "  [$id] $description... "

    # Execute check using safe_execute (not eval)
    if safe_execute "$check" "$id"; then
        echo -e "${GREEN}PASS${NC}"
        _log_rail_signal "check" "$id" "pass" "$description"
        return 0
    else
        _log_rail_signal "check" "$id" "fail" "$description"

        if [[ "$type" == "hard" ]]; then
            echo -e "${RED}BLOCKED${NC}"
            echo "    → $message"

            # Try auto-fix if available
            if [[ -n "$auto_fix" && "$auto_fix" != "null" ]]; then
                echo -e "    ${YELLOW}Attempting auto-fix...${NC}"
                _log_rail_signal "autofix" "$id" "attempt" ""

                if safe_execute "$auto_fix" "${id}_autofix"; then
                    # Re-check after fix
                    if safe_execute "$check" "${id}_recheck"; then
                        echo -e "    ${GREEN}Auto-fix successful${NC}"
                        _log_rail_signal "autofix" "$id" "success" ""
                        return 0
                    fi
                fi
                echo -e "    ${RED}Auto-fix failed${NC}"
                _log_rail_signal "autofix" "$id" "failed" ""
            fi

            return 1
        else
            echo -e "${YELLOW}WARN${NC}"
            echo "    → $message"
            return 0
        fi
    fi
}

# Execute all rails from a JSON file
execute_rails_file() {
    local rails_file="$1"

    if [[ ! -f "$rails_file" ]]; then
        echo -e "${RED}Rails file not found: $rails_file${NC}"
        return 1
    fi

    local filename=$(basename "$rails_file")
    echo ""
    echo -e "${BLUE}Executing rails: $filename${NC}"
    echo ""

    local failed=0
    local total=$(jq '.rails | length' "$rails_file")

    for i in $(seq 0 $((total - 1))); do
        local rail=$(jq ".rails[$i]" "$rails_file")
        if ! execute_rail "$rail"; then
            failed=$((failed + 1))
        fi
    done

    echo ""
    if [[ $failed -gt 0 ]]; then
        echo -e "${RED}$failed rail(s) blocked execution${NC}"
        return 1
    else
        echo -e "${GREEN}All rails passed${NC}"
        return 0
    fi
}

# Execute all rails in a directory
execute_all_rails() {
    local dir="${1:-$RAILS_DIR}"
    local failed=0

    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}                    ARIA Rails Executor                     ${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"

    for rails_file in "$dir"/*.json; do
        [[ -f "$rails_file" ]] || continue
        if ! execute_rails_file "$rails_file"; then
            failed=$((failed + 1))
        fi
    done

    echo ""
    if [[ $failed -gt 0 ]]; then
        echo -e "${RED}Some rails failed. Execution blocked.${NC}"
        return 1
    else
        echo -e "${GREEN}All rails passed. Proceeding.${NC}"
        return 0
    fi
}

# ============================================
# CLI
# ============================================

case "${1:-all}" in
    "all")
        execute_all_rails "${2:-$RAILS_DIR}"
        ;;
    "file")
        if [[ -z "$2" ]]; then
            echo "Usage: $0 file <rails-file.json>"
            exit 1
        fi
        execute_rails_file "$2"
        ;;
    "help"|*)
        echo "ARIA Rails Executor"
        echo ""
        echo "Usage: $0 <command> [args]"
        echo ""
        echo "Commands:"
        echo "  all [dir]       Execute all rails in directory (default: .aria/rails)"
        echo "  file <file>     Execute rails from specific JSON file"
        echo "  help            Show this help"
        echo ""
        echo "Rail JSON format:"
        echo '  {'
        echo '    "rails": ['
        echo '      {'
        echo '        "id": "unique_id",'
        echo '        "description": "What this checks",'
        echo '        "type": "hard|soft",'
        echo '        "check": "shell command (0=pass)",'
        echo '        "message": "Error message",'
        echo '        "auto_fix": "optional fix command"'
        echo '      }'
        echo '    ]'
        echo '  }'
        ;;
esac
