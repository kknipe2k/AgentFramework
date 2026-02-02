# ARIA: Agentic Rail-based Intent Architecture

> **The missing orchestration layer for autonomous AI development**

---

## The Problem

AI coding assistants are powerful but unpredictable. Without guardrails:

- Changes ship without verification → bugs in production
- No stopping point for risky actions → unintended deletions, config breaks
- No record of decisions → "why did it do that?"
- Same ceremony for trivial fixes and major features → wasted time

**You're either micromanaging every step or hoping for the best.**

ARIA fixes this.

---

## What ARIA Does

ARIA sits between you and your AI assistant. It adds:

| Feature | What It Does |
|---------|--------------|
| **Router** | Analyzes task complexity, selects appropriate process level |
| **Verification Gates** | Mandatory checks after every code change |
| **HITL Checkpoints** | Stops for approval before risky actions |
| **Decision Traces** | Logs every decision with rationale and confidence |
| **Mode Scaling** | LITE for quick fixes, FULL for complex features |

**Think of it as "rails, not rules."** The AI stays productive while respecting boundaries.

---

## How It Works

### 1. The Router

When you give ARIA a task, it first sizes the work:

```
You: "Add retry logic to the API client"

ARIA:
SIZE: SMALL
MODE: LITE
Reason: 1 task, ~50 lines, no risky areas
```

**Sizing criteria:**

| Factor | SMALL | MEDIUM | LARGE | X-LARGE |
|--------|-------|--------|-------|---------|
| Tasks | 1-5 | 6-15 | 16-40 | 40+ |
| Lines of code | <2,000 | 2,000-10,000 | 10,000-50,000 | 50,000+ |
| Files | 1-5 | 6-20 | 21-50 | 50+ |
| Risky areas | No | Read-only | Yes (one) | Yes (multiple) |

**Size maps to mode:**

- **SMALL → LITE**: Quick execution, minimal ceremony
- **MEDIUM → STANDARD**: Planning phase, verification gates
- **LARGE → FULL**: Risk assessment, detailed planning, design notes
- **X-LARGE → FULL+**: Design doc, architecture review, epic-level tracking

### 2. Planning

For STANDARD mode and above, ARIA creates a plan:

```json
{
  "id": "plan-20240114-103000",
  "title": "Add retry logic to API client",
  "status": "pending_approval",
  "tasks": [
    {
      "id": 1,
      "title": "Add exponential backoff retry wrapper",
      "description": "Create retry utility matching existing patterns",
      "status": "pending",
      "hitl": false,
      "estimated_minutes": 15
    }
  ],
  "risks": ["May affect request timing in tests"]
}
```

**You approve before execution:**

```
ARIA: Plan ready. 1 task, ~15 minutes.
[a]pprove / [r]evise / [c]ancel
```

### 3. Execution with Verification

Tasks execute one at a time. After each task:

```bash
$ bash .aria/verify.sh

✓ No secrets detected
✓ Tests passing (47/47)
✓ Lint clean
✓ TypeScript OK
✓ Build succeeds

VERIFICATION PASSED
```

**If verification fails, execution STOPS.** No proceeding past failures. No "I'll fix it later."

### 4. HITL Checkpoints

Before risky actions, ARIA stops and asks:

```
HITL CHECKPOINT: About to delete src/legacy/auth-v1.ts

This file contains 247 lines of authentication code.
It appears to be replaced by src/auth/index.ts.

Proceed? [y]es / [n]o / [e]xplain
```

**HITL triggers on:**
- File deletions
- Config file changes (package.json, tsconfig, etc.)
- "Don't touch" areas defined in project-context.md
- New dependency installations
- Security-sensitive code (auth, payments)

### 5. Decision Traces

Every significant decision is logged:

```xml
<decision>
  <action>Use existing retry pattern from utils/retry.ts</action>
  <context>Read src/utils/retry.ts, found 3 similar implementations</context>
  <rationale>Consistency with existing codebase patterns</rationale>
  <alternatives>Custom retry implementation, No retry</alternatives>
  <confidence>0.85</confidence>
</decision>
```

**Why this matters:**
- **Debugging**: Understand why something was done when it breaks
- **Learning**: Improve AI decisions over time
- **Compliance**: Audit trail for regulated industries
- **Handoff**: New team members understand the reasoning

---

## The Four Modes

### LITE Mode

**For:** Quick fixes, simple tasks, 1-5 steps

```
LITE MODE ACTIVE

Features ON:
✓ Quick planning (optional)
✓ Basic verification (if tests exist)
✓ Git commit on completion

Features OFF:
✗ Formal planning phase
✗ HITL checkpoints (except destructive actions)
✗ Design notes
✗ Progress announcements
```

**Example:** "Fix the typo in the error message"

### STANDARD Mode

**For:** Medium tasks, 6-15 steps, some complexity

```
STANDARD MODE ACTIVE

Features ON:
✓ Planning phase with approval
✓ Verification after each task
✓ HITL for risky actions
✓ Git commit per task
✓ Progress tracking

Features OFF:
✗ Design notes (only key decisions)
✗ Full failure escalation
```

**Example:** "Add user profile editing"

### FULL Mode

**For:** Complex features, 16-40 tasks, production code

```
FULL MODE ACTIVE

Features ON:
✓ Risk assessment
✓ Detailed planning with estimates
✓ Mandatory verification
✓ All HITL checkpoints
✓ Design notes for all reasoning
✓ Failure escalation (3 failures → options)
✓ Context refresh prompts
```

**Example:** "Implement OAuth2 authentication"

### FULL+ Mode

**For:** Enterprise projects, 40+ tasks, multiple systems

```
FULL+ MODE ACTIVE

Features ON:
✓ Everything in FULL, plus:
✓ Mandatory design doc
✓ Architecture review checkpoint
✓ HITL gates per epic
✓ Context refresh between epics
✓ Epic-level progress tracking
```

**Example:** "Build multi-tenant SaaS platform"

---

## Use Cases

### 1. Build (Greenfield)

New projects from scratch:

```
Router → Brainstorm → Plan → Execute → Report
```

### 2. Modify (Existing Codebase)

Changes to mature code:

```
Router → Plan → Execute → Report
```

ARIA reads `project-context.md` for "don't touch" areas.

### 3. Research

Analyze papers/articles:

```
Extract concepts → IDEA.md → Slides (optional) → Prototype (optional)
```

### 4. Deep Research

Web research on any topic:

```
Depth selection → Query strategy → Search loop → Synthesis
```

With HITL gates, source tracking, and confidence scoring.

---

## The Verification Gate

This is non-negotiable. After EVERY code change:

```bash
bash .aria/verify.sh
```

**What it checks:**

1. **Secrets detection** - No API keys, passwords in code
2. **Tests** - All tests passing
3. **Linting** - Code style enforced
4. **Type checking** - TypeScript compilation
5. **Build** - Production build succeeds

**If it fails, you stop.** Period.

---

## Failure Escalation

When things go wrong repeatedly:

```
ESCALATION: 3 consecutive failures on [issue]

Options:
[r]etry with different approach
[f]resh session (start new context)
[s]kip this task
[a]bort execution

What would you like to do?
```

**Why not auto-retry forever?**
- Repeated failures may indicate context drift
- Human judgment needed for strategy change
- Prevents infinite loops of the same mistake

---

## What Gets Logged

| File | Purpose |
|------|---------|
| `current-plan.json` | Active plan with tasks |
| `progress.json` | Completion status |
| `decisions.jsonl` | Decision trace log |
| `signals.jsonl` | Tool calls (auto-captured) |
| `design-notes.md` | AI reasoning (FULL mode) |

---

## Quick Start

### 1. Setup

```bash
git clone https://github.com/your-org/aria
cd your-project
bash path/to/aria/.aria/scripts/setup-project.sh
```

### 2. Use with Claude Code

ARIA loads automatically via CLAUDE.md. Just start:

```
"Add a retry mechanism to the API client"
```

ARIA handles:
- Sizing the task
- Creating a plan
- Getting your approval
- Executing with verification
- Committing changes

### 3. External Mode (Terminal)

```bash
bash .aria/ralph/ralph.sh "Add retry logic"
```

RALPH runs autonomously with the same safety rails.

---

## Key Concepts Recap

| Concept | One-liner |
|---------|-----------|
| **Router** | Right-sizes process to task complexity |
| **Modes** | LITE/STANDARD/FULL/FULL+ scale ceremony appropriately |
| **HITL** | Human approval before risky actions |
| **Verification** | Mandatory checks after every change |
| **Traces** | Full decision log with rationale |
| **Escalation** | Human decides strategy after repeated failures |

---

## The Philosophy

> **Rails, not rules.**

ARIA doesn't prevent the AI from working. It ensures:
1. Verification happens
2. Risky actions get approval
3. Decisions are traceable
4. Process scales to complexity

The goal is **productive autonomy with appropriate oversight.**

---

## Try the Interactive Demo

Open `aria-demo.html` in your browser for an interactive walkthrough with:
- Live router demo (adjust sliders to see mode changes)
- Clickable workflow steps
- HITL checkpoint simulation
- Example decision traces

---

## Links

- [Interactive Demo](aria-demo.html)
- [Full Documentation](../CLAUDE.md)
- [Skills Reference](../skills/REGISTRY.md)
- [Workflow Map](../docs/WORKFLOW-MAP.md)

---

*ARIA: Because autonomous doesn't mean unsupervised.*
