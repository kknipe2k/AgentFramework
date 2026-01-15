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
# SAFE STATE FILE OPERATIONS (Issue #7)
# ============================================

# Lock file timeout (seconds)
ARIA_LOCK_TIMEOUT="${ARIA_LOCK_TIMEOUT:-5}"

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

# Write to file with exclusive lock (flock)
# Usage: echo "content" | aria_locked_write /path/to/file
# Returns: 0 on success, 1 on timeout/failure
aria_locked_write() {
    local target_file="$1"
    local lock_file="${target_file}.lock"

    # Create lock file directory if needed
    mkdir -p "$(dirname "$lock_file")" 2>/dev/null

    # Use flock with timeout for exclusive lock
    (
        if flock -w "$ARIA_LOCK_TIMEOUT" 200 2>/dev/null; then
            cat > "$target_file"
            exit $?
        else
            echo "aria_locked_write: timeout acquiring lock for $target_file" >&2
            exit 1
        fi
    ) 200>"$lock_file"

    return $?
}

# Safely append to JSONL file with locking
# Usage: aria_append_jsonl /path/to/file.jsonl '{"key":"value"}'
# Returns: 0 on success, 1 on failure
aria_append_jsonl() {
    local jsonl_file="$1"
    local json_line="$2"
    local lock_file="${jsonl_file}.lock"

    # Create directory if needed
    mkdir -p "$(dirname "$jsonl_file")" 2>/dev/null

    # Use flock with timeout for exclusive lock
    (
        if flock -w "$ARIA_LOCK_TIMEOUT" 200 2>/dev/null; then
            echo "$json_line" >> "$jsonl_file"
            exit $?
        else
            echo "aria_append_jsonl: timeout acquiring lock for $jsonl_file" >&2
            exit 1
        fi
    ) 200>"$lock_file"

    return $?
}

# Read JSON file with shared lock (allows concurrent reads)
# Usage: content=$(aria_read_json /path/to/file.json)
# Returns: file content on success, empty on failure
aria_read_json() {
    local json_file="$1"
    local lock_file="${json_file}.lock"

    if [[ ! -f "$json_file" ]]; then
        echo ""
        return 1
    fi

    # Create lock directory if needed
    mkdir -p "$(dirname "$lock_file")" 2>/dev/null

    # Use flock with shared lock (-s) for concurrent reads
    (
        if flock -s -w "$ARIA_LOCK_TIMEOUT" 200 2>/dev/null; then
            cat "$json_file"
            exit $?
        else
            echo "aria_read_json: timeout acquiring lock for $json_file" >&2
            exit 1
        fi
    ) 200>"$lock_file"

    return $?
}

# Safely write JSON file (atomic + locked)
# Usage: echo '{"key":"value"}' | aria_write_json /path/to/file.json
# Returns: 0 on success, 1 on failure
aria_write_json() {
    local json_file="$1"
    local lock_file="${json_file}.lock"
    local tmp_file
    local tmp_dir

    # Create directories
    mkdir -p "$(dirname "$json_file")" 2>/dev/null
    mkdir -p "$(dirname "$lock_file")" 2>/dev/null

    # Use same directory for temp to ensure atomic mv
    tmp_dir="$(dirname "$json_file")"
    tmp_file="$tmp_dir/.tmp.$(basename "$json_file").$$"

    # Use flock with timeout for exclusive lock
    (
        if flock -w "$ARIA_LOCK_TIMEOUT" 200 2>/dev/null; then
            # Write to temp file
            if cat > "$tmp_file"; then
                # Atomic move
                if mv "$tmp_file" "$json_file"; then
                    exit 0
                else
                    rm -f "$tmp_file" 2>/dev/null
                    exit 1
                fi
            else
                rm -f "$tmp_file" 2>/dev/null
                exit 1
            fi
        else
            echo "aria_write_json: timeout acquiring lock for $json_file" >&2
            exit 1
        fi
    ) 200>"$lock_file"

    return $?
}

# Update JSON file with jq expression (read-modify-write atomically)
# Usage: aria_update_json /path/to/file.json '.key = "value"'
# Returns: 0 on success, 1 on failure
aria_update_json() {
    local json_file="$1"
    local jq_expression="$2"
    local lock_file="${json_file}.lock"
    local tmp_file
    local tmp_dir

    if ! command -v jq >/dev/null 2>&1; then
        echo "aria_update_json: jq is required" >&2
        return 1
    fi

    # Create directories
    mkdir -p "$(dirname "$json_file")" 2>/dev/null
    mkdir -p "$(dirname "$lock_file")" 2>/dev/null

    tmp_dir="$(dirname "$json_file")"
    tmp_file="$tmp_dir/.tmp.$(basename "$json_file").$$"

    # Use flock for exclusive lock during read-modify-write
    (
        if flock -w "$ARIA_LOCK_TIMEOUT" 200 2>/dev/null; then
            # Read current content (or empty object if not exists)
            local current
            if [[ -f "$json_file" ]]; then
                current=$(cat "$json_file")
            else
                current='{}'
            fi

            # Apply jq transformation
            if echo "$current" | jq "$jq_expression" > "$tmp_file"; then
                # Atomic move
                if mv "$tmp_file" "$json_file"; then
                    exit 0
                else
                    rm -f "$tmp_file" 2>/dev/null
                    exit 1
                fi
            else
                rm -f "$tmp_file" 2>/dev/null
                exit 1
            fi
        else
            echo "aria_update_json: timeout acquiring lock for $json_file" >&2
            exit 1
        fi
    ) 200>"$lock_file"

    return $?
}
