# Gap Analysis — Agent Runtime (Living Document)

> **APPEND-ONLY.** This file is the project's running ledger of code↔spec gaps,
> contradictions, ambiguities, open questions, and prioritized fix backlog.
>
> Per `CLAUDE.md` §20 Gap Analysis Protocol, **no prior entry may be edited,
> reordered, or deleted.** New milestones append a section at the bottom only.
> If a prior finding is later resolved, do NOT modify the original entry — add
> a status line to the current milestone's "Carry-forward" section that
> references the prior entry by milestone tag and states the resolution.
>
> Authored by Claude during the **Phase Closeout — Gap Analysis** stage of
> each parent milestone (Stage E in M01; the final stage of M02–M11). User
> reviews alongside the milestone PR. Approval gates the merge.
>
> CI enforces append-only via a diff check (added in M01 Stage D's CI workflow).

---

## How to use this document

- **What it is:** the cumulative quality + spec audit across all milestones to
  date. Every milestone's entry reviews the *whole* codebase and *whole* spec,
  not just what shipped this milestone.
- **What it isn't:** per-stage retrospective (those live in
  `docs/build-prompts/retrospectives/`). Retrospectives evaluate the build
  *process* — did the prompt-driven workflow work? This file evaluates the
  build *product* — does the code match the spec, what did the spec get
  wrong, what's the prioritized fix backlog?
- **When updated:** at the very end of every parent milestone, after Stage D
  (or final stage) commits, before the milestone PR is opened. The Phase
  Closeout CLI prompt instructs Claude to append the new entry per the
  template below.
- **Review:** user reviews the new entry alongside the milestone PR diff and
  the parent-milestone summary. Approval is required before the PR opens.
- **Where the carry-forward goes:** if M02 finds that an M01 fix-backlog item
  is now resolved (or was always wrong), M02's "Carry-forward" section says
  so. M01's entry is **never edited.** This preserves the audit trail.

---

## Entry template

Every milestone entry has six sections. If a section has nothing to report,
write **"None observed."** — do not omit the section.

```markdown
## M[NN] — <Title> (<YYYY-MM-DD>, commit `<sha>`)

> Author: Claude (per `CLAUDE.md` §20)
> Stages aggregated: M[NN].A through M[NN].<X>
> Reviewed against: agent-runtime-spec.md, schemas/*.v1.json, prior gap
> analysis entries (if any).

### Codebase deep dive

<200–500 words. Cumulative review of the code shipped to date — not just
this milestone. What's solid, what's notable, what surprised. Reference
specific files/modules. If something is structurally weak in a way that
will compound, name it here.>

### Adherence to spec

<For each area touched by this milestone, classify with file:line citations.>

- ✅ **<area>** — matches spec at `<spec section / file:line>` — cite code at
  `<crate/file.rs:line>`
- ⚠️ **<area>** — deviates from spec at `<spec section>` — code at
  `<crate/file.rs:line>` — reason: <one line>
- ❌ **<area>** — contradicts spec at `<spec section>` — code at
  `<crate/file.rs:line>` — resolution: <plan, with milestone tag>

### Spec review (forward-looking)

<Cumulative scan of the spec. Items here may surface from this milestone's
work or from re-reading prior sections with fresh eyes.>

- **Missing items:**
  - `<spec section>` — <what's missing> — surfaces in <future milestone>
- **Contradictions:**
  - `<spec section A>` vs. `<spec section B>` — <description> — recommend:
    <fix in next docs(spec) PR>
- **Ambiguity:**
  - `<spec section>` — <ambiguity> — Claude resolved this milestone by
    <choice>; should the spec lock the choice?
- **Open questions:**
  - <question that the spec doesn't answer; relevant to which milestone>
- **Recommended spec changes:**
  - `<file:section>` — <change> — rationale: <one line>

### Fix backlog

<Code AND spec fixes. Severity levels are non-elastic — if everything is
"important," the prioritization is meaningless.>

- 🔴 **Critical** (must fix before next milestone starts):
  - `<area>` — <fix> — owner: <code | spec> — at `<file:line>`
- 🟡 **Important** (should fix this release cycle, may queue for a dedicated
  prep session before a later milestone):
  - `<area>` — <fix> — owner: <code | spec>
- 🟢 **Nice-to-have** (queue for v1.0+ unless trivially incidental):
  - `<area>` — <fix>

### Carry-forward from prior milestones

<For every unresolved fix-backlog item from any prior milestone entry,
state current status. Do NOT modify the prior entry.>

- **M[prior NN] critical item "<name>"** — <status: resolved at
  `<file:line>` / still open / deferred to <milestone> with rationale>

<If this is M01, write "N/A — first milestone.">

### Sign-off

**Claude:** I have generated this gap analysis after the final stage of
M[NN]. This is my honest assessment of the cumulative code-vs-spec state.
User review pending. The PR remains undrafted until this entry is approved.

**Surfaced at:** <YYYY-MM-DD HH:MM TZ>
```

---

## Milestone entries

<!-- ============================================================ -->
<!-- New entries are appended below this line. Earliest first.    -->
<!-- DO NOT edit, reorder, or delete any entry below.             -->
<!-- ============================================================ -->

## Pre-M01 — Spec Prep Audit (2026-05-02, baseline commit `6688d41`)

> Author: Claude (per `CLAUDE.md` §20)
> Stages aggregated: pre-implementation prep only (no code stages yet)
> Reviewed against: `agent-runtime-spec.md`, `schemas/*.v1.json`,
> `docs/build-prompts/M01-foundation.md`, `CLAUDE.md`
> Context: spec audit conducted 2026-05-02 to verify M01 readiness.
> Resolved-pre-M01 items already landed in commits `f71ad9c` (spec
> cleanups + lefthook decision) and `6688d41` (gap-analysis protocol
> introduction). This entry establishes the baseline so the M01 Stage E
> entry has prior findings to carry forward against.

### Codebase deep dive

**N/A — no implementation code exists yet.** The repository contains:
the runtime spec (`agent-runtime-spec.md`), JSON schemas
(`schemas/*.v1.json`), two reference frameworks (`examples/aria/`,
`examples/ralph/`), OSS scaffolding (LICENSE, SECURITY, CONTRIBUTING,
.github/), `CLAUDE.md` runtime memory, MVP build checklist, four ADRs
(0001 archetype, 0002 Tauri, 0003 Engineering Charter, 0004 defer paid
code-signing), and the M01 milestone prompt staged into A/B/C/D/E.
No Cargo workspace, no Rust crates, no Tauri shell yet — those land
in M01.

This entry exists to record the baseline of known spec/process gaps
*before* implementation begins. The M01 entry (created in Stage E) will
re-evaluate after the first round of code↔spec contact and report
status on each outstanding item via its Carry-forward section.

### Adherence to spec

**N/A — no implementation to evaluate.** Spec adherence will first be
measurable at M01 Stage E, after the workspace, types, drone, and fuzz
harness ship.

### Spec review (forward-looking)

The 2026-05-02 audit reviewed 10 user-flagged items, 3 minor items, and
11 additional risk areas (IPC frame format, drone command surface,
snapshot DDL, recovery semantics, heartbeat parameters, MSRV pinning,
CI matrix concreteness, pre-commit hook decision, schema typify
cleanliness, `unsafe_code` lint placement, coverage delta gating).
Verification details are in the conversation transcript at
`session_01Yb2a1gERV6rv5evYpj7c7d`.

#### Resolved before M01 (in commits `f71ad9c` + `6688d41`)

- ✅ `agent-runtime-spec.md` line 470 (§0c stack table, MCP client row) —
  unresolved "rmcp or direct JSON-RPC" → rmcp primary, JSON-RPC fallback
  if rmcp gap surfaces; final decision deferred to M06 prep.
- ✅ `agent-runtime-spec.md` line 660 (§1c drone-per-session) —
  `child_process.fork('drone.ts', ...)` Node.js leftover →
  `tokio::process::Command::new("runtime-drone").args(...)`.
- ✅ `agent-runtime-spec.md` line 584 (§1d drone IPC) — `StopReason`
  TypeScript union syntax inside Rust code block → proper Rust enum
  with `#[serde(rename_all = "snake_case")]`.
- ✅ `agent-runtime-spec.md` ~line 2282 (§9 Phase 9 prose) — said
  "Defers to v1.0" but §0d table marks Phase 9 ✅ in v0.1; resolved to
  "Ships in v0.1 per §0d release scope (Workbench-MVP)".
- ✅ `agent-runtime-spec.md` ~lines 2336–2412 (§11 Persistence Layer
  DDL) — `signals` table existed only in §2b, `heartbeats` was missing
  entirely, VDR's `signal_ids` and `context_type` were appended via
  §2b ALTER. Consolidated: `signals` + 3 indexes and `heartbeats` +
  1 index added to main DDL; VDR columns inlined; §2b ALTER lines
  replaced with a pointer.
- ✅ `agent-runtime-spec.md` line 2690 (§11 degraded modes matrix) —
  "keytar throws" (Node.js library) → "`keyring` crate returns
  `Error::PlatformFailure`" (the actual cross-platform Rust dep).
- ✅ `agent-runtime-spec.md` ~line 2540 (Project Structure schemas/
  listing) — missing `common.v1.json` (which exists and is correctly
  listed in `schemas/README.md`) added.
- ✅ `CLAUDE.md` §12 — pre-commit hook "TBD lefthook or pre-commit
  framework" resolved to **lefthook** (single Go binary, no Python
  dependency); install command updated to `lefthook install`;
  `lefthook.yml` stub added to M01 Stage A's Files-to-Change with
  `cargo fmt --check` + `cargo clippy` glob on `*.rs`.
- ✅ Gap-analysis protocol absent — added `CLAUDE.md` §20 (Hard Rule:
  prior entries immutable), this living document, Stage E template in
  `TEMPLATE.md` and full Stage E in `M01-foundation.md`. CI append-only
  enforcement gate is part of Stage E (lands alongside this entry's
  successor).

#### Still outstanding (deferred — none block M01)

- 🟢 **§10 numbering gap** — spec jumps from §9 (Phase 9) to §11
  (Reconciliation). Cosmetic. Recommendation: file in a `docs(spec):`
  cleanup PR after M01 lands.
- 🟡 **Phase 3 React Flow specifics** — `agent-runtime-spec.md`
  ~lines 1150–1188 names node types but gives no React Flow API
  specifics or state management library choice. `docs/MVP-v0.1.md` §M3
  fills this in ("React + React Flow + Zustand"). Surfaces at M03
  prep; the spec text itself should be expanded at that time.
- 🟡 **Session lifecycle FSM diagram** —
  `agent-runtime-spec.md` §11 (`sessions.status` column) lists six
  values (`active | suspended | complete | crashed | recovered |
  budget_exceeded`) but no transition diagram. State machine logic is
  M04+ work; spec diagram should land at that time.
- 🟡 **Windows named pipe API specifics** —
  `agent-runtime-spec.md` §1d details Unix domain sockets but leaves
  Windows named pipe path format (`\\.\pipe\<name>`), security
  descriptors, and `ServerOptions` API implicit. v0.1 is Windows-only.
  Tokio's `windows::named_pipe` module is well-documented so M01
  Stage C can proceed via implementer lookup, but a spec subsection
  would prevent drift across future implementations. Address during
  M01 Stage C (as an inline implementer note in the drone module) and
  fold the resolved details back into a `docs(spec):` PR after M01.

#### Anticipated M01 friction (not gaps; pre-flagged)

- 🟡 **typify `oneOf` verbosity in Stage B** —
  `schemas/common.v1.json::HookRef` (3-variant `oneOf`) and
  `schemas/framework.v1.json::sizing` (2-variant `oneOf`) generate
  verbose Rust enums via typify. Compiles cleanly, but clippy
  pedantic/nursery may complain about generated code. The
  `#[allow(clippy::pedantic, clippy::nursery)]` header on
  `crates/runtime-core/src/generated/` files (already specified in
  `M01-foundation.md` line 626) suppresses noise. Expect 1–2
  self-correction rounds reconciling typify output during Stage B.

### Fix backlog

- 🔴 **Critical** (must fix before M01 Stage A starts): **None.**
  All blockers were resolved before this entry was written.

- 🟡 **Important** (should fix this release cycle):
  - **Coverage delta gating mechanism** — owner: docs / `CLAUDE.md` §5.
    `CLAUDE.md` says "Coverage drops vs prior `main` block PR merge"
    but no concrete `cargo-llvm-cov` diff invocation or threshold-delta
    script. M01 uses absolute thresholds (100% drone, ≥80% workspace)
    so this doesn't bite M01. Becomes relevant from M02 when `main`
    has a coverage baseline. **Define the mechanism in `CLAUDE.md` §5
    before M02 Stage A starts.**
  - **Phase 3 spec expansion** — owner: spec / `agent-runtime-spec.md` §3.
    Add React Flow node type specifics + Zustand state management
    discussion at M03 prep.
  - **Session FSM diagram** — owner: spec / `agent-runtime-spec.md`
    §11 sessions table. Add transition diagram at M04 prep when state
    machine logic lands.
  - **Windows named pipe spec subsection** — owner: spec /
    `agent-runtime-spec.md` §1d. Add path format / security descriptor
    / `ServerOptions` API detail. Address inline during M01 Stage C
    and fold back into a `docs(spec):` PR after M01.
  - **typify `oneOf` clippy suppression confirmation** — owner: code /
    M01 Stage B. Confirm the generated-file `#[allow(...)]` header
    suppresses pedantic/nursery on the `HookRef` and `sizing` enums;
    if it doesn't, surface as a friction item in the M01 Stage E
    entry's Adherence section.

- 🟢 **Nice-to-have** (queue for v1.0+ unless trivially incidental):
  - **§10 numbering gap** (`agent-runtime-spec.md`) — cosmetic; fix
    in a `docs(spec):` cleanup PR.

### Carry-forward from prior milestones

**N/A — first entry.** M01's Stage E entry will carry forward this
backlog and report status on each Important and Nice-to-have item.

### Sign-off

**Claude:** This is the baseline pre-implementation entry, established
2026-05-02. It records what the spec audit found and resolved before
M01 Stage A begins, and what remains outstanding with target
milestones. Per `CLAUDE.md` §20 this entry is **immutable** once
committed; future milestones report status changes via their
Carry-forward sections. M01 Stage E will be the next entry, after the
workspace + types + drone + fuzz harness ship.

**Surfaced at:** 2026-05-02 (UTC).

---

## Pre-M01 Addendum — Backlog Carry-Forward Seeds (2026-05-02, prior commit `e24fa58`)

> Author: Claude (per `CLAUDE.md` §20)
> Stages aggregated: pre-implementation prep (addendum to Pre-M01 entry above)
> Reviewed against: `CLAUDE.md` §5 (TDD), §9 (Style + anti-patterns)
> Context: User surfaced two backlog items after the Pre-M01 baseline
> entry was committed. Per §20 the prior entry is immutable; this
> addendum is the append-only way to record them so they cannot be
> forgotten or overlooked.

### Codebase deep dive

**N/A — addendum, not a new milestone.** No new code review since the
Pre-M01 baseline entry above.

### Adherence to spec

**N/A — no implementation yet.**

### Spec review (forward-looking)

None observed beyond the Pre-M01 baseline entry. This addendum adds
two backlog items only.

### Fix backlog

- 🔴 **Critical:** None.

- 🟡 **Important:**
  - **Reuse-first vs. duplication-first bias** — owner: docs / `CLAUDE.md` §9.
    Currently §9 reads: *"Premature abstraction. Three similar lines is
    better than a wrong abstraction. Wait for the fourth before
    extracting."* This is a duplication-first bias appropriate for early
    milestones (M01–M06) when surface area is small and abstractions
    risk being wrong. **Decision: leave §9 as-is for now.** **Revisit at
    M07–M08** when there's enough surface area to make abstractions
    defensible. At that time, evaluate whether to amend §9 to a
    reuse-first preference. Do NOT make this change preemptively;
    record the revisit point here so it isn't forgotten.
  - **UI consistency: existing look and feel** — owner: code / M03 prompts
    (frontend lands). All modals and screens in M03 onward must reuse
    existing component patterns and visual language — no per-feature
    re-skinning, no new dialog primitives where existing ones fit.
    M03's stage prompts should embed this as an explicit constraint
    when the milestone is authored. Carry forward into M03 prep.

- 🟢 **Nice-to-have:** None.

### Carry-forward from prior milestones

- **Pre-M01 baseline entry** — all items remain as recorded. This
  addendum adds two items above; nothing in the prior entry is
  modified.

### Sign-off

**Claude:** This addendum captures two backlog items the user surfaced
after the Pre-M01 baseline was committed. Both target later milestones
(§9 revisit at M07–M08, UI consistency at M03). Per `CLAUDE.md` §20
the prior Pre-M01 entry is immutable; this is the append-only way to
extend the backlog. M01 Stage E will carry both items forward and
report status.

**Surfaced at:** 2026-05-02 (UTC).

---

## M01 — Foundation (2026-05-02, commit `6bc8d28`)

> Author: Claude (per `CLAUDE.md` §20)
> Stages aggregated: M01.A (workspace), M01.B (types), M01.C (drone), M01.D (fuzz + polish)
> Reviewed against: `agent-runtime-spec.md` §1 §1b §1c §1d §2 §11 §12,
> `schemas/*.v1.json`, M01.A–D retrospectives + `M01-summary.md`.

### Codebase deep dive

M01 lands four real things on top of the empty-repo baseline: a Cargo
workspace with five member crates plus a Tauri stub
(`crates/{runtime-core, runtime-main, runtime-drone, runtime-sandbox,
xtask}` + `src-tauri/`); a typify-driven type-generation pipeline
(`cargo xtask regenerate-types`) that produces ~22k lines of generated
Rust under `crates/runtime-core/src/generated/` from
`schemas/*.v1.json`, plus three hand-curated modules
(`event.rs::AgentEvent` 30+ variants, `drone.rs::{DroneEvent,
DroneCommand}`, `error.rs::RuntimeError`); a Phase-1 drone
(`crates/runtime-drone/src/{db, snapshot, heartbeat, ipc,
command_handler, shutdown, lib, main}.rs`) implementing heartbeat,
append-only snapshot writer with SHA-256 `state_hash`, SQLite WAL with
the four pragmas in spec order, framed-JSON IPC over Unix domain
socket / Windows named pipe via `LinesCodec`, and SIGTERM/SIGINT/
CTRL_BREAK/CTRL_C emergency-snapshot handling; and a `cargo-fuzz`
harness on `drone_command_decode` with a 6-seed corpus + a CI fuzz-
smoke job + a nightly 1h fuzz workflow.

What's solid: the schemas-as-source-of-truth pipeline (drift-detected
in CI), the `*_with` / `*_inner` test-seam pattern in `runtime-drone`'s
`lib.rs`/`shutdown.rs` (lifts coverage from 87% to 95%+ without leaking
implementation surface), the dual-gate coverage policy
(workspace ≥80% + drone safety primitive ≥95% with documented
OS-signal-orchestrator exclusions), property tests on serde round-trip
across `AgentEvent`/`DroneEvent`/`DroneCommand`, and the per-crate
READMEs that document IPC framing, SQLite schema, and platform notes
inline. Hard gates G1–G5 cleared 4/4 stages.

What's structurally weak and likely to compound: (1) `runtime-drone`'s
`db.rs` schema initialization implements 7 of 8 spec §11 tables;
`mcp_servers` is deferred (deliberate — MCP work is M06+) but the
omission isn't documented at the call site. (2) The drone module
decomposition diverges from spec §"Project Structure" line 2515
(`protocol.rs` / `recovery.rs` / `process_manager.rs`) — the actual
shape (`db.rs` / `heartbeat.rs` / `ipc.rs` / `command_handler.rs` /
`snapshot.rs` / `shutdown.rs`) is more granular and clearer, but the
spec's illustrative listing is now wrong. (3) `DroneCommand::SnapshotNow`
in `runtime-core/src/drone.rs:72-77` extends spec §1d's `{reason}` to
`{reason, state_json}` — necessary because the drone has to know what
state to snapshot, but the spec is underspecified here and the
implementation diverges silently.

What surprised: the typify-emitted volume (~22k lines across 5 files)
required broadening the generated-file `#[allow(...)]` header to
six lints; the cross-platform 100% coverage gate on a safety primitive
is structurally infeasible without either firing real OS signals
cross-platform or refactoring the public `run()` API to accept a
signal source as a parameter — codified as the dual-gate ≥95% + OS-
signal-exclusion policy in commit `1dec4ba`.

### Adherence to spec

- ✅ **Workspace + crate layout** — matches spec §"Project Structure"
  for top-level shape (workspace root + `crates/{runtime-core,
  runtime-main, runtime-drone, runtime-sandbox}` + `src-tauri/` +
  `xtask/`) — `Cargo.toml`, `crates/*/Cargo.toml`. Five member crates
  exist; only `runtime-core` and `runtime-drone` carry real logic in M01
  (the others are placeholder stubs per the milestone scope).
- ✅ **SQLite WAL pragmas in spec §1c order** — code at
  `crates/runtime-drone/src/db.rs:39-48` issues `journal_mode=WAL`,
  `synchronous=NORMAL`, `busy_timeout=5000`, `foreign_keys=ON` in the
  exact order spec §1c lines 675-680 require. Verified by unit test
  `pragmas_set_in_correct_order` at `db.rs:157-180`.
- ✅ **Append-only snapshot writer with SHA-256 `state_hash`** —
  spec §1 lines 518-522 + §11 `snapshots` table.
  `crates/runtime-drone/src/snapshot.rs::write` computes
  `sha256(state_json)` and inserts a fresh row per call (no UPDATE
  path).
- ✅ **5s heartbeat interval** — spec §1 line 513
  ("every 5 seconds"); `crates/runtime-drone/src/lib.rs:22`
  (`HEARTBEAT_INTERVAL = Duration::from_secs(5)`).
- ✅ **Framed JSON-newline IPC over Unix domain socket / Windows
  named pipe** — spec §1d lines 728-757; code at
  `crates/runtime-drone/src/ipc.rs:68-119` uses
  `tokio_util::codec::LinesCodec` with `cfg(unix)` /
  `cfg(windows)` accept loops.
- ✅ **SIGTERM/SIGINT/CTRL_BREAK emergency-snapshot handler** —
  spec §1 lines 530-533; code at
  `crates/runtime-drone/src/shutdown.rs` writes a final snapshot
  before clean exit; integration-tested via the Unix subprocess
  test at `crates/runtime-drone/tests/integration.rs`.
- ✅ **Schemas as source of truth** — spec §12 line 2799 + `CLAUDE.md`
  §14. `crates/runtime-core/src/generated/{agent, common, framework,
  skill, tool}.rs` are typify-emitted from `schemas/*.v1.json` via
  `cargo xtask regenerate-types`; CI drift check at
  `crates/xtask/tests/check_drift.rs`.
- ✅ **Workspace lints (`forbid(unsafe_code)`, clippy
  pedantic+nursery, deny warnings)** — spec §12 lines 2760-2762;
  configured at `Cargo.toml` workspace root with sandbox override.
- ✅ **DCO sign-off + Apache 2.0 + Conventional Commits** — every
  M01 commit signed (`Signed-off-by:`) and conforms to
  Conventional Commits (`feat(scope): …` / `docs(scope): …`).
- ⚠️ **`DroneCommand::SnapshotNow` field set** — spec §1d line 574
  declares `SnapshotNow { reason: String }`; code at
  `crates/runtime-core/src/drone.rs:72-77` declares
  `SnapshotNow { reason, state_json: serde_json::Value }`. The
  `state_json` field is necessary because the drone needs the state
  payload to snapshot — but the deviation is silent. Resolution:
  fold `state_json` into spec §1d in the post-M01 `docs(spec):` PR
  (carry-forward to M02).
- ⚠️ **`DroneEvent::SnapshotWritten` field set** — spec §1d line 563
  declares `SnapshotWritten { snapshot_id, session_id }`; code at
  `crates/runtime-core/src/drone.rs:20-29` adds `reason` and
  `timestamp`. Useful for debugging and the dashboard, but a silent
  extension. Resolution: fold into the same `docs(spec):` PR.
- ⚠️ **`DroneEvent::Heartbeat.status` type** — spec §1d line 562
  declares `status: HeartbeatStatus` (a typed enum) but the spec
  never defines `HeartbeatStatus`. Code at
  `crates/runtime-core/src/drone.rs:13-18` uses `String`.
  Resolution: define `HeartbeatStatus` as
  `enum { Ok, Degraded, Stalled }` in the spec (matches the
  `heartbeats.status` text values written by `heartbeat::run`) and
  promote the type in `runtime-core` in M02 prep.
- ⚠️ **§11 `mcp_servers` table missing from drone schema init** —
  spec §11 lines 2435-2444 lists `mcp_servers` as one of 8 tables;
  code at `crates/runtime-drone/src/db.rs:50-141` creates 7
  (`sessions, snapshots, signals, heartbeats, vdr, token_usage,
  skills`). MCP work lands in M06, so deferral is reasonable, but
  the omission isn't documented at `db.rs::init_schema`.
  Resolution: M02 Stage A decides whether to add the
  `mcp_servers` table now (fields are stable; no MCP code yet) OR
  document the deferral inline at `db.rs::init_schema` with a
  `// SPEC §11: mcp_servers deferred to M06; not used by Phase 1`
  comment. No action in M01 Stage E.
- ⚠️ **`runtime-drone` module decomposition diverges from spec
  §"Project Structure"** — spec line 2515 lists
  `protocol.rs / heartbeat.rs / snapshot.rs / recovery.rs /
  process_manager.rs`; M01 ships
  `db.rs / heartbeat.rs / snapshot.rs / ipc.rs / command_handler.rs /
  shutdown.rs / lib.rs / main.rs`. Implementation shape is clearer
  (single responsibility per module) but the spec's illustrative
  listing is now stale. Resolution: update spec §"Project Structure"
  drone listing in the post-M01 `docs(spec):` PR; the spec's
  illustrative tree was always advisory but should match
  reality before M02.
- ⚠️ **Spec §1d `JsonCodec<T>::new()` pseudo-code** — spec lines
  745-747 show `FramedRead::new(read, JsonCodec::<DroneEvent>::new())`
  but `tokio_util::codec` provides no such codec; code at
  `crates/runtime-drone/src/ipc.rs:32-39` uses `LinesCodec` with
  manual `serde_json::to_string` / `from_str`. Already captured in
  M01.C retro; tracked here as the spec-side fix.
- ⚠️ **Spec §"Project Structure" `runtime-core/src/{capability,
  signal}.rs`** — listed at spec lines 2493-2494 but not present in
  M01; capability lands in M05+, signal in M02. Reasonable phase
  deferral, but the spec's illustrative listing implies all four
  files exist from the start. Resolution: add a "files marked '✱'
  arrive in their owning phase" annotation to spec §"Project
  Structure" in the post-M01 `docs(spec):` PR.
- ❌ **None observed.** No outright contradictions where code
  ships behavior that spec forbids or vice versa.

### Spec review (forward-looking)

- **Missing items:**
  - `agent-runtime-spec.md` §1d line 562 — `HeartbeatStatus` enum is
    referenced by `DroneEvent::Heartbeat` but never defined. Define
    as `{ Ok, Degraded, Stalled }` matching the text values
    `heartbeat::run` writes to the `heartbeats.status` column.
    Surfaces in M02 (event pipeline consumes `DroneEvent`).
  - `agent-runtime-spec.md` §1d line 745 — `JsonCodec<T>` is a
    pseudo-codec; replace with `LinesCodec` + manual JSON pattern as
    actually shipped at `crates/runtime-drone/src/ipc.rs:32-39`.
    Surfaces in any future fresh-session reading of §1d.
  - `agent-runtime-spec.md` §"Project Structure" lines 2515-2520 —
    drone module listing is stale; current shape is `db, heartbeat,
    snapshot, ipc, command_handler, shutdown, lib, main`.
  - `agent-runtime-spec.md` §1 / §1d — no explicit definition of
    what `state_json` should contain when main calls
    `SnapshotNow { reason, state_json }`. Currently main is
    responsible for serializing the full session state into a
    `serde_json::Value`; no schema. M04 (session lifecycle) will
    need to lock this; surfaces there.
- **Contradictions:** None observed.
- **Ambiguity:**
  - `agent-runtime-spec.md` §1c line 670 — "v1 caps at 8 concurrent
    active sessions and queues additional requests" — surfaces in
    M04 session-lifecycle work. M01 doesn't enforce this; the drone
    is per-session and main has no session table semantics yet.
- **Open questions:**
  - Cross-platform integration test for the drone subprocess
    lifecycle. The current integration test at
    `crates/runtime-drone/tests/integration.rs` is `#[cfg(unix)]`;
    Windows has no equivalent (named-pipe handling differs enough
    to warrant a separate test). v0.1 is Windows-only per §0d, so
    this gap matters. Surfaces in M02 prep — adding a Windows
    integration test is straightforward and would lift Windows
    coverage on `ipc.rs` `cfg(windows)` paths.
  - When M02's `AnthropicProvider` adds a long-lived SSE loop, the
    same `_with` / `_inner` test-seam pattern that lifted drone
    coverage from 87% to 95% should be applied; document this in
    `CLAUDE.md` §9 before M02 Stage A.
- **Recommended spec changes:** Bundled in the post-M01
  `docs(spec):` cleanup PR (target: open before M02 Stage A
  begins) — items above on `HeartbeatStatus`,
  `SnapshotNow.state_json`, `SnapshotWritten.{reason, timestamp}`,
  `JsonCodec`→`LinesCodec`, drone module listing,
  `runtime-core/src/{capability, signal}.rs` annotation, plus the
  Pre-M01-baseline §10 numbering cosmetic gap and Windows named-pipe
  inlined details from `crates/runtime-drone/src/ipc.rs:13-30`.

### Fix backlog

- 🔴 **Critical** (must fix before M02 starts): **None.** All M01
  acceptance criteria are met; no shipped behavior is incorrect.
  Spec deviations on `DroneCommand::SnapshotNow.state_json` and the
  `mcp_servers` table omission are forward-looking issues, not
  defects.

- 🟡 **Important** (should fix this release cycle):
  - **`mcp_servers` table — add now or document the deferral.**
    Owner: code / `crates/runtime-drone/src/db.rs:50-141`. M01
    creates 7 of 8 spec §11 tables; `mcp_servers` is missing.
    Two options: (a) add the table now in M02 Stage A (fields
    are stable per spec §11 lines 2435-2444) to ship the full
    §11 schema and avoid a migration in M06; (b) document the
    deferral inline at `init_schema` with a
    `// SPEC §11: mcp_servers deferred to M06` comment. M01
    Stage E does not pick — decision moves to M02 Stage A.
  - **Post-M01 `docs(spec):` PR.** Owner: spec /
    `agent-runtime-spec.md` §1d + §"Project Structure" + §10.
    Bundle the spec changes recommended above (HeartbeatStatus
    definition; SnapshotNow/SnapshotWritten field extensions;
    JsonCodec pseudo-code → LinesCodec + manual JSON; drone module
    listing; capability/signal annotations; Windows named-pipe
    details from `ipc.rs` module docs; §10 numbering cosmetic).
    Open before M02 Stage A so the spec is the contract M02 reads.
  - **Coverage delta gating mechanism.** Owner: docs / `CLAUDE.md`
    §5. Pre-M01 carry-forward; still open. M01 used absolute
    thresholds (workspace ≥80%, drone ≥95%) per the M01.C
    codification commit `1dec4ba`. Becomes relevant from M02 when
    `main` accumulates a coverage baseline. Define the
    `cargo-llvm-cov` diff invocation + threshold-delta script in
    `CLAUDE.md` §5 before M02 Stage A.
  - **`*_with` / `_inner` test-seam pattern.** Owner: docs /
    `CLAUDE.md` §9. Document as the canonical TDD-friendly
    approach to OS-signal-driven async functions; it lifted drone
    coverage from 87% → 95% and applies again to
    `AnthropicProvider`'s SSE loop in M02. Cite
    `crates/runtime-drone/src/{lib.rs, shutdown.rs}` as the
    archetype.
  - **Phase 3 React Flow + Zustand spec expansion.** Owner: spec /
    `agent-runtime-spec.md` §3. Pre-M01 carry-forward; still open.
    Address at M03 prep.
  - **Session FSM diagram.** Owner: spec /
    `agent-runtime-spec.md` §11 sessions table. Pre-M01
    carry-forward; still open. Address at M04 prep.
  - **UI consistency: existing look and feel.** Owner: code / M03
    prompts. Pre-M01 addendum carry-forward; still open. M03 stage
    prompts must embed this as an explicit constraint.
  - **Reuse-first vs duplication-first §9 bias revisit.** Owner:
    docs / `CLAUDE.md` §9. Pre-M01 addendum carry-forward;
    deferred to M07–M08 per the addendum decision. No M01 action.
  - **Windows drone integration test.** Owner: code /
    `crates/runtime-drone/tests/`. The current integration test is
    `#[cfg(unix)]`; v0.1 is Windows-only per §0d. Add a
    `tests/integration_windows.rs` (or `cfg`-gate the existing one
    to both platforms) to exercise `ipc::accept_loop` on
    `cfg(windows)`. Lifts the `ipc.rs` 84.70% Windows-platform
    coverage and exercises the named-pipe path. Address in M02
    prep alongside the SSE loop test seam.
  - **`.gitattributes` line-ending normalization.** Owner: code /
    `.gitattributes`. M01.D session start observed seven generated
    files showing `LF will be replaced by CRLF` warnings on
    Windows checkout (content-identical, tooling artifact only).
    Add `*.rs text eol=lf` and `*.json text eol=lf` to
    `.gitattributes` so future Windows fresh sessions don't have
    to re-confirm the no-content-diff invariant. Trivially
    incidental — could move to 🟢 if not bundled.

- 🟢 **Nice-to-have** (queue for v1.0+ unless trivially incidental):
  - **§10 numbering gap** (`agent-runtime-spec.md`) — Pre-M01
    carry-forward. Cosmetic; bundle into the post-M01 `docs(spec):`
    PR.
  - **Tauri release-build caching.** Tauri release builds take
    3.5+ minutes for placeholder crates (444 transitive deps).
    `Swatinem/rust-cache@v2` is in CI; if M02+ builds get slower,
    consider a Tauri-skipping CI lane for code-only changes.
    Track but don't act unless friction surfaces.

### Carry-forward from prior milestones

- **Pre-M01 baseline `f71ad9c`+`6688d41` "Resolved before M01"
  items** — all 9 items remain resolved as recorded in the
  Pre-M01 entry; no regressions observed in M01 work.
- **Pre-M01 baseline 🟡 "Coverage delta gating mechanism"** —
  **still open.** M01 used absolute thresholds; the delta
  mechanism is not yet defined. Address before M02 Stage A.
  Re-listed in M01's 🟡 backlog above.
- **Pre-M01 baseline 🟡 "Phase 3 spec expansion"** — **still open,
  pre-M03.** Re-listed in M01's 🟡 backlog above.
- **Pre-M01 baseline 🟡 "Session FSM diagram"** — **still open,
  pre-M04.** Re-listed in M01's 🟡 backlog above.
- **Pre-M01 baseline 🟡 "Windows named pipe spec subsection"** —
  **partially resolved at the code level.** Implementer notes
  shipped at `crates/runtime-drone/src/ipc.rs:13-30` (path
  format, `ServerOptions` defaults, security descriptor) and
  documented in `crates/runtime-drone/README.md` "Platform-specific
  notes". Spec rebase still open — bundled into the post-M01
  `docs(spec):` PR.
- **Pre-M01 baseline 🟡 "typify `oneOf` clippy suppression
  confirmation"** — **resolved at M01.B.** The generated-file
  `#[allow(clippy::pedantic, clippy::nursery, clippy::all,
  missing_docs, unused_imports, rustdoc::invalid_html_tags)]`
  header successfully suppresses pedantic/nursery; `cargo clippy
  --workspace --all-targets -- -D warnings` returns clean across
  all 5 generated files. See
  `crates/runtime-core/src/generated/*.rs` headers.
- **Pre-M01 baseline 🟢 "§10 numbering gap"** — **still open,
  cosmetic.** Re-listed in M01's 🟢 backlog above.
- **Pre-M01 addendum 🟡 "Reuse-first vs duplication-first bias"** —
  **deferred to M07–M08 per the addendum decision.** No M01
  action.
- **Pre-M01 addendum 🟡 "UI consistency: existing look and
  feel"** — **carry forward into M03 prep.** No M01 work touched
  the renderer (the renderer doesn't exist yet). Re-listed in
  M01's 🟡 backlog above.

### Sign-off

**Claude:** I have generated this gap analysis after the final
implementation stage of M01 (Stage D commit `6bc8d28`). This is my
honest assessment of the cumulative code-vs-spec state. Hard gates
G1–G5 cleared in all four stages; no Critical-severity findings.
Forward-looking spec gaps and the `mcp_servers` deferral are the
highest-priority M02-prep items. User review pending; per
`CLAUDE.md` §20 this entry is immutable once committed — future
milestones report status updates via their Carry-forward sections.

**Surfaced at:** 2026-05-02 (UTC).

---

## M02 — Event Pipeline (2026-05-04, commit `4bd809a`)

> Author: Claude (per `CLAUDE.md` §20)
> Stages aggregated: M02.A (build hygiene + scaffolding), M02.B (LLMProvider trait + AnthropicProvider stub), M02.C (real HTTP+SSE), M02.D (AgentSdk + drone IPC client + event translation), M02.E (Tauri shell + skeleton renderer + frontend CI + Playwright).
> Reviewed against: `agent-runtime-spec.md` §0–§0d, §1, §1c, §1d, §2, §2a, §2b, §2c, §10, §11, §12, §13; `schemas/*.v1.json`; M02.A–E retrospectives + `M02-summary.md`; M01 entry's Carry-forward backlog (Pre-M01 baseline + Pre-M01 addendum + M01 itself).

### Codebase deep dive (cumulative — M01 + M02)

M02 turns the M01 foundation into a live event pipeline. New crates and surface:

- **Provider abstraction** (`crates/runtime-main/src/providers/{mod,anthropic,anthropic_sse}.rs`). The `LLMProvider` trait at `mod.rs:357` is async + `Send + Sync`, returns `BoxStream<'_, ProviderEvent>` from `stream`, exposes `count_tokens` / `list_models` / `estimate_cost`. Real HTTP+SSE in `anthropic.rs` (excluded from the ≥95% gate as a real-network OS-call wrapper); wire-format state machine in `anthropic_sse.rs::stream_events` (98.33% coverage, exercised by 12 unit tests + 8 wiremock tests). Cache-aware `CostBreakdown` (5m write 1.25× / 1h write 2.0× / read 0.1×) plumbed through `estimate_cost` so M04 budget integration only swaps the data source. The trait surface bends cleanly for v1.0+ multi-provider — `name() -> &str`, `supports() -> ProviderSupport`, no Anthropic-specific shapes leak through.

- **AgentSdk + drone IPC client** (`crates/runtime-main/src/{sdk/{mod,agent_sdk,event_pipeline,decision_extractor}, drone_ipc/{mod,client,connection}}.rs`). `AgentSdk<P: LLMProvider>` is generic over provider for v1.0+ (deliberately not `Box<dyn>` — documented decision). `EventPipeline` translates `ProviderEvent → AgentEvent` with consecutive-`TextDelta` bundling. `DecisionRecord` heuristic extracts `Decision:`/`Rationale:`/`Tool used:` markers (line-by-line, last-pair-wins; M04 verify+rails replaces with structured emitter). `DroneClient::send` uses 5-attempt 200ms→1.6s exponential backoff; `Connection::from_streams` is the testable `*_with` seam (unit-tested via `tokio::io::duplex`). The production `Connection::open` is excluded as the OS-call holdout. Cancellation safety: 5 explicit cancellation tests cover stream-mid-burst, mid-tool-use, mid-snapshot drops; no orphan tasks observed.

- **Tauri shell + renderer** (`src-tauri/src/{main,commands}.rs`, `src-tauri/capabilities/default.json`, `src/{App.tsx,components/*,lib/*,types/agent_event.ts}`, `crates/runtime-main/src/key_store.rs`). Capability set is locked down to `core:default` + `core:event:{default,allow-listen,allow-emit}` only — no `shell:*`, `fs:*`, `http:*`, `dialog:*`. The renderer never holds the API key, never speaks HTTP, never touches the filesystem; every privileged action goes through `#[tauri::command]`. `commands.rs::run_smoke_session_with` follows the M01.C / M02.C / M02.D `*_with` archetype; `run_smoke_session` is the production wrapper. `key_store.rs` wraps `keyring 3.6` + `secrecy 0.10`; reads return `SecretString` so the key never `Debug`-prints. Renderer is React 18 + TS 5.6 strict + Vitest 2.1 + Playwright 1.48 + ESLint 9 flat-config + Prettier 3.

- **runtime-core extensions.** `event.rs::AgentEvent` extended with `ToolSource { Builtin, Mcp, Generated }` enum, `AgentSpawned.session_id` field, `ToolInvoked.{source, server}` fields. `signal.rs` scaffolds `Signal` + 8 variants per spec §2b Signal Schema v2 plus `ContextType` + `PreSignalId`/`ParentSignalId`/`RetryOfSignalId` newtypes (no signals are emitted yet — emission integration lands in M04).

- **runtime-drone extensions** (M02.A). Eighth table `mcp_servers` added with a richer 22-field schema covering transport (CHECK-constrained: `stdio | http | sse | streamable_http`), stdio-mode fields (command/args_json/env_json), remote-mode fields (url/headers_json), auth (kind/token_ref/oauth_state_json — keychain refs, never literals), lifecycle (status/last_error/last_connected_at/retry_count), timeouts, scope tracking, capability caching, and a SQL-level mutual-exclusion CHECK enforcing the stdio-vs-remote invariant. `HeartbeatStatus` promoted from `String` to typed enum `{ Ok, Degraded, Stalled }`. New `DroneCommand::GracefulShutdown` plumbed through `command_handler` → `run_inner`'s combined shutdown source via `oneshot` (required for the Windows IPC integration test).

What's solid: provider abstraction's trait surface (no Anthropic-specific shapes leak); `*_with` test-seam pattern proven across four substrates (named pipes, HTTP+SSE, in-process streams, Tauri commands); coverage delta gating via Codecov is now required-on (project + patch thresholds in `codecov.yml`); zero-telemetry boundary held (no analytics deps, no crash reporter, no phone-home — verified at every audit step); capability lockdown at the Tauri allowlist boundary held end-to-end; subprocess-fixture path-resolution via `current_exe()` (avoids the `cargo llvm-cov --workspace` cross-target-dir trap).

What's structurally weak and likely to compound: (1) **`src/types/agent_event.ts` is hand-mirrored** from `runtime_core::AgentEvent` — at M02 this is 10 variants + `ToolSource` enum and the schema is currently stable, but the M02.D `ToolSource` + `AgentSpawned.session_id` additions would have silently drifted the TS side under any pressure. M03 codegen target. (2) **`signal.rs::ContextType` enum diverges from spec §2b's documented set.** The shipped variants are `AgentLoop / SkillLoad / ToolInvoke / HookExecute / PlanCreate / HitlPrompt / SessionLifecycle`; spec §2b lines 1071-1072 documents `skill | framework | code | search | verify | commit | subagent`. Different domains — the shipped enum is operation-context-shaped; the spec's is artifact-source-shaped. M04 emission integration must reconcile (either rename the runtime variants to match spec or update the spec). (3) **M02.E ships renderer-level Playwright only**; full desktop-shell E2E (`tauri-driver` + WebdriverIO + Linux+Windows matrix) is a M03 carry-forward — four `test.skip()`-with-rationale tests in `tests/e2e/smoke.spec.ts` mark the holdout. (4) **vitest threshold not enforced** — the 80% threshold in `vitest.config.ts` only triggers when `--coverage` is passed; the default `npm run test` runs without coverage. M03 prep. (5) **`AnthropicProvider::count_tokens` is a chars/4 approximation** at M02; M04 budget integration replaces with the real `/v1/messages/count_tokens` endpoint. (6) **`EventPipeline::next_event` for `ToolResult` translates with `duration_ms: 0`** — M02 doesn't run tools; M03 wires real durations.

What surprised: cross-platform integration tests deliver 2–3pp coverage gains on adjacent code paths the unit tests don't reach (M02.A `ipc.rs` 84.70% → 86.89% from the Windows subprocess test; M02.C `anthropic_sse.rs` 98.33% on first measurement from unit + wiremock dual-coverage); `OnceLock<reqwest::Client>` is `const fn`-constructible because `OnceLock::new()` has been `const` since Rust 1.70; `cargo llvm-cov --workspace` uses a distinct target dir that exposes hardcoded `target/debug/<binary>` paths in subprocess fixtures (caught by M02.D, fixed via `current_exe()` derivation, retrofit candidate for `crates/runtime-drone/tests/integration*.rs`).

### Adherence to spec

- ✅ **Zero telemetry held.** Spec §13 lines 2934-2939: no analytics SDK, no crash reporter, no "anonymous metrics," no phone-home. Verified: no analytics dep added across all five stages; `cargo audit` + `npm audit` pass with `--audit-level=high`; no outbound network calls except the user-initiated Anthropic call. Code: zero matches for `analytics` / `sentry` / `bugsnag` / `posthog` / `mixpanel` in `Cargo.toml` + `package.json`.
- ✅ **Direct Anthropic API (no third-party SDK).** Spec §0c stack table + §2c lines 950-972. Code at `crates/runtime-main/src/providers/anthropic.rs` uses `reqwest` + `eventsource-stream`; no `@anthropic-ai/sdk` or `anthropic-rs` dep. Verified in `package.json` + `Cargo.toml`.
- ✅ **SSE wire format matches Anthropic spec.** Spec §2c line 1018-1025 (ProviderEvent variants) → code at `providers/mod.rs:24-63` (TextDelta, ToolUse, ToolResult, ThinkingDelta, MessageStop, Error). Wire format tracked verbatim in `anthropic_sse.rs::stream_events` against the Anthropic streaming spec (verification URL + date in module docs); 8 wiremock tests cover happy path, auth/rate-limit, tool use, thinking, server-emitted error, malformed bytes skipped, partial-chunk reassembly.
- ✅ **`LLMProvider` trait shape matches spec §2c.** `name() / supports() / stream() / count_tokens() / list_models() / estimate_cost()` at `providers/mod.rs:357-397`. `BoxStream` return preserves object-safety + lifetime-of-`&self` borrowing.
- ✅ **AgentSdk generic over provider per spec §2c lines 926-947.** `AgentSdk<P: LLMProvider>` at `crates/runtime-main/src/sdk/agent_sdk.rs:71-77`. Generic-not-`dyn` decision documented in retro + commit msg + this entry under ⚠️.
- ✅ **Tauri capability lockdown matches spec §10.** `src-tauri/capabilities/default.json:7-11` enumerates `core:default` + `core:event:{default,allow-listen,allow-emit}` only. No `shell:*`, `fs:*`, `http:*`, `dialog:*`. Per spec §10 line 745 + the lockdown rationale at `agent-runtime-spec.md:2168` "Tauri allowlist — the renderer cannot reach any backend command not explicitly allowlisted".
- ✅ **Renderer never holds the API key.** `crates/runtime-main/src/key_store.rs` reads via `keyring::Entry` + `SecretString`; `src-tauri/src/commands.rs::run_smoke_session_with` reads the key main-side and passes `SecretString` to `AnthropicProvider::new`. Verified by inspection: no `invoke_*` returns API key bytes; no event payload contains the key.
- ✅ **Schemas-as-source-of-truth held for Rust types.** `crates/runtime-core/src/generated/{agent,common,framework,skill,tool}.rs` regenerable via `cargo xtask regenerate-types`; CI drift check at `crates/xtask/tests/check_drift.rs` still green.
- ✅ **DCO sign-off + Conventional Commits.** All five M02 commits signed (`Signed-off-by:`) and conform: `feat(workspace) / feat(runtime-main) / feat(workspace)`.
- ✅ **`*_with` test-seam pattern documented in `docs/style.md`.** Per M01-summary's M02-prep decision; cited as the canonical TDD-friendly approach to OS-call / network-call wrappers. Demonstrated four times in M02 (`anthropic_sse::stream_events`, `AgentSdk::run_agent_with_provider_stream`, `Connection::from_streams`, `commands::run_smoke_session_with`).
- ✅ **Coverage delta gating mechanism via Codecov.** M02.A: `codecov.yml` adds project + patch thresholds (`target: auto`, `threshold: 0.5%`). M02 PR will be the first activation. CLAUDE.md §5 records the Codecov-enforced delta gate; absolute thresholds (workspace ≥80%, drone ≥95%, runtime-main ≥95%) remain authoritative for hard floors. Pre-M01 baseline 🟡 carry-forward closed.
- ✅ **`mcp_servers` table created.** M02.A added the 8th table per spec §11 lines 2495-2503 with a deliberately richer 22-field schema (transport, stdio/remote mode fields, auth, lifecycle, timeouts, scope, capability caching, SQL CHECK constraints). Forward-compatible for M06 MCP work. M01 🟡 "mcp_servers — add now or document deferral" carry-forward closed via option (a).
- ✅ **Windows drone integration test.** `crates/runtime-drone/tests/integration_windows.rs` added at M02.A; exercises the named-pipe `accept_loop` end-to-end via subprocess + `IpcSnapshot` → drone-side `event_type` column. Lifted `ipc.rs` Windows-platform coverage from 84.70% → 86.89%. M01 🟡 carry-forward closed.
- ✅ **`.gitattributes` line-ending normalization.** `*.rs text eol=lf` + `*.json text eol=lf` + analogous TS/JSON entries added at M02.A. M01 🟡 carry-forward closed.
- ⚠️ **Decision extractor is a heuristic.** Spec §2 line 916 declares `decision_record` event with `decision`, `rationale`, `tool_used`. Code at `crates/runtime-main/src/sdk/decision_extractor.rs:42-64` scans line-by-line for `Decision:` / `Rationale:` / `Tool used:` markers. False-positive concern: the heuristic matches `Decision:` anywhere in text including code blocks, quoted user content, or model-emitted tutorials. M04 verify+rails replaces with a structured emitter injected by the prompt template. Documented at `decision_extractor.rs:1-7` module doc + retro + carry-forward.
- ⚠️ **`AnthropicProvider::count_tokens` is a chars/4 approximation.** Spec §2c line 1007 declares `count_tokens(messages) -> Result<u64, ProviderError>` with no implementation guidance. Code at `crates/runtime-main/src/providers/anthropic.rs::count_tokens` uses chars/4. M04 budget integration replaces with the real `/v1/messages/count_tokens` endpoint. Documented in CHANGELOG + retro.
- ⚠️ **`AgentSdk` is generic over `P: LLMProvider`, not `Box<dyn>`.** Spec §2c lines 926-930 declare `AgentSdk<P: LLMProvider>` — the spec text matches the M02 implementation (no deviation). v1.0+ multi-provider switching at runtime would require `Box<dyn LLMProvider + Send + Sync>` (object-safety holds — trait has no `where Self: Sized`); the refactor cost is one type substitution. Documented as the M02 design decision in `agent_sdk.rs:71-77` + retro.
- ⚠️ **`EventPipeline` translates `ProviderEvent::ToolResult` with `duration_ms: 0`.** Spec §2 line 845 declares `tool_result.duration_ms: number`. Code at `event_pipeline.rs:64-71` synthesizes `duration_ms: 0` because M02 doesn't actually run tools — the SSE `ToolResult` variant is provider-side; main-side tool execution lands in M03. Forward-compat shape; semantic correctness deferred to M03.
- ⚠️ **`signal.rs::ContextType` enum diverges from spec §2b.** Spec §2b line 1072 documents `context.type ∈ {skill, framework, code, search, verify, commit, subagent}`. Code at `crates/runtime-core/src/signal.rs:25-40` ships `AgentLoop / SkillLoad / ToolInvoke / HookExecute / PlanCreate / HitlPrompt / SessionLifecycle`. Different domains: shipped enum is operation-context-shaped, spec is artifact-source-shaped. No emission code consumes the enum yet (M04+ work). Resolution: M04 reconciles when emission integration happens — could go either direction depending on which shape is correct (runtime's operational discovery vs spec's theoretical enum). Defer the call to M04 closeout.
- ⚠️ **`mcp_servers` schema is richer than spec §11.** Spec §11 lines 2495-2503 lists 7 fields (`id, name, url, auth_key_ref, added_at, last_connected, status`). Code at `crates/runtime-drone/src/db.rs:172-216` ships 22+ fields covering transport (with CHECK constraining `stdio | http | sse | streamable_http`), stdio/remote mode (mutual-exclusion CHECK), env_json/args_json/headers_json/oauth_state_json, retry_count, timeouts, scope/plugin_id, capability caching. Deliberate extension based on Claude Code / Claude Desktop / VS Code MCP client schemas. Forward-compatible for M06. Resolution: file a dedicated ADR justifying the divergence — the schema is an architectural decision about MCP shape (which transports the runtime supports, how OAuth refresh state persists, how capability discovery caches) that goes beyond elaborating fields §11 didn't enumerate. Target: ADR before M06 Stage A (when the MCP client wires against this schema). Bundle a one-line cross-reference into the post-M02 `docs(spec):` PR pointing readers from §11 to the ADR.
- ⚠️ **`AnthropicProvider.with_base_url` is `pub` for wiremock injection but undocumented in spec.** Spec §2c shows the production `new(api_key)` constructor only. Code at `providers/anthropic.rs:43-51` exposes `with_base_url(api_key, base_url)`; used only by wiremock tests + the `anthropic_smoke.rs` gated integration test. No spec issue — implementation detail.
- ⚠️ **runtime-main coverage exclusion list grew to three modules.** `providers/anthropic.rs` (M02.C — real-network reqwest+SSE wrapper), `drone_ipc/connection.rs::open` (M02.D — cfg-platform OS-call), `key_store.rs` (M02.E — platform keyring-call). All three carry the same OS-signal-class rationale (structurally infeasible to test cross-platform without real OS resources). CLAUDE.md §5 documents each with one-line justification. CI gate `cargo llvm-cov --package runtime-main --ignore-filename-regex "..."`. No spec deviation; documenting the holdout list grew during M02.
- ⚠️ **vitest 80% coverage threshold configured but not enforced by default `npm run test`.** `vitest.config.ts` carries `thresholds: { lines: 80, functions: 80, branches: 80, statements: 80 }` but only triggers when `--coverage` is passed. Default `test` script runs Vitest without coverage. M03 prep — either enable `--coverage` by default in `test` script or add `test:coverage` + CI step. Not a regression; new gate added at M02.E that hasn't yet activated.
- ⚠️ **Tauri 2.x desktop-shell E2E (full WebView2 / WebKitGTK driving) deferred to M03.** Per Tauri 2.x official docs, full E2E needs `tauri-driver` + WebdriverIO with Linux + Windows matrix (macOS unsupported by tauri-driver). M02.E ships renderer-level Playwright against the Vite dev server; four `test.skip()`-with-rationale tests in `tests/e2e/smoke.spec.ts` mark the holdout. Documented in CLAUDE.md §6 frontend gates section. M03 carry-forward.
- ⚠️ **Frontend type sync is hand-mirrored.** `src/types/agent_event.ts` mirrors 10 of `runtime_core::AgentEvent`'s variants + `ToolSource` enum; the M02.D `ToolSource` + `AgentSpawned.session_id` additions would have silently drifted the TS side without protection. M03 prep — add `schemas/event.v1.json` + `cargo xtask regenerate-types --frontend` (or `json-schema-to-typescript` codegen) per CLAUDE.md §14 schemas-as-source-of-truth pattern.
- ❌ **None observed.** No outright contradictions where code ships behavior the spec forbids or vice versa. All deviations are forward-compat or documentation-side and resolvable via the post-M02 `docs(spec):` PR.

### Spec review (forward-looking)

- **Missing items:**
  - `agent-runtime-spec.md` §2c — `signature_delta` (Anthropic thinking-mode signature payload) is a real wire event the spec doesn't document; the SSE state machine parses + silently drops it (verified in 2 wiremock tests). Add a one-paragraph note to §2c that thinking-mode emits `signature_delta` events that callers should treat as no-ops. Surfaces in any future fresh-session reading of §2c.
  - `agent-runtime-spec.md` §2c — `ping` keep-alive events are valid SSE events (Anthropic API emits them mid-stream); spec doesn't mention. Add to the §2c wire-format note.
  - `agent-runtime-spec.md` §1d — IPC reconnect surface is undefined. M02.D pins 5 attempts with 200ms→1.6s exponential backoff (no trailing sleep); surfaces `DroneIpcError::Disconnected { retries }` on exhaustion. Add a one-paragraph "reconnect policy" subsection to §1d.
  - `agent-runtime-spec.md` §1d — drone-side snapshots-table column rename: spec §1d / §11 declares `DroneCommand::SnapshotNow { reason }` but the snapshots-table column is `event_type`. Documented at `crates/runtime-drone/src/snapshot.rs:30` but easy to miss. Either rename the column to `reason` (breaking schema change — bumps `snapshots.v2`) or document the rename in spec §11's snapshots-table description.
  - `agent-runtime-spec.md` §1d — `DroneCommand::SnapshotNow.state_json` field carry-forward from M01: spec still declares `{reason}` only; code declares `{reason, state_json}`. Bundled into post-M02 `docs(spec):` PR. (Also raised in M01 entry; still open at code↔spec level.)
  - `agent-runtime-spec.md` §1d — `DroneEvent::SnapshotWritten` carries `reason` + `timestamp` extensions in code that aren't in spec. Same story; bundled.
  - `agent-runtime-spec.md` §"Project Structure" — drone module listing still stale (M01 carry-forward); add the M02 additions to runtime-main (`providers/`, `sdk/`, `drone_ipc/`, `key_store.rs`).
  - `agent-runtime-spec.md` §M2 acceptance criteria — should explicitly note the Tauri 2.x E2E framework decision (`tauri-driver` + WebdriverIO, Linux+Windows matrix) so M03 prep doesn't re-discover it.
  - `agent-runtime-spec.md` §M3 — Phase 3 React Flow + Zustand expansion (Pre-M01 carry-forward) — still open; M03-blocking.
  - `agent-runtime-spec.md` §11 sessions table — Session lifecycle FSM diagram (Pre-M01 carry-forward) — still open; M04-blocking.
  - `agent-runtime-spec.md` §3a — plan model field shapes — M04-blocking.
  - `agent-runtime-spec.md` (project-wide) — model deprecation policy: when does the runtime stop accepting deprecated model IDs (e.g., when Anthropic retires `claude-3-opus-20240229`)? Spec doesn't say; M04 prep.

- **Contradictions:**
  - None observed.

- **Ambiguity:**
  - `agent-runtime-spec.md` §2c — `ProviderEvent::Error` semantics: terminal (stream ends) or recoverable (downstream retries)? M02 implementation chose **terminal** (the wiremock `error_event_emits_provider_error` test confirms the stream yields the `Error` event then terminates without `MessageStop`). Spec doesn't lock the choice. Recommendation: lock to terminal in spec §2c.
  - `agent-runtime-spec.md` §2c — `LLMProvider::stream` cancellation safety: trait doc says "All async methods must be cancellation-safe" (in code at `providers/mod.rs:344`) but spec doesn't. Add to §2c.
  - `agent-runtime-spec.md` §2c — retry policy: trait carries no retry state, but `RateLimit { retry_after_secs }` implies caller-side retry. Caller responsibility documented in trait doc but not spec.
  - `agent-runtime-spec.md` §2b — `ContextType` set divergence (see Adherence ⚠️ above). Lock the spec value set or bring runtime into alignment.

- **Open questions:**
  - When M03's renderer subscribes to `events()` long-lived, the M02.D `DroneClient` design's "single consumer; reconnect breaks the subscription" semantic will need an upgrade. Currently a reconnect drops the read half and respawns; the subscriber hits end-of-stream. Worth deciding M03-prep whether to (a) keep single-consumer + UI handles reconnect-loss, (b) buffer events client-side, or (c) re-subscribe automatically.
  - Tauri command-surface error wire format (`{"type": "...", "message": "..."}`) is currently in code (`commands.rs::CmdError`) but not in any schema. Should there be an `error.v1.json` schema documenting the renderer-facing wire shape so frontend codegen can pick it up? M03 prep.

- **Recommended spec changes:**
  - Bundle into a post-M02 `docs(spec):` PR (target: open before M03 Stage A begins) — items above on `signature_delta` / `ping` / IPC reconnect surface / snapshots column rename / drone module listing / runtime-main module listing / `ContextType` reconciliation / `ProviderEvent::Error` terminal semantics / cancellation safety / `mcp_servers` schema-extension rationale / Tauri 2.x E2E framework note in §M2-§M3 / model deprecation policy (the latter as a one-paragraph stub for M04 to flesh out).

### Fix backlog

Severity is non-elastic. Critical = "must fix before M03 starts."

- 🔴 **Critical** (must fix before M03 starts): **None.** All M02 acceptance criteria met; no shipped behavior is incorrect; all deviations are forward-compat or documentation-side. Hard gates G1–G5 cleared in all five stages.

- 🟡 **Important** (should fix this release cycle):
  - **TS `AgentEvent` codegen from schema.** Owner: code / `schemas/event.v1.json` + frontend codegen step. Hand-mirrored types drift; `ToolSource` + `AgentSpawned.session_id` would have silently drifted under any pressure. M03 prep — schema first; codegen via `json-schema-to-typescript` or a TS-target xtask.
  - **Tauri 2.x desktop-shell E2E via `tauri-driver` + WebdriverIO.** Owner: code / `tests/e2e/` + CI. Linux + Windows matrix (no macOS — tauri-driver unsupported). Four `test.skip()` Playwright tests in `tests/e2e/smoke.spec.ts` mark the carry-forward set. M03-blocking for the renderer-validates-end-to-end criterion.
  - **vitest `--coverage` enabled in default `test` script.** Owner: code / `package.json` + CI. M03 prep — either enable by default in `test` script or add `test:coverage` + CI step. The 80% threshold in `vitest.config.ts` is configured but not currently enforced.
  - **Decision extractor → structured emitter migration.** Owner: code / `crates/runtime-main/src/sdk/decision_extractor.rs` + M04 verify+rails prompt-template work. M04-blocking. Replaces the line-by-line heuristic with a structured emitter injected by the prompt; eliminates the false-positive concern for `Decision:` matched in code blocks / quoted content.
  - **`count_tokens` → real `/v1/messages/count_tokens` endpoint.** Owner: code / `crates/runtime-main/src/providers/anthropic.rs::count_tokens`. M04 budget integration depends. The chars/4 approximation is documented but inaccurate; budget enforcement needs the real number.
  - **`signal.rs::ContextType` reconcile with spec §2b.** Owner: code / `crates/runtime-core/src/signal.rs` OR spec / `agent-runtime-spec.md` §2b. M04 reconciles when emission integration happens — could go either direction depending on which shape is correct (runtime's operational discovery vs spec's theoretical enum). Defer the call to M04 closeout; M04 emission integration needs this resolved by then.
  - **Post-M02 `docs(spec):` PR.** Owner: spec / `agent-runtime-spec.md`. Bundle: `signature_delta` + `ping` notes (§2c); IPC reconnect surface (§1d); snapshots column rename + `SnapshotNow.state_json` + `SnapshotWritten` extensions (§1d / §11); cross-reference from §11 `mcp_servers` to the new MCP-schema ADR (the ADR itself is a separate item below); `ProviderEvent::Error` terminal semantics + cancellation-safety language (§2c); Tauri 2.x E2E framework decision (§M2 / §M3); drone module listing + runtime-main module listing (§"Project Structure"); §10 numbering gap (Pre-M01 cosmetic). `ContextType` reconciliation (§2b) **deferred to M04 closeout** per the Adherence ⚠️ above. Open before M03 Stage A.
  - **MCP-schema divergence ADR.** Owner: docs / `docs/adr/NNNN-mcp-servers-schema.md`. Justify the 22-field `mcp_servers` schema's divergence from spec §11's 7-field shape — transport set (`stdio | http | sse | streamable_http`), stdio-vs-remote mutual-exclusion CHECK, env/args/headers/oauth_state JSON columns, capability caching, scope/plugin_id, retry+timeout fields. Forward-compat for M06+. Target: file before M06 Stage A.
  - **CLAUDE.md §15 / `docs/gotchas.md` consolidation.** Owner: docs / `CLAUDE.md` §15. Consolidate the recurring clippy pedantic+nursery patterns + Tauri 2.x E2E framework + ESLint flat-config + Vite root convention + serde tag-shape + Vitest+RTL idiom + subprocess-fixture path-resolution + OOM-bound test fixture into a single subsection so the patterns don't re-discover themselves stage-by-stage. (See `M02-summary.md` Decisions for the consolidated list.)
  - **`TEMPLATE.md` updates.** Owner: docs / `docs/build-prompts/TEMPLATE.md` + `STAGE-PROMPT-PROTOCOL.md`. Coverage holdouts subsection; default safety-primitive test plan = unit-on-`*_with` + integration-end-to-end; `WEBCHECK:` header for fast-moving tooling stages; "Pre-existing legacy file inventory" subsection.
  - **Phase 3 React Flow + Zustand spec expansion.** Owner: spec / `agent-runtime-spec.md` §3. Pre-M01 carry-forward; **still open**. M03-blocking.
  - **Session FSM diagram.** Owner: spec / `agent-runtime-spec.md` §11 sessions table. Pre-M01 carry-forward; **still open**. M04-blocking.
  - **UI consistency: existing look and feel.** Owner: code / M03 prompts. Pre-M01 addendum carry-forward; **still open**. M03 stage prompts must embed this constraint when authored.
  - **Plan model field shapes.** Owner: spec / `agent-runtime-spec.md` §3a. New M04-blocking item.
  - **Retrofit `crates/runtime-drone/tests/integration*.rs` to `current_exe()`-derived paths.** Owner: code / drone tests. Currently they use `CARGO_MANIFEST_DIR/../../target/debug/runtime-drone.exe`; safe under per-package coverage but not under workspace coverage. Pre-M03 cleanup.
  - **Long-lived `events()` subscription survives reconnect (or doesn't — pick one).** Owner: code / `crates/runtime-main/src/drone_ipc/connection.rs`. M03 prep when the renderer subscribes long-lived.
  - **`error.v1.json` schema for `CmdError` wire format.** Owner: code / `schemas/error.v1.json` + frontend codegen. M03 prep.

- 🟢 **Nice-to-have** (queue for v1.0+ unless trivially incidental):
  - **Vite 5.4 → 7+ bump.** Owner: code / `package.json`. Vite 5.x carries dev-only esbuild CVE (browser-cross-origin to dev server); not exploitable in production bundle; passes `--audit-level=high`. M03 may bump as part of frontend-tooling-version drift cleanup.
  - **Delete legacy `src/counter.{js,test.js}`.** Owner: code / `src/`. Pre-runtime CommonJS files (commit `8c854c2`) conflicting with the new `"type": "module"`. Currently in `.prettierignore` + `eslint.config.js ignores` to avoid blocking M02. M03 cleanup PR.
  - **`secrecy/serde` workspace feature.** Owner: code / workspace `Cargo.toml`. Advertised but unused at every M02 stage. M03 evaluates; if M03 doesn't use it either, drop in a `chore(workspace):` PR.
  - **`keyring 3.6 → 4.0` upgrade.** Owner: code / workspace `Cargo.toml` + `crates/runtime-main/src/key_store.rs`. M03 / M04 evaluates when multi-platform CI matrix exercises real keychain calls. Decision deferred from M02.B/D/E.
  - **Tauri release-build caching** (M01 carry-forward). Owner: code / CI. Tauri release builds 3.5+ minutes (444 transitive deps); `Swatinem/rust-cache@v2` mitigates. Track but don't act unless friction surfaces.
  - **§10 numbering gap** in spec — RESOLVED at PR #36 per the post-M01 spec PR; bundled into the post-M02 `docs(spec):` PR closeout if any tail items remain.

### Carry-forward from prior milestones

For every Important item from prior gap-analysis entries, status as of M02 commit `4bd809a`:

- **Pre-M01 baseline 🟡 "Coverage delta gating mechanism"** — **RESOLVED at M02.A.** `codecov.yml` adds project + patch delta thresholds (`target: auto`, `threshold: 0.5%`); Codecov check is required-on (not informational). CI workflow uploads per-crate flag-tagged coverage. Absolute thresholds (workspace ≥80%, drone ≥95%, runtime-main ≥95%) remain authoritative for hard floors. Cited in `CLAUDE.md` §5.
- **Pre-M01 baseline 🟡 "Phase 3 React Flow + Zustand spec expansion"** — **STILL OPEN, M03-blocking.** Re-listed in M02 🟡 backlog. Address at M03 prep before stage authoring.
- **Pre-M01 baseline 🟡 "Session FSM diagram in spec §11"** — **STILL OPEN, M04-blocking.** Re-listed in M02 🟡 backlog.
- **Pre-M01 baseline 🟡 "Windows named pipe spec subsection"** — **RESOLVED at PR #36** (the post-M01 `docs(spec):` PR — the spec rebase) with code-level docs already shipped at M01.C in `crates/runtime-drone/src/ipc.rs:13-30` and `crates/runtime-drone/README.md`.
- **Pre-M01 baseline 🟡 "typify `oneOf` clippy suppression"** — **STILL RESOLVED.** No regressions in M02; the generated-file `#[allow(clippy::pedantic, clippy::nursery, clippy::all, missing_docs, unused_imports, rustdoc::invalid_html_tags)]` header continues to suppress noise across `crates/runtime-core/src/generated/*.rs`. M02.A added new generated content (`framework.rs` grew ~966 lines per the diff) without touching the header.
- **Pre-M01 baseline 🟢 "§10 numbering gap"** — **RESOLVED at PR #36** (cosmetic; closed via the post-M01 `docs(spec):` PR).
- **Pre-M01 addendum 🟡 "Reuse-first vs duplication-first §9 bias"** — **STILL DEFERRED to M07–M08** per the addendum's original decision. No M02 action; surface area still too small to make abstractions defensible.
- **Pre-M01 addendum 🟡 "UI consistency: existing look and feel"** — **CARRY-FORWARD INTO M03 prep.** No M02 work touched a "renderer adding new UX patterns" surface that would have triggered the rule (Stage E shipped the *first* renderer surface — there was no prior look and feel to be consistent with). M03 stage prompts must embed this constraint when authored.
- **M01 🟡 "`mcp_servers` table — add now or document the deferral"** — **RESOLVED at M02.A** via option (a). Added with full 22-field schema (transport, stdio/remote-mode mutual-exclusion CHECK, auth/keychain refs, lifecycle, timeouts, scope, capability caching). Forward-compatible for M06.
- **M01 🟡 "Post-M01 `docs(spec):` PR"** — **RESOLVED at PR #36** (pre-M02 work). Bundled the M01-flagged spec changes (HeartbeatStatus definition; SnapshotNow/SnapshotWritten extensions; JsonCodec → LinesCodec; drone module listing; capability/signal annotations; Windows named-pipe details; §10 numbering cosmetic).
- **M01 🟡 "Coverage delta gating mechanism"** (re-listed in M01 from Pre-M01 baseline) — **RESOLVED at M02.A** (see Pre-M01 carry-forward above).
- **M01 🟡 "`*_with` / `_inner` test-seam pattern"** — **RESOLVED at M02.A** (`docs/style.md` "Function design / `*_with` / `*_inner` test-seam pattern" subsection cites the M01.C archetype with executable example). **Applied verbatim at M02.C** (`anthropic_sse::stream_events`), **M02.D** (`AgentSdk::run_agent_with_provider_stream`, `Connection::from_streams`), and **M02.E** (`commands::run_smoke_session_with`). Pattern now demonstrated four times across two distinct I/O substrates.
- **M01 🟡 "Phase 3 React Flow + Zustand spec expansion"** (re-listed) — **STILL OPEN, M03-blocking.** Re-listed in M02 🟡 backlog.
- **M01 🟡 "Session FSM diagram"** (re-listed) — **STILL OPEN, M04-blocking.** Re-listed in M02 🟡 backlog.
- **M01 🟡 "UI consistency: existing look and feel"** (re-listed) — **CARRY-FORWARD INTO M03 prep** (see Pre-M01 addendum carry-forward above).
- **M01 🟡 "Reuse-first vs duplication-first §9 bias revisit"** (re-listed) — **STILL DEFERRED to M07–M08** per the addendum decision.
- **M01 🟡 "Windows drone integration test"** — **RESOLVED at M02.A.** `crates/runtime-drone/tests/integration_windows.rs` exercises the named-pipe `accept_loop` end-to-end via subprocess + `IpcSnapshot` + `event_type` column read; lifted Windows-platform `ipc.rs` coverage from 84.70% → 86.89%.
- **M01 🟡 "`.gitattributes` line-ending normalization"** — **RESOLVED at M02.A.** `*.rs text eol=lf` + `*.json text eol=lf` + analogous TS/JSON/MD entries added; future Windows fresh sessions don't surface CRLF/LF warnings.
- **M01 🟢 "§10 numbering gap"** (re-listed) — **RESOLVED at PR #36** (cosmetic).
- **M01 🟢 "Tauri release-build caching"** — **STILL OPEN, advisory.** No M02 friction observed; tracked in M02 🟢 backlog. M03 may revisit if frontend-tooling work slows CI further.

### Sign-off

**Claude:** I have generated this gap analysis after the final implementation stage of M02 (Stage E commit `4bd809a`). This is my honest assessment of the cumulative code-vs-spec state across M01 + M02. Hard gates G1–G5 cleared in all five M02 stages; no Critical-severity findings. The largest 🟡 backlog clusters are the post-M02 `docs(spec):` PR, the CLAUDE.md §15 + `TEMPLATE.md` consolidation, the M03-prep TS codegen + Tauri 2.x E2E setup, and the M04-prep `ContextType` reconciliation + count_tokens / decision-extractor migrations. User review pending; per `CLAUDE.md` §20 this entry is **immutable** once committed — future milestones report status updates via their Carry-forward sections.

**Surfaced at:** 2026-05-04 (UTC).
