---
name: planning
version: 1.0.0
description: Create implementation plans with HITL approval gates. Breaks work into discrete, verifiable tasks before execution begins.

triggers:
  semantic:
    - "create a plan"
    - "/plan"
    - "let's plan this"
    - "mode_start"
  programmatic:
    - event: session_start
      when: { "!=": [{ "var": "session.mode" }, "LITE"] }
    - event: plan_revised
      when: { ">=": [{ "var": "plan.revision_count" }, 1] }

mode_variants:
  LITE:
    include_sections: ["quick"]
    description_override: "Quick task list, no formal approval needed."
  STANDARD:
    include_sections: ["full", "approval"]
  FULL:
    include_sections: ["full", "approval", "risks", "estimates"]
  FULL+:
    include_sections: ["full", "approval", "risks", "estimates", "design_doc"]

required_tools: ["Read", "Write", "Glob"]
required_skills: ["discovery"]

capabilities:
  tools_called:    ["Read", "Write", "Glob"]
  skills_loaded:   ["discovery"]
  file_access:     { read: ["**/*"], write: [".aria-runtime/state/current-plan.json"] }
  network:         []
  shell:           false
  spawn_agents:    []

provenance:
  generator:    "hand-authored"
  source:       ".aria/skills/planning.md (ported)"
  authored_at:  "2026-04-18T00:00:00Z"
  content_hash: "sha256:placeholder-replace-on-first-load"
---

# Planning Skill

Create discrete, verifiable tasks before execution begins. Plans are the contract between the agent and the user.

## quick

Write a short bulleted task list. No formal structure. No HITL gate. Used for LITE mode where the work is small enough that ceremony costs more than it saves.

```
1. Read X
2. Edit Y
3. Run verify
```

That's it. Move on.

## full

Build a structured plan saved to the plan store. Required fields:

- `id` — generated
- `title` — one line
- `tasks[]` — each with `id`, `title`, `description`, `status: pending`, `hitl: bool`, `estimated_minutes`, `acceptance_criteria[]`

Tasks must be:
- **Discrete** — each is independently verifiable
- **Ordered** — dependencies clear
- **Bounded** — estimated time, max one verify cycle each

After writing the plan to `.aria-runtime/state/current-plan.json`, emit `plan_created`. Wait for the approval gate.

## approval

When `plan_creation.approval_required: true` (set by mode), do NOT begin executing. The runtime suspends after `plan_created` and surfaces an approval panel. User chooses:

- **Approve** → execution begins task by task
- **Revise** → user edits the plan or sends feedback; you regenerate
- **Cancel** → session continues without a plan

Do not act before you receive `plan_approved`.

## risks

For FULL/FULL+ modes, identify and surface risks before approval:

- Areas of the codebase that could break (from `discovery` skill)
- External dependencies that might fail (network, third-party APIs)
- Patterns that historically fail (from prior session learnings if available)
- "Don't touch" zones that the work brushes against

Add a `risks[]` array to the plan. Surface in the approval panel.

## estimates

For FULL/FULL+ modes, every task gets `estimated_minutes`. Use prior signals (skill performance history) when available; otherwise estimate by complexity heuristic:

- Read-only tasks (analysis, discovery): 5–15 min
- Single-file edits with tests: 15–30 min
- Multi-file refactors: 30–60 min
- New feature with cross-cutting changes: 60+ min, consider splitting

Mark tasks `estimated_minutes > 90` as candidates for further decomposition.

## design_doc

For FULL+ mode only. Before planning, write a design document to `.aria-runtime/docs/DESIGN-{plan_id}.md` covering:

1. Problem statement
2. Proposed approach
3. Architecture sketch
4. API surface changes
5. Migration path (if breaking)
6. Open questions

Get user approval on the design doc *before* generating the plan. The plan implements what the design doc specifies.

---

## Outputs

- `.aria-runtime/state/current-plan.json` — Plan JSON
- `plan_created` event with task count, approval_required flag
- (FULL+) `.aria-runtime/docs/DESIGN-{plan_id}.md`

## Failure modes

- Plan rejected by user 3+ times → consider sizing was wrong; emit a `request_capability { capability_kind: 'skill', capability_name: 'sizing' }` to escalate.
- Tasks too large (estimated > 90 min) → split before submitting for approval.
- Required dependencies (e.g., discovery findings) not loaded → call `LoadSkill { skill_name: 'discovery' }` first.
