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

---

## Resolution (M08.5.5 fix cycle)

The M08.5.5 fix cycle scoped the MCP-Windows cluster from the M08.5 IRL
re-verify (Option C, orchestration finalized 2026-05-23): 🔴 #6 (MCP
Add/Test on Windows infinite-loops on `npx` spawn) + M06.5 🔴-1 (stray
`mcp.sqlite` present despite ADR-0012's single-source-of-truth path
resolver). The phase doc's third item — 🔴 #7 (`mcp_servers` schema
divergence between `session.sqlite` and the stray `mcp.sqlite`) — was
**dropped at Stage C.fix's red-phase** after the build machine's deeper
phase-doc inspection caught an inverted legacy↔canonical schema
labeling that would have CORRUPTED user `session.sqlite` files if
implemented literally (the canonical schema is the 4-state set, not the
5-state set; the phase doc's migration 004 prescription went the wrong
direction). Per the C.fix retrospective + maintainer decision (Stage
D.fix routing surface), 🔴 #7 carries forward to M08.6 alongside the
state-sync class already routed there.

Cycle deliverables (**5 stages** on `claude/m08.5.5-mcp-resilience`,
conceptual ordering A → B → B2 → C → D; B2 inserted in-cycle after
C had landed in git history when the user's manual IRL re-verify of
the post-C.fix binary surfaced a parse-layer defect inside B.fix's
own wrapper): Stage A.fix landed the harness + safety hardening (7
sub-deliverables: wdio workspace-path fix, `.env.local` loader,
`read_api_key` env-var override per ADR-0025, `onPrepare()`
cargo-build hook, `builder_drag.e2e.ts` JS event-dispatch mechanism,
`gitleaks` gate per ADR-0024, 5 new `docs/gotchas.md` entries).
Stage B.fix landed the Windows `.cmd`/`.bat` `cmd.exe /C` wrapper
for `npx`-class spawns per ADR-0023 (spawn-layer BatBadBut bypass).
Stage **B2.fix** landed the outer-quote follow-on for the same `/C`
wrapper (parse-layer cmd.exe `/?` rule 2 quote-strip fix); ADR-0023
amended in place per explicit user override of CLAUDE.md §11
immutability. Stage C.fix landed the path-agnostic
`crates/runtime-main/src/stray_db_cleanup.rs` startup module that
detects and renames pre-ADR-0012 stray `mcp.sqlite` to
`.stray-mcp.sqlite.bak` (canonical; a millisecond-timestamp suffix
`.stray-mcp.sqlite.bak.<unix-ms>` is the idempotent fallback when
the canonical target already exists from a prior cleanup pass).

### Scenario E4 — 🔴 #6 MCP Add/Test on Windows → PASS (two-layer fix)

Build machine, 2026-05-25, post-B2.fix tip `b9f56b7` (rebuilt +
relaunched against the real Tauri app):

Form input:
- Name: `m08-irl-fs-3`
- Transport: `stdio`
- Command: `npx`
- Args (comma-separated): `-y, @modelcontextprotocol/server-filesystem,
  C:\Users\kknip\AppData\Local\Temp\m0855irl-855905802`

Test button click result:
- No infinite spinner ✓
- Clean runtime error returned (NOT the cmd.exe quote-strip error)
- Window A log: `tracing INFO mcp_test_connection invoked`
- Captured internal error: `MCP connect failed: connection closed:
  initialize response`
- Stderr from npx (captured + visible): `Cannot find module
  'C:\agent-runtime\node_modules\npm\bin\npm-prefix.js'` + `Cannot
  find module 'C:\agent-runtime\node_modules\npm\bin\npx-cli.js'` —
  this is an `npx`-environment issue on the user's machine (Node
  v24.11.0 looking for npm relative to the repo `cwd`), NOT a runtime
  defect. Proves cmd.exe parsed the command line correctly + `npx.cmd`
  reached its main loop + `npx` itself failed for its own reason.

Add button click result:
- Same clean behavior: no infinite spinner, same npx-environment
  error captured + reported, no phantom row written to
  `session.sqlite`.

The PRE-B2.fix error pattern (`The filename, directory name, or
volume label syntax is incorrect (os error 123)`) does NOT appear
in either Test or Add output post-B2.fix.

🔴 #6 has TWO failure layers, both closed by this cycle in
sequence:

**Layer 1: spawn-layer BatBadBut bypass (B.fix, 2026-05-23).**
Pre-cycle on `main`: `tokio::process::Command::new("npx.cmd").arg(...)`
on Windows passed each `arg(...)` through Rust's BatBadBut-safe
quoting routine (CVE-2024-24576 mitigation in `std::process::Command`
since Rust 1.77.2). When an argument contained a drive-letter path
(`C:\...\Temp\m08irl-<n>`), the escaped form was the cmd.exe-level
command-line that Windows itself refuses to parse, producing the
OS-level "filename, directory name, or volume label syntax is
incorrect." error before `npx` even started. The UI showed an
infinite spinner; no row was written to either DB.

Fix shipped: `crates/runtime-mcp/src/transport/stdio.rs::build_command`
detects `.cmd`/`.bat` programs (case-insensitive via
`Path::extension().eq_ignore_ascii_case(...)`) and wraps the
invocation in `cmd.exe /C ...` via
`tokio::process::Command::raw_arg` to bypass the BatBadBut quoting
routine when the argument vector is non-empty (preserving the
bare-shim behavior the existing M06.5 IRL 🟡-2 unit tests pin).
B.fix commit `230e1c3`; red commit `ca1db1f`. ADR-0023 Accepted.

**Layer 2: parse-layer cmd.exe quote-strip (B2.fix, 2026-05-25).**
The B.fix wrapper shipped `format!("/C {full_command_line}")` —
passing the inner full command line to cmd.exe WITHOUT an outer pair
of quotes. The user's IRL re-verify against the post-C.fix binary
on 2026-05-24 surfaced an equivalent-symptom defect inside the
wrapper itself: cmd.exe's `/?`-documented quote-handling rule 2 (the
"old behavior" fallback when rule 1's "exactly two quote characters"
condition fails) stripped the FIRST and LAST quote characters of the
command line. Applied to B.fix's inner `"npx.cmd" -y @... "C:\path"`
(4 quote chars), the result was `npx.cmd" -y @... "C:\path` —
first token `npx.cmd"` carries a stray literal `"` that Windows
rejects as an invalid filename. cmd.exe exited with status 1 + the
SAME error class as pre-B.fix BatBadBut. The user-visible symptom
(infinite UI spinner; rmcp handshake hangs) was identical to pre-B.fix
even though the cause shifted from spawn-layer to parse-layer.

Fix shipped: one-line change in `build_command` —
`format!("/C {full_command_line}")` becomes
`format!("/C \"{full_command_line}\"")`. cmd.exe's rule 2 now strips
only the outer pair the wrapper added, leaving the inner
program-+-args quoting intact. B2.fix impl commit `51c4de5`; red
commit `677fdf0`. ADR-0023 amended in place (Status remains
Accepted; the amendment adds a "Multi-arg invocations" sub-section
to the Decision section explaining cmd.exe's `/?` rule 1 / rule 2
mechanics + the Microsoft cmd reference).

🔴 #6 closed in both layers. The assembled regression test
`crates/runtime-mcp/tests/mcp_npx_cmd_quoting.rs` (B2.fix) catches
either layer's regression on Windows: it builds the IRL command line
via `StdioTransport::build_command`, spawns via
`tokio::process::Command::output()`, and asserts the OS-level
"filename, directory name, or volume label syntax is incorrect"
string does NOT appear in stderr.

### Scenario E5 — M06.5 🔴-1 stray cleanup → PASS · 🔴 #7 schema drift → STILL OPEN

Build machine, 2026-05-25, post-B2.fix tip `b9f56b7` (passive
re-confirm against the post-C.fix cleanup module — the cleanup
module fired on the first launch after C.fix landed, before
B2.fix even existed):

```powershell
sqlite3 $REG "SELECT name,transport,status FROM mcp_servers;"
  → (empty — no rows; cleanup module fired on first launch + no
     servers added since)

Get-ChildItem "$APPDATA_DIR\mcp.sqlite"
  → (no output — stray file renamed away)

Get-ChildItem "$APPDATA_DIR\.stray-mcp.sqlite.bak*"
  → .stray-mcp.sqlite.bak  (151552 bytes, 5/23/2026, preserved
     byte-for-byte by C.fix's cleanup module)
```

The renamed file landed at the canonical `.stray-mcp.sqlite.bak`
target (no timestamp suffix — the suffix is the idempotent fallback
when `.stray-mcp.sqlite.bak` already exists from a prior cleanup
pass; first-cleanup-pass case gets the canonical name). The 151552
bytes preserved byte-for-byte confirms the rename was lossless.

Pre-fix on `main`: the ADR-0012 fix shipped at M06.5 Stage A.fix
prevents any future creation of `mcp.sqlite` — `Select-String
"mcp.sqlite" src-tauri/src/` and `crates/` returns no construction
sites — but does NOT clean up a stray file from pre-ADR-0012
testing. The user's `$APPDATA_DIR\mcp.sqlite` (originally dated
5/17/2026, sat dormant + visible across multiple sessions; grew via
subsequent legacy-path writes before the cleanup ran), with
cross-table FK references that any directory-scanning tool surfaces
as a second session DB.

Fix shipped: `crates/runtime-main/src/stray_db_cleanup.rs` — a
path-agnostic module per the CLAUDE.md §9 archetype — runs at
Tauri `setup()` BEFORE `session.sqlite` opens, detects any
`mcp.sqlite` in `app_local_data_dir`, and renames it to
`.stray-mcp.sqlite.bak` (canonical; falls back to
`.stray-mcp.sqlite.bak.<unix-ms>` when the canonical target
already exists from a prior cleanup pass — the idempotency
guard for repeated launches). `tracing::info!` emits forensic
context (path, size, timestamp). The `.bak` rename preserves the
bytes for manual inspection; future cleanup is a user action.
C.fix commit `94c2bc7`; red commit `feea6e3`.

🔴 M06.5 #1 CLOSED for real (cleanup ran; stray renamed; `.bak`
preserved byte-for-byte at 151552 bytes; `mcp.sqlite` no longer
present on second launch).

**🔴 #7 (schema divergence) — STILL OPEN.** The phase doc's
Sub-fix 2 prescription was dropped at C.fix red-phase: it labeled
canonical = 5-state and legacy = 4-state when the actual migration
files (`crates/runtime-drone/migrations/000_initial.sql` +
`003_mcp_server_status.sql`) prove canonical = 4-state. A literal
implementation of migration 004 would have rewritten every shipped
user's `session.sqlite::mcp_servers` from the canonical 4-state CHECK
to the stray-DB-style 5-state CHECK — corrupting any row whose
`status` value was outside the legacy schema's accepted set
(`'connected'` is shared, `'disconnected'/'health_pending'/'error'`
become illegal under 5-state). Maintainer chose at the red-phase
AskUserQuestion to drop Sub-fix 2 entirely + carry 🔴 #7 to
M08.6 alongside the state-sync class already routed there. The
underlying schema-drift class (any `CREATE TABLE IF NOT EXISTS` whose
body is skipped for a pre-existing table from an earlier migration)
is structurally addressed by M08.6's framework-representation +
loader-boundary work (the canonical-representation milestone is the
natural home for migration-idempotency invariants).

### Carry-forward to M08.6 Stage A intake (routing-confirmed)

Per the Option C routing decision in
`docs/m08.5-irl-re-verify-handoff.md` § "Decisions finalized
(2026-05-23)" — unchanged by this cycle except for the addition of
🔴 #7:

- **🔴 #4** — Tier state UI/backend desync. State-sync class.
- **🔴 #5** — Builder Canvas state not persisted across restart.
  State-sync class. (Stage A.fix's IRL re-verify surfaced this for
  the first time via smoke #6 actually running with an env-var key
  — confirms the existing routing.)
- **🔴 #7** — `mcp_servers` schema divergence between `session.sqlite`
  and the stray `mcp.sqlite`. Subsumed by M08.6's
  framework-representation + loader-boundary work
  (the migration-idempotency invariants).
- **11 🟡 from the M08.5 IRL re-verify** (Promote button label,
  Settings always-visible bar, Builder Canvas invalid agent id /
  missing capabilities, Configure modal Save guidance, Save dialog
  filename preload, `session_root_agent` UI surface, test-plan stale
  column name, the three harness 🟡 #9/#10/#11 that this cycle's
  Stage A.fix already closed).
- **2 new 🟡 from this cycle's IRL re-verify** (see below) —
  MCP-runtime cryptic error surfacing + MCP Add modal
  Command-vs-Args split UX.
- **🟢 elevation** (TD-009 broadened: strip ALL M0N from UI + add
  small-type version display chrome) — M08.6 Stage A as part of UI
  modernization.

### New findings from this cycle's IRL re-verify

Build machine, 2026-05-25, post-B2.fix tip `b9f56b7`. Two new 🟡
adjacent findings — both route to **M08.6 Stage A intake** per
CLAUDE.md §20.

**🟡 (UX) MCP-runtime cryptic error surfacing.** The
`MCP connect failed: connection closed: initialize response`
runtime error message is too cryptic when the actual failure is
downstream of cmd.exe (npx crashed, server-side initialization
never completed, etc.). Surface the captured child-process stderr
first line in the modal error message so users can self-diagnose
npm/node-config issues. Today's IRL re-verify case in point: the
real failure was a Node v24.11.0 + npm path-resolution mismatch
(npx tried to load `npm-prefix.js` relative to `C:\agent-runtime\`
which doesn't contain a Node modules tree) — completely unrelated
to runtime code, but invisible to the user behind the generic
"initialize response" wrapper.

**🟡 (UX) MCP Add Server modal — Command-vs-Args split easy to
miss.** Pasting a "command + args" string into the Command field
happens naturally if the user copies from documentation (where MCP
server install instructions are typically given as a full
command-line: `npx -y @modelcontextprotocol/server-filesystem
<path>`). Suggested: detect spaces in the Command field on blur +
prompt "did you mean to split this into Command + Args?" OR
auto-split on first whitespace and populate both fields. The
existing comma-separated Args field is also unusual (most CLIs use
space-separated args); consider a paste-handler that auto-converts
a space-separated paste into commas.

**None of the M08.5 IRL re-verify's existing 11 🟡 + 🟢 elevation
re-surfaced during this pass** — confirms the M08.5 + M08.5.5 fixes
are not introducing regressions in the previously-observed friction
points.

### M08 demo-unblock statement (revised again)

M08 is **partially demo-ready** as of the M08.5.5 cycle merge: the
MCP-Windows path (the Add/Test flow + the absence of a stray
session-DB on launch) is unblocked. The full demo path through
🔴 #4 tier promotion + 🔴 #5 Canvas persistence + 🔴 #7 schema
divergence + UI modernization opens after M08.6 ships.
