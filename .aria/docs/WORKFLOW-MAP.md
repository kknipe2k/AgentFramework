# ARIA Workflow Map

> Complete E2E reference showing all workflows, skills, files, and decision points

---

## Entry Point: /aria-start

```
/aria-start → Launch Dashboard → HITL Router → Workflow Selection
                   │                  │
           localhost:8420             │
                             ┌────────┼────────┐
                             ▼        ▼        ▼
                          [b]uild  [m]odify  [r]esearch
                             │        │        │
                             ▼        ▼        ▼
                          Router   Router   Research
                          (size)   (size)   Flow
```

### Size Router (for BUILD/MODIFY)

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
                         prototyping (SPEC only)
                               │
                         SPEC-*.json
                               │
                         executing.md
                               │
                    ┌──────────┴──────────┐
                    │                     │
              analyzer ─→ implementer ─→ verify-app
                                          │
                                     verify.sh
                                    (HTML/CSS/JS
                                     Playwright)
                                          │
                              prototype-*.html
```

---

### BUG FIX (Quick Fixes)

```
Bug Report → debugging → plan (lite) → execute → verify → done
```

---

### DEEP RESEARCH (Web Research)

```
Question ──→ deep-research ──→ [HITL: depth] ──→ [HITL: strategy]
                                    │                    │
                             Quick/Deep          Broad/Focused/
                                                 Comparative
                                    │                    │
                                    ▼                    ▼
                         ┌──────────────────────────────────┐
                         │         SEARCH LOOP              │
                         │                                  │
                         │  WebSearch → quality rating      │
                         │      ↓                           │
                         │  findings with confidence        │
                         │      ↓                           │
                         │  [HITL: checkpoint]              │
                         │  continue/redirect/synthesize    │
                         └──────────────────────────────────┘
                                    │
                                    ▼
                         [HITL: synthesis approach]
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
             Executive       Structured       Comparative
              Summary         Analysis          Matrix
                    │               │               │
                    └───────────────┴───────────────┘
                                    │
                                    ▼
                    research-output.json + IDEA.md
                                    │
                         [HITL: continue?]
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
                slides         prototype         done
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

## All Skills (14)

### Core Skills (All Modes)

| Skill | Trigger | Output |
|-------|---------|--------|
| **aria-start** | `/aria-start`, session init | Dashboard + workflow selection |
| **planning** | "plan this", start of work | `current-plan.json` |
| **executing** | Approved plan exists | Code, commits |
| **debugging** | Test failure, error | Working code |
| **discovery** | Modify flow, unfamiliar codebase | `project-context.md` |
| **tdd** | "write tests first", "tdd" | Tests → Implementation |

### Extended Skills (STANDARD+)

| Skill | Trigger | Output |
|-------|---------|--------|
| **brainstorming** | "brainstorm", "explore" | `IDEA.md` |
| **prototyping** | "mockup", "prototype" | `prototypes/SPEC-*.json` (spec only) |
| **tracking** | During/after execution | `progress.json` |
| **context-refresh** | 3+ failures, between phases | Handoff summary |
| **report-writer** | All tasks complete, "summary" | Summary + dashboard offer |

### Research Skills (Explicit)

| Skill | Trigger | Output |
|-------|---------|--------|
| **researcher** | "analyze this paper" | `research-output.json` |
| **deep-research** | "research X", "investigate" | `research-output.json`, `IDEA.md` |
| **slide-generation** | "generate slides" | `FOCUS.md`, slides |

### Meta Skills (STANDARD+)

| Skill | Trigger | Output |
|-------|---------|--------|
| **meta-reasoning** | Complex decisions, model selection | Recommendation, policy update |

---

## Skill Dependencies

```
aria-start ──→ HITL Router ──→ [BUILD|MODIFY|RESEARCH]
                                      │
                                      ▼
brainstorming ──→ prototyping ──→ planning ──→ executing
                      │                            │
                      │ (SPEC only)                │
                      │                            ├──→ debugging (on failure)
                      ▼                            │
                 executing ←─────────────────────────┘
                 (agents: analyzer → implementer → verify-app)
                      │
                      ▼
                 verify.sh ──→ tracking (parallel)
                                    │
                                    ▼
                              report-writer
```

**Unified Agent Pattern:**
All building tasks (code AND prototypes) use the same agent loop:
1. `analyzer` - Review spec/requirements
2. `implementer` - Build component
3. `verify-app` - Test functionality
4. `verify.sh` - Lint, test, accessibility

---

## Complete File Map

### State Files

| Path | Created By | Read By |
|------|------------|---------|
| `.aria/state/current-plan.json` | planning | executing, tracking, report-writer |
| `.aria/state/progress.json` | tracking | report-writer, dashboard |
| `.aria/state/decisions.jsonl` | executing | report-writer, dashboard |
| `.aria/state/signals.jsonl` | hooks | dashboard |

### Learning Files (NEW)

| Path | Created By | Read By |
|------|------------|---------|
| `.aria/learned/policy.json` | offline-learner | meta-reasoning |
| `.aria/learned/priors/*.json` | offline-learner | meta-reasoning |
| `.aria/learned/history/episodes.jsonl` | offline-learner | reporting |
| `.aria/logs/model_learning.json` | model-selector | offline-learner |

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
| `.aria/prototypes/SPEC-*.json` | prototyping | Prototype specifications |
| `.aria/prototypes/*.html` | executing (from spec) | Built prototypes |
| `.aria/prototypes/tests/*.spec.js` | executing | Playwright tests |
| `.aria/reports/SESSION-*.md` | report-writer | Saved reports |

### Scripts

| Path | Purpose | Invoked |
|------|---------|---------|
| `.aria/scripts/serve-dashboard.py` | Lineage dashboard | End of workflow |
| `.aria/scripts/generate-slides.py` | Create presentations | Research flow |
| `.aria/scripts/setup-project.sh` | Create workspace | Manual |
| `.aria/verify.sh` | Verification gate | After every task |

### Library Files (NEW)

| Path | Purpose | Invoked |
|------|---------|---------|
| `.aria/lib/meta-reasoning.sh` | Thompson Sampling, model selection | Complex decisions |
| `.aria/lib/offline-learner.py` | Offline RL learning pipeline | Post-session |

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
| Deep research depth | After trigger | [1] Quick / [2] Standard / [3] Deep / [4] Exhaustive |
| Deep research strategy | After depth | [a] Broad / [b] Focused / [c] Comparative / [d] Temporal |
| Deep research checkpoint | Mid-research | [c]ontinue / [r]edirect / [d]eepen / [s]ynthesize / [a]bort |

---

## Quick Commands

| Command | Action |
|---------|--------|
| `/aria-start` | **Session init**: Dashboard + workflow router |
| `/aria:plan` | Start planning |
| `/aria:status` | Show progress |
| `/aria:verify` | Run verification |
| `/aria-summary` | Generate summary |
| `/aria-dashboard` | Launch dashboard |

---

## Offline Learning Pipeline

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

**Run learning:**
```bash
python .aria/lib/offline-learner.py learn
python .aria/lib/offline-learner.py stats
```

---

*Interactive version: `.aria/docs/WORKFLOW-MAP.html`*
*Last updated: 2026-01-18*
