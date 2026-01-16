#!/bin/bash
# ARIA Human-in-the-Loop (HITL) System
# Pauses execution and waits for human intervention
# Terminal-based notification and response

# Exit on error, undefined vars, and pipeline failures
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh" || { echo "Failed to load common.sh"; exit 1; }

# Check dependencies (minimal - just git for context)
aria_check_deps git || exit 1

ARIA_DIR="$SCRIPT_DIR"
STATE_DIR="$ARIA_DIR/state"
HITL_DIR="$ARIA_DIR/hitl"
LOGS_DIR="$ARIA_DIR/logs"

# Colors from common.sh
RED="$ARIA_RED"
GREEN="$ARIA_GREEN"
YELLOW="$ARIA_YELLOW"
BLUE="$ARIA_BLUE"
MAGENTA="$ARIA_MAGENTA"
NC="$ARIA_NC"

mkdir -p "$HITL_DIR" "$LOGS_DIR"

# ============================================
# TRACEABILITY - Uses emit_signal() from common.sh
# ============================================
# HITL events are logged via the centralized emit_signal() function
# (single-writer pattern for signals.jsonl)

# Wrapper for HITL signal emission (delegates to emit_signal)
_log_hitl_signal() {
    local event_type="$1"      # request_created, response_received, timeout
    local request_id="$2"
    local request_type="$3"    # help, confirm, choice, input
    local details="${4:-}"
    local response="${5:-}"

    # Build optional key=value pairs
    local -a extra_args=("request_id=${request_id}" "request_type=${request_type}")

    if [[ -n "$details" ]]; then
        extra_args+=("details=${details}")
    fi

    if [[ -n "$response" ]]; then
        extra_args+=("response=${response}")
    fi

    # Delegate to centralized emit_signal (single owner of signals.jsonl)
    emit_signal "hitl_${event_type}" "hitl" "human_intervention" "${extra_args[@]}"
}

# ============================================
# CONFIGURATION
# ============================================

# Notification method - simplified to terminal only
HITL_NOTIFY="${HITL_NOTIFY:-terminal}"

# Timeout waiting for human (0 = forever)
HITL_TIMEOUT="${HITL_TIMEOUT:-0}"

# Poll interval for response file
HITL_POLL_INTERVAL="${HITL_POLL_INTERVAL:-2}"

# Slack webhook (optional)
HITL_SLACK_WEBHOOK="${HITL_SLACK_WEBHOOK:-}"

# Email (optional)
HITL_EMAIL="${HITL_EMAIL:-}"

# ============================================
# NOTIFICATION METHODS
# ============================================

notify_terminal() {
    local title="$1"
    local message="$2"

    # Terminal bell
    echo -e "\a"

    # Bright output
    echo ""
    echo -e "${MAGENTA}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${MAGENTA}║           🚨 HUMAN INTERVENTION REQUIRED 🚨               ║${NC}"
    echo -e "${MAGENTA}╠═══════════════════════════════════════════════════════════╣${NC}"
    echo -e "${MAGENTA}║${NC} ${YELLOW}$title${NC}"
    echo -e "${MAGENTA}╠═══════════════════════════════════════════════════════════╣${NC}"
    echo -e "${MAGENTA}║${NC} $message"
    echo -e "${MAGENTA}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

notify_desktop() {
    local title="$1"
    local message="$2"

    # Try multiple notification methods
    if command -v notify-send >/dev/null 2>&1; then
        notify-send -u critical "ARIA: $title" "$message" 2>/dev/null || true
    elif command -v osascript >/dev/null 2>&1; then
        # macOS
        osascript -e "display notification \"$message\" with title \"ARIA: $title\" sound name \"Blow\"" 2>/dev/null || true
    elif command -v powershell.exe >/dev/null 2>&1; then
        # WSL/Windows
        powershell.exe -Command "[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms'); [System.Windows.Forms.MessageBox]::Show('$message','ARIA: $title')" 2>/dev/null || true
    fi
}

notify_sound() {
    # Play sound alert
    if command -v paplay >/dev/null 2>&1; then
        paplay /usr/share/sounds/freedesktop/stereo/dialog-warning.oga 2>/dev/null &
    elif command -v afplay >/dev/null 2>&1; then
        # macOS
        afplay /System/Library/Sounds/Ping.aiff 2>/dev/null &
    elif command -v aplay >/dev/null 2>&1; then
        # Try system beep
        echo -e "\a"
    fi
}

notify_slack() {
    local title="$1"
    local message="$2"
    local request_id="$3"

    if [[ -z "$HITL_SLACK_WEBHOOK" ]]; then
        return
    fi

    local payload=$(cat <<EOF
{
    "text": "🚨 *ARIA Human Intervention Required*",
    "blocks": [
        {
            "type": "header",
            "text": {"type": "plain_text", "text": "🚨 Human Intervention Required"}
        },
        {
            "type": "section",
            "text": {"type": "mrkdwn", "text": "*$title*\n$message"}
        },
        {
            "type": "section",
            "text": {"type": "mrkdwn", "text": "Request ID: \`$request_id\`\nRespond with: \`aria hitl respond $request_id \"your response\"\`"}
        }
    ]
}
EOF
)

    curl -s -X POST -H 'Content-type: application/json' --data "$payload" "$HITL_SLACK_WEBHOOK" >/dev/null 2>&1 || true
}

notify_email() {
    local title="$1"
    local message="$2"
    local request_id="$3"

    if [[ -z "$HITL_EMAIL" ]]; then
        return
    fi

    if command -v mail >/dev/null 2>&1; then
        echo -e "ARIA Human Intervention Required\n\n$title\n\n$message\n\nRequest ID: $request_id\n\nRespond by running:\naria hitl respond $request_id \"your response\"" | \
            mail -s "🚨 ARIA: $title" "$HITL_EMAIL" 2>/dev/null || true
    fi
}

notify_file() {
    local title="$1"
    local message="$2"
    local request_id="$3"

    # Create a visible file for human to notice
    local notice_file="$HITL_DIR/HUMAN_NEEDED_${request_id}.txt"

    cat > "$notice_file" << EOF
═══════════════════════════════════════════════════════════
           🚨 HUMAN INTERVENTION REQUIRED 🚨
═══════════════════════════════════════════════════════════

Title: $title

$message

═══════════════════════════════════════════════════════════
                    HOW TO RESPOND
═══════════════════════════════════════════════════════════

Option 1: Create response file
  echo "your response here" > $HITL_DIR/response_${request_id}.txt

Option 2: Use CLI
  ./.aria/aria-engine.sh hitl respond $request_id "your response"

Option 3: Approve/Reject shortcuts
  ./.aria/hitl.sh approve $request_id
  ./.aria/hitl.sh reject $request_id "reason"

═══════════════════════════════════════════════════════════
Request ID: $request_id
Created: $(date)
EOF

    echo "$notice_file"
}

# Send all configured notifications
send_notifications() {
    local title="$1"
    local message="$2"
    local request_id="$3"

    for method in $HITL_NOTIFY; do
        case "$method" in
            terminal) notify_terminal "$title" "$message" ;;
            desktop)  notify_desktop "$title" "$message" ;;
            sound)    notify_sound ;;
            slack)    notify_slack "$title" "$message" "$request_id" ;;
            email)    notify_email "$title" "$message" "$request_id" ;;
            file)     notify_file "$title" "$message" "$request_id" ;;
        esac
    done

    # Always create file notification
    notify_file "$title" "$message" "$request_id"
}

# ============================================
# REQUEST MANAGEMENT
# ============================================

create_request() {
    local reason="$1"
    local context="$2"
    local request_type="${3:-general}"

    local request_id=$(date +%Y%m%d_%H%M%S)_$$
    local request_file="$HITL_DIR/request_${request_id}.json"

    cat > "$request_file" << EOF
{
    "id": "$request_id",
    "type": "$request_type",
    "reason": "$reason",
    "context": "$context",
    "status": "pending",
    "created_at": "$(date -Iseconds)",
    "responded_at": null,
    "response": null,
    "responder": null
}
EOF

    # Log to signals.jsonl for traceability
    _log_hitl_signal "request_created" "$request_id" "$request_type" "$reason"

    echo "$request_id"
}

get_request_status() {
    local request_id="$1"
    local request_file="$HITL_DIR/request_${request_id}.json"

    if [[ -f "$request_file" ]]; then
        grep -o '"status": *"[^"]*"' "$request_file" | cut -d'"' -f4
    else
        echo "not_found"
    fi
}

get_response() {
    local request_id="$1"

    # Check for response file first
    local response_file="$HITL_DIR/response_${request_id}.txt"
    if [[ -f "$response_file" ]]; then
        cat "$response_file"
        return 0
    fi

    # Check request JSON
    local request_file="$HITL_DIR/request_${request_id}.json"
    if [[ -f "$request_file" ]]; then
        local response=$(grep -o '"response": *"[^"]*"' "$request_file" | cut -d'"' -f4)
        if [[ -n "$response" ]] && [[ "$response" != "null" ]]; then
            echo "$response"
            return 0
        fi
    fi

    return 1
}

set_response() {
    local request_id="$1"
    local response="$2"
    local responder="${3:-human}"

    local request_file="$HITL_DIR/request_${request_id}.json"
    local response_file="$HITL_DIR/response_${request_id}.txt"

    # Get request type for logging
    local request_type="unknown"
    if [[ -f "$request_file" ]]; then
        request_type=$(grep -o '"type": *"[^"]*"' "$request_file" 2>/dev/null | cut -d'"' -f4 || echo "unknown")
    fi

    # Write response file
    echo "$response" > "$response_file"

    # Update request JSON if it exists
    if [[ -f "$request_file" ]]; then
        # Simple sed replacement (not perfect JSON but works)
        sed -i "s/\"status\": *\"pending\"/\"status\": \"responded\"/" "$request_file" 2>/dev/null || true
        sed -i "s/\"response\": *null/\"response\": \"$response\"/" "$request_file" 2>/dev/null || true
        sed -i "s/\"responded_at\": *null/\"responded_at\": \"$(date -Iseconds)\"/" "$request_file" 2>/dev/null || true
    fi

    # Clean up notice file
    rm -f "$HITL_DIR/HUMAN_NEEDED_${request_id}.txt"

    # Log to signals.jsonl for traceability
    _log_hitl_signal "response_received" "$request_id" "$request_type" "responder:$responder" "$response"

    # Log the intervention (legacy logging)
    log_intervention "$request_id" "$response" "$responder"
}

# ============================================
# WAITING MECHANISM
# ============================================

wait_for_response() {
    local request_id="$1"
    local timeout="${2:-$HITL_TIMEOUT}"

    local elapsed=0
    local response_file="$HITL_DIR/response_${request_id}.txt"

    echo -e "${YELLOW}Waiting for human response...${NC}"
    echo "Request ID: $request_id"
    echo ""
    echo "To respond:"
    echo "  echo \"your response\" > $response_file"
    echo "  OR"
    echo "  ./.aria/hitl.sh respond $request_id \"your response\""
    echo ""

    while true; do
        # Check for response
        if [[ -f "$response_file" ]]; then
            local response=$(cat "$response_file")
            echo ""
            echo -e "${GREEN}Response received: $response${NC}"
            return 0
        fi

        # Check timeout
        if [[ $timeout -gt 0 ]] && [[ $elapsed -ge $timeout ]]; then
            echo ""
            echo -e "${RED}Timeout waiting for human response${NC}"
            # Log timeout to signals.jsonl
            _log_hitl_signal "timeout" "$request_id" "unknown" "timeout_seconds:$timeout"
            return 1
        fi

        # Show waiting indicator
        printf "."
        sleep $HITL_POLL_INTERVAL
        elapsed=$((elapsed + HITL_POLL_INTERVAL))
    done
}

# ============================================
# LOGGING
# ============================================

log_intervention() {
    local request_id="$1"
    local response="$2"
    local responder="$3"

    local log_file="$LOGS_DIR/hitl.log"

    echo "[$(date -Iseconds)] REQUEST=$request_id RESPONDER=$responder RESPONSE=\"$response\"" >> "$log_file"

    # Also add to progress.txt if Ralph mode
    local progress_file="$ARIA_DIR/ralph/progress.txt"
    if [[ -f "$progress_file" ]]; then
        cat >> "$progress_file" << EOF

## $(date '+%Y-%m-%d %H:%M') - Human Intervention
- Request ID: $request_id
- Response: $response
- Responder: $responder
EOF
    fi
}

# ============================================
# MAIN ENTRY POINTS
# ============================================

# Request human help and wait
request_help() {
    local reason="$1"
    local context="${2:-}"

    local request_id=$(create_request "$reason" "$context" "help")

    send_notifications "Help Needed" "$reason" "$request_id"

    if wait_for_response "$request_id"; then
        get_response "$request_id"
        return 0
    else
        return 1
    fi
}

# Request confirmation (yes/no)
request_confirm() {
    local question="$1"
    local context="${2:-}"

    local request_id=$(create_request "$question" "$context" "confirm")

    send_notifications "Confirmation Required" "$question\n\nRespond with: yes / no" "$request_id"

    if wait_for_response "$request_id"; then
        local response=$(get_response "$request_id")
        if [[ "$response" =~ ^[Yy](es)?$ ]]; then
            return 0
        else
            return 1
        fi
    else
        return 1
    fi
}

# Request choice from options
request_choice() {
    local question="$1"
    shift
    local options=("$@")

    local options_text=""
    local i=1
    for opt in "${options[@]}"; do
        options_text="$options_text\n  $i) $opt"
        i=$((i + 1))
    done

    local request_id=$(create_request "$question" "Options:$options_text" "choice")

    send_notifications "Choice Required" "$question$options_text\n\nRespond with option number or text" "$request_id"

    if wait_for_response "$request_id"; then
        get_response "$request_id"
        return 0
    else
        return 1
    fi
}

# Request text input
request_input() {
    local prompt="$1"
    local context="${2:-}"

    local request_id=$(create_request "$prompt" "$context" "input")

    send_notifications "Input Required" "$prompt" "$request_id"

    if wait_for_response "$request_id"; then
        get_response "$request_id"
        return 0
    else
        return 1
    fi
}

# Quick approve shortcut
approve() {
    local request_id="$1"
    set_response "$request_id" "yes" "human"
    echo -e "${GREEN}Approved request $request_id${NC}"
}

# Quick reject shortcut
reject() {
    local request_id="$1"
    local reason="${2:-rejected}"
    set_response "$request_id" "no: $reason" "human"
    echo -e "${RED}Rejected request $request_id: $reason${NC}"
}

# Respond to a pending request
respond() {
    local request_id="$1"
    local response="$2"

    if [[ -z "$request_id" ]] || [[ -z "$response" ]]; then
        echo "Usage: hitl respond <request_id> <response>"
        return 1
    fi

    set_response "$request_id" "$response" "human"
    echo -e "${GREEN}Response recorded for $request_id${NC}"
}

# List pending requests
list_pending() {
    echo ""
    echo -e "${BLUE}Pending HITL Requests:${NC}"
    echo ""

    local found=0
    for notice in "$HITL_DIR"/HUMAN_NEEDED_*.txt; do
        if [[ -f "$notice" ]]; then
            found=1
            local request_id=$(basename "$notice" .txt | sed 's/HUMAN_NEEDED_//')
            local title=$(grep "^Title:" "$notice" | cut -d: -f2-)
            echo "  [$request_id] $title"
        fi
    done

    if [[ $found -eq 0 ]]; then
        echo "  No pending requests"
    fi
    echo ""
}

# Show status
status() {
    local pending=$(ls "$HITL_DIR"/HUMAN_NEEDED_*.txt 2>/dev/null | wc -l)
    local total=$(ls "$HITL_DIR"/request_*.json 2>/dev/null | wc -l)

    echo ""
    echo -e "${BLUE}HITL Status:${NC}"
    echo "  Pending requests: $pending"
    echo "  Total requests:   $total"
    echo "  Notification methods: $HITL_NOTIFY"
    echo ""

    if [[ $pending -gt 0 ]]; then
        list_pending
    fi
}

# ============================================
# CLI
# ============================================

main() {
    local command="${1:-help}"
    shift || true

    case "$command" in
        "help"|"request")
            request_help "$@"
            ;;
        "confirm")
            request_confirm "$@"
            ;;
        "choice")
            request_choice "$@"
            ;;
        "input")
            request_input "$@"
            ;;
        "respond")
            respond "$@"
            ;;
        "approve")
            approve "$@"
            ;;
        "reject")
            reject "$@"
            ;;
        "list"|"pending")
            list_pending
            ;;
        "status")
            status
            ;;
        "test")
            # Test notification
            echo "Testing notifications..."
            send_notifications "Test Alert" "This is a test of the HITL system" "test_$(date +%s)"
            echo "Check for notifications"
            ;;
        *)
            echo "ARIA Human-in-the-Loop System"
            echo ""
            echo "Usage: $0 <command> [args]"
            echo ""
            echo "Commands:"
            echo "  help <reason>              - Request help and wait for response"
            echo "  confirm <question>         - Request yes/no confirmation"
            echo "  choice <question> <opts>   - Request choice from options"
            echo "  input <prompt>             - Request text input"
            echo ""
            echo "  respond <id> <response>    - Respond to a pending request"
            echo "  approve <id>               - Quick approve (yes)"
            echo "  reject <id> [reason]       - Quick reject (no)"
            echo ""
            echo "  list                       - List pending requests"
            echo "  status                     - Show HITL status"
            echo "  test                       - Test notifications"
            echo ""
            echo "Environment:"
            echo "  HITL_NOTIFY        - Notification methods (terminal desktop sound slack email file)"
            echo "  HITL_TIMEOUT       - Timeout in seconds (0 = forever)"
            echo "  HITL_SLACK_WEBHOOK - Slack webhook URL"
            echo "  HITL_EMAIL         - Email address for notifications"
            ;;
    esac
}

main "$@"
