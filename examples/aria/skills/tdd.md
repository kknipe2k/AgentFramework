---
name: tdd
version: 1.0.0
description: Test-driven development. Write the failing test first, then the minimum implementation to pass, then refactor.

triggers:
  semantic:
    - "tdd"
    - "test first"
    - "test-driven"
  programmatic:
    - event: task_started
      when: { "in": [{ "var": "task.tags" }, ["tdd"]] }
    - event: mode_locked
      when: { "==": [{ "var": "session.mode" }, "FULL+"] }

mode_variants:
  LITE:     { include_sections: ["short_loop"] }
  STANDARD: { include_sections: ["short_loop", "regression"] }
  FULL:     { include_sections: ["short_loop", "regression", "full_loop"] }
  FULL+:    { include_sections: ["short_loop", "regression", "full_loop", "mandatory"] }

required_tools: ["Read", "Write", "Edit", "Bash"]
required_skills: []

capabilities:
  tools_called:    ["Read", "Write", "Edit", "Bash"]
  skills_loaded:   []
  file_access:
    read:  ["**/*"]
    write: ["src/**", "tests/**", "test/**", "**/*.test.*", "**/*.spec.*"]
  network:         []
  shell:           true
  spawn_agents:    []

provenance:
  generator:    "hand-authored"
  source:       ".aria/skills/tdd.md (ported)"
  authored_at:  "2026-04-18T00:00:00Z"
  content_hash: "sha256:placeholder-replace-on-first-load"
---

# TDD Skill

Tests are the contract. Write them first.

## short_loop

The minimum cycle:

1. **Red.** Write a single failing test that captures the next bit of behavior. Run it. Confirm it fails for the *right reason* (assertion, not setup error).
2. **Green.** Write the minimum code that makes the test pass. Not the cleanest. Not the most general. The minimum.
3. **Refactor.** With the test passing, clean up. The test pins behavior; refactor freely.

One micro-cycle should take 5–15 minutes. If a cycle is longer, the test is too big — split it.

## regression

Every reported bug gets a regression test before any fix. Sequence:

1. Write a test that reproduces the bug. Run it. Confirm it fails matching the bug report.
2. Apply the fix. Run the regression test. Confirm green.
3. Run the full suite. Confirm nothing else broke.
4. Commit fix + regression test together (one commit, atomically).

The commit message references the bug ID. The regression test prevents recurrence.

## full_loop

For larger features, layered TDD:

1. **Acceptance test** (slow, high-level) — captures the user-visible behavior. Often a smoke test or e2e.
2. **Unit tests** (fast, focused) — drive each piece of the implementation.
3. Outer-loop / inner-loop ping-pong:
   - Run acceptance test. It fails.
   - Drop to unit-test inner loop until enough units pass that the acceptance test could pass.
   - Run acceptance test. If still red, identify the next missing piece, return to inner loop.
   - Continue until acceptance test green.

This is "outside-in" TDD. It keeps the work grounded in user-visible value while letting unit tests guide implementation detail.

## mandatory

For FULL+ mode, TDD is not optional. Every code change requires:

- A test that fails before the change.
- A test that passes after the change.
- The before/after test outputs are recorded in the design notes.

The runtime checks for new test files in commits via the `post_task` hook; commits without test changes fail the verify gate when this skill is loaded under FULL+.

---

## Outputs

- New / updated test files
- Source changes scoped to make tests pass
- (FULL+) before/after test output captured in design notes

## Failure modes

- Test that "passes by accident" (assertion never runs) — confirm the test fails when assertion is removed; if not, the test is broken.
- Implementation that makes the test pass but violates the spirit of the requirement — load `executing` for procedural review.
- Refactor that breaks unrelated tests — revert refactor, file as a separate task.
