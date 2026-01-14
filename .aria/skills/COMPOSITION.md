# Skill Composition

> How skills chain together for different workflows

---

## Common Workflows

### 1. New Project (Build)

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ brainstorm  │ ──▶ │  prototype  │ ──▶ │   planning  │ ──▶ │  executing  │
│             │     │  (optional) │     │             │     │             │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
                                                                   │
                                              ┌────────────────────┤
                                              │                    │
                                              ▼                    ▼
                                        ┌───────────┐      ┌─────────────┐
                                        │ debugging │      │  tracking   │
                                        │(on fail)  │      │ (parallel)  │
                                        └───────────┘      └─────────────┘
                                                                   │
                                                                   ▼
                                                           ┌─────────────┐
                                                           │report-writer│
                                                           └─────────────┘
```

**Mode variations:**
- LITE: planning → executing (skip brainstorm, prototype, report)
- STANDARD: brainstorm? → planning → executing → tracking → report
- FULL/FULL+: All steps

---

### 2. Bug Fix (Modify)

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  debugging  │ ──▶ │  planning   │ ──▶ │  executing  │ ──▶ │   verify    │
│ (diagnose)  │     │   (lite)    │     │   (fix)     │     │             │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

**Flow:**
1. debugging: Reproduce, isolate, hypothesize
2. planning: 1-3 task fix plan
3. executing: Implement fix
4. verify: Confirm fix works

---

### 3. Research-to-Code

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ researcher  │ ──▶ │ brainstorm  │ ──▶ │  prototype  │
│             │     │             │     │ (optional)  │
└─────────────┘     └─────────────┘     └─────────────┘
                                              │
                           ┌──────────────────┘
                           ▼
                    ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
                    │  planning   │ ──▶ │  executing  │ ──▶ │report-writer│
                    │             │     │             │     │             │
                    └─────────────┘     └─────────────┘     └─────────────┘
```

**Outputs at each stage:**
- researcher → `concepts.json`
- brainstorm → `IDEA.md`
- prototype → `.aria/prototypes/*.html`
- planning → `current-plan.json`
- executing → Code + commits
- report-writer → `REPORT.md`

---

### 4. Feature Addition (Modify)

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  discovery  │ ──▶ │  planning   │ ──▶ │  executing  │ ──▶ │  tracking   │
│ (if new)    │     │             │     │             │     │             │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

**Flow:**
1. discovery: Understand codebase (skip if already familiar)
2. planning: Break down feature
3. executing: Implement with verification
4. tracking: Log progress

---

### 5. Codebase Exploration

```
┌─────────────┐     ┌─────────────┐
│  discovery  │ ──▶ │  document   │
│             │     │  (output)   │
└─────────────┘     └─────────────┘
        │
        ▼
  project-context.md
```

**When:** User says "what does this do", "explore this codebase"

---

## Parallel Execution

Some skills run alongside others, not sequentially:

| Primary Skill | Parallel Skill | Relationship |
|---------------|----------------|--------------|
| executing | tracking | tracking observes executing |
| executing | debugging | debugging invoked on failure |
| executing | context-refresh | refresh triggered between phases |

### Tracking Integration

```
executing: Task 1 ──▶ Task 2 ──▶ Task 3 ──▶ Task 4
              │          │          │          │
              ▼          ▼          ▼          ▼
tracking:   log        log        log        log
              │          │          │          │
              └──────────┴──────────┴──────────┘
                              │
                              ▼
                        progress.json
```

---

## Skill Nesting

Skills can invoke other skills mid-execution:

### executing → debugging

```
executing:
  ├── Task 1 ✓
  ├── Task 2 ✓
  ├── Task 3 ✗ (test failure)
  │       │
  │       └──▶ debugging (invoked)
  │                 │
  │                 ├── Reproduce
  │                 ├── Isolate
  │                 ├── Fix
  │                 └── Return to executing
  │
  ├── Task 3 ✓ (retry after fix)
  └── Task 4 ✓
```

### planning → brainstorming

```
planning:
  ├── Read requirements
  ├── Unclear on approach?
  │       │
  │       └──▶ brainstorming (invoked)
  │                 │
  │                 ├── Explore options
  │                 ├── Get user choice
  │                 └── Return to planning
  │
  └── Create plan with chosen approach
```

---

## Handoff Patterns

### Data Passed Between Skills

| From | To | Data |
|------|-----|------|
| discovery | planning | `project-context.md`, don't-touch areas |
| brainstorming | planning | `IDEA.md`, chosen approach |
| brainstorming | prototyping | Key screens/concepts |
| prototyping | planning | Prototype files as reference |
| researcher | brainstorming | `concepts.json` |
| planning | executing | `current-plan.json` |
| executing | tracking | Task events, timing |
| executing | context-refresh | Progress state, decisions |
| context-refresh | executing | Handoff summary, preserved state |
| tracking | report-writer | `progress.json`, metrics |
| debugging | executing | Fix applied, ready to retry |

### Handoff Format

When handing off to another skill:

```markdown
## Handoff: [From Skill] → [To Skill]

**Context:**
- [What was done]
- [Key decisions made]

**Inputs for next skill:**
- [File or data 1]
- [File or data 2]

**Recommendation:**
- [Suggested approach for next skill]
```

---

## Anti-Patterns

### Don't Do This

| Anti-Pattern | Problem | Instead |
|--------------|---------|---------|
| Skip planning, go straight to executing | No verification checkpoints | Always plan, even if lite |
| Run debugging before reproducing | Wasted effort | Reproduce first |
| Prototype before brainstorming | May prototype wrong thing | Brainstorm first |
| Skip tracking in FULL mode | No metrics for report | Tracking is required |
| Chain 5+ skills in LITE mode | Defeats purpose of LITE | Keep LITE simple |

### Mode-Appropriate Chains

| Mode | Max Chain Length | Example |
|------|------------------|---------|
| LITE | 2-3 skills | planning → executing |
| STANDARD | 4-5 skills | brainstorm → plan → execute → track → report |
| FULL | 5-6 skills | brainstorm → prototype → plan → execute → track → report |
| FULL+ | 6+ skills | Full chain with epic-level iteration |

---

## Quick Reference

### "I need to..." → Skill Chain

| Need | Chain |
|------|-------|
| Build something new | brainstorm → plan → execute |
| Fix a bug | debug → plan(lite) → execute |
| Add a feature | discovery? → plan → execute |
| Understand this codebase | discovery |
| Understand a paper | researcher → brainstorm |
| Create a mockup | brainstorm → prototype |
| Long session, need reset | context-refresh |
| Finish and report | execute → tracking → report-writer |

---

*See [REGISTRY.md](./REGISTRY.md) for individual skill details*
