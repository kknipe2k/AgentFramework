---
name: debugging
version: 1.0.0
description: Diagnose test failures, runtime errors, and verification gate failures. Triage before changing code.

triggers:
  semantic:
    - "debug"
    - "test failure"
    - "error in"
    - "verification failed"
  programmatic:
    - event: hook_failed
      when: { "==": [{ "var": "hook_id" }, "verify_standard"] }
    - event: task_failed
      when: { ">=": [{ "var": "task.failure_count" }, 1] }

mode_variants:
  LITE:     { include_sections: ["triage"] }
  STANDARD: { include_sections: ["triage", "fix"] }
  FULL:     { include_sections: ["triage", "fix", "regression_test"] }
  FULL+:    { include_sections: ["triage", "fix", "regression_test", "rca"] }

required_tools: ["Read", "Bash", "Grep"]
required_skills: []

capabilities:
  tools_called:    ["Read", "Bash", "Grep", "Glob"]
  skills_loaded:   []
  file_access:     { read: ["**/*"], write: [] }
  network:         []
  shell:           true
  spawn_agents:    []

provenance:
  generator:    "hand-authored"
  source:       ".aria/skills/debugging.md (ported)"
  authored_at:  "2026-04-18T00:00:00Z"
  content_hash: "sha256:placeholder-replace-on-first-load"
---

# Debugging Skill

Triage failures before touching code. Speculative fixes compound bugs.

## triage

Before any fix attempt:

1. **Read the actual error.** Not the summary. The full stack trace, the failing assertion, the line of test output that flipped.
2. **Reproduce locally.** Run the failing command in isolation. Confirm it fails the same way every time. If flaky, note that — flaky failures need different treatment than deterministic ones.
3. **Identify the smallest failing case.** Which test? Which input? Which line? Narrow until the failure is one assertion or one error message.
4. **Form a hypothesis.** State, in one sentence, what you believe is wrong. Do not fix yet.

Triage output (write to design notes if FULL/FULL+):

```
Failure: <one-line summary>
Reproducible: yes/no/intermittent
Smallest case: <test name + input>
Hypothesis: <what's wrong>
Confidence: 0.0–1.0
```

If confidence < 0.6: do not proceed to `fix`. Gather more information first (read related code, check recent commits, search for similar patterns).

## fix

Only after triage establishes confidence ≥ 0.6:

1. Make the minimal change that addresses the hypothesis.
2. Run the previously-failing case. Confirm it now passes.
3. Run the full verify level appropriate to mode.
4. If anything else broke, revert. Re-triage.

The minimal change is rarely the cleanest. Resist the urge to refactor while debugging. File a follow-up task for cleanup.

## regression_test

For FULL/FULL+: every fix gets a regression test that fails before the fix and passes after. Add the test before committing the fix.

Sequence:
1. Write the regression test that captures the bug.
2. Confirm it fails.
3. Apply the fix.
4. Confirm the regression test now passes.
5. Confirm prior tests still pass.
6. Commit fix + regression test together.

## rca

For FULL+ only: write a root-cause-analysis document to `.aria-runtime/docs/RCA-{plan_id}-{task_id}.md`:

- What happened (symptom)
- Why it happened (technical root cause)
- Why it wasn't caught (process root cause)
- What changes prevent recurrence (code, tests, process)
- Whether similar patterns exist elsewhere

RCAs feed into the team's post-incident review process. Even if the team doesn't have one, writing the RCA forces clearer thinking.

---

## Outputs

- (LITE) Inline triage statement.
- (STANDARD+) Triage block in design notes; fix commit referencing the triage.
- (FULL+) Regression test + RCA document.

## When to escalate via `request_capability`

- After 2 fix attempts both fail → request `request_capability { capability_kind: 'skill', capability_name: 'pair-debugging' }` (if such a skill exists in the framework) or surface to HITL.
- If the failure is in code marked `dont_touch` → cannot fix; escalate to user immediately.
- If the failure crosses a runtime/framework boundary → escalate; do not attempt to fix runtime code.
