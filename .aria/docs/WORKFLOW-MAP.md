# ARIA Workflow Map

> Complete E2E reference showing all workflows, skills, files, and decision points

---

## Entry Point: Router

```
User Request → Router (CLAUDE.md) → Size Assessment → Mode Selection
                                          │
              ┌───────────────────────────┼───────────────────────────┐
              │                           │                           │
         SMALL (1-5)               MEDIUM (6-15)               LARGE (16-40)
              │                           │                           │
           LITE                      STANDARD                       FULL
```

---

## Workflows

### BUILD (Greenfield)

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ LITE:     plan? ────────────────────────────→ execute → verify → done          │
├─────────────────────────────────────────────────────────────────────────────────┤
│ STANDARD: brainstorm? → plan → [approve] → execute → verify → report → dash?   │
├─────────────────────────────────────────────────────────────────────────────────┤
│ FULL:     brainstorm → prototype? → plan → [approve] → execute → verify        │
│           → report → dashboard                                                   │
├─────────────────────────────────────────────────────────────────────────────────┤
│ FULL+:    design doc → brainstorm → prototype → plan → [approve] → execute     │
│           → verify → report → dashboard                                          │
└─────────────────────────────────────────────────────────────────────────────────┘
```

**Files Created:**
| File | Skill | Purpose |
|------|-------|---------|
| `.aria/docs/IDEA.md` | brainstorming | Options, recommendation |
| `.aria/prototypes/*.html` | prototyping | Visual mockups |
| `.aria/state/current-plan.json` | planning | Task breakdown |
| `.aria/state/progress.json` | tracking | Completion status |
| `.aria/state/decisions.jsonl` | executing | Decision trace |

---

### MODIFY (Existing Codebase)

```
discovery → project-context.md → plan → [approve] → execute → verify → report → dash?
```

**Key Difference:** Starts with discovery skill to understand existing codebase.

---

### RESEARCH (Paper/Article Analysis)

```
Article ──→ researcher ──→ research-output.json
                               │
                               ▼
                         brainstorming ──→ IDEA.md
                               │
                     ┌─────────┴─────────┐
                     │                   │
              HITL: slides?              │
                     │                   │
              ┌──────┴──────┐            │
              ▼             ▼            │
        slide-generation    skip         │
              │                          │
         FOCUS.md                        │
              │                          │
        ┌─────┴─────┐                    │
        ▼           ▼                    │
   NotebookLM    pptx                    │
        │           │                    │
   slides.pdf  slides.pptx               │
                    │                    │
                    └────────────────────┤
                                         │
                               ┌─────────┴─────────┐
                               │                   │
                        HITL: prototype?      HITL: done
                               │                   │
                    ┌──────────┼──────────┐        │
                    ▼          ▼          ▼        ▼
               [1] mockup  [2] learning  [3] ref  report
                    │          │          │
                    └──────────┴──────────┘
                               │
                         prototyping
                               │
                    prototype-*.html
```

---

### BUG FIX (Quick Fixes)

```
Bug Report → debugging → plan (lite) → execute → verify → done
```

---

### END OF WORKFLOW (STANDARD+)

```
All tasks complete
        │
        ▼
  report-writer ──→ Summary Report
        │
        ▼
   HITL: View dashboard?
        │
   ┌────┼────┬────────┐
   ▼    ▼    ▼        ▼
  [y]  [n]  [s]      done
   │         │
   │    save to .aria/reports/
   │
   ▼
serve-dashboard.py
   │
   ▼
localhost:8420
   │
   ▼
Hierarchical Drill-down:
SESSION → SKILL → DECISION → SIGNALS → COMMIT
```

---

## All Skills (12)

### Core Skills (All Modes)

| Skill | Trigger | Output |
|-------|---------|--------|
| **planning** | "plan this", start of work | `current-plan.json` |
| **executing** | Approved plan exists | Code, commits |
| **debugging** | Test failure, error | Working code |
| **discovery** | Modify flow, unfamiliar codebase | `project-context.md` |
| **tdd** | "write tests first", "tdd" | Tests → Implementation |

### Extended Skills (STANDARD+)

| Skill | Trigger | Output |
|-------|---------|--------|
| **brainstorming** | "brainstorm", "explore" | `IDEA.md` |
| **prototyping** | "mockup", "prototype" | `prototypes/*.html` |
| **tracking** | During/after execution | `progress.json` |
| **context-refresh** | 3+ failures, between phases | Handoff summary |
| **report-writer** | All tasks complete, "summary" | Summary + dashboard offer |

### Research Skills (Explicit)

| Skill | Trigger | Output |
|-------|---------|--------|
| **researcher** | "analyze this paper" | `research-output.json` |
| **slide-generation** | "generate slides" | `FOCUS.md`, slides |

---

## Skill Dependencies

```
brainstorming ──→ prototyping ──→ planning ──→ executing
                                      │            │
                                      │            ├──→ debugging (on failure)
                                      │            │
                                      └────────────┴──→ tracking (parallel)
                                                             │
                                                             ▼
                                                       report-writer
```

---

## Complete File Map

### State Files

| Path | Created By | Read By |
|------|------------|---------|
| `.aria/state/current-plan.json` | planning | executing, tracking, report-writer |
| `.aria/state/progress.json` | tracking | report-writer, dashboard |
| `.aria/state/decisions.jsonl` | executing | report-writer, dashboard |
| `.aria/state/signals.jsonl` | hooks | dashboard |

### Documentation Files

| Path | Created By | Purpose |
|------|------------|---------|
| `.aria/docs/IDEA.md` | brainstorming | Research synthesis |
| `.aria/docs/research-output.json` | researcher | Extracted concepts |
| `.aria/docs/DESIGN.md` | FULL+ planning | Architecture document |
| `.aria/design-notes.md` | executing | Implementation reasoning |
| `.aria/project-context.md` | discovery | Codebase knowledge |

### Output Files

| Path | Created By | Purpose |
|------|------------|---------|
| `.aria/outputs/FOCUS.md` | slide-generation | Core ideas + synthesis |
| `.aria/outputs/slides-*.pdf` | slide-generation (NBLM) | Presentation |
| `.aria/outputs/slides-*.pptx` | slide-generation (pptx) | Presentation |
| `.aria/prototypes/*.html` | prototyping | Visual mockups |
| `.aria/reports/SESSION-*.md` | report-writer | Saved reports |

### Scripts

| Path | Purpose | Invoked |
|------|---------|---------|
| `.aria/scripts/serve-dashboard.py` | Lineage dashboard | End of workflow |
| `.aria/scripts/generate-slides.py` | Create presentations | Research flow |
| `.aria/scripts/setup-project.sh` | Create workspace | Manual |
| `.aria/verify.sh` | Verification gate | After every task |

---

## Mode Matrix

| Feature | LITE | STANDARD | FULL | FULL+ |
|---------|:----:|:--------:|:----:|:-----:|
| Task count | 1-5 | 6-15 | 16-40 | 40+ |
| Planning | Optional | Yes | Yes + risks | Yes + design doc |
| Brainstorming | Optional | Yes | Yes | Yes |
| Prototyping | - | Optional | Yes | Mandatory |
| Verification | If exists | Every task | Mandatory | Mandatory |
| HITL | Destructive | Risky | All risky | Per epic |
| Design notes | No | Key decisions | All | All + architecture |
| Failure threshold | 1 | 2 | 3 | 3 |
| Context refresh | No | Between phases | Between steps | Between epics |
| End report | Minimal | Yes + dash? | Full + dash | Full + validation |

---

## HITL Checkpoint Summary

| Checkpoint | When | Options |
|------------|------|---------|
| Plan approval | After planning | [a]pprove / [r]evise / [c]ancel |
| Task HITL | Before risky task | [y]es / [n]o / [s]kip |
| Slides decision | After IDEA.md | [y]es / [n]o |
| Slide method | If slides=yes | [1] NotebookLM / [2] pptx |
| Prototype decision | After slides | [p]rototype / [d]one |
| Prototype variant | If prototype=yes | [1] mockup / [2] learning / [3] reference |
| Dashboard offer | End of workflow | [y]es / [n]o / [s]ave |
| Failure escalation | 3 failures | [r]etry / [f]resh / [s]kip / [a]bort |

---

## Quick Commands

| Command | Action |
|---------|--------|
| `/aria:plan` | Start planning |
| `/aria:status` | Show progress |
| `/aria:verify` | Run verification |
| `/aria-summary` | Generate summary |
| `/aria-dashboard` | Launch dashboard |

---

*Interactive version: `.aria/docs/WORKFLOW-MAP.html`*
