# M08 IRL Findings — post-M08 Workbench walk-through

Findings from the manual real-app pass of `docs/M08-irl-test-plan.md` against the
M08 merge (`main`). Real Tauri desktop app on Windows (run elevated), no mocks.
Feeds the M08.5 fix-cycle + M09 Stage A.

## Header

- Pass: post-M08 IRL, Scenarios A–E.
- App: `main`, post-M08 merge.
- Machine: Windows; app + terminals run as admin (elevated).
- Completed: A ✓ · B blocked (drag) · C ✓ · D ✓ · E partial (E4/E5 blocked).

## Findings

| # | Sev | Step | Observed vs expected | Cause / disposition |
|---|---|---|---|---|
| 1 | 🔴 | B | Palette → canvas drag-to-instantiate is dead. Grab cursor on hover, but no dragstart / ghost / drop. React Flow node *reposition* (pointer-drag) works → HTML5-native-DnD-specific. | Tauri's native drag handler swallows HTML5 DnD. Suspected fix: `dragDropEnabled: false` on the window in `tauri.conf.json`. Playwright e2e missed it (mocks Tauri, plain browser). → M08.5 fix-cycle. |
| 2 | 🔴 | D1 | Tester runs a hardcoded "smoke" session, not the loaded framework. Ran twice with ARIA loaded; both scoped graphs showed the "smoke" agent. Token capture works (run 2: in 24 / out 179). | `run_test_session_with` reuses the smoke-session construction and never substitutes the candidate framework's agents. Breaks MVP §M8 criterion 5. F1 regression + V missed it (asserted run+token+isolation, not "ran the candidate"). → M08.5 fix-cycle. |
| 3 | 🔴 | E4 | "Add MCP Server" modal — Test / Add / Cancel buttons all non-responsive. Modal renders (tier banner shows) but no button reacts. | Blocks adding any MCP server. → M08.5 fix-cycle. |
| 4 | 🟡 | C1 | Loading a framework stacks all nodes at {0,0}. | `nodePositions` only fills on drag-drop; a loaded framework has none → {0,0} fallback. Auto-layout-on-load deferred ("not D1 scope"). Repositioning by hand works. → M09.A / fix-cycle. |
| 5 | 🟡 | C2 | Inspector capability summary reads "none" for every item on ARIA (which has file/shell caps), though Validate reports no problems. | Dead/empty Inspector surface. → M09.A. |
| 6 | 🟡 | E1 | UI does not disclosure-gate by tier — a Novice sees Promoted-only controls (MCP management). | Panels should show/hide/disable per capability tier. → M09.A. |
| 7 | 🟢 | D1 | Tester first run reported token spend 0·0·0; the second run captured 24/179 correctly. | Intermittent first-run zero — watch. → tech-debt. |
| 8 | 🟢 | E3 | Budget "Save cap" gives no click feedback (the save itself is real — calls `set_global_budget`). | → tech-debt. |
| 9 | 🟢 | A | `<h1>` reads "Agent Runtime — M03 live graph" — stale milestone label. Drop the M0 reference; add a small version. | → tech-debt / quick fix. |

## Blocked

- **M06.5 IRL 🔴-1** (MCP-registry mis-resolve) re-confirm — could not execute: the MCP Add modal is dead (finding #3). Re-attempt after #3 is fixed.

## Disposition

- **3 🔴 → an M08.5 fix-cycle** before M09 — staged X.5 prompts, strict red→green TDD, gates, and **real-app regression tests**. The root failure class: no quality gate and not the Verifier runs the real Tauri desktop app (Playwright mocks Tauri); the fixes must be regression-guarded against that class.
- **3 🟡 → M09 Stage A** intake.
- **3 🟢 → `docs/tech-debt.md`**.

## Sign-off

- [x] Findings reviewed; the M08.5 fix-cycle scoped (3 🔴).
- [x] M08 is **not demo-ready** until the 🔴 fix-cycle lands and a re-run of `docs/M08-irl-test-plan.md` is clean. *Resolved at M08.5.E.fix — see Resolution section below.*

---

## Resolution (M08.5 fix cycle)

The M08.5 fix cycle scoped three 🔴 from the M08 IRL pass; all three
regression tests committed on `claude/m08.5-irl-fix` and the IRL
re-verify (2026-05-23, Windows build machine) confirmed PASS for each.
The re-verify ALSO surfaced **4 new 🔴 + 11 new 🟡 + 1 🟢 elevation**
that route per the M08.5.5 + M08.6 split (Option C, orchestration
finalized 2026-05-23). M08 is therefore **NOT yet demo-ready**: M08.5
closed its scoped 🔴 cleanly but the next cycle must close the new ones
before the demo path opens.

### Scenario B — 🔴-1 palette drag → PASS

In Builder mode, dragging an Agent item from the Palette onto the empty
canvas instantiated a node where dropped. The node was draggable to a
new spot and persisted. The JSON tab and Inspector both reflected the
canvas state live.

Pre-fix on `main`: drag was dead — the OS handler swallowed the pointer
gesture before WebView2 could synthesize HTML5 DnD.

Fix shipped: `src-tauri/tauri.conf.json` sets `dragDropEnabled: false`
on the main window (B.fix commit `bdb76e5`).

🔴-1 closed.

### Scenario D1 — 🔴-2 Tester root agent name → PASS

Loaded a framework with `session_root_agent: "demo-agent"` and inline
agent role `lead-orchestrator`. Inspector → Test opened the Tester
modal. Entered task `say hello world`, clicked Run. Tester scoped-graph
pane showed the root node labeled `lead-orchestrator` (the agent's
role) — NOT `smoke`. Token spend populated (in 20 / out 23 / total
43 / 1585 ms). Result green PASS. VDR panel showed `null` (known
M08.V 🟡 #1 carry-forward, expected).

Pre-fix on `main`: Tester emitted hardcoded `agent_name: "smoke"`
regardless of the candidate framework's root agent.

Fix shipped: `framework_loader::capability_map::root_agent_role()`
walks `framework.agents[]` (C.fix commit `eaaddda`); SDK derives root
agent_name from the resolver when capability_wiring is present;
smoke-path byte-stability preserved via `map_or_else` fallback.

🔴-2 closed.

### Scenario E4 — 🔴-3 MCP Add-Server modal buttons → PASS (with Windows-spawn caveat)

Settings → MCP Servers → Add opened the modal cleanly. All three
buttons responded to clicks (backend invoked in every case):

- Test → backend spawned `npx`; Window A logged repeated `The filename,
  directory name, or volume label syntax is incorrect.` OS errors;
  UI showed infinite spinner (no failure surface).
- Cancel → modal closed cleanly.
- Add → backend invoked; same `npx` spawn failure as Test; modal stuck
  in infinite loop.

Pre-fix on `main`: all three buttons dead — clicks dispatched at the
WebDriver layer but the bound onClick never ran at the WebView2 layer
(one of three candidate causes: trapped stacking context, transparent
fixed interceptor, off-viewport action row).

Fix shipped: `MCPServerAddModal.tsx` wrapped in
`createPortal(modal, document.body)` + backdrop `z-index: 50 → 1000` +
panel `max-height: 90vh; overflow: auto` (D.fix commit `caf6f13`).
Diagnosis-independent triple, robust across all three candidate causes
per phase doc D.3.2.

🔴-3 closed for button responsiveness. NEW 🔴 #6 logged below: MCP
Add/Test on Windows infinite-loops on `npx` spawn — separate bug,
M08.5.5 scope.

### Scenario E5 — M06.5 🔴-1 MCP-registry mis-resolve → STILL OPEN

`sqlite3 $REG "SELECT name,transport,status FROM mcp_servers;"`
(note: column is `transport`, not `transport_type` — test plan typo,
🟡 #8 below) returned EMPTY for `session.sqlite`.

Stray `mcp.sqlite` exists at `$APPDATA_DIR\mcp.sqlite` (4096 bytes,
dated 5/17/2026). The file contains a complete session-DB schema
(tables: `sessions`, `signals`, `snapshots`, `vdr`, `plans`, `tasks`,
`mcp_servers`, `heartbeats`, `token_usage`, `skills`, `_migrations`,
`sqlite_sequence`) plus an `fs-test` row from 5/17/2026 prior testing.

The two databases have INCOMPATIBLE `mcp_servers` status CHECK
constraints:

- `session.sqlite` mcp_servers: status CHECK
  `('connected', 'disconnected', 'health_pending', 'error')` —
  default `'disconnected'`
- stray `mcp.sqlite` mcp_servers: status CHECK
  `('configured', 'connected', 'errored', 'disabled', 'failed')` —
  default `'configured'`

Today's Add attempt infinite-looped (🔴 #6) and never wrote a row to
either DB; the row visible in stray `mcp.sqlite` is the 5/17 prior
testing artifact.

Result: **M06.5 🔴-1 was NOT resolved by M08.5**. The MCP registry is
still resolving to the wrong DB AND the schemas have drifted between
the two DBs (suggesting migration didn't apply consistently). Routes
to M08.5.5 alongside 🔴 #6 (same MCP-resilience cluster).

### NEW findings from the IRL re-verify

#### 🔴 candidates (4 new + 1 still-open) — routing

| # | Finding | Class | Route |
|---|---|---|---|
| 4 | Tier state UI/backend desync — UI displays Novice but backend treats user as Promoted (MCP modal opens), Promote button no-op; persists across `npm run tauri dev` restart | State-sync | **M08.6 Stage A** |
| 5 | Builder Canvas state not persisted across app restart — user-edited agent reverted to old state after Window-A restart | State-sync | **M08.6 Stage A** |
| 6 | MCP Add/Test on Windows: infinite loop + OS error `The filename, directory name, or volume label syntax is incorrect.`; no UI surface for the failure; no row written to either DB. Likely `npx` path resolution or argument quoting in `runtime-mcp` spawn on Windows | MCP-Windows | **M08.5.5** |
| 7 | Schema drift between `session.sqlite` and stray `mcp.sqlite` — different status CHECK constraints in two SQLite DBs on the same machine for the same table name | State-sync / representation | **M08.6 Stage A** |
| M06.5 🔴-1 | Stray `mcp.sqlite` still present; MCP registry writes to wrong DB; `session.sqlite` mcp_servers empty | MCP-Windows | **M08.5.5** |

#### 🟡 candidates (11 new) — all route to M08.6 Stage A intake

1. **Promote button label** doesn't update when at target tier
   ("Promote to Promoted" reads as nonsense when already Promoted —
   should hide, switch to "Demote", or display "Promoted (active)").
2. **Settings panel always-visible** large horizontal bar — no
   progressive disclosure / no collapse.
3. **Builder Canvas generates agents with INVALID id format** (observed:
   `demo-agent@1.0.0`; schema `agent.v1.json` requires
   `^[a-z][a-z0-9-]*$` — no `@`, no `.`).
4. **Builder Canvas omits required `capabilities` block** when creating
   agents. Tester then rejects with opaque
   `data did not match any variant of untagged enum FrameworkAgentsItem`.
5. **Configure modal on Canvas lacks Save button** AND lacks
   toast/hint pointing to Inspector → Save.
6. **Inspector → Save dialog** has no preloaded filename, no extension
   hint; defaults to previous-session directory unrelated to current
   framework.
7. **`session_root_agent` has no UI surface** in the Builder; must
   hand-edit JSON tab.
8. **`docs/M08-irl-test-plan.md` stale column name**: `transport_type`
   should be `transport`.
9. **`wdio.conf.ts:31` hardcodes `src-tauri/target/release/`** — Cargo
   workspace members share workspace-root `target/`; real fix updates
   `APP_BIN_PATH` to `target/release/agent-runtime.exe`. Workaround
   documented at `docs/build-machine-tauri-driver-setup.md` Phase 3.5
   (Windows junction).
10. **`npx tauri build` does NOT build the sibling subprocess binaries**
    (`runtime-drone`, `runtime-sandbox`). They must be built separately
    via `cargo build --release -p runtime-drone -p runtime-sandbox` or
    the Tauri app crashes at startup with
    `drone IPC unavailable: spawn drone subprocess`. CI does this
    explicitly (A.fix follow-ups #3 + #4); local setup guide now
    documents it (Phase 3.3).
11. WebDriver Actions API multi-step pointer drag (B.fix's
    regression test in `tests/e2e-tauri/builder_drag.e2e.ts`) no longer
    synthesizes HTML5 `dragstart` on Chromium 148+. The IMPL ships
    correctly (B.fix's `dragDropEnabled: false` is IRL-verified on
    Windows 2026-05-23 — the node appears when dragged manually). The
    TEST MECHANISM is stale: Chromium 148+ tightened the dragstart
    synthesis threshold; the 5px intermediate pointerMove no longer
    trips it. Today's CI run (2026-05-23) confirmed: Edge
    `148.0.3967.70` on the Windows runner + Chrome `148.x` on the
    Linux runner both fail for the same reason. CI is therefore NOT
    a viable verifier of record going forward. The test is skipped
    on this branch via `it.skip(...)` (separate follow-up commit
    after this E.fix amend). M08.5.5 Stage A.fix takes the WebDriver
    mechanism fix as a first deliverable; four candidate paths
    documented in the handoff doc.

#### 🟢 elevation

`TD-009` (stale `<h1>` "Agent Runtime — M03 live graph" label):
broadened from "fix the label" to **strip ALL M0N references from
UI + add proper small-type version display in usual chrome location**.
Routes to M08.6 Stage A as part of the UI-modernization carry-forward.

#### 🟢 standing ask (re-confirmed)

UI modernization: progressive disclosure + expand/collapse for ALL
modals + DESIGN.md / Stage D protocol introduction (3-brain stack)
land post-M08.6. See `docs/protocol-revisions-irl-fail-rate.md` +
`docs/m08.5-irl-re-verify-handoff.md`.

### M08 demo-unblock statement (revised)

M08 is NOT yet demo-ready. The three scoped 🔴 closed cleanly, but the
IRL re-verify surfaced 4 new 🔴 + M06.5 🔴-1 still-open. Routing per
Option C:

- **M08.5.5 fix cycle** (~3 stages): 🔴 #6 MCP Windows infinite-loop +
  M06.5 🔴-1 stray mcp.sqlite + schema drift. Closes the
  MCP-resilience cluster.
- **M08.6 milestone** (framework representation per ADR-0022 +
  state-sync intake): 🔴 #4 tier UI desync + 🔴 #5 Canvas-not-persisted
  + 🔴 #7 DB schema drift + 11 🟡 + 🟢 elevation. The canonical-
  representation loader work naturally addresses the loader/state-sync
  gaps; the rest is Stage A intake.

Demo-ready after M08.5.5 closes the MCP path AND M08.6 closes the
state-sync path.
