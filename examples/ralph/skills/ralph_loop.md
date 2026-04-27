---
name: ralph_loop
version: 1.0.0
description: Procedural guide for Ralph's continuous PRD-driven loop. One persistent agent reads a goal store, picks the next story, works on it, updates the store, repeats.

triggers:
  semantic:
    - "ralph"
    - "/ralph"
    - "continue PRD"
  programmatic:
    - event: session_start
      when: { "==": [{ "var": "framework.name" }, "ralph"] }

mode_variants:
  RALPH: { include_sections: ["loop", "story_completion", "stuck"] }

required_tools: ["Read", "Write", "Edit", "Glob", "Grep", "LoadSkill", "request_capability"]
required_skills: ["executing", "debugging"]

capabilities:
  tools_called:    ["Read", "Write", "Edit", "Glob", "Grep", "LoadSkill", "request_capability"]
  skills_loaded:   ["executing", "debugging"]
  file_access:
    read:  ["**/*"]
    write: ["src/**", "tests/**", "test/**", ".ralph/prd.json"]
  network:         []
  shell:           false
  spawn_agents:    []

provenance:
  generator:    "hand-authored"
  source:       ".aria/ralph/ralph.sh + .aria/ralph/prompt.md"
  authored_at:  "2026-04-18T00:00:00Z"
  content_hash: "sha256:placeholder-replace-on-first-load"
---

# Ralph Loop

A single agent. A persistent goal store. Iterate until done.

## loop

Each iteration:

1. **Read the PRD.** `.ralph/prd.json` is the goal store. Read it fresh every iteration; it may have been updated by a prior iteration's `Edit`.
2. **Pick the next incomplete story.** Stories have `passed: false` and the lowest `priority` number among incomplete. Tie-break by `id` ascending.
3. **Read the story's acceptance criteria.** They are the contract for "done."
4. **Implement.** Use the `executing` skill's loop. Keep changes minimal. Match patterns from existing code.
5. **Verify.** The runtime auto-fires the `post_task` hook (verify). If pass, continue. If warn (Ralph uses `on_failure: warn` — not block), log the warning and continue with the user-visible failure noted.
6. **Update the PRD.** If acceptance criteria are met, set `passed: true` and bump the story's `passes` count. If not, increment `failure_count`.
7. **Loop back to step 1.**

The runtime provides between-iteration snapshots (drone) so a crash mid-iteration restarts from the last clean snapshot, not the start of session.

## story_completion

A story is "complete" when:

- All `acceptance_criteria` are observably met (verify hook attests).
- `passes >= required_passes` (default 1; framework can require N consecutive passes for confidence).
- No regressions in adjacent tests (verify ran clean, not just the story-specific tests).

Mark the story `passed: true` and set `completed_at` to the current timestamp.

If after 3 failed attempts on the same story, the runtime auto-fires `on_failure_threshold` HITL. Wait for user guidance:
- Retry with hint
- Skip the story (mark `skipped: true`)
- Abort the loop

## stuck

If you find yourself making the same change repeatedly, or unable to satisfy criteria after 3 attempts, **load the `debugging` skill** and triage. Don't keep flailing.

If after `debugging` triage you still can't make progress:
- Call `request_capability { capability_kind: 'skill', capability_name: 'pair-debugging' }` — you may be missing a skill the framework forgot to include.
- Or surface to user via the failure-threshold path.

Do NOT improvise tools or skills you don't have. Do NOT bypass `dont_touch` even if a fix seems obvious. Both surface as `capability_violation` and halt the loop.

---

## Outputs per iteration

- Source / test file changes (per `executing`)
- Updated `.ralph/prd.json` with story status changes
- Decision records in VDR (runtime auto-emits)

## Loop termination

The loop ends when:

1. All stories in `prd.json` have `passed: true` → emit `plan_complete`. Session ends successfully.
2. Budget cap hit → runtime auto-fires `budget_exceeded`, hard-stops the agent.
3. User invokes `/ralph-status` and chooses "abort" → graceful shutdown after current iteration.
4. 3 consecutive iterations escalate to HITL with no resolution → user typically aborts.

The session-end hook then fires `report-writer` (if installed) for a summary.
