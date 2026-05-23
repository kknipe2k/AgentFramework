# Protocol revisions for IRL fail-rate reduction

Companion to `docs/m08.5-irl-re-verify-handoff.md`. Codifies three process
changes to reduce the IRL re-verify pass's high adjacent-bug-discovery rate
(M08.5 closed its 3 scoped 🔴 cleanly but the IRL pass surfaced 4 new 🔴 +
8 new 🟡 — symptomatic of the cycle starting from a frozen 🔴 list and
diagnose-steps falling back to defensive fixes when the build machine can't
run the real app).

---

## Problem statement (root cause of the fail rate)

Three structural gaps drove the M08.5 IRL re-verify outcome:

1. **The build machine cannot run the real app.** `tauri-driver` +
   `msedgedriver` are not installed; the build authors real-app tests but
   relies on CI to verify them. Stage prompts' `diagnose_root_cause` step
   degrades to "phase-doc defensive fallback" instead of live-DOM
   inspection. The D.fix MCP modal triple-fix shipped — but the build
   couldn't ever observe the actual cause, only ship a defensive guess.
2. **The 🔴 list is frozen at the parent milestone's IRL pass.** Adjacent
   bugs in the same workflow surface as IRL re-verify "surprises" because
   nothing in the cycle exercises the full user path. Example: M08.5
   closed 🔴-3 (modal buttons respond) but didn't surface that Test/Add
   then infinite-loop on Windows because of an `npx` spawn bug. That's not
   a 🔴-3 regression — it's an adjacent bug invisible to the cycle.
3. **No assembled-user-workflow CI gate.** Every Component (Builder,
   Tester, MCP modal, Tier, Settings) has unit / assembled-Rust /
   tauri-driver tests in isolation. The FULL chain (Canvas-build →
   Tester-run → Promote → MCP-Add → tool-call) has no single gate. Bugs
   that emerge only at the seams between these stages (e.g., Builder
   generates schema-invalid JSON → Tester schema-rejects with opaque
   error) escape until IRL.

Each gap is independently fixable; all three together raise the IRL pass
from "discovery event" to "confirmation event."

---

## Revision 1 — Install tauri-driver on the build machine (one-time)

**Scope.** One-time tooling install on the Windows build machine.
**Effort.** ~1-2 h. Mostly waiting for `cargo install tauri-driver` and
verifying `msedgedriver` version-match with the runner's Edge.

**Why this matters most.** Today the build's `diagnose_root_cause` step
falls back to defensive fixes whenever live-DOM inspection isn't possible.
With tauri-driver local:

- `diagnose_root_cause` becomes a real live-DOM inspection
  (`elementFromPoint` at the dead button, computed styles, viewport check)
  — phase doc D.3.2's "defensive-triple fallback" path becomes a last
  resort, not the default.
- The strict-TDD red-phase test can be EXECUTED locally to capture
  right-reason failure output, not just predicted. C.fix already did this
  (pure Rust); B.fix and D.fix couldn't because they're tauri-driver. With
  local tooling, all three could.
- Stage 0 (Revision 2) becomes feasible — the build can run the app on
  its own machine and discover adjacent bugs in 15 min, not in an
  hour-long user-driven IRL pass.

**Steps.**
1. `cargo install tauri-driver --version 0.1.4` (or current per the
   tauri-driver releases on crates.io).
2. Install `msedgedriver` version-matched to the build machine's Edge
   version. Cf. <https://developer.microsoft.com/en-us/microsoft-edge/tools/webdriver/>.
3. Verify: `npm run test:e2e:tauri` runs locally and the existing 6
   smoke + 2 regression tests pass + report a row.
4. Update `docs/gotchas.md` #23 (macOS not supported) to note Linux/Windows
   build-machine setup is the norm.

**Documentation changes.**
- `STAGE-PROMPT-PROTOCOL.md` §10 (quality-gate ordering): tauri-driver
  gate becomes a build-machine required gate when the diff touches
  `tests/e2e-tauri/`, `src/**`, `src-tauri/**`, or
  `src/components/**`. Currently CI-only.
- `CLAUDE.md` §6 "E2E gates": add `npm run test:e2e:tauri` to the local
  gate list once tooling is installed.

---

## Revision 2 — Stage 0 "real-app discovery" pre-flight

**Scope.** New protocol step inserted before strict-TDD red-phase on every
X.5 fix cycle.

**Effort per cycle.** 15-30 min build time. Net reduction in IRL re-verify
fail rate justifies this.

**What it is.** Before strict-TDD red-phase on each 🔴, the build (or user)
walks the FULL user workflow the 🔴 lives in — clicking through real UI in
the running app, capturing every adjacent bug encountered. The 🔴 list
grows accordingly. Each adjacent bug gets a triage call:

- Same severity / blocks the same user value → ADD to this cycle's 🔴
  list.
- Lower severity / unrelated → log as 🟡 and route per protocol.
- Pre-existing / not introduced by this cycle's scope → log + carry to
  next milestone's intake.

**What it would have caught in M08.5.**
- 🔴-3 (modal): adjacent "Test/Add infinite-loop on Windows" → ADD to
  D.fix scope.
- M06.5 🔴-1: adjacent "session.sqlite mcp_servers empty + stray
  mcp.sqlite still present" → confirms still-open status BEFORE the E.fix
  amend assumed RESOLVED.
- Builder Canvas: adjacent "generates schema-invalid agents (id format +
  missing capabilities)" → would have gated C.fix's framework fixture or
  surfaced as a new 🔴 to fix in cycle.

**Documentation changes.**
- `STAGE-PROMPT-PROTOCOL.md` §9 (`<execution_steps>`): add
  `real_app_discovery` as a new step BEFORE `diagnose_root_cause`. Its
  output: a `<discovery_log>` block listing observed adjacent bugs with
  per-bug triage.
- `CLAUDE.md` §6 "Quality gates": add "Real-app discovery pre-flight" as
  a non-skippable step on X.5 fix cycles touching user-visible surfaces.
- `docs/build-prompts/M08.5-irl-fix.md` template (the phase doc pattern):
  add a Stage 0 stanza for future X.5 fix cycles.
- `docs/M0N-irl-test-plan.md` patterns: when an X.5 cycle is dispatched,
  Stage 0's discovery log is a NEW SECTION appended to the IRL test plan
  (so the discovery isn't lost / becomes the authoritative pre-fix
  baseline).

---

## Revision 3 — One end-to-end Canvas→Tester→MCP CI gate

**Scope.** A single new `tests/e2e-tauri/full_workflow.e2e.ts` test that
exercises the entire user workflow as one chain.

**Effort.** 1 stage of build work (~3-4h). Depends on Revision 1 (the
build machine running tauri-driver locally for diagnose) and Revision 2
(having stable Stage-0 discovery to define the chain).

**The chain.**
1. App launches on Builder mode.
2. Drag an Agent from Palette to Canvas.
3. Configure agent (role + model).
4. Inspector → Save framework to `$SCRATCH`.
5. Inspector → Load framework from `$SCRATCH`.
6. Inspector → Test → enter task → Run → assert root-agent label is the
   candidate framework's role (not "smoke") AND result is PASS.
7. Settings → Promote → assert tier display updates to Promoted.
8. Settings → MCP Servers → Add (npx + filesystem args) → assert no
   infinite loop, server appears in list, row in session.sqlite (NOT in
   stray mcp.sqlite).
9. Optional: invoke a tool from the added MCP server → assert result.

If ANY step fails, the test fails. This catches bugs at the SEAMS between
components.

**What it would catch.** All four new 🔴 from the M08.5 IRL pass:
- 🔴 #4 tier UI desync → step 7 fails (UI shows wrong tier after Promote).
- 🔴 #5 Canvas state lost on restart → not directly caught (this test
  doesn't restart the app — separate test needed).
- 🔴 #6 MCP Add infinite-loop on Windows → step 8 fails (timeout).
- 🔴 #7 / M06.5 🔴-1 schema drift + stray mcp.sqlite → step 8 fails (row
  not in session.sqlite OR stray file appears).

**Documentation changes.**
- New file: `tests/e2e-tauri/full_workflow.e2e.ts`.
- `.github/workflows/ci.yml` `e2e-tauri-driver` job: add the new test (it
  ships with the existing 8 tests already gated).
- `CLAUDE.md` §6 "E2E gates": call out the full-workflow test as the
  "headline integration gate."
- `docs/coverage-policy.md` §C: log the new gate as a milestone entry.

---

## Implementation order (recommended)

**Order is dependency-driven, not priority-driven.**

1. **Revision 1 first** (one-time tooling install). Unblocks Revisions 2
   and 3 — they both depend on the build machine running the real app
   locally.
2. **Revision 2 second** (Stage 0 protocol change). Codifies the
   discovery step that Revision 1 enables. Should land before the next
   X.5 fix cycle so the new pattern is the default.
3. **Revision 3 third** (end-to-end CI gate). Best authored ONCE the
   user workflow is stable post-M08.5.5 / M08.6 — building the test
   against a moving workflow is wasted effort.

Total effort to land all three: ~2 days build time + protocol-doc
authoring. ROI: IRL re-verify shifts from discovery event (the
fail-rate-driving pattern) to confirmation event.

---

## Pairs with the post-M08.6 critical-review checklist

`docs/post-m08.6-critical-review.md` lists 4 IRL test items the
architectural audit raised. Three of those four (event-pipeline
reconciliation under drone failure; MCP timeout/heartbeat audit; sandbox
error → GapNode end-to-end) are natural Revision-3 chains. The full
end-to-end gate's chain can extend to include the architectural-audit IRL
items, making one CI test gate the single audit-trail for "does the
assembled app actually work?"

The post-M08.6 critical review and these three protocol revisions are
complementary, not competing.

---

## When to land these

- **Revisions 1 + 2**: BEFORE the next X.5 fix cycle dispatches (whether
  that's M08.5.5 per the routing decision or a future M0X.5). Otherwise
  the next cycle inherits the same fail-rate-driving gaps.
- **Revision 3**: AFTER M08.6 ships (the user workflow stabilizes post-
  framework-representation work). Building the gate against M08.5's
  workflow when M08.6 will reshape it is premature.

Both windows are short — Revisions 1+2 are pre-cycle authoring, Revision
3 is a 1-stage build task post-M08.6.
