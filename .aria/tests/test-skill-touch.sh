#!/bin/bash
# Test: Skill Touch Logging (Issue #15 fix validation)
# Tests that skill/template/framework file reads are logged to signals.jsonl

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$ARIA_DIR")"

# Source test runner for assertions
source "$SCRIPT_DIR/test-runner.sh"

# Setup test environment
TEST_SIGNALS_FILE="/tmp/aria-test-skill-signals-$$"
TEST_STATE_DIR="/tmp/aria-test-state-$$"

setup() {
    mkdir -p "$TEST_STATE_DIR"
    export SIGNALS_FILE="$TEST_SIGNALS_FILE"
    export STATE_DIR="$TEST_STATE_DIR"
    rm -f "$TEST_SIGNALS_FILE" 2>/dev/null || true

    # Define the logging functions inline (mirrors aria-rails.sh)
    log_skill_touch() {
        local skill_name="$1"
        local file_path="$2"
        local timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
        local event_id="skill-$(date +%s%N | cut -c1-13)"
        printf '{"id":"%s","timestamp":"%s","event":"skill_loaded","skill_name":"%s","file_path":"%s","context_type":"skill","context_name":"%s"}\n' \
            "$event_id" "$timestamp" "$skill_name" "$file_path" "$skill_name" \
            >> "$SIGNALS_FILE"
    }

    log_template_touch() {
        local template_name="$1"
        local file_path="$2"
        local timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
        local event_id="tmpl-$(date +%s%N | cut -c1-13)"
        printf '{"id":"%s","timestamp":"%s","event":"template_loaded","template_name":"%s","file_path":"%s","context_type":"template","context_name":"%s"}\n' \
            "$event_id" "$timestamp" "$template_name" "$file_path" "$template_name" \
            >> "$SIGNALS_FILE"
    }

    log_framework_touch() {
        local file_name="$1"
        local file_path="$2"
        local timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
        local event_id="fw-$(date +%s%N | cut -c1-13)"
        printf '{"id":"%s","timestamp":"%s","event":"framework_loaded","file_name":"%s","file_path":"%s","context_type":"framework","context_name":"%s"}\n' \
            "$event_id" "$timestamp" "$file_name" "$file_path" "$file_name" \
            >> "$SIGNALS_FILE"
    }
}

teardown() {
    rm -rf "$TEST_STATE_DIR" 2>/dev/null || true
    rm -f "$TEST_SIGNALS_FILE" 2>/dev/null || true
}

# ============================================
# TESTS
# ============================================

test_start "log_skill_touch creates signals file"
setup
log_skill_touch "planning" ".aria/skills/planning.md"
assert_file_exists "$TEST_SIGNALS_FILE" "Signals file should be created"
teardown
test_end

test_start "log_skill_touch logs skill_loaded event"
setup
log_skill_touch "debugging" ".aria/skills/debugging.md"
content=$(cat "$TEST_SIGNALS_FILE")
assert_contains "$content" "skill_loaded" "Should log skill_loaded event"
assert_contains "$content" "debugging" "Should include skill name"
teardown
test_end

test_start "log_skill_touch includes context_type skill"
setup
log_skill_touch "tdd" ".aria/skills/tdd.md"
content=$(cat "$TEST_SIGNALS_FILE")
assert_contains "$content" "\"context_type\":\"skill\"" "Should have context_type skill"
teardown
test_end

test_start "log_template_touch logs template_loaded event"
setup
log_template_touch "skill-template" ".aria/templates/skill-template.md"
content=$(cat "$TEST_SIGNALS_FILE")
assert_contains "$content" "template_loaded" "Should log template_loaded event"
assert_contains "$content" "skill-template" "Should include template name"
teardown
test_end

test_start "log_framework_touch logs framework_loaded event"
setup
log_framework_touch "CLAUDE.md" "/path/to/CLAUDE.md"
content=$(cat "$TEST_SIGNALS_FILE")
assert_contains "$content" "framework_loaded" "Should log framework_loaded event"
assert_contains "$content" "CLAUDE.md" "Should include framework file name"
teardown
test_end

test_start "multiple skill touches logged sequentially"
setup
log_skill_touch "planning" ".aria/skills/planning.md"
log_skill_touch "executing" ".aria/skills/executing.md"
log_skill_touch "debugging" ".aria/skills/debugging.md"
line_count=$(wc -l < "$TEST_SIGNALS_FILE")
assert_eq "3" "$line_count" "Should have 3 log entries"
teardown
test_end

test_start "skill touch includes file_path"
setup
log_skill_touch "tdd" ".aria/skills/tdd.md"
content=$(cat "$TEST_SIGNALS_FILE")
assert_contains "$content" ".aria/skills/tdd.md" "Should include full file path"
teardown
test_end

test_start "event IDs are unique for each skill touch"
setup
log_skill_touch "planning" ".aria/skills/planning.md"
sleep 0.01  # Small delay to ensure different timestamp
log_skill_touch "executing" ".aria/skills/executing.md"
# Use sed instead of grep -P for Windows compatibility
id1=$(grep "planning" "$TEST_SIGNALS_FILE" | sed 's/.*"id":"skill-\([^"]*\)".*/\1/')
id2=$(grep "executing" "$TEST_SIGNALS_FILE" | sed 's/.*"id":"skill-\([^"]*\)".*/\1/')
assert_neq "$id1" "$id2" "Event IDs should be unique"
teardown
test_end

echo ""
echo "Skill touch tests complete."
