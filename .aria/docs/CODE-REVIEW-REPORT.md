# ARIA Framework Complete Code Review

**Review Date:** 2026-01-15
**Reviewer:** Comprehensive Automated Review
**Framework Version:** Current (commit c467934)
**Scope:** Full codebase audit for production readiness

---

## Executive Summary

### Overall Assessment: **PARTIALLY PRODUCTION-READY**

The ARIA framework demonstrates **ambitious and innovative design** with excellent documentation and well-thought-out workflows. However, there are **significant issues** that require immediate attention before professional deployment.

| Category | Grade | Summary |
|----------|-------|---------|
| **Architecture** | B+ | Solid design, two entry points well-defined |
| **Documentation** | A- | Comprehensive, but some gaps with code |
| **Cross-Platform** | C | Major Windows compatibility issues |
| **Error Handling** | C+ | Inconsistent, many silent failures |
| **Security** | B- | Good intentions, weak implementation |
| **Testing** | D | No test suite for framework itself |
| **Session Tracking** | B | Functional but incomplete |
| **Code Quality** | C+ | Mixed - some excellent, some lazy |

### Critical Issues Count
- **FIX NOW:** 18 issues
- **FIX LATER:** 24 issues
- **PARKING LOT:** 12 items

---

## Section 1: Architecture Analysis

### 1.1 Strengths (Best Practices)

1. **Two Entry Points Well-Defined**
   - External mode (`ralph.sh`) for autonomous operation
   - Hybrid mode (`CLAUDE.md`) for IDE integration
   - Clear separation of concerns

2. **Mode-Based Scaling (LITE/STANDARD/FULL/FULL+)**
   - Appropriate complexity matching
   - Clear criteria for mode selection
   - Well-documented transitions

3. **Skill-Based Architecture**
   - Modular capability system
   - Clear composition patterns in `COMPOSITION.md`
   - Good separation between core and extended skills

4. **State Management Pattern**
   - Centralized in `.aria/state/`
   - JSON-based for interoperability
   - JSONL for append-only logs (decisions, signals)

5. **Safety Rails Concept**
   - Pre/post tool use hooks
   - Hard vs soft blocks distinction
   - Verification gates mandatory

### 1.2 Weaknesses (Architectural Issues)

1. **Missing Central Orchestrator for Claude Code**
   - `aria-engine.sh` exists but isn't integrated with Claude hooks
   - `.claude/hooks/aria-rails.sh` duplicates logic instead of delegating
   - **Impact:** Code duplication, maintenance burden

2. **Inconsistent Script Entry Points**
   - Some scripts expect to be run from project root
   - Others use `SCRIPT_DIR` for relative paths
   - No standardized working directory contract

3. **State File Race Conditions**
   - Multiple scripts can write to `progress.json` simultaneously
   - No file locking mechanism
   - **Impact:** Potential state corruption

4. **Skill Loading Not Standardized**
   - Skills are markdown files with no formal schema
   - No validation that skill files exist before reference
   - Missing skill results in silent failure

---

## Section 2: Cross-Platform Compatibility

### 2.1 CRITICAL: Windows Compatibility Issues

| Script | Issue | Severity |
|--------|-------|----------|
| `verify.sh` | Uses bash-only syntax, no .bat equivalent | HIGH |
| `common.sh` | ANSI colors break in cmd.exe | MEDIUM |
| `ralph.sh` | Uses `declare -A` (associative arrays) - bash 4+ only | HIGH |
| `hitl.sh` | Uses `read` with options not in Git Bash | MEDIUM |
| `git-ops.sh` | Uses `mktemp` which behaves differently | LOW |
| All `.sh` files | Shebang `#!/bin/bash` not portable | HIGH |

### 2.2 Specific Code Issues

**ralph.sh:41-42 - Bash 4+ Only:**
```bash
# Track consecutive failures per story
declare -A story_failures
```
**Problem:** Windows Git Bash ships with bash 3.x by default. Associative arrays require bash 4+.
**Fix:** Use external file or simpler data structure.

**common.sh - Color Codes:**
```bash
ARIA_RED='\033[0;31m'
```
**Problem:** These escape sequences don't render in Windows cmd.exe or PowerShell ISE.
**Fix:** Add detection: `if [[ "$TERM" == "dumb" ]] || [[ -z "$TERM" ]]; then ARIA_RED=""; fi`

**settings.json - Hook Commands:**
```json
"command": "bash .claude/hooks/aria-rails.sh PreToolUse \"$TOOL_NAME\" \"$TOOL_INPUT\""
```
**Problem:** Relies on `bash` being in PATH. On Windows, may need explicit path.
**Recent Fix:** Commit 8064807 added `bash` prefix - good, but needs testing on Windows.

### 2.3 setup-project.bat Analysis

The Windows batch file exists but has issues:

**Line 70:**
```batch
mklink "%PROJECT%\CLAUDE.md" "%ARIA%\CLAUDE.md" >nul
```
**Problem:** `mklink` requires Administrator privileges by default.
**Fix (already partially done):** Uses junctions (`/J`) for directories which don't need admin. But file symlinks still need admin or Developer Mode.

**Line 104:**
```batch
for %%s in (verify.sh common.sh ...) do (
    if exist "%ARIA%\.aria\%%s" mklink "%PROJECT%\.aria\%%s" "%ARIA%\.aria\%%s" >nul 2>&1
)
```
**Problem:** Linking .sh files on Windows is useless - they can't be executed directly.
**Fix:** Either copy files or create wrapper .bat files.

---

## Section 3: Error Handling Audit

### 3.1 Patterns Found

| Pattern | Count | Assessment |
|---------|-------|------------|
| `set -e` at script start | 18/24 | GOOD - early exit on error |
| `|| exit 1` after critical commands | 12/50 | POOR - inconsistent |
| `2>/dev/null` suppressing errors | 47 | CONCERNING - hides problems |
| Explicit error messages | 23 | MODERATE |
| Return code checking | 15/40 | POOR |

### 3.2 Specific Issues

**verify.sh:389-398:**
```bash
if npm test 2>&1 | tee "$LOGS_DIR/unit_tests.log"; then
    echo -e "${GREEN}Unit tests passed${NC}"
    echo "pass" > "$STATE_DIR/unit_tests"
    rm -f "$STATE_DIR/tests_failed"
else
    ...
fi
```
**Issue:** `npm test` exit code is lost due to pipe to `tee`. Need `set -o pipefail` or `${PIPESTATUS[0]}`.
**Grade:** C - Common mistake, needs fix.

**ralph.sh:483:**
```bash
output=$(echo "$full_prompt" | claude --dangerously-skip-permissions -p $model_flag 2>&1 | tee /dev/stderr) || true
```
**Issue:** `|| true` swallows ALL errors, including fatal ones. No way to detect if Claude crashed vs completed.
**Grade:** D - Needs proper error handling.

**model-selector.sh:96-98:**
```bash
local input_cost=$(echo "scale=6; $input_tokens * ${INPUT_COSTS[$model]} / 1000000" | bc)
```
**Issue:** If `bc` is not installed, silently fails. The `aria_check_deps` runs but script continues with empty values.
**Grade:** C - Dependency check exists but not enforced.

### 3.3 Silent Failures

Files with concerning `2>/dev/null` patterns:

1. **hitl.sh:multiple** - Suppresses git errors silently
2. **git-ops.sh:multiple** - Suppresses checkpoint failures
3. **aria-rails.sh:multiple** - Suppresses safety check failures

**Recommendation:** Replace with explicit error handling or at minimum log to a debug file.

---

## Section 4: Security Analysis

### 4.1 Strengths

1. **Secret Detection in safety.json:**
```json
"check": "test -z \"$(git diff --cached 2>/dev/null)\" || ! git diff --cached 2>/dev/null | grep -qE '(api[_-]?key|secret|password|token)\\s*[=:]\\s*[A-Za-z0-9_-]{16,}'"
```
Good pattern but regex could miss some formats.

2. **Pre-commit Hooks Available**
   - Secret detection before commit
   - Can block dangerous operations

3. **HITL Checkpoints for Destructive Actions**
   - Good concept
   - Requires approval for file deletion

### 4.2 Weaknesses

**verify.sh - Command Injection Risk:**
```bash
APP_URL="${APP_URL:-http://localhost:3000}"
# Later used in:
curl -s -o /dev/null -w "%{http_code}" "$APP_URL"
```
**Risk:** If `APP_URL` contains shell metacharacters, could inject commands.
**Fix:** Validate URL format before use.

**ralph.sh - Prompt Injection:**
```bash
full_prompt="$full_prompt
## Current PRD
\`\`\`json
$(cat "$PRD_FILE")
\`\`\`"
```
**Risk:** If PRD file contains crafted content, could manipulate AI behavior.
**Impact:** Low (user controls PRD), but worth noting.

**aria-rails.sh - Eval-like Patterns:**
```bash
eval "$check_cmd"
```
**Location:** Line ~150 (approximate)
**Risk:** If `safety.json` is modified maliciously, arbitrary code execution.
**Fix:** Use safer command execution or validate JSON schema.

### 4.3 Missing Security Features

1. No input sanitization on task descriptions
2. No output escaping in HTML dashboard
3. No rate limiting on model API calls
4. No credential storage best practices documented

---

## Section 5: Session Tracking & Metrics

### 5.1 Current Implementation (model-selector.sh)

**Capabilities:**
- Token usage tracking per model
- Cost estimation with configurable rates
- Learning system for model success rates
- Budget tracking and alerts

**Assessment:** B - Functional but needs improvements

### 5.2 What's Working

1. **Token Usage:**
```json
{
  "total_input_tokens": 0,
  "total_output_tokens": 0,
  "total_cost": 0.0,
  "by_model": {...}
}
```
Tracked in `logs/token_usage.json`

2. **Learning System:**
```json
{
  "task_types": {
    "feature": {"opus": {"success": 0, "fail": 0}, ...}
  }
}
```
Tracked in `logs/model_learning.json`

### 5.3 What's Missing

| Missing Feature | Priority | Notes |
|-----------------|----------|-------|
| Actual token counting | HIGH | Currently estimates (chars/4) |
| Session duration tracking | MEDIUM | No start/end timestamps per session |
| Skill touch tracking | HIGH | Skills loaded but not logged |
| Agent decision logging | MEDIUM | Decisions.jsonl exists but not always written |
| HITL touch tracking | HIGH | No count of human interventions |
| Cost per session breakdown | MEDIUM | Only aggregate |
| Prompt/response size history | LOW | Would help optimization |

### 5.4 Code Issue

**model-selector.sh:485-486:**
```bash
# Estimate tokens (rough: 4 chars = 1 token)
input_tokens=$(( ${#full_prompt} / 4 ))
output_tokens=$(( ${#output} / 4 ))
```
**Problem:** This is a rough estimate. Different models tokenize differently. Claude's tokenizer averages ~3.5 characters per token for English, but varies.
**Fix:** Use Claude's tokenizer API or the `tiktoken` library approximation.

---

## Section 6: Skill-by-Skill Evaluation

### 6.1 Skill Grades

| Skill | Grade | Completeness | Clarity | Accuracy | Notes |
|-------|-------|--------------|---------|----------|-------|
| planning.md | A- | 95% | Excellent | Accurate | Professional quality |
| executing.md | B+ | 85% | Good | Accurate | Needs mode variations |
| debugging.md | B | 80% | Good | Accurate | Missing troubleshooting trees |
| discovery.md | B- | 75% | Moderate | Accurate | Light on details |
| tdd.md | A | 98% | Excellent | Accurate | Best in collection |
| brainstorming.md | B+ | 85% | Good | Accurate | Well structured |
| prototyping.md | B | 80% | Good | Mostly accurate | Long but thorough |
| researcher.md | C+ | 70% | Moderate | Accurate | Too brief |
| slide-generation.md | C | 65% | Light | Partially accurate | References missing scripts |
| report-writer.md | B | 80% | Good | Accurate | Solid |
| tracking.md | B+ | 85% | Good | Mostly accurate | Missing some features |
| context-refresh.md | C+ | 70% | Moderate | Accurate | Underdeveloped |
| REGISTRY.md | A- | 90% | Excellent | Accurate | Good index |
| COMPOSITION.md | A | 95% | Excellent | Accurate | Excellent diagrams |

### 6.2 Detailed Skill Assessments

#### planning.md - Grade: A-
**Strengths:**
- Clear JSON schema for plans
- HITL integration well-documented
- Mode variations explained

**Weaknesses:**
- No example of complex multi-phase plan
- Missing error recovery guidance

**Professional Assessment:** Production-ready

---

#### tdd.md - Grade: A
**Strengths:**
- Complete TDD cycle explanation
- Code examples in multiple languages
- Anti-patterns documented
- Coverage guidelines clear

**Weaknesses:**
- None significant

**Professional Assessment:** Exemplary skill document

---

#### researcher.md - Grade: C+
**Strengths:**
- Clear purpose
- Basic workflow outlined

**Weaknesses:**
- Too brief (3.6 KB vs 13 KB for tdd.md)
- No examples of output format
- Missing integration details with brainstorming
- No guidance on handling large papers

**Professional Assessment:** Needs expansion to match other skills

---

#### slide-generation.md - Grade: C
**Strengths:**
- Concept is clear

**Weaknesses:**
- References `generate-slides.py` without verifying it works
- NotebookLM integration is vague
- No actual examples of generated output
- FOCUS.md template not provided

**Professional Assessment:** Underdeveloped, needs work

---

#### context-refresh.md - Grade: C+
**Strengths:**
- Addresses real problem (context drift)
- Clear trigger conditions

**Weaknesses:**
- Only 4 KB - too light
- No concrete handoff format examples
- Missing state preservation details
- No metrics on when to trigger

**Professional Assessment:** Needs significant expansion

---

## Section 7: Documentation vs Code Discrepancies

### 7.1 CLAUDE.md vs Actual Implementation

| Documented Feature | Implementation Status | Gap |
|-------------------|----------------------|-----|
| Four modes (LITE/STANDARD/FULL/FULL+) | Partial | Mode selection not enforced by code |
| Verification gate after every task | YES | verify.sh exists and works |
| Decision tracing | Partial | Hooks exist but don't always write |
| Context refresh prompts | NO | Referenced but not implemented |
| Failure escalation (3 failures) | YES | ralph.sh implements this |
| Subagent isolation | Documented | Not enforced, optional |
| Report generation | Partial | report-writer.md exists, script missing |
| Dashboard at localhost:8420 | YES | serve-dashboard.py works |

### 7.2 Missing Implementations

1. **`rails-executor.sh`** - Referenced in CLAUDE.md but file doesn't exist at `.aria/rails-executor.sh`
   - Actually exists at `.claude/hooks/aria-rails.sh`
   - Documentation inconsistency

2. **`generate-slides.py`** - Script exists but:
   - Requires `python-pptx` not in requirements
   - No requirements.txt file at all

3. **HITL Decision Recording** - `hitl.sh` doesn't always write to `decisions.jsonl`

### 7.3 Stale References

1. **CLAUDE.md Line ~450:**
```markdown
`.aria/scripts/reconcile.sh` - Verify claims match signals
```
Script exists but hasn't been updated for current signal format.

2. **REGISTRY.md** - Lists skills that don't have matching implementation scripts

---

## Section 8: Categorized Fix Recommendations

### 8.1 FIX NOW (Critical - Block Production Use)

| # | Issue | File(s) | Fix |
|---|-------|---------|-----|
| 1 | Bash 4+ associative arrays | ralph.sh | Use file-based storage or simple arrays |
| 2 | `set -o pipefail` missing | verify.sh, verify-executor.sh | Add to all scripts using pipes |
| 3 | Windows .sh files unusable | All .sh | Create .bat wrappers or cross-platform scripts |
| 4 | Token counting is estimate | model-selector.sh | Use proper tokenizer |
| 5 | No test suite | Framework-wide | Add basic test coverage |
| 6 | Error swallowing `\|\| true` | ralph.sh:483 | Proper error handling |
| 7 | Race conditions on state files | Multiple | Add file locking |
| 8 | Missing requirements.txt | Scripts dir | Create with dependencies |
| 9 | `eval` usage in rails | aria-rails.sh | Safer command execution |
| 10 | URL injection risk | verify-executor.sh | Input validation |
| 11 | Dashboard XSS potential | index.html | Sanitize user content |
| 12 | HITL not writing decisions | hitl.sh | Fix decision logging |
| 13 | Context refresh not implemented | Multiple | Implement or remove docs |
| 14 | Session start/end not tracked | model-selector.sh | Add timestamps |
| 15 | Skill touch not logged | Hooks | Add skill load signals |
| 16 | HITL count not tracked | hitl.sh | Add counter |
| 17 | No agent decision summary | report-writer | Implement |
| 18 | verify.sh exit code lost | verify.sh | Fix pipe handling |

### 8.2 FIX LATER (Important but Not Blocking)

| # | Issue | File(s) | Fix |
|---|-------|---------|-----|
| 1 | Color codes on Windows | common.sh | Add terminal detection |
| 2 | Inconsistent error messages | Multiple | Standardize format |
| 3 | Silent `2>/dev/null` | 47 occurrences | Log to debug file |
| 4 | researcher.md too light | Skills | Expand documentation |
| 5 | slide-generation.md incomplete | Skills | Add examples, templates |
| 6 | context-refresh.md underdeveloped | Skills | Expand significantly |
| 7 | No skill validation on load | Hooks | Add file existence check |
| 8 | PRD format not validated | ralph.sh | Add JSON schema validation |
| 9 | Budget can go negative | model-selector.sh | Add hard stop |
| 10 | Dashboard data not persisted | serve-dashboard.py | Add export function |
| 11 | No rollback on failed verification | git-ops.sh | Implement auto-rollback |
| 12 | Mode not enforced by code | Multiple | Add mode enforcement |
| 13 | Subagent isolation optional | CLAUDE.md | Make clear when required |
| 14 | No progress percentage | tracking.md | Calculate completion % |
| 15 | mktemp behavior differences | git-ops.sh | Cross-platform temp |
| 16 | read command options | hitl.sh | Use portable subset |
| 17 | No cleanup of old state files | State dir | Add rotation |
| 18 | Learning data unbounded | model-selector.sh | Add pruning |
| 19 | Missing DESIGN.md template | Templates | Add for FULL+ |
| 20 | No project initialization wizard | Scripts | Add interactive setup |
| 21 | Hooks timeout too short | settings.json | Make configurable |
| 22 | No dry-run mode | Multiple | Add --dry-run flag |
| 23 | Cost rates hardcoded | model-selector.sh | Make configurable |
| 24 | No session resumption | ralph.sh | Add checkpoint resume |

### 8.3 PARKING LOT (Future Enhancements)

| # | Feature | Description | Priority |
|---|---------|-------------|----------|
| 1 | Whisper integration | Voice input for HITL | LOW |
| 2 | Visual diff in dashboard | Show code changes | MEDIUM |
| 3 | Multi-agent coordination | Parallel subagents | LOW |
| 4 | Custom model endpoints | Support other LLMs | MEDIUM |
| 5 | Real-time dashboard | WebSocket updates | LOW |
| 6 | Skill marketplace | Share custom skills | LOW |
| 7 | Git bisect integration | Auto bug finding | MEDIUM |
| 8 | PR template generation | Auto-fill descriptions | MEDIUM |
| 9 | Dependency graph visualization | Show skill deps | LOW |
| 10 | Cost prediction | Estimate before run | MEDIUM |
| 11 | Team collaboration | Multi-user sessions | LOW |
| 12 | Audit trail export | Compliance reports | MEDIUM |

---

## Section 9: Code Quality Assessment

### 9.1 Examples of Good Code

**model-selector.sh - Learning System:**
```bash
# Get success rate for a model on a task type
get_success_rate() {
    local model="$1"
    local task_type="$2"

    init_learning

    python3 << EOF
import json
# ... clean Python handling
EOF
}
```
**Assessment:** Proper separation, clear purpose, good use of embedded Python for JSON.

**verify-executor.sh - Project Detection:**
```bash
detect_project_type() {
    if [[ -f "package.json" ]]; then
        if grep -q '"react"' package.json 2>/dev/null; then
            echo "react"
        # ... comprehensive detection
    fi
}
```
**Assessment:** Thorough, handles multiple project types, sensible fallbacks.

### 9.2 Examples of Slipshod Code

**ralph.sh:483 - Error Swallowing:**
```bash
output=$(echo "$full_prompt" | claude --dangerously-skip-permissions -p $model_flag 2>&1 | tee /dev/stderr) || true
```
**Problems:**
1. `|| true` swallows ALL errors
2. Pipe loses exit code
3. No distinction between Claude error and success
4. `$model_flag` unquoted (word splitting risk)

**Fix:**
```bash
set -o pipefail
if ! output=$(echo "$full_prompt" | claude --dangerously-skip-permissions -p "$model_flag" 2>&1 | tee /dev/stderr); then
    echo "ERROR: Claude execution failed" >&2
    log_iteration $iteration "$next_story" "CLAUDE_ERROR" $duration
    continue
fi
```

---

**verify.sh:131 - Secret Detection Regex:**
```bash
if git diff --cached --name-only 2>/dev/null | xargs grep -lE "(api[_-]?key|secret|password|token)\s*[=:]\s*['\"][A-Za-z0-9_\-]{10,}['\"]" 2>/dev/null; then
```
**Problems:**
1. Won't catch `API_KEY=sk-...` (no quotes)
2. Won't catch multiline secrets
3. Won't catch base64 encoded secrets
4. Won't catch JSON format `"key": "value"`
5. Silent failure if no files staged

**Better:**
```bash
# Use dedicated secret scanner like gitleaks or trufflehog
# Or at minimum, more comprehensive regex
SECRET_PATTERNS=(
    "['\"]?[Aa][Pp][Ii][_-]?[Kk][Ee][Yy]['\"]?\s*[=:]\s*['\"]?[A-Za-z0-9_\-]{16,}['\"]?"
    "['\"]?[Ss][Ee][Cc][Rr][Ee][Tt]['\"]?\s*[=:]\s*['\"]?[A-Za-z0-9_\-]{16,}['\"]?"
    "-----BEGIN [A-Z]+ PRIVATE KEY-----"
    "ghp_[A-Za-z0-9]{36}"  # GitHub PAT
    "sk-[A-Za-z0-9]{48}"   # OpenAI key
)
```

---

**hitl.sh - Inconsistent Return Codes:**
```bash
request_human_help() {
    # ... lots of code ...
    return 1  # Always returns 1?
}
```
**Problem:** Function returns 1 even on success in some paths. Callers can't distinguish success from failure.

### 9.3 Unprofessionally Light Areas

1. **researcher.md** - Only 3.6 KB for a complex workflow
2. **context-refresh.md** - Only 4 KB, missing critical details
3. **pause.sh** - Stub-like implementation (3.2 KB)
4. **discover.sh** - Large (18 KB) but much is boilerplate

### 9.4 Dead Code / Unused

1. **archive/** - Entire directory of old ARIA language spec (should be in separate branch)
2. **src/** - Example counter code seems abandoned
3. **planner/prompt.md** - Not loaded by any code I can find

---

## Section 10: Recommendations Summary

### Immediate Actions (Before any professional use)

1. **Add Test Suite**
   - Unit tests for critical bash functions
   - Integration tests for workflows
   - Use bats-core or shunit2

2. **Fix Windows Compatibility**
   - Test all scripts in Git Bash
   - Create PowerShell equivalents for critical paths
   - Add CI for Windows

3. **Add requirements.txt**
```
python-pptx>=0.6.21
# For slide generation
```

4. **Fix Error Handling**
   - Add `set -o pipefail` to all scripts
   - Replace `|| true` with proper handling
   - Log errors instead of `/dev/null`

5. **Complete Tracking System**
   - Add skill load tracking to hooks
   - Add HITL touch counter
   - Fix decision logging

### Short-term (Within 1-2 sprints)

1. Expand light skill documents
2. Add schema validation for JSON files
3. Implement context-refresh properly
4. Add session start/end timestamps
5. Create comprehensive .bat equivalents

### Long-term

1. Consider rewriting critical scripts in Python for cross-platform
2. Add real token counting
3. Build proper test infrastructure
4. Consider TypeScript for dashboard interactivity

---

## Appendix A: File-by-File Issues

### .aria/verify.sh
- Line 35: Missing `set -o pipefail`
- Line 131: Weak secret regex
- Lines 150-160: Silent npm/pytest failures possible

### .aria/ralph/ralph.sh
- Line 41-42: Bash 4+ only associative arrays
- Line 483: Error swallowing
- Line 664: Unquoted variable expansion

### .aria/model-selector.sh
- Line 15: `bc` dependency may fail silently
- Line 485: Token estimate not accurate
- Line 571: Budget can go negative

### .claude/hooks/aria-rails.sh
- Line ~150: `eval` usage
- Multiple: Error suppression

### .aria/verify-executor.sh
- Line 29: URL injection possible
- Line 389: Pipe exit code lost

---

## Appendix B: Skill Invocation Matrix

| Skill | Called By | Triggers | Outputs |
|-------|-----------|----------|---------|
| planning.md | User, brainstorming | "plan", "/plan", mode start | current-plan.json |
| executing.md | planning | Plan approval | Code changes, commits |
| debugging.md | executing (on failure) | Test failure, error | Fix, design-notes entry |
| discovery.md | User, modify flow | New codebase | project-context.md |
| tdd.md | executing, planning | "tdd", critical task | Tests + implementation |
| brainstorming.md | User, research flow | "brainstorm", unclear approach | IDEA.md |
| prototyping.md | brainstorming, research | Prototype decision | .aria/prototypes/* |
| researcher.md | User | Article/paper URL | concepts.json, IDEA.md |
| slide-generation.md | researcher | HITL slides decision | FOCUS.md, slides.pptx |
| report-writer.md | Workflow end | Completion | REPORT.md |
| tracking.md | executing (parallel) | Every task | progress.json updates |
| context-refresh.md | Long sessions | 3+ failures, phase end | Handoff summary |

---

*Review completed. This document should be treated as a living audit that requires follow-up action items to be tracked and resolved.*
