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

# ============================================
# STANDARDIZED MESSAGE FUNCTIONS (Issue #2)
# ============================================
# Use these consistently instead of direct echo with colors.
# Benefits:
#   - Consistent formatting across all scripts
#   - Automatic signal emission for traceability
#   - Proper stderr handling for errors/warnings
#
# Usage Guide:
#   aria_error "message"     # Fatal error, exits with code 1
#   aria_fail "message"      # Non-fatal failure, continues execution
#   aria_warn "message"      # Warning, continues execution
#   aria_success "message"   # Success feedback
#   aria_info "message"      # Informational message
#   aria_debug "message"     # Debug (only if ARIA_DEBUG=1)
#   aria_status "OK" "msg"   # Status with OK/FAIL indicator

# Fatal error - logs, emits signal, and exits
# Usage: aria_error "Description of error"
aria_error() {
    local message="$*"
    echo -e "${ARIA_RED}✗ ERROR: ${message}${ARIA_NC}" >&2

    # Emit signal for traceability (suppress failures)
    emit_signal "error" "message" "fatal" "message=$message" 2>/dev/null || true

    exit 1
}

# Non-fatal failure - logs but continues
# Usage: aria_fail "Description of failure"
aria_fail() {
    local message="$*"
    echo -e "${ARIA_RED}✗ FAILED: ${message}${ARIA_NC}" >&2

    # Emit signal for traceability (suppress failures)
    emit_signal "failure" "message" "non_fatal" "message=$message" 2>/dev/null || true
}

# Warning - logs but continues
# Usage: aria_warn "Description of warning"
aria_warn() {
    local message="$*"
    echo -e "${ARIA_YELLOW}⚠ WARNING: ${message}${ARIA_NC}" >&2
}

# Success message
# Usage: aria_success "Description of success"
aria_success() {
    local message="$*"
    echo -e "${ARIA_GREEN}✓ ${message}${ARIA_NC}"
}

# Info message
# Usage: aria_info "Informational message"
aria_info() {
    local message="$*"
    echo -e "${ARIA_BLUE}ℹ ${message}${ARIA_NC}"
}

# Debug message (only shown if ARIA_DEBUG=1)
# Usage: aria_debug "Debug details"
aria_debug() {
    if [[ "${ARIA_DEBUG:-0}" == "1" ]]; then
        echo -e "${ARIA_MAGENTA}[DEBUG] $*${ARIA_NC}" >&2
    fi
}

# Status line with indicator
# Usage: aria_status "OK" "Component is working"
#        aria_status "FAIL" "Component failed"
#        aria_status "WARN" "Component has issues"
aria_status() {
    local status="$1"
    local message="$2"

    case "$status" in
        "OK"|"PASS"|"SUCCESS")
            echo -e "  ${ARIA_GREEN}[OK]${ARIA_NC}   $message"
            ;;
        "FAIL"|"ERROR")
            echo -e "  ${ARIA_RED}[FAIL]${ARIA_NC} $message"
            ;;
        "WARN"|"WARNING")
            echo -e "  ${ARIA_YELLOW}[WARN]${ARIA_NC} $message"
            ;;
        "SKIP"|"SKIPPED")
            echo -e "  ${ARIA_YELLOW}[SKIP]${ARIA_NC} $message"
            ;;
        "INFO")
            echo -e "  ${ARIA_BLUE}[INFO]${ARIA_NC} $message"
            ;;
        *)
            echo -e "  [$status] $message"
            ;;
    esac
}

# HITL-style error block for critical failures
# Usage: aria_hitl_error "Title" "Details" "Options hint"
aria_hitl_error() {
    local title="$1"
    local details="${2:-}"
    local options="${3:-[r]etry / [s]kip / [a]bort}"

    echo "" >&2
    echo -e "${ARIA_RED}════════════════════════════════════════════════════════${ARIA_NC}" >&2
    echo -e "${ARIA_RED}  ${title}${ARIA_NC}" >&2
    echo -e "${ARIA_RED}════════════════════════════════════════════════════════${ARIA_NC}" >&2
    if [[ -n "$details" ]]; then
        echo "" >&2
        echo -e "  ${details}" >&2
    fi
    echo "" >&2
    echo -e "  Options: ${options}" >&2
    echo "" >&2
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
# TIMEOUT HANDLING (Issue #21)
# ============================================
# Provides configurable timeouts for hooks and operations.
# Emits signals on timeout for traceability.
#
# Timeout defaults (all in seconds, can be overridden via env vars):
ARIA_HOOK_TIMEOUT="${ARIA_HOOK_TIMEOUT:-60}"          # General hook timeout
ARIA_LINT_TIMEOUT="${ARIA_LINT_TIMEOUT:-120}"         # Linting operations
ARIA_TEST_TIMEOUT="${ARIA_TEST_TIMEOUT:-300}"         # Test suites
ARIA_BUILD_TIMEOUT="${ARIA_BUILD_TIMEOUT:-600}"       # Build operations
ARIA_E2E_TIMEOUT="${ARIA_E2E_TIMEOUT:-600}"           # E2E tests
ARIA_TYPECHECK_TIMEOUT="${ARIA_TYPECHECK_TIMEOUT:-120}" # TypeScript checks

# Check if timeout command is available
_aria_has_timeout() {
    command -v timeout >/dev/null 2>&1
}

# Run a command with timeout, emit signal on timeout
# Usage: aria_run_with_timeout TIMEOUT_SECS OPERATION_NAME command [args...]
# Returns: Command exit code, or 124 on timeout
#
# Example:
#   aria_run_with_timeout "$ARIA_TEST_TIMEOUT" "unit_tests" npm test
#   aria_run_with_timeout 60 "quick_lint" npm run lint
aria_run_with_timeout() {
    local timeout_secs="${1:-60}"
    local operation_name="${2:-unknown}"
    shift 2

    local exit_code=0
    local start_time
    start_time=$(date +%s)

    if _aria_has_timeout; then
        # Use timeout command (GNU coreutils)
        timeout --signal=TERM "$timeout_secs" "$@"
        exit_code=$?

        if [[ $exit_code -eq 124 ]]; then
            # Timeout occurred
            local elapsed=$(($(date +%s) - start_time))
            _aria_debug_log "TIMEOUT: $operation_name after ${elapsed}s (limit: ${timeout_secs}s)"

            # Emit signal for traceability
            if type emit_signal >/dev/null 2>&1; then
                emit_signal "operation_timeout" "timeout" "$operation_name" \
                    "timeout_secs=$timeout_secs" \
                    "elapsed_secs=$elapsed" \
                    "command=$1"
            fi

            echo -e "${ARIA_RED:-}⏱ TIMEOUT: $operation_name exceeded ${timeout_secs}s${ARIA_NC:-}" >&2
        fi
    else
        # Fallback: run without timeout but track duration
        "$@"
        exit_code=$?

        local elapsed=$(($(date +%s) - start_time))
        if [[ $elapsed -gt $timeout_secs ]]; then
            _aria_debug_log "SLOW_OPERATION: $operation_name took ${elapsed}s (limit: ${timeout_secs}s)"

            # Emit warning signal
            if type emit_signal >/dev/null 2>&1; then
                emit_signal "operation_slow" "timeout" "$operation_name" \
                    "timeout_secs=$timeout_secs" \
                    "elapsed_secs=$elapsed" \
                    "command=$1"
            fi
        fi
    fi

    return $exit_code
}

# Get appropriate timeout for an operation type
# Usage: timeout_secs=$(aria_get_timeout "test")
aria_get_timeout() {
    local operation_type="${1:-default}"

    case "$operation_type" in
        lint|linting)     echo "$ARIA_LINT_TIMEOUT" ;;
        test|tests|unit)  echo "$ARIA_TEST_TIMEOUT" ;;
        build|compile)    echo "$ARIA_BUILD_TIMEOUT" ;;
        e2e|integration)  echo "$ARIA_E2E_TIMEOUT" ;;
        typecheck|types)  echo "$ARIA_TYPECHECK_TIMEOUT" ;;
        hook|pre-commit)  echo "$ARIA_HOOK_TIMEOUT" ;;
        *)                echo "$ARIA_HOOK_TIMEOUT" ;;
    esac
}

# Convenience wrapper: run with auto-detected timeout
# Usage: aria_timed "test" npm test
aria_timed() {
    local operation_type="${1:-default}"
    shift
    local timeout_secs
    timeout_secs=$(aria_get_timeout "$operation_type")
    aria_run_with_timeout "$timeout_secs" "$operation_type" "$@"
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
# PRD VALIDATION (Issue #8)
# ============================================
# Validates PRD (Product Requirements Document) format before use.
# Prevents garbage-in-garbage-out by ensuring required fields exist.
#
# Required PRD structure:
#   {
#     "feature": "string",
#     "branchName": "string",
#     "userStories": [
#       {
#         "id": "US-XXX",
#         "title": "string",
#         "priority": number,
#         "passes": boolean,
#         "acceptanceCriteria": ["string", ...]
#       }
#     ]
#   }

# Validate PRD JSON file
# Usage: aria_validate_prd /path/to/prd.json
# Returns: 0 if valid, 1 if invalid (with error messages)
aria_validate_prd() {
    local prd_file="${1:-}"
    local errors=()
    local warnings=()

    # Check file exists
    if [[ -z "$prd_file" ]]; then
        echo "aria_validate_prd: No file specified" >&2
        return 1
    fi

    if [[ ! -f "$prd_file" ]]; then
        echo "aria_validate_prd: File not found: $prd_file" >&2
        return 1
    fi

    # Check JSON is valid
    if ! jq empty "$prd_file" 2>/dev/null; then
        echo "aria_validate_prd: Invalid JSON in $prd_file" >&2
        emit_signal "prd_validation_failed" "prd" "validation" \
            "file=$prd_file" \
            "error=invalid_json" 2>/dev/null || true
        return 1
    fi

    # Check required top-level fields
    local feature
    feature=$(jq -r '.feature // empty' "$prd_file")
    if [[ -z "$feature" ]]; then
        errors+=("Missing required field: feature")
    fi

    local branch_name
    branch_name=$(jq -r '.branchName // empty' "$prd_file")
    if [[ -z "$branch_name" ]]; then
        errors+=("Missing required field: branchName")
    fi

    # Check userStories exists and is array
    local stories_type
    stories_type=$(jq -r '.userStories | type' "$prd_file" 2>/dev/null || echo "null")
    if [[ "$stories_type" != "array" ]]; then
        errors+=("userStories must be an array (got: $stories_type)")
    else
        # Validate each story
        local story_count
        story_count=$(jq '.userStories | length' "$prd_file")

        if [[ "$story_count" -eq 0 ]]; then
            warnings+=("userStories array is empty")
        fi

        # Check each story has required fields
        local invalid_stories
        invalid_stories=$(jq -r '
            .userStories | to_entries[] |
            select(
                .value.id == null or .value.id == "" or
                .value.title == null or .value.title == ""
            ) | .key
        ' "$prd_file" 2>/dev/null | head -5)

        if [[ -n "$invalid_stories" ]]; then
            errors+=("Stories missing required fields (id, title): indices $invalid_stories")
        fi

        # Check for duplicate story IDs
        local duplicate_ids
        duplicate_ids=$(jq -r '.userStories | map(.id) | group_by(.) | map(select(length > 1) | .[0]) | .[]' "$prd_file" 2>/dev/null)
        if [[ -n "$duplicate_ids" ]]; then
            errors+=("Duplicate story IDs: $duplicate_ids")
        fi

        # Warn if priority not set
        local missing_priority
        missing_priority=$(jq '[.userStories[] | select(.priority == null)] | length' "$prd_file" 2>/dev/null || echo "0")
        if [[ "$missing_priority" -gt 0 ]]; then
            warnings+=("$missing_priority stories missing priority field")
        fi
    fi

    # Report results
    if [[ ${#errors[@]} -gt 0 ]]; then
        echo -e "${ARIA_RED:-}PRD validation FAILED:${ARIA_NC:-}" >&2
        for err in "${errors[@]}"; do
            echo "  ✗ $err" >&2
        done

        emit_signal "prd_validation_failed" "prd" "validation" \
            "file=$prd_file" \
            "error_count=${#errors[@]}" \
            "errors=${errors[*]}" 2>/dev/null || true
        return 1
    fi

    if [[ ${#warnings[@]} -gt 0 ]]; then
        echo -e "${ARIA_YELLOW:-}PRD validation passed with warnings:${ARIA_NC:-}" >&2
        for warn in "${warnings[@]}"; do
            echo "  ⚠ $warn" >&2
        done
    fi

    emit_signal "prd_validated" "prd" "validation" \
        "file=$prd_file" \
        "story_count=${story_count:-0}" 2>/dev/null || true

    return 0
}

# Quick PRD check (silent, just returns exit code)
# Usage: aria_prd_valid /path/to/prd.json && echo "valid"
aria_prd_valid() {
    local prd_file="${1:-}"
    [[ -f "$prd_file" ]] && jq -e '.feature and .branchName and .userStories' "$prd_file" >/dev/null 2>&1
}

# ============================================
# SKILL VALIDATION (Issue #7)
# ============================================
# Validates ARIA skill files before loading.
# Prevents silent failures from malformed skills.
#
# Required skill structure:
#   # Skill Name (H1 title)
#   > One-line description
#   ---
#   version: x.x.x
#   modes: [...]
#   ...
#   ---
#   ## When to Use

# Validate a skill file
# Usage: aria_validate_skill /path/to/skill.md
# Returns: 0 if valid, 1 if invalid (with error messages)
aria_validate_skill() {
    local skill_file="${1:-}"
    local errors=()
    local warnings=()
    local skill_name=""

    # Check file exists and is readable
    if [[ -z "$skill_file" ]]; then
        echo "aria_validate_skill: No file specified" >&2
        return 1
    fi

    if [[ ! -f "$skill_file" ]]; then
        echo "aria_validate_skill: File not found: $skill_file" >&2
        return 1
    fi

    if [[ ! -r "$skill_file" ]]; then
        echo "aria_validate_skill: Cannot read: $skill_file" >&2
        return 1
    fi

    # Check for H1 title (# Title)
    local title_line
    title_line=$(head -5 "$skill_file" | grep -m1 "^# " || true)
    if [[ -z "$title_line" ]]; then
        errors+=("Missing skill title (# Title)")
    else
        skill_name="${title_line#\# }"
    fi

    # Check for description (> One-liner)
    local desc_line
    desc_line=$(head -10 "$skill_file" | grep -m1 "^> " || true)
    if [[ -z "$desc_line" ]]; then
        warnings+=("Missing skill description (> description)")
    fi

    # Check for metadata block (between --- markers)
    local has_metadata_start
    local has_metadata_end
    has_metadata_start=$(head -20 "$skill_file" | grep -c "^---$" || echo "0")
    if [[ "$has_metadata_start" -lt 2 ]]; then
        warnings+=("Missing or incomplete metadata block (--- markers)")
    else
        # Check for required metadata fields
        local metadata_block
        metadata_block=$(sed -n '/^---$/,/^---$/p' "$skill_file" | head -20)

        if ! echo "$metadata_block" | grep -q "version:"; then
            warnings+=("Missing 'version:' in metadata")
        fi
        if ! echo "$metadata_block" | grep -q "modes:"; then
            errors+=("Missing 'modes:' in metadata (required)")
        fi
    fi

    # Check for "## When to Use" section
    if ! grep -q "^## When to Use" "$skill_file"; then
        errors+=("Missing '## When to Use' section (required)")
    fi

    # Report results
    if [[ ${#errors[@]} -gt 0 ]]; then
        echo -e "${ARIA_RED:-}Skill validation FAILED for: ${skill_name:-$skill_file}${ARIA_NC:-}" >&2
        for err in "${errors[@]}"; do
            echo "  ✗ $err" >&2
        done
        for warn in "${warnings[@]}"; do
            echo "  ⚠ $warn" >&2
        done

        emit_signal "skill_validation_failed" "skill" "validation" \
            "file=$skill_file" \
            "skill_name=${skill_name:-unknown}" \
            "error_count=${#errors[@]}" 2>/dev/null || true
        return 1
    fi

    if [[ ${#warnings[@]} -gt 0 ]]; then
        echo -e "${ARIA_YELLOW:-}Skill validation passed with warnings: ${skill_name:-$skill_file}${ARIA_NC:-}" >&2
        for warn in "${warnings[@]}"; do
            echo "  ⚠ $warn" >&2
        done
    fi

    emit_signal "skill_validated" "skill" "validation" \
        "file=$skill_file" \
        "skill_name=${skill_name:-unknown}" 2>/dev/null || true

    return 0
}

# Validate all skills in a directory
# Usage: aria_validate_all_skills /path/to/skills/
# Returns: Number of invalid skills (0 = all valid)
aria_validate_all_skills() {
    local skills_dir="${1:-.aria/skills}"
    local invalid_count=0
    local valid_count=0

    if [[ ! -d "$skills_dir" ]]; then
        echo "Skills directory not found: $skills_dir" >&2
        return 1
    fi

    echo "Validating skills in $skills_dir..."

    for skill_file in "$skills_dir"/*.md; do
        [[ -f "$skill_file" ]] || continue

        # Skip registry and composition files
        local basename
        basename=$(basename "$skill_file")
        if [[ "$basename" == "REGISTRY.md" || "$basename" == "COMPOSITION.md" ]]; then
            continue
        fi

        if aria_validate_skill "$skill_file" 2>/dev/null; then
            valid_count=$((valid_count + 1))
        else
            invalid_count=$((invalid_count + 1))
            aria_validate_skill "$skill_file"  # Show errors
        fi
    done

    echo ""
    echo "Results: $valid_count valid, $invalid_count invalid"

    return $invalid_count
}

# Quick skill check (silent, just returns exit code)
# Usage: aria_skill_valid /path/to/skill.md && echo "valid"
aria_skill_valid() {
    local skill_file="${1:-}"
    [[ -f "$skill_file" ]] && \
    grep -q "^# " "$skill_file" && \
    grep -q "^## When to Use" "$skill_file"
}

# ============================================
# SUBAGENT ISOLATION (Issue #13)
# ============================================
# Enforces task isolation in STANDARD+ modes.
# Implementation tasks should use subagents to prevent context pollution.
#
# Mode requirements:
#   LITE:     Direct execution allowed (no isolation required)
#   STANDARD: Subagent isolation recommended, violation logged
#   FULL:     Subagent isolation required, violation blocks
#   FULL+:    Subagent isolation required, violation blocks

# Current mode file
ARIA_MODE_FILE="${ARIA_MODE_FILE:-${ARIA_STATE_DIR:-$(dirname "${BASH_SOURCE[0]}")/state}/current-mode}"

# Get current ARIA mode
# Returns: LITE, STANDARD, FULL, or FULL+ (defaults to STANDARD)
aria_get_mode() {
    if [[ -f "$ARIA_MODE_FILE" ]]; then
        cat "$ARIA_MODE_FILE"
    else
        echo "STANDARD"
    fi
}

# Set current ARIA mode
aria_set_mode() {
    local mode="${1:-STANDARD}"
    local valid_modes=("LITE" "STANDARD" "FULL" "FULL+")

    # Validate mode
    local is_valid=false
    for valid in "${valid_modes[@]}"; do
        if [[ "$mode" == "$valid" ]]; then
            is_valid=true
            break
        fi
    done

    if [[ "$is_valid" != "true" ]]; then
        echo "aria_set_mode: Invalid mode '$mode'. Must be: ${valid_modes[*]}" >&2
        return 1
    fi

    mkdir -p "$(dirname "$ARIA_MODE_FILE")" 2>/dev/null || true
    echo "$mode" > "$ARIA_MODE_FILE"

    emit_signal "mode_set" "mode" "enforcement" \
        "mode=$mode" 2>/dev/null || true

    echo "Mode set to: $mode"
}

# Check if subagent isolation is required for current mode
# Returns: 0 if required, 1 if optional
aria_isolation_required() {
    local mode
    mode=$(aria_get_mode)

    case "$mode" in
        LITE)      return 1 ;;  # Not required
        STANDARD)  return 1 ;;  # Recommended but not required
        FULL)      return 0 ;;  # Required
        FULL+)     return 0 ;;  # Required
        *)         return 1 ;;
    esac
}

# Check if a feature is enabled in current mode
# Usage: aria_feature_enabled "subagents" && echo "enabled"
aria_feature_enabled() {
    local feature="${1:-}"
    local mode
    mode=$(aria_get_mode)

    case "$feature" in
        subagents|isolation)
            # Required in FULL+, optional otherwise
            [[ "$mode" == "FULL" || "$mode" == "FULL+" ]]
            ;;
        design_notes)
            # Only in FULL+
            [[ "$mode" == "FULL" || "$mode" == "FULL+" ]]
            ;;
        brainstorming|prototyping)
            # STANDARD and above
            [[ "$mode" != "LITE" ]]
            ;;
        context_refresh)
            # STANDARD and above
            [[ "$mode" != "LITE" ]]
            ;;
        tracking|progress)
            # STANDARD and above
            [[ "$mode" != "LITE" ]]
            ;;
        hitl_checkpoints)
            # All modes (destructive actions always require HITL)
            return 0
            ;;
        *)
            # Unknown features default to enabled
            return 0
            ;;
    esac
}

# Track task execution context
# Used to detect direct execution when isolation is required
ARIA_TASK_CONTEXT="${ARIA_TASK_CONTEXT:-main}"  # main or subagent

# Mark that we're in a subagent context
aria_enter_subagent() {
    export ARIA_TASK_CONTEXT="subagent"

    emit_signal "subagent_entered" "isolation" "context" \
        "task_context=subagent" 2>/dev/null || true
}

# Mark that we've exited subagent context
aria_exit_subagent() {
    export ARIA_TASK_CONTEXT="main"

    emit_signal "subagent_exited" "isolation" "context" \
        "task_context=main" 2>/dev/null || true
}

# Check and log isolation violations
# Usage: aria_check_isolation "task_id" "action_description"
# Returns: 0 if OK, 1 if violation (logs warning/error based on mode)
aria_check_isolation() {
    local task_id="${1:-unknown}"
    local action="${2:-unknown_action}"
    local mode
    mode=$(aria_get_mode)

    # If we're in a subagent, all good
    if [[ "$ARIA_TASK_CONTEXT" == "subagent" ]]; then
        return 0
    fi

    # If we're in main context, check if isolation is required
    case "$mode" in
        LITE)
            # No isolation needed
            return 0
            ;;
        STANDARD)
            # Log warning but allow
            emit_signal "isolation_violation_warning" "isolation" "enforcement" \
                "task_id=$task_id" \
                "action=$action" \
                "mode=$mode" \
                "context=$ARIA_TASK_CONTEXT" \
                "severity=warning" 2>/dev/null || true
            echo -e "${ARIA_YELLOW:-}⚠ ISOLATION WARNING: Direct execution in STANDARD mode${ARIA_NC:-}" >&2
            echo -e "${ARIA_YELLOW:-}  Consider using subagent for task: $task_id${ARIA_NC:-}" >&2
            return 0  # Allow but warn
            ;;
        FULL|FULL+)
            # Log error and block
            emit_signal "isolation_violation_blocked" "isolation" "enforcement" \
                "task_id=$task_id" \
                "action=$action" \
                "mode=$mode" \
                "context=$ARIA_TASK_CONTEXT" \
                "severity=error" 2>/dev/null || true
            echo -e "${ARIA_RED:-}✗ ISOLATION VIOLATION: Direct execution blocked in $mode mode${ARIA_NC:-}" >&2
            echo -e "${ARIA_RED:-}  Task '$task_id' must use subagent for implementation${ARIA_NC:-}" >&2
            return 1  # Block
            ;;
        *)
            return 0
            ;;
    esac
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

# ============================================
# DESIGN DOCUMENT INITIALIZATION (Issue #19)
# ============================================
# Helper to create a new DESIGN.md from template for FULL+ mode.
# Ensures proper initialization and traceability.

ARIA_DESIGN_TEMPLATE="${ARIA_DESIGN_TEMPLATE:-$(dirname "${BASH_SOURCE[0]}")/templates/DESIGN-template.md}"
ARIA_DOCS_DIR="${ARIA_DOCS_DIR:-$(dirname "${BASH_SOURCE[0]}")/docs}"

# Initialize a new design document
# Usage: aria_init_design_doc <project_name> [--force]
aria_init_design_doc() {
    local project_name="${1:-Unnamed Project}"
    local force="${2:-}"
    local design_file="$ARIA_DOCS_DIR/DESIGN.md"

    # Create docs directory if needed
    mkdir -p "$ARIA_DOCS_DIR"

    # Check if design doc already exists
    if [[ -f "$design_file" && "$force" != "--force" ]]; then
        echo -e "${ARIA_YELLOW}Design doc already exists: $design_file${ARIA_NC}" >&2
        echo -e "${ARIA_YELLOW}Use --force to overwrite${ARIA_NC}" >&2
        return 1
    fi

    # Check template exists
    if [[ ! -f "$ARIA_DESIGN_TEMPLATE" ]]; then
        echo -e "${ARIA_RED}Design template not found: $ARIA_DESIGN_TEMPLATE${ARIA_NC}" >&2
        return 1
    fi

    # Copy template and fill in basic info
    local today
    today=$(date +%Y-%m-%d)

    sed -e "s/\[Project Name\]/$project_name/" \
        -e "s/\[YYYY-MM-DD\]/$today/g" \
        "$ARIA_DESIGN_TEMPLATE" > "$design_file"

    # Emit signal for traceability
    emit_signal "design_doc_created" "design" "init" \
        "project=$project_name" \
        "file=$design_file" \
        "template=$ARIA_DESIGN_TEMPLATE"

    echo -e "${ARIA_GREEN}Created design doc: $design_file${ARIA_NC}"
    echo -e "${ARIA_BLUE}Edit the document to fill in project details${ARIA_NC}"
    echo -e "${ARIA_BLUE}Then run architecture review before implementation${ARIA_NC}"

    return 0
}

# Check if design doc exists and is approved (for FULL+ mode)
# Usage: aria_check_design_approved
# Returns: 0 if approved, 1 if not
aria_check_design_approved() {
    local design_file="$ARIA_DOCS_DIR/DESIGN.md"

    if [[ ! -f "$design_file" ]]; then
        echo -e "${ARIA_RED}No design document found${ARIA_NC}" >&2
        echo -e "${ARIA_YELLOW}FULL+ mode requires a design doc. Run:${ARIA_NC}" >&2
        echo -e "${ARIA_YELLOW}  aria_init_design_doc \"Project Name\"${ARIA_NC}" >&2
        return 1
    fi

    # Check if status is approved
    if grep -q "Status: approved" "$design_file" 2>/dev/null; then
        return 0
    fi

    # Not approved yet
    local current_status
    current_status=$(grep "^Status:" "$design_file" 2>/dev/null | head -1 | sed 's/Status: *//')

    echo -e "${ARIA_YELLOW}Design doc status: ${current_status:-unknown}${ARIA_NC}" >&2
    echo -e "${ARIA_YELLOW}FULL+ mode requires approved design doc${ARIA_NC}" >&2
    echo -e "${ARIA_YELLOW}Complete architecture review to approve${ARIA_NC}" >&2

    return 1
}

# ============================================
# PROGRESS BAR DISPLAY (Issue #14)
# ============================================
# Visual progress indicators for task tracking.
# Used by tracking skill and execution announcements.

ARIA_PROGRESS_FILE="${ARIA_PROGRESS_FILE:-$(dirname "${BASH_SOURCE[0]}")/state/progress.json}"

# Draw a progress bar
# Usage: aria_draw_progress_bar <current> <total> [width] [char_filled] [char_empty]
# Example: aria_draw_progress_bar 3 10 → "[█████████░░░░░░░░░░░░░░░░░░░░░] 30% (3/10)"
aria_draw_progress_bar() {
    local current="${1:-0}"
    local total="${2:-1}"
    local width="${3:-30}"
    local char_filled="${4:-█}"
    local char_empty="${5:-░}"

    # Avoid division by zero
    if [[ "$total" -le 0 ]]; then
        total=1
    fi

    # Calculate percentage and filled width
    local pct=$((current * 100 / total))
    local filled=$((current * width / total))
    local empty=$((width - filled))

    # Build the bar
    local bar=""
    for ((i=0; i<filled; i++)); do
        bar+="$char_filled"
    done
    for ((i=0; i<empty; i++)); do
        bar+="$char_empty"
    done

    # Output with percentage
    printf "[%s] %d%% (%d/%d)" "$bar" "$pct" "$current" "$total"
}

# Show task progress announcement (mode-aware)
# Usage: aria_show_task_progress <task_num> <total_tasks> <task_title> [status]
aria_show_task_progress() {
    local task_num="$1"
    local total_tasks="$2"
    local task_title="$3"
    local status="${4:-in_progress}"  # in_progress, completed, failed, skipped

    local mode
    mode=$(aria_get_mode 2>/dev/null || echo "STANDARD")

    local bar
    bar=$(aria_draw_progress_bar "$task_num" "$total_tasks" 20)

    case "$status" in
        "completed")
            echo -e "${ARIA_GREEN}✓${ARIA_NC} Task $task_num/$total_tasks: $task_title"
            ;;
        "failed")
            echo -e "${ARIA_RED}✗${ARIA_NC} Task $task_num/$total_tasks: $task_title (FAILED)"
            ;;
        "skipped")
            echo -e "${ARIA_YELLOW}⏭${ARIA_NC} Task $task_num/$total_tasks: $task_title (skipped)"
            ;;
        *)
            echo -e "${ARIA_BLUE}▶${ARIA_NC} Task $task_num/$total_tasks: $task_title"
            ;;
    esac

    # Show progress bar (except LITE mode which is minimal)
    if [[ "$mode" != "LITE" ]]; then
        echo -e "  ${ARIA_BLUE}Progress:${ARIA_NC} $bar"
    fi
}

# Update progress.json with current state
# Usage: aria_update_progress <task_id> <status> [notes]
aria_update_progress() {
    local task_id="$1"
    local status="$2"  # pending, in_progress, completed, failed, skipped
    local notes="${3:-}"

    local state_dir
    state_dir=$(dirname "$ARIA_PROGRESS_FILE")
    mkdir -p "$state_dir"

    # Initialize progress file if needed
    if [[ ! -f "$ARIA_PROGRESS_FILE" ]]; then
        cat > "$ARIA_PROGRESS_FILE" << 'PROGRESS_INIT'
{
  "plan_id": null,
  "status": "in_progress",
  "started": null,
  "last_updated": null,
  "completion": {
    "tasks_total": 0,
    "tasks_done": 0,
    "tasks_in_progress": 0,
    "tasks_blocked": 0,
    "tasks_skipped": 0,
    "percent_complete": 0
  },
  "current": {
    "task_id": null,
    "task_name": null
  },
  "task_history": []
}
PROGRESS_INIT
    fi

    # Update using Python for reliable JSON handling
    python3 << EOF
import json
from datetime import datetime

try:
    with open('$ARIA_PROGRESS_FILE', 'r') as f:
        data = json.load(f)
except:
    data = {"completion": {"tasks_total": 0, "tasks_done": 0, "tasks_in_progress": 0, "tasks_skipped": 0}, "task_history": []}

# Update timestamp
data['last_updated'] = datetime.now().isoformat()

# Update current task
if '$status' == 'in_progress':
    data['current'] = {'task_id': '$task_id', 'task_name': '$notes' if '$notes' else None}
    data['completion']['tasks_in_progress'] = 1

# Track status changes
if '$status' == 'completed':
    data['completion']['tasks_done'] = data['completion'].get('tasks_done', 0) + 1
    data['completion']['tasks_in_progress'] = 0
elif '$status' == 'skipped':
    data['completion']['tasks_skipped'] = data['completion'].get('tasks_skipped', 0) + 1
    data['completion']['tasks_in_progress'] = 0

# Calculate percentage
total = data['completion'].get('tasks_total', 0)
done = data['completion'].get('tasks_done', 0)
if total > 0:
    data['completion']['percent_complete'] = round(done * 100 / total)

# Add to history
data.setdefault('task_history', []).append({
    'task_id': '$task_id',
    'status': '$status',
    'timestamp': datetime.now().isoformat(),
    'notes': '$notes' if '$notes' else None
})

# Keep history manageable (last 100 entries)
data['task_history'] = data['task_history'][-100:]

with open('$ARIA_PROGRESS_FILE', 'w') as f:
    json.dump(data, f, indent=2)
EOF

    # Emit signal for traceability
    emit_signal "progress_updated" "tracking" "progress" \
        "task_id=$task_id" \
        "status=$status" 2>/dev/null || true
}

# Show full progress status (for /aria:status command)
# Usage: aria_show_progress_status
aria_show_progress_status() {
    echo ""
    echo -e "${ARIA_BLUE}═══════════════════════════════════════════════════════════${ARIA_NC}"
    echo -e "${ARIA_BLUE}                    PROGRESS STATUS                         ${ARIA_NC}"
    echo -e "${ARIA_BLUE}═══════════════════════════════════════════════════════════${ARIA_NC}"
    echo ""

    if [[ ! -f "$ARIA_PROGRESS_FILE" ]]; then
        echo -e "  ${ARIA_YELLOW}No progress tracking active${ARIA_NC}"
        echo ""
        return
    fi

    python3 << EOF
import json
from datetime import datetime

try:
    with open('$ARIA_PROGRESS_FILE', 'r') as f:
        data = json.load(f)
except Exception as e:
    print(f"  Error reading progress: {e}")
    exit(1)

c = data.get('completion', {})
total = c.get('tasks_total', 0)
done = c.get('tasks_done', 0)
in_prog = c.get('tasks_in_progress', 0)
skipped = c.get('tasks_skipped', 0)
pct = c.get('percent_complete', 0)

# Draw progress bar
width = 30
filled = int(done * width / total) if total > 0 else 0
bar = '\u2588' * filled + '\u2591' * (width - filled)

print(f"  Tasks:    {done}/{total} completed")
print(f"  Progress: [{bar}] {pct}%")
print()

if in_prog:
    current = data.get('current', {})
    task_id = current.get('task_id', 'unknown')
    task_name = current.get('task_name', '')
    print(f"  Current:  Task {task_id}" + (f" - {task_name}" if task_name else ""))
    print()

if skipped:
    print(f"  Skipped:  {skipped} tasks")
    print()

# Status
status = data.get('status', 'unknown')
started = data.get('started', '')
updated = data.get('last_updated', '')

print(f"  Status:   {status}")
if started:
    print(f"  Started:  {started[:19]}")
if updated:
    print(f"  Updated:  {updated[:19]}")
EOF

    echo ""
}
