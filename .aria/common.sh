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
