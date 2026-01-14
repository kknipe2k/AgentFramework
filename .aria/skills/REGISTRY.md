# Skill Registry

> Central index of all ARIA skills with invocation patterns

---

## Quick Reference

| Skill | Trigger | Modes | Category |
|-------|---------|-------|----------|
| [planning](./planning.md) | Start of work, "plan this" | ALL | Core |
| [executing](./executing.md) | Approved plan exists | ALL | Core |
| [debugging](./debugging.md) | Test failure, error | ALL | Core |
| [tracking](./tracking.md) | During/after execution | STANDARD+ | Core |
| [brainstorming](./brainstorming.md) | "explore", new project | STANDARD+ | Creative |
| [prototyping](./prototyping.md) | "mockup", visual needed | STANDARD+ | Creative |
| [researcher](./researcher.md) | Article/paper input | Explicit | Research |
| [report-writer](./report-writer.md) | Project complete | STANDARD+ | Research |

---

## Skill Categories

### Core (Always Available)
Essential skills used in every workflow.

| Skill | Purpose | Inputs | Outputs |
|-------|---------|--------|---------|
| **planning** | Break work into tasks | Requirements | `current-plan.json` |
| **executing** | Implement tasks | Approved plan | Code, commits |
| **debugging** | Fix failures | Error/test failure | Working code |
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
| **report-writer** | Generate reports | Completed work | `REPORT.md` |

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
| Project complete | report-writer | Summary needed |

### Explicit (User Request)
These require explicit user request:

| Request | Skill |
|---------|-------|
| "analyze this paper" | researcher |
| "create a report" | report-writer |
| "debug this" | debugging |

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
| tracking | optional | ✓ | ✓ | ✓ |
| brainstorming | optional | ✓ | ✓ | ✓ |
| prototyping | - | optional | ✓ | ✓ |
| researcher | explicit | explicit | explicit | explicit |
| report-writer | - | ✓ | ✓ | ✓ |

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

*Last updated: 2026-01-14*
