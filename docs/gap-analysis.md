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
