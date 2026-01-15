#!/bin/bash
# ARIA Common Functions
# Source this file in other scripts: source "$(dirname "$0")/common.sh"

# Colors
export ARIA_RED='\033[0;31m'
export ARIA_GREEN='\033[0;32m'
export ARIA_YELLOW='\033[1;33m'
export ARIA_BLUE='\033[0;34m'
export ARIA_MAGENTA='\033[0;35m'
export ARIA_NC='\033[0m'

# ============================================
# WINDOWS COMPATIBILITY CHECK (Issue #6)
# ============================================
# ARIA requires a Unix-like shell environment.
# This function detects unsupported Windows environments
# and provides clear guidance to users.

aria_check_windows_compat() {
    # Check if we're on Windows (MSYS, Cygwin, or Git Bash are OK)
    case "$(uname -s)" in
        MINGW*|MSYS*|CYGWIN*)
            # Git Bash, MSYS2, or Cygwin - these work fine
            return 0
            ;;
        Linux)
            # Could be WSL - check for it
            if grep -qi microsoft /proc/version 2>/dev/null; then
                # WSL detected - works fine
                return 0
            fi
            # Native Linux - works fine
            return 0
            ;;
        Darwin)
            # macOS - works fine
            return 0
            ;;
        *)
            # Unknown - assume it might work
            return 0
            ;;
    esac
}

# Print Windows compatibility message and exit
# Called when scripts detect they're running in an unsupported environment
aria_windows_unsupported() {
    cat << 'EOF'
╔═══════════════════════════════════════════════════════════════════════════════╗
║                        ARIA: Windows Compatibility Notice                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║  ARIA requires a Unix-like shell environment to run.                          ║
║                                                                               ║
║  RECOMMENDED OPTIONS:                                                         ║
║                                                                               ║
║  1. Git Bash (easiest)                                                        ║
║     - Install Git for Windows: https://git-scm.com/download/win              ║
║     - Open "Git Bash" from Start menu                                         ║
║     - Run ARIA scripts from there                                             ║
║                                                                               ║
║  2. Windows Subsystem for Linux (WSL)                                         ║
║     - Open PowerShell as Administrator                                        ║
║     - Run: wsl --install                                                      ║
║     - Restart and open Ubuntu from Start menu                                 ║
║                                                                               ║
║  3. VS Code with Git Bash Terminal                                            ║
║     - Open VS Code settings (Ctrl+,)                                          ║
║     - Search "terminal.integrated.defaultProfile.windows"                     ║
║     - Set to "Git Bash"                                                       ║
║                                                                               ║
║  For Claude Code in VS Code:                                                  ║
║     Configure VS Code to use Git Bash or WSL as the integrated terminal.      ║
║     Claude Code will then execute ARIA scripts correctly.                     ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
EOF
    exit 1
}

# Auto-check on source (only if ARIA_SKIP_WINDOWS_CHECK is not set)
if [[ -z "${ARIA_SKIP_WINDOWS_CHECK:-}" ]]; then
    # If running in CMD or PowerShell directly, $BASH won't be set properly
    # and common bash features won't work
    if [[ -z "${BASH_VERSION:-}" ]]; then
        aria_windows_unsupported
    fi
fi

# Check for required dependencies
# Usage: aria_check_deps cmd1 cmd2 cmd3
aria_check_deps() {
    local missing=()
    for cmd in "$@"; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            missing+=("$cmd")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        echo -e "${ARIA_RED}ARIA: Missing required tools: ${missing[*]}${ARIA_NC}" >&2
        echo "" >&2
        echo "Install with:" >&2
        echo "  macOS:  brew install ${missing[*]}" >&2
        echo "  Ubuntu: sudo apt-get install ${missing[*]}" >&2
        echo "  Alpine: apk add ${missing[*]}" >&2
        return 1
    fi
    return 0
}

# Check minimum dependencies for ARIA
aria_check_core_deps() {
    aria_check_deps git jq
}

# Log with timestamp
aria_log() {
    echo "[$(date '+%H:%M:%S')] $*"
}

# Error and exit
aria_error() {
    echo -e "${ARIA_RED}ERROR: $*${ARIA_NC}" >&2
    exit 1
}

# Warning (continues)
aria_warn() {
    echo -e "${ARIA_YELLOW}WARNING: $*${ARIA_NC}" >&2
}

# Success message
aria_success() {
    echo -e "${ARIA_GREEN}$*${ARIA_NC}"
}

# Info message
aria_info() {
    echo -e "${ARIA_BLUE}$*${ARIA_NC}"
}

# Get ARIA directory (where scripts live)
aria_get_dir() {
    local script_path="${BASH_SOURCE[1]:-$0}"
    cd "$(dirname "$script_path")" && pwd
}

# Get project root (parent of .aria)
aria_get_project_root() {
    local aria_dir
    aria_dir="$(aria_get_dir)"
    dirname "$aria_dir"
}

# ============================================
# FILE OWNERSHIP MODEL (Issue #7)
# ============================================
# Single-writer pattern: Each state file has ONE owner function.
# Non-owners must call the owner function to write.
# This prevents race conditions by design (sequential writes only).
#
# File Owners:
#   signals.jsonl  → emit_signal()
#   decisions.jsonl → emit_decision()
#
# Why this pattern:
#   - JSONL appends are inherently atomic (single write operation)
#   - Centralized write logic ensures consistent schema
#   - No need for flock complexity
#   - Better traceability (all writes go through one path)
# ============================================

# State file paths (can be overridden)
ARIA_STATE_DIR="${ARIA_STATE_DIR:-$(dirname "${BASH_SOURCE[0]}")/state}"
ARIA_SIGNALS_FILE="${ARIA_SIGNALS_FILE:-$ARIA_STATE_DIR/signals.jsonl}"
ARIA_DECISIONS_FILE="${ARIA_DECISIONS_FILE:-$ARIA_STATE_DIR/decisions.jsonl}"

# ============================================
# emit_signal - SINGLE OWNER of signals.jsonl
# ============================================
# Usage: emit_signal EVENT CONTEXT_TYPE CONTEXT_NAME [key=value ...]
#
# Required:
#   EVENT        - Event name (e.g., "hitl_request_created", "session_started")
#   CONTEXT_TYPE - Category (e.g., "hitl", "session", "rail", "skill")
#   CONTEXT_NAME - Specific context (e.g., "human_intervention", "planning")
#
# Optional key=value pairs are added to JSON object.
# Special handling for numeric values (no quotes if value is a number).
#
# Example:
#   emit_signal "hitl_request_created" "hitl" "human_intervention" \
#       "request_id=req-123" "request_type=confirm" "timeout=30"
#
# Returns: 0 on success, 1 on failure
emit_signal() {
    local event="${1:-}"
    local context_type="${2:-}"
    local context_name="${3:-}"

    # Validate required fields
    if [[ -z "$event" || -z "$context_type" || -z "$context_name" ]]; then
        echo "emit_signal: requires EVENT CONTEXT_TYPE CONTEXT_NAME" >&2
        return 1
    fi

    shift 3 2>/dev/null || true

    # Generate unique ID and timestamp
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local event_id="sig-$(date +%s%N 2>/dev/null | cut -c1-13 || date +%s)-$$"

    # Ensure state directory exists
    mkdir -p "$ARIA_STATE_DIR" 2>/dev/null || true

    # Build JSON object
    local json="{\"id\":\"${event_id}\",\"timestamp\":\"${timestamp}\",\"event\":\"${event}\""

    # Add optional key=value pairs
    for kv in "$@"; do
        local key="${kv%%=*}"
        local value="${kv#*=}"

        # Escape quotes in value
        value="${value//\\/\\\\}"
        value="${value//\"/\\\"}"

        # Check if value is numeric (integer or float)
        if [[ "$value" =~ ^-?[0-9]+\.?[0-9]*$ ]]; then
            json="${json},\"${key}\":${value}"
        else
            json="${json},\"${key}\":\"${value}\""
        fi
    done

    # Add context fields at the end
    json="${json},\"context_type\":\"${context_type}\",\"context_name\":\"${context_name}\"}"

    # Atomic append (single write operation)
    echo "$json" >> "$ARIA_SIGNALS_FILE" 2>/dev/null
    return $?
}

# ============================================
# emit_decision - SINGLE OWNER of decisions.jsonl
# ============================================
# Usage: emit_decision ACTION CONTEXT RATIONALE ALTERNATIVES CONFIDENCE [VERIFIED]
#
# Required:
#   ACTION       - What was decided/done
#   CONTEXT      - What was looked at to decide
#   RATIONALE    - Why this approach
#   ALTERNATIVES - What else was considered
#   CONFIDENCE   - 0.0-1.0 confidence score
#
# Optional:
#   VERIFIED     - true/false/null (default: null)
#
# Example:
#   emit_decision "Add retry wrapper to API client" \
#       "Read utils/retry.ts, saw 3 similar uses" \
#       "Consistency with existing patterns" \
#       "Custom retry logic, no retry" \
#       "0.85"
#
# Returns: 0 on success, 1 on failure
emit_decision() {
    local action="${1:-}"
    local context="${2:-}"
    local rationale="${3:-}"
    local alternatives="${4:-}"
    local confidence="${5:-}"
    local verified="${6:-null}"

    # Validate required fields
    if [[ -z "$action" || -z "$confidence" ]]; then
        echo "emit_decision: requires ACTION and CONFIDENCE at minimum" >&2
        return 1
    fi

    # Validate confidence is numeric (0.0-1.0 range)
    if ! [[ "$confidence" =~ ^[0-9]*\.?[0-9]+$ ]]; then
        echo "emit_decision: CONFIDENCE must be 0.0-1.0, got: $confidence" >&2
        return 1
    fi

    # Check range (use awk for floating point comparison)
    local in_range
    in_range=$(awk -v c="$confidence" 'BEGIN { print (c >= 0 && c <= 1) ? "yes" : "no" }')
    if [[ "$in_range" != "yes" ]]; then
        echo "emit_decision: CONFIDENCE must be 0.0-1.0, got: $confidence" >&2
        return 1
    fi

    # Generate unique ID and timestamp
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local decision_id="dec-$(date +%s%N 2>/dev/null | cut -c1-13 || date +%s)-$$"

    # Ensure state directory exists
    mkdir -p "$ARIA_STATE_DIR" 2>/dev/null || true

    # Escape quotes in string fields
    action="${action//\\/\\\\}"
    action="${action//\"/\\\"}"
    context="${context//\\/\\\\}"
    context="${context//\"/\\\"}"
    rationale="${rationale//\\/\\\\}"
    rationale="${rationale//\"/\\\"}"
    alternatives="${alternatives//\\/\\\\}"
    alternatives="${alternatives//\"/\\\"}"

    # Build JSON object
    local json="{\"id\":\"${decision_id}\",\"timestamp\":\"${timestamp}\""
    json="${json},\"action\":\"${action}\""
    json="${json},\"context\":\"${context}\""
    json="${json},\"rationale\":\"${rationale}\""
    json="${json},\"alternatives\":\"${alternatives}\""
    json="${json},\"confidence\":${confidence}"

    # Handle verified field (boolean or null)
    if [[ "$verified" == "true" || "$verified" == "false" ]]; then
        json="${json},\"verified\":${verified}"
    else
        json="${json},\"verified\":null"
    fi

    json="${json}}"

    # Atomic append (single write operation)
    echo "$json" >> "$ARIA_DECISIONS_FILE" 2>/dev/null
    return $?
}

# ============================================
# ATOMIC FILE UTILITIES
# ============================================
# These are general utilities, not owners of specific files.
# Use for files that don't need the ownership pattern.

# Atomically write content to a file (write to temp, then move)
# Usage: echo "content" | aria_atomic_write /path/to/file
# Returns: 0 on success, 1 on failure
aria_atomic_write() {
    local target_file="$1"
    local tmp_file
    local tmp_dir

    # Use same directory for temp to ensure atomic mv (same filesystem)
    tmp_dir="$(dirname "$target_file")"
    tmp_file="$tmp_dir/.tmp.$(basename "$target_file").$$"

    # Ensure directory exists
    mkdir -p "$tmp_dir" 2>/dev/null || true

    # Read stdin to temp file
    if cat > "$tmp_file"; then
        # Atomic move
        if mv "$tmp_file" "$target_file"; then
            return 0
        else
            rm -f "$tmp_file" 2>/dev/null
            return 1
        fi
    else
        rm -f "$tmp_file" 2>/dev/null
        return 1
    fi
}
