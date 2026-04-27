# `examples/ralph/` — Ralph Archetype (continuous-loop variant)

Sibling to `examples/aria/`. Same runtime primitives, different composition.

## What's different

| Dimension | aria | ralph |
|---|---|---|
| `loop_policy` | `fresh_context_per_task` | `continuous` (one persistent agent + goal_store) |
| `approval_required` | `true` | `false` (autonomous) |
| HITL density | full (gap, plan, task, failure, capability, budget, risky-tool) | minimal (gap, failure, dont_touch, capability, budget) |
| Modes | LITE / STANDARD / FULL / FULL+ | single RALPH mode |
| Root agent | `orchestrator` spawning 7 children | single `ralph-agent` |
| Goal source | plan generated each session | persistent PRD at `.ralph/prd.json` |
| Session cap | $5 | $10 (longer-running) |

## What's shared

Every runtime primitive — modes, hooks, rails, dont_touch, budget, HITL, tools, skills, agents, capability enforcement — is the *same primitive*, just composed differently. Ralph reuses several aria artifacts directly via `source: external`:

- `tools/aria_verify.md`, `git_checkpoint.md`, `git_rollback.md`, `select_cheaper_model.md`
- `skills/executing.md`, `debugging.md`

Only the Ralph-specific pieces are unique:

- `skills/ralph_loop.md` — the continuous-loop procedural guide
- `agents/ralph-agent.md` — the persistent agent

This is the §0 archetype thesis at work: **two distinct agentic systems built from the same runtime primitives.** If a third archetype (e.g., research-deep-dive, evaluation-harness) needed something the runtime didn't expose, the runtime would be missing a primitive.

## How it runs

```
1. User edits .ralph/prd.json with user stories
2. /ralph command spawns ralph-agent with continuous loop_policy
3. ralph-agent loops:
   a. Read goal_store (.ralph/prd.json)
   b. Pick next incomplete story
   c. Implement (in same context — no fresh respawn)
   d. Runtime auto-fires post_task hook (verify)
   e. If verify passed: ralph-agent updates goal_store
   f. Loop back to (a)
4. Continues until goal_store is fully complete OR
   - max_failures reached on a story (HITL escalation)
   - budget cap hit (HITL or hard_stop)
   - user invokes /ralph-status to inspect / pause
```

## Why "minimal HITL"?

Ralph's value is autonomous batch progress. Heavy HITL gates defeat the purpose. The framework still has hard floors:
- `on_dont_touch_edit` — protects critical paths
- `on_capability_violation` — Phase 8 §8.security L2 is non-negotiable
- `on_failure_threshold` — repeated failures still escalate
- `on_budget_threshold` — budget overrun stops the loop

Anything else (per-task, plan-approval, risky-tool) is opt-in via the user's tier.

## When to use ralph vs aria

- **aria**: interactive development, plan approval valuable, mode-aware verification rigor.
- **ralph**: batch backlog work, you trust the PRD, you want unattended progress.

A user can install both frameworks and switch per session.
