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
# SILENT ERROR HANDLING (Issue #3)
# ============================================
# Captures errors for traceability while suppressing user-facing noise.
# Instead of `command 2>/dev/null`, use `aria_silent command` or `aria_try command`.
#
# Benefits:
#   - Errors are logged to debug file for troubleshooting
#   - Commands still suppress stderr from terminal
#   - Optional: Emit signals on failure for full traceability

# Debug log location (disabled by default to avoid noise)
ARIA_DEBUG_LOG="${ARIA_DEBUG_LOG:-}"
ARIA_DEBUG_LEVEL="${ARIA_DEBUG_LEVEL:-0}"  # 0=off, 1=errors, 2=all

# Internal: Write to debug log if enabled
_aria_debug_log() {
    if [[ -n "$ARIA_DEBUG_LOG" && -n "$1" ]]; then
        local timestamp
        timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
        mkdir -p "$(dirname "$ARIA_DEBUG_LOG")" 2>/dev/null || true
        echo "[$timestamp] $*" >> "$ARIA_DEBUG_LOG"
    fi
}

# Run a command silently but capture stderr for debugging
# Usage: aria_silent command [args...]
# Returns: Exit code of command
# Stderr is captured to debug log (if ARIA_DEBUG_LOG is set)
#
# Example:
#   aria_silent git status          # Instead of: git status 2>/dev/null
#   aria_silent rm -f "$file"       # Instead of: rm -f "$file" 2>/dev/null
aria_silent() {
    local stderr_capture
    local exit_code

    if [[ -n "$ARIA_DEBUG_LOG" && "$ARIA_DEBUG_LEVEL" -ge 1 ]]; then
        # Capture stderr while suppressing it
        stderr_capture=$("$@" 2>&1 >/dev/null)
        exit_code=$?
        if [[ $exit_code -ne 0 && -n "$stderr_capture" ]]; then
            _aria_debug_log "STDERR [$exit_code]: $* -> $stderr_capture"
        fi
    else
        # Fast path: just suppress stderr
        "$@" 2>/dev/null
        exit_code=$?
    fi

    return $exit_code
}

# Try to run a command, returning success/failure
# Usage: aria_try command [args...] && echo "success" || echo "failed"
# Returns: Exit code of command
# Does NOT suppress stderr (use aria_silent for that)
#
# Example:
#   if aria_try command -v jq; then
#       echo "jq is installed"
#   fi
aria_try() {
    "$@"
    return $?
}

# Run a command silently, emit signal on failure (for traceability)
# Usage: aria_silent_traced "operation_name" command [args...]
# Returns: Exit code of command
#
# Example:
#   aria_silent_traced "cleanup_temp" rm -rf /tmp/aria-*
aria_silent_traced() {
    local operation_name="$1"
    shift

    local stderr_capture
    local exit_code

    # Capture stderr
    stderr_capture=$("$@" 2>&1 >/dev/null)
    exit_code=$?

    if [[ $exit_code -ne 0 ]]; then
        # Log to debug file
        _aria_debug_log "TRACED_FAILURE: $operation_name -> exit=$exit_code stderr=$stderr_capture"

        # Emit signal for traceability (if emit_signal is available)
        if type emit_signal >/dev/null 2>&1; then
            emit_signal "silent_operation_failed" "debug" "$operation_name" \
                "exit_code=$exit_code" \
                "command=$1"
        fi
    fi

    return $exit_code
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

# ============================================
# STATE FILE CLEANUP (Issue #17)
# ============================================
# Manages retention of old state files to prevent disk bloat.
# All deletions are logged for traceability.

# Default retention settings (can be overridden)
ARIA_RETENTION_COUNT="${ARIA_RETENTION_COUNT:-10}"      # Keep last N files
ARIA_RETENTION_DAYS="${ARIA_RETENTION_DAYS:-30}"        # Or files newer than N days

# Clean up old files in a directory, keeping the most recent N
# Usage: aria_cleanup_by_count /path/to/dir "pattern" [keep_count]
# Example: aria_cleanup_by_count "$STATE_DIR/handoffs" "handoff-*.md" 5
aria_cleanup_by_count() {
    local dir="$1"
    local pattern="$2"
    local keep="${3:-$ARIA_RETENTION_COUNT}"

    if [[ ! -d "$dir" ]]; then
        return 0
    fi

    # Count matching files
    local count
    count=$(find "$dir" -maxdepth 1 -name "$pattern" -type f 2>/dev/null | wc -l)

    if [[ "$count" -le "$keep" ]]; then
        return 0  # Nothing to clean
    fi

    local to_delete=$((count - keep))
    local deleted=0

    # Delete oldest files (by modification time)
    while IFS= read -r file; do
        if [[ -f "$file" ]]; then
            rm -f "$file"
            ((deleted++))
        fi
    done < <(find "$dir" -maxdepth 1 -name "$pattern" -type f -printf '%T@ %p\n' 2>/dev/null | \
             sort -n | head -n "$to_delete" | cut -d' ' -f2-)

    if [[ "$deleted" -gt 0 ]]; then
        emit_signal "cleanup_by_count" "maintenance" "retention" \
            "dir=$dir" \
            "pattern=$pattern" \
            "deleted=$deleted" \
            "retained=$keep"
    fi

    return 0
}

# Clean up files older than N days
# Usage: aria_cleanup_by_age /path/to/dir "pattern" [days]
aria_cleanup_by_age() {
    local dir="$1"
    local pattern="$2"
    local days="${3:-$ARIA_RETENTION_DAYS}"

    if [[ ! -d "$dir" ]]; then
        return 0
    fi

    local deleted=0

    while IFS= read -r file; do
        if [[ -f "$file" ]]; then
            rm -f "$file"
            ((deleted++))
        fi
    done < <(find "$dir" -maxdepth 1 -name "$pattern" -type f -mtime "+$days" 2>/dev/null)

    if [[ "$deleted" -gt 0 ]]; then
        emit_signal "cleanup_by_age" "maintenance" "retention" \
            "dir=$dir" \
            "pattern=$pattern" \
            "deleted=$deleted" \
            "older_than_days=$days"
    fi

    return 0
}

# Clean up stale lock files (older than 1 hour)
aria_cleanup_stale_locks() {
    local state_dir="${ARIA_STATE_DIR:-$(dirname "${BASH_SOURCE[0]}")/state}"

    local deleted=0
    while IFS= read -r file; do
        if [[ -f "$file" ]]; then
            rm -f "$file"
            ((deleted++))
        fi
    done < <(find "$state_dir" -maxdepth 1 -name "*.lock" -type f -mmin +60 2>/dev/null)

    if [[ "$deleted" -gt 0 ]]; then
        emit_signal "cleanup_stale_locks" "maintenance" "retention" \
            "deleted=$deleted"
    fi
}

# Run all cleanup tasks
# Usage: aria_run_cleanup [--dry-run]
aria_run_cleanup() {
    local dry_run=""
    [[ "${1:-}" == "--dry-run" ]] && dry_run="true"

    local aria_dir="${ARIA_STATE_DIR:-$(dirname "${BASH_SOURCE[0]}")}"
    local state_dir="$aria_dir/state"
    local logs_dir="$aria_dir/logs"

    if [[ -n "$dry_run" ]]; then
        echo "DRY RUN - Would clean:"
        echo "  Handoffs: $(find "$state_dir/handoffs" -name "handoff-*.md" 2>/dev/null | wc -l) files (keep last $ARIA_RETENTION_COUNT)"
        echo "  Usage logs: $(find "$logs_dir" -name "token_usage_*.json" 2>/dev/null | wc -l) files"
        echo "  Failure logs: $(find "$logs_dir" -name "story_failures_*.log" 2>/dev/null | wc -l) files"
        echo "  Lock files: $(find "$state_dir" -name "*.lock" -mmin +60 2>/dev/null | wc -l) stale"
        return 0
    fi

    # Clean handoffs (keep last N)
    aria_cleanup_by_count "$state_dir/handoffs" "handoff-*.md" "$ARIA_RETENTION_COUNT"

    # Clean old usage logs
    aria_cleanup_by_count "$logs_dir" "token_usage_*.json" "$ARIA_RETENTION_COUNT"

    # Clean old failure logs
    aria_cleanup_by_count "$logs_dir" "story_failures_*.log" "$ARIA_RETENTION_COUNT"

    # Clean stale lock files
    aria_cleanup_stale_locks

    # Clean old signals (by age - keep 30 days worth)
    # Note: signals.jsonl itself is not cleaned, only archived versions
    aria_cleanup_by_age "$state_dir" "signals-*.jsonl.bak" "$ARIA_RETENTION_DAYS"
}
