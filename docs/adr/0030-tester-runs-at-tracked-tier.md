# ADR-0030: The Tester runs at the user's tracked tier

**Status:** Proposed
**Date:** 2026-06-04
**Deciders:** @kknipe2k
**Tags:** capability, security, scope, builder, tier

## Context

The Builder's Tester runs a candidate framework in an isolated session
(ADR-0019). Until M08.8.C, **no production path wired the user's tier into
the run-loop enforcer** (TD-036): both run-loop entries built a fresh
`CapabilityEnforcer::new()`, which defaults to `Tier::Novice`
(`crates/runtime-main/src/capability/enforcer.rs:64`), and never called
`set_tier`. The `run_test_session_with_tier` seam existed
(`crates/runtime-main/src/builder/tester.rs`), but the Tauri command
(`test_framework` → `test_framework_with`) delegated to the default-Novice
`run_test_session_with`, so **every** real-app Tester run executed at Novice
regardless of the user's actual tier.

Two consequences:

1. **The Promoted scope-gate was unreachable in-app.** At Novice the L4 tier
   gate forbids every Write *before* the L1 `file_access` scope is consulted
   (`enforcer.rs` checks tier first). A Promoted user could never observe the
   L1 scope-gate `CapabilityViolation(Write)` in the running app — every Write
   was `TierViolation`-denied at L4 first. The scope gate was only expressible
   through the `run_test_session_with_tier` test-path seam.

2. **The tier UI desynced from the enforced tier (M08.6-IRL #19).**
   `SettingsPanel::TierControl` displays `currentTier` (the tracked tier), but
   the run enforced Novice regardless — **UI-tier ≠ enforced-tier**. Promoting
   in Settings changed the displayed tier and nothing about what the run
   could do.

ADR-0019's own rationale states that capability enforcement during a test is
"the *real* enforcement path … so a test result faithfully predicts a live
run's capability behavior." A Tester pinned to Novice **violates** that
property for a Promoted user — the test predicts a Novice run, not the user's
run. The always-Novice behavior was therefore an *unfinished wire* (TD-036),
not a decision ADR-0019 made.

## Decision

**We run the Builder's Tester at the user's tracked tier**, read from
`CurrentTierState` (the in-memory cache loaded from `tier.json` at startup and
mutated by `request_tier_transition`) at each `test_framework` invocation, and
threaded into the run-loop enforcer via `set_tier` before the first dispatch
(`test_framework` → `test_framework_with` → `run_test_session_with_tier`).

The tier is read **per invocation**. Each Tester run builds a fresh enforcer
and a fresh isolated session, so a tier transition between runs is reflected
automatically — there is no long-lived enforcer requiring mid-run
re-application. This is the root fix for #19: the run now enforces exactly the
tier the Settings UI shows.

This **refines** ADR-0019; it does **not** supersede it. ADR-0019's
isolated-session model — throwaway DB, discard-on-close, test-defaults HITL,
not §1c — stands unchanged. Only the enforcer's *tier* changes: from the
default-Novice stub to the user's tracked tier. The enforcement *logic*
(`CapabilityEnforcer::check`, the L4 tier gate, the L1 scope gate) is
byte-identical (Hard Rule 8).

**Scope boundary — the smoke session is intentionally not wired.**
`run_smoke_session_with` (`src-tauri/src/commands.rs`) constructs its SDK via
`AgentSdk::new`, which carries **no capability enforcer**, and runs a fixed
no-tool "say hello" prompt that invokes zero tools. A tier there would gate
nothing; adding `with_capability_wiring` to give it an enforcer would widen
the capability surface of a path that has none (a "no other capability-
enforcement change" scope violation) to no observable effect. The Tester is
the observable, enforcement-bearing path; smoke stays as-is. (Recorded here so
the exclusion is stated, not silently dropped.)

## Consequences

### Positive

- A Promoted user's Tester run now enforces Promoted: an in-scope Write
  succeeds, and an out-of-scope Write surfaces the L1 scope-gate
  `CapabilityViolation(Write)` in the running app (pairs with TD-034's in-app
  agent-output surfacing — both are needed for the real-app scope-gate IRL).
- #19 is fixed at the root: the Settings tier display and the enforced tier
  are the same value (`CurrentTierState`).
- The Tester once again "faithfully predicts a live run's capability behavior"
  (ADR-0019's stated property), now for every tier, not just Novice.
- A Novice user's Tester run still enforces Novice — the conservative default
  is preserved for Novice; only Promoted users gain reach, never beyond their
  actual tier.

### Negative

- `test_framework_with` grows to an 8-argument seam (the tier joins the
  collaborator set); it carries `#[allow(clippy::too_many_arguments)]`,
  mirroring `run_test_session_with_tier`. An arg-struct refactor is deferred
  (consistent with the existing Tester seams).
- The `test_framework` Tauri command now reads `CurrentTierState`. As an
  OS-touching command wrapper it stays outside the unit-test surface (the
  tauri-shell patch-gate holdout); the tier-threading *logic* is covered by
  the assembled `test_framework_with` tests.

### Neutral / future implications

- A tier transition mid-Tester-run is not re-applied to an in-flight run (the
  run completes at the tier it started with). This is correct for the
  sequential, per-invocation Tester model; a future §1c live-session model
  would route a mid-session transition through `set_tier` on the live
  enforcer (already documented at `enforcer.rs:65-66`).
- The smoke-session exclusion (above) is revisited only if smoke ever gains a
  tool-bearing prompt + a capability enforcer; until then it is correctly
  tier-agnostic.

## Alternatives Considered

### Alternative A: Keep the Tester pinned to Novice

**Rejected because:** it leaves TD-036 open and #19 unresolved, makes the
Promoted scope-gate unreachable in-app, and contradicts ADR-0019's stated
"faithfully predicts a live run" property for any non-Novice user. The
milestone deliverable is precisely to close this gap.

### Alternative B: Wire the tier into the smoke session as well

**Rejected because:** `run_smoke_session_with` has no capability enforcer
(`AgentSdk::new`) and runs a no-tool prompt, so a tier gates nothing there.
Making it non-vacuous would require adding `with_capability_wiring` — a new
capability-enforcement surface on a path that has none, for no observable
effect, violating the Stage C scope lock ("no other capability-enforcement
change"). The Tester is the observable, enforcement-bearing path.

### Alternative C: Read the tier once at app startup and cache it in the enforcer

**Rejected because:** the Tester builds a fresh enforcer per run; there is no
long-lived run-loop enforcer to cache into. Reading `CurrentTierState` at each
`test_framework` invocation is simpler and automatically reflects a tier
transition between runs — the correct granularity for the per-invocation
Tester.

## Related

- Spec sections: §8.security L4 (tier gate), Phase 9 (the Tester), §0d
  (release scope — Novice + Promoted)
- Prior ADRs: **ADR-0019** (the Tester isolated-session model — this refines
  it), ADR-0021 (the `tauri-driver` real-app E2E gate — the close bar),
  ADR-0007 (the in-process `HitlSeam`)
- Tech debt: **TD-036** (production never wires the user's tier — closed by
  this ADR's implementation), TD-034 (no agent output visible in-app — pairs
  for the in-app scope-gate observation)
- Findings: M08.6-IRL #19 (tier-UI desync — root-fixed here), #20 (the
  "Promote to Promoted" mislabel — relabelled in the same change)

## Notes

The implementation landed in M08.8.C under strict two-commit TDD: a red commit
(`476eb45`) with two assembled tests proving the production wire — a Promoted
run reaches the L1 scope gate (`CapabilityViolation{Write}`), a Novice run is
L4 tier-denied (`TierViolation`) on the same Write — and an impl commit
(`c15c6a8`) threading the tier without touching the behavioral tests. The
Promoted-vs-Novice contrast is the load-bearing proof the wire reads the tier
rather than hardcoding one; a hand-mutant (ignore the threaded tier, pass
Novice) fails the Promoted test, confirming the blocking mutation gate.
