# ARIA Framework Code Review - FIX TRACKER

**Original Review Date:** 2026-01-15
**Fix Tracking Started:** 2026-01-15
**Current Status:** IN PROGRESS

---

## Fix Progress Dashboard

| Category | Total | Fixed | In Progress | Parking Lot | Remaining |
|----------|-------|-------|-------------|-------------|-----------|
| **FIX NOW (Critical)** | 20 | 16 | 0 | 1 | 3 |
| **CONCEPT GAPS** | 5 | 5 | 0 | 0 | 0 |
| **FIX LATER** | 24 | 0 | 0 | 0 | 24 |
| **PARKING LOT** | 12 | - | - | 12 | - |

**Last Updated:** 2026-01-16

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

### Issue #6: Windows .sh Files Unusable ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **Files** | `.aria/common.sh`, `.aria/project-context.md` |
| **Commit** | (pending - this session) |
| **Date Fixed** | 2026-01-15 |

**Original Problem:**
All ARIA scripts are bash (`.sh`). Windows CMD/PowerShell cannot run these natively, causing cryptic errors for Windows users.

**Solution Implemented:**
Runtime detection + clear user guidance (Option B approach).

**Changes Made:**

1. **common.sh - Windows compatibility check:**
   - Detects if running without proper bash environment
   - Displays clear ASCII box with three options:
     - Git Bash (easiest)
     - WSL/WSL2
     - VS Code with Git Bash terminal
   - Instructions specific to Claude Code in VS Code

2. **project-context.md - Platform requirements:**
   - Added table showing supported platforms
   - Clear instructions for VS Code terminal configuration

**Runtime Message (shown to Windows CMD/PowerShell users):**
```
╔═══════════════════════════════════════════════════════════════╗
║           ARIA: Windows Compatibility Notice                  ║
╠═══════════════════════════════════════════════════════════════╣
║  ARIA requires a Unix-like shell environment.                 ║
║  OPTIONS:                                                     ║
║  1. Git Bash - https://git-scm.com/download/win              ║
║  2. WSL - Run: wsl --install                                  ║
║  3. VS Code with Git Bash Terminal                            ║
╚═══════════════════════════════════════════════════════════════╝
```

**Supported Environments:**
| Platform | Support |
|----------|---------|
| macOS | ✅ Full |
| Linux | ✅ Full |
| Windows + Git Bash | ✅ Full |
| Windows + WSL/WSL2 | ✅ Full |
| Windows CMD/PowerShell | ❌ (guided to alternatives) |

---

### Issue #7: Race Conditions on State Files ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **File** | `.aria/common.sh` + all scripts |
| **Commit** | (pending - this session) |
| **Date Fixed** | 2026-01-15 |

**Original Problem:**
Multiple processes could read/write state files (progress.json, signals.jsonl) simultaneously, causing potential corruption.

**Solution Implemented:**
Replaced flock-based locking with **File Ownership Model** (single-writer pattern):

| File | Owner | Non-owners call |
|------|-------|-----------------|
| `signals.jsonl` | `emit_signal()` | All scripts delegate here |
| `decisions.jsonl` | `emit_decision()` | All scripts delegate here |

**New Functions in common.sh:**
- `emit_signal()` - SINGLE OWNER of signals.jsonl
  - Usage: `emit_signal EVENT CONTEXT_TYPE CONTEXT_NAME [key=value ...]`
  - Handles timestamps, IDs, JSON escaping centrally
  - All scripts must use this to write signals

- `emit_decision()` - SINGLE OWNER of decisions.jsonl
  - Usage: `emit_decision ACTION CONTEXT RATIONALE ALTERNATIVES CONFIDENCE [VERIFIED]`
  - Validates confidence 0.0-1.0
  - Centralized decision logging

- `aria_atomic_write()` - General-purpose atomic file writes

**Why Ownership Model:**
- JSONL appends are inherently atomic (single write operation)
- Centralized write logic ensures consistent schema
- No need for flock complexity (simpler, more portable)
- Better traceability (all writes go through one path)
- Prevents scattered direct file access

**Scripts Updated:**
- `hitl.sh` - `_log_hitl_signal()` now delegates to `emit_signal()`
- `model-selector.sh` - `_log_session_signal()` now delegates to `emit_signal()`
- `rails-executor.sh` - `_log_rail_signal()` now delegates to `emit_signal()`
- `ralph/ralph.sh` - `_log_failure_tracking()` and `_log_agent_invocation()` delegate
- `tests/test-runner.sh` - `_log_test_event()` delegates to `emit_signal()`

**Tests Added:**
- `test-safe-state.sh` with 18 assertions for ownership model

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

### Issue #13: Context Refresh Not Implemented ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **File** | `.aria/context-refresh.sh` |
| **Commit** | (pending - this session) |
| **Date Fixed** | 2026-01-15 |

**Original Problem:**
CLAUDE.md documents context refresh as a feature (STANDARD+, FULL, FULL+ modes) but no implementation existed. The skill file `.aria/skills/context-refresh.md` was documentation only.

**Solution Implemented:**
Created `context-refresh.sh` script with full functionality:

**Commands:**
- `save [name]` - Save checkpoint state to JSON
- `handoff [name]` - Generate markdown handoff summary
- `list` - Show available checkpoints and handoffs
- `load` - Display checkpoint details for resuming
- `cleanup [n]` - Keep only last N handoffs (default: 3)

**Checkpoint State Saved:**
```json
{
  "refresh_point": "after_phase_1",
  "timestamp": "ISO-8601",
  "plan_id": "plan-YYYYMMDD-HHMMSS",
  "progress": {
    "completed_tasks": ["1", "2"],
    "current_task": "3",
    "remaining_tasks": ["4", "5"],
    "total": 5,
    "completed": 2
  },
  "key_decisions": ["decision 1", "decision 2"],
  "files_modified": ["src/file.ts"],
  "blockers": [],
  "notes": ""
}
```

**Handoff Summary Generated:**
- Project name and branch
- Progress (X/Y tasks)
- Key files modified (from git)
- Key decisions (from decisions.jsonl)
- Don't-touch areas (from project-context.md)
- Next action to continue

**Traceability:**
All operations logged via `emit_signal()`:
- `context_checkpoint_saved` - When checkpoint is saved
- `context_handoff_created` - When handoff is generated
- `context_checkpoint_loaded` - When checkpoint is loaded

**Tests Added:**
- `test-context-refresh.sh` with 10 test cases

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

### Issue #17: No Agent Decision Summary ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **Files** | `.aria/scripts/generate-summary.py` (new) |
| **Commit** | (pending - this session) |
| **Date Fixed** | 2026-01-15 |

**Original Problem:**
Reports lacked decision traceability:
- No way to measure human intervention frequency
- Key decisions not extracted for reports
- No aggregate decision statistics

**Solution Implemented:**
Created `generate-summary.py` script with comprehensive reporting:

**Features:**
- Extracts decisions from `decisions.jsonl`
- Counts HITL interactions from `signals.jsonl`
- Ranks decisions by confidence score
- Calculates decision statistics (total, verified, confidence breakdown)
- Tracks session metadata (mode, workflow, duration)
- Reports skills loaded during session

**Output Formats:**
- `--format text` - Human-readable console output
- `--format json` - Structured JSON for APIs
- `--format markdown` - For report generation

**Metrics Included:**
- Total decisions with confidence breakdown
- HITL checkpoints (requests, responses, timeouts)
- Key decisions (top 5 by confidence)
- Skills loaded
- Token usage summary

**Usage:**
```bash
python .aria/scripts/generate-summary.py                    # Text output
python .aria/scripts/generate-summary.py --format markdown  # For reports
python .aria/scripts/generate-summary.py --format json      # For APIs
python .aria/scripts/generate-summary.py -o summary.md      # Save to file
```

**Integration:**
Report-writer skill can now call this script to generate decision summaries for inclusion in final reports.

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

### Issue #19: Windows Dashboard Errors ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **File** | `.aria/scripts/serve-dashboard.py` |
| **Commit** | `e5e43ba` |
| **Date Fixed** | 2026-01-16 |

**Original Problem:**
Dashboard failed on Windows with two errors:
1. `FileNotFoundError` for missing `favicon.ico`
2. `TypeError: argument of type 'HTTPStatus' is not iterable` in `log_message`

**Solution Implemented:**
1. **favicon.ico handling:** Added handler that returns HTTP 204 (No Content) instead of trying to serve a non-existent file
2. **log_message fix:** Added type check to verify `args[0]` is a string before using `in` operator

**Code Changes:**
```python
# Handle favicon.ico gracefully (return empty 204)
if path == '/favicon.ico':
    self.send_response(204)  # No Content
    self.end_headers()
    return

# Fixed log_message
def log_message(self, format, *args):
    # args[0] might be HTTPStatus enum on some platforms, not a string
    if args and isinstance(args[0], str) and '/api/' in args[0]:
        print(f"[API] {args[0]}")
```

---

### Issue #20: Windows CRLF Line Endings ✅ FIXED

| Field | Value |
|-------|-------|
| **Status** | ✅ FIXED |
| **File** | `.gitattributes` (new) |
| **Commit** | `4a2caa2` |
| **Date Fixed** | 2026-01-16 |

**Original Problem:**
Shell scripts cloned on Windows had CRLF line endings (`\r\n`) instead of LF (`\n`), causing bash to fail with:
```
.claude/hooks/aria-rails.sh: line 5: $'\r': command not found
```

**Solution Implemented:**
Created `.gitattributes` file to force LF line endings for shell scripts:

```gitattributes
# Force LF line endings for shell scripts (prevents CRLF issues on Windows)
*.sh text eol=lf
*.bash text eol=lf
*.py text eol=lf
```

**Note:** Existing clones need to re-checkout files or re-clone for the fix to take effect.

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

### 2026-01-16

| Time | Issue | Action | Commit |
|------|-------|--------|--------|
| - | #19 | Fixed Windows dashboard errors (favicon.ico, log_message TypeError) | `e5e43ba` |
| - | #20 | Added .gitattributes to force LF line endings for shell scripts | `4a2caa2` |
| - | GAP#1 | Created Boris Pattern 3 agents (analyzer.md, implementer.md) | `87f99c8` |
| - | GAP#2 | Completed observability system (dashboard, metrics, hierarchical view) | `1d8e13f` |
| - | GAP#3 | Removed Slack/Email notification references from docs | `87f99c8` |
| - | GAP#4 | Documented CLAUDE.md size acceptance as design decision | `87f99c8` |
| - | GAP#5 | Updated learnings structure with Architecture/Testing/Gotchas categories | `288e638` |

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
| - | #11 | Applied escapeHtml() consistently to all user content in dashboard | `29e2cd5` |
| - | #17 | Created generate-summary.py for decision summaries in reports | `a63c9d6` |
| - | #7 | Added safe state file operations with flock and atomic writes | (pending) |

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
