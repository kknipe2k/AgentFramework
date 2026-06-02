# ADR-0029: Resolve-and-resume pulled forward into v0.1 (scheduled gap-resume rung)

**Status:** Proposed
**Date:** 2026-06-01
**Deciders:** @kknipe2k
**Tags:** scope, capability, gap, hitl

## Context

Gap detection is the project's signature behavior (spec §4 / §4b): when a
running agent needs a capability it doesn't have, it calls
`request_capability`, the runtime raises a `*Missing` gap, and the session
suspends cleanly and is directed to resolution. The full flow has two
halves:

1. **Suspend** — the gap fires and the session halts, recoverable
   (suspend-and-record). **Landed at M08.7.D (rung 4)**: the `drive_stream`
   interception wires `request_capability` into the run loop, emits the
   `*Missing` gap (`requested_via: request_capability`), and breaks the turn
   loop; the gap event is persisted to the drone signal chain (recoverable
   per §1b). Exercised on the assembled path; the live-model IRL (a real
   Anthropic model calls `request_capability` → clean suspend) is the
   maintainer-run close gate, encoded as `gap_detection_live.rs` (eval E-04).

2. **Resolve-and-resume** — the user grants / installs / declines the
   missing capability through the HITL surface; on grant, the drone reloads
   from the snapshot, the agent's pending tool result is delivered (for
   `request_capability`) or the agent is respawned (for static
   `tool_missing`), and the session **resumes** (spec §4b:1697-1717).

At M08.7.D the **resume half was framed as deferred to M08.6.7 / M09** (the
phase-doc Stage D scope-lock note; the rung-4 retrospective "what was NOT
exercised"). The grounded reason the suspend half landed alone: v0.1 had no
resolve-and-resume machinery in the run loop, and the grant/install/decline
UI is an M08.6.7-adjacent surface. The scope-lock explicitly read "NOT the
full grant/install/decline resolution UI (... partly M08.6.7 / M09)."

The decision this ADR records: **the resume half is not an M09 item.** A gap
that suspends but can never be resolved leaves the project's signature
behavior visibly half-built in v0.1 — the user hits a gap, the session
halts, and there is no in-product way to grant the capability and continue.
Deferring resume to M09 means v0.1 ships a dead-end gap. That contradicts
the v0.1 product identity (a runtime that "suspends cleanly and directs the
user to the Agent Builder to resolve it" — §0 / §4): "resolve it" implies a
resume.

## Decision

**We pull resolve-and-resume forward into v0.1 as a scheduled M08.7
gap-resume rung — it is NOT deferred to M09.**

The resolve-and-resume work (the gap → HITL grant/install/decline →
snapshot-reload → deliver-pending-tool-result / respawn → agent-resumes
round-trip) becomes a **scheduled rung** in the M08.7 execution ladder,
authored to full X.1–X.6 detail at its sub-phase entry (grounded-reason
deferral, §4 rule 11 — its spec depends empirically on what rung 4's suspend
wire + the M08.6.7 HITL/grant surface actually look like). It is tracked as
an in-v0.1 deliverable, with the same cluster-gate close discipline as every
other rung: an assembled regression that drives the real resume path + a
maintainer IRL watch of a real gap being granted and the session resuming.

This supersedes the M08.7.D phase-doc scope-lock framing ("partly M08.6.7 /
M09") **only for the question of milestone ownership** — the suspend/resume
split itself (rung 4 = suspend; the gap-resume rung = resume) is unchanged.
The grant/install/decline **UI** still composes with the M08.6.7 HITL/Builder
surface; this ADR fixes the **engine resume path** as v0.1 scope and
schedules the rung that wires it.

Per §0d the v0.1 Release Scope Matrix is updated: gap **resolution** (not
just detection + suspend) is in v0.1.

## Consequences

### Positive
- v0.1's signature gap flow is **end-to-end** — detect → suspend → resolve →
  resume — not a dead-end. The product promise ("directs the user to resolve
  it") is honored in v0.1.
- The resume path gets the cluster-gate close discipline (assembled eval +
  IRL) in v0.1, rather than landing unverified in a later milestone.
- The rung-4 suspend wire (`gap_suspended` loop-break) gains its natural
  counterpart; the `GapEventCollector` / `RequestCapabilityDisposition`
  seam is the documented extension point.

### Negative
- v0.1 scope grows by one rung. Per CLAUDE.md §12 ("adding features means
  equivalent removals or pushing to v1.0+"), the offsetting line is drawn at
  the v1.0 capability split (the grant/use capability model — explicitly a
  v1.0 ADR per the rung-N ARIA note) and concurrent orchestration; those
  stay out of v0.1. The gap-resume rung is the *minimum* round-trip
  (single-session, STANDARD mode, the four gap kinds), not the full v1.0
  resolution model.
- The gap-resume rung depends on the M08.6.7 HITL/grant UI surface for the
  user-facing decision; the sequencing (M08.7 → M08.6.7) means the rung's UI
  half may land after its engine half. The rung is authored at its entry to
  ground that dependency (not guessed now).

### Neutral / future implications
- The v1.0 line stays explicit (§4 rule 11): the **grant/use capability
  split** and **concurrent orchestration** are NOT in v0.1's gap-resume rung
  — only the single-session resolve-and-resume round-trip is.
- `docs/execution-status.md` row 4 (gaps) flips to `executes — observed
  (suspend-and-record)` at the rung-4 IRL; its v0.1 target behavior line is
  amended to note resolve-and-resume is the scheduled gap-resume rung
  (this ADR), so the ledger does not read as if suspend is the whole story.

## Alternatives Considered

### Alternative A: Defer resolve-and-resume to M09 (the M08.7.D framing)
**Rejected because:** it ships a dead-end gap in v0.1 — the user hits a
capability gap, the session suspends, and there is no in-product way to
resolve and continue. That contradicts the v0.1 product identity (§0 / §4 —
"directs the user to … resolve it"). A signature behavior that only does its
first half is a rule-11 half-truth at the product level.

### Alternative B: Build resolve-and-resume inside rung 4 (M08.7.D)
**Rejected because:** rung 4's grounded scope was the suspend wire, and the
resume path depends on the M08.6.7 HITL/grant UI surface that did not exist
at M08.7.D — building it in rung 4 would have been guessing at an unbuilt
dependency (the exact paint-not-execute trap rule 11 guards against). A
separate scheduled rung, authored at its entry, grounds the dependency.

### Alternative C: Treat it as a tech-debt item, not a scheduled rung
**Rejected because:** the zero-propagation rule (cluster-pattern.md §2)
bans "route to a later milestone." A v0.1-scope capability with a known
resume gap is a **scheduled cluster**, not floating debt — it gets a rung,
an eval, and an IRL.

## Related

- Spec sections: §0 (product identity), §0d (Release Scope Matrix — gap
  resolution now in v0.1), §4 / §4b (the gap flow + `request_capability`),
  §1b (snapshot recovery / resume-rebuilds)
- Prior ADRs: ADR-0019 (Tester suspend-and-record model — the gap-resume
  rung's live-session resume is the complement, not a contradiction);
  ADR-0027 (skill injection — the rung-3 reuse precedent for wiring a built
  handler into the loop)
- Code: `crates/runtime-main/src/sdk/agent_sdk.rs`
  (`dispatch_request_capability`, `TurnFeedback.gap_suspended`, the
  `run_agent` suspend break — the documented extension point for the resume
  path); `crates/runtime-main/src/sdk/request_capability.rs`
  (`handle_request_capability`); `crates/runtime-main/tests/gap_detection_live.rs`
  (eval E-04 — the rung-4 suspend IRL)
- Ledger: `docs/execution-status.md` row 4 (gaps); the M08.7 rung table in
  `docs/build-prompts/M08.7-execution-engine.md`

## Notes

This ADR is filed at the M08.7.D close as a maintainer scope decision: the
resolve-and-resume half of the gap flow is v0.1 scope, scheduled as a gap-resume
rung, not deferred to M09. Status `Proposed`; flips `Accepted` in the M08.7
PR. The rung's full X.1–X.6 authoring happens at its sub-phase entry (the
grounded-reason deferral pattern), where its M08.6.7 HITL/grant-UI
dependency is grounded rather than guessed.
