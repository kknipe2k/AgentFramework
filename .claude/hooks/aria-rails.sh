#!/bin/bash
# ARIA RAILS v4 - Full Traceability Signal Logging
# Captures rich context for every tool call enabling forensic debugging
# When something fails, you can see exactly what, when, where, why

# Get data from environment variables (Claude Code CLI sets these)
HOOK_EVENT="${1:-PreToolUse}"
TOOL_NAME="${TOOL_NAME:-$2}"
TOOL_INPUT="${TOOL_INPUT:-$3}"

# Paths
ARIA_DIR=".aria"
STATE_DIR="$ARIA_DIR/state"
SIGNALS_FILE="$STATE_DIR/signals.jsonl"
PENDING_DIR="$STATE_DIR/pending"
INTENT_FILE="$ARIA_DIR/intent.md"
RALPH_DIR="$ARIA_DIR/ralph"
PRD_FILE="$RALPH_DIR/prd.json"

# Create directories
mkdir -p "$STATE_DIR" "$PENDING_DIR"

# ============================================
# UTILITY FUNCTIONS
# ============================================

# Get timestamp with milliseconds
get_timestamp() {
    date -u +%Y-%m-%dT%H:%M:%S.%3NZ 2>/dev/null || date -u +%Y-%m-%dT%H:%M:%SZ
}

# Generate unique signal ID
gen_signal_id() {
    local prefix="${1:-sig}"
    echo "${prefix}-$(date +%s%N 2>/dev/null | cut -c1-13 || date +%s)"
}

# Extract JSON field (Windows-compatible, no grep -P)
extract_json_field() {
    local json="$1"
    local field="$2"
    echo "$json" | sed -n 's/.*"'"$field"'"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1
}

# Extract JSON number field
extract_json_number() {
    local json="$1"
    local field="$2"
    echo "$json" | sed -n 's/.*"'"$field"'"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p' | head -1
}

# Escape string for JSON
json_escape() {
    local str="$1"
    str="${str//\\/\\\\}"
    str="${str//\"/\\\"}"
    str="${str//$'\t'/\\t}"
    str="${str//$'\n'/\\n}"
    str="${str//$'\r'/}"
    echo "$str"
}

# Truncate string with ellipsis
truncate() {
    local str="$1"
    local max="${2:-200}"
    if [[ ${#str} -gt $max ]]; then
        echo "${str:0:$max}..."
    else
        echo "$str"
    fi
}

# Detect file category from path
get_file_category() {
    local path="$1"
    case "$path" in
        *.test.*|*.spec.*|*_test.*|*_spec.*|*/test/*|*/tests/*|*/__tests__/*)
            echo "test" ;;
        *.config.*|*.json|*.yaml|*.yml|*.toml|*.env*|tsconfig.*|package.json|Cargo.toml)
            echo "config" ;;
        *.md|*.txt|*.rst|README*|CHANGELOG*|LICENSE*)
            echo "doc" ;;
        *.aria/skills/*)
            echo "skill" ;;
        *.aria/templates/*)
            echo "template" ;;
        *.aria/*)
            echo "framework" ;;
        *)
            echo "source" ;;
    esac
}

# Get file info if exists
get_file_info() {
    local path="$1"
    if [[ -f "$path" ]]; then
        local lines=$(wc -l < "$path" 2>/dev/null || echo "0")
        local bytes=$(wc -c < "$path" 2>/dev/null || echo "0")
        echo "{\"exists\":true,\"lines\":$lines,\"bytes\":$bytes}"
    else
        echo "{\"exists\":false}"
    fi
}

# Detect Ralph mode
is_ralph_mode() {
    [[ -f "$PRD_FILE" ]] && [[ "${ARIA_RALPH_MODE:-0}" == "1" ]]
}

# ============================================
# CONTEXT DETECTION
# ============================================

detect_context() {
    local tool="$1"
    local input="$2"
    local file_path=""
    local command=""
    local pattern=""
    local context_type=""
    local context_name=""
    local context_detail=""

    case "$tool" in
        "Read"|"Edit"|"Write"|"MultiEdit")
            file_path=$(extract_json_field "$input" "file_path")

            if [[ "$file_path" == *".aria/skills/"* ]]; then
                context_type="skill"
                context_name=$(basename "$file_path" .md)
                context_detail="Loading skill instructions"
            elif [[ "$file_path" == *".aria/templates/"* ]]; then
                context_type="template"
                context_name=$(basename "$file_path" .md)
                context_detail="Loading template"
            elif [[ "$file_path" == *"CLAUDE.md" ]]; then
                context_type="framework"
                context_name="CLAUDE.md"
                context_detail="Reading framework instructions"
            elif [[ "$file_path" == *"project-context.md" ]]; then
                context_type="framework"
                context_name="project-context"
                context_detail="Reading project context"
            elif [[ "$file_path" == *"progress.json" ]]; then
                context_type="tracking"
                context_name="progress"
                context_detail="Updating task progress"
            elif [[ "$file_path" == *"current-plan.json" ]]; then
                context_type="planning"
                context_name="plan"
                context_detail="Accessing plan"
            elif [[ "$file_path" == *"decisions.jsonl" ]]; then
                context_type="tracing"
                context_name="decisions"
                context_detail="Recording decision"
            else
                context_type="code"
                context_name=$(get_file_category "$file_path")
            fi
            ;;

        "Bash")
            command=$(extract_json_field "$input" "command")

            if echo "$command" | grep -qE "^(npm test|yarn test|pnpm test|pytest|jest|cargo test|go test|make test|bun test)"; then
                context_type="verify"
                context_name="test"
                context_detail="Running tests"
            elif echo "$command" | grep -qE "^(npm run lint|eslint|pylint|flake8|cargo clippy)"; then
                context_type="verify"
                context_name="lint"
                context_detail="Running linter"
            elif echo "$command" | grep -qE "^(tsc|npx tsc|cargo check)"; then
                context_type="verify"
                context_name="typecheck"
                context_detail="Type checking"
            elif echo "$command" | grep -q "^git commit"; then
                context_type="git"
                context_name="commit"
                context_detail="Creating commit"
            elif echo "$command" | grep -q "^git push"; then
                context_type="git"
                context_name="push"
                context_detail="Pushing to remote"
            elif echo "$command" | grep -q "^git "; then
                context_type="git"
                context_name=$(echo "$command" | awk '{print $2}')
                context_detail="Git operation"
            elif echo "$command" | grep -qE "^(npm install|yarn add|pip install|cargo add)"; then
                context_type="deps"
                context_name="install"
                context_detail="Installing dependencies"
            elif echo "$command" | grep -qE "^(npm run|yarn|pnpm|make|cargo run)"; then
                context_type="build"
                context_name="run"
                context_detail="Running build/script"
            else
                context_type="shell"
                context_name="command"
            fi
            ;;

        "Glob")
            pattern=$(extract_json_field "$input" "pattern")
            context_type="search"
            context_name="glob"
            context_detail="Finding files: $pattern"
            ;;

        "Grep")
            pattern=$(extract_json_field "$input" "pattern")
            context_type="search"
            context_name="grep"
            context_detail="Searching: $(truncate "$pattern" 50)"
            ;;

        "Task")
            local subagent=$(extract_json_field "$input" "subagent_type")
            local desc=$(extract_json_field "$input" "description")
            context_type="agent"
            context_name="$subagent"
            context_detail="$desc"
            ;;

        "WebFetch"|"WebSearch")
            context_type="web"
            context_name=$(echo "$tool" | tr '[:upper:]' '[:lower:]')
            ;;

        "TodoWrite")
            context_type="tracking"
            context_name="todos"
            context_detail="Updating task list"
            ;;

        *)
            context_type="tool"
            context_name="$tool"
            ;;
    esac

    echo "$context_type|$context_name|$context_detail"
}

# ============================================
# RICH SIGNAL LOGGING
# ============================================

log_pre_signal() {
    local tool="$1"
    local input="$2"
    local timestamp=$(get_timestamp)
    local signal_id=$(gen_signal_id "sig")

    # Parse context
    local context_raw=$(detect_context "$tool" "$input")
    local context_type=$(echo "$context_raw" | cut -d'|' -f1)
    local context_name=$(echo "$context_raw" | cut -d'|' -f2)
    local context_detail=$(echo "$context_raw" | cut -d'|' -f3)

    # Extract tool-specific data
    local file_path=$(extract_json_field "$input" "file_path")
    local command=$(extract_json_field "$input" "command")
    local pattern=$(extract_json_field "$input" "pattern")
    local subagent=$(extract_json_field "$input" "subagent_type")
    local description=$(extract_json_field "$input" "description")
    local prompt=$(extract_json_field "$input" "prompt")

    # Get file info for Read operations
    local file_info=""
    if [[ "$tool" == "Read" ]] && [[ -n "$file_path" ]]; then
        file_info=$(get_file_info "$file_path")
    fi

    # Escape for JSON
    file_path=$(json_escape "$file_path")
    command=$(json_escape "$(truncate "$command" 500)")
    pattern=$(json_escape "$pattern")
    description=$(json_escape "$description")
    prompt=$(json_escape "$(truncate "$prompt" 300)")
    context_detail=$(json_escape "$context_detail")

    # Build the signal JSON
    local signal=$(cat <<EOF
{
  "id": "$signal_id",
  "type": "tool",
  "event": "pre",
  "timestamp": "$timestamp",
  "tool": {
    "name": "$tool",
    "file_path": "$file_path",
    "command": "$command",
    "pattern": "$pattern",
    "subagent_type": "$subagent",
    "description": "$description",
    "prompt_preview": "$prompt"
  },
  "context": {
    "type": "$context_type",
    "name": "$context_name",
    "detail": "$context_detail"
  },
  "file_info": $file_info
}
EOF
)

    # Compact to single line and write
    echo "$signal" | tr -d '\n' | sed 's/  */ /g' >> "$SIGNALS_FILE"
    echo "" >> "$SIGNALS_FILE"

    # Store pre-signal for duration calculation
    echo "$timestamp" > "$PENDING_DIR/$signal_id"
    echo "$signal_id"
}

log_post_signal() {
    local tool="$1"
    local input="$2"
    local pre_signal_id="$3"
    local timestamp=$(get_timestamp)
    local signal_id=$(gen_signal_id "sig")

    # Calculate duration if we have pre-signal
    local duration_ms=""
    local pre_timestamp=""
    if [[ -f "$PENDING_DIR/$pre_signal_id" ]]; then
        pre_timestamp=$(cat "$PENDING_DIR/$pre_signal_id")
        rm -f "$PENDING_DIR/$pre_signal_id"
        # Simple duration calc (seconds only, ms requires more complex parsing)
        local pre_epoch=$(date -d "$pre_timestamp" +%s 2>/dev/null || echo "")
        local post_epoch=$(date +%s)
        if [[ -n "$pre_epoch" ]]; then
            duration_ms=$(( (post_epoch - pre_epoch) * 1000 ))
        fi
    fi

    # Parse context
    local context_raw=$(detect_context "$tool" "$input")
    local context_type=$(echo "$context_raw" | cut -d'|' -f1)
    local context_name=$(echo "$context_raw" | cut -d'|' -f2)

    # Extract result data based on tool
    local file_path=$(extract_json_field "$input" "file_path")
    local exit_code=$(extract_json_number "$input" "exit_code")
    local success="true"
    local error=""
    local result_detail=""

    # Determine success/failure
    if [[ -n "$exit_code" ]] && [[ "$exit_code" != "0" ]]; then
        success="false"
        error="Exit code: $exit_code"
    fi

    # Get result details based on tool type
    case "$tool" in
        "Read")
            if [[ -f "$file_path" ]]; then
                local lines=$(wc -l < "$file_path" 2>/dev/null || echo "0")
                result_detail="Read $lines lines"
            fi
            ;;
        "Edit"|"Write")
            result_detail="File modified"
            ;;
        "Glob")
            result_detail="File search completed"
            ;;
        "Grep")
            result_detail="Content search completed"
            ;;
        "Task")
            result_detail="Agent task completed"
            ;;
    esac

    # Escape for JSON
    file_path=$(json_escape "$file_path")
    error=$(json_escape "$error")
    result_detail=$(json_escape "$result_detail")

    # Build the signal JSON
    local signal=$(cat <<EOF
{
  "id": "$signal_id",
  "type": "tool",
  "event": "post",
  "timestamp": "$timestamp",
  "duration_ms": ${duration_ms:-null},
  "tool": {
    "name": "$tool",
    "file_path": "$file_path"
  },
  "result": {
    "success": $success,
    "exit_code": ${exit_code:-null},
    "error": ${error:+\"$error\"}${error:-null},
    "detail": ${result_detail:+\"$result_detail\"}${result_detail:-null}
  },
  "context": {
    "type": "$context_type",
    "name": "$context_name"
  },
  "correlation": {
    "pre_signal_id": "$pre_signal_id"
  }
}
EOF
)

    # Compact to single line and write
    echo "$signal" | tr -d '\n' | sed 's/  */ /g' >> "$SIGNALS_FILE"
    echo "" >> "$SIGNALS_FILE"
}

# Log skill load event
log_skill_load() {
    local skill_name="$1"
    local file_path="$2"
    local timestamp=$(get_timestamp)
    local signal_id=$(gen_signal_id "skill")

    local signal=$(cat <<EOF
{
  "id": "$signal_id",
  "type": "skill",
  "event": "loaded",
  "timestamp": "$timestamp",
  "skill": {
    "name": "$skill_name",
    "path": "$file_path"
  }
}
EOF
)

    echo "$signal" | tr -d '\n' | sed 's/  */ /g' >> "$SIGNALS_FILE"
    echo "" >> "$SIGNALS_FILE"
}

# Log agent spawn
log_agent_spawn() {
    local agent_type="$1"
    local description="$2"
    local prompt_preview="$3"
    local timestamp=$(get_timestamp)
    local signal_id=$(gen_signal_id "agent")

    description=$(json_escape "$description")
    prompt_preview=$(json_escape "$(truncate "$prompt_preview" 300)")

    local signal=$(cat <<EOF
{
  "id": "$signal_id",
  "type": "agent",
  "event": "spawned",
  "timestamp": "$timestamp",
  "agent": {
    "type": "$agent_type",
    "description": "$description",
    "prompt_preview": "$prompt_preview"
  }
}
EOF
)

    echo "$signal" | tr -d '\n' | sed 's/  */ /g' >> "$SIGNALS_FILE"
    echo "" >> "$SIGNALS_FILE"

    # Store for completion tracking
    echo "$timestamp|$agent_type" > "$PENDING_DIR/agent-$signal_id"
    echo "$signal_id"
}

# Log error event
log_error() {
    local category="$1"
    local message="$2"
    local tool="$3"
    local related_signal="$4"
    local timestamp=$(get_timestamp)
    local signal_id=$(gen_signal_id "error")

    message=$(json_escape "$message")

    local signal=$(cat <<EOF
{
  "id": "$signal_id",
  "type": "error",
  "timestamp": "$timestamp",
  "error": {
    "category": "$category",
    "message": "$message",
    "tool": "$tool"
  },
  "context": {
    "related_signal_id": "$related_signal"
  }
}
EOF
)

    echo "$signal" | tr -d '\n' | sed 's/  */ /g' >> "$SIGNALS_FILE"
    echo "" >> "$SIGNALS_FILE"
}

# ============================================
# SIGNAL-BASED TEST TRIGGERING
# ============================================

# Trigger relevant tests based on file modified
trigger_tests_for_file() {
    local file_path="$1"
    local test_dir="$ARIA_DIR/tests"

    # Silently skip if no test directory
    [[ ! -d "$test_dir" ]] && return 0

    # Map file patterns to tests (run in background, don't block)
    case "$file_path" in
        *serve-dashboard.py*|*dashboard*)
            bash "$test_dir/unit/test-dashboard.sh" > /dev/null 2>&1 || true
            ;;
        *generate-slides.py*|*slide-generation*)
            bash "$test_dir/unit/test-slide-generation.sh" > /dev/null 2>&1 || true
            ;;
        *deep-research*|*researcher*)
            bash "$test_dir/unit/test-deep-research.sh" > /dev/null 2>&1 || true
            ;;
        *detect-stack*|*generate-tests*|*test-generation*)
            bash "$test_dir/unit/test-test-generation.sh" > /dev/null 2>&1 || true
            ;;
        *cc-usage*|*token_usage*|*metrics*)
            bash "$test_dir/unit/test-cc-usage.sh" > /dev/null 2>&1 || true
            ;;
    esac
}

# Verify slide generation signals were emitted
verify_slide_signals() {
    local verify_script="$ARIA_DIR/scripts/verify-slide-signals.py"

    # Skip if verification script doesn't exist
    [[ ! -f "$verify_script" ]] && return 0

    # Wait a moment for signals to be written
    sleep 1

    # Run verification (silently, don't block the hook)
    python "$verify_script" --since 5 > /dev/null 2>&1 || {
        log_error "slide_verification" "Slide signals verification failed" "Bash" ""
    }
}

# ============================================
# RAIL CHECKS (Same as before but with logging)
# ============================================

check_intent() {
    if is_ralph_mode; then
        if [[ ! -f "$PRD_FILE" ]]; then
            log_error "hook_block" "No PRD found for Ralph mode" "" ""
            echo '{"error": "BLOCKED: No PRD found. Initialize with: ./.aria/ralph/ralph.sh init"}'
            exit 2
        fi
        return 0
    fi

    if [[ ! -f "$INTENT_FILE" ]]; then
        log_error "hook_block" "No intent defined" "" ""
        echo '{"error": "BLOCKED: No intent defined. Create .aria/intent.md first."}'
        exit 2
    fi
}

check_no_destructive() {
    local cmd="$1"
    local dangerous_patterns=(
        "rm -rf /"
        "rm -rf ~"
        "rm -rf \*"
        "> /dev/sd"
        "mkfs\."
        "dd if=.* of=/dev"
        ":(){ :|:& };:"
        "chmod -R 777 /"
        "DROP DATABASE"
        "DROP TABLE"
    )

    for pattern in "${dangerous_patterns[@]}"; do
        if echo "$cmd" | grep -qE "$pattern"; then
            log_error "hook_block" "Destructive command: $pattern" "Bash" ""
            echo "{\"error\": \"BLOCKED: Destructive command detected: $pattern\"}"
            exit 2
        fi
    done

    if echo "$cmd" | grep -qE "git push.*(--force|-f).*(main|master|production)"; then
        log_error "hook_block" "Force push to protected branch" "Bash" ""
        echo '{"error": "BLOCKED: Force push to protected branch"}'
        exit 2
    fi
}

# ============================================
# STATE TRACKING
# ============================================

STATE_EDIT_COUNT="$STATE_DIR/edit_count"
STATE_LAST_TEST="$STATE_DIR/last_test"
STATE_LAST_COMMIT="$STATE_DIR/last_commit"
STATE_TESTS_FAILED="$STATE_DIR/tests_failed"
STATE_LAST_PRE_SIGNAL="$STATE_DIR/last_pre_signal"

get_edit_count() { cat "$STATE_EDIT_COUNT" 2>/dev/null || echo 0; }
increment_edits() { echo $(($(get_edit_count) + 1)) > "$STATE_EDIT_COUNT"; }

track_test_result() {
    local exit_code="$1"
    echo "$(get_edit_count)" > "$STATE_LAST_TEST"
    if [[ "$exit_code" == "0" ]]; then
        rm -f "$STATE_TESTS_FAILED"
    else
        touch "$STATE_TESTS_FAILED"
    fi
}

track_commit() {
    echo "$(get_edit_count)" > "$STATE_LAST_COMMIT"
}

# ============================================
# MAIN HOOK LOGIC
# ============================================

case "$HOOK_EVENT" in
    "PreToolUse")
        if [[ -z "$TOOL_NAME" ]]; then
            # No tool data available (VS Code extension limitation)
            exit 0
        fi

        # Log the pre-signal with full context
        pre_signal_id=$(log_pre_signal "$TOOL_NAME" "$TOOL_INPUT")
        echo "$pre_signal_id" > "$STATE_LAST_PRE_SIGNAL"

        # Additional logging for special cases
        case "$TOOL_NAME" in
            "Read")
                file_path=$(extract_json_field "$TOOL_INPUT" "file_path")
                if [[ "$file_path" == *".aria/skills/"* ]]; then
                    skill_name=$(basename "$file_path" .md)
                    log_skill_load "$skill_name" "$file_path"
                fi
                ;;
            "Task")
                subagent=$(extract_json_field "$TOOL_INPUT" "subagent_type")
                desc=$(extract_json_field "$TOOL_INPUT" "description")
                prompt=$(extract_json_field "$TOOL_INPUT" "prompt")
                log_agent_spawn "$subagent" "$desc" "$prompt"
                ;;
            "Edit"|"Write"|"MultiEdit")
                check_intent
                ;;
            "Bash")
                cmd=$(extract_json_field "$TOOL_INPUT" "command")
                check_no_destructive "$cmd"
                ;;
        esac
        ;;

    "PostToolUse")
        if [[ -z "$TOOL_NAME" ]]; then
            exit 0
        fi

        # Get pre-signal ID for correlation
        pre_signal_id=$(cat "$STATE_LAST_PRE_SIGNAL" 2>/dev/null || echo "")

        # Log post-signal with results
        log_post_signal "$TOOL_NAME" "$TOOL_INPUT" "$pre_signal_id"

        # Track state changes
        case "$TOOL_NAME" in
            "Edit"|"Write"|"MultiEdit")
                increment_edits

                # Trigger relevant tests based on file modified
                file_path=$(extract_json_field "$TOOL_INPUT" "file_path")
                if [[ -n "$file_path" ]]; then
                    trigger_tests_for_file "$file_path" &
                fi
                ;;
            "Bash")
                cmd=$(extract_json_field "$TOOL_INPUT" "command")
                exit_code=$(extract_json_number "$TOOL_INPUT" "exit_code")

                if echo "$cmd" | grep -qE "^(npm test|yarn test|pytest|jest|cargo test|go test)"; then
                    track_test_result "${exit_code:-0}"
                fi

                if echo "$cmd" | grep -q "^git commit"; then
                    track_commit
                fi

                # Trigger test verification after slide generation
                if echo "$cmd" | grep -q "generate-slides.py"; then
                    verify_slide_signals &
                fi
                ;;
        esac
        ;;

    "Stop")
        # Session end - could add summary signal here
        ;;
esac

exit 0
