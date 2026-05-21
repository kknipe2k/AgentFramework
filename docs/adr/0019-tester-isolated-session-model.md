# ADR-0019: The Tester isolated-session model

**Status:** Proposed
**Date:** 2026-05-21
**Deciders:** @kknipe2k
**Tags:** scope, capability, security, isolation, builder

## Context

Spec Phase 9 (the Tester) requires the Builder to run a candidate
framework — loaded from the canvas, **not** saved to disk first — in "an
isolated session with a separate SQLite database; capability violations
during test surfaced as test failures, not as live HITL prompts (test
sessions don't block on user input — defaults applied); test runs do not
write to any user data directory; results discarded on close unless
explicitly saved."

Two tensions had to be reconciled before M08 Stage F1 could be built:

1. **§1c vs §0d.** Spec Phase 9 describes the Tester's isolated session
   as a "drone-managed sandbox per §1c", but §0d marks §1c multi-session
   (concurrent live sessions, a drone pool) ❌ for v0.1. Stage A's intake
   (A.3.7) surfaced the reconciling reading for maintainer confirmation.

2. **Capability-enforcement behavior.** A live session raises a HITL /
   gap prompt on a §8.security L2 capability violation and blocks on the
   user's decision. A test session must not block — it has no user
   attending it. The *response* to a violation must change, while the
   enforcement *logic* stays byte-identical (Hard Rule 8).

The Tester is also the milestone's carry-forward discharge: it is the
first **production** code path that drives a tool-bearing framework
through `AgentSdk::run_agent` (M07.V 🟡 #5), byte-loads imported
artifacts with `skills_lock::verify` (🟡 #2), and drives
`McpDispatcher::on_server_connected` (🟡 #3). The discharge wiring is
load-bearing on the isolation model below.

## Decision

We adopt an isolated-session model for the Builder's Tester: **a
sequential, throwaway, build-time test session** — not the §1c
concurrent-session pool.

- **Throwaway database.** Each `test_framework` invocation resolves a
  fresh `std::env::temp_dir().join("runtime-tester-<uuid>.sqlite")` path
  in the Tauri shell. The test-session drone is spawned against that
  path; nothing is written to `AppHandle::path().app_local_data_dir()`
  or any other user data directory.
- **Discard on close.** Teardown reaps the drone subprocess and then
  deletes the throwaway database file. Results are discarded unless the
  user explicitly saves them (the explicit-save affordance is Stage F2's
  modal surface).
- **Test-defaults HITL.** The session is woven with
  `HitlSeam::test_defaults()` — every `await_response` resolves
  immediately with the default choice instead of registering a pending
  await. §8.security L2 capability violations are collected onto
  `TestOutcome.capability_failures` as **test failures**, never raised
  as a live HITL / gap prompt. `CapabilityEnforcer::check` is unchanged:
  this is a *response* variation, not a *logic* variation (Hard Rule 8).
- **Not §1c.** The Tester is invoked from build mode, where no live
  runtime session is executing. It needs *an* isolated session, run
  sequentially; it does **not** need the §1c concurrent-session pool.
  The v0.1 Tester therefore stays inside §0d scope.
- **Connect-handler placement.** The §5a MCP connect handler
  (`connect_test_session_mcp`, the production caller of
  `McpDispatcher::on_server_connected`) lives in
  `src-tauri/src/commands.rs`, not `crates/runtime-main`. `runtime-mcp`
  depends on `runtime-main`; the reverse edge would be a Cargo
  dependency cycle, so `runtime-main` cannot reference
  `runtime_mcp::McpDispatcher`. The shell is the only crate that sees
  both, and is already the MCP composition root (`build_mcp_dispatcher`).

The Tester reuses the smoke-session construction
(`AgentSdk::with_capability_wiring` → optional `with_mcp_dispatch` →
`run_agent`); it does not introduce a new session engine.

## Consequences

### Positive

- The Tester satisfies Phase 9 without pulling the §1c multi-session
  feature into v0.1 — scope stays as §0d defines it.
- A test run is hermetic: a fresh database per run, deleted on close,
  zero writes to user data. A Tester run can never corrupt the user's
  `session.sqlite`.
- Capability enforcement during a test is the *real* enforcement path;
  only the violation response differs, so a test result faithfully
  predicts a live run's capability behavior.
- The Tester being the production tool-driving session discharges three
  M07.V Dec-6 carry-forwards (🟡 #2 / #3 / #5) on one real code path.

### Negative

- The connect-handler placement diverges from the Stage F1 phase doc's
  `<construction_reachability_check>`, which named `tester.rs`. The
  divergence is forced by the crate dependency graph; the discharge of
  🟡 #3 is still real (a genuine production caller), only in a different
  file.
- A test run spawns a real `runtime-drone` subprocess and so carries the
  same startup latency as a smoke session.

### Neutral / future implications

- A future §1c multi-session feature (post-v0.1) would supersede the
  "sequential, build-time" qualifier here; the throwaway-DB +
  discard-on-close + test-defaults-HITL model would still apply per
  session.
- The integrity pre-flight verifies imported artifacts against a
  `skills.lock` co-located with the throwaway DB. A richer Tester
  artifact-resolution story (registry-keyed `name@version` lookup) is a
  future refinement, not a v0.1 requirement.

## Alternatives Considered

### Alternative A: Run the Tester against the live user session database

**Rejected because:** spec Phase 9 explicitly forbids writing to a user
data directory, and a test run mutating `session.sqlite` is a
data-integrity failure — a failed candidate framework could leave the
user's real session state corrupt.

### Alternative B: Implement the §1c concurrent-session pool now

**Rejected because:** §0d marks §1c ❌ for v0.1. The Tester needs *an*
isolated session, not a *pool* of concurrent ones — it runs from build
mode where no live session is executing. Building the pool would be
an out-of-scope feature expansion.

### Alternative C: Place the §5a connect handler in `crates/runtime-main`

**Rejected because:** `runtime-mcp` depends on `runtime-main`, so
`runtime-main` cannot reference `runtime_mcp::McpDispatcher` without a
Cargo dependency cycle. Adding `on_server_connected` to the
`McpToolDispatch` trait (which `runtime-main` owns) was also considered
and rejected — it widens a capability/MCP-boundary trait (Hard Rule 8)
to solve a placement problem the shell already solves cleanly.

## Related

- Spec sections: Phase 9 (the Tester), §1c (multi-session), §0d
  (release scope), §5a step 5 (MCP re-resolution), §2214 (hash-on-load),
  §8.security L2 (capability enforcement)
- Prior ADRs: ADR-0007 (the in-process `HitlSeam`), ADR-0011 (the
  agent-with-tools production driver carry-forward (d)), ADR-0014
  (`skills.lock` — integrity > availability)
- Stage A intake: M08-workbench.md A.3.9 (the §1c-vs-§0d reconciliation)

## Notes

The Stage A `<construction_reachability_check>` mapped three M07.V Dec-6
wires `inputs_reachable="false"`; Stage F1 inverted each to `"true"`:

- 🟡 #5 — `crates/runtime-main/src/builder/tester.rs::run_test_session_with`
  → `AgentSdk::run_agent` with a tool-bearing candidate framework.
- 🟡 #2 — `crates/runtime-main/src/builder/tester.rs::load_verified_artifact`
  → `skills_lock::verify`, reached from `run_test_session_with`'s
  integrity pre-flight.
- 🟡 #3 — `src-tauri/src/commands.rs::connect_test_session_mcp`
  → `McpDispatcher::on_server_connected` (see the connect-handler
  placement decision above).
