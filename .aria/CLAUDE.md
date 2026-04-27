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
8. **On completion:** Generate summary report with metrics
9. **HITL:** "View dashboard? [y]es / [n]o / [s]ave report"

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
6. **On completion:** Full summary report with metrics comparison
7. **HITL:** "View dashboard? [y]es / [n]o / [s]ave report"
8. If [y]es: Launch dashboard at http://localhost:8420

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
6. **On completion:** Full project report + architecture validation
7. **HITL:** "View dashboard? [y]es / [n]o / [s]ave report"
8. If [y]es: Launch dashboard at http://localhost:8420

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

ARIA supports four primary use cases:

### 1. Build (Greenfield)

Build a new application from scratch.

```
Router → Brainstorm → Prototype (optional) → Plan → Execute → Report
```

- Size determines mode and hierarchy depth
- All phases apply based on mode
- **Prototype phase** (if used):
  - `prototyping.md` generates SPEC-*.json
  - `executing.md` builds prototype using agent loop
  - `verify.sh` validates (HTML, CSS, JS, Playwright)

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

**MANDATORY SEQUENTIAL STEPS - DO NOT SKIP OR COMBINE:**

```
STEP 1: Extract concepts
        → Run researcher skill
        → Output: .aria/docs/research-output.json
        → WAIT for completion

STEP 2: Synthesize findings
        → Run brainstorming skill
        → Output: .aria/docs/IDEA.md
        → WAIT for completion

STEP 3: HITL - Slides decision (STOP AND ASK)
        ┌────────────────────────────────────────┐
        │ HITL: Generate presentation slides?    │
        │ [y]es / [n]o                           │
        └────────────────────────────────────────┘
        → WAIT for user response before proceeding

STEP 4: If slides = yes (STOP AND ASK)
        → Generate FOCUS.md first
        ┌────────────────────────────────────────┐
        │ HITL: Slide generation method?         │
        │ [1] NotebookLM (richer, needs auth)    │
        │ [2] Local pptx (no dependencies)       │
        └────────────────────────────────────────┘
        → WAIT for user response
        → Generate slides with chosen method
        → Output: .aria/outputs/slides-*.pdf|pptx

STEP 5: HITL - Prototype decision (STOP AND ASK)
        ┌────────────────────────────────────────┐
        │ HITL: Create prototype?                │
        │ [p]rototype / [d]one with docs         │
        └────────────────────────────────────────┘
        → WAIT for user response before proceeding
        → If [d]one: Skip to STEP 7 (MANDATORY)

STEP 6: If prototype = yes (STOP AND ASK)
        ┌────────────────────────────────────────┐
        │ HITL: What type of prototype?          │
        │ [1] mockup - minimal demo              │
        │ [2] learning tool - guided, interactive│
        │ [3] reference - production-style       │
        └────────────────────────────────────────┘
        → WAIT for user response

        STEP 6a: Generate prototype spec
        → Run prototyping skill
        → Output: .aria/prototypes/SPEC-[name].json
        → Present spec for approval

        STEP 6b: Build prototype (via executing.md)
        → executing.md receives spec + variant
        → Agent loop: analyzer → implementer → verify-app
        → verify.sh runs: linting, Playwright, accessibility
        → Output: .aria/prototypes/prototype-[name].html

        → Continue to STEP 7 (MANDATORY)

STEP 7: MANDATORY - Final report & dashboard (ALWAYS EXECUTE)
        ┌────────────────────────────────────────┐
        │ This step is REQUIRED regardless of    │
        │ whether prototype was built or skipped │
        └────────────────────────────────────────┘
        → List all artifacts created
        → Run report-writer skill
        → Display summary with metrics
        ┌────────────────────────────────────────┐
        │ HITL: View dashboard?                  │
        │ [y]es - launch localhost:8420          │
        │ [n]o - done                            │
        │ [s]ave - export to .aria/reports/      │
        └────────────────────────────────────────┘
        → WAIT for user response
        → Execute chosen action
        → WORKFLOW COMPLETE
```

**CRITICAL: Each HITL checkpoint is BLOCKING. Do NOT proceed until user responds.**
**CRITICAL: STEP 7 is MANDATORY - always execute after prototype decision (yes or no).**

**Slide Generation Details:**

After IDEA.md is created, offer slide deck generation:

```
HITL: Generate presentation slides?
[y]es / [n]o, skip to prototype decision
```

If yes:
```
Sources for Focus doc:
1. .aria/docs/IDEA.md
2. [original paper/article]
3. [additional sources...]

Confirm sources? [y]es / [e]dit / [c]ancel

→ Generate FOCUS.md (Core Ideas + Synthesis Matrix)

HITL: Output method?
[1] NotebookLM (richer design, requires auth)
[2] Local pptx (reliable, no dependency)

→ Generate slides
```

See `.aria/skills/slide-generation.md` for prompts and details.

**Prototype Variants:**

| Variant | Description | Best For |
|---------|-------------|----------|
| **[1] Working mockup** | Minimal functional demo of core concept, quick and simple | Technical users, quick validation |
| **[2] Learning tool** | Guided step-by-step workflows, hover tooltips with definitions, verbose explanations, progressive disclosure, animated transitions, visual feedback on interactions, interactive exploration of parameters | New users, education, onboarding |
| **[3] Reference impl** | Production-style code structure, proper patterns, extensible | Developers building on it |

**Research outputs:**
- `.aria/docs/IDEA.md` - Analysis and brainstorm
- `.aria/outputs/FOCUS.md` - Core ideas + synthesis matrix (if slides requested)
- `.aria/outputs/slides-*.pdf|pptx` - Presentation deck (if slides requested)
- `.aria/reports/RESEARCH-[topic].md` - Final report (NotebookLM-ready)
- `.aria/prototypes/` - Optional prototype if requested

### 4. Deep Research (Any Question)

Systematic web research on any topic with iterative refinement and HITL gates.

**Triggers:** "research X", "investigate", "deep dive into", "find out about"

```
STEP 1: Depth Selection (HITL)
        +----------------------------------------+
        | RESEARCH DEPTH SELECTION               |
        | [1] Quick (5-10 min)                   |
        | [2] Standard (15-30 min) - RECOMMENDED |
        | [3] Deep (30-60 min)                   |
        | [4] Exhaustive (60+ min)               |
        +----------------------------------------+

STEP 2: Strategy Selection (HITL)
        +----------------------------------------+
        | QUERY STRATEGY                         |
        | [a] Broad Scan - start wide, narrow    |
        | [b] Focused Drill - specific queries   |
        | [c] Comparative - X vs Y analysis      |
        | [d] Temporal - track changes over time |
        | [e] Custom - define your own           |
        +----------------------------------------+

STEP 3: Query Approval (HITL)
        -> Present proposed queries
        -> [a]pprove / [e]dit / [c]hange strategy

STEP 4: Search Loop
        -> Execute queries (WebSearch tool)
        -> Evaluate source quality (A/B/C/D rating)
        -> Extract findings with confidence scores
        -> Identify gaps and contradictions
        -> Formulate follow-up queries

STEP 5: Mid-Research Checkpoint (HITL)
        +----------------------------------------+
        | RESEARCH CHECKPOINT                    |
        | [c]ontinue - proceed with new queries  |
        | [r]edirect - change focus              |
        | [d]eepen - more on specific finding    |
        | [s]ynthesize - enough info gathered    |
        | [a]bort - stop research                |
        +----------------------------------------+

STEP 6: Synthesis Approach (HITL)
        +----------------------------------------+
        | SYNTHESIS OPTIONS                      |
        | [1] Executive Summary - brief          |
        | [2] Structured Analysis - full IDEA.md |
        | [3] Comparative Matrix - side-by-side  |
        | [4] Annotated Bibliography             |
        | [5] All of the above                   |
        +----------------------------------------+

STEP 7: Output & Continue (HITL)
        -> Generate research-output.json
        -> Generate IDEA.md
        +----------------------------------------+
        | Continue?                              |
        | [s]lides - generate presentation       |
        | [p]rototype - build working demo       |
        | [b]oth - slides then prototype         |
        | [d]one - research complete             |
        +----------------------------------------+
```

**Confidence Scoring:**
| Score | Label | Meaning |
|-------|-------|---------|
| 0.9+ | VERY HIGH | Multiple authoritative sources agree |
| 0.7-0.89 | HIGH | Good sources, some corroboration |
| 0.5-0.69 | MEDIUM | Limited sources or some uncertainty |
| 0.3-0.49 | LOW | Single source or quality concerns |
| <0.3 | UNVERIFIED | Treat as hypothesis |

**Deep Research outputs:**
- `.aria/docs/research-output.json` - Full research trace
- `.aria/docs/IDEA.md` - Synthesized findings
- Source quality ratings and confidence scores
- Contradiction analysis
- Gap identification

**When to use Deep Research vs Native Claude Search:**
| Aspect | Native WebSearch | ARIA Deep Research |
|--------|------------------|-------------------|
| Speed | Fast (seconds) | Slower (minutes) |
| Depth | Single query | Iterative refinement |
| Oversight | None | HITL gates |
| Traceability | None | Full decision trail |
| Source tracking | Implicit | Explicit with ratings |
| Output | Prose response | Structured artifacts |

See `.aria/skills/deep-research.md` for complete workflow details.

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

### Core Skills (All Modes)

| Skill | When to Use |
|-------|-------------|
| `.aria/skills/aria-start.md` | **Session init**: Dashboard + workflow router |
| `.aria/skills/planning.md` | Creating implementation plans |
| `.aria/skills/executing.md` | Implementing tasks from approved plan |
| `.aria/skills/debugging.md` | Test failures, errors, troubleshooting |
| `.aria/skills/discovery.md` | Exploring unfamiliar codebase (Modify flow) |
| `.aria/skills/tdd.md` | Test-driven development |

### Extended Skills (STANDARD+)

| Skill | When to Use |
|-------|-------------|
| `.aria/skills/brainstorming.md` | Exploring options before planning |
| `.aria/skills/prototyping.md` | Generate prototype specs (executing.md builds) |
| `.aria/skills/tracking.md` | Progress, time, token metrics |
| `.aria/skills/context-refresh.md` | Between phases, after 3+ failures |
| `.aria/skills/report-writer.md` | End-of-workflow summary + dashboard |

### Meta Skills (STANDARD+)

| Skill | When to Use |
|-------|-------------|
| `.aria/skills/meta-reasoning.md` | **Systematic decision-making with offline RL** |

### Research Skills (Explicit)

| Skill | When to Use |
|-------|-------------|
| `.aria/skills/researcher.md` | Extracting concepts from articles/papers |
| `.aria/skills/deep-research.md` | Web research with HITL gates on any topic |
| `.aria/skills/slide-generation.md` | Create presentations from IDEA.md |

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
| `.aria/state/decisions.jsonl` | Decision trace log |
| `.aria/state/signals.jsonl` | Tool call signals (from hooks) |
| `.aria/design-notes.md` | AI reasoning log |
| `.aria/project-context.md` | Project knowledge (if exists) |
| `.aria/docs/DESIGN.md` | Design doc (FULL+ only) |

### Learning Files (Offline RL)

| File | Purpose |
|------|---------|
| `.aria/learned/policy.json` | Current learned policy (loaded at session start) |
| `.aria/learned/priors/model-selection.json` | Beta priors for model × context |
| `.aria/learned/priors/strategy-selection.json` | Beta priors for strategy × context |
| `.aria/learned/priors/confidence-calibration.json` | Confidence adjustment factors |
| `.aria/learned/history/episodes.jsonl` | Historical (state, action, reward) tuples |
| `.aria/logs/model_learning.json` | Model performance tracking |

---

## Offline Reinforcement Learning

ARIA learns from past sessions to improve decision-making over time.

### How It Works

```
SESSION N                         BETWEEN SESSIONS
┌──────────────┐                  ┌──────────────────────┐
│ Execute with │                  │ Learning Pipeline    │
│ current      │──────────────────▶│                      │
│ policy       │   signals.jsonl  │ 1. Extract episodes  │
│              │   decisions.jsonl│ 2. Calculate rewards │
│              │   outcomes       │ 3. Update priors     │
└──────────────┘                  │ 4. Export policy     │
       ▲                          └──────────┬───────────┘
       │                                     │
       └─────────────────────────────────────┘
                  SESSION N+1 uses improved policy
```

### What Gets Learned

| Decision Point | What's Learned | Data Source |
|----------------|----------------|-------------|
| Model selection | Which models succeed for task types | `model_learning.json` |
| Strategy selection | Which approaches work when | `decisions.jsonl` |
| Confidence calibration | Agent over/under-confidence | `decisions.jsonl` |
| Dead-end detection | Patterns that precede failures | `signals.jsonl` |

### Triggering Learning

```bash
# After session ends (or manually)
python .aria/lib/offline-learner.py learn

# View current policy
python .aria/lib/offline-learner.py export-policy

# Query for specific recommendation
python .aria/lib/offline-learner.py query feature 7 auth

# View statistics
python .aria/lib/offline-learner.py stats
```

### Using Learned Policy

The meta-reasoning skill automatically uses learned priors:

```bash
source .aria/lib/meta-reasoning.sh

# Get model recommendation (uses Thompson Sampling)
meta_select_model "feature" 6 "api"
# Output: sonnet|0.78|Learned from 15 past observations

# Full meta-reasoning cycle
meta_reason "Implement retry logic" "feature" 6
```

### Recording Outcomes

Outcomes are recorded automatically via hooks. Manual recording:

```bash
source .aria/lib/meta-reasoning.sh
meta_record_outcome "sonnet" "feature" 6 "success" "US-001"
```

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
| `/aria-start` | **Session init**: Launch dashboard + workflow router |
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

## Decision Tracing

For consequential decisions, emit a decision block to enable traceability and precedent lookup.

### When to Emit

Emit `<decision>` blocks when:
- Modifying files (architectural choices)
- Choosing between alternatives
- Deviating from existing patterns
- Skipping something (and why)

**Skip for:** Trivial reads, routine navigation, obvious single-path actions.

### Schema

```xml
<decision>
  <action>what you're doing</action>
  <context>what you looked at to decide</context>
  <rationale>why this approach</rationale>
  <alternatives>what else you considered</alternatives>
  <confidence>0.0-1.0</confidence>
</decision>
```

### Storage (REQUIRED for traceability)

After emitting a `<decision>` block in your response, **also call `emit_decision`** via Bash to persist it to `decisions.jsonl`:

```bash
source .aria/common.sh && emit_decision \
  "ACTION: what you're doing" \
  "CONTEXT: what you looked at" \
  "RATIONALE: why this approach" \
  "ALTERNATIVES: what else considered" \
  "CONFIDENCE: 0.0-1.0"
```

**Example:**
```bash
source .aria/common.sh && emit_decision \
  "Add retry wrapper to API client" \
  "Read utils/retry.ts, saw 3 similar uses" \
  "Consistency with existing patterns" \
  "Custom retry logic, no retry" \
  "0.85"
```

This ensures decisions are stored in `.aria/state/decisions.jsonl` for:
- Dashboard visualization
- Precedent queries
- Reconciliation with signals

### Mode Variations

| Mode | Decision Tracing |
|------|-----------------|
| LITE | Skip (speed over traceability) |
| STANDARD | Key decisions only |
| FULL/FULL+ | All consequential decisions |

### Storage & Visualization

Decisions are stored in `.aria/state/decisions.jsonl`. Signals (tool calls) are logged to `.aria/state/signals.jsonl` via hooks.

**View traces:**
```bash
python .aria/scripts/serve-dashboard.py  # Web dashboard at localhost:8420
.aria/scripts/trace-view.sh              # CLI: recent session
.aria/scripts/query-decisions.sh auth    # CLI: search by keyword
.aria/scripts/reconcile.sh               # CLI: verify claims match signals
```

---

## Full Lineage Tracking

For complete traceability, emit these additional blocks to build full workflow lineage.

### HITL Block

Emit when a human-in-the-loop checkpoint occurs:

```xml
<hitl>
  <checkpoint>what approval was requested</checkpoint>
  <response>approved|rejected|revised</response>
  <details>any modifications or notes</details>
</hitl>
```

**Example:**
```xml
<hitl>
  <checkpoint>Delete legacy auth module</checkpoint>
  <response>approved</response>
  <details>User confirmed migration complete</details>
</hitl>
```

### Task Block

Emit when starting or completing a task from the plan:

```xml
<task>
  <id>task number from plan</id>
  <title>task title</title>
  <status>started|completed|blocked</status>
  <notes>any relevant notes</notes>
</task>
```

**Example:**
```xml
<task>
  <id>3</id>
  <title>Add retry logic to API client</title>
  <status>completed</status>
  <notes>Used existing pattern from utils/retry.ts</notes>
</task>
```

### What Hooks Capture Automatically

The following are detected and tagged automatically via hooks (no emission needed):

| Context Type | Detected When | Tag |
|--------------|---------------|-----|
| skill | Read .aria/skills/*.md | skill:planning, skill:tdd, etc. |
| template | Read .aria/templates/*.md | template:skill-template |
| framework | Read CLAUDE.md | framework:CLAUDE.md |
| plan | Read/Write current-plan.json | plan:plan_update |
| progress | Read/Write progress.json | progress:task_update |
| verify | Bash npm test, pytest, etc. | verify:test_run |
| commit | Bash git commit | commit:git_commit |
| subagent | Task tool call | subagent:general-purpose |

### Complete Lineage Structure

```
SESSION: 2024-01-14 (STANDARD mode)
│
├── TASK 1: Add retry logic
│   ├── SKILL: planning loaded
│   ├── SKILL: tdd loaded
│   │   ├── DECISION: Use existing retry pattern (0.85)
│   │   │   ├── SIGNAL: Read utils/retry.ts
│   │   │   ├── SIGNAL: Read src/api/client.ts
│   │   │   └── SIGNAL: Edit src/api/client.ts
│   │   └── DECISION: Skip edge case tests (0.6)
│   ├── HITL: "Modify auth config?" → approved
│   ├── VERIFY: npm test → passed
│   └── COMMIT: abc123 "Add retry logic"
│
├── TASK 2: Fix auth bug
│   └── ...
```

---

*ARIA Hybrid - Fast execution with enforced verification*
