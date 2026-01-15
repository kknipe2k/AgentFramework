# ARIA Framework Code Review - FIX TRACKER

**Original Review Date:** 2026-01-15
**Fix Tracking Started:** 2026-01-15
**Current Status:** IN PROGRESS

---

## Fix Progress Dashboard

| Category | Total | Fixed | In Progress | Parking Lot | Remaining |
|----------|-------|-------|-------------|-------------|-----------|
| **FIX NOW (Critical)** | 18 | 12 | 0 | 1 | 5 |
| **FIX LATER** | 24 | 0 | 0 | 0 | 24 |
| **PARKING LOT** | 12 | - | - | 12 | - |

**Last Updated:** 2026-01-15

---

## FIX NOW - Critical Issues Tracker

### Issue #1: Bash 4+ Associative Arrays ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **File** | `.aria/ralph/ralph.sh` |
| **Original Line** | 41-42 |
| **Commit** | `f6fc856` |
| **Date Fixed** | 2026-01-15 |

**Original Problem:**
```bash
declare -A story_failures
```
Windows Git Bash ships with Bash 3.x. Associative arrays require Bash 4+.

**Solution Implemented:**
- Replaced with POSIX-compatible file-based storage (`.story_failures`)
- Added full traceability to `signals.jsonl`
- Atomic file operations (temp file + mv)
- Auto-archive to `.aria/logs/` on completion

**New Functions Added:**
- `get_story_failures(story_id)` - Read count
- `set_story_failures(story_id, count)` - Write count (atomic)
- `init_story_failures()` - Initialize for run
- `cleanup_story_failures()` - Archive on completion
- `_log_failure_tracking()` - Signal emission for traceability

**Traceability:**
All operations logged to `signals.jsonl` with event types:
- `init` - Run started
- `set` - Count updated
- `increment` - Failure occurred
- `reset` - Success cleared count
- `threshold_reached` - HITL triggered
- `cleanup` - Run completed

---

### Issue #2: `set -o pipefail` Missing ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **Files** | 16 scripts (all updated) |
| **Commit** | `5ac0d16` |
| **Date Fixed** | 2026-01-15 |

**Original Problem:**
```bash
# Exit code of npm test was LOST in pipelines
npm test 2>&1 | tee "$LOGS_DIR/unit_tests.log"
```

**Solution Implemented:**
All 16 scripts updated from `set -e` to `set -euo pipefail`:
- `-e` - Exit on error (already present)
- `-u` - Exit on undefined variable (NEW)
- `-o pipefail` - Return exit code of first failing command in pipeline (NEW)

**Scripts Updated:**
1. ✅ `.aria/verify.sh`
2. ✅ `.aria/verify-executor.sh`
3. ✅ `.aria/ralph/ralph.sh`
4. ✅ `.aria/model-selector.sh`
5. ✅ `.aria/git-ops.sh`
6. ✅ `.aria/hitl.sh`
7. ✅ `.aria/aria-engine.sh`
8. ✅ `.aria/agent-runner.sh`
9. ✅ `.aria/rails-executor.sh`
10. ✅ `.aria/hooks/install.sh`
11. ✅ `.aria/hooks/pre-commit`
12. ✅ `.aria/hooks/pre-push`
13. ✅ `.aria/scripts/setup-project.sh`
14. ✅ `.aria/scripts/trace-view.sh`
15. ✅ `.aria/scripts/query-decisions.sh`
16. ✅ `.aria/scripts/reconcile.sh`

**Verification:** All scripts pass `bash -n` syntax check

---

### Issue #3: Error Swallowing `|| true` ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **File** | `.aria/ralph/ralph.sh` |
| **Commit** | `2ce7c75` |
| **Date Fixed** | 2026-01-15 |

**Original Problem:**
```bash
output=$(echo "$full_prompt" | claude ... 2>&1 | tee /dev/stderr) || true
```
The `|| true` masked all Claude CLI errors - API failures, auth issues, network problems were silently ignored.

**Solution Implemented:**
Created comprehensive agent invocation system with full traceability:

**New Functions Added:**
- `_log_agent_invocation()` - Log all invocations to signals.jsonl
- `_check_agent_output_for_errors()` - Detect error patterns in output
- `invoke_agent()` - Wrapper with proper error handling

**Error Categories Detected:**
- `api_error` - API/auth/rate limit issues (recoverable)
- `network_error` - Connection/timeout issues (recoverable)
- `model_error` - Model not found/overloaded (recoverable)
- `cli_error` - Command not found/permissions (fatal)

**Behavior:**
- Exit code 0: Success, logged to signals.jsonl
- Exit code 1: Recoverable error, increments failure count, triggers HITL after 3 failures
- Exit code 2: Fatal error, stops execution immediately

**Traceability:**
All invocations logged to `signals.jsonl` with:
- Event ID, timestamp, agent type, status
- Exit code, model used, error type (if any)

---

### Issue #4: No Test Suite ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **Scope** | Framework-wide |
| **Commit** | (pending - this session) |
| **Date Fixed** | 2026-01-15 |

**Solution Implemented:**
Created comprehensive test framework with full traceability:

**Test Framework Structure:**
```
.aria/tests/
├── test-runner.sh          # Test framework with assertions
├── test-framework.sh       # Structure & syntax validation (32 assertions)
├── test-invoke-agent.sh    # Issue #3 fix validation (14 assertions)
└── test-story-failures.sh  # Issue #1 fix validation (11 assertions)
```

**Test Runner Features:**
- Lightweight bash test framework (no external dependencies)
- 15+ assertion functions (assert_eq, assert_contains, assert_file_exists, etc.)
- Full traceability - all results logged to signals.jsonl
- Color-coded output with pass/fail summary
- CI-ready (proper exit codes)

**Test Categories:**
1. **Framework Structure Tests** - Validates directories, files, scripts exist
2. **Syntax Validation Tests** - Confirms all scripts pass `bash -n`
3. **Pipefail Tests** - Verifies Issue #2 fix (`set -euo pipefail`)
4. **Story Failures Tests** - Validates Issue #1 fix (file-based storage)
5. **Invoke Agent Tests** - Validates Issue #3 fix (error handling)

**Integration:**
- Tests run automatically as part of `verify.sh` (Check 6)
- Can be run standalone: `.aria/tests/test-runner.sh`
- Individual test files can be run: `.aria/tests/test-runner.sh test-framework.sh`

---

### Issue #5: Token Counting is Estimate 🅿️ PARKED

| Field | Value |
|-------|-------|
| **Status** | 🅿️ PARKED |
| **File** | `.aria/model-selector.sh` |
| **Reason** | Acceptable limitation - estimate is "good enough" for cost tracking |

**Problem:**
```bash
input_tokens=$(( ${#full_prompt} / 4 ))  # Rough estimate
```

**Why Parked:**
- Token counting is for cost *estimation*, not billing
- Adding Python dependency (tiktoken) is heavy for bash framework
- Better heuristics are still just estimates
- Higher-priority traceability issues take precedence

---

### Issue #6: Windows .sh Files Unusable ⏳ PENDING

| Field | Value |
|-------|-------|
| **Status** | ⏳ PENDING |
| **Files** | All `.sh` files |
| **Impact** | Windows users cannot use framework |

---

### Issue #7: Race Conditions on State Files ⏳ PENDING

| Field | Value |
|-------|-------|
| **Status** | ⏳ PENDING |
| **Files** | Multiple |
| **Impact** | Potential state corruption |

---

### Issue #8: Missing requirements.txt ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **File** | `.aria/scripts/requirements.txt` |
| **Commit** | `0b7612d` (code review commit) |
| **Date Fixed** | 2026-01-15 |

---

### Issue #9: `eval` Usage in Rails ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **File** | `.aria/rails-executor.sh` |
| **Commit** | (pending - this session) |
| **Date Fixed** | 2026-01-15 |

**Original Problem:**
```bash
eval "$check" >/dev/null 2>&1
```
Commands from JSON rail files were executed via `eval`, allowing arbitrary code execution if JSON files were modified.

**Solution Implemented:**
1. **Command Validation** - `validate_command()` blocks dangerous patterns:
   - `rm -rf /`, `mkfs.`, fork bombs, etc.
   - Command substitution `$(...)` and backticks
   - Piping curl/wget to shell
   - Nested eval/exec

2. **Safe Execution** - `safe_execute()` replaces `eval`:
   - Validates command first
   - Uses `bash -c` (runs in subshell, limiting damage)
   - Blocks execution if dangerous pattern detected

3. **Traceability** - `_log_rail_signal()`:
   - Logs all rail checks to signals.jsonl
   - Events: check_pass, check_fail, autofix_attempt, blocked
   - Full audit trail of rail execution

---

### Issue #10: URL Injection Risk ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **File** | `.aria/verify-executor.sh` |
| **Commit** | (pending - this session) |
| **Date Fixed** | 2026-01-15 |

**Original Problem:**
```bash
APP_URL="${APP_URL:-http://localhost:3000}"
curl -s -o /dev/null -w "%{http_code}" "$APP_URL"
```
If APP_URL contained shell metacharacters (`;|&$`), commands could be injected.

**Solution Implemented:**
Added `validate_url()` function that checks URLs at startup:

1. **Format validation** - Must start with `http://` or `https://`
2. **Metacharacter blocking** - Rejects URLs containing: `;|&$`><(){}[]!#`
3. **Newline blocking** - Rejects URLs with `\n` or `\r`
4. **Early exit** - Script exits if APP_URL or API_URL are invalid

Both APP_URL and API_URL are validated before any curl commands run.

---

### Issue #11: Dashboard XSS Potential ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **File** | `.aria/dashboard/index.html` |
| **Commit** | (pending - this session) |
| **Date Fixed** | 2026-01-15 |

**Original Problem:**
The dashboard had an `escapeHtml()` function but it was not consistently applied, leaving multiple XSS vectors:
- `renderLineage`: Tool names, hashes, action text unescaped
- `renderDecisions`: Tool names, file paths, commands unescaped
- `renderCommits`: Tool names in badges unescaped

**Solution Implemented:**
Applied `escapeHtml()` consistently to all user-controlled content:

**Fixed in renderLineage:**
- `s.tool` - Signal tool names
- `node.tool` - Signal type titles
- `node.hash` - Commit hashes
- `node.name` - Skill/template/subagent names
- `node.action` - HITL/decision actions
- `node.response` - HITL responses
- `node.command` - Verify commands

**Fixed in renderDecisions:**
- `s.tool` - Signal tool names
- `s.file_path`, `s.command` - Signal details

**Fixed in renderCommits:**
- `tool` - Tool badge names

**The escapeHtml function (unchanged):**
```javascript
function escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
```

**Note:** This uses DOM-based encoding which handles all special characters including `<`, `>`, `&`, `"`, and `'`.

---

### Issue #12: HITL Not Writing Decisions ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **File** | `.aria/hitl.sh` |
| **Commit** | (pending - this session) |
| **Date Fixed** | 2026-01-15 |

**Original Problem:**
HITL events were logged to `hitl.log` and `progress.txt` but NOT to `signals.jsonl`, breaking traceability.

**Solution Implemented:**
Added `_log_hitl_signal()` function with full traceability:

**Events Logged:**
- `hitl_request_created` - When HITL checkpoint triggers
- `hitl_response_received` - When human responds
- `hitl_timeout` - When response times out

**Signal Data Captured:**
- Event ID, timestamp
- Request ID, request type (help/confirm/choice/input)
- Details, response text
- Context type: `hitl`, context name: `human_intervention`

**Integration Points:**
- `create_request()` - logs request creation
- `set_response()` - logs human response
- `wait_for_response()` - logs timeout events

**Tests Added:**
- `test-hitl-signals.sh` with 12 assertions validating signal logging

---

### Issue #13: Context Refresh Not Implemented ⏳ PENDING

| Field | Value |
|-------|-------|
| **Status** | ⏳ PENDING |
| **Files** | Multiple |
| **Impact** | Documented feature doesn't exist |

---

### Issue #14: Session Start/End Not Tracked ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **File** | `.aria/model-selector.sh` |
| **Commit** | (pending - this session) |
| **Date Fixed** | 2026-01-15 |

**Original Problem:**
Sessions had a `session_start` timestamp in token_usage.json but:
- No `session_end` timestamp
- No logging to signals.jsonl for traceability
- No way to calculate session duration or query session history

**Solution Implemented:**
Added session lifecycle tracking with full traceability:

**New Functions:**
- `_log_session_signal()` - Logs session events to signals.jsonl
- `_generate_session_id()` - Creates unique session identifiers
- `start_session()` - Starts session, logs to signals.jsonl
- `get_session_id()` - Returns current session ID
- `end_session()` - Ends session with duration/cost metrics

**Events Logged:**
- `session_started` - When workflow begins (mode, workflow type)
- `session_ended` - When workflow completes (status, duration, cost)

**Signal Data Captured:**
- Session ID, timestamp
- Mode (LITE/STANDARD/FULL/FULL+)
- Workflow type (build/modify/research)
- Duration in seconds
- Total cost, token counts
- Final status (completed/aborted/failed)

**CLI Commands Added:**
- `session-start [mode] [workflow]` - Start new session
- `session-end [status]` - End session with metrics
- `session-id` - Get current session ID

**Usage - Query sessions:**
```bash
grep "session_started" .aria/state/signals.jsonl | jq -r '.session_id'
grep "session_ended" .aria/state/signals.jsonl | jq '.metrics'
```

**Tests Added:**
- `test-session-tracking.sh` with 11 assertions validating session lifecycle

---

### Issue #15: Skill Touch Not Logged ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **File** | `.claude/hooks/aria-rails.sh` |
| **Commit** | (pending - this session) |
| **Date Fixed** | 2026-01-15 |

**Original Problem:**
When Claude reads skill files (`.aria/skills/*.md`), there was no explicit logging to track which skills were loaded during a session.

**Solution Implemented:**
Added explicit skill/template/framework touch logging functions to aria-rails.sh:

**New Functions:**
- `log_skill_touch()` - Logs `skill_loaded` event when skill file is read
- `log_template_touch()` - Logs `template_loaded` event when template is read
- `log_framework_touch()` - Logs `framework_loaded` event for CLAUDE.md, project-context.md

**Events Logged:**
- `skill_loaded` - When `.aria/skills/*.md` files are read
- `template_loaded` - When `.aria/templates/*.md` files are read
- `framework_loaded` - When CLAUDE.md or project-context.md is read

**Signal Data Captured:**
- Event ID, timestamp
- Skill/template/framework name
- File path
- Context type and name

**Usage - Query skill usage:**
```bash
grep "skill_loaded" .aria/state/signals.jsonl | jq -r '.skill_name' | sort | uniq -c
```

**Tests Added:**
- `test-skill-touch.sh` with 11 assertions validating skill touch logging

---

### Issue #16: HITL Count Not Tracked ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **Resolved By** | Issue #12 fix |
| **Date Fixed** | 2026-01-15 |

**Note:** HITL events are now logged to signals.jsonl (Issue #12), making them countable:
```bash
grep "hitl_request_created" .aria/state/signals.jsonl | wc -l
```

---

### Issue #17: No Agent Decision Summary ⏳ PENDING

| Field | Value |
|-------|-------|
| **Status** | ⏳ PENDING |
| **File** | `.aria/hitl.sh` |
| **Impact** | Cannot measure human intervention frequency |

---

### Issue #17: No Agent Decision Summary ⏳ PENDING

| Field | Value |
|-------|-------|
| **Status** | ⏳ PENDING |
| **File** | report-writer |
| **Impact** | Final reports lack decision traceability |

---

### Issue #18: verify.sh Exit Code Lost ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **File** | `.aria/verify.sh` |
| **Resolved By** | Issue #2 fix (set -euo pipefail) |
| **Date Fixed** | 2026-01-15 |

**Note:** This was a subset of Issue #2. Fixed by updating verify.sh to use `set -euo pipefail`.

---

## FIX LATER - Important Issues

| # | Issue | Status | Notes |
|---|-------|--------|-------|
| 1 | Color codes on Windows | ⏳ PENDING | |
| 2 | Inconsistent error messages | ⏳ PENDING | |
| 3 | Silent `2>/dev/null` | ⏳ PENDING | 47 occurrences |
| 4 | researcher.md too light | ⏳ PENDING | |
| 5 | slide-generation.md incomplete | ⏳ PENDING | |
| 6 | context-refresh.md underdeveloped | ⏳ PENDING | |
| 7 | No skill validation on load | ⏳ PENDING | |
| 8 | PRD format not validated | ⏳ PENDING | |
| 9 | Budget can go negative | ⏳ PENDING | |
| 10 | Dashboard data not persisted | ⏳ PENDING | |
| 11 | No rollback on failed verification | ⏳ PENDING | |
| 12 | Mode not enforced by code | ⏳ PENDING | |
| 13 | Subagent isolation optional | ⏳ PENDING | |
| 14 | No progress percentage | ⏳ PENDING | |
| 15 | mktemp behavior differences | ⏳ PENDING | |
| 16 | read command options | ⏳ PENDING | |
| 17 | No cleanup of old state files | ⏳ PENDING | |
| 18 | Learning data unbounded | ⏳ PENDING | |
| 19 | Missing DESIGN.md template | ⏳ PENDING | |
| 20 | No project initialization wizard | ⏳ PENDING | |
| 21 | Hooks timeout too short | ⏳ PENDING | |
| 22 | No dry-run mode | ⏳ PENDING | |
| 23 | Cost rates hardcoded | ⏳ PENDING | |
| 24 | No session resumption | ⏳ PENDING | |

---

## PARKING LOT - Future Enhancements

| # | Feature | Priority | Status |
|---|---------|----------|--------|
| 1 | Whisper integration | LOW | 🅿️ PARKED |
| 2 | Visual diff in dashboard | MEDIUM | 🅿️ PARKED |
| 3 | Multi-agent coordination | LOW | 🅿️ PARKED |
| 4 | Custom model endpoints | MEDIUM | 🅿️ PARKED |
| 5 | Real-time dashboard | LOW | 🅿️ PARKED |
| 6 | Skill marketplace | LOW | 🅿️ PARKED |
| 7 | Git bisect integration | MEDIUM | 🅿️ PARKED |
| 8 | PR template generation | MEDIUM | 🅿️ PARKED |
| 9 | Dependency graph visualization | LOW | 🅿️ PARKED |
| 10 | Cost prediction | MEDIUM | 🅿️ PARKED |
| 11 | Team collaboration | LOW | 🅿️ PARKED |
| 12 | Audit trail export | MEDIUM | 🅿️ PARKED |

---

## Fix Log (Chronological)

### 2026-01-15

| Time | Issue | Action | Commit |
|------|-------|--------|--------|
| - | #8 | Created requirements.txt | `0b7612d` |
| - | #1 | Fixed Bash 4+ associative arrays with file-based storage + traceability | `f6fc856` |
| - | #2 | Updated 16 scripts to use `set -euo pipefail` for proper pipeline error handling | `5ac0d16` |
| - | #18 | Resolved by Issue #2 fix | `5ac0d16` |
| - | #3 | Replaced `\|\| true` with proper invoke_agent() error handling + traceability | `2ce7c75` |
| - | #4 | Created test framework with 57 assertions, integrated with verify.sh | `4bfa64e` |
| - | #5 | Moved to parking lot (acceptable limitation for cost estimation) | - |
| - | #12 | Added HITL signal logging to signals.jsonl for traceability | (pending) |
| - | #15 | Added skill/template/framework touch logging to signals.jsonl | `35190a6` |
| - | #16 | Resolved by Issue #12 (HITL events now countable) | - |
| - | #9 | Replaced eval with safe_execute + command validation | `b9f86af` |
| - | #10 | Added URL validation to block shell metacharacters | `0e79c0a` |
| - | #14 | Added session lifecycle tracking with signals.jsonl logging | `3fb874e` |
| - | #11 | Applied escapeHtml() consistently to all user content in dashboard | (pending) |

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Fixed and committed |
| 🔄 | In progress |
| ⏳ | Pending (not started) |
| 🅿️ | Parked (future enhancement) |
| ❌ | Won't fix (with reason) |

---

*This is a living document. Update after each fix is completed.*
