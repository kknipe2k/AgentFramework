# M06 IRL Test Plan — Whole-App Manual Verification (cumulative M01–M06)

> **Purpose.** A focused, executable manual test pass against the *running* Tauri app as of the M06 (MCP Basic) merge (`081044a`). Not all-encompassing — the highest-yield checks weighted toward the M04-IRL bug classes (the prior IRL pass found 5 real bugs, all in the live-render / window-sizing / error-surfacing / new-UI surface).
>
> **How findings feed forward.** This mirrors `docs/M04-irl-test-plan.md` (whose findings fed M05 Stage A). M06-IRL findings are tagged 🔴 / 🟡 / 🟢 and carried into **M07 Stage A** (🔴 = fix before M07 work; 🟡 = M07.A absorb; 🟢 = `docs/tech-debt.md`). The Findings Log at the bottom is the capture surface.
>
> **Authored by:** Claude (orchestration), reviewed by maintainer. Manual execution by maintainer on the build/test machine.

---

## 0. Critical scope fence (read first — prevents chasing non-bugs)

v0.1's **only live agentic path** is `run_smoke_session` — a hardcoded, no-tools, single-turn prompt ("say only the word: hello"). It emits **no `ProviderEvent::ToolUse`**, runs no plan loop, spawns no sub-agents, hits no budget threshold, triggers no failure escalation.

Therefore the following are **shipped + contract-verified (Stage-V) but NOT reachable through normal app use until M07** (the agent-with-tools / multi-turn loop). Exercising them by normal use will *look* like bugs but are **not** — do not log them as findings:

| Shipped, not live in v0.1 | Why | Verified by |
|---|---|---|
| Plan-approval flow (ApprovalPanel) | no plan loop in smoke path | M04 Stage-V + Playwright |
| HITL / failure-escalation modal | smoke never fails/escalates | M04 Stage-V |
| Budget header bar thresholds | single no-tool call never crosses threshold | M04 Stage-V |
| Recovery / Uncertainty prompts | no in-flight tool-call uncertainty in smoke | M04 Stage-V |
| GapPanel (gap detection) | no framework load in smoke path | M05 Stage-V |
| Capability-violation modal / tier-badge change | L1 enforcer fires only on tool dispatch (none in smoke) | M05 Stage-V |
| Agent *using* an MCP tool end-to-end | concrete `McpDispatcher` construction + `ConnectionResolver for McpClient` + live loop = ADR-0011 trace-#11b **M07 carry-forward** | M06.V #11a (mock-verified seam) |
| §5a short-name ambiguity / alias resolution *at dispatch* | dispatch-path; no live dispatch in v0.1 | M06.V #6 (🟡→M07.A) |

These need M07's loop OR the `window.__graphStore` test-injection affordance — both out of scope for a *normal-use* IRL pass.

**What IS live + user-exercisable in v0.1:** first-run/key, the no-tools smoke session + its live-graph render, the full MCP server-management UI (M06), persistence/restart integrity, error-surfacing + dev-logging. This plan tests exactly that surface.

---

## 1. Environment / prerequisites

- Built Tauri app runnable on the test machine (`npm run tauri dev` or the packaged binary).
- Ability to **clear/inspect the OS keychain** (Windows Credential Manager; service names `agent-runtime` for the API key, `agent-runtime/mcp` for MCP secrets).
- A valid Anthropic API key (and a deliberately-bad one for the error path).
- `npx` available (Node) — the reference server is `@modelcontextprotocol/server-filesystem`.
- A throwaway scratch directory for the filesystem MCP server to expose.
- Access to the app-local-data dir to inspect `skills.audit.jsonl` and `mcp_servers.sqlite`.
- DevTools / stdout visible (to confirm structured logging per gotcha #31).

Record machine + OS + app build SHA (`081044a` or later) in the Findings Log header.

---

## 2. Test cards

Each card: **Action** → **Expected observable** → **Pass/Fail** → **Targets** (bug-class / gotcha the card is designed to catch). Execute in any order, but the Scenarios (§3) sequence them efficiently.

### C1 — First-run API-key setup + persistence (M02)
- **Action.** Launch with no stored key (clear the `agent-runtime` keychain entry first). Observe the setup prompt. Enter a valid key. Quit. Relaunch.
- **Expected.** First launch shows a key-entry prompt; after entry the app proceeds to the main surface. Relaunch shows **no** re-prompt (key read from keychain).
- **Pass/Fail.** PASS = no re-prompt on relaunch AND no plaintext key on disk. FAIL = re-prompt, OR key found in a config file/log.
- **Targets.** Gotcha #29 (keyring 3.x silent-stub backend — if the key "saves" but relaunch re-prompts, the stub backend is in use). M02 keychain contract.

### C2 — Live smoke session end-to-end (M02 — the one true live path)
- **Action.** Trigger `run_smoke_session` (the app's smoke/run affordance).
- **Expected.** A real network call to Anthropic; the streamed response ("hello") arrives; events propagate main→drone→renderer; the live graph renders an agent node with token/event activity that updates as the stream arrives.
- **Pass/Fail.** PASS = visible streamed render + a node appears + tokens/events update. FAIL = silent no-op, hang, or a node that never populates.
- **Targets.** M02 event pipeline end-to-end; gotcha #66 (the render must reflect the stream, not just "the call returned ok").

### C3 — Live graph fills the window (M03)
- **Action.** With the graph populated (post-C2), resize the window to a typical desktop size (≥1280px) and small.
- **Expected.** The graph canvas fills the available window width/height; pan + zoom work; nodes are not clipped into a narrow column.
- **Pass/Fail.** PASS = canvas tracks window size, pan/zoom functional. FAIL = graph constrained to a fixed narrow column (e.g., ~720px) at large window sizes.
- **Targets.** Gotcha #70 — *this was a real M04 IRL bug* ("view screen too small — needs to fill width of window"). Highest-yield card.

### C4 — Node label renders non-blank (M03/M04)
- **Action.** Inspect the agent node (and any other nodes) rendered from C2.
- **Expected.** Every node shows a human-meaningful label. No node shows an empty string, "untitled", or a raw id where a name belongs.
- **Pass/Fail.** PASS = all labels non-empty + meaningful. FAIL = blank/"untitled"/raw-id label.
- **Targets.** Gotcha #71 — *real M04 IRL bug class* (schema field absent → renderer shows blank string instead of a fallback).

### C5 — Error surfacing is human-readable (M02 — gotcha #30/#31)
- **Action.** Set a deliberately bad/empty API key (or clear it post-setup), trigger the smoke session.
- **Expected.** The UI shows a **human-readable** error (e.g., "authentication failed" / "no API key"). DevTools/stdout shows a **structured** error line (tracing initialized).
- **Pass/Fail.** PASS = readable UI error + structured log. FAIL = `"[object Object]"` in the UI, OR zero log output from inside the Tauri command.
- **Targets.** Gotcha #30 (`unwrapCmdError` — structured Rust error must not collapse to `[object Object]`); gotcha #31 (`tracing_subscriber` init — commands must log).

### C6 — Restart / recovery integrity (M01/M04)
- **Action.** After a successful smoke run (C2), hard-restart the app. Observe the graph + session state.
- **Expected.** Prior session history **rebuilds from drone SQLite** (replayed projection) — it does **not** re-execute (no second Anthropic call; verify no new network activity / no token cost). The graph reflects the prior session's final state.
- **Pass/Fail.** PASS = history visible post-restart with zero re-execution. FAIL = blank state, OR a second live Anthropic call fires on restart.
- **Targets.** M01 snapshot/append-only; M04 recovery ("resume rebuilds, doesn't re-execute" — gotcha #15); gotcha #69 (IPC multi-call — the post-restart drone IPC reads must work, not just the first session's).

### C7 — MCP add stdio + Test + status-color + persist (M06)
- **Action.** Settings → MCP Servers → Add. Name `fs-test`, transport **stdio**, command `npx`, args `-y @modelcontextprotocol/server-filesystem <scratch-dir>`. Observe the tier-eval display *before* Confirm. Confirm. Click **Test connection**. Inspect the MCPNode on the graph. Quit + relaunch.
- **Expected.** Tier-eval outcome shown pre-Confirm (Promoted→Novice fallback messaging per §8.security L4). Test returns a real tool list (`read_file`, `write_file`, `list_directory`, …) with canonical `fs-test__<tool>` names. MCPNode shows a **connected** indicator whose **computed color** is the connected color (not merely the class present). After relaunch the server is still listed (registry persisted to `mcp_servers.sqlite`).
- **Pass/Fail.** PASS = tool list populated + correct-color indicator + survives restart. FAIL = empty tool list on a reachable server (wrong-field read), OR class present but no color (CSS-missing), OR server gone after restart (persistence broken).
- **Targets.** Gotcha #67 (*M04 IRL bug class* — component rendered ≠ CSS exists; assert computed style, not className). Gotcha #68 (wrong-field read — tool list must reflect the discovered tools, not an unpopulated field). Stage C registry/SQLite + Stage E UI + Stage D §5a naming.

### C8 — MCP per-server auth: keychain, never in audit (M06 — §13.5)
- **Action.** Add an MCP server with an `auth_secret` populated. After add, inspect `skills.audit.jsonl` and the `agent-runtime/mcp` keychain.
- **Expected.** The secret value lands in the OS keychain. `skills.audit.jsonl` contains an `mcp_installed` line **then** an `mcp_auth_granted` line, in that order, correlated to the server name — and **neither line (nor any line) contains the secret string**.
- **Pass/Fail.** PASS = secret in keychain, ordered correlated audit lines, zero secret leakage in audit/logs. FAIL = secret absent from keychain (stub backend, gotcha #29), OR secret string appears anywhere in `skills.audit.jsonl` or stdout.
- **Targets.** Spec §13.5 (no secret in audit); gotcha #66 (correlated emission ordering); gotcha #29 (keychain backend).

### C9 — MCP offline detection (M06 health-ping)
- **Action.** With `fs-test` connected (C7), kill the `npx`/server-filesystem subprocess externally (Task Manager / `kill`). Wait through one health-ping interval (~30s).
- **Expected.** The MCPNode transitions to **error/disconnected**; an `mcp_missing` event surfaces (and routes through the existing `on_gap` HITL trigger — observable as the gap/missing surface, not a crash).
- **Pass/Fail.** PASS = node transitions + `mcp_missing` surfaces within ~1–2 ping intervals. FAIL = node stuck "connected" after the subprocess is dead, OR the app crashes/hangs.
- **Targets.** Stage C lifecycle (30s health-ping); reuse of existing `mcp_missing`/`on_gap` (no new failure pathway).

### C10 — MCP add-modal validation + transport switch + tier (M06 Stage E)
- **Action.** Open Add modal. Try server name `Bad_Name!` (violates `^[a-z0-9][a-z0-9-]*$`). Switch transport stdio↔http. Add an **http** server (any reachable test URL, or observe field behavior).
- **Expected.** Invalid name disables/rejects submit with a clear validation cue. Switching to http replaces command/args fields with a `url` field. Tier-eval outcome is displayed before Confirm for both transports. An http server, once added, persists with `transport_type = http`.
- **Pass/Fail.** PASS = invalid name blocked, fields swap correctly per transport, tier-eval shown, http persists with correct type. FAIL = invalid name accepted, fields don't swap, tier-eval absent, or http persists as stdio.
- **Targets.** Stage E form validation + transport-conditional fields; gotcha #67 (any new validation/error class must have a real CSS rule); M05.D tier display.

---

## 3. Walkthrough scenarios (sequence the cards efficiently)

### Scenario A — Cold start → first run (covers C1–C5)
1. Clear the `agent-runtime` keychain entry. Launch → **C1** (setup prompt → enter valid key).
2. Trigger the smoke session → **C2** (live stream renders).
3. While the graph is populated, resize the window large + small → **C3** (fills window, pan/zoom).
4. Inspect node labels → **C4** (non-blank).
5. Clear/replace the key with a bad value, retrigger → **C5** (readable error, structured log; not `[object Object]`).

### Scenario B — MCP server lifecycle + audit + persistence (covers C7–C10 + C8)
1. Settings → MCP Servers → Add `fs-test` stdio (npx filesystem) → confirm **C10** tier-eval shown pre-Confirm → Confirm.
2. Test connection → **C7** (tool list, MCPNode connected with real color).
3. Inspect `skills.audit.jsonl` → `mcp_installed` present, no secret.
4. Add a second server **with an auth secret** → **C8** (keychain entry; ordered `mcp_installed`→`mcp_auth_granted`; zero secret leakage).
5. Try invalid name `Bad_Name!`, then switch to http → **C10** (rejected; fields swap).
6. Kill the filesystem subprocess → **C9** (node → error + `mcp_missing`).
7. Quit + relaunch → both servers still listed, correct transport types/status (**C7** persistence).
8. Remove `fs-test` → audit `mcp_uninstalled`; quit/relaunch → confirmed gone.

### Scenario C — Restart / recovery integrity (covers C6 + cross-cut)
1. Run a fresh smoke session (C2). Note the final graph state + token count.
2. Add one MCP server (any stdio).
3. Hard-restart the app (kill, not graceful quit).
4. Confirm: **C6** — history replays from SQLite, **no second Anthropic call** (watch network / token cost = unchanged), graph reflects the prior session; the MCP server is still listed.

---

## 4. Findings Log (capture surface — feeds M07 Stage A)

> Severity: 🔴 fix before M07 work begins · 🟡 M07 Stage A absorbs · 🟢 `docs/tech-debt.md`. One row per finding. Cite the card (C#), the observed-vs-expected, and the suspected bug class.

| # | Card | Severity | Observed vs expected | Suspected class / gotcha | Disposition |
|---|---|---|---|---|---|
| _(fill during execution)_ | | | | | |

**Environment header (fill at start):** machine / OS / app build SHA / Anthropic key source / date.

**Disposition rule:** 🔴 → blocks M07.A start (fix + re-test the card); 🟡 → recorded in M07 Stage A `<read_prior_milestones>` carry-forward; 🟢 → `docs/tech-debt.md` TD-NNN. Any card that cannot be executed (environment blocker) is logged as **BLOCKED** with the reason — not PASS.

---

## 5. Sign-off

- [ ] All 10 cards executed or explicitly BLOCKED with reason.
- [ ] Scenarios A/B/C walked end-to-end.
- [ ] Findings Log completed + each finding dispositioned.
- [ ] 🔴 count = 0 to proceed to M07 Stage A without a fix cycle; any 🔴 triggers a focused fix + card re-test before M07.A.
- [ ] Scope fence (§0) respected — no shipped-but-M07 surface logged as a bug.

> Findings carried into M07 Stage A per the M04-IRL → M05.A precedent. This artifact is the manual-verification companion to the Stage-V contract-fidelity pass; V verifies the contract, IRL verifies the lived experience (the gotcha #66 "tests-pass-but-contract-fails" / #67 / #70 / #71 classes the automated gates structurally cannot see).
