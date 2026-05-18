# ADR-0013: Cross-process run identity (the drone-seeded session id is canonical; the in-process SDK adopts it)

**Status:** Accepted
**Date:** 2026-05-17
**Deciders:** @kknipe2k
**Tags:** persistence, ipc-adjacent, identity, scope

## Context

M06 IRL testing (`docs/M06-irl-findings.md` finding 🔴-2) surfaced that
the agent signal stream never reached the live drone DB: after a smoke
run `session.sqlite` had `signals = 0` while `heartbeats`/`snapshots`
populated the same file. The phase-doc diagnosis (`docs/build-prompts/
M06.5-irl-fix.md` B.1) identified one root cause — `AgentSdk::emit`
only did `event_tx.send` and never called `write_signal` — and scoped
B.fix to adding that emission.

Building the assembled-app regression test surfaced a **second
necessary condition** the phase doc did not diagnose, invisible to
every green automated gate because Stage-V verified the drone and the
SDK in isolation and `recovery_lifecycle.rs` is green *only because it
manually passes a session id equal to the drone's seeded id* (the exact
Stage-V blind spot the IRL cycle exists to catch — applied here to the
phase doc's own root-cause claim):

Verified root cause (citations on `main` @ `71107c7`):

- `crates/runtime-drone/migrations/000_initial.sql:44` — `signals
  .session_id` is a `FOREIGN KEY` into `sessions(id)`.
- `crates/runtime-drone/src/db.rs` — `db::init` runs
  `PRAGMA foreign_keys=ON`.
- `crates/runtime-drone/src/lib.rs:143` — at startup the drone runs
  `INSERT OR IGNORE INTO sessions (id, …)` for **its `--session-id`**
  and only that id; no other `sessions` row is seeded in production.
- `src-tauri/src/drone_lifecycle.rs:87` — `DroneLifecycle::spawn`
  minted `Uuid::new_v4().to_string()` for the drone's `--session-id`.
- `src-tauri/src/commands.rs` — `run_smoke_session_with` minted an
  **independent** `SessionId::new()` for the `AgentSdk`.

The SDK therefore wrote every signal under an id with no matching
`sessions` row. SQLite's `INSERT OR IGNORE` does **not** suppress
foreign-key constraint violations, so each signal `INSERT` was silently
rejected — `signals = 0` even once the emission was correctly wired.

The drone process is the system-of-record (ADR-0007) and the only
process that seeds the canonical `sessions` row. But *which* id is the
run's identity, and *who owns minting it*, was never decided: the drone
and the in-process SDK each minted their own. As in ADR-0012 (its
sibling — single source-of-truth *path*), the defect is that the
**composition layer never decided** the canonical value; here the value
is run identity rather than the DB path. The decision was implicit and
therefore drifted.

If we do not decide this explicitly, M07 (the agent-with-tools loop +
recovery) builds on a `signals` table that is structurally empty in the
assembled app, and every downstream consumer that correlates state by
session (recovery/replay per spec §1b, audit, the VDR/plan projectors)
is hollow.

## Decision

We adopt a **single canonical run identity per app run** in v0.1: the
**drone-seeded session id is canonical**, and the in-process SDK
**adopts** it rather than minting its own.

Concretely: `DroneLifecycle` exposes the id it seeded the drone with
via `DroneLifecycle::sdk_session_id() -> SessionId` (parsing its
hyphenated-UUID `--session-id`); the Tauri shell registers that
`SessionId` as managed state at setup, immediately after
`DroneLifecycle::spawn` succeeds; `run_smoke_session` /
`run_smoke_session_with` construct the `AgentSdk` with that shared
`SessionId` instead of `SessionId::new()`. Every signal the SDK writes
then carries an id for which the drone has already seeded a `sessions`
row, so the `signals → sessions` foreign key accepts it.

This is the **identity analogue of ADR-0012**: where ADR-0012 makes the
drone session DB *path* the single source of truth, this ADR makes the
drone-seeded session *id* the single source of truth for run identity.
Both are composition-layer unifications of a value the shell previously
let two sides mint independently; neither changes the drone, the IPC
protocol, the schema, or adds a dependency.

`SessionId` (`runtime_main::sdk::SessionId`, a newtype over `Uuid`) and
the drone `--session-id` already share the hyphenated-UUID string form,
so adoption is an exact round-trip; `DroneLifecycle::sdk_session_id`
falls back to a fresh `SessionId` only for the `spawn_with` test seam
(arbitrary non-UUID ids), which no production path invokes it on.

## Consequences

### Positive

- The `signals → sessions` FK is satisfied by construction: the SDK
  writes under an id the drone has already seeded. 🔴-2 closed at its
  (second) root, complementing the emission fix.
- No drone change, no IPC-protocol change, no schema change, no new
  dependency — a composition-layer identity unification plus an
  assembled-app regression test. Fix-cycle-scoped, parallel to
  ADR-0012.
- Recovery/replay (spec §1b), audit, and the VDR/plan projectors now
  correlate signals to a real session row — the foundation M07's loop +
  recovery build on is no longer hollow in the assembled app.
- The cross-process run-identity invariant is now explicit and
  reviewable, preventing the next in-process subsystem from minting a
  divergent id.

### Negative

- Couples the SDK's run identity to drone-lifecycle availability: if the
  drone fails to spawn, no canonical `SessionId` is managed. This is
  acceptable in v0.1 because drone-spawn failure already aborts app
  startup (`src-tauri/src/main.rs` setup), so a command that needs the
  `SessionId` is never reachable without it.
- One canonical id per app run encodes the §0d single-session v0.1
  assumption into the identity surface (see future implications).

### Neutral / future implications

- **Multi-session (post-v0.1, §0d):** when the runtime supports
  concurrent sessions, run identity must derive from the active
  `SessionId` surface rather than a single app-startup-managed value;
  the drone-per-session model would seed one `sessions` row per session
  and the SDK would adopt the id of the session it runs under. That is
  an explicit post-v0.1 evolution and would itself be reviewed.
- The route-the-registry/identity-through-drone-IPC alternative
  (ADR-0012 Alternative A) remains the forward path if a later milestone
  needs strict single-writer / drone-minted identity; it is §11
  IPC-ADR-gated and explicitly out of v0.1.

## Alternatives Considered

### Alternative A: Seed a `sessions` row on first `WriteSignal` (or relax `foreign_keys`) drone-side

**Rejected because:** it is a `runtime-drone` change (the M06.5 scope
locks forbid it) and weakens referential integrity — recovery/replay
correlate `signals`/`snapshots`/`plans` to `sessions(id)`; auto-seeding
on any write, or dropping the FK, lets orphaned-session signals
accumulate and defeats the integrity the FK exists to enforce.

### Alternative B: SDK mints the id; add a `DroneCommand::OpenSession` IPC to create the `sessions` row

**Rejected because:** it is an IPC-protocol change (§11 ADR-gated in its
own right), expands a focused fix cycle, and inverts ownership — the
drone is already the process that seeds `sessions` at startup; having
the SDK drive session creation duplicates that responsibility. This is
the heavier forward path if strict drone-minted identity is ever
required (parallel to ADR-0012 Alternative A).

### Alternative C: Drop the `signals → sessions` foreign key

**Rejected because:** it is a schema change (new migration + §14 schema
versioning + ADR) and removes the referential guarantee recovery/replay
and the projectors rely on. The defect is a mismatched id, not an
over-strict constraint; removing the constraint masks the class of bug
instead of deciding the identity.

## Related

- Spec sections: §1b (recovery rebuilds history by session), §11
  (reconciliation / signal log), §0d (release scope — single-session
  v0.1)
- Prior ADRs: **ADR-0012 (sibling — single source-of-truth session DB
  *path*; this ADR is the identity analogue: single source-of-truth run
  *id*)**, ADR-0007 (drone as in-process system-of-record + the process
  that seeds the canonical `sessions` row), ADR-0010 (MCP dispatch — M07
  reads/correlates by the session this ADR makes canonical)
- Findings: `docs/M06-irl-findings.md` 🔴-2
- Code: `crates/runtime-drone/migrations/000_initial.sql:44`;
  `crates/runtime-drone/src/lib.rs:143`; `crates/runtime-drone/src/db.rs`
  (`PRAGMA foreign_keys=ON`); `src-tauri/src/drone_lifecycle.rs`
  (`sdk_session_id`); `src-tauri/src/main.rs` (managed-state
  registration); `src-tauri/src/commands.rs`
  (`run_smoke_session[_with]`)
- External: SQLite foreign-key enforcement + `ON CONFLICT` interaction
  (<https://www.sqlite.org/foreignkeys.html>)

## Notes

Status is **Accepted** in the M06.5 Stage B.fix PR before merge, per
CLAUDE.md §11 and the **same precedent as ADR-0012 accepted in Stage
A.fix**. This ADR records a decision whose *absence* was the second
(undiagnosed-by-the-phase-doc) root cause of 🔴-2; making the
cross-process run-identity invariant explicit is the structural
regression guard, complementary to the Stage B.fix assembled-app
`smoke_signal_persistence` test which models the corrected
shared-identity composition. Sibling to ADR-0012: same composition-layer
single-source-of-truth pattern (path there, identity here), same
fix-cycle scope discipline (no drone/IPC/schema change), same
Alternative-A forward path if strict drone-owned ownership is later
required.
