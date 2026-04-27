---
name: executing
version: 1.0.0
description: Procedural guide for implementing a single task from an approved plan. Read → analyze → implement → verify → next.

triggers:
  semantic:
    - "execute the plan"
    - "implement task"
    - "/execute"
  programmatic:
    - event: plan_approved
    - event: task_started

mode_variants:
  LITE:     { include_sections: ["loop"] }
  STANDARD: { include_sections: ["loop", "subagents"] }
  FULL:     { include_sections: ["loop", "subagents", "design_notes"] }
  FULL+:    { include_sections: ["loop", "subagents", "design_notes", "epic_gates"] }

required_tools: ["Read", "Write", "Edit", "Glob", "Grep", "SpawnAgent", "LoadSkill"]
required_skills: []

capabilities:
  tools_called:    ["Read", "Write", "Edit", "Glob", "Grep", "SpawnAgent", "LoadSkill"]
  skills_loaded:   []
  file_access:
    read:  ["**/*"]
    write: ["src/**", "tests/**", "test/**", ".aria-runtime/state/**"]
  network:         []
  shell:           false
  spawn_agents:    ["analyzer", "implementer", "verify-app", "simplifier"]

provenance:
  generator:    "hand-authored"
  source:       ".aria/skills/executing.md (ported)"
  authored_at:  "2026-04-18T00:00:00Z"
  content_hash: "sha256:placeholder-replace-on-first-load"
---

# Executing Skill

How to execute a task from an approved plan.

## loop

The base loop, executed once per task in plan order:

```
1. ANNOUNCE   "Starting task N/M: {title}"
2. HITL CHECK if task.hitl: surface checkpoint, wait
3. IMPLEMENT  do the work
4. VERIFY     runtime auto-fires task_defaults.post_hooks
5. COMMIT     runtime auto-commits if verify passed
6. UPDATE     runtime updates plan progress
7. NEXT       continue to next pending task
```

Steps 4–6 are runtime concerns — you do not invoke verify or git directly. The hook fires at task boundary; runtime commits if the hook returns `passed: true`.

If verify fails:
- `task_failed` fires. Failure counter increments.
- If `failure_count < max_failures`, the task is rescheduled. You may load `debugging` and retry.
- If `failure_count >= max_failures`, `task_escalated` fires and HITL takes over per `hitl_policy.on_failure_threshold`.

## subagents

For STANDARD+ modes, do not implement directly in your context. Spawn:

```
1. SpawnAgent { agent_id: "analyzer", prompt: "<task description + plan context>" }
   → wait for analyzer report
2. SpawnAgent { agent_id: "implementer", prompt: "<task + analyzer findings>" }
   → wait for implementer to finish (it cannot call SpawnAgent)
3. SpawnAgent { agent_id: "verify-app", prompt: "<task acceptance criteria>" }
   → wait for verify-app result
```

Subagent isolation gives each agent fresh context per task. Failures don't pollute future tasks.

You as the orchestrator stay in this skill. You do not become any subagent.

## design_notes

For FULL/FULL+, log reasoning to `.aria-runtime/design-notes-{plan_id}.md` after each task:

```
## Task {id}: {title}

### Approach
<one-paragraph: why this approach over alternatives>

### Key decisions
- Decision 1: rationale, alternative considered, confidence
- Decision 2: ...

### Assumptions
- Assumption 1: <stated explicitly so a reviewer can challenge it>
```

Design notes are read by the reviewer agent at session end and feed into the report-writer's output.

## epic_gates

For FULL+ only: tasks are grouped into epics. Between epics:

1. Pause execution.
2. Surface an epic summary panel: tasks complete, time spent, design decisions logged, risks remaining.
3. HITL gate: user approves continue / pauses / aborts.
4. If approved: context refresh. The next epic starts with a fresh planner agent re-reading the design doc.

Epic gates limit context drift on large multi-day workflows.

---

## Outputs

- Source / test file changes (via implementer subagent)
- (FULL+) `.aria-runtime/design-notes-{plan_id}.md`
- Task status updates in plan store

## Failure modes

- Subagent returns unusable output → re-spawn with clarified prompt; on second failure, escalate.
- Subagent attempts capability outside its declared set → runtime blocks, surfaces HITL; do not auto-grant.
- Verify keeps failing despite passing locally → environment drift; load `debugging` and check for env-dependent assertions.
