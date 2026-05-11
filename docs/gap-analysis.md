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

---

## M03 — Live Graph (2026-05-06, Stage F commit on parent-milestone branch `claude/m03-live-graph`)

> Author: Claude (per `CLAUDE.md` §20)
> Stages aggregated: M03.A (build hygiene + carry-forward), M03.B (React Flow + Zustand foundation + 3 basic nodes), M03.C (8 remaining node types + animated edges + colors), M03.D (inspector panel + token weight + dagre layout), M03.E (VDR projection + SQL inspector + replay), M03.F (Tauri-driver E2E + Phase Closeout).
> Reviewed against: `agent-runtime-spec.md` §0–§0d, §1, §1c, §1d, §2, §2a, §2b, §2c, §3, §10, §11, §12, §13; `schemas/*.v1.json`; M03.A–F retrospectives + `M03-summary.md`; M01 + M02 entries' Carry-forward backlogs (Pre-M01 baseline + Pre-M01 addendum + M01 + M02).
> First milestone authored on the v1.2 XML stage-prompt protocol and the first to ship the new `<gotchas_graduation>` subsection enforcement.

### Codebase deep dive (cumulative — M01 + M02 + M03)

M03 turns the M02 event pipeline into a live graph + persistence + Tauri-shell E2E. New crates and surface across the milestone:

- **Renderer live-graph foundation** (`src/lib/{graphStore,layout,tokenScale}.ts`, `src/components/GraphCanvas.tsx`, `src/components/InspectorPanel.tsx`, `src/components/SqlInspector.tsx`, `src/components/nodes/{Agent,Tool,Skill,MCP,Gap,HITL,Plan,Task,Verify,Hook,Framework}Node.tsx`, `src/types/agent_event.ts` regenerated from `schemas/event.v1.json`). Zustand v5 store with `applyEvent(event: AgentEvent)` reducer over the 36-variant discriminated union, exhaustive `_exhaustive: never` switch as the schema-drift forcing function. React Flow v12 with `nodeTypes` map at module level (re-render trap avoidance per WebView v9 docs), `<Controls>` + `<Background>` + `<MiniMap>`, `useMemo`-keyed dagre layout (TB rankdir, recomputes only on graph-shape changes). All 11 spec §3 node types ship as renderable components — three are event-wired (AgentNode + ToolNode via M02 events; FrameworkNode via session_start; MCPNode via lazy-spawn on `tool_invoked.source='mcp'`); six (Gap/HITL/Plan/Task/Verify/Hook) ship with synthetic-state testing + unwired event handlers awaiting M04+/M05 schema additions. CSS palette via `:root` custom properties; animated edges via React Flow's `Edge.animated: true` + `react-flow__edge--animated` CSS keyframes; gap-pulse keyframe on GapNode per spec §3 Behavior. ARIA-compliant InspectorPanel (`aria-modal="false"` non-modal, ESC + close-button dismissal, Zustand selector subscription).

- **Token-spend visualization** (Stage D). `schemas/event.v1.json` gains optional `tokens_in`/`tokens_out` on `tool_result` and `tokens_total` on `agent_complete` (additive minor in-place per `schemas/README.md`); `crates/runtime-core/src/event.rs` hand-extended to match (the typify list excludes `event.v1.json` per Stage A — flagged as M04 carry-forward). `crates/runtime-main/src/providers/{mod,anthropic_sse}.rs::ProviderEvent::ToolResult/MessageStop` extended with the same fields; `SseState` accumulates input + output tokens across `message_start.usage` + `message_delta.usage` (real Anthropic SSE wire data, not estimates) and surfaces them on `MessageStop.total_tokens`. `EventPipeline` forwards through translation. CSS `transform: scale(0.8 → 1.5)` per cumulative tokens via `tokenScale.ts` — shared between AgentNode + ToolNode (factor-of-three reuse threshold met).

- **VDR projection + SQL inspector + replay-from-signals** (Stage E). `crates/runtime-drone/src/vdr.rs` (NEW) — drone-internal continuous projector reads signals 4 (decision) + 5 (verify) per spec §2b → writes correlated `vdr` table rows. Idempotent via UNIQUE INDEX on `vdr.contributing_signal_id`. SELECT-only validation in `is_select_only` is **lexical** (empty rejection + compound-semicolon rejection + lowercase-`select` allowlist + PRAGMA rejection) — the original Phase doc draft used `Connection::open_in_memory().prepare()` which rejects every legitimate SELECT (in-memory probe DB has no schema); the actually-shipped lexical validator is correct per the security boundary "doesn't mutate state". `crates/runtime-drone/src/command_handler.rs` extended with `QuerySessionDb { sql }` + `ReadSignals { session_id }` arms + UTF-8-safe `truncate_for_log` helper. `crates/runtime-main/src/sdk/replay.rs` (NEW) — pure-function inverse of M02.D's EventPipeline: signal log → AgentEvent stream. `src-tauri/src/commands.rs` adds `query_session_db_with` + `replay_session_with` testable seams (M02 archetype); production wrappers route through `DroneClient::noop()` because v0.1 has no real drone subprocess yet (M04+ scope). New `<SqlInspector>` renderer component + `invokeQuerySessionDb` + `invokeReplaySession` ipc.ts wrappers + `App.tsx` replay-on-mount via `localStorage.lastSessionId`.

- **Tauri-driver E2E framework** (Stage F). `wdio.conf.ts` configures WebdriverIO v9 + `tauri-driver` as a service per <https://v2.tauri.app/develop/tests/webdriver/>. macOS early-exit so local dev on a Mac is a no-op rather than a hard failure (`tauri-driver` is upstream-unsupported on macOS). `tests/e2e-tauri/smoke.e2e.ts` ships 6 tests covering app launch + setup, save-key flow, smoke happy path with real Anthropic API, click-to-inspect, SQL inspector execute, reload reconstructs from persisted signals. New `e2e-tauri-driver` CI job — Linux + Windows matrix (no macOS); Linux uses `xvfb-run` + `webkit2gtk-driver`; Windows uses pre-installed msedgedriver + Edge WebView2; `tauri-driver` installed via `cargo install --locked`. The four `test.skip()`-with-rationale entries that M02.E carried forward in `tests/e2e/smoke.spec.ts` are deleted; the three active renderer-level Playwright tests remain (different layer, faster feedback). `package.json` `overrides.serialize-javascript: ^7.0.5` patches the only high-severity transitive audit finding from the new mocha tree (GHSA-5c6j-r48x-rmvq + GHSA-qj8w-gfj5-8c6v).

What's solid: the schema-as-source-of-truth pipeline now extends to TS via `cargo xtask regenerate-types` (Stage A) + the resulting 36-variant discriminated union is the canonical event surface; the *_with archetype now scaled across six substrates (drone IPC named pipes / HTTP+SSE / in-process streams / Tauri commands / drone command-handler arms / SqlInspector); zero-telemetry boundary held end-to-end (no analytics deps, no crash reporter, no phone-home — verified at every audit step including Stage F's new mocha tree); capability lockdown at the Tauri allowlist boundary held end-to-end; the `[END] Decisions for the next stage` discipline is functioning as a hard checklist applied before code (M03.B Decisions 100% applicable to C; C 100% applicable to D; D feeds E; E feeds F — five demonstrations across M03); per-stage retrospectives consistent across all six stages with honest self-assessment.

What's structurally weak and likely to compound: (1) **`crates/runtime-core/src/event.rs` is hand-maintained** — `event.v1.json` is in the TS codegen list but not the Rust typify list per Stage A's xtask design. Stage D's schema bump exercised the brittleness (manual `event.rs` hand-edits to mirror the schema). M04 carry-forward to extend the typify list. (2) **The production `DroneClient::noop()` pattern** — Stage E ships v0.1 with production Tauri command wrappers routing through a noop drone client because no actual drone subprocess wires up at startup yet. The `*_with` testable seams exercise the full chain in unit + integration tests; production end-to-end persistence is M04+ scope. (3) **`vdr.rs` projector is dormant in v0.1 production** — exercised only by tests. M04+ wires `WriteSignal` that calls `project_signal` after each insert, which makes the success criterion (c) "vdr table populates with at least one row from the decision_record event" actually true end-to-end. (4) **`runtime-main` `client.rs` per-module regression** — Stage E's new request/response methods (`query_session_db`, `read_signals`, `await_event`) added ~120 lines + a 5-second `tokio::time::timeout` Err branch that's structurally hard to cover at unit level (the test would need `tokio::time::pause()` + `advance()` which collides with the polled stream). The module dropped 100% → 94.00% line; documented per CLAUDE.md §5 as accepted regression with retro entry. M04+ may close via the `connection.rs::backoff_grows_exponentially_between_attempts` archetype. (5) **localStorage `lastSessionId` persistence is webview-scoped** — sufficient for v0.1 single-instance use; v1.0 multi-session per spec §1c may require SQLite-side persistence. (6) **Phase doc sample-body drift recurred 3 stages of M03** — A, B, D all caught snake_case-vs-camelCase prompt-vs-shipped reality drift; the prompt continues to mislead fresh sessions across consecutive stages even after the lesson was logged in M03.A retro Decisions. Structural risk for any milestone that lands a schema in Stage X and consumes it in Stage X+1.

What surprised: the calibration drift continued — M03 mean ratio 0.32× (M01 0.3× / M02 0.7×); detailed Phase doc + locked archetype + Decisions inheritance compound to make M03 the fastest-delivered milestone yet relative to estimate. WebdriverIO v9 dropped `PromiseLike` from `ChainablePromiseElement` — v8's `await $('selector')` pattern silently breaks at type-check time in v9; took 3 round-trips on Stage F to land the chainable pattern correctly. The session-start sequencing blocker (Stage E uncommitted vs. F prompt premise) is a closeout-specific operational issue — the user's per-stage approval discipline + Claude's §12 "ask first" rule jointly resolved it cleanly, but a v1.3 `<pre_flight_check>` slot would prevent it preventively.

### Adherence to spec

- ✅ **All 11 spec §3 node types renderable.** Code at `src/components/nodes/*.tsx` — `AgentNode` + `ToolNode` + `SkillNode` (M03.B), `MCPNode` + `GapNode` + `HITLNode` + `PlanNode` + `TaskNode` + `VerifyNode` + `HookNode` + `FrameworkNode` (M03.C). Each has type-appropriate data + Handle/Position imports + ARIA + `data-testid`. HITLNode is `role="alert"` + `aria-live="assertive"` per WAI APG; GapNode has `gap-pulse` keyframe per spec §3 Behavior.
- ✅ **Animated edges (active call) + dashed edges (skill load)** per MVP §M3 Deliverable. Code at `src/lib/graphStore.ts` (animated-edge state machine: `tool_invoked` → `animated: true`; `tool_result` clears the flag on the inbound edge) + `src/styles.css` (`.react-flow__edge.animated` keyframe `dash-flow` 1s linear; `.react-flow__edge--dashed` static for skill-load).
- ✅ **Click-to-inspect side panel.** Code at `src/components/InspectorPanel.tsx`. Right-rail layout (per M03.C Decisions for Stage D), ARIA-compliant non-modal dialog, ESC + close-button dismissal, Zustand selector subscription on `selectedNodeId`. M03.D acceptance criteria #2 ("Click any node → side panel shows full event payload") met for the data side; the "+ correlated VDR row" portion is M03.E's `<SqlInspector>` (separate component below the graph; full join-into-inspector is M04+ when VDR has populated rows from real signal-write code).
- ✅ **Token-spend visualization (node weight).** Code at `src/lib/tokenScale.ts` + AgentNode/ToolNode `style={{ transform: scale(...) }}`. Cumulative token tracking through `applyEvent('tool_result')` → AgentNode totals + per-call ToolNode fields. Real token data from M03.D's anthropic_sse.rs `cumulative_tokens` accumulation across `message_start.usage` + `message_delta.usage`.
- ✅ **VDR projection from signal stream — populated table.** Code at `crates/runtime-drone/src/vdr.rs::project_signal` + `project_session`. Reads signals 4 (decision) + 5 (verify) per spec §2b; writes correlated rows to the `vdr` table; idempotent via UNIQUE INDEX on `contributing_signal_id` (schema migration in `db.rs::init_schema`). 6 unit tests + 2 drone-side integration roundtrips at `crates/runtime-drone/tests/{vdr_projection,integration}.rs`.
- ✅ **Simple SQL inspector** per MVP §M3. Code at `src/components/SqlInspector.tsx`. Textarea + Execute button + results table or error paragraph; SELECT-only validation parser-based (lexical structure analysis in `vdr.rs::is_select_only`); rejected statements surface as `Alert(Critical)` from drone. 5 unit tests cover the contract.
- ✅ **Tauri 2.x desktop-shell E2E framework** per MVP §M3 Deliverable. Code at `wdio.conf.ts` + `tests/e2e-tauri/smoke.e2e.ts` + `.github/workflows/ci.yml::e2e-tauri-driver`. WebdriverIO v9 + `tauri-driver` (cargo-installed); Linux + Windows matrix; macOS skipped (upstream-unsupported). Six tests covering renderer load + smoke happy path + click-to-inspect + SQL inspector + replay reconstruction. The four `test.skip()`-with-rationale entries from M02.E removed.
- ✅ **React + React Flow + Zustand for state; no Redux, no MobX.** `package.json` shows React 18 + `@xyflow/react ^12` + `zustand ^5`; no Redux/MobX deps. Verified by inspection.
- ✅ **Renderer Vitest coverage ≥80% on graph reducers.** `src/lib/graphStore.ts` 97.39% line (≥95% safety primitive); workspace src/ 96.07% line (≥80%); per CLAUDE.md §5 + `vitest.config.ts` thresholds.
- ✅ **Schemas-as-source-of-truth held for TS types.** `cargo xtask regenerate-types` (M03.A) regenerates `src/types/agent_event.ts` from `schemas/event.v1.json` + drift-check job in `.github/workflows/ci.yml`. M03.D's schema bump exercised the pipeline cleanly (zero diff after re-run).
- ⚠️ **Production drone subprocess not yet wired in v0.1.** Spec §1c declares "drone-per-session" architecture; M03.E ships the architectural primitives (DroneCommand variants, *_with seams, replay translator) with full unit + integration test coverage but the production Tauri command wrappers route through `DroneClient::noop()` at runtime. The end-to-end persistence acceptance criterion "Graph reconstructs after page reload (state from SQLite)" is met by the unit + integration test stack (replay translator + drone-side roundtrips) but not yet by production wiring. Resolution: M04 spawns the drone subprocess at Tauri startup + manages `Arc<DroneClient>` via Tauri's managed state. Documented in `src-tauri/src/commands.rs` doc comments + CHANGELOG.
- ⚠️ **`crates/runtime-core/src/event.rs` is hand-maintained** despite `schemas/event.v1.json` being the canonical source. The Stage A xtask design intentionally puts `event.v1.json` in the TS codegen list (json-schema-to-typescript) but not the Rust typify list — Rust types match the schema by manual hand-edit. Stage D exercised the brittleness with a successful schema bump but it's a pothole. Resolution: M04 Stage A carry-forward to extend the xtask Rust typify list.
- ⚠️ **`runtime-main` `client.rs` per-module coverage regression** (100% → 94.00% line). M03.E's new `query_session_db` + `read_signals` + `await_event` request/response methods added ~120 lines; 4 new tests cover ~94% of them. Uncovered paths are pre-existing `connect()` body (real socket required) + `await_event`'s 5-second `tokio::time::timeout` Err branch (structurally hard to cover at unit level) + ~3 lines in the `read_signals` Alert filter branch. Per CLAUDE.md §5 retro entry (preserved-or-improved invariant): accepted regression with rationale documented in M03.E retrospective. Above the ≥95% safety primitive overall (97.50% runtime-main); M04+ may close via the `connection.rs::backoff_grows_exponentially_between_attempts` `tokio::time::pause()` archetype.
- ⚠️ **`runtime-main` overall coverage drift** — M02.E baseline was 99.01% / 99.37%; M03.E end is 97.50%. The drop concentrates in `client.rs` (above) + minor `anthropic_sse.rs` rounding (98.33% → 97.47% on the new SseState::add_usage branches). Above the ≥95% safety primitive bar; documented as M03 carry-forward.
- ⚠️ **MVP §M3 acceptance criterion (a) "open the SQL inspector and run `SELECT * FROM signals` to see the live signal log"** — partial in v0.1 production. The unit + integration tests exercise the full chain (drone-side roundtrips return JSON rows); the production end-to-end path returns empty rows because the production drone is `DroneClient::noop()`. Same root cause as the production-wiring ⚠️ above. M04+ wires real drone subprocess.
- ⚠️ **`vdr.rs` projector is dormant in v0.1 production** — exercised only by tests because no signal-write code exists yet (M04+ owns this). Documented in `vdr.rs` doc comments. The MVP §M3 acceptance criterion #6 ("VDR populated table from signal stream") is met by the unit + integration tests (signals seeded directly via `init_in_existing` + `WriteSignal`-equivalent); production end-to-end at M04+.
- ⚠️ **Six of 11 spec §3 node types ship without event-wired reducer cases** (Gap, HITL, Plan, Task, Verify, Hook). Schema doesn't yet declare the events those components consume — `gap_*` is M05 (gap detection); `hitl_*`, `plan_*`, `task_*`, `verify_*`, `hook_*` are M04. The components ship with synthetic-state testing pattern locked + ARIA + spec §3 Visual Design styling so M04+/M05 just promote the existing `_exhaustive: never` no-op cases to wired cases without renderer-test churn. Documented in `graphStore.ts` JSDoc.
- ⚠️ **`tokenScale.ts` clamp range** (`0.8 → 1.5`) is calibrated for v0.1 smoke-session token magnitudes (tens to hundreds of tokens); a multi-hour session could reach 10k+ tokens, hitting the 1.5× cap. At the cap, all heavy nodes look identical. Documented in `tokenScale.ts` JSDoc as v0.1 calibration; M04+ may want a logarithmic curve when persistence-replay reveals real visual ambiguity at scale.
- ⚠️ **Stage F `wdio.conf.ts` references the file `wdio.conf.ts` (not `wdio.config.ts`)** — `eslint.config.js`'s `**/*.config.{ts,js}` glob doesn't match `.conf.ts`. Workaround: explicit `'wdio.conf.ts'` entry in the eslint override block + `parserOptions: { projectService: false, project: null }`. Documented in CHANGELOG. The `wdio.conf.ts` filename is the canonical Tauri 2.x WebDriver config name per <https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/>; renaming would break the convention.
- ⚠️ **CI `e2e-tauri-driver` job is unverified locally.** The new job hasn't run against a real CI matrix yet — the user's three-artifact review approval triggers the PR push which triggers CI. The job presumes `ANTHROPIC_TEST_KEY` is configured as a GitHub repo secret (~$0.001 per CI run × 2 OS) AND that Linux's keyring stack works on stock GitHub runners (libsecret + dbus). Either may surface friction on first PR push. Resolution path: Stage F's commit ships the infrastructure; the user iterates on CI configuration once the PR push reveals which paths fail.
- ❌ **None observed.** No outright contradictions where code ships behavior that spec forbids or vice versa. All deviations are forward-compat, documentation-side, or v0.1-scoped (production wiring deferrals to M04+).

### Spec review (forward-looking)

- **Missing items:**
  - `agent-runtime-spec.md` §3 — token-spend visualization clamp / scale function: spec says "Token spend shown as node weight — larger spend = visually larger node" but doesn't lock a clamp range or growth curve. M03 ships `clamp(0.8, 1 + tokens/1000, 1.5)` — calibrated for v0.1 smoke-session magnitudes. Spec should note the v0.1 calibration + document the scale function shape so M04+ persistence-replay tests can re-evaluate.
  - `agent-runtime-spec.md` §3 — InspectorPanel layout decision (right-rail vs bottom-drawer) is implementation choice but worth a one-line preference: M03 chose right-rail per the canvas-at-70vh constraint + desktop-screen-aspect-ratio. Locking the choice in spec prevents drift.
  - `agent-runtime-spec.md` §3 — handle layout per node type: Gap/HITL target-only (terminal states; resolution comes via UI side-channel), Framework source-only (graph root, no upstream), the rest both. M03.C surfaced this as ambiguity; spec should document the convention so M04+ doesn't re-litigate.
  - `agent-runtime-spec.md` §M3 — should explicitly note the v0.1-vs-M04 split: VDR projector dormant in v0.1 production; production drone subprocess wires at M04 startup; SQL inspector returns empty rows in v0.1 unless tests pre-seed.
  - `agent-runtime-spec.md` §2c — `signature_delta` (Anthropic thinking-mode signature) note carry-forward from M02 — still open; bundle into the post-M03 `docs(spec):` PR.
  - `agent-runtime-spec.md` §1d — IPC reconnect surface (200ms→1.6s exponential backoff, 5 attempts) carry-forward from M02 — still open.
  - `agent-runtime-spec.md` §1d / §11 — drone snapshots-table column rename (`event_type` vs `reason`) carry-forward from M02 — still open.
  - `agent-runtime-spec.md` §3a — plan model field shapes — M04-blocking; new from M02 still open.
  - `agent-runtime-spec.md` §11 — sessions table FSM diagram — Pre-M01 carry-forward, M04-blocking; still open.
  - `agent-runtime-spec.md` (project-wide) — model deprecation policy carry-forward from M02 — still open.

- **Contradictions:**
  - None observed.

- **Ambiguity:**
  - `agent-runtime-spec.md` §2b — `ContextType` enum set divergence (Pre-M02 carry-forward; runtime ships `AgentLoop / SkillLoad / ToolInvoke / HookExecute / PlanCreate / HitlPrompt / SessionLifecycle`; spec documents `skill | framework | code | search | verify | commit | subagent`). M03 didn't write any signal-emission code so the divergence didn't bite — but M04 emission integration must reconcile (either rename runtime variants to match spec or update spec). **Defer the call to M04 closeout** as already documented in M02 entry.
  - `agent-runtime-spec.md` §2b — `is_select_only` security boundary: spec doesn't specify HOW SELECT-only is enforced (regex vs lexical vs schema-aware). M03.E ships lexical analysis (compound-semicolon rejection + lowercase-`select` allowlist + PRAGMA rejection). Recommendation: lock the choice in spec — lexical is sufficient because the security boundary is "doesn't mutate state"; execution-time errors (malformed SELECT, missing columns) surface as Critical alerts.
  - `agent-runtime-spec.md` §3 — replay-from-signals vs snapshot-the-store as the persistence mechanism. M03.E chose replay-from-signals (spec append-only model + applyEvent idempotence end-to-end). Spec doesn't lock the choice; should.
  - `agent-runtime-spec.md` §M3 — session-id discovery: M03.E uses `localStorage.lastSessionId`; sufficient for v0.1 webview-scoped single-instance; v1.0 multi-session per spec §1c may need SQLite-side persistence. Spec should note the v0.1 choice + flag the v1.0 carry-forward.

- **Open questions:**
  - When M04's session-lifecycle work spawns the actual drone subprocess at Tauri startup, the M03.E `DroneClient::noop()` production wrappers should be replaced with `Arc<DroneClient>` via Tauri's managed state. The replacement is mechanical but the semantics of the noop fallback (when the drone hasn't started yet, when the drone has crashed, when the drone is reconnecting) need to be documented.
  - Long-lived `events()` subscription survives reconnect — carry-forward from M02 entry. M03 didn't extend the subscription path (replay is one-shot on mount); when M04+ adds long-lived event subscriptions for session-lifecycle, this needs to be decided.
  - Tauri command-surface error wire format `error.v1.json` schema — carry-forward from M02 entry. Still open.

- **Recommended spec changes:**
  - Bundle into a post-M03 `docs(spec):` PR (target: open before M04 Stage A begins) — items above on token-scale calibration; InspectorPanel layout convention; per-node-type handle layout; v0.1-vs-M04 production-wiring split; lexical SELECT-only validation; replay-from-signals as the persistence model; localStorage lastSessionId v0.1 choice. Also bundle the carry-forward items still open from M02 entry's "Recommended spec changes": `signature_delta` + `ping` + IPC reconnect surface + snapshots column rename + drone module listing + runtime-main module listing + `ProviderEvent::Error` terminal semantics + cancellation-safety language + Tauri 2.x E2E framework note + drone module listing + model deprecation policy. Open before M04 Stage A.

### Fix backlog

Severity is non-elastic. Critical = "must fix before M04 starts."

- 🔴 **Critical** (must fix before M04 starts): **None.** All M03 acceptance criteria met (with the v0.1 production-wiring split documented as ⚠️ deviations); no shipped behavior is incorrect; all deviations are forward-compat or documentation-side. Hard gates G1–G5 cleared in all six stages. Aggregate axis means healthy (Process 38.83/40, Product 37.67/40, Pattern 30.00/35).

- 🟡 **Important** (should fix this release cycle):
  - **Production drone subprocess wiring at Tauri startup.** Owner: code / `src-tauri/src/main.rs` + `crates/runtime-main/src/drone_ipc/`. M04 owns this. Spawn the drone subprocess + manage `Arc<DroneClient>` via Tauri's managed state; replace M03.E's production `DroneClient::noop()` wrappers. Closes the ⚠️ on `query_session_db` / `replay_session` end-to-end persistence and the MVP §M3 acceptance criterion "Graph reconstructs after page reload (state from SQLite)" at the production layer.
  - **`vdr.rs` projector wired at signal-write call-site.** Owner: code / `crates/runtime-drone/src/command_handler.rs`. M04 owns this. When `WriteSignal` lands in M04, call `vdr::project_signal(conn, signal_id)` after each insert. Closes the ⚠️ on VDR populating from real session activity (currently only test-seeded).
  - **Extend xtask Rust typify list to include `event.v1.json`.** Owner: code / `crates/xtask/src/main.rs`. M04 Stage A carry-forward (folded into build-hygiene scope). Closes the ⚠️ on `event.rs` hand-maintenance brittleness.
  - **Add `tokio::time::pause()`-driven coverage for `await_event` timeout path.** Owner: code / `crates/runtime-main/src/drone_ipc/client.rs`. M04+ Stage X. Closes the ⚠️ on `client.rs` per-module regression (100% → 94.00%); archetype is `connection.rs::backoff_grows_exponentially_between_attempts`.
  - **Post-M03 `docs(spec):` PR.** Owner: spec / `agent-runtime-spec.md`. Bundle: token-scale calibration (§3); InspectorPanel layout convention (§3); per-node-type handle layout convention (§3); v0.1-vs-M04 production-wiring split (§M3); lexical SELECT-only validation (§2b); replay-from-signals as the persistence model (§3); localStorage lastSessionId v0.1 choice (§M3); plus the M02 carry-forward bundle (`signature_delta` + `ping` + IPC reconnect + snapshots column rename + drone/runtime-main module listings + `ProviderEvent::Error` terminal semantics + cancellation-safety + Tauri 2.x E2E note + model deprecation policy). Open before M04 Stage A.
  - **`docs/gotchas.md` consolidation from M03 closeout.** Owner: docs / `docs/gotchas.md`. Graduate the eight `<gotchas_graduation>`-graduated items: (1) snake_case schema discipline (M03.A/B/D recurrence); (2) `prettier --write` / `cargo fmt --all` as first `verify_gates` step (M03.B/C/D/E); (3) React Flow + happy-dom `act()` warning surface (M03.B/C); (4) synthetic-state testing pattern (M03.C/D); (5) trust TS narrowing don't assert (M03.B/D); (6) WebdriverIO v9 chainable pattern (M03.F); (7) npm `overrides` for transitive audit failures (M03.F); (8) eslint config-glob `**/*.config.{ts,js}` ≠ `.conf.ts` (M03.F). Most are forward-applicable to M04+. Land in protocol-iteration session before M04.A.
  - **STAGE-PROMPT-PROTOCOL.md v1.3 protocol-iteration session before M04.A.** Owner: docs / `STAGE-PROMPT-PROTOCOL.md`. Five v1.3 candidates: closeout-only `<pre_flight_check>` slot; `<schema_drift_check>` optional slot; `<fan_out_grep>` optional slot; `<dependency_audit_check>` optional slot; `<runtime_environment>` optional slot. Generalizes session-friction patterns observed across M03.
  - **CI `e2e-tauri-driver` job verification on first PR push.** Owner: ops / `.github/workflows/ci.yml`. The new job is unverified locally — once the M03 PR pushes, CI runs the matrix. Likely friction: (a) keyring/dbus on stock Linux runner; (b) `ANTHROPIC_TEST_KEY` secret presence; (c) `tauri-driver` cargo install time. If any path fails, iterate via follow-up commits on the M03 branch BEFORE merge.
  - **`signal.rs::ContextType` reconcile with spec §2b.** Owner: code or spec. M02 carry-forward — still deferred to M04 closeout per the M02 entry's Adherence ⚠️ section. M04 emission integration needs this resolved by then.
  - **Decision extractor → structured emitter migration.** M02 carry-forward — M04-blocking. Replace the line-by-line heuristic with a structured emitter injected by the prompt template.
  - **`count_tokens` → real `/v1/messages/count_tokens` endpoint.** M02 carry-forward — M04 budget integration depends.
  - **Phase 3 React Flow + Zustand spec expansion.** Pre-M01 carry-forward — **RESOLVED at M03 implementation level** but the SPEC text expansion is still open. Bundle into the post-M03 `docs(spec):` PR. Document the React Flow v12 + Zustand v5 + dagre v2 + 11 node types + ARIA + animated edges + token-weight scaling decisions in spec §3.
  - **Session FSM diagram.** Pre-M01 carry-forward — M04-blocking; still open.
  - **UI consistency: existing look and feel.** Pre-M01 addendum carry-forward — **RESOLVED at M03 implementation level** (M03 inherits M02's SetupPanel/SmokeButton visual language; new components Inspector + SqlInspector + 11 nodes follow the same palette via `:root` CSS custom properties). Documented at `src/styles.css`. M04 stage prompts must continue this discipline (re-listed in M04 carry-forward below).
  - **Plan model field shapes.** M02 carry-forward — M04-blocking.
  - **MCP-schema divergence ADR.** M02 carry-forward — target M06 prep; not affected by M03.
  - **Long-lived `events()` subscription survives reconnect (or doesn't — pick one).** M02 carry-forward — M04-blocking when the renderer subscribes long-lived for session-lifecycle.
  - **`error.v1.json` schema for `CmdError` wire format.** M02 carry-forward — M04 prep.
  - **Vite 5 → 7 bump.** M02 carry-forward — **RESOLVED at M03.A** via `package.json` bump to `vite ^7.1.0` + the `host: '127.0.0.1'` IPv4-binding fix + `optimizeDeps.include` extension + Playwright `workers: 1 / timeout: 60_000 / webServer.timeout: 90_000 / baseURL: 'http://127.0.0.1:1420'`.
  - **Delete legacy `src/counter.{js,test.js}`.** M02 carry-forward — **RESOLVED at M03.A** (deleted via Stage A's `src/counter.{js,test.js}` removal).
  - **`secrecy/serde` workspace feature.** M02 carry-forward — **RESOLVED at M03.A** (dropped via Stage A's secrecy/serde feature removal; grep verified zero callsites).
  - **`keyring 3.6 → 4.0` upgrade.** M02 carry-forward — **DEFERRED to M04+** when multi-platform CI matrix exercises real keychain calls. M03.F's CI `e2e-tauri-driver` job will start exercising real keychain on Linux + Windows; if the stub backend bites, M04 reopens.

- 🟢 **Nice-to-have** (queue for v1.0+ unless trivially incidental):
  - **Logarithmic token-scale curve for `tokenScale.ts`.** Owner: code / `src/lib/tokenScale.ts`. M04+ persistence-replay may surface real multi-hour sessions where the v0.1 calibration `clamp(0.8, 1 + tokens/1000, 1.5)` saturates at 1.5×; a log curve would preserve discriminability at scale. Defer until persistence-replay reveals real ambiguity.
  - **Tauri release-build caching.** M01 + M02 carry-forward; advisory. Tauri release builds 3.5+ minutes (444 transitive deps); `Swatinem/rust-cache@v2` mitigates. M03's new `e2e-tauri-driver` job adds a release-build step on every PR push × 2 OS — if CI time becomes painful, evaluate a Tauri-skipping CI lane.
  - **`docs/gotchas.md` cross-reference index.** As `docs/gotchas.md` grows past 31 entries (it's at 31 now per M02 closeout consolidation; M03 adds 8), a cross-reference index by surface area (Rust / TS / CI / Tauri / Schema / Test) would help fresh sessions find relevant entries quickly. Track but don't act unless friction surfaces.
  - **§10 numbering gap** — RESOLVED at PR #36 per M02 entry; cited here for completeness.
  - **`vitest --coverage` enabled in default `test` script.** M02 carry-forward — **RESOLVED at M03.A** (`package.json` `test` script now runs `vitest run --coverage` per Stage A's deliverable).

### Carry-forward from prior milestones

For every Important item from prior gap-analysis entries, status as of M03 commit (Stage F):

- **Pre-M01 baseline 🟡 "Coverage delta gating mechanism"** — RESOLVED at M02.A (Codecov-enforced delta gates per `codecov.yml`). Re-listed in M01 + M02 entries; unchanged in M03.
- **Pre-M01 baseline 🟡 "Phase 3 React Flow + Zustand spec expansion"** — **PARTIALLY RESOLVED at M03 implementation level (M03.B/C/D shipped React Flow v12 + Zustand v5 + dagre v2 + 11 node types + ARIA + animated edges + token-weight + InspectorPanel)**; **STILL OPEN at the spec text level** — bundle into the post-M03 `docs(spec):` PR. Re-listed in M03 🟡 backlog above.
- **Pre-M01 baseline 🟡 "Session FSM diagram in spec §11"** — STILL OPEN, M04-blocking. Re-listed in M03 🟡 backlog.
- **Pre-M01 baseline 🟡 "Windows named pipe spec subsection"** — RESOLVED at PR #36 per M02 entry.
- **Pre-M01 baseline 🟡 "typify `oneOf` clippy suppression"** — STILL RESOLVED. M03 added no new schema content beyond `event.v1.json` Stage D bump (token fields); no regressions.
- **Pre-M01 baseline 🟢 "§10 numbering gap"** — RESOLVED at PR #36 per M02 entry.
- **Pre-M01 addendum 🟡 "Reuse-first vs duplication-first §9 bias"** — STILL DEFERRED to M07–M08 per the addendum's original decision. M03's renderer surface area (5 lib files + 14 components + 18 unit-test files) doesn't yet make abstractions defensible — the per-component-with-shared-archetype pattern (e.g., 11 node types each ~30-40 lines mirroring AgentNode) is the right call at this phase; abstraction extraction would force prop threading without clarity gain. Re-evaluate at M07–M08.
- **Pre-M01 addendum 🟡 "UI consistency: existing look and feel"** — **RESOLVED at M03 implementation level** (M03 inherits M02's SetupPanel/SmokeButton visual language; new components — InspectorPanel, SqlInspector, 11 node types — follow the same `:root` CSS custom property palette per M03.C styles.css; spec §3 Visual Design palette directly drives `--node-active/complete/error/gap/hitl`). M04 stage prompts must continue this discipline; re-listed in M04 carry-forward.
- **M01 🟡 "`mcp_servers` table — add now or document deferral"** — RESOLVED at M02.A per M02 entry.
- **M01 🟡 "Post-M01 `docs(spec):` PR"** — RESOLVED at PR #36 per M02 entry.
- **M01 🟡 "`*_with` test-seam pattern"** — RESOLVED at M02.A + EXTENDED at M03.E. M03.E added `query_session_db_with` + `replay_session_with` Tauri command seams + drone command-handler `handle_query_session_db` + `handle_read_signals` arms following the same archetype. Pattern is now demonstrated across SIX substrates (drone IPC named pipes, HTTP+SSE, in-process streams, Tauri commands × 2 milestones, drone command-handler arms).
- **M01 🟡 "Phase 3 React Flow + Zustand spec expansion"** (re-listed) — see Pre-M01 baseline above.
- **M01 🟡 "Windows drone integration test"** — RESOLVED at M02.A per M02 entry. Stage E added 2 more Unix subprocess roundtrip tests (`query_session_db_roundtrip_returns_rows`, `read_signals_roundtrip_preserves_ordering`).
- **M01 🟡 "`.gitattributes` line-ending normalization"** — STILL RESOLVED. M03 added no Windows line-ending warnings during stage executions.
- **M01 🟢 "Tauri release-build caching"** — STILL OPEN, advisory. M03.F's new `e2e-tauri-driver` job adds release-build pressure × 2 OS per PR push; if CI time becomes painful in M04+, revisit.
- **M02 🟡 "TS `AgentEvent` codegen from schema"** — RESOLVED at M03.A. `cargo xtask regenerate-types` produces `src/types/agent_event.ts` from `schemas/event.v1.json` via `json-schema-to-typescript`; CI drift check in `.github/workflows/ci.yml` keeps it byte-stable.
- **M02 🟡 "Tauri 2.x desktop-shell E2E via `tauri-driver` + WebdriverIO"** — RESOLVED at M03.F. WebdriverIO v9 + `tauri-driver` (cargo-installed) + Linux + Windows matrix + 6 E2E tests + macOS skip per upstream limitation. The four `test.skip()` carry-forwards from M02.E removed.
- **M02 🟡 "vitest `--coverage` enabled in default `test` script"** — RESOLVED at M03.A.
- **M02 🟡 "Decision extractor → structured emitter migration"** — STILL OPEN, M04-blocking. Re-listed in M03 🟡 backlog.
- **M02 🟡 "`count_tokens` → real `/v1/messages/count_tokens` endpoint"** — STILL OPEN, M04-blocking. Re-listed in M03 🟡 backlog.
- **M02 🟡 "`signal.rs::ContextType` reconcile with spec §2b"** — STILL DEFERRED to M04 closeout per the original deferral. M03 didn't write any signal-emission code; the divergence is dormant.
- **M02 🟡 "Post-M02 `docs(spec):` PR"** — STILL OPEN. Becoming "post-M03 `docs(spec):` PR" with M03 additions bundled (per M03 🟡 backlog above).
- **M02 🟡 "MCP-schema divergence ADR"** — STILL OPEN, target M06 prep. M03 didn't touch MCP code.
- **M02 🟡 "CLAUDE.md §15 / `docs/gotchas.md` consolidation"** — RESOLVED at the post-M02 protocol-iteration session (PR #43 / commit `e4641a7` per the git log: "docs(protocol): post-M02 iteration — gotchas + retrospective + template carry-forwards"). `docs/gotchas.md` is now at 31 entries; M03's `<gotchas_graduation>` adds 8 more = 39 candidates for the post-M03 protocol-iteration session.
- **M02 🟡 "`TEMPLATE.md` updates (Coverage holdouts subsection, default safety-primitive test plan, WEBCHECK: header, legacy-file inventory)"** — RESOLVED at the post-M02 protocol-iteration session (commit `e4641a7`); the v1.2 XML stage-prompt protocol that M03 was authored on includes these.
- **M02 🟡 "Phase 3 React Flow + Zustand spec expansion"** (re-listed) — see Pre-M01 baseline above; PARTIALLY RESOLVED at M03 implementation level; STILL OPEN at spec text level.
- **M02 🟡 "Session FSM diagram"** (re-listed) — STILL OPEN, M04-blocking.
- **M02 🟡 "UI consistency: existing look and feel"** (re-listed) — RESOLVED at M03 implementation level (see Pre-M01 addendum above).
- **M02 🟡 "Plan model field shapes"** — STILL OPEN, M04-blocking.
- **M02 🟡 "Retrofit `crates/runtime-drone/tests/integration*.rs` to `current_exe()`-derived paths"** — RESOLVED at M03.A per the M03.A retrospective ("Retrofitted `crates/runtime-drone/tests/integration.rs` and `integration_windows.rs` to derive paths from `std::env::current_exe()`").
- **M02 🟡 "Long-lived `events()` subscription survives reconnect"** — STILL OPEN, M04-blocking. M03.E's replay-from-signals path is one-shot on mount, not long-lived; the M02 concern is genuinely M04+ scope.
- **M02 🟡 "`error.v1.json` schema for `CmdError` wire format"** — STILL OPEN, M04 prep.
- **M02 🟢 "Vite 5.4 → 7+ bump"** — RESOLVED at M03.A.
- **M02 🟢 "Delete legacy `src/counter.{js,test.js}`"** — RESOLVED at M03.A.
- **M02 🟢 "`secrecy/serde` workspace feature"** — RESOLVED at M03.A.
- **M02 🟢 "`keyring 3.6 → 4.0` upgrade"** — STILL DEFERRED to M04+. M03.F's new CI job will start exercising real keychain calls × 2 OS; if the 3.6 stub-backend issue surfaces (unlikely with apt-installed `webkit2gtk-driver` pulling libsecret on Linux + Windows-native on `windows-latest`), M04 reopens.
- **M02 🟢 "Tauri release-build caching"** (re-listed) — STILL OPEN, advisory.

### Gotchas graduation (v1.2 protocol — first milestone to ship this section)

Per `STAGE-PROMPT-PROTOCOL.md` v1.2 + the closeout prompt's `<gap_analysis_requirements><gotchas_graduation>` subsection. Every per-stage `<gotchas>` entry across M03.A–F gets a disposition: **kept** | **graduated** | **resolved** | **expired**.

| Stage | Gotcha | Disposition | Target/Resolution |
|---|---|---|---|
| A | Scope creep into Stage B's React Flow work — resist | resolved | M03.A commit `bb8202e` (held the line; A landed without React Flow code per `git diff bb8202e^..bb8202e -- src/components/`). |
| A | secrecy/serde feature drop side effects — verify no callsite breaks | resolved | M03.A commit `bb8202e` (grep `secrecy::` across workspace returned zero callsites; feature dropped cleanly per Stage A retro Friction event 0). |
| A | TS codegen drift — output must be byte-stable across re-runs | kept | Stays in per-stage `<gotchas>` for M04+ when `xtask` Rust list extends to `event.v1.json` (deferred from Stage D as M04 Stage A carry-forward); the `cargo xtask regenerate-types --check` drift check in CI continues to enforce. |
| A | Vite 7 IPv6/IPv4 binding behavior — host: '127.0.0.1' fix needed | graduated | New `docs/gotchas.md` entry at protocol-iteration session ("Vite 7 binds the dev server to IPv6 when `host` is unset / `false`; pin `host: '127.0.0.1'` for cross-tooling DNS reliability — Playwright's bundled chromium uses IPv4 via Node 17+ DNS"). Surfaced as Stage A surprise event #1; recurs on any future fresh session that bumps Vite + uses Playwright. |
| A | Vite 7 dep-optimizer cold-start (~53s on Windows for Tauri/React stack) | graduated | New `docs/gotchas.md` entry at protocol-iteration session ("Vite 7 Rolldown-scout warmup runs 30–60s on a fresh `node_modules/.vite/deps` cache; Playwright per-test timeout + webServer timeout must absorb"). Surfaced as Stage A surprise event #2; pairs with Vite 7 IPv6 entry. |
| A | EventList non-exhaustiveness break after schema regen | expired | n/a — `EventList.tsx` was deleted in Stage B (replaced by GraphCanvas + node components per M03.B); the non-exhaustiveness break only existed at Stage A (regenerate types) → Stage B (delete file) transition. No forward applicability beyond M03.B. |
| B | nodeTypes map MUST be defined outside the component (re-render trap) | kept | Stays in per-stage `<gotchas>` of any M04+ stage that adds React Flow node types (M04 plan/task/verify/hook events may add wired cases for existing node types but unlikely to add new component types). The trap re-applies for any future fresh session adding to the `GraphCanvas.tsx` `nodeTypes` map. |
| B | Zustand v5 selector pattern (use selectors, not bare hook) | kept | Stays in per-stage `<gotchas>` of any M04+ stage that consumes graphStore from new components (M04 ApprovalPanel for plan approval; M04 BudgetPanel; M05 GapPanel). |
| B | React Flow + happy-dom act() warnings | graduated | New `docs/gotchas.md` entry at protocol-iteration session ("React Flow + happy-dom: synchronous Zustand dispatches in tests should be wrapped in `act()`; downstream React Flow internal layout-effect warnings (MarkerDefinitions, EdgeRendererComponent, Pane, ZoomPane) are unavoidable noise — accept, don't chase"). Recurred in M03.B + M03.C. |
| B | Trust narrowing — discriminated-union narrowing produces correct type after .type === 'X' filter; no assertion needed | graduated | New `docs/gotchas.md` entry at protocol-iteration session ("TS discriminated-union narrowing after `.type === 'X'` filter produces the correct type; explicit `as TheNode` assertions are dead weight + trip `@typescript-eslint/no-unnecessary-type-assertion`"). Recurred in M03.B + M03.D. |
| B | RTL DOM-ref staleness after awaited re-render (gotchas.md #27) | kept | Stays in per-stage `<gotchas>` for any M04+ stage adding renderer-state-machine tests with awaited re-renders. Already in `docs/gotchas.md` #27 from M02.E; M03.B re-validated it. |
| C | Scope creep into M4 territory — Gap/HITL/Plan/Task/Verify/Hook event wiring | resolved | M03.C commit `5dbc138` (held the line; the 6 event-less components ship with synthetic-state testing pattern + unwired event handlers per Stage C scope locks). M04 owns wiring `plan_*`/`task_*`/`verify_*`/`hook_*`/`hitl_*`; M05 owns `gap_*`. |
| C | Synthetic-state testing pattern locked here | graduated | New `docs/gotchas.md` entry at protocol-iteration session ("For renderer components without yet-defined event variants, prefer synthetic-state testing — pass populated state directly into `<NodeComponent>` rather than dispatching events through the store. Locks the contract pre-M4/M5 wiring without test churn."). Pattern proven across M03.C (initial application) + M03.D (durability check); forward-applicable to M04+/M05. |
| C | Exhaustive applyEvent — `_exhaustive: never` switch as schema-drift forcing function | kept | Stays in per-stage `<gotchas>` of any M04+ stage that adds new wired cases to `graphStore.applyEvent`. The exhaustiveness check naturally forces "either handle every variant of the discriminated union or compile error". |
| C | nodeTypes map module-level (continued from B with 11 entries now) | kept | Same as Stage B's entry — stays in per-stage `<gotchas>` for any M04+ stage extending the map (unlikely but possible). |
| C | ARIA per WAI APG (HITLNode role="alert" + aria-live="assertive"; GapNode data-kind + gap-pulse keyframe) | kept | Stays in per-stage `<gotchas>` for M04+/M05 stages that wire these components to events; the ARIA attributes were chosen for accessibility + semantic correctness, must be preserved when the components become event-driven. |
| C | Color palette in :root CSS custom properties | kept | Stays in per-stage `<gotchas>` for any M04+ stage adding new node types or status enums; the `:root` palette is the single source of color truth + must be extended (not bypassed via per-component overrides). |
| D | Schema bump → run `cargo xtask regenerate-types` | kept | Stays in per-stage `<gotchas>` of any M04+ stage that bumps `schemas/event.v1.json` (likely M04 Stage A or wherever plan/task/verify/hook events land). The xtask's drift check enforces. |
| D | `event.v1.json` NOT in Rust typify list — `event.rs` hand-edited | kept | Stays in per-stage `<gotchas>` until M04 Stage A extends the typify list (M04 carry-forward). After M04.A, this entry can be `resolved`. |
| D | Layout in `useEffect`/`useMemo`, not in store (visualization vs state) | kept | Stays in per-stage `<gotchas>` for any M04+ stage adding new visualization layers (e.g., budget visualization, plan timeline). The store stays state-only; visualization computation in component-side hooks. |
| D | CSS `transform: scale()` for token weight (perf > JS-computed sizing) | kept | Stays in per-stage `<gotchas>` for any M04+ stage adding visual emphasis to nodes (budget thresholds, capability badges). CSS transforms beat JS-computed sizing for both performance + animation smoothness. |
| D | `aria-modal="false"` on InspectorPanel (informational, not blocking) | kept | Stays in per-stage `<gotchas>` for any M04+ stage adding panels (ApprovalPanel for plan approval, BudgetPanel, GapPanel). Modal dialogs block input affordance; informational dialogs (inspector-style) should be `aria-modal="false"`. |
| D | Existing graphStore tests must pass unchanged after token-tracking edits | resolved | M03.D commit `489d2e5` (token-tracking is additive — toMatchObject subset semantics tolerate the new fields; existing assertions hold). The ad-hoc fixture updates in AgentNode.test.tsx + ToolNode.test.tsx (3 + 2 lines respectively) were forced by direct `baseData` construction not subset matching. |
| D | Snake_case schema field names (recurrence from A + B) | graduated | New `docs/gotchas.md` entry at protocol-iteration session ("snake_case schema discipline: schemas use snake_case (`tokens_in`, `agent_id`, `duration_ms`); generated TS preserves it; renderer-side data interfaces translate to camelCase at the reducer boundary; do NOT hand-mirror snake_case-vs-camelCase across the boundary. Trust generated TS over prompt sample bodies."). Recurred 3 stages of M03 (A, B, D) — strongest forward-applicability signal. |
| D | Bulk-fixture-script fan-out for struct/enum extension (Stage D Sev 3 friction) | graduated | New `docs/gotchas.md` entry at protocol-iteration session ("Extending a struct/enum/interface shape: pre-flight grep for every existing usage (every match arm + construction site + test fixture) BEFORE making the change. The schema-bump-rippling-into-21-fixture-sites pattern recurs in M04 (plan/task event additions), M05 (gap event additions), M06 (MCP server-id field additions)."). Surfaced as Stage D Sev 3 friction event 1; pattern lesson generalizes. |
| D | `prettier --write` / `cargo fmt --all` as first verify_gates step (recurrence from B + C) | graduated | New `docs/gotchas.md` entry at protocol-iteration session ("On any multi-file authoring stage, run `prettier --write` (TS) / `cargo fmt --all` (Rust) as the FIRST `verify_gates` step. Catches mechanical formatting before downstream lints depend on the formatted output. Pattern from M03.B/C/D/E: every stage's first round-1 friction was 1–9 files tripping formatting."). Recurred in M03.B + M03.C + M03.D + M03.E — four stages. |
| E | SELECT-only validation parser-based (the prompt's draft) → lexical-only (actually shipped) | resolved | M03.E commit `b0421bb` (`vdr.rs::is_select_only` ships lexical analysis: empty rejection + compound-semicolon rejection + lowercase-`select` allowlist + PRAGMA rejection). The `Connection::open_in_memory().prepare()` approach in the prompt's draft is structurally wrong (in-memory probe DB has no schema; rejects every legitimate SELECT). Recommendation in M03.E retro Decisions: revise §E.3 sample body before milestone PR opens. |
| E | VDR projection drone-INTERNAL not main-side IPC | kept | Stays in per-stage `<gotchas>` for M04+ stages adding signal-write code. The drone-internal projection model (per spec §1) is the locked architecture; M04's `WriteSignal` command-handler arm should call `vdr::project_signal` directly rather than IPC roundtrip. |
| E | `Connection::open_in_memory().prepare()` schema-aware behavior surprise | resolved | M03.E commit `b0421bb` (dropped the prepare-step-based validation; lexical-only is correct + simpler + sufficient per the security boundary). |
| E | localStorage scope (webview-only; sufficient for v0.1 single-instance) | kept | Stays in per-stage `<gotchas>` for any future stage that considers cross-instance state (v1.0 multi-session per spec §1c). v0.1 single-instance use is fine; v1.0 may need SQLite-side persistence. |
| E | Request/response over single-consumer event stream — explicit filtering needed | kept | Stays in per-stage `<gotchas>` for M04+ stages adding new request/response IPC patterns (M04 session-lifecycle, M05 gap-flow). The `await_event` filter pattern + 5-second timeout + Heartbeat skip is the archetype. |
| E | Hand-rolled Row → JSON conversion (no new third-party crate per CLAUDE.md §6) | resolved | M03.E commit `b0421bb` (`vdr::execute_select` ships ~60 lines of hand-rolled `rusqlite::ValueRef` → `serde_json::Value` conversion across NULL / Integer / Real / Text / Blob; no `serde_rusqlite` dep added). Pattern reinforces CLAUDE.md §6. |
| E | App.test.tsx localStorage cleanup needed for replay-on-mount path | resolved | M03.E commit `b0421bb` (`tests/unit/App.test.tsx` `beforeEach`/`afterEach` clear `localStorage.lastSessionId`; existing test `surfaces_command_error_via_error_paragraph` continues to pass with the additional mount-time IPC call). |
| E | Drone projector call-site is at signal-write code (M04+), NOT heartbeat (Sev 2 prompt drift surfaced as Ambiguity event) | resolved | M03.E commit `b0421bb` skipped the §E.2-listed `heartbeat.rs` edit (heartbeat doesn't write signals); documented in `vdr.rs` doc comment that the projector is exercised by tests in v0.1 + future M04+ signal-write call-sites will invoke it. The architectural decision (drone-internal projection) is preserved; only the prompt's `heartbeat.rs` reference was a misnomer. |
| E | MSRV-incompatible stdlib API check before using recent stabilizations | graduated | New `docs/gotchas.md` entry at protocol-iteration session ("Before using a recent stdlib API like `Option::is_none_or` (Rust 1.82) / `Result::is_err_and` / similar, check `cargo +msrv check` or the [stabilized-since] markers — clippy's `incompatible_msrv` only fires on the gated lint set. The project MSRV is 1.80." Recurred from M02). |
| E | Locked .exe from orphan drone subprocess on test re-run | kept | Stays in per-stage `<gotchas>` for any M04+ stage that runs `cargo test` against tests spawning subprocesses (drone integration tests, future runtime-sandbox tests). The orphan-cleanup discipline (Get-Process before re-running) is the workaround. |
| F | Carry-forward section required even when empty (write "None observed.") | kept | Stays as a closeout-stage `<gotchas>` discipline for M04+ closeouts. The "required even when empty" rule per CLAUDE.md §3.4 + §20 is structural to gap-analysis. |
| F | Severity non-elastic — surface 🔴 Critical findings rather than rationalize down | kept | Stays as a closeout-stage `<gotchas>` discipline for M04+ closeouts. Severity inflation degrades the prioritization signal. |
| F | `expired` disposition requires rationale beyond bare `n/a` | kept | Stays as a closeout-stage `<gotchas>` discipline. The v1.2 protocol validator (when shipped) checks rationale length; manual authoring per M03.F (this entry) honors the rule. |
| F | tauri-driver's `tests/e2e-tauri/` directory must NOT be confused with `tests/e2e/` (Playwright) | kept | Stays in per-stage `<gotchas>` for any M04+ stage that touches either E2E suite. Two test types, two CI jobs, two config files — must not be merged. |
| F | wdio.conf.ts macOS early-exit (`process.exit(0)`) for local dev guard | kept | Stays in per-stage `<gotchas>` if any M04+ stage modifies `wdio.conf.ts`. The guard exists for local Mac development; CI already skips the whole job on macos-latest. |
| F | Three-artifact review pushback BLOCKS the PR push | kept | Stays as a closeout-stage `<gotchas>` discipline for M04+ closeouts. The user's pushback on any of (code diff, retros/summary, gap-analysis entry) blocks until revised. |
| F | WebdriverIO v9 chainable pattern — `await $('selector')` doesn't work in v9 | graduated | New `docs/gotchas.md` entry at protocol-iteration session ("WebdriverIO v9 chainable pattern: `$()` and `$$()` return chainables that are NOT `PromiseLike`; call methods directly on the chainable without intermediate `await`; `$$().length` is `Promise<number>`; use `import type {} from 'webdriverio'` side-effect import to bring `WebdriverIO.Browser` augmentations into scope; @wdio/globals leaves `Browser` interface empty by design"). Surfaced as Stage F Sev 2 friction event 2; forward-applicable to any future stage touching WebdriverIO. |
| F | npm `overrides` for transitive audit failures | graduated | New `docs/gotchas.md` entry at protocol-iteration session ("When transitive deps trip `npm audit --audit-level=high` and an upstream patch exists, prefer npm `overrides` to force the patched version vs. swapping the dep entirely. Verify upstream API compatibility post-install. Pattern: `package.json` `overrides.<dep-name>: <patched-version-range>`"). Surfaced as Stage F Sev 2 friction event 3; generalizes. |
| F | eslint config-glob `**/*.config.{ts,js}` doesn't match `.conf.ts` | graduated | New `docs/gotchas.md` entry at protocol-iteration session ("`eslint.config.js` overrides matching `**/*.config.{ts,js}` do NOT match `wdio.conf.ts` (`.conf.ts` not `.config.ts`); add explicit filename to override list + set `parserOptions.projectService: false` for files outside the tsconfig include"). Surfaced as Stage F Sev 1 friction event 4. |

**Summary:** 28 stage-gotcha entries across A–F + 18 stage-friction-event entries (logged as `<gotchas>` would have caught them) → 46 total dispositions:
- **resolved:** 11 (stage-local fixes that committed and have no forward applicability — e.g., scope-creep guards held, secrecy/serde dropped, EventList deleted, schema-bump fan-out one-time fix).
- **graduated:** 12 (recurring or forward-applicable patterns now in `docs/gotchas.md`).
- **kept:** 21 (still apply forward; stay in per-stage `<gotchas>` of M04+ stages that touch the same surface).
- **expired:** 2 (stage-local with explicit rationale why they don't apply forward — e.g., EventList non-exhaustiveness break only existed at A→B transition).

The 12 graduations populate `docs/gotchas.md` from the existing 31 entries → 43 entries at the post-M03 protocol-iteration session.

### Sign-off

**Claude:** I have generated this gap analysis after the final implementation stage of M03 (Stage F — this commit). This is my honest assessment of the cumulative code-vs-spec state across M01 + M02 + M03. Hard gates G1–G5 cleared in all six M03 stages; no Critical-severity findings. Aggregate axis means healthy (Process 38.83/40, Product 37.67/40, Pattern 30.00/35); time-box calibration drift continues at ~0.32× (M01 0.3× / M02 0.7× / M03 0.32×). The largest 🟡 backlog clusters are the M04-prep production-drone-subprocess wiring + `vdr.rs` projector activation + xtask Rust typify list extension + `client.rs` per-module coverage close + post-M03 `docs(spec):` PR + `docs/gotchas.md` consolidation (12 graduations from this `<gotchas_graduation>`) + STAGE-PROMPT-PROTOCOL.md v1.3 protocol-iteration. The first-of-its-kind `<gotchas_graduation>` subsection (v1.2 protocol enforcement) covers 28 per-stage gotchas + 18 stage-friction events across A–F with disposition. User review pending; per `CLAUDE.md` §20 this entry is **immutable** once committed — future milestones report status updates via their Carry-forward sections.

**Surfaced at:** 2026-05-06 (UTC).

---

## M04 — Plan + Verify + HITL + Budget (2026-05-11, Stage G commit on parent-milestone branch `claude/m04-plan-verify-hitl-budget`)

> Author: Claude (per `CLAUDE.md` §20)
> Stages aggregated: M04.A1 (build hygiene + xtask codegen extensions + client.rs await_event timeout coverage), M04.A2 (production wiring — drone subprocess at Tauri startup + count_tokens real endpoint + CmdError migration + events() reconnect lock), M04.B (§3a Plan & Task primitive — schemas + FSM + projection + WriteSignal IPC + structured emitter + migration runner), M04.C (Plan UI — ApprovalPanel non-modal + plan/task node visual upgrades + 3 Tauri commands resolving in-process ApprovalSeam), M04.D (§4a Verify & Rails — hooks/dont_touch/rails/executor primitive + pre_file_edit firing point + revert_to_snapshot consumption), M04.E (§6a HITL primitive — 9 triggers + 3 UI variants + 3 notifiers + in-process HitlSeam + tauri-plugin-notification), M04.F (§2a Budget — enforcer + cost cache + downshift_hook + §1b Recovery — resume + uncertainty + MCP no-op seam).
> Reviewed against: `agent-runtime-spec.md` §0–§0d, §1b, §1d, §2a, §2b, §2c, §3a, §4a, §6a, §10, §11, §13; `schemas/*.v1.json` (agent, common, framework, skill, tool, event, plan, task, hitl, budget, error); M04.A1–F retrospectives + `M04-summary.md`; M01 + M02 + M03 entries' Carry-forward backlogs (Pre-M01 baseline + Pre-M01 addendum + M01 + M02 + M03); M03.5 protocol-iteration outputs (v1.3 STAGE-PROMPT-PROTOCOL.md tags + the 12 `docs/gotchas.md` graduations from M03 closeout).
> Second milestone authored on the v1.2 XML stage-prompt protocol (the v1.3 candidates from M03.5 were applied at stage authoring); first milestone to ship five new safety-primitive Rust modules in a single parent-milestone branch (plan, hooks, hitl, budget, recovery). Cumulative diff 27,034 / 809 across 129 files.

### Codebase deep dive (cumulative — M01 + M02 + M03 + M04)

M04 turns the M03 live graph + persistence foundation into a working orchestrator surface: plan + task state machines with append-only projection; the 9-trigger HITL primitive with seam-resolved in-process round-trip + 3 notifier surfaces + 3 panel surfaces; the §4a verify hooks + JSONLogic-allowlisted rails + globset-backed don't-touch evaluator; the §2a budget enforcer with 4 threshold actions + cost cache + opus→sonnet→haiku downshift ladder; the §1b recovery primitive with resume-rebuilds-history (per gotcha #15) + tool-call-uncertain 4-action prompt; and the §1d events() reconnect locked behavior (subscriptions do NOT survive, renderers resubscribe).

- **Production drone subprocess + Tauri-managed state** (Stage A2). `src-tauri/src/drone_lifecycle.rs` (NEW, 468 lines) spawns `runtime-drone` via `tokio::process::Command::kill_on_drop(true)` + computes the platform-specific IPC address (Unix socket / Windows named pipe) + manages `Arc<DroneClient>` through Tauri's setup hook. `src-tauri/src/main.rs:55-56` registers the client at app-handle time; every command takes `tauri::State<'_, Arc<DroneClient>>` (or downstream seam handles). M03's `DroneClient::noop()` production-wiring deferral CLOSED. Production end-to-end persistence path now wires through the real subprocess — `query_session_db`, `replay_session`, `WriteSignal`, `RecoverSession` all operate on real SQLite + signal-stream data.

- **`runtime_core::generated::error::CmdError` migration** (Stage A1 + A2). Schemas-as-source-of-truth pipeline extends to `error.v1.json` (new), generated to `crates/runtime-core/src/generated/error.rs` (typify) + `src/types/error.ts` (json-schema-to-typescript). M02-era hand-rolled `CmdError` with `thiserror::Error` struct-variants replaced by the typify-emitted tuple-variant-over-`ErrorMessage`-newtype shape. `crates/runtime-core/src/cmd_error_ext.rs` (NEW, 233 lines, 100% coverage) supplies inherent helper constructors (`provider`, `drone`, `key_store`, `internal`, `message`) + `Display` + `std::error::Error` impls in a separate module (orphan-rule satisfied because both types are local to `runtime-core`). `From<KeyStoreError> for CmdError` placed in `runtime-main/src/key_store.rs` (orphan-rule: KeyStoreError is local there). 17-callsite mechanical migration across `src-tauri/src/commands.rs`; wire format unchanged.

- **Real `/v1/messages/count_tokens` endpoint** (Stage A2). M02's chars/4 approximation in `crates/runtime-main/src/providers/anthropic.rs::count_tokens` replaced by a real POST against `https://api.anthropic.com/v1/messages/count_tokens`. New types `CountTokensRequest` + `CountTokensResponse` mirror Anthropic's documented wire format. 4 new wiremock tests cover happy path / auth / rate-limit / malformed-body. M04 budget integration consumes the real number.

- **Plan + Task FSMs + structured emitter + WriteSignal IPC + migration runner** (Stage B). `schemas/plan.v1.json` + `schemas/task.v1.json` + generated mirrors at `crates/runtime-core/src/generated/{plan,task}.rs` + `src/types/{plan,task}.ts`. `crates/runtime-main/src/plan/state_machine.rs` (NEW, 592 lines, 99.28% line, 28 unit tests) is an exhaustive transition matrix covering 8 plan states + 6 task states with idempotence + escalation-boundary tests. `crates/runtime-drone/src/plan_projector.rs` (NEW, 783 lines, 97.88% line, 18 unit tests) consumes the 11 plan/task event variants from the signal stream → UPSERT-with-CASE into `plans` + `tasks` tables (idempotent + out-of-order tolerant). `crates/runtime-main/src/sdk/approval.rs` (NEW, 308 lines, 99.02% line, 11 unit tests) is the `tokio::sync::oneshot`-backed `ApprovalSeam` — the in-process correlation between the renderer's approve/revise/abort click and the SDK's awaiter. `crates/runtime-main/src/sdk/structured_emitter.rs` (NEW, 544 lines, 95.92% line) replaces M02's `decision_extractor` line-by-line heuristic with a parser-based extractor that ONLY accepts `<<DECISION>>...<<END>>` / `<<RATIONALE>>...<<END>>` / `<<TOOL>>...<<END>>` blocks (false-positive elimination — the load-bearing M02-closure invariant). `DroneCommand::WriteSignal { signal_id, kind, body }` IPC variant + drone-side `handle_write_signal` arm + main-side `DroneClient::write_signal` method. `crates/runtime-drone/migrations/{000_initial,001_plans_tasks}.sql` + `migration_runner.rs` replace inline `init_schema` — first stage of the migrations-as-source-of-truth pattern; M5+ stages add migrations not edits.

- **Approval panel + plan/task visual upgrades** (Stage C). `src/components/ApprovalPanel.tsx` (NEW, 218 lines, 98% line, 10 unit tests) is the non-modal `aria-modal="false"` plan-approval surface — three actions (approve/revise/abort) with idle-vs-draft state pattern. `approve_plan`, `revise_plan`, `abort_plan` Tauri commands resolve the in-process `Arc<ApprovalSeam>` directly (no drone IPC round-trip — cross-process oneshots don't exist; the in-process resolution is the structurally-correct architecture). Stage C's GraphCanvas memoization bug fix (positions-by-id Map vs M03.D's count-keyed memo) is a load-bearing correction for any future stage adding visually-driven data updates to nodes.

- **Verify hooks + Rails + don't-touch primitive** (Stage D). `crates/runtime-main/src/hooks/{mod,executor,rails,dont_touch,shell}.rs` (NEW, ~1,580 lines aggregate; per-module coverage 87–98%). `executor.rs` wraps shell-spawn (cross-platform `bash` / `powershell` per gotcha #32) + maps the exit code → spec §4a `on_failure` action (rollback / continue / abort_task). `rails.rs` implements a JSONLogic interpreter over a typed 15-operator allowlist — typed enum precondition that fails closed (`UnsupportedOperator` for anything outside the allowlist). `dont_touch.rs` is a globset-backed cross-platform case-insensitive matcher that rejects writes to framework-protected paths. `shell.rs::TokioShellSpawner::spawn` is the OS-spawn holdout (testable seam at 100% coverage). `RevertReason::HookRollback { hook_id }` struct-variant + `RevertReason::UserRollback` extend `runtime_core::DroneCommand::RevertToSnapshot`. The Hook + HookRef + Rail types are consumed from `runtime_core::generated::{common, framework}` (already authored in those schemas pre-M04; the audit gotcha caught the assumption). Six event variants extended to match spec §4a: `verify_started/passed/failed`, `hook_started/passed/failed`, `rail_triggered`. Schema edits to `event.v1.json` + `framework.v1.json` (adds `pre_file_edit` firing point per spec §4a).

- **HITL primitive — 9 triggers, 3 UI variants, 3 notifiers** (Stage E). `schemas/hitl.v1.json` + generated mirrors. `crates/runtime-main/src/hitl/{mod,seam,policy,notifiers/{mod,desktop,sound,terminal_bell}}.rs` (NEW, ~1,950 lines aggregate; safety-primitive coverage 92–100% per module). `seam.rs` mirrors `ApprovalSeam` — `HitlSeam` is the in-process correlation seam with `(prompt_id, choice) → resolution` semantics + 1-hour timeout + ReceiverDropped path. `policy.rs` is the 9-trigger evaluator with per-trigger precondition table — `BudgetThreshold(percent)`, `OnFailureRollback(failure_count, max_failures)`, `OnPlanRejected`, etc. — each with documented evaluation rules + 22 unit tests covering exhaustive happy path + UI override + missing/disabled paths + matches_tool variants. `notifiers/` ships 3 built-ins per spec §6a: `terminal_bell.rs` (`\x07` BEL emission), `sound.rs` (system sound via cross-platform shell), `desktop.rs` (Tauri 2.x notification plugin with permission flow). All three are NOT-FATAL — notifier failures log and continue (the spec rule). `crates/runtime-main/tests/hitl_failure_escalation.rs` (NEW, 216 lines, 3 integration tests) drives the full lifecycle: 3 failures → policy fires → escalation → notifier dispatches → seam resolves. `src/components/{HITLPanel,HITLModal,HITLToast}.tsx` (NEW, 3 components, 100% coverage each, 32 unit tests aggregate) are the 3 spec §6a UI variants. `respond_hitl` Tauri command resolves the in-process `Arc<HitlSeam>` directly (mirrors Stage C). Stage E REPLACED the pre-existing provisional `HitlRequested`/`HitlResolved` minimal shape with the spec §6a `HitlNotifyEvent`-aligned shape (no live producers existed pre-Stage E — verified via grep audit).

- **Budget enforcer + Recovery primitive** (Stage F). `schemas/budget.v1.json` + generated mirrors. `crates/runtime-main/src/budget/{mod,cost,enforcer,hook}.rs` (NEW, ~1,200 lines aggregate; safety-primitive coverage 98.90–100% per module). `enforcer.rs` is the 3-scope tightest-cap-wins evaluator: 4 actions (Warn/Downshift/Suspend/HardStop) fire in firing-order with idempotence (re-recording the same percent does not re-fire). `cost.rs` is an LRU-keyed `CostCache` (capacity-0 disables cleanly). `hook.rs` implements the spec §2a hardcoded `DefaultLadder` (opus→sonnet→haiku with the Sonnet→Haiku-only-when-remaining<10%-AND-avg-task-cost>remaining/3 conditional). `crates/runtime-main/src/recovery/{mod,resume,uncertainty}.rs` (NEW, ~525 lines aggregate; 96.46–98.48% per module). `resume.rs` is the resume coordinator — async callback pattern returns `ResumePlan { snapshot_id, plans, tasks, uncertain_tool_invocations, has_state }`; v0.1 ships the MCP-no-op seam (M5/M6 wire real path). `uncertainty.rs` implements the 4 spec §1b actions (Retry/Skip/MarkComplete/Abort) emitting `tool_call_uncertainty_resolved` decision signals. New IPC pair `DroneCommand::RecoverSession { session_id }` + `DroneEvent::SessionRecovered`. `src/components/{BudgetHeaderBar,RecoveryDialog,UncertaintyPrompt}.tsx` (NEW) — color-gradient sticky header / cold-start recovery prompt / 4-action uncertainty modal. `set_global_budget`, `request_resume`, `respond_uncertainty` Tauri commands.

What's solid: M04 lands FIVE new safety-primitive Rust modules (plan/, hooks/, hitl/, budget/, recovery/) each at ≥95% line coverage, every one following the `*_with` testable-seam archetype now in its tenth+ substrate aggregate from M01–M04; the in-process seam architecture (ApprovalSeam + HitlSeam via `Arc<T>` in Tauri-managed state, resolved by Tauri commands) is now demonstrated in two places with the same shape and is the established v0.1 pattern for renderer→backend correlation flows; the schema-as-source-of-truth pipeline now covers 11 schemas (added plan, task, hitl, budget, error in M04) with the xtask Rust typify list extended in Stage A1; the migrations-as-source-of-truth pattern landed in Stage B replacing inline `init_schema`; the structured emitter eliminates M02's false-positive class of decision-record extraction via parse-only-blocks contract; the M02 `decision_extractor` heuristic carry-forward and M02 `count_tokens` chars/4 carry-forward are both fully closed at the code level; the production drone subprocess is now spawned at Tauri startup via `Arc<DroneClient>` managed state, closing the M03 production-wiring deferral; the §1d long-lived events() reconnect note is now locked at the integration test level (subscriptions do NOT survive reconnect — renderers resubscribe — verified by `crates/runtime-main/tests/drone_reconnect_events.rs`); zero-telemetry boundary held end-to-end across all seven stages (verified at every audit step including Stage E's Tauri notification plugin add); capability lockdown at the Tauri allowlist boundary held end-to-end (`notification:default` added at Stage E for the desktop notifier; no `shell:*`, `fs:*`, `http:*`, `dialog:*` added); the `[END] Decisions for the next stage` discipline is functioning as a hard checklist — A1 Decisions fed A2 100%; A2 fed B 100%; B fed C 100%; C fed D 100%; D fed E 100%; E fed F 100% (six consecutive demonstrations across M04, on top of M03's five).

What's structurally weak and likely to compound: (1) **Budget event shapes diverge from spec §2a** in three ways — `scope` field missing from all 4 variants; `spent_usd`/`cap_usd` missing from `BudgetDownshift`; `budget_warn` vs spec's `budget_warning` discriminator. Stage F WIRES the existing names/shapes deliberately per CLAUDE.md §12 (touching public event shapes would balloon scope mid-milestone); reshape is a post-M04 spec PR + minor-version-bump candidate. Renderer adapts by preserving the prior `BudgetState` snapshot when `BudgetDownshift` arrives. (2) **Downshift hook framework-tool-dispatch deferred** — `framework.budget.downshift_hook.tool_name` is in the schema and parsed into `BudgetPolicy::downshift_hook` but no code path dispatches it; v0.1 uses hardcoded `DefaultLadder`. M5/M9 generators wire the framework-tool-dispatch surface. (3) **`plan_loop.rs` driver deferred to M07** (per Stage B + Stage C decisions) — FSM + ApprovalSeam + structured_emitter ARE the spec §3a primitives that v0.1 emits; the driver that ties them to agent-loop execution has no callsite without the framework JSON loader (M07 territory). (4) **`DontTouchEvaluator` + Write-tool dispatcher integration site deferred to M05** — Stage D ships `DontTouchEvaluator::evaluate` as a callable primitive with 9 unit tests; v0.1 has no central Write tool dispatcher in `runtime-main/src/sdk/` (no tool execution surface yet — that's M05 capability enforcer scope per spec §4a + §8). (5) **HITL event payload carries `hook_id + category + firing_point`** rather than spec §4a's `ref: HookRef` — recoverable from framework JSON cache when M07 lands; deliberate scope-down. (6) **`UncertainInvocation.toolName` field carried but only `invocationId` populated end-to-end** — drone-side recovery surfaces the stranded `tool_invoked` signal id; correlating back to the original tool name + input payload requires a separate signals lookup deferred to M05+. (7) **`crates/runtime-main/src/drone_ipc/client.rs` per-module coverage regression** — M04.A1 closed the M03 carry-forward (94.00% → 96.75% via `await_event_timeout` test); Stage F's new `recover_session` IPC method paths drop the file to 89.45–89.90% (with `--skip recovery_lifecycle`). The integration test (`recovery_lifecycle.rs`) covers the path against a real drone subprocess on Linux CI but flakes under Windows-local llvm-cov parallelism. Per CLAUDE.md §5 documented coverage regression with deterministic-unit-test follow-up in M05+ (`tokio::io::duplex` + `tokio::time::pause()` archetypes apply). (8) **Windows-local `cargo llvm-cov` flake on subprocess-spawning integration tests** — `recovery_lifecycle.rs`, `plan_recovery.rs`, `drone_reconnect_events.rs` all flake under llvm-cov instrumentation when default parallelism is used; documented mitigation: `-- --test-threads=1 --skip recovery_lifecycle --skip plan_recovery --skip drone_reconnect`. Linux CI not affected. Developer-experience cost only. (9) **Phase doc inventory + architecture drift recurred across stages** — M04.A2 caught WriteSignal/prompt_template/event_translation/src-tauri/lib.rs references that don't exist (Sev 4 friction); M04.B + C + D + E + F each surfaced HOW-claim mismatches (cross-process flow assumptions that the codebase couldn't satisfy). The gotcha #41 grep-verify-claims rule catches WHAT-claims; HOW-claims need a separate `<architecture_check>` slot (carry-forward to a v1.4 protocol-iteration session). (10) **`<schema_audit>` corollary to gotcha #41** — M04.D missed that Hook + HookRef + Rail were already authored in common.v1.json + framework.v1.json; M04.F audit-baseline event names drift (`budget_warning` vs `budget_warn`). Authoring-time audit of spec-section $defs across all existing schemas would close this; carry-forward.

What surprised: the calibration drift held but loosened — M04 mean ratio ~0.55× (M03 0.32× / M02 0.7× / M01 0.3×); higher than M03 because M04 introduced multiple novel architectures (plan FSM, structured emitter, HITL policy evaluator, budget enforcer, recovery primitive) rather than M03's wide-but-shallow renderer surface. The `*_with` archetype's coverage-multiplier property became visible across M04 — every safety primitive hit ≥95% line on first complete test pass after Stage B's pattern-locked discipline; no follow-up coverage rounds needed for hitl/, budget/, recovery/. The Tauri 2.x ecosystem has stabilized since M03 PR #47's churn — Stage E's notification plugin integration was textbook-clean with zero version-pin or API-drift iterations (the WEBCHECK verbatim-quote discipline did its job when the upstream is well-maintained). The phase-doc-vs-codebase drift problem recurred 5 stages of M04 (A2, B, C, D, E, F all surfaced HOW-claim or WHAT-claim mismatches); the v1.3 `<gotchas>` audit-baseline pattern was load-bearing but incomplete — needs the v1.4 `<architecture_check>` + `<schema_audit>` slot proposals to fully close.

### Adherence to spec

- ✅ **§3a Plan + Task primitive — exhaustive FSM, projection, schemas, events.** Code at `crates/runtime-main/src/plan/state_machine.rs` (8 plan states + 6 task states with idempotence + escalation-boundary; 28 unit tests); `crates/runtime-drone/src/plan_projector.rs` (11 event variants projected to `plans` + `tasks` tables, idempotent + out-of-order tolerant); `schemas/{plan,task}.v1.json` + generated mirrors; 11 plan/task event variants in `event.v1.json`. `crates/runtime-main/src/sdk/approval.rs` is the spec-aligned `ApprovalSeam` (tokio::sync::oneshot-backed in-process correlation).
- ✅ **§3a Plan approval UI — ApprovalPanel non-modal + 3 in-process Tauri commands.** Code at `src/components/ApprovalPanel.tsx` (`aria-modal="false"` + ARIA-compliant + ESC dismiss + 3 actions in idle-vs-draft state pattern); `src-tauri/src/commands.rs` `approve_plan`/`revise_plan`/`abort_plan` each resolve `Arc<ApprovalSeam>` directly via Tauri-managed state.
- ✅ **§4a Verify hooks — post_task, pre_commit, post_file_edit, pre_file_edit.** Code at `crates/runtime-main/src/hooks/{mod,executor,rails,dont_touch,shell}.rs`. Schema edits to `framework.v1.json` add the `pre_file_edit` firing point. 6 event variants (`verify_started/passed/failed`, `hook_started/passed/failed`, `rail_triggered`) match spec §4a field set.
- ✅ **§4a Rails JSONLogic-allowlisted + don't-touch globsets.** `rails.rs` 15-operator typed allowlist; `dont_touch.rs` globset-backed cross-platform case-insensitive matcher; 9 + 24 unit tests cover the contract.
- ✅ **§4a `RevertToSnapshot` drone command** — extended at Stage D with `RevertReason::HookRollback { hook_id }` struct-variant; the drone-side `handle_revert` consumes `&RevertReason` reason-variant-agnostic. Same dispatcher serves Stage F recovery (`RevertReason::UserRollback`) — variant-agnostic by design.
- ✅ **§6a HITL primitive — 9 triggers, 3 UI variants, 3 notifiers.** Code at `crates/runtime-main/src/hitl/{mod,seam,policy,notifiers/*}.rs`. 9-trigger exhaustive policy table at `policy.rs` with per-trigger precondition rules + UI variant defaults per spec §6a table. 3 UI variants at `src/components/HITL{Panel,Modal,Toast}.tsx`. 3 notifiers with NON-FATAL failure semantics. `HitlSeam` matches `ApprovalSeam` archetype — in-process oneshot correlation with 1-hour timeout. `respond_hitl` Tauri command resolves `Arc<HitlSeam>` directly via Tauri-managed state.
- ✅ **§2a Budget enforcer — all 4 actions + downshift_hook + tightest-cap-wins.** Code at `crates/runtime-main/src/budget/{enforcer,cost,hook}.rs`. 4 threshold actions (`Warn`/`Downshift`/`Suspend`/`HardStop`) emitted in firing order with idempotence — re-recording at the same percent does not re-fire. 3-scope tightest-cap-wins via `record_spend_with_scopes(incremental, framework_total, global_total)`. `DefaultLadder` implements spec §2a opus→sonnet→haiku rule including the conditional Sonnet→Haiku gate. Cost-aware cache via `CostCache` LRU keyed by stable content hash.
- ✅ **§1b Recovery from snapshot — resume rebuilds history (not re-execute) + 4 uncertainty actions.** Code at `crates/runtime-main/src/recovery/{resume,uncertainty}.rs`. `request_resume_with` returns `ResumePlan { snapshot_id, plans, tasks, uncertain_tool_invocations, has_state }`. Integration test `recovery_lifecycle.rs` asserts no-tool-reinvocation per gotcha #15 invariant. 4 `ToolCallUncertaintyAction` variants (Retry/Skip/MarkComplete/Abort) emit `tool_call_uncertainty_resolved` decision signals.
- ✅ **§1d events() long-lived subscription reconnect behavior locked.** v0.1 behavior: subscriptions do NOT survive reconnect; renderers resubscribe. Integration test at `crates/runtime-main/tests/drone_reconnect_events.rs::old_events_subscription_does_not_observe_post_reconnect_drone` proves the invariant. M02 carry-forward CLOSED.
- ✅ **Production drone subprocess wiring at Tauri startup.** Code at `src-tauri/src/drone_lifecycle.rs::DroneLifecycle::spawn` + `src-tauri/src/main.rs` setup hook. `Arc<DroneClient>` is Tauri-managed-state; every command accepts `tauri::State<'_, Arc<DroneClient>>`. M03 carry-forward CLOSED.
- ✅ **`vdr.rs` projector wired at signal-write call-site.** `crates/runtime-drone/src/command_handler.rs::handle_write_signal` calls `vdr::project_signal(conn, signal_id)` after the decision insert. M03 carry-forward CLOSED.
- ✅ **xtask Rust typify list extended.** `crates/xtask/src/main.rs` schemas list now covers all 11 schemas. M03 carry-forward CLOSED.
- ✅ **Real `/v1/messages/count_tokens` endpoint.** `crates/runtime-main/src/providers/anthropic.rs::count_tokens` POSTs to `https://api.anthropic.com/v1/messages/count_tokens`. M02 carry-forward CLOSED.
- ✅ **Structured-emitter migration from line-by-line `decision_extractor`.** `crates/runtime-main/src/sdk/structured_emitter.rs` parses ONLY `<<DECISION>>...<<END>>`-delimited blocks. `decision_extractor.rs` DELETED. False-positive elimination proven at `tests/sdk_event_translation.rs::unstructured_decision_text_does_not_emit_decision_record`. M02 carry-forward CLOSED.
- ✅ **`error.v1.json` schema for `CmdError` wire format.** `schemas/error.v1.json` + generated mirrors at `crates/runtime-core/src/generated/error.rs` + `src/types/error.ts`. M02 carry-forward CLOSED.
- ✅ **Migrations-as-source-of-truth pattern.** `crates/runtime-drone/migrations/{000_initial,001_plans_tasks}.sql` + `migration_runner.rs` replace inline `init_schema`. M5+ adds migrations not edits.
- ✅ **Schema as source of truth held for 11 schemas.** `cargo xtask regenerate-types` regenerates Rust + TS from `schemas/*.v1.json`; CI drift check enforces byte-stability. M04 added 5 new schemas without drift.
- ✅ **DCO sign-off + Conventional Commits + Apache 2.0** — all 7 M04 work commits signed; titles conform.
- ✅ **Zero telemetry held end-to-end** — Stage E added `tauri-plugin-notification` (system notification only, no phone-home); no analytics deps, no crash reporter; `cargo audit` + `npm audit` pass at `--audit-level=high`.
- ✅ **Capability lockdown matches spec §10.** `src-tauri/capabilities/default.json` adds `notification:default` only (the desktop notifier requires it); no `shell:*`, `fs:*`, `http:*`, `dialog:*` added.
- ⚠️ **Budget event shapes diverge from spec §2a** — three drifts. (a) `scope` field documented in spec §2a for `BudgetWarning`/`BudgetSuspended`/`BudgetExceeded` is missing from `event.v1.json` schema. (b) `BudgetDownshift` should carry `spent_usd`/`cap_usd` per spec §2a; current schema carries `from_model`/`to_model`/`reason` only. (c) Spec §2a `budget_warning` discriminator vs the schema's actual `budget_warn`. Stage F WIRES the existing names/shapes deliberately; renderer adapts via prior-snapshot state retention pattern. Resolution: bundle into a post-M04 `docs(spec):` PR with the schema reshape as a minor version bump (`event.v1.1.json`). Documented in CHANGELOG.
- ⚠️ **Downshift hook framework-tool-dispatch deferred** — `framework.budget.downshift_hook.tool_name` is in the schema and parsed but no code path dispatches it. v0.1 uses hardcoded `DefaultLadder`. The hook trait at `crates/runtime-main/src/budget/hook.rs` exists at the seam so M5/M9 generators can plug in the framework-defined tool dispatch without changing call sites.
- ⚠️ **`plan_loop.rs` driver deferred to M07.** FSM + ApprovalSeam + structured_emitter ARE the spec §3a primitives; the driver wrapping them around `run_agent_with_provider_stream` requires framework JSON loaded into the SDK context (M07 Registry per spec §0d). Documented at `crates/runtime-main/src/sdk/mod.rs` + Stage B retrospective.
- ⚠️ **`DontTouchEvaluator` + Write-tool dispatcher integration site deferred to M05.** Stage D ships `dont_touch::DontTouchEvaluator::evaluate(path)` as a callable primitive; v0.1 has no central Write tool dispatcher in `runtime-main/src/sdk/` (M05 capability enforcer scope per spec §4a + §8). Documented inline + in CHANGELOG.
- ⚠️ **HITL event payload carries `hook_id + category + firing_point`** rather than spec §4a's `ref: HookRef` — full HookRef recoverable from framework JSON cache by `hook_id` when M07 framework loader lands. Deliberate scope-down per CLAUDE.md §12.
- ⚠️ **Pre-existing `HitlRequested`/`HitlResolved` provisional shape REPLACED at Stage E** with the spec §6a `HitlNotifyEvent`-aligned shape. Replacement was per CLAUDE.md §12 — audit verified no live producers existed pre-Stage E.
- ⚠️ **`UncertainInvocation.toolName` field carried but only `invocationId` (signal id) populated end-to-end** — populating `toolName` requires correlating the invocation signal back to its payload. Deferred. v0.1 UI shows "Tool call uncertain" without the original tool name; v1.0 may surface it.
- ⚠️ **Per-trigger HITL timeout configurability deferred to v1.0.** `HitlPolicy::timeout_seconds` is a single global default per framework JSON. Per-trigger override is spec §6a aspirational shape but not in current `HitlTriggerPolicy` field set. v0.1 frameworks accept the global setting.
- ⚠️ **Tool-call uncertainty `mark-complete` action does NOT surface an output-text input field.** Spec §1b says `mark complete — assume it completed (provide output if known)`. v0.1 UI ships 4 buttons without the text field. Forward to v1.0.
- ⚠️ **`runtime-main/src/drone_ipc/client.rs` per-module coverage regression.** M04.A1 closed M03 carry-forward (94.00% → 96.75% via `await_event_timeout_when_peer_silent` test). Stage F's new `recover_session` request/response method paths drop the file to 89.45–89.90% line (with `--skip recovery_lifecycle`). Integration test `recovery_lifecycle.rs` covers the path against a real drone subprocess on Linux CI; Windows-local llvm-cov flakes under instrumentation. Per CLAUDE.md §5 documented coverage regression with deterministic-unit-test follow-up in M05+ (`tokio::io::duplex` + `tokio::time::pause()` archetypes apply to the new recover_session paths).
- ⚠️ **Windows-local `cargo llvm-cov` flake on subprocess-spawning integration tests.** `recovery_lifecycle.rs`, `plan_recovery.rs`, `drone_reconnect_events.rs` all flake under llvm-cov instrumentation when default parallelism is used; mitigation: `-- --test-threads=1 --skip recovery_lifecycle --skip plan_recovery --skip drone_reconnect`. Linux CI not affected. Developer-experience cost only.
- ⚠️ **`signal.rs::ContextType` enum divergence from spec §2b (M02 carry-forward)** — still deferred. M04 wrote signal-emission code at WriteSignal + decision-record + uncertainty-resolved paths but those emissions don't exercise the full `ContextType` variant set. M05 capability enforcer signal-emission integration is the natural reconciliation point.
- ⚠️ **CI `e2e-tauri-driver` job stays disabled** — explicitly disabled per the M04 phase doc Key constraints to avoid CI churn during the multi-stage milestone. M03's expected resolution was deferred. Forward to a focused infrastructure session.
- ⚠️ **`HitlContext::BudgetThreshold` integration with Stage F's `BudgetWarn` event.** Stage E's policy.rs has `HitlContext::BudgetThreshold(percent)` + a precondition table; Stage F's enforcer emits `BudgetWarn`/`BudgetSuspended` events but doesn't yet drive the HITL policy evaluator (the integration site lands when `plan_loop.rs` wires the SDK → enforcer → policy → seam chain in M07).
- ❌ **None observed.** No outright contradictions where code ships behavior that spec forbids or vice versa. All deviations are forward-compat, documentation-side, or v0.1-scoped (production-wiring deferrals to M05+/M07).

### Spec review (forward-looking)

- **Missing items:**
  - `agent-runtime-spec.md` §2a — budget event `scope` field documented in spec for all 4 variants but missing from `event.v1.json` schema. Bundle into post-M04 `docs(spec):` PR with the schema reshape (minor version bump `event.v1.1.json`).
  - `agent-runtime-spec.md` §2a — `BudgetDownshift` should carry `spent_usd`/`cap_usd` per spec; current shape carries `from_model`/`to_model`/`reason`. Same bundle.
  - `agent-runtime-spec.md` §2a — `budget_warning` (spec) vs `budget_warn` (schema) discriminator rename. Same bundle.
  - `agent-runtime-spec.md` §3a — `plan_loop.rs` driver placement: SDK-side (consumed by `run_agent_with_provider_stream`) or Tauri-managed orchestrator? Stage B + C deferred; M07 framework JSON loader will wire it. Spec should lock the placement before M07 prep.
  - `agent-runtime-spec.md` §4a — Write-tool dispatcher integration site contract. Stage D ships `DontTouchEvaluator` as a callable primitive; the integration point (every Write call routes through rail evaluator BEFORE OS write) is M05 capability-enforcer scope but spec doesn't explicitly name the boundary.
  - `agent-runtime-spec.md` §6a — per-trigger HITL timeout configurability path. Spec mentions but `HitlTriggerPolicy` doesn't have the field. Forward to v1.0 reshape.
  - `agent-runtime-spec.md` §6a — `HitlContext::BudgetThreshold` → `BudgetWarn` event integration site contract. Stage E ships the policy machinery; M07 wires the chain.
  - `agent-runtime-spec.md` §1b — `mark-complete` action output-text input field per spec wording. Forward to v1.0 UI.
  - `agent-runtime-spec.md` §1b — `UncertainInvocation.toolName` correlation flow (signal-id → original invocation payload).
  - `agent-runtime-spec.md` §1b — `reconnect_mcp_servers` v0.1 no-op contract. M5/M6 wires real path.
  - `agent-runtime-spec.md` §11 — `vdr` table `contributing_signal_id` semantics. Stage E + Stage B wire it; spec should document the UNIQUE INDEX invariant.
  - `agent-runtime-spec.md` §M4 — should explicitly note the v0.1 deferrals (no plan_loop driver until M07; no Write-tool dispatcher integration until M05; downshift hook uses hardcoded ladder until M5/M9).
  - `agent-runtime-spec.md` §11 sessions table FSM diagram — Pre-M01 carry-forward; M04 implemented plan/task FSMs at code level (28 unit tests at `state_machine.rs`) but the SESSIONS FSM (per spec `sessions.status` column 6 values) still has no diagram. Forward to a docs(spec) PR.
  - `agent-runtime-spec.md` §1d / §11 — drone snapshots column rename (`event_type` vs `reason`) carry-forward from M02. Still open.
  - `agent-runtime-spec.md` §2c — `signature_delta` (Anthropic thinking-mode signature) carry-forward from M02. Still open.
  - `agent-runtime-spec.md` §2c — `ping` keep-alive carry-forward from M02. Still open.
  - `agent-runtime-spec.md` (project-wide) — model deprecation policy carry-forward from M02. Becoming relevant as Stage F's `DefaultLadder` hardcodes specific model ids.

- **Contradictions:**
  - None observed.

- **Ambiguity:**
  - `agent-runtime-spec.md` §2a — `scope` field semantics on `BudgetWarn`/`BudgetSuspended` (which scope fired: session/framework/global)? Stage F's enforcer tracks 3 scopes separately + emits with tightest-cap-wins, but the event payload doesn't carry which scope fired. Defer to v1.0 reshape.
  - `agent-runtime-spec.md` §3a — `plan_revised` event carries `tasks` (replaced) — full replacement vs delta semantics. Stage B FSM holds it as full replacement; structured_emitter emits full replacement. Lock the choice in spec.
  - `agent-runtime-spec.md` §1b — `tool_call_uncertain` flag write semantics. Stage B + F use a simple heuristic (any `tool_invoked` without matching `tool_result` within the snapshot window). v1.0 may need a smarter heuristic if multi-tool concurrent execution lands.
  - `agent-runtime-spec.md` §6a — `HitlContext::BudgetThreshold(percent)` precondition: exact threshold % vs ≥ threshold %. Stage E's policy.rs uses ≥ threshold semantics. Lock the choice in spec.
  - `agent-runtime-spec.md` §1d — drone IPC: `RecoverSession` carries `session_id` (current implementation) vs a session-state pointer. Stage F's IPC variant takes `session_id`. Lock in spec.

- **Open questions:**
  - When does the orchestrator agent's plan loop land? Stage B deferred `plan_loop.rs`; Stage C's ApprovalSeam works without it. M07 framework JSON loader is the natural placement. Need to lock placement (SDK vs Tauri-managed orchestrator) before M07 prep.
  - The `framework.budget.downshift_hook.tool_name` ladder vs hardcoded `DefaultLadder`: M5/M9 generators dispatch the framework tool; the hook trait is the seam. When does framework-tool-dispatch land?
  - `Arc<HitlSeam>` + `Arc<ApprovalSeam>` Tauri-managed-state pattern: how does it survive a session change? v0.1 single-session per §0d so no transition; v1.0 multi-session needs per-session seam storage.
  - Tauri command-surface error wire format `error.v1.json` schema — RESOLVED at M04.A1; new question: should `error.v1.json` cover non-Tauri-command errors (drone IPC errors, provider errors)? Stage A1 scoped to Tauri-command boundary only.

- **Recommended spec changes:**
  - Bundle into a post-M04 `docs(spec):` PR (target: open before M05 Stage A begins) — items above on budget event shape reshape (§2a); `plan_loop.rs` driver placement (§3a); Write-tool dispatcher integration site (§4a); per-trigger HITL timeout (§6a); `mark-complete` output-text field (§1b); `UncertainInvocation.toolName` correlation (§1b); `reconnect_mcp_servers` v0.1 no-op contract (§1b); `vdr` table `contributing_signal_id` semantics (§11); §M4 v0.1 deferrals note; sessions FSM diagram (§11); plus all M02 + M03 carry-forward items still open: `signature_delta`/`ping` (§2c); IPC reconnect surface (§1d); drone snapshots column rename (§1d/§11); model deprecation policy.
  - File a dedicated ADR for the in-process seam architecture (`ApprovalSeam` + `HitlSeam` via `Arc<T>` Tauri-managed state). The pattern is now demonstrated 2× in M04 and will recur in M05 (capability approval prompts), M06 (MCP auth prompts), M07 (registry import confirmations). Target: ADR-NNNN before M05 Stage A.

### Fix backlog

Severity is non-elastic. Critical = "must fix before M05 starts."

- 🔴 **Critical** (must fix before M05 starts): **None.** All MVP §M4 acceptance criteria met (with v0.1 deferrals documented as ⚠️ — `plan_loop` driver, Write-tool dispatcher integration, downshift framework-tool-dispatch are all M05+/M07 scope). All 7 work stages cleared hard gates G1–G5. Aggregate axis means healthy (Process 38.43/40, Product 37.43/40, Pattern 29.57/35). No shipped behavior is incorrect.

- 🟡 **Important** (should fix this release cycle):
  - **Budget event shape reshape per spec §2a** — rename `budget_warn` → `budget_warning`; add `scope` field to all 4 variants; add `spent_usd`/`cap_usd` to `BudgetDownshift`. Owner: code + spec. Minor version bump (`event.v1.1.json`). Bundle into post-M04 `docs(spec):` PR (spec side) + a `chore(schemas):` PR (code side, follows after the spec lands). M05+ wiring depends on the reshape.
  - **Downshift hook framework-tool-dispatch wiring.** Owner: code. M5/M9 generators dispatch framework-defined tool. The hook trait at `crates/runtime-main/src/budget/hook.rs` is the seam; v0.1 uses hardcoded `DefaultLadder`.
  - **`plan_loop.rs` driver — SDK-side wrapper for FSM + ApprovalSeam + structured_emitter.** Owner: code. M07 framework JSON loader. Drives the agent loop's plan-state-machine transitions + emit-and-await pattern.
  - **`DontTouchEvaluator` + Write-tool dispatcher integration site.** Owner: code. M05 capability enforcer routes every Write tool call through `DontTouchEvaluator::evaluate` BEFORE OS write.
  - **`UncertainInvocation.toolName` correlation flow.** Owner: code + spec. M05+. Correlate the recovered `invocationId` back to the original tool name + input via signals lookup.
  - **`HitlContext::BudgetThreshold` → `BudgetWarn` event chain wiring.** Owner: code. M07 wires SDK → enforcer → policy → seam.
  - **`runtime-main/src/drone_ipc/client.rs` per-module coverage close.** Owner: code. M05+. New `recover_session` paths drop coverage to 89.45–89.90% (with `--skip recovery_lifecycle`). Add `tokio::io::duplex`-driven unit tests for the request/response + filter paths; archetype is the existing `await_event_timeout_when_peer_silent` test.
  - **Windows-local `cargo llvm-cov` subprocess-test flake mitigation.** Owner: ops + docs. Workaround documented: `-- --test-threads=1 --skip recovery_lifecycle --skip plan_recovery --skip drone_reconnect`. Linux CI not affected. Forward to a focused workflow session that either pins the subprocess-spawn deadline higher under instrumentation or splits the integration tests into a CI-only matrix cell.
  - **In-process seam architecture ADR.** Owner: docs. File `docs/adr/NNNN-in-process-seam-pattern.md` documenting `ApprovalSeam` + `HitlSeam` via `Arc<T>` Tauri-managed-state. Forward-applicable to M05/M06/M07.
  - **STAGE-PROMPT-PROTOCOL.md v1.4 protocol-iteration session.** Owner: docs. Candidates: `<architecture_check>` slot (HOW-claims vs gotcha #41's WHAT-claims; carry-forward from M04.C/D/E/F); `<schema_audit>` slot (full spec-section $defs survey vs narrow specific-item audit; carry-forward from M04.D/F); `<schema_root_check>` slot (top-level `$ref` detection for json-schema-to-typescript compatibility; carry-forward from M04.E); `<phase_doc_inventory_audit>` slot (verify every file in inventory rows against `git ls-files` at authoring time; M04.A1/A2 recurrence); audit-baseline drift verification.
  - **`docs/gotchas.md` consolidation from M04 closeout.** Owner: docs. Graduate the 23 `<gotchas_graduation>`-graduated items below (gotchas.md ~43 → ~66 entries). Forward-applicable to M05+.
  - **Post-M04 `docs(spec):` PR.** Owner: spec / `agent-runtime-spec.md`. Bundle items listed in "Spec review (forward-looking) > Recommended spec changes" above. Open before M05 Stage A.
  - **Per-trigger HITL timeout configurability.** Owner: code + spec. v1.0. Single global default per framework JSON sufficient for v0.1.
  - **Mark-complete action output-text input field.** Owner: code + spec. v1.0 UI surface.
  - **`signal.rs::ContextType` reconcile with spec §2b** (M02 carry-forward, deferred from M04 closeout to M05). Owner: code OR spec. M05 capability enforcer signal-emission integration is the natural reconciliation point.
  - **MCP-schema divergence ADR** (M02 carry-forward). Owner: docs. Target M06 prep.
  - **Session FSM diagram in spec §11 sessions table** (Pre-M01 carry-forward). Owner: spec. M04 implemented plan/task FSMs at code level; sessions FSM diagram still missing at spec level. Forward to docs(spec) PR.
  - **CI `e2e-tauri-driver` job re-enablement** (M03 carry-forward, deferred at M04 per Key constraints). Owner: ops. Forward to a focused infrastructure session.

- 🟢 **Nice-to-have** (queue for v1.0+ unless trivially incidental):
  - **Per-trigger HITL timeout** (re-listed; v1.0 reshape).
  - **Mark-complete output-text** (re-listed; v1.0 UI).
  - **`BudgetDownshift` renderer state retention for downshift-event-without-prior-warn edge case** — M04.F documented ambiguity. The current pattern preserves prior `BudgetState` snapshot; a downshift without a prior warn renders `$0.00 / $0.00`. M5/M9 may surface this.
  - **`docs/gotchas.md` cross-reference index** (M03 carry-forward; M03.5 graduated 12 items so gotchas.md is at ~43 entries; M04 closeout adds 23 more → ~66). At 60+ entries an index by surface area (Rust / TS / CI / Tauri / Schema / Test) would help. Track but don't act unless friction surfaces.
  - **`keyring 3.6 → 4.0` upgrade** (M02 carry-forward) — STILL DEFERRED. M04 didn't surface new key-store friction.
  - **Tauri release-build caching** (M01+M02+M03 carry-forward) — STILL OPEN, advisory. CI time hasn't surfaced as friction in M04.
  - **Logarithmic token-scale curve for `tokenScale.ts`** (M03 carry-forward) — STILL OPEN, advisory. M04 didn't surface multi-hour-session ambiguity.
  - **`#[allow(<lint>, reason = "...")]` style-guide convention** — Stage F surfaced this pattern across the budget enforcer's cast-precision_loss + too-many-lines + too_long_first_doc_paragraph suppressions. Forward-applicable to any future cast-heavy code; could go into `docs/style.md`. Minor.

### Carry-forward from prior milestones

For every Important item from prior gap-analysis entries, status as of M04 commit (Stage G):

- **Pre-M01 baseline 🟡 "Coverage delta gating mechanism"** — RESOLVED at M02.A per M02 entry. Stable through M04.
- **Pre-M01 baseline 🟡 "Phase 3 React Flow + Zustand spec expansion"** — PARTIALLY RESOLVED at M03 implementation level; STILL OPEN at spec text level. Bundle into post-M04 `docs(spec):` PR.
- **Pre-M01 baseline 🟡 "Session FSM diagram in spec §11"** — STILL OPEN. M04 implemented plan/task FSMs at code level (28 unit tests at `state_machine.rs`); sessions FSM diagram still missing at spec level. Forward.
- **Pre-M01 baseline 🟡 "Windows named pipe spec subsection"** — RESOLVED at PR #36 per M02 entry. Stable through M04.
- **Pre-M01 baseline 🟡 "typify `oneOf` clippy suppression"** — STILL RESOLVED. M04 added 5 new generated files (`generated/{plan,task,hitl,budget,error}.rs` + extensions to `event.rs`); the existing generated-file `#[allow(...)]` header continues to suppress across all generated content. M04.A1 + M04.D + M04.E surfaced new typify edge cases (inline-enum panics; root-`$ref` json-schema-to-typescript incompatibility) resolved via schema-shape conventions (`$defs` extraction).
- **Pre-M01 baseline 🟢 "§10 numbering gap"** — RESOLVED at PR #36 per M02 entry.
- **Pre-M01 addendum 🟡 "Reuse-first vs duplication-first §9 bias"** — STILL DEFERRED to M07–M08. M04 added 5 new safety-primitive modules; some helpers extracted (build_action, build_spawn_args, compose_title_body, etc.) but no architectural abstraction extraction. Re-evaluate at M07–M08.
- **Pre-M01 addendum 🟡 "UI consistency: existing look and feel"** — RESOLVED at M03 implementation level + STAYS RESOLVED at M04. M04's 6 new renderer components (ApprovalPanel, HITLPanel/Modal/Toast, BudgetHeaderBar, RecoveryDialog, UncertaintyPrompt) all use the existing `:root` CSS custom-property palette + ARIA conventions inherited from M03.
- **M01 🟡 "`mcp_servers` table — add now or document deferral"** — RESOLVED at M02.A per M02 entry.
- **M01 🟡 "Post-M01 `docs(spec):` PR"** — RESOLVED at PR #36 per M02 entry.
- **M01 🟡 "`*_with` test-seam pattern"** — STILL RESOLVED + EXTENDED at M04. M04 adds the pattern across many more substrates: `DroneLifecycle::spawn_with` (Tauri shell subprocess spawn, A2); `respond_hitl_with` + `approve_plan_with` + `revise_plan_with` + `abort_plan_with` (Tauri command in-process seam resolution, C + E); `execute_shell_with` (hook shell-spawn, D); `request_resume_with` + `respond_uncertainty_with` + `set_global_budget_with` (Tauri command async callbacks + state mutations, F); `emit_bell_with` + `emit_sound_with` + `Desktop::with_dispatcher` (notifier dispatchers, E); `handle_write_signal_with` (drone command-handler async DB ops, B). Pattern is now demonstrated across TEN+ substrates aggregate from M01–M04.
- **M01 🟡 "Phase 3 React Flow + Zustand spec expansion"** (re-listed) — see Pre-M01 baseline above.
- **M01 🟡 "Windows drone integration test"** — RESOLVED at M02.A. M04 adds 6 more cross-platform integration tests using the same `current_exe()`-derived subprocess pattern: `drone_reconnect_events.rs` (A2), `plan_lifecycle.rs` + `plan_recovery.rs` (B), `hook_integration.rs` (D), `hitl_failure_escalation.rs` (E), `budget_threshold.rs` + `recovery_lifecycle.rs` (F).
- **M01 🟡 "`.gitattributes` line-ending normalization"** — STILL RESOLVED. M04 added no Windows line-ending warnings.
- **M01 🟢 "Tauri release-build caching"** — STILL OPEN, advisory. No M04 friction observed.
- **M02 🟡 "TS `AgentEvent` codegen from schema"** — RESOLVED at M03.A. M04.A1 extended `xtask` Rust typify list to include `event.v1.json` (closes the M03 carry-forward of the `event.rs` hand-maintenance brittleness — though Stage B + D + E + F still extend `event.v1.json` with new variants and regenerate cleanly).
- **M02 🟡 "Tauri 2.x desktop-shell E2E via `tauri-driver` + WebdriverIO"** — RESOLVED at M03.F; M04 keeps the `e2e-tauri-driver` job DISABLED per Key constraints. Still pending real CI exercise.
- **M02 🟡 "vitest `--coverage` enabled in default `test` script"** — RESOLVED at M03.A.
- **M02 🟡 "Decision extractor → structured emitter migration"** — RESOLVED at M04.B. `crates/runtime-main/src/sdk/structured_emitter.rs` (NEW, 544 lines, 95.92% line) replaces `decision_extractor.rs` (DELETED). False-positive elimination test at `tests/sdk_event_translation.rs::unstructured_decision_text_does_not_emit_decision_record` proves the contract.
- **M02 🟡 "`count_tokens` → real `/v1/messages/count_tokens` endpoint"** — RESOLVED at M04.A2. POST against `https://api.anthropic.com/v1/messages/count_tokens` with `anthropic_wiremock.rs` coverage (4 new test cases).
- **M02 🟡 "`signal.rs::ContextType` reconcile with spec §2b"** — STILL DEFERRED to M05. M04 wrote signal-emission code at WriteSignal + decision-record + uncertainty-resolved paths but those emissions don't exercise the full `ContextType` variant set (consumers use signal-as-record, not signal-as-context-tag). M05 capability enforcer signal-emission integration is the natural reconciliation point.
- **M02 🟡 "Post-M02 `docs(spec):` PR"** — STILL OPEN. Bundled into post-M03 `docs(spec):` PR (still open per M03 entry), now becoming post-M04 `docs(spec):` PR with M04 additions bundled.
- **M02 🟡 "MCP-schema divergence ADR"** — STILL OPEN, target M06 prep. M04 didn't touch MCP code.
- **M02 🟡 "CLAUDE.md §15 / `docs/gotchas.md` consolidation"** — RESOLVED at M02→M03 protocol-iteration session per M03 entry. `docs/gotchas.md` is now at ~43 entries; M04 closeout adds 23 graduation candidates (below) → ~66 entries at post-M04 protocol-iteration session.
- **M02 🟡 "`TEMPLATE.md` updates"** — RESOLVED at M02→M03 protocol-iteration session per M03 entry; v1.2 protocol locked. M04 inherited and applied throughout.
- **M02 🟡 "Plan model field shapes"** — RESOLVED at M04.B. `schemas/plan.v1.json` + `schemas/task.v1.json` + generated mirrors; FSM at `state_machine.rs` (99.28%; 28 unit tests).
- **M02 🟡 "Retrofit `crates/runtime-drone/tests/integration*.rs` to `current_exe()`-derived paths"** — RESOLVED at M03.A per M03 entry. M04 follows the same pattern for 6 new integration tests.
- **M02 🟡 "Long-lived `events()` subscription survives reconnect"** — RESOLVED at M04.A2. v0.1 behavior locked: subscriptions do NOT survive reconnect; renderer resubscribes. Integration test at `crates/runtime-main/tests/drone_reconnect_events.rs::old_events_subscription_does_not_observe_post_reconnect_drone` proves the invariant.
- **M02 🟡 "`error.v1.json` schema for `CmdError` wire format"** — RESOLVED at M04.A1. `schemas/error.v1.json` (new) + generated to `crates/runtime-core/src/generated/error.rs` (typify) + `src/types/error.ts` (json-schema-to-typescript). `cmd_error_ext.rs` (NEW, 233 lines, 100% coverage) supplies inherent helper constructors + `Display` + `std::error::Error` impls.
- **M02 🟢 "Vite 5.4 → 7+ bump"** — RESOLVED at M03.A.
- **M02 🟢 "Delete legacy `src/counter.{js,test.js}`"** — RESOLVED at M03.A.
- **M02 🟢 "`secrecy/serde` workspace feature"** — RESOLVED at M03.A.
- **M02 🟢 "`keyring 3.6 → 4.0` upgrade"** — STILL DEFERRED. M04 didn't surface new key-store friction.
- **M02 🟢 "Tauri release-build caching"** (re-listed) — STILL OPEN, advisory.
- **M03 🟡 "Production drone subprocess wiring at Tauri startup"** — RESOLVED at M04.A2. `src-tauri/src/drone_lifecycle.rs` spawns the drone via `tokio::process::Command` + manages `Arc<DroneClient>` via Tauri's setup hook. Every command takes `tauri::State<'_, Arc<DroneClient>>`.
- **M03 🟡 "`vdr.rs` projector wired at signal-write call-site"** — RESOLVED at M04.B. `crates/runtime-drone/src/command_handler.rs::handle_write_signal` calls `vdr::project_signal` after decision insert.
- **M03 🟡 "Extend xtask Rust typify list to include `event.v1.json`"** — RESOLVED at M04.A1. `crates/xtask/src/main.rs` schemas list extended with `event` + new `error`. M04.B/D/E/F all regenerated cleanly on schema edits.
- **M03 🟡 "Add `tokio::time::pause()`-driven coverage for `await_event` timeout path"** — RESOLVED at M04.A1 (94.00% → 96.75%). However Stage F's new `recover_session` IPC paths regress the file to 89.45–89.90% (with `--skip recovery_lifecycle`); re-listed as M04 🟡 above.
- **M03 🟡 "Post-M03 `docs(spec):` PR"** — STILL OPEN; folded into post-M04 `docs(spec):` PR.
- **M03 🟡 "`docs/gotchas.md` consolidation from M03 closeout"** — RESOLVED at M03→M04 protocol-iteration session (M03.5). 12 graduated items landed in `docs/gotchas.md`; M04 stages cite the resulting numbers (gotcha #14 snake_case-schema discipline, #15 recovery rebuilds history, #18 JSONLogic operator allowlist, #21 clippy lint clusters, #29 schema-root-must-be-concrete-type, #32 Tauri cross-stack, #33 schema-regen-before-sample-bodies, #34 cargo fmt --all first, #41 grep-verify-claims).
- **M03 🟡 "STAGE-PROMPT-PROTOCOL.md v1.3 protocol-iteration session"** — RESOLVED at M03.5. v1.3 protocol locked: `<pre_flight_check>` closeout slot; `<schema_drift_check>`; `<fan_out_grep>`; `<dependency_audit_check>`; `<runtime_environment>`. M04 stages applied the v1.3 tags throughout.
- **M03 🟡 "CI `e2e-tauri-driver` job verification on first PR push"** — STAYS DEFERRED at M04 per Key constraints. Carry-forward to M05+.
- **M03 🟡 "`signal.rs::ContextType` reconcile with spec §2b"** (re-listed) — STILL DEFERRED to M05; see M02 carry-forward above.
- **M03 🟡 "Decision extractor → structured emitter migration"** — RESOLVED at M04.B (see M02 carry-forward above).
- **M03 🟡 "`count_tokens` → real endpoint"** — RESOLVED at M04.A2 (see M02 carry-forward above).
- **M03 🟡 "Phase 3 React Flow + Zustand spec expansion"** (re-listed) — see Pre-M01 baseline above.
- **M03 🟡 "Session FSM diagram"** (re-listed) — STILL OPEN at spec level.
- **M03 🟡 "UI consistency"** (re-listed) — RESOLVED + STAYS RESOLVED (see Pre-M01 addendum above).
- **M03 🟡 "Plan model field shapes"** (re-listed) — RESOLVED at M04.B.
- **M03 🟡 "Long-lived `events()` subscription survives reconnect"** (re-listed) — RESOLVED at M04.A2.
- **M03 🟡 "`error.v1.json` schema for `CmdError` wire format"** (re-listed) — RESOLVED at M04.A1.
- **M03 🟢 "Logarithmic token-scale curve"** — STILL OPEN, advisory.
- **M03 🟢 "Tauri release-build caching"** (re-listed) — STILL OPEN, advisory.
- **M03 🟢 "`docs/gotchas.md` cross-reference index"** — STILL TRACKED. gotchas.md at ~43 → ~66 after M04 closeout. Defer until 60+ entries (now triggered; consider in v1.4 protocol-iteration session).
- **M03 🟢 "§10 numbering gap"** — RESOLVED at PR #36 per M02 entry.
- **M03 🟢 "`vitest --coverage` enabled"** — RESOLVED at M03.A.

### Gotchas graduation (v1.2 protocol)

Per `STAGE-PROMPT-PROTOCOL.md` v1.2 + the M04.G closeout prompt's `<gap_analysis_requirements><gotchas_graduation>` subsection. Every per-stage `<gotchas>` trap from M04.A1–F (plus closeout-friction-events from each retrospective) gets a disposition: **kept** | **graduated** | **resolved** | **expired**.

| Stage | Gotcha / friction event | Disposition | Target/Resolution |
|---|---|---|---|
| A1 | typify 0.6.2 panics on root-`oneOf` with validated inline strings — extract to `$defs` (Sev 3) | graduated | New `docs/gotchas.md` entry ("Validated string subschemas in event-variant or top-level `oneOf` schemas must live in `$defs` with a `title` — typify 0.6.2 panics at `type_entry.rs:511` when no title is available. Inline `{ \"type\": \"string\", \"minLength\": 1 }` properties trip the variant generator's name derivation. Recurred M04.A1 (error.v1.json `message`) + M04.B (ApprovedBy enum) + M04.D (HookCategoryRef + OnFailureRef + RailPolicy)."). |
| A1 | `cargo fmt --all` as first `verify_gates` step (Sev 1) | kept | Already graduated at M03 closeout (gotcha #34); stays in per-stage `<gotchas>` of M05+ Rust-heavy stages. M04 confirmed the pattern across all 7 stages. |
| A1 | New generated TS file goes in BOTH `.prettierignore` AND `eslint.config.js` `ignores` per the `agent_event.ts` precedent | graduated | New `docs/gotchas.md` entry ("Any new file at `src/types/*.ts` produced by `cargo xtask regenerate-types` must be added to BOTH `.prettierignore` AND `eslint.config.js`'s `ignores` list — match the `agent_event.ts` + `error.ts` + `plan.ts` + `task.ts` + `hitl.ts` + `budget.ts` precedent. Easy to miss without reading the existing ignore lists."). |
| A1 | `tokio::time::pause()` + `start_paused = true` deterministic timeout coverage | kept | Stays in per-stage `<gotchas>` for M05+ stages adding timeout-bound async paths. M04.A1 closed the M03 carry-forward (94.00% → 96.75% on `client.rs::await_event`); same archetype applies to any future request/response IPC method. |
| A1 | M04 Phase doc inventory drift (`error.rs` claimed DOES NOT EXIST when it does) | resolved | M04.A1 commit `f5dcbd5` (landed typify output at `generated/error.rs` matching convention; resolved lib.rs glob via explicit `pub use generated::{agent, common, framework, skill, tool};`). Lesson generalized to `<phase_doc_inventory_audit>` v1.4 protocol candidate. |
| A2 | Phase doc inventory drift on integration points (WriteSignal IPC, prompt_template, event_translation, src-tauri/lib.rs — all claimed but don't exist; Sev 4) | resolved | M04.A2 commit `2bf4d67` via user-approved scope reduction (3 of 3 deferred items deferred to Stage B or M07; the remaining 6 tractable items shipped). Lesson generalized: phase doc inventory rows MUST be verified against `git ls-files` at authoring time. v1.4 `<phase_doc_inventory_audit>` candidate. |
| A2 | CmdError shape migration (tuple-variant over `ErrorMessage` newtype vs M02 struct-variant; ~17 callsites; Sev 3) | graduated | New `docs/gotchas.md` entry ("Wire-equal-but-Rust-shape-different schema migration: when a typify-generated type replaces a hand-rolled type, the wire format stays the same but Rust callsite shape differs. Add inherent helper constructors + `Display` + `std::error::Error` impls in a separate module within the same crate (orphan rule satisfied). Place `From<ForeignType>` impls in the crate that owns the source type. Forward-applicable to M05+/M06 schema reshapes."). |
| A2 | clippy 12-lint batch (recurring gotcha #21 patterns; Sev 2) | kept | Recurring pattern; stays as closeout-stage observation. Pre-empt via `cargo clippy --fix --allow-dirty` first-pass per M04.D/E/F decisions. |
| A2 | `futures::stream::unfold` panics on poll-after-Ready(None) (Sev 2) | graduated | New `docs/gotchas.md` entry ("Streams built on `futures::stream::unfold` panic on poll-after-Ready(None). When draining via `.next()` until None, track termination state explicitly — do not poll again. Forward-applicable to any test draining drone-IPC `events()` stream or other unfold-based streams."). |
| A2 | Windows file-lock persistence after subprocess SIGKILL even with `kill_on_drop(true)` (Sev 2) | graduated | New `docs/gotchas.md` entry ("Windows file lock on subprocess binaries can persist for a few seconds after process exit even with `tokio::process::Command::kill_on_drop(true)`. If `cargo test` reports `Access is denied (os error 5)` on a `runtime-*` binary, run `Stop-Process -Name <bin> -Force` and retry. Forward-applicable to any test that spawns runtime-drone or future runtime-* subprocesses."). |
| A2 | `cargo fmt --all` then `cargo clippy --workspace --all-targets -- -D warnings` as standard sequence (recurring gotcha #34) | kept | Same as A1 entry; stays as closeout observation. |
| B | clippy `too_long_first_doc_paragraph` + 12+ lints (Sev 2) | graduated | New `docs/gotchas.md` entry ("Module-level `//!` doc first paragraph: keep to one short sentence (≤80 chars) followed by section headings or blank line before continuation. Clippy `too_long_first_doc_paragraph` lint fires on first paragraphs spanning 4+ lines, recurring across M04.B/D/E/F. The lint reports without source spans; iterative shorten OR `#![allow(clippy::too_long_first_doc_paragraph, reason = \"...\")]` at module level."). |
| B | Cross-package coverage attribution gap (safety primitives exercised primarily via cross-package integration tests; Sev 3) | graduated | New `docs/gotchas.md` entry ("Per-package coverage gates (`cargo llvm-cov --package <crate> --fail-under-lines 95`) count only tests living in the same package. Safety primitives exercised via cross-package integration tests (e.g., `runtime-main` tests calling `runtime-drone` via subprocess IPC) need package-internal unit tests too. Pattern: write the integration test for the wire path; write package-internal unit tests for the same logic to satisfy the per-package coverage gate. Forward-applicable to M05 (capability enforcer), M06 (MCP client)."). |
| B | Snapshot timestamp granularity in tests (Sev 1) | graduated | New `docs/gotchas.md` entry ("Production `snapshot::write` uses second-granularity timestamps. Tests that assert latest-by-timestamp ordering between rapid back-to-back writes must seed explicit timestamps via raw INSERT (or wait 1+ second between writes). Forward-applicable to any test of latest-row ordering with timestamp keys."). |
| B | M02 `decision_extractor` heuristic test rewrite for structured emitter (Sev 1) | resolved | M04.B commit `962525e` rewrote `tests/sdk_event_translation.rs` to assert the new structured-emitter contract. Test rename: `decision_pattern_in_text_emits_decision_record` → `unstructured_decision_text_does_not_emit_decision_record`. False-positive elimination proven. |
| B | `plan_loop.rs` deferral (ambiguity event; per §12 own-technical-decisions) | kept | Stays in fix backlog as M07-blocking 🟡. The FSM + ApprovalSeam + structured_emitter ARE the spec §3a primitives; the driver wrapper has no callsite without framework JSON loader. Same HOW-claim category as M04.C/D/E surfaced — `<architecture_check>` v1.4 slot proposal. |
| C | `'human'` not assignable to `ApprovedBy` (schema-derived enum drift; Sev 1) | graduated | New `docs/gotchas.md` entry ("Schema-derived enums (e.g., `ApprovedBy = 'user' | 'auto'`) are generated from `schemas/*.v1.json` $defs. Phase doc sample bodies or test fixtures using enum-member-literals must match the actual schema values. Pre-flight grep the schema $defs before authoring sample bodies. Recurred M04.C ApprovedBy + M04.F audit-baseline event names."). |
| C | clippy `doc_markdown` lint on `plan_loop` unbackticked (Sev 1) | kept | Already in docs/gotchas.md from M03; recurring. Stays as closeout observation. |
| C | GraphCanvas `useMemo([nodes.length])` count-keyed memo bug — fix to positions-by-id Map + per-render data merge (Sev 3) | graduated | New `docs/gotchas.md` entry ("GraphCanvas-style layout memos must cache POSITIONS by ID (`Map<string, {x, y}>`), not entire node-object arrays. M03.D's count-keyed `useMemo([nodes.length, edges.length])` silently dropped data updates that didn't change array length — M04.C plan FSM transitions surfaced the latent bug. Fix: merge fresh nodes from the store with positions on every render. Forward-applicable to any future stage adding visually-driven node-data updates."). |
| C | Vite dep-optimizer cold-start re-optimizes on first request to NEW code paths within a spec file (Sev 3 timing) | graduated | New `docs/gotchas.md` entry ("When adding a new component reachable from a NEW spec file (no other spec exercises its imports), the spec's first `page.goto` pays a Vite dep-optimization pass on top of the smoke baseline. Bump the describe-block timeout to match `webServer.timeout` (90s) via `test.describe.configure({ timeout: 90_000 })` until the dep is added to `vite.config.ts` `optimizeDeps.include` if hot. Surfaced M04.C `plan_approval.spec.ts`."). |
| C | Drone-resolves-seam architecture mismatch (Sev 2; HOW-claim category) | resolved (carry-forward to v1.4 protocol) | M04.C commit `1138486` via §12 own-technical-decisions — 3 commands resolve in-process `Arc<ApprovalSeam>` directly. Lesson generalized: HOW-claims about cross-process flow need a v1.4 `<architecture_check>` slot (carry-forward; recurred M04.D + E + F). |
| C | PlanNode cumulative token spend deferral (Sev 2; phase doc internal contradiction) | resolved | M04.C commit `1138486` skipped the field per the prompt's `<gotchas>` trap forbidding state-field additions without Stage B amendment. Documented as deferral in CHANGELOG + retro. |
| C | `window.__graphStore` Playwright affordance pattern for state injection | graduated | New `docs/gotchas.md` entry ("Renderer-level Playwright with state injection: module mocking across the `@tauri-apps/api` ESM boundary doesn't work in Playwright (only in Vitest's `vi.mock`). Tests that drive renderer state without a live Tauri runtime should expose the Zustand store on `window.__graphStore` (App.tsx affordance) and use `page.evaluate(() => window.__graphStore.getState().applyEvent(...))` to inject state. Click→invoke linkage stays in Vitest; state-driven surface tests stay in Playwright. Pairs with the `e2e-tauri-driver` disabled-status carry-forward."). |
| D | Hook + HookRef + Rail prior-art audit gap (Sev 3 major scope discovery; types already authored) | resolved (carry-forward to v1.4 protocol) | M04.D commit `0884ec1` via §12 own-technical-decisions — dropped the new `schemas/hook.v1.json` plan; consumed existing `common.v1.json` + `framework.v1.json` types. Lesson generalized: `<schema_audit>` v1.4 slot proposal (full spec-section $defs survey at authoring time vs narrow specific-item audit). |
| D | Write-tool dispatcher integration-site doesn't exist in runtime-main (Sev 2; HOW-claim) | resolved (carry-forward to v1.4 protocol) | M04.D commit `0884ec1` via §12 own-technical-decisions — shipped `DontTouchEvaluator` as callable primitive; integration site is M05 capability enforcer territory. Same HOW-claim pattern; `<architecture_check>` slot proposal recurrence. |
| D | typify panic on inline enum properties in event variants (recurring from A1 + B) | kept | Stays in per-stage `<gotchas>` of M05+ stages adding schema event variants. Same graduated entry as A1; recurrence reinforces. |
| D | rustdoc intra-doc link scope (submodule refs in `//!` docs resolve at module-end; Sev 1) | graduated | New `docs/gotchas.md` entry ("Rustdoc resolves intra-doc links (e.g., `` [`HitlPolicy`] ``, `` [`Desktop::with_dispatcher`] ``) at the END of the module's declaration scope, but the lint that fires on unresolved links is anchored at the doc-location (BEFORE the submodule / cross-module item is declared). Use plain backticks for submodule + cross-module + cross-crate refs in module-level `//!` docs to avoid `cargo doc -D missing_docs` errors. Recurred M04.D `hooks/mod.rs` + M04.E `hitl/*` + M04.F `budget/mod.rs` + `recovery/mod.rs`."). |
| D | Windows-local `cargo llvm-cov` flake on subprocess-spawning tests under default parallelism (Sev 3) | graduated | New `docs/gotchas.md` entry ("Windows-local `cargo llvm-cov` may flake on subprocess-spawning integration tests under default parallelism: `connect: io: The system cannot find the file specified` because the instrumented child takes longer to bind the named pipe than the test's retry deadline. Mitigations: `cargo llvm-cov ... -- --test-threads=1` (serializes) + `--skip plan_recovery --skip drone_reconnect --skip recovery_lifecycle` (filters out the worst offenders). CI Linux is faster and not affected. Pairs with gotcha #22 subprocess test path derivation."). |
| D | HookRef-in-event simplification (Sev 2; per §12 deliberate scope-down from spec §4a's `ref: HookRef`) | resolved (carry-forward to v1.0 spec reshape) | M04.D commit `0884ec1` — event payload carries `hook_id + category + firing_point`; full HookRef recoverable from framework JSON cache by `hook_id` when M07 lands. v1.0 may add the full HookRef payload back. |
| D | fmt-first bypass (gotcha #34 noted but didn't catch structural lints) | kept (corollary added) | Corollary to gotcha #34: "fmt is one of two mechanical first-passes; the other is `cargo clippy --fix --allow-dirty` which auto-resolves many gotcha-#21 patterns. Run BOTH as first-pass." Added to graduation set above. |
| E | Top-level `$ref` unsupported by json-schema-to-typescript (Sev 2) | graduated | New `docs/gotchas.md` entry ("Top-level `$ref` in JSON Schema breaks json-schema-to-typescript: the tool errors at `parseNonLiteral` and requires a concrete `type: object` (or similar) at the schema root. typify (Rust side) supports both forms; TS codegen requires concrete-type-at-root. Inline the root definition's fields at the schema root and keep `$defs` for shared sub-types. Recurred M04.E `hitl.v1.json` initial draft; M04.F `budget.v1.json` follows the corrected pattern from the start."). |
| E | clippy 15-lint batch (recurring; Sev 2) | kept | Same as A2 entry; stays as closeout observation. |
| E | rustdoc intra-doc link to submodule items (recurring from D; Sev 2) | graduated (same as D entry) | See D entry. M04.E recurrence reinforces; M04.F third recurrence confirms forward-applicability. |
| E | Windows-local llvm-cov flake (recurring from D; Sev 2) | graduated (same as D entry) | See D entry. |
| E | vitest fake-timers + `@testing-library/user-event` compose-issue (Sev 2) | graduated | New `docs/gotchas.md` entry ("vitest fake timers + `@testing-library/user-event` don't compose cleanly. When `vi.useFakeTimers()` is active, `userEvent.setup({ advanceTimers: vi.advanceTimersByTime }).click(...)` may time out at the test's default 5s budget — the user-event library's internal `wait` calls don't reliably progress fake timers. Use `fireEvent.click(...)` + `await waitFor(...)` directly for click-driven assertions when fake timers are needed elsewhere in the same test file; isolate fake timers to timer-specific tests. Recurred M04.E HITLToast."). |
| E | `respond_hitl` architectural mismatch (drone IPC vs in-process seam; Sev 2; HOW-claim) | resolved (carry-forward to v1.4 protocol) | M04.E commit `2996cff` via §12 own-technical-decisions — mirrored Stage C pattern: `respond_hitl(seam: tauri::State<'_, Arc<HitlSeam>>)`. Lesson generalized: `<architecture_check>` v1.4 slot proposal recurrence. |
| E | HitlRequested/HitlResolved shape replace vs extend (Sev 2; per §12; audit-verified no live producers) | resolved | M04.E commit `2996cff` replaced shape since no live producers existed pre-Stage E. Documented in CHANGELOG. |
| E | Internal-helper `_testing` named-export pattern for React component modules | graduated | New `docs/gotchas.md` entry ("Internal helper fns inside React component modules: re-export via `_testing` named export. Component-internal pure-helper functions often need direct unit-test coverage but should not pollute the public API. Pattern: `export const _testing = { helperFn };` at the component module's end; tests import the named export. The `_`-prefix marks the export as internal-test-only and is recognized by the eslint `no-unused-vars` allowed-unused-prefix rule. Proven across M04.E HITLPanel/Modal/Toast + M04.F BudgetHeaderBar/RecoveryDialog/UncertaintyPrompt."). |
| E | Tauri 2.x notification plugin install discipline (gotcha #32 verbatim-quote held; install + permission + capability flow) | kept | Stays in per-stage `<gotchas>` for M05+ stages installing Tauri 2.x plugins. M04.E proved the discipline works when upstream is well-maintained — gotcha #32 corollary: "third-party plugin churn is mostly upstream-quality-dependent; pin the WEBCHECK to authoritative docs at a specific date." |
| F | Schema discriminator name drift (`budget_warn` vs spec's `budget_warning`; Sev 2) | resolved (carry-forward to v1.0 spec reshape) | M04.F commit `47e86bc` per §12 own-technical-decisions — WIRES existing names. Reshape carry-forward to post-M04 `docs(spec):` PR + event.v1.1.json minor bump. Lesson: `<schema_audit>` v1.4 slot would catch this at authoring time. |
| F | Budget event shape spec divergence (`scope`/`spent_usd`/`cap_usd` missing; Sev 2 ambiguity) | resolved (carry-forward to v1.0 spec reshape) | Same as above. |
| F | Downshift renderer state retention pattern (preserve prior snapshot for events that don't carry full state; Sev 2 ambiguity) | resolved | M04.F commit `47e86bc` — renderer preserves last-known `BudgetState` snapshot when `BudgetDownshift` arrives; tested at `BudgetHeaderBar.test.tsx::flips_to_downshift_status_after_budget_downshift_event`. Pattern documented inline. |
| F | clippy 15-lint batch (recurring; Sev 2) | kept | Same as A2 + E entry; stays as closeout observation. |
| F | rustdoc intra-doc link recurrence (third stage M04.D + E + F; Sev 2) | graduated (same as D entry) | Confirmed forward-applicability across three stages. |
| F | Windows-local llvm-cov flake recurrence (M04.D + E + F; Sev 2; adds `--skip recovery_lifecycle` filter) | graduated (same as D entry) | M04.F adds `--skip recovery_lifecycle` to the workaround filter list. |
| F | `fireEvent.click` vs `fireEvent.submit` for `type="submit"` button in vitest (Sev 2) | graduated | New `docs/gotchas.md` entry ("jsdom does not propagate `click` on a `type='submit'` button to the form's `onSubmit` handler. Use `fireEvent.submit(form)` to drive the form submit cycle in vitest. `userEvent.click` DOES propagate (it dispatches the full click→submit chain) but is slower in test runs. Recurred M04.F BudgetHeaderBar settings tests."). |
| F | `cargo test --skip` is substring match; integration tests with shared prefix filter both (Sev 2) | graduated | New `docs/gotchas.md` entry ("`cargo test --skip <prefix>` is a substring match, not a file-name filter. When integration tests in `tests/<file>.rs` share a test-function name PREFIX with lib unit tests in `#[cfg(test)] mod tests`, `--skip <prefix>` filters BOTH sets — coverage drops on lib unit tests too. Rename integration tests to have a distinct prefix (e.g., `recovery_lifecycle_*` instead of `recover_session_*` when lib tests are `recover_session_*`). Recurred M04.F."). |
| F | clippy `too_long_first_doc_paragraph` reports without source spans (recurring; Sev 2) | graduated (same as B entry) | Same as B entry; F adds the `cargo clippy --message-format=json | grep file_name` debug pattern. |
| F | `#[allow(<lint>, reason = "...")]` with verbose rationale | graduated | New `docs/gotchas.md` entry ("When suppressing a clippy lint via `#[allow]`, always include the `reason` parameter with a 1-sentence justification. Examples: `#[allow(clippy::cast_precision_loss, reason = \"u64 token counts up to 2^53 are exact in f64\")]`, `#[allow(clippy::too_many_lines, reason = \"integration test with 5 phases\")]`. Forward-applicable to any cast-heavy or doc-format code; pairs with the `too_long_first_doc_paragraph` `#![allow]` pattern."). |
| F | React form-submit handler stale-closure capture (Sev 2; passing currentInput explicitly) | graduated | New `docs/gotchas.md` entry ("React form-submit handler stale-closure capture: when a form's `onSubmit` reads from `useState` state inside an `async` function, the closure captures state at render time. If the user changes input then submits in the same render, the handler may see stale state. Defensive pattern: pass the current input value explicitly into the handler from the form's submit event (e.g., `onSubmit={(e) => { e.preventDefault(); void saveGlobal(globalCap); }}`) rather than relying on the closure. Recurred M04.F BudgetHeaderBar settings."). |

**Summary:** 42 stage-gotcha + stage-friction-event entries across A1–F → 42 total dispositions:
- **resolved:** 12 (stage-local fixes that committed and have no forward applicability — phase-doc drift corrections, §12 own-technical-decisions resolutions, event-shape replacements, single-stage scope-downs).
- **graduated:** 23 (recurring or forward-applicable patterns now in `docs/gotchas.md`).
- **kept:** 7 (still apply forward; stay in per-stage `<gotchas>` of M05+ stages that touch the same surface).
- **expired:** 0.

The 23 graduations populate `docs/gotchas.md` from the M03.5 ~43 entries → ~66 entries at the post-M04 protocol-iteration session. M04 closeout's `<gotchas_graduation>` is the second to ship this section (M03 was first); the v1.2 protocol enforcement held cleanly.

Carry-forwards to the v1.4 STAGE-PROMPT-PROTOCOL.md protocol-iteration session (post-M04):
- `<architecture_check>` slot — HOW-claims about cross-process flow, IPC topology, integration-site presence (M04.C/D/E/F recurrence).
- `<schema_audit>` slot — full spec-section $defs survey at authoring time vs narrow specific-item audit (M04.D/F recurrence).
- `<schema_root_check>` slot — top-level `$ref` detection for json-schema-to-typescript compatibility (M04.E surface).
- `<phase_doc_inventory_audit>` slot — verify every file in inventory rows against `git ls-files` at authoring time (M04.A1/A2 recurrence).
- Audit-baseline drift verification (recurring across stages).

### Sign-off

**Claude:** I have generated this gap analysis after the final implementation stage of M04 (Stage G — this commit). This is my honest assessment of the cumulative code-vs-spec state across M01 + M02 + M03 + M03.5 + M04. Hard gates G1–G5 cleared in all seven M04 work stages; no Critical-severity findings. Aggregate axis means healthy (Process 38.43/40, Product 37.43/40, Pattern 29.57/35); time-box calibration drift continues at ~0.55× (M01 0.3× / M02 0.7× / M03 0.32× / M04 0.55×). The largest 🟡 backlog clusters are the post-M04 `docs(spec):` PR (budget event shape reshape + spec FSM diagrams + plan_loop placement + Write-tool dispatcher contract + HITL timeout + 11 carry-forward items), the in-process seam architecture ADR (forward-applicable to M05/M06/M07), the STAGE-PROMPT-PROTOCOL.md v1.4 protocol-iteration session (4 new slot proposals), `client.rs` per-module coverage close (regressed by Stage F's new `recover_session` paths), and the `docs/gotchas.md` consolidation (23 graduations from M04 closeout's `<gotchas_graduation>` → ~66 total entries). The second-of-its-kind `<gotchas_graduation>` subsection (v1.2 protocol enforcement) covers 42 per-stage gotchas + friction events across A1–F with disposition. M02 + M03 carry-forward final disposition recorded; M03.5 carry-forwards (v1.3 protocol tags + 12 docs/gotchas.md graduations) all applied. User review pending; per `CLAUDE.md` §20 this entry is **immutable** once committed — future milestones report status updates via their Carry-forward sections.

**Surfaced at:** 2026-05-11 (UTC).
