#!/bin/bash
# ARIA Mode Enforcer (Issue #12)
# Enforces LITE/STANDARD/FULL/FULL+ mode behavior.
# Mode determines which features are enabled and required.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh" || { echo "Failed to load common.sh"; exit 1; }

# ============================================
# MODE DEFINITIONS (from CLAUDE.md)
# ============================================
# LITE:     Quick tasks, 1-5 tasks, low risk
# STANDARD: Medium tasks, 4-10 steps, some risk
# FULL:     Complex tasks, 10+ steps, high risk
# FULL+:    Enterprise apps, 40+ tasks, multiple systems

# Feature matrix by mode
declare -A MODE_FEATURES=(
    # Feature flags: 0=off, 1=optional, 2=required
    # Format: "MODE:FEATURE=level"
    ["LITE:formal_planning"]="0"
    ["LITE:brainstorming"]="1"
    ["LITE:verification"]="1"
    ["LITE:hitl_checkpoints"]="1"
    ["LITE:design_notes"]="0"
    ["LITE:subagents"]="0"
    ["LITE:context_refresh"]="0"
    ["LITE:progress_tracking"]="1"

    ["STANDARD:formal_planning"]="2"
    ["STANDARD:brainstorming"]="1"
    ["STANDARD:verification"]="2"
    ["STANDARD:hitl_checkpoints"]="2"
    ["STANDARD:design_notes"]="1"
    ["STANDARD:subagents"]="1"
    ["STANDARD:context_refresh"]="1"
    ["STANDARD:progress_tracking"]="2"

    ["FULL:formal_planning"]="2"
    ["FULL:brainstorming"]="2"
    ["FULL:verification"]="2"
    ["FULL:hitl_checkpoints"]="2"
    ["FULL:design_notes"]="2"
    ["FULL:subagents"]="2"
    ["FULL:context_refresh"]="2"
    ["FULL:progress_tracking"]="2"

    ["FULL+:formal_planning"]="2"
    ["FULL+:brainstorming"]="2"
    ["FULL+:verification"]="2"
    ["FULL+:hitl_checkpoints"]="2"
    ["FULL+:design_notes"]="2"
    ["FULL+:subagents"]="2"
    ["FULL+:context_refresh"]="2"
    ["FULL+:progress_tracking"]="2"
)

# ============================================
# MODE ENFORCEMENT FUNCTIONS
# ============================================

# Get feature level for current mode
# Returns: 0 (off), 1 (optional), 2 (required)
get_feature_level() {
    local feature="$1"
    local mode
    mode=$(aria_get_mode)
    local key="${mode}:${feature}"

    echo "${MODE_FEATURES[$key]:-1}"  # Default to optional
}

# Check if a feature is enabled (level > 0)
is_feature_enabled() {
    local feature="$1"
    local level
    level=$(get_feature_level "$feature")
    [[ "$level" -gt 0 ]]
}

# Check if a feature is required (level = 2)
is_feature_required() {
    local feature="$1"
    local level
    level=$(get_feature_level "$feature")
    [[ "$level" -eq 2 ]]
}

# Enforce a feature requirement
# Usage: enforce_feature "feature_name" "action_description"
# Returns: 0 if allowed, 1 if blocked
enforce_feature() {
    local feature="$1"
    local action="${2:-unknown}"
    local mode
    mode=$(aria_get_mode)
    local level
    level=$(get_feature_level "$feature")

    case "$level" in
        0)
            # Feature is OFF for this mode - log and skip
            emit_signal "feature_skipped" "mode" "enforcement" \
                "mode=$mode" \
                "feature=$feature" \
                "action=$action" \
                "reason=disabled_in_mode"
            echo -e "${ARIA_YELLOW}⏭ Skipping $feature (not available in $mode mode)${ARIA_NC}"
            return 1
            ;;
        1)
            # Feature is OPTIONAL - allow silently
            return 0
            ;;
        2)
            # Feature is REQUIRED - check it's being used
            return 0
            ;;
    esac
}

# Require a feature (fail if not enabled)
# Usage: require_feature "feature_name" "reason"
require_feature() {
    local feature="$1"
    local reason="${2:-Required for this operation}"
    local mode
    mode=$(aria_get_mode)

    if ! is_feature_enabled "$feature"; then
        emit_signal "feature_requirement_failed" "mode" "enforcement" \
            "mode=$mode" \
            "feature=$feature" \
            "reason=$reason"
        echo -e "${ARIA_RED}✗ Feature '$feature' required but not available in $mode mode${ARIA_NC}" >&2
        echo -e "${ARIA_RED}  $reason${ARIA_NC}" >&2
        return 1
    fi
    return 0
}

# ============================================
# MODE SIZING (from CLAUDE.md Router)
# ============================================

# Determine mode from project characteristics
# Usage: determine_mode --tasks N --loc N --files N --deps N --critical
determine_mode() {
    local tasks=0
    local loc=0
    local files=0
    local deps=0
    local critical=false

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --tasks)   tasks="$2"; shift 2 ;;
            --loc)     loc="$2"; shift 2 ;;
            --files)   files="$2"; shift 2 ;;
            --deps)    deps="$2"; shift 2 ;;
            --critical) critical=true; shift ;;
            *) shift ;;
        esac
    done

    local size="SMALL"

    # X-LARGE criteria
    if [[ $tasks -gt 40 || $loc -gt 50000 || $files -gt 50 || $deps -gt 15 ]]; then
        size="X-LARGE"
    # LARGE criteria
    elif [[ $tasks -gt 15 || $loc -gt 10000 || $files -gt 20 || $deps -gt 5 ]]; then
        size="LARGE"
    # MEDIUM criteria
    elif [[ $tasks -gt 5 || $loc -gt 2000 || $files -gt 5 || $deps -gt 1 ]]; then
        size="MEDIUM"
    fi

    # Critical systems bump to at least MEDIUM
    if [[ "$critical" == "true" && "$size" == "SMALL" ]]; then
        size="MEDIUM"
    fi

    # Map size to mode
    case "$size" in
        "SMALL")   echo "LITE" ;;
        "MEDIUM")  echo "STANDARD" ;;
        "LARGE")   echo "FULL" ;;
        "X-LARGE") echo "FULL+" ;;
    esac
}

# ============================================
# MODE DISPLAY
# ============================================

show_mode_status() {
    local mode
    mode=$(aria_get_mode)

    echo ""
    echo -e "${ARIA_BLUE}═══════════════════════════════════════════════════════════${ARIA_NC}"
    echo -e "${ARIA_BLUE}                    ARIA MODE STATUS                        ${ARIA_NC}"
    echo -e "${ARIA_BLUE}═══════════════════════════════════════════════════════════${ARIA_NC}"
    echo ""
    echo -e "  Current Mode: ${ARIA_GREEN}$mode${ARIA_NC}"
    echo ""

    echo "  Feature Status:"
    local features=("formal_planning" "brainstorming" "verification" "hitl_checkpoints" "design_notes" "subagents" "context_refresh" "progress_tracking")

    for feature in "${features[@]}"; do
        local level
        level=$(get_feature_level "$feature")
        local status_icon
        case "$level" in
            0) status_icon="${ARIA_RED}✗${ARIA_NC} OFF" ;;
            1) status_icon="${ARIA_YELLOW}○${ARIA_NC} optional" ;;
            2) status_icon="${ARIA_GREEN}✓${ARIA_NC} required" ;;
        esac
        printf "    %-20s %s\n" "$feature:" "$status_icon"
    done

    echo ""
}

# ============================================
# CLI
# ============================================

main() {
    local command="${1:-status}"
    shift || true

    case "$command" in
        "set")
            aria_set_mode "${1:-STANDARD}"
            ;;
        "get")
            aria_get_mode
            ;;
        "status")
            show_mode_status
            ;;
        "check")
            # Check if a feature is enabled
            local feature="${1:-}"
            if [[ -z "$feature" ]]; then
                echo "Usage: $0 check <feature>"
                exit 1
            fi
            if is_feature_enabled "$feature"; then
                echo "$feature: enabled"
                exit 0
            else
                echo "$feature: disabled"
                exit 1
            fi
            ;;
        "require")
            # Require a feature
            require_feature "${1:-unknown}" "${2:-Required}"
            ;;
        "size")
            # Determine mode from sizing
            determine_mode "$@"
            ;;
        "help"|*)
            echo "ARIA Mode Enforcer"
            echo ""
            echo "Usage: $0 <command> [args]"
            echo ""
            echo "Commands:"
            echo "  set <mode>          Set mode (LITE|STANDARD|FULL|FULL+)"
            echo "  get                 Get current mode"
            echo "  status              Show mode status and features"
            echo "  check <feature>     Check if feature is enabled"
            echo "  require <feature>   Require a feature (exits 1 if disabled)"
            echo "  size [options]      Determine mode from project size"
            echo ""
            echo "Size options:"
            echo "  --tasks N           Number of tasks"
            echo "  --loc N             Lines of code"
            echo "  --files N           Number of files"
            echo "  --deps N            New dependencies"
            echo "  --critical          System is critical (auth/payments/etc)"
            echo ""
            echo "Features:"
            echo "  formal_planning, brainstorming, verification, hitl_checkpoints,"
            echo "  design_notes, subagents, context_refresh, progress_tracking"
            echo ""
            echo "Examples:"
            echo "  $0 set FULL"
            echo "  $0 check subagents && echo 'use subagents'"
            echo "  $0 size --tasks 15 --files 10 --critical"
            ;;
    esac
}

main "$@"
