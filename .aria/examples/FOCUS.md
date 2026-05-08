# ARIA: Core Concepts for Presentation

> **Upload this file to NotebookLM for slide generation**

---

## Executive Summary (1 slide)

**ARIA** = Agentic Rail-based Intent Architecture

An orchestration layer for autonomous AI development that adds:
- **Verification gates** after every code change
- **Human-in-the-loop** checkpoints before risky actions
- **Decision tracing** for full auditability
- **Mode scaling** to match process to complexity

**One-liner:** "Rails, not rules" - productive autonomy with appropriate oversight.

---

## The Problem (1-2 slides)

### Without Guardrails

AI coding assistants are powerful but unpredictable:

| Issue | Consequence |
|-------|-------------|
| No verification | Bugs ship to production |
| No stopping points | Unintended deletions, config breaks |
| No decision records | "Why did it do that?" |
| Same process for everything | Wasted time on trivial tasks |

### The False Choice

Current options:
1. **Micromanage** - Review every step (defeats the purpose)
2. **YOLO** - Hope for the best (risky)

**ARIA provides a third option:** Structured autonomy with strategic checkpoints.

---

## How ARIA Works (4-5 slides)

### Slide 1: The Router

ARIA analyzes every task first:

```
Input: "Add retry logic to API client"

Output:
SIZE: SMALL
MODE: LITE
Reason: 1 task, ~50 lines, no risky areas
```

**Sizing factors:** Tasks, lines of code, files, risky areas (auth/payments/DB)

### Slide 2: Mode Selection

| Size | Mode | Process Level |
|------|------|---------------|
| SMALL | LITE | Quick execution, minimal ceremony |
| MEDIUM | STANDARD | Planning + verification gates |
| LARGE | FULL | Risk assessment + design notes |
| X-LARGE | FULL+ | Design doc + architecture review |

**Key insight:** A typo fix shouldn't need the same ceremony as a new auth system.

### Slide 3: Verification Gates

After EVERY code change:

```bash
bash .aria/verify.sh

✓ No secrets detected
✓ Tests passing
✓ Lint clean
✓ TypeScript OK
✓ Build succeeds
```

**Rule:** If verification fails, execution STOPS. No exceptions.

### Slide 4: HITL Checkpoints

Before risky actions, ARIA asks:

```
HITL CHECKPOINT: About to delete src/legacy/auth.ts

Proceed? [y]es / [n]o / [e]xplain
```

**Triggers:**
- File deletions
- Config changes
- "Don't touch" areas
- New dependencies
- Security-sensitive code

### Slide 5: Decision Traces

Every significant decision logged:

```
Action: Use existing retry pattern
Context: Found 3 similar implementations
Rationale: Consistency with codebase
Alternatives: Custom impl, no retry
Confidence: 0.85
```

**Why it matters:** Debugging, learning, compliance, handoff.

---

## Key Differentiators (1-2 slides)

### vs. "Just Use AI"

| Aspect | Raw AI | ARIA |
|--------|--------|------|
| Verification | Hope | Mandatory |
| Risky actions | Proceeds | Asks first |
| Decisions | Black box | Fully traced |
| Process | One-size | Scaled to task |

### vs. Traditional Dev Process

| Aspect | Traditional | ARIA |
|--------|-------------|------|
| Speed | Slow (manual review) | Fast (automated gates) |
| Consistency | Varies | Enforced |
| Documentation | Often skipped | Auto-generated |
| Oversight | All or nothing | Strategic checkpoints |

---

## Use Cases (1 slide)

| Use Case | Flow | Example |
|----------|------|---------|
| **Build** | Router → Plan → Execute → Report | New CLI tool |
| **Modify** | Router → Plan → Execute → Report | Add feature to existing app |
| **Research** | Extract → Synthesize → Slides/Prototype | Analyze paper |
| **Deep Research** | Query → Search → Synthesize | Market analysis |

---

## Architecture Overview (1 slide)

```
┌─────────────────────────────────────────────────┐
│                    ARIA                          │
├─────────────────────────────────────────────────┤
│  Router    │  Planner   │  Executor  │  Verify  │
├─────────────────────────────────────────────────┤
│  HITL      │  Traces    │  Signals   │  Rails   │
├─────────────────────────────────────────────────┤
│              AI Assistant (Claude)               │
└─────────────────────────────────────────────────┘
```

**Components:**
- **Router**: Sizes tasks, selects mode
- **Planner**: Creates structured plans
- **Executor**: Runs tasks with verification
- **HITL**: Human checkpoints
- **Traces**: Decision logging
- **Rails**: Safety guardrails

---

## The Verification Gate (1 slide)

```
┌─────────────┐
│   Execute   │
│    Task     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   verify.sh │──────► FAIL → STOP
└──────┬──────┘
       │ PASS
       ▼
┌─────────────┐
│   Commit    │
└──────┬──────┘
       │
       ▼
   Next Task
```

**Non-negotiable:** No proceeding past failed verification.

---

## Synthesis Matrix

| Concept | Problem Solved | Mechanism | Benefit |
|---------|---------------|-----------|---------|
| **Router** | Wrong process for task size | Analyze complexity → select mode | Right-sized ceremony |
| **Modes** | One-size-fits-all | LITE/STANDARD/FULL/FULL+ | Efficiency + safety |
| **HITL** | Risky actions proceed silently | Checkpoint before destructive ops | Human control |
| **Verification** | Bugs ship without checks | Mandatory verify.sh after changes | Quality gates |
| **Traces** | Black-box decisions | Log action/context/rationale | Auditability |
| **Escalation** | Infinite retry loops | 3 failures → human decision | Fail gracefully |

---

## Memorable Takeaways

1. **"Rails, not rules"** - Guide without blocking
2. **"Right-sized process"** - Match ceremony to complexity
3. **"Verification is non-negotiable"** - No proceeding past failures
4. **"Strategic checkpoints"** - Human control where it matters
5. **"Traceable decisions"** - Know why, not just what

---

## Call to Action (1 slide)

**Try ARIA:**

1. Clone the repo
2. Run setup script
3. Start with a simple task
4. Watch the rails in action

**Links:**
- Interactive demo: `aria-demo.html`
- Full docs: `CLAUDE.md`
- GitHub: [repo-url]

---

## Slide Outline for NotebookLM

1. Title: ARIA - Agentic Rail-based Intent Architecture
2. The Problem: Unpredictable AI without guardrails
3. The Solution: Rails, not rules
4. How It Works: Router
5. How It Works: Mode Selection
6. How It Works: Verification Gates
7. How It Works: HITL Checkpoints
8. How It Works: Decision Traces
9. Key Differentiators
10. Use Cases
11. Architecture
12. Takeaways
13. Try It / Links

---

*Optimized for NotebookLM slide generation. Upload this document and request presentation slides.*
