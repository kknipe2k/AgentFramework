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

- [ ] Findings reviewed; the M08.5 fix-cycle scoped (3 🔴).
- [ ] M08 is **not demo-ready** until the 🔴 fix-cycle lands and a re-run of `docs/M08-irl-test-plan.md` is clean.
