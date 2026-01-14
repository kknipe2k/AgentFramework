# Planning Skill

> Create structured implementation plans with HITL approval

---
version: 1.0.0
modes: [LITE, STANDARD, FULL, FULL+]
triggers: ["plan", "design", "figure out", start of work]
inputs: [requirements, IDEA.md (optional), prototypes (optional)]
outputs: [current-plan.json, plan summary for approval]
dependencies: [brainstorming (optional)]
---

## When to Use

Use this skill when:
- Starting a new feature or task
- User asks to "plan", "design", or "figure out" something
- Before any significant code changes
- After brainstorming/prototyping is complete

## Workflow

### Step 1: Understand Requirements

Ask 2-3 clarifying questions (max). Focus on:
- What's the expected outcome?
- Any constraints or preferences?
- What should NOT be changed?

Don't over-ask. Get enough to start, refine as you go.

### Step 2: Read Context

Before planning, read:
1. `.aria/project-context.md` (if exists) - Project knowledge, don't-touch areas
2. `.aria/docs/IDEA.md` (if exists) - Brainstorm output
3. `.aria/prototypes/` (if exists) - Visual references
4. Relevant existing code - Understand current patterns
5. `.aria/state/current-plan.json` (if exists) - Check for existing plan

### Step 3: Determine Hierarchy Depth

Based on project size (from Router), use appropriate hierarchy:

| Size | Hierarchy | Breakdown |
|------|-----------|-----------|
| SMALL | Tasks only | 1-5 tasks, no grouping |
| MEDIUM | Phases → Tasks | 2-4 phases, 3-5 tasks each |
| LARGE | Major Steps → Phases → Tasks | 2-3 major steps, 2-3 phases each |
| X-LARGE | Epics → Major Steps → Phases → Tasks | 2-4 epics, full hierarchy |

### Step 4: Create Plan

Break work into appropriate hierarchy:

**For SMALL (Tasks only):**
```
Task 1: Description (~10 min)
Task 2: Description (~15 min)
Task 3: Description (~20 min)
```

**For MEDIUM (Phases → Tasks):**
```
Phase 1: Setup
  Task 1.1: Description (~10 min)
  Task 1.2: Description (~10 min)

Phase 2: Core Implementation
  Task 2.1: Description (~20 min)
  Task 2.2: Description (~30 min)
```

**For LARGE (Major Steps → Phases → Tasks):**
```
Major Step 1: Backend Foundation
  Phase 1.1: Database Setup
    Task 1.1.1: Description (~15 min)
    Task 1.1.2: Description (~20 min)
  Phase 1.2: API Structure
    Task 1.2.1: Description (~20 min)

Major Step 2: Frontend
  Phase 2.1: Components
    Task 2.1.1: Description (~25 min)
```

**For X-LARGE (Epics → Major Steps → Phases → Tasks):**
```
Epic 1: User Management
  Major Step 1.1: Authentication
    Phase 1.1.1: Login Flow
      Task 1.1.1.1: Description (~20 min)
```

### Step 5: Estimate Time & Tokens

**Time estimation guidelines:**

| Task Complexity | Time | Tokens |
|-----------------|------|--------|
| Simple (rename, config) | 5-10 min | ~2,000 |
| Medium (new function, component) | 15-30 min | ~5,000-10,000 |
| Complex (new module, integration) | 30-60 min | ~10,000-20,000 |

**Aggregate estimates:**
- Sum task times for phase total
- Sum phase times for major step total
- Add 20% buffer for context switching

### Step 6: Set Refresh Points

Based on size, plan context refresh:

| Size | Refresh Points |
|------|----------------|
| SMALL | None (short enough) |
| MEDIUM | Between phases |
| LARGE | Between major steps + phases |
| X-LARGE | Between epics + major steps + phases |

Mark refresh points in plan:
```
[REFRESH] Phase 2 complete - context refresh recommended
```

### Step 7: Get Approval

Present plan and ask:
```
Plan ready. [a]pprove / [r]evise / [e]dit / [c]ancel?
```

Wait for response. Do NOT proceed without approval.

---

## Plan Format (JSON)

Save to `.aria/state/current-plan.json`:

### SMALL Format
```json
{
  "id": "plan-20260111-143022",
  "size": "SMALL",
  "mode": "LITE",
  "title": "Brief description",
  "status": "pending_approval",
  "created": "2026-01-11T14:30:22Z",
  "estimates": {
    "total_minutes": 45,
    "total_tokens": 15000
  },
  "tasks": [
    {
      "id": "1",
      "title": "Task title",
      "description": "What to do",
      "status": "pending",
      "hitl": false,
      "estimated_minutes": 15,
      "estimated_tokens": 5000,
      "files": ["file.ts"]
    }
  ],
  "hitl_checkpoints": [],
  "risks": []
}
```

### MEDIUM Format
```json
{
  "id": "plan-20260111-143022",
  "size": "MEDIUM",
  "mode": "STANDARD",
  "title": "Brief description",
  "status": "pending_approval",
  "estimates": {
    "total_minutes": 120,
    "total_tokens": 40000
  },
  "refresh_points": ["after_phase_1", "after_phase_2"],
  "phases": [
    {
      "id": "1",
      "title": "Phase title",
      "status": "pending",
      "estimated_minutes": 60,
      "tasks": [
        {
          "id": "1.1",
          "title": "Task title",
          "status": "pending",
          "hitl": false,
          "estimated_minutes": 20,
          "files": ["file.ts"]
        }
      ]
    }
  ]
}
```

### LARGE Format
```json
{
  "id": "plan-20260111-143022",
  "size": "LARGE",
  "mode": "FULL",
  "title": "Brief description",
  "status": "pending_approval",
  "estimates": {
    "total_minutes": 300,
    "total_tokens": 100000
  },
  "refresh_points": ["after_major_step_1", "after_phase_1.2"],
  "major_steps": [
    {
      "id": "1",
      "title": "Major step title",
      "status": "pending",
      "estimated_minutes": 150,
      "phases": [
        {
          "id": "1.1",
          "title": "Phase title",
          "status": "pending",
          "estimated_minutes": 75,
          "tasks": [
            {
              "id": "1.1.1",
              "title": "Task title",
              "status": "pending",
              "hitl": false,
              "estimated_minutes": 25,
              "files": ["file.ts"]
            }
          ]
        }
      ]
    }
  ]
}
```

### X-LARGE Format
```json
{
  "id": "plan-20260111-143022",
  "size": "X-LARGE",
  "mode": "FULL+",
  "title": "Brief description",
  "status": "pending_approval",
  "design_doc": ".aria/docs/DESIGN.md",
  "estimates": {
    "total_minutes": 600,
    "total_tokens": 200000
  },
  "refresh_points": ["after_epic_1", "after_major_step_1.1"],
  "epics": [
    {
      "id": "1",
      "title": "Epic title",
      "status": "pending",
      "hitl_gate": true,
      "estimated_minutes": 300,
      "major_steps": [
        {
          "id": "1.1",
          "title": "Major step",
          "phases": [...]
        }
      ]
    }
  ]
}
```

---

## Task Status Values

| Status | Meaning |
|--------|---------|
| `pending` | Not started |
| `in_progress` | Currently working on |
| `done` | Completed and verified |
| `blocked` | Waiting for HITL or dependency |
| `skipped` | Explicitly skipped |

---

## HITL Markers

Mark `"hitl": true` for tasks involving:
- Security-sensitive code (auth, payments, encryption)
- Destructive operations (deletes, migrations)
- Configuration changes
- External service integration
- Anything in "don't touch" areas

---

## Plan Summary Format

When presenting plan to user, format based on size:

### SMALL
```
## Plan: [Title]

**Estimate:** [N] tasks, ~[M] minutes, ~[K] tokens

1. [ ] Task 1 - description (~10 min)
2. [ ] Task 2 - description (~15 min)
3. [HITL] Task 3 - description (~20 min)

[a]pprove / [r]evise / [c]ancel?
```

### MEDIUM
```
## Plan: [Title]

**Estimate:** [N] tasks in [P] phases, ~[M] minutes, ~[K] tokens
**Refresh points:** After each phase

### Phase 1: [Name] (~30 min)
1.1 [ ] Task - description (~15 min)
1.2 [ ] Task - description (~15 min)

### Phase 2: [Name] (~45 min)
2.1 [ ] Task - description (~20 min)
2.2 [HITL] Task - description (~25 min)

**Risks:** [list]

[a]pprove / [r]evise / [c]ancel?
```

### LARGE / X-LARGE
```
## Plan: [Title]

**Size:** LARGE | Mode: FULL
**Estimate:** [N] tasks, [P] phases, [S] major steps
**Time:** ~[M] minutes | **Tokens:** ~[K]
**Refresh points:** Between major steps

### Major Step 1: [Name] (~2 hours)

#### Phase 1.1: [Name] (~45 min)
- 1.1.1 [ ] Task (~15 min)
- 1.1.2 [ ] Task (~15 min)
- 1.1.3 [HITL] Task (~15 min)

#### Phase 1.2: [Name] (~75 min)
[REFRESH after this phase]
...

**HITL Checkpoints:** [list]
**Risks:** [list]
**Don't Touch:** [list]

[a]pprove / [r]evise / [c]ancel?
```

---

## Revision Handling

If user says "revise":
1. Ask what to change
2. Update plan
3. Present again for approval

If user says "edit":
1. Let user provide specific changes
2. Apply changes to plan
3. Present again for approval

If user says "cancel":
1. Clear plan state
2. Ask for new direction

---

## Output

After planning, you should have:

1. **Plan JSON** saved to `.aria/state/current-plan.json`
2. **User approval** to proceed
3. **Clear estimates** for time and tokens
4. **Refresh points** identified

Then hand off to Executing skill.

---

## Tips

- **Be concise** - Plans should be scannable, not novels
- **Be specific** - "Implement X" not "Work on stuff"
- **Front-load risk** - Do uncertain tasks early
- **Small tasks** - 15-30 min each, easier to verify
- **Preserve context** - Note files you'll need to read
- **Buffer time** - Add 20% to estimates for context switching
- **Match hierarchy to size** - Don't over-structure small projects
