# ADR-0021: The real-app (tauri-driver) regression gate

**Status:** Accepted
**Date:** 2026-05-22
**Deciders:** @kknipe2k
**Tags:** ci, testing, quality-gate, builder, process

## Context

The post-M08 IRL pass (`docs/M08-irl-findings.md`) — a manual walk-through
of `docs/M08-irl-test-plan.md` against the real Tauri desktop app on
Windows — found **three 🔴 defects** in the merged M08 Workbench:

1. Palette → canvas drag-to-instantiate is dead (Tauri's native
   drag-drop handler swallows HTML5 DnD).
2. The Tester emits the candidate framework's root agent as `"smoke"`
   (a hardcoded `agent_name`), so the run reads as a smoke session.
3. The "Add MCP Server" modal renders but Test / Add / Cancel are all
   non-responsive.

Every M08 quality gate was green and the M08 Verifier (Stage V) returned
**Sound, 0🔴**. All three defects still shipped. The single structural
reason: **no automated gate, and not the Verifier, exercises the real
Tauri desktop app.**

- The `e2e` CI job runs Playwright against the Vite dev server with
  `@tauri-apps/api` module-mocked, in a plain Chromium — it cannot see a
  Tauri-shell behavior (native drag-drop, WebView2 event handling, the
  real window).
- The `e2e-tauri-driver` CI job — which *would* drive the built Tauri
  binary via `tauri-driver` + WebdriverIO — has been `if: false` since
  M03 PR #47. It was disabled after a WebdriverIO-v9 ↔ tauri-driver-2.x
  capability-matching escalation (CLAUDE.md §7 budget), "deferred to
  M04," and never revived through M04–M08. `wdio.conf.ts` and
  `tests/e2e-tauri/smoke.e2e.ts` stayed in the tree but nothing runs
  them (gotcha #54).

ADR-0011 ("m06f-scope-seam-not-running-app") drew the seam-vs-running-app
testing boundary deliberately: M06f tested the seam, not the assembled
running app. The v1.8 assembled-app-regression mandate (CLAUDE.md §6)
then required each fix's regression test to exercise the assembled
*composition*. But "assembled composition" was satisfied at the Rust
integration-test layer (`crates/runtime-main/tests/`) — which still
cannot see a renderer/Tauri-shell defect. The 🔴-escape class is the
**renderer + Tauri-shell behavior that only manifests in the real
desktop window.** This ADR closes that side of the boundary.

## Decision

**The `e2e-tauri-driver` CI job becomes a required, blocking quality
gate from M08.5 onward.** It builds the Tauri binary and drives it
through `tauri-driver` + WebdriverIO on Linux and Windows.

- **The job is un-disabled.** The `if: false` guard is removed; the job
  runs on every PR and `main` push, and its failure blocks merge — the
  same standing as the `e2e` (Playwright) job.
- **Platform scope: Windows + Linux.** macOS stays excluded —
  `tauri-driver` upstream has no WKWebView driver (gotcha #23). macOS
  renderer behavior is covered by the Playwright `e2e` job. v0.1 is a
  Windows preview (§0d), so Windows + Linux is the gate that matters.
- **The wdio / tauri-driver version combination is pinned to whatever
  clears Windows + Linux CI green**, decided at M08.5 Stage A and
  recorded in the Stage A retrospective. The decision is a cross-stack
  integration call (gotcha #32): the config is quoted verbatim from a
  verified-working upstream reference, never hand-authored. The
  WebdriverIO-v9 ↔ `tauri-driver` matrix is unstable (the only
  community example confirmed green on a Windows runner pins
  WebdriverIO v7); Stage A evaluates, in order, the official
  `tauri-driver` path, the `@wdio/tauri-service` / `@crabnebula/tauri-driver`
  modern paths, and a WebdriverIO-v7 pin, and keeps whichever is green.
- **Every renderer-or-Tauri-shell-behavior 🔴 fix from M08.5 onward
  lands a real-app regression test in `tests/e2e-tauri/`.** A Rust
  `crates/**/tests/` integration test satisfies the assembled-app
  mandate for backend composition; it does **not** satisfy it for a
  renderer/shell behavior — that requires a `tauri-driver` test.

## Consequences

### Positive

- The 🔴-escape class is closed: a defect that only manifests in the
  real desktop window now has a gate that can fail on it. The three
  M08-IRL 🔴 each get a `tests/e2e-tauri/` regression test that fails on
  pre-fix `main`.
- The M03 PR #47 carry-forward (`e2e-tauri-driver` deferred; gotcha #54)
  is discharged.
- The v1.8 assembled-app-regression mandate gains a real renderer/shell
  surface — "assembled running app" can now mean the literal running
  Tauri app, not only the Rust composition layer.

### Negative

- CI time and cost grow. The job builds the app in release mode on two
  OSes; the `smoke.e2e.ts` happy-path test makes a real low-budget
  Anthropic call (~$0.002 per PR run, the existing `ANTHROPIC_TEST_KEY`
  secret). This is the cost of a real-app gate; it is bounded and
  accepted.
- The wdio / tauri-driver version matrix needs maintenance — a known
  cross-stack-fragility surface (the reason the job was disabled at
  M03). The version pin + its upstream reference are recorded so a
  future break is diagnosable.

### Neutral / future implications

- macOS desktop-shell behavior remains uncovered by a real-app gate.
  The Playwright `e2e` job runs renderer logic cross-platform, and v0.1
  does not target macOS. When macOS becomes a v1.0 target, a
  macOS-capable driver (`@crabnebula/tauri-driver`, or the community
  WKWebView driver) supersedes this exclusion — a future ADR.
- The gate is a desktop-shell *behavior* check, not a coverage gate; it
  is not in the `cargo llvm-cov` / `vitest --coverage` measurement and
  does not change any coverage threshold.

## Alternatives Considered

### Alternative A: Keep Playwright-only; do not revive tauri-driver

**Rejected because:** Playwright mocks `@tauri-apps/api` and runs in a
plain browser — it is structurally blind to exactly the defect class
that escaped (native drag-drop, WebView2 behavior, the real window).
Keeping Playwright-only leaves the 🔴-escape gap open; the next
renderer/shell 🔴 would escape identically.

### Alternative B: Make `@crabnebula/tauri-driver` (macOS-capable) the primary driver now

**Considered, deferred to v1.0:** the `@crabnebula/tauri-driver` fork
and the community Choochmeque cross-platform WebDriver add macOS support.
v0.1 is Windows-only (§0d) with CI on Windows + Linux; the official
`tauri-driver` Windows+Linux path is sufficient and is the
better-documented reference. Stage A may still adopt `@crabnebula/tauri-driver`
if it is the combination that clears CI green — that is a Stage A
integration call, not an architecture commitment. macOS coverage is a
v1.0 decision.

### Alternative C: Run the real-app job non-blocking (advisory)

**Rejected because:** an advisory job that does not block merge is what
the disabled `e2e-tauri-driver` job effectively was — present in the
tree, not enforced. The 🔴-escape happened *because* nothing enforced
the real-app surface. A gate that does not gate is not a gate.

## Related

- IRL findings: `docs/M08-irl-findings.md` (the 3🔴 this gate would have
  caught)
- Build prompt: `docs/build-prompts/M08.5-irl-fix.md`
- Prior ADRs: ADR-0011 (m06f scope — seam, not running app; this ADR
  closes the running-app side), ADR-0008 (Stage V verifier — which this
  gate complements: V is fresh-context contract-fidelity, this gate is
  real-app behavior)
- Gotchas: #23 (tauri-driver, not Playwright `_electron`; macOS
  unsupported), #32 (cross-stack integration examples quoted verbatim
  from a working upstream reference), #54 (the `e2e-tauri-driver`-disabled
  carry-forward), #70 (viewport / window behavior not caught by unit
  tests)
- Spec / MVP: MVP §M8 (the Workbench acceptance criteria the IRL pass
  exercised)

## Notes

This ADR is filed at M08.5 Stage A (the stage that revives the gate) and
flips `Proposed → Accepted` in the Stage A impl commit — the
M06.5.A.fix / ADR-0012 precedent (the stage that implements an ADR flips
it). CLAUDE.md §6 (the gate lists) and the §"E2E gates" section are
reconciled to reflect the now-active gate in M08.5 Stage E.

### Resolved version matrix (M08.5 Stage A.fix)

The WebdriverIO ↔ `tauri-driver` combination kept is the repo's existing
stack — **WebdriverIO 9.27.1** (the version `package-lock.json` already
pins; `package.json` declares `^9.0.0`, and the official Tauri
WebdriverIO example verifies `^9.19.0`) + **`tauri-driver` latest via
`cargo install tauri-driver --locked`** (2.0.6 as of 2026-05). No version
downgrade, no WebdriverIO-v7 pin, and no `@wdio/tauri-service` /
`@crabnebula/tauri-driver` fallback were needed; the A.3.2 decision
procedure's Primary path cleared.

The M03 PR #47 disable attributed the failure to "wdio v9 ↔ tauri-driver
2.x compat unresolved." Stage A.fix re-diagnosed it (grep-verified
against the codebase) to **two concrete config bugs, neither
version-related**:

1. **Linux** (`could not exec the app binary`) — `wdio.conf.ts` pointed
   `APP_BIN_PATH` at `src-tauri/target/release/`, but `src-tauri` is a
   member of the Cargo workspace rooted at the repo root, so `cargo` /
   `tauri build` emit the binary to the shared workspace-root
   `target/release/`. Fixed in `wdio.conf.ts`.
2. **Windows** (`msedgedriver not on PATH`) — the CI job had no
   msedgedriver setup; `tauri-driver` requires `msedgedriver.exe` on
   `PATH`, version-matched to the runner's Edge WebView2. Fixed by adding
   the official `msedgedriver-tool` step to the `e2e-tauri-driver` job.

The existing `wdio.conf.ts` capabilities object (`browserName` omitted,
`tauri:options.application`) was already the correct official shape — the
M03 capability-string iteration churn ('edge'/'webkit2gtk' → 'wry' →
omit) had been chasing the wrong cause.

**Upstream references** (gotcha #32 — cross-stack config quoted verbatim,
never hand-authored): the official Tauri 2.x WebDriver docs,
`tauri-apps/tauri-docs` @ `v2` branch, commit
`a9d8348ff518e4a052bcd7435c07e34a9dfe1af1` —
`src/content/docs/develop/Tests/WebDriver/ci.md` (the Windows
`msedgedriver-tool` step, quoted verbatim into `ci.yml`) and
`src/content/docs/develop/Tests/WebDriver/Example/webdriverio.mdx` (the
WebdriverIO config shape, which the existing `wdio.conf.ts` already
matched).

**Branch protection:** the repo's `main` branch carries no GitHub branch
protection at all, so neither the `e2e` (Playwright) job nor any other
job is a branch-protection "required check." "The same standing as the
`e2e` job" (Decision, above) is therefore satisfied by the job running
unconditionally on every PR and `main` push and being non-advisory (no
`continue-on-error`) — which un-disabling it achieves. If the maintainer
later enables branch protection on `main`, `e2e-tauri-driver` should be
added to the required-checks list alongside `e2e` and the Rust gates;
that is a repo-settings action, not a workflow-file change.
