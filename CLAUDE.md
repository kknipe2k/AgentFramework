# ARIA Hybrid Mode

> Internal guidance + External verification

This project uses ARIA for AI-assisted development. When working in this codebase, follow the workflow and rules below.

---

## Router (FIRST STEP)

Size the project BEFORE doing any work. This determines which mode activates.

### Sizing Criteria

| Factor | SMALL | MEDIUM | LARGE | X-LARGE |
|--------|-------|--------|-------|---------|
| Tasks | 1-5 | 6-15 | 16-40 | 40+ |
| Lines of code | <2,000 | 2,000-10,000 | 10,000-50,000 | 50,000+ |
| Files | 1-5 | 6-20 | 21-50 | 50+ |
| New dependencies | 0-1 | 2-5 | 6-15 | 15+ |
| Auth/payments/DB | No | Read-only | Yes (one) | Yes (multiple) |

**Quick sizing:**
- If ANY factor is X-LARGE → size is X-LARGE
- If ANY factor is LARGE (and none X-LARGE) → size is LARGE
- If ANY factor is MEDIUM (and none LARGE/X-LARGE) → size is MEDIUM
- Otherwise → SMALL

### Size → Mode Mapping

| Size | Mode | Hierarchy | Example |
|------|------|-----------|---------|
| SMALL | LITE | Tasks only | CLI tool, script, bug fix |
| MEDIUM | STANDARD | Phases → Tasks | Basic API, simple web app |
| LARGE | FULL | Major Steps → Phases → Tasks | Full-stack app, complex feature |
| X-LARGE | FULL+ | Epics → Major Steps → Phases → Tasks | Enterprise app, platform |

**X-LARGE additions:** Mandatory design doc, architecture review checkpoint, HITL gates per epic, context refresh between epics.

### Router Output

After sizing, announce:

```
SIZE: [SMALL|MEDIUM|LARGE|X-LARGE]
MODE: [LITE|STANDARD|FULL|FULL+]
Reason: [1-line justification]
```

Then proceed with that mode's behavior below.

**User override:** If user says "use FULL mode" or "keep it lite", respect that.

---

## Mode Definitions (SINGLE SOURCE OF TRUTH)

> **Canonical Reference:** This section is the authoritative definition of ARIA modes.
> Other files (skills, docs) should reference this section, not duplicate it.

The Router above selects the mode. Reference these definitions for mode behavior.

### Feature Summary

| Feature | LITE | STANDARD | FULL | FULL+ |
|---------|------|----------|------|-------|
| Formal planning | Quick (optional) | Yes | Yes + risks | Yes + design doc |
| Brainstorming | Optional | Yes | Yes + research | Yes + prototyping |
| Verification gate | If tests exist | Every task | Mandatory | Mandatory |
| HITL checkpoints | Destructive only | Risky actions | All risky | Per epic + risky |
| Design notes | No | Key decisions | All reasoning | All + architecture |
| Failure escalation | Report & stop | 2 failures retry | 3-failure prompt | 3-failure prompt |
| Context refresh | No | Between phases | Between major steps | Between epics, major steps, phases |
| Progress announcements | Minimal | Yes | Yes + estimates | Yes + epic status |
| Implementation isolation | No (direct) | Subagents | Subagents | Subagents |

### Mode Details

#### LITE Mode
**Use when:** Quick task, 1-5 tasks, low risk, user wants speed

```
LITE MODE ACTIVE

Features ON:
✓ Quick planning (optional) - brief task list if helpful
✓ Light brainstorming (optional) - explore approaches if needed
✓ Basic verification (verify.sh) - if tests exist
✓ Git commit on completion

Features OFF:
✗ Formal planning phase - keep it light
✗ Implementation isolation - direct in main session
✗ HITL checkpoints - only for destructive actions
✗ Design notes - skip unless complex decision
✗ Failure escalation - just report and stop
✗ Context refresh prompts - not needed
✗ Progress announcements - minimal output
```

**Behavior:**
1. Clarify requirements (1-2 questions max)
2. Quick plan if helpful (no formal approval needed)
3. Implement directly (no subagents)
4. Run verify.sh if tests exist
5. Commit and done

---

#### STANDARD Mode
**Use when:** Medium task, 4-10 steps, some risk, normal workflow

```
STANDARD MODE ACTIVE

Features ON:
✓ Planning phase - create plan, get approval
✓ Verification gate - after each task
✓ HITL checkpoints - for risky actions
✓ Git commit per task
✓ Progress announcements

Features OFF:
✗ Design notes - only for key decisions
✗ Context refresh prompts - unless stuck
✗ Full failure escalation - retry twice then stop
```

**Behavior:**
1. Create plan (save to `.aria/state/current-plan.json`)
2. Present plan: `[a]pprove / [r]evise / [c]ancel`
3. Execute tasks one at a time
4. Verify after each task
5. HITL checkpoint before risky actions
6. Commit after each verified task
7. On 2 consecutive failures: stop and report

---

#### FULL Mode
**Use when:** Complex task, 10+ steps, high risk, production code, user wants maximum oversight

```
FULL MODE ACTIVE

Features ON:
✓ Planning phase with risk assessment
✓ Verification gate - mandatory every task
✓ HITL checkpoints - all risky actions
✓ Design notes - log all reasoning
✓ Failure escalation - 3 failures triggers options
✓ Context refresh prompt - offer after extended work
✓ Git commit per task
✓ Progress announcements with estimates
✓ Final summary report
```

**Behavior:**
1. Assess risks, identify "don't touch" areas
2. Create detailed plan with time estimates
3. Present plan with risks: `[a]pprove / [r]evise / [c]ancel`
4. For each task:
   - Announce: "Task N/M: {title} (~X min)"
   - Log reasoning to design notes
   - Implement
   - Verify (mandatory)
   - HITL if risky
   - Commit
5. After 3 consecutive failures: escalation prompt
6. On completion: summary report

---

#### FULL+ Mode
**Use when:** Enterprise app, 40+ tasks, multiple auth/payment/DB systems, platform-level work

```
FULL+ MODE ACTIVE

Features ON:
✓ Everything in FULL mode, plus:
✓ Mandatory design doc before coding
✓ Architecture review checkpoint
✓ HITL gates per epic (not just tasks)
✓ Context refresh between epics
✓ Epic-level progress tracking
✓ Final architecture validation
```

**Behavior:**
1. Create design doc (save to `.aria/docs/DESIGN.md`)
2. Architecture review checkpoint with user
3. Break into epics, each epic gets FULL mode treatment
4. For each epic:
   - HITL gate before starting
   - Execute with FULL mode rules
   - Context refresh after completion
   - Epic summary report
5. After 3 consecutive failures: escalation prompt
6. On completion: full project report + architecture validation

---

### Mode Selection Output

After assessment, announce:

```
MODE: [LITE|STANDARD|FULL|FULL+]
Reason: [brief justification]
Tasks: ~[N] estimated
```

Then proceed with that mode's behavior.

**User can override:** If user says "use FULL mode" or "keep it lite", respect that.

---

## Use Cases

ARIA supports three primary use cases:

### 1. Build (Greenfield)

Build a new application from scratch.

```
Router → Brainstorm → Prototype (optional) → Plan → Execute → Report
```

- Size determines mode and hierarchy depth
- All phases apply based on mode

### 2. Modify (Existing Codebase)

Come into a mature codebase and make changes (refactor, add features, fix bugs).

```
Router → Plan → Execute → Report
```

- Read `project-context.md` first for "don't touch" areas
- Verification ensures existing tests still pass
- Size the *change*, not the whole codebase

### 3. Research (Article/Paper Analysis)

Analyze an article or research paper, create documentation, optionally prototype.

```
Input article → Researcher skill → Brainstorm → IDEA.md
HITL: [p]rototype / [d]one with docs
  └─ If done → Generate report, stop
  └─ If prototype → Choose variant → Build
```

**If prototyping, ask about variant:**

| Variant | Description | Best For |
|---------|-------------|----------|
| **[1] Working mockup** | Minimal functional demo of core concept, quick and simple | Technical users, quick validation |
| **[2] Learning tool** | Guided step-by-step workflows, hover tooltips with definitions, verbose explanations, progressive disclosure, animated transitions, visual feedback on interactions, interactive exploration of parameters | New users, education, onboarding |
| **[3] Reference impl** | Production-style code structure, proper patterns, extensible | Developers building on it |

```
HITL: What type of prototype?
[1] mockup - minimal demo
[2] learning tool - guided, explanatory, interactive
[3] reference - production-style code
```

**Research outputs:**
- `.aria/docs/IDEA.md` - Analysis and brainstorm
- `.aria/reports/RESEARCH-[topic].md` - Final report (NotebookLM-ready)
- `.aria/prototypes/` - Optional prototype if requested

### Input Disambiguation: Repos

When input is a **repository** (not an article/paper), trigger a decision point:

```
HITL: This looks like a repository. What's your intent?

[r]esearch - Learn from this codebase (generate IDEA.md, prototype options)
[e]xplore - Analyze for changes (features, bugs, refactoring)
[d]ocument - Generate documentation only
```

| Choice | Flow | Output |
|--------|------|--------|
| **[r]esearch** | Research flow | IDEA.md + prototype HITL |
| **[e]xplore** | Modify flow | Plan for changes |
| **[d]ocument** | Report only | project-context.md |

**Trigger words that should prompt this HITL:**
- "research this repo"
- "analyze this codebase"
- "look at this project"
- "what does this repo do"

**Skip HITL if intent is clear:**
- "fix the bug in..." → Modify flow
- "add feature X" → Modify flow
- "learn from this repo" → Research flow

---

## Quick Start

**To build something:**
```
Plan first, then execute. Ask for approval before implementing.
```

**Workflow:**
1. Understand requirements (ask clarifying questions)
2. Create a plan (save to `.aria/state/current-plan.json`)
3. Get HITL approval: `[a]pprove / [r]evise / [c]ancel`
4. Execute tasks one at a time
5. Run verification after EACH task
6. Generate report when done

---

## Skills

Load and follow these skills:

| Skill | When to Use |
|-------|-------------|
| `.aria/skills/brainstorming.md` | Exploring options before planning |
| `.aria/skills/prototyping.md` | Visual mockups and API specs |
| `.aria/skills/planning.md` | Creating implementation plans |
| `.aria/skills/executing.md` | Implementing tasks |
| `.aria/skills/tracking.md` | Progress, time, token metrics |
| `.aria/skills/researcher.md` | Extracting concepts from articles |
| `.aria/skills/report-writer.md` | Generating final reports |

---

## Verification Gate (MANDATORY)

After EVERY task that modifies code, run:

```bash
bash .aria/verify.sh
```

**Rules:**
- If verification PASSES: Continue to next task
- If verification FAILS: STOP immediately. Report the failure. Wait for guidance.
- NEVER skip verification
- NEVER proceed past a failed gate

### Fallback: When verify.sh Doesn't Exist

If `.aria/verify.sh` is not present, use this fallback verification:

```
FALLBACK VERIFICATION (no verify.sh found)

1. Check for test command:
   - package.json has "test" script → run `npm test`
   - pytest.ini or pyproject.toml exists → run `pytest`
   - Cargo.toml exists → run `cargo test`

2. Check for lint command:
   - package.json has "lint" script → run `npm run lint`
   - .eslintrc* exists → run `npx eslint .`

3. Check for type errors:
   - tsconfig.json exists → run `npx tsc --noEmit`

4. If NONE of the above exist:
   - Run `git diff --stat` to show what changed
   - Ask user: "No automated tests found. Manual review needed?"
```

**LITE mode exception:** If no verify.sh AND no test infrastructure, skip verification but warn:
```
⚠️ No verification available - proceeding without tests
```

---

## Failure Escalation

Track consecutive failures. Threshold depends on mode:
- **LITE:** Report and stop (no retry)
- **STANDARD:** After **2 failures** → stop and report
- **FULL/FULL+:** After **3 failures** → escalation prompt

```
ESCALATION: 3 consecutive failures on [issue]

Options:
[r]etry with different approach
[f]resh session (start new context)
[s]kip this task
[a]bort execution

What would you like to do?
```

**Why this matters:**
- Repeated failures may indicate context drift
- Fresh session = clean slate, re-read files
- User decides, not automatic refresh

**Do NOT:**
- Keep retrying indefinitely
- Silently skip failing tasks
- Refresh context without asking

---

## HITL Checkpoints

Before these actions, STOP and ask user to confirm:

- [ ] Deleting any files
- [ ] Modifying files in "don't touch" areas (see `.aria/project-context.md`)
- [ ] Changing configuration files (package.json, tsconfig, etc.)
- [ ] Any action marked `[HITL]` in the plan
- [ ] Installing new dependencies
- [ ] Modifying security-sensitive code (auth, payments, etc.)

Format:
```
HITL CHECKPOINT: About to [action]
Proceed? [y]es / [n]o / [e]xplain
```

---

## State Files

| File | Purpose |
|------|---------|
| `.aria/state/current-plan.json` | Current plan with tasks |
| `.aria/state/progress.json` | Task completion status |
| `.aria/design-notes.md` | AI reasoning log |
| `.aria/project-context.md` | Project knowledge (if exists) |
| `.aria/docs/DESIGN.md` | Design doc (FULL+ only) |

---

## Plan Format

When creating a plan, use this JSON structure:

```json
{
  "id": "plan-YYYYMMDD-HHMMSS",
  "title": "Feature description",
  "status": "pending_approval",
  "created": "ISO timestamp",
  "tasks": [
    {
      "id": 1,
      "title": "Task title",
      "description": "What to do",
      "status": "pending",
      "hitl": false,
      "estimated_minutes": 15
    }
  ],
  "hitl_checkpoints": ["Before modifying X", "After completing Y"],
  "risks": ["Risk 1", "Risk 2"]
}
```

---

## Execution Rules

1. **One task at a time** - Complete and verify before starting next
2. **Announce progress** - "Starting task N: {title}"
3. **Small commits** - Commit after each task if tests pass
4. **Log reasoning** - Write assumptions/decisions to `.aria/design-notes.md`
5. **Fail fast** - Stop on verification failure, don't try to fix silently

---

## Task Isolation (IMPLEMENTATION ONLY)

**Planning and brainstorming always happen in the main session** - subagent isolation is specifically for implementation tasks to prevent context drift.

For fresh context per implementation task, use the **Task tool** to spawn subagents:

```
For each task in plan:
1. Use Task tool with subagent_type="general-purpose"
2. Prompt includes: task description, relevant files, skill instructions
3. Subagent implements ONLY that task
4. Subagent returns: files changed, status, any blockers
5. Main session runs verify.sh
6. Main session commits if passed
7. Main session updates progress.json
```

**Why subagents for implementation:**
- Fresh context per task (no pollution from previous failures)
- Isolated implementation (can't drift)
- Main session stays in control (orchestrator role)

**When to use:**
- LITE mode: Skip subagents (direct implementation, 1-5 tasks doesn't need isolation)
- STANDARD mode: Use subagents for implementation tasks
- FULL/FULL+ mode: Use subagents for all implementation tasks

**What stays in main session (all modes):**
- Planning and task breakdown
- Brainstorming and research
- Verification (verify.sh)
- Git commits
- Progress tracking

---

## Don't Touch Areas

Check `.aria/project-context.md` for areas that should NOT be modified without explicit approval. Common examples:
- Authentication/security modules
- Payment processing
- Database migrations (in production)
- CI/CD configuration

---

## Commands (if using slash commands)

| Command | Action |
|---------|--------|
| `/aria:plan` | Start planning mode |
| `/aria:status` | Show current plan and progress |
| `/aria:verify` | Run verification manually |

---

## Two Entry Points

ARIA supports two modes:

| Mode | Entry Point | Use Case |
|------|-------------|----------|
| **Hybrid** (this) | `CLAUDE.md` | Running inside Claude Code / VS Code |
| **External** | `ralph.sh` | Running from terminal, calling Claude as subprocess |

Both use the same verification gates and plan format.

---

*ARIA Hybrid - Fast execution with enforced verification*
