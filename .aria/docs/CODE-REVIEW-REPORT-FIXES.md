# ARIA Framework Code Review - FIX TRACKER

**Original Review Date:** 2026-01-15
**Fix Tracking Started:** 2026-01-15
**Current Status:** IN PROGRESS

---

## Fix Progress Dashboard

| Category | Total | Fixed | In Progress | Parking Lot | Remaining |
|----------|-------|-------|-------------|-------------|-----------|
| **FIX NOW (Critical)** | 18 | 3 | 0 | 0 | 15 |
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
| **Commit** | (pending - this session) |
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

### Issue #3: Error Swallowing `|| true` ⏳ PENDING

| Field | Value |
|-------|-------|
| **Status** | ⏳ PENDING |
| **File** | `.aria/ralph/ralph.sh:583` (was 483) |
| **Impact** | Claude errors completely masked |

**Problem:**
```bash
output=$(echo "$full_prompt" | claude ... 2>&1 | tee /dev/stderr) || true
```

---

### Issue #4: No Test Suite ⏳ PENDING

| Field | Value |
|-------|-------|
| **Status** | ⏳ PENDING |
| **Scope** | Framework-wide |
| **Impact** | No automated validation of framework itself |

---

### Issue #5: Token Counting is Estimate ⏳ PENDING

| Field | Value |
|-------|-------|
| **Status** | ⏳ PENDING |
| **File** | `.aria/model-selector.sh` |
| **Impact** | Cost tracking inaccurate |

**Problem:**
```bash
input_tokens=$(( ${#full_prompt} / 4 ))  # Rough estimate
```

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

### Issue #9: `eval` Usage in Rails ⏳ PENDING

| Field | Value |
|-------|-------|
| **Status** | ⏳ PENDING |
| **File** | `.claude/hooks/aria-rails.sh` |
| **Impact** | Security risk if safety.json modified |

---

### Issue #10: URL Injection Risk ⏳ PENDING

| Field | Value |
|-------|-------|
| **Status** | ⏳ PENDING |
| **File** | `.aria/verify-executor.sh` |
| **Impact** | Command injection possible via APP_URL |

---

### Issue #11: Dashboard XSS Potential ⏳ PENDING

| Field | Value |
|-------|-------|
| **Status** | ⏳ PENDING |
| **File** | `.aria/dashboard/index.html` |
| **Impact** | User content not sanitized |

---

### Issue #12: HITL Not Writing Decisions ⏳ PENDING

| Field | Value |
|-------|-------|
| **Status** | ⏳ PENDING |
| **File** | `.aria/hitl.sh` |
| **Impact** | Decision audit trail incomplete |

---

### Issue #13: Context Refresh Not Implemented ⏳ PENDING

| Field | Value |
|-------|-------|
| **Status** | ⏳ PENDING |
| **Files** | Multiple |
| **Impact** | Documented feature doesn't exist |

---

### Issue #14: Session Start/End Not Tracked ⏳ PENDING

| Field | Value |
|-------|-------|
| **Status** | ⏳ PENDING |
| **File** | `.aria/model-selector.sh` |
| **Impact** | Cannot calculate session duration |

---

### Issue #15: Skill Touch Not Logged ⏳ PENDING

| Field | Value |
|-------|-------|
| **Status** | ⏳ PENDING |
| **File** | Hooks |
| **Impact** | No record of which skills were used |

---

### Issue #16: HITL Count Not Tracked ⏳ PENDING

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
| - | #2 | Updated 16 scripts to use `set -euo pipefail` for proper pipeline error handling | (pending) |
| - | #18 | Resolved by Issue #2 fix | (pending) |

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
