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
