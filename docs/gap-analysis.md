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
