# Skill Registry

> Central index of all ARIA skills with invocation patterns

---

## Quick Reference

| Skill | Trigger | Modes | Category |
|-------|---------|-------|----------|
| [planning](./planning.md) | Start of work, "plan this" | ALL | Core |
| [executing](./executing.md) | Approved plan exists | ALL | Core |
| [debugging](./debugging.md) | Test failure, error | ALL | Core |
| [discovery](./discovery.md) | New codebase, Modify flow | ALL | Core |
| [tdd](./tdd.md) | "write tests first", "tdd" | ALL | Core |
| [context-refresh](./context-refresh.md) | Between phases, 3+ failures | STANDARD+ | Core |
| [tracking](./tracking.md) | During/after execution | STANDARD+ | Core |
| [brainstorming](./brainstorming.md) | "explore", new project | STANDARD+ | Creative |
| [prototyping](./prototyping.md) | "mockup", visual needed | STANDARD+ | Creative |
| [researcher](./researcher.md) | Article/paper input | Explicit | Research |
| [report-writer](./report-writer.md) | Project complete | STANDARD+ | Research |
| [slide-generation](./slide-generation.md) | After IDEA.md, "slides" | Explicit | Research |

---

## Skill Categories

### Core (Always Available)
Essential skills used in every workflow.

| Skill | Purpose | Inputs | Outputs |
|-------|---------|--------|---------|
| **planning** | Break work into tasks | Requirements | `current-plan.json` |
| **executing** | Implement tasks | Approved plan | Code, commits |
| **debugging** | Fix failures | Error/test failure | Working code |
| **discovery** | Explore codebase | Codebase access | `project-context.md` |
| **tdd** | Test-driven development | Requirements | Tests, implementation |
| **context-refresh** | Reset long sessions | Progress state | Handoff summary |
| **tracking** | Monitor progress | Execution events | `progress.json`, metrics |

### Creative (STANDARD+)
Ideation and visualization skills.

| Skill | Purpose | Inputs | Outputs |
|-------|---------|--------|---------|
| **brainstorming** | Explore options | Problem statement | `IDEA.md` |
| **prototyping** | Visual mockups | Concept | HTML in `.aria/prototypes/` |

### Research (Explicit Invocation)
Article-to-code and documentation skills.

| Skill | Purpose | Inputs | Outputs |
|-------|---------|--------|---------|
| **researcher** | Extract concepts | Article/paper | Concept JSON |
| **report-writer** | End-of-workflow summary | State files | Summary + dashboard offer |
| **slide-generation** | Create presentations | IDEA.md, sources | FOCUS.md, slides |

---

## Invocation Patterns

### Automatic (Mode-Based)
These skills activate based on the current mode:

```
LITE:       planning(lite) → executing
STANDARD:   brainstorming? → planning → executing → tracking → report
FULL:       brainstorming → prototyping? → planning → executing → tracking → report
FULL+:      brainstorming → prototyping → planning → executing → tracking → report
```

### On-Demand (Trigger-Based)
These skills activate when conditions are met:

| Trigger | Skill | Example |
|---------|-------|---------|
| Test failure | debugging | `npm test` fails |
| "explore options" | brainstorming | User wants alternatives |
| "show me a mockup" | prototyping | Visual needed |
| Article/URL input | researcher | Research flow |
| All tasks complete | report-writer | Auto-triggers, offers dashboard |
| "summary", "metrics" | report-writer | Manual request |
| New/unfamiliar codebase | discovery | Modify flow entry |
| "write tests first" | tdd | Test-driven approach |
| 3+ consecutive failures | context-refresh | Context drift recovery |
| Between phases/epics | context-refresh | FULL/FULL+ mode |

### Explicit (User Request)
These require explicit user request:

| Request | Skill |
|---------|-------|
| "analyze this paper" | researcher |
| "show summary", "metrics" | report-writer |
| "debug this" | debugging |
| "generate slides" | slide-generation |
| "create presentation" | slide-generation |
| `/aria-summary` | report-writer |
| `/aria-dashboard` | dashboard (lineage view) |

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

### Dependency Rules

1. **planning** can run standalone or after brainstorming
2. **executing** requires approved plan from planning
3. **debugging** invoked by executing on failure
4. **tracking** runs parallel to executing (not blocking)
5. **report-writer** requires tracking data

---

## Mode Compatibility

| Skill | LITE | STANDARD | FULL | FULL+ |
|-------|:----:|:--------:|:----:|:-----:|
| planning | ✓ (lite) | ✓ | ✓ | ✓ |
| executing | ✓ | ✓ | ✓ | ✓ |
| debugging | ✓ | ✓ | ✓ | ✓ |
| discovery | ✓ | ✓ | ✓ | ✓ |
| tdd | ✓ (lite) | ✓ | ✓ (full) | ✓ (full) |
| context-refresh | - | ✓ | ✓ | ✓ |
| tracking | optional | ✓ | ✓ | ✓ |
| brainstorming | optional | ✓ | ✓ | ✓ |
| prototyping | - | optional | ✓ | ✓ |
| researcher | explicit | explicit | explicit | explicit |
| report-writer | - | ✓ | ✓ | ✓ |
| slide-generation | explicit | explicit | explicit | explicit |

---

## Adding New Skills

New skills should follow the [skill template](../templates/skill-template.md):

1. Create file in appropriate category folder
2. Use standard header format
3. Register in this file
4. Add to COMPOSITION.md if it chains with others

### Standard Header Format

```markdown
# [Skill Name]

> [One-line purpose]

---
version: 1.0.0
modes: [LITE, STANDARD, FULL, FULL+] or subset
triggers: [what activates this skill]
inputs: [required context]
outputs: [files, state changes]
dependencies: [other skills required]
---

## When to Use
...
```

---

*Last updated: 2026-01-15*
