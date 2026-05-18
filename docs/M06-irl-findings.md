# M06 IRL Findings — M07 Stage A gating input

> **What this is.** Results of executing `docs/M06-irl-test-plan.md` against the running app after the M06 (MCP Basic) merge. Companion to that plan; mirrors `docs/M04-irl-test-plan.md` → M05.A. Severity per the plan: 🔴 fix before M07 work · 🟡 M07 Stage A absorbs · 🟢 `docs/tech-debt.md` / doc-fix. Severity is non-elastic.
>
> **Environment.** Windows 11; app built from `main` @ M06 merge (`081044a`); Node v24.11.0; npx 11.7.0; Anthropic key in OS keychain; reference server `@modelcontextprotocol/server-filesystem`.

---

## Verdict

**2 🔴 (block M07 Stage A start) · 4 🟡 (M07.A absorbs) · 5 🟢.**

M06 milestone code is merged and its unit + Stage-V suites are green — but IRL surfaced **two blocking persistence defects that automated testing structurally could not see** because Stage-V verified components in isolation (drone, registry, transport each tested standalone) while the *assembled running app* fails. This is exactly the IRL value proposition (the gotcha #66 "tests-pass-but-contract-fails" class, M04-IRL precedent). The MCP transport/connect path itself is solid; the failures are in persistence-path resolution and signal-write wiring.

A focused fix cycle is required before M07.A. The two 🔴s plausibly share one root theme: persistence-path resolution + signal-write wiring in the assembled app.

---

## 🔴 Blocking (fix + re-test before M07 Stage A)

### 🔴-1 — MCP registry write resolves to a stray DB; added server invisible/unusable

- **Repro.** App → Add MCP Server (`fs-test`, stdio, `npx.cmd`, filesystem args) → Test lists 14 real tools (transport works) → Add → audit shows `mcp_installed` → **MCP Servers panel still says "No MCP servers installed."**
- **Ground truth.** Three DB files in `%LOCALAPPDATA%\dev.aria-runtime.app\`: the row landed in `mcp.sqlite` (4 KB, stale 12:49); the **live drone DB is `session.sqlite`** (3 MB, actively written, `mcp_servers` empty there); plus a 0-byte stray `mcp_servers.sqlite`. Three divergent path resolutions; the system reads `session.sqlite` → server invisible and (downstream) unusable.
- **Why 🔴 for M07.** M07's deliverable is the agent-with-tools loop that *dispatches* MCP tools; it reads the registry to resolve servers/tools. It would read the empty live DB → MCP dispatch non-functional. This is the registry-path-divergence risk flagged at M06.D pre-flight, now realized in the running app.
- **Fix direction.** Single registry source-of-truth = the live drone session DB; read path == write path; UI refresh after add; regression test asserting add → list round-trips through the *same* store in the assembled app (not isolated `Registry`).

### 🔴-2 — Agent signal stream not persisted to the live drone DB

- **Repro.** Run smoke test (multiple times) → renderer shows the agent node, status `complete`, `tokensTotal 34` (the main→renderer Tauri path works). Query the live drone DB.
- **Ground truth.** `session.sqlite`: `signals = 0`, `token_usage = 0` — while `heartbeats = 15155` and `snapshots = 14` in the *same* DB. The drone is alive and persisting (heartbeats/snapshots) to the correct file, but the **agent signal stream lands nothing**. Signals exist in none of the three DBs.
- **Why 🔴 for M07.** The drone is the system-of-record (ADR-0007). With no `signals`, recovery/replay (M01/M04), audit, and every Stage-V "signals landed" wire-trace are **hollow in the running app**. M07's loop + recovery build on this foundation. Every Stage-V pass exercised the drone in isolation, not the assembled smoke→drone path — the defect was invisible to automated gates.
- **Fix direction.** Root-cause the main→drone signal-persist path in the *assembled* app (WriteSignal IPC landing, smoke→drone emission, or a path issue parallel to 🔴-1). Regression test must assert against the running app's drone DB after a real run, not an isolated drone harness.

---

## 🟡 M07 Stage A absorbs

### 🟡-1 — HITL `ui_variant` not honored (broad consumer set incl. the security gate)
`panel` / `modal` / `toast` all render as the same left-justified inline block at the bottom of the page. `modal` (`on_risky_tool`, and the ADR-0007 `on_capability_violation` security gate) has no overlay / centering / `aria-modal` / focus-trap — Tab escapes. `toast` self-dismissed with `timeout_at_unix_ms` set to ~year 2255 and no user action (a prompt can vanish while the seam still awaits). `UncertaintyPrompt` also renders inline though its spec expects `role=dialog` + `aria-modal`. Shared renderer root; `#66/#67` class. Strong 🟡 — the security-gate modal and a lost-prompt path are affected; matters the moment M07's loop fires these live.

### 🟡-2 — `npx` "program not found" on Windows
App spawns the stdio command via raw process-create, which does not resolve Windows `.cmd`/`.bat` shims; `npx` → "program not found"; `npx.cmd` works. Every MCP doc says `npx -y @modelcontextprotocol/...`. Linux-CI blind spot. Fix: resolve `.cmd` shims on Windows (or document `npx.cmd`).

### 🟡-3 — Stale error banner persists after a successful Test
The `mcp_test_connection` error from a failed attempt is not cleared when a subsequent Test succeeds — the banner says "connect failed: program not found" while the tool list renders successfully below it. Actively misleads (user concludes failure on a success).

### 🟡-4 — Budget settings panel not wired to current state
Clicking the budget bar opens a settings panel that accepts a saved amount, but it is not bound to the live budget state (doesn't reflect the active cap/spent; save not wired). `#68` wiring class. Matters when budget goes live in M07.

---

## 🟢 tech-debt / doc-fix

- 🟢-1 Window title reads "Agent Runtime — M03 live graph" (app is M06). Stale label.
- 🟢-2 `favicon.ico` 404 in the dev server.
- 🟢-3 New graph nodes don't auto-fit to viewport after a state change (user must pan/minimap). Minor UX.
- 🟢-4 *(test-plan defect, mine)* the plan's `$REG = mcp_servers.sqlite` is wrong — there is no standalone registry file; the registry is the drone session DB. Fixed in the plan in this PR.
- 🟢-5 *(test-plan defect, mine)* Scenario C assumed auto history-replay on restart; v0.1 has no auto-resume (RecoveryDialog path, out of scope §0). Fixed in the plan in this PR.

---

## Passed (recorded so M07 does not re-investigate)

- **First-run / key:** key saves to OS keychain; relaunch does not re-prompt (no stub backend; gotcha #29 clear).
- **Smoke render:** smoke session calls Anthropic live, streams, renders an agent node (status `complete`, `tokensTotal 34`) — the main→renderer path is sound (note: this does NOT persist signals — see 🔴-2).
- **Graph fills window** (gotcha #70 — the literal M04 IRL bug — NOT present here): canvas tracks window width; pan/zoom work.
- **Node labels** non-blank/meaningful (gotcha #71 clear).
- **Approval panel** (D3): surfaces with title + task count + Approve/Revise/Cancel; dismisses on `plan_approved`; PlanNode transitions to `in_progress` (0/3).
- **Gap panel** (D8): critical-severity red bar + tool name + suggested action; GapNode renders; dismisses on `gap_resolved`.
- **Budget bar** (D7): four visibly-distinct states (amber → orange "Downshifted" → red "suspended" → deep-red "session terminated") — gotcha #67 clear for budget; click opens settings (state-wiring is 🟡-4).
- **HITL panel / Uncertainty** functional content (D4, D10): correct buttons/actions + remaining-count; rendering shell is 🟡-1.
- **MCP transport (real, Layer A):** `cargo test -p runtime-mcp --features integration` green incl. `stdio_against_reference_server_everything` (real rmcp over stdio against a real server) + namespace_resolution 15/15 + registry 11/11 + auth 7/7. App-side Test connection listed 14 real filesystem tools — connect path is solid; only the registry *persistence* path fails (🔴-1).

## Not run (blocked by a root above — not failures, not re-logged)

- B11–B17 (offline/remove/restart-persist/auth) — depend on the added server being visible; blocked by 🔴-1.
- D9 (capability-violation modal) — same component/root as 🟡-1; not re-tested.

---

## Disposition / routing

- **🔴-1, 🔴-2 → focused fix cycle before M07 Stage A begins.** Likely one persistence-path/wiring root. Each fix lands a regression test that exercises the *assembled running app*, not an isolated component (the gap Stage-V could not see).
- **🟡-1..4 → M07 Stage A** `<read_prior_milestones>` carry-forward (M06-IRL → M07.A, mirroring M04-IRL → M05.A).
- **🟢-1..3 → `docs/tech-debt.md`.** 🟢-4, 🟢-5 → fixed in `docs/M06-irl-test-plan.md` in this PR.
- This document is the M07 Stage A gating input. M07.A does not start until 🔴-1 and 🔴-2 are fixed and their cards re-tested green.

---

## Resolution (M06.5 fix cycle)

> **Appended by M06.5 Stage C.fix** (2026-05-18). Prior sections above are
> **unmodified** — this section is append-only (the file is not the
> `docs/gap-analysis.md` append-only ledger, but the same audit-trail
> discipline applies; mirrors the M04-irl → M05.A precedent). The M06.5
> fix cycle (`docs/build-prompts/M06.5-irl-fix.md`) **code-resolved** the
> two 🔴 cards on branch `claude/m06.5-irl-fix` (commits `7fc3277` /
> `9653718`; ADR-0012 / ADR-0013) with **automated assembled-composition
> regression tests as the interim verification of record**. The
> **real-app IRL re-confirmation is deferred-and-tracked to the post-M07
> IRL pass** per the between-milestone IRL model (maintainer-directed,
> 2026-05-18) — gotcha #23 (a Tauri 2.x window cannot be driven/observed
> from the agent side) is *why* the real-app step cannot run in-stage;
> the disposition is **deferred, not closed**.

### 🔴-1 — MCP registry write resolves to a stray DB → **CODE-RESOLVED (interim verification of record); real-app IRL re-confirmation deferred to post-M07 IRL pass**

- **Fix:** M06.5 Stage A.fix, impl commit **`7fc3277`** (red `68b45ed`).
  `open_mcp_client` and the drone now resolve the session DB through one
  shared path-agnostic seam `session_db::session_db_path` +
  `SESSION_DB_FILENAME` constant (`src-tauri/src/session_db.rs`, new);
  the independent `dir.join("mcp.sqlite")` is gone. The `Registry` stays
  path-agnostic (untouched). **ADR-0012** (single source-of-truth
  session DB; registry shares it via a second WAL connection) flipped
  `Proposed → Accepted` in this cycle.
- **Interim verification of record (durable, automated — the Stage-V
  blind spot, now pinned):** three assembled-path regression tests in
  `src-tauri/src/session_db.rs`, green in the full v1.6 canonical CI
  suite (`cargo test --workspace --features runtime-mcp/test-helpers`,
  the `cargo-test` job in `.github/workflows/ci.yml`). These exercise
  the assembled `src-tauri` composition (not the isolated `Registry`
  Stage-V verified) and stand as the **interim** verification until the
  post-M07 real-app IRL pass re-confirms:
  - `registry_path_equals_drone_session_db_path` — the path
    `open_mcp_client` resolves is byte-identical to the path
    `resolve_db_path` gives the drone (the assertion Stage-V
    structurally lacked).
  - `add_server_then_list_round_trips_through_the_same_store` —
    add→list across a *separately-opened* connection at the resolved
    path (ADR-0012's two-connection invariant; the
    drone-reads-what-the-UI-wrote contract).
  - `no_stray_mcp_sqlite_path_literal_constructed` — regression pin
    against re-introducing a divergent registry filename.
- **Real-app IRL re-confirmation (Scenario B4/B5/B7) — DEFERRED to the
  post-M07 IRL pass (tracked, not closed).** gotcha #23: a Tauri 2.x
  window cannot be programmatically driven or observed from the agent
  side, so the real-app UI re-run cannot execute in-stage; per the
  between-milestone IRL model (maintainer-directed 2026-05-18) it is
  deferred to the post-M07 real-app IRL pass and tracked in the
  carry-forward below — **the code fix + the interim automated tests
  unblock M07.A; the real-app card is not marked closed until that
  pass re-confirms it.** Recorded **before** (verbatim, this document
  §🔴-1): *"Three DB files in `%LOCALAPPDATA%\dev.aria-runtime.app\`:
  the row landed in `mcp.sqlite` (4 KB, stale 12:49); the **live drone
  DB is `session.sqlite`** (3 MB, actively written, `mcp_servers` empty
  there); plus a 0-byte stray `mcp_servers.sqlite`. Three divergent
  path resolutions; the system reads `session.sqlite` → server
  invisible and (downstream) unusable."* — and *"audit shows
  `mcp_installed` → **MCP Servers panel still says "No MCP servers
  installed."**"* The code fix makes both call sites resolve the one
  `session.sqlite` and the round-trip test proves cross-connection
  visibility in that single file; the post-M07 IRL pass re-runs the
  real-app repro to confirm the observable user-facing behavior.

### 🔴-2 — Agent signal stream not persisted to the live drone DB → **CODE-RESOLVED (signals; interim verification of record); real-app IRL re-confirmation deferred to post-M07 IRL pass; `token_usage` carries as a distinct finding**

- **Fix:** M06.5 Stage B.fix, impl commit **`9653718`** (red
  `fdd0c8e`; labelled follow-up `f1129e4` — test composition-model +
  mechanical clippy/fmt + the `signal_kind` unit test). A private
  `persist_signal` at the single `AgentSdk::emit` choke point
  (`crates/runtime-main/src/sdk/agent_sdk.rs`) now persists every
  signal-bearing `AgentEvent` to the drone via the **existing**
  `DroneClient::write_signal` → `DroneCommand::WriteSignal` → drone
  `handle_write_signal` (which also runs the VDR + plan projectors),
  under the run's `SessionId`, additive to the unchanged
  `event_tx.send`, best-effort (a transient drone-IPC failure is
  logged, never aborts the run). No new field / constructor /
  IPC-protocol change.
- **Second necessary condition (surfaced + maintainer-approved during
  the assembled-app regression build; the phase doc diagnosed only the
  missing emission):** `signals.session_id` is a FK into `sessions(id)`
  under `PRAGMA foreign_keys=ON`, and the drone seeds exactly one
  `sessions` row = its `--session-id`; `DroneLifecycle::spawn` minted a
  `Uuid` independent of `run_smoke_session`'s `SessionId::new()`, so
  every signal was silently FK-rejected even with the emission wired.
  `DroneLifecycle::sdk_session_id()` now exposes the seeded id; it is
  managed state; `run_smoke_session[_with]` builds the `AgentSdk` with
  that shared `SessionId` (`src-tauri/src/{drone_lifecycle,main,commands}.rs`
  — composition-layer fix parallel to 🔴-1/ADR-0012, no drone/IPC
  change). Recorded as **ADR-0013** (cross-process run identity; the
  drone-seeded session id is canonical, the in-process SDK adopts it),
  `Proposed → Accepted` in this cycle — the 🔴-2 sibling to ADR-0012.
- **Interim verification of record (durable, automated — the Stage-V
  blind spot, now pinned; stands until the post-M07 real-app IRL pass
  re-confirms):** assembled real-drone-subprocess regression in
  `crates/runtime-main/tests/smoke_signal_persistence.rs`, green in the
  full v1.6 canonical CI suite (same `cargo-test` job). Drives
  `AgentSdk::run_agent` (the exact path `run_smoke_session_with` wraps)
  against a real drone subprocess with a stub provider (no live
  Anthropic) — **not** a manual `client.write_signal()` like the
  existing-green `recovery_lifecycle.rs` (the Stage-V blind spot):
  - `smoke_session_persists_signals_to_live_drone_db` — signals land
    under the run's session id.
  - `smoke_session_signal_count_matches_emitted_event_count` — wiring
    complete, not partial.
  - `transient_signal_write_failure_does_not_abort_run` — drone killed
    mid-run → run still `Ok`, renderer sink intact.
  - plus the `signal_kind_maps_each_coarse_category` unit test pinning
    the `AgentEvent → signals.type` mapping.
- **`token_usage = 0` — DISTINCT OPEN FINDING, carries to M07.A (NOT
  resolved by B.fix).** The IRL ground truth had **both** `signals = 0`
  *and* `token_usage = 0`. B.fix closes the signal stream; it does
  **not** populate `token_usage`, because **no production code writes
  `token_usage` anywhere** — the sole `INSERT` is `#[cfg(test)]` in
  `crates/runtime-drone/src/vdr.rs`, and `handle_write_signal` runs
  only the VDR + plan projectors, neither of which targets
  `token_usage` (`command_handler.rs` projector set;
  `vdr.rs` `is_projection_eligible` = `decision|verify`). This is a
  *separate missing-projector defect*, not part of 🔴-2's
  missing-emission, and was maintainer-approved as out of B.fix scope.
  It carries to **M07 Stage A** alongside 🟡-1..4 (a new distinct
  finding, tracked here so M07.A absorbs it; severity: persistence
  completeness for budget/recovery in M07's live loop — to be
  triaged at M07.A intake).
- **Real-app IRL re-confirmation (Scenario A6 + C-7) — DEFERRED to the
  post-M07 IRL pass (tracked, not closed)**, same disposition as 🔴-1:
  gotcha #23 is why the real-app re-run cannot execute in-stage; per
  the between-milestone IRL model (maintainer-directed 2026-05-18) it
  is deferred to the post-M07 real-app IRL pass and tracked in the
  carry-forward below. The code fix + the interim automated tests
  unblock M07.A; the real-app card is not closed until that pass
  re-confirms. Recorded **before** (verbatim, this document §🔴-2):
  *"`session.sqlite`: `signals = 0`, `token_usage = 0` — while
  `heartbeats = 15155` and `snapshots = 14` in the *same* DB. The
  drone is alive and persisting (heartbeats/snapshots) to the correct
  file, but the **agent signal stream lands nothing**. Signals exist
  in none of the three DBs."* The code fix makes the assembled smoke
  path persist signals under the run's (now shared) session id;
  `token_usage` remains `0` by the distinct open finding above; the
  post-M07 IRL pass re-runs the real-app repro to confirm the
  observable behavior.

### Disposition update (M07.A gate)

- **🔴-1, 🔴-2 → CODE-RESOLVED with automated assembled-composition
  tests as the interim verification of record.** The code fixes
  (`7fc3277` / `9653718`; ADR-0012 / ADR-0013) plus the durable
  automated regression tests above (which exercise the assembled
  composition Stage-V could not see) are the **interim** verification.
  **The two 🔴 no longer block M07 Stage A start.** The **real-app IRL
  re-confirmation is deferred-and-tracked to the post-M07 IRL pass**
  per the between-milestone IRL model (maintainer-directed) — gotcha
  #23 is why it cannot run in-stage; the cards are **deferred, not
  closed**, until that pass re-confirms the observable behavior.
- **Post-M07 real-app IRL carry-forward list** (the between-milestone
  IRL pass after M07 re-runs these against the built app):
  - **🔴-1 + 🔴-2 real-app IRL re-confirmation** — Scenario B4/B5/B7
    (registry add→list visible in the MCP Servers panel + the
    `session.sqlite` `mcp_servers` row) and Scenario A6 + C-7
    (`session.sqlite` `signals > 0` under the run session id). Code is
    resolved + interim-automated-verified; this is the deferred
    real-app observable confirmation.
  - **`token_usage = 0`** — a distinct missing-projector finding (no
    production `token_usage` writer; sole `INSERT` is `#[cfg(test)]` in
    `runtime-drone/vdr.rs`).
  - **🟡-1..4** — HITL `ui_variant` not honored; `npx` Windows `.cmd`
    shim; stale Test error banner; budget settings not state-wired.
    Untouched by this fix cycle, per scope lock.
- **🟡-1..4 also carry to M07 Stage A `<read_prior_milestones>`
  unchanged** (M06-IRL → M07.A, mirroring M04-IRL → M05.A), in addition
  to the post-M07 real-app IRL re-run above.
- **🟢-1..5 → unchanged** (🟢-1..3 in `docs/tech-debt.md`; 🟢-4/-5
  fixed in `docs/M06-irl-test-plan.md`).
- Per CLAUDE.md §20, this fix cycle adds **no `docs/gap-analysis.md`
  entry**; the resolution of 🔴-1/🔴-2 (and the new `token_usage`
  carry) flows into **M07's gap-analysis Carry-forward** section.
- The assembled-app-regression mandate (each fix's regression test must
  exercise the assembled running-app path, not the isolated component
  that already passes its Stage-V unit test) is recorded as the
  empirical input for **Cycle 2 (M06.6)** to graduate into a permanent
  verification stage — recorded only; this cycle changed no protocol
  artifact.
