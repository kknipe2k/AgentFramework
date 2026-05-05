# M03 Live Graph — Specification + Stage Prompts

**Protocol version:** v1.2 (first milestone authored on the XML stage-prompt schema per `STAGE-PROMPT-PROTOCOL.md`).
**Date:** 2026-05-05
**Status:** Design approved — implement one stage at a time, in order
**Scope:** Bring the live graph alive. Replace the M02 skeleton renderer's `<ul>` event list with a React Flow + Zustand graph that renders `AgentEvent`s as 11 typed nodes with animated edges, click-to-inspect side panel, token-spend visualization, VDR projection, graph persistence, and Tauri 2.x desktop-shell E2E (tauri-driver + WebdriverIO; Linux + Windows matrix). End state: user clicks "Run smoke test", live graph renders nodes spawning + edges animating in real time; user clicks any node, side panel shows full payload + correlated VDR row; reload preserves the graph from SQLite. Spec §3 + §M3 acceptance criteria.

---

## Background and Design Decision

**Problem.** M02 shipped the event pipeline live: Anthropic SSE → ProviderEvent → AgentEvent → Tauri IPC → React renderer. The renderer is a flat `<ul>` event list — useful for proving the pipeline works, useless as the product's face. Spec §3 declares the live graph as the user-facing surface: "The graph is the product's face. It renders the full agentic runtime as it happens. Every spawned agent is a node. Every skill invocation is an edge. Every gap is visible." M03 builds it.

**Solution.** Six stages on one feature branch (`claude/m03-live-graph`), each a fresh Claude Code session per the v1.2 XML stage-prompt protocol. Stage A absorbs all M02 carry-forward 🟡 Important items (build hygiene + new deps + legacy-file deletes) so Stages B–F focus on the real M03 deliverables. Stage B lays the React Flow + Zustand foundation with three basic node types. Stage C lights up the remaining eight node types + animated edges + color encoding. Stage D adds the click-to-inspect side panel + token-spend node weight + zoom/pan controls. Stage E projects VDR from the signal stream + adds a SQL inspector + persists the graph to SQLite. Stage F lands the Tauri 2.x desktop-shell E2E suite (tauri-driver + WebdriverIO; Linux + Windows matrix) and runs the Phase Closeout per CLAUDE.md §20.

**Why one PR for the parent milestone.** Same as M01 + M02 — stages-as-commits-on-one-branch gives incremental discipline (each stage is reviewable; each stage retrospective surfaces friction early) without the overhead of six PR reviews for one logical milestone. Consistent with the per-milestone-as-PR pattern in `docs/build-prompts/README.md`.

**Why six stages, not fewer.** React Flow + 11 node types + animated edges + inspector + token viz + VDR + persistence + E2E is genuinely more surface area than M02's event pipeline. Calibrated estimate: ~25–31h actual (~2× M02). Splitting into six stages keeps each in the 4–6h range per CLAUDE.md §5 single-session budget. Stage F bundles the E2E suite + Phase Closeout because the closeout is doc-only (gap-analysis entry + summary) and fits comfortably alongside the E2E build.

**Why first milestone on v1.2 protocol.** M01 + M02 are v1.0 grandfathered (per `STAGE-PROMPT-PROTOCOL.md` v1.2 changelog item #8). M03 is the first milestone where the XML schema is mandatory. Each stage's `X.5 CLI Prompt` is a fenced ```xml block with `<work_stage_prompt id="M03.X">` (or `<closeout_stage_prompt id="M03.F">` for Stage F) — strict reference-first per Authoring Rules §10; mandatory `<execution_steps>` slot; section-name refs (no URI fragments); closeout-stage `<gotchas_graduation>` in the gap-analysis subsection.

**Key constraints.**
- §0d Release Scope Matrix — M03 is in scope. Out-of-scope items (plan model + HITL + budget data wiring → M4; MCP node connecting to a real server → M6; macOS Tauri-shell E2E → unsupported by tauri-driver, deferred indefinitely) stay deferred.
- React + React Flow + Zustand for state; no Redux, no MobX (per MVP §M3 acceptance criteria).
- Renderer Vitest coverage ≥80% on graph reducers (per MVP §M3 acceptance criteria + post-M02 vitest --coverage default).
- All M02 hard-gate inheritance — workspace ≥80%, runtime-drone ≥95%, runtime-main ≥95% with documented OS-call exclusions, frontend prettier+eslint+tsc strict + audit, codecov delta gates, gap-analysis append-only — none relaxed.
- UI consistency carry-forward (Pre-M01 addendum via M01 entry) — all M03 modals/screens reuse existing component patterns and visual language; no per-feature re-skinning. M02's SetupPanel + button styling is the baseline.

**License.** Apache 2.0; DCO sign-off (`git commit -s`) on every commit.

**Existing patterns to mirror.**
- M01 archetype: `crates/runtime-drone/src/snapshot.rs` + `db.rs` + `heartbeat.rs` + `command_handler.rs` (TDD-discipline + ≥95% coverage with documented OS-signal exclusions).
- M02 archetype: `crates/runtime-main/src/providers/anthropic_sse.rs` + `tests/anthropic_wiremock.rs` (`*_with` testable seam pattern + wire-format state machine + wiremock harness).
- M02 archetype: `crates/runtime-main/src/sdk/event_pipeline.rs` + `tests/sdk_event_translation.rs` (event-translation pipeline + bounded-stream test fixtures per `docs/gotchas.md` #28).
- M02 archetype: `src-tauri/src/commands.rs::set_api_key_with` + `run_smoke_session_with` (testable seam over Tauri command surface; `*_with` seam + wrapper over OS calls — matches the §13.5 Dev Logging instrumentation pattern).
- M02 architecture: `src/lib/ipc.ts::unwrapCmdError` (renderer-side typed error unwrap per `docs/gotchas.md` #30).

**Pre-existing legacy file inventory** (per `docs/build-prompts/TEMPLATE.md` post-M02 protocol-iteration addition).

The renderer tree under `src/` was created by M02 Stage E. M03 Stage A must enumerate every tracked-but-orphaned file in the tree and assign a disposition before code begins:

| File | Status | Disposition for M03 |
|---|---|---|
| `src/counter.js` | legacy CommonJS in `.prettierignore` + `eslint.config.js ignores` | **DELETE** in Stage A |
| `src/counter.test.js` | legacy CommonJS in `.prettierignore` + `eslint.config.js ignores` | **DELETE** in Stage A |
| `src/App.tsx` | M02 — composes `SetupPanel` + `SmokeButton` + `EventList` via `useReducer` | **REFACTOR** in Stage B (replace EventList with React Flow Canvas; keep SetupPanel + SmokeButton) |
| `src/main.tsx` | M02 — React 18 root | preserve |
| `src/styles.css` | M02 — minimal stylesheet | extend in Stage B (graph canvas styling); preserve M02 component styles |
| `src/types/agent_event.ts` | M02 — hand-mirrored from `runtime_core::AgentEvent` (M02.E surprise event 5; flagged as drift risk) | **REGENERATE** in Stage A from new `schemas/event.v1.json` via extended `cargo xtask regenerate-types` (per `CLAUDE.md` §14 schemas-as-source-of-truth) |
| `src/lib/eventReducer.ts` | M02 — `useReducer`-shape reducer over events | **DELETE** in Stage B (Zustand store replaces useReducer; keep tests refactored to test the Zustand store's actions) |
| `src/lib/ipc.ts` | M02 — typed Tauri invoke wrappers + `unwrapCmdError` helper | preserve + extend in Stage B (graph-related event subscription) |
| `src/components/EventList.tsx` | M02 — flat `<ul>` of events | **DELETE** in Stage B (React Flow Canvas replaces it) |
| `src/components/SetupPanel.tsx` | M02 — API-key input + save button | preserve as-is (M03 retains the same API-key onboarding) |
| `src/components/SmokeButton.tsx` | M02 — "Run smoke test" trigger | preserve; Stage D extends to disable during graph-active states |
| `tests/unit/eventReducer.test.ts` | M02 | **DELETE** in Stage B (eventReducer being deleted; Zustand store gets new test file) |
| `tests/unit/components.test.tsx` | M02 — covers EventList variants | **REFACTOR** in Stage B/C (drop EventList tests; keep SetupPanel + SmokeButton coverage; add graph-canvas + node-component tests) |
| `tests/unit/ipc.test.ts` | M02 | preserve + extend |
| `tests/unit/App.test.tsx` | M02 — App-level state-machine tests | **REFACTOR** in Stage B (state machine via Zustand; keep happy-path + error-path scenarios) |
| `tests/e2e/smoke.spec.ts` | M02 — 3 active Playwright + 4 `test.skip()` carry-forwards for full Tauri-shell E2E | **DELETE all 4 `test.skip()` entries** in Stage F (replaced by tauri-driver + WebdriverIO suite); preserve the 3 active renderer-level tests as M03 still wants Vite-dev-server smoke coverage |
| `tests/setup.ts` | M02 — Vitest setup | preserve |

No legacy from earlier milestones beyond the M02 tree.

---

## Document Structure

| Stage | Summary | Estimated effort |
|---|---|---|
| **A** | Build hygiene + carry-forward closures (delete `src/counter.{js,test.js}`, retrofit drone integration tests to `current_exe()`, add `event.v1.json` schema + xtask TS codegen, vitest --coverage default, Vite 5→7 bump, drop `secrecy/serde` feature, add `@xyflow/react` + `zustand` deps) | ~2–3h |
| **B** | React Flow + Zustand foundation; three basic node types (`AgentNode`, `ToolNode`, `SkillNode`); replace EventList with React Flow Canvas; Vitest tests for the graph store | ~5–6h |
| **C** | Remaining eight node types (`MCPNode`, `GapNode`, `HITLNode`, `PlanNode`, `TaskNode`, `VerifyNode`, `HookNode`, `FrameworkNode`) + animated edges (active call) + dashed edges (skill load) + spec §3 color encoding | ~5–6h |
| **D** | Click-to-inspect side panel + token-spend visualization (node weight) + zoom/pan/select controls | ~4–5h |
| **E** | VDR projection from signal stream + simple SQL inspector + graph persistence to SQLite + reload reconstruction | ~4–5h |
| **F** | Tauri 2.x desktop-shell E2E (`@crabnebula/tauri-driver` + WebdriverIO; Linux + Windows matrix; macOS unsupported per tauri-driver upstream) + 4+ E2E tests + Phase Closeout (gap-analysis entry per CLAUDE.md §20) | ~5–6h |

Total: ~25–31 hours estimated. ~10 hours human direction (six approval gates + one PR review).

**Estimation calibration.** M01: estimated 29–46h, ran ~9–14h (ratio 0.3×). M02: estimated 13h, ran ~8.8h (ratio 0.7×). The estimates have been getting tighter as authoring fidelity improves. M03's 25–31h estimate is conservative but reflects the new domain (React Flow, Zustand v5, tauri-driver), the surface area (11 node types + edges + inspector + persistence), and the v1.2 protocol's stricter authoring discipline (XML schema + WEBCHECK headers + execution_steps). If M03 actuals run to 0.7× the estimate (matching M02), expect ~17–22h actual.

---

## Implementation Workflow

Each stage runs through this exact cycle:

```
1. /clear                     — fresh context (only between stages)
2. Paste CLI Prompt below     — XML <work_stage_prompt> or <closeout_stage_prompt>
                                pasted into a fresh Claude Code session
3. WEBCHECK pass              — verify prompt's claims about API shapes /
                                version pins / best practices against the
                                URLs in the stage's WEBCHECK header before
                                writing code (per CLAUDE.md §12 web-first +
                                STAGE-PROMPT-PROTOCOL.md v1.2)
4. Read prior stage retros    — Stage B+ reads M03.A retro [END] Decisions
                                section, applies decisions BEFORE code
5. Write failing tests first  — per CLAUDE.md §5 TDD discipline
6. cargo test --workspace +   — confirm new tests fail before any production
   npm run test                 code (red phase)
7. implement                  — Claude makes production changes
8. cargo test --workspace +   — all tests green
   npm run test
9. cargo clippy + fmt + audit — zero warnings
   + npm run lint + tsc        + frontend gates
10. cargo llvm-cov + npm test — coverage thresholds met (workspace ≥80%,
    -- --coverage                runtime-drone ≥95%, runtime-main ≥95%,
                                src/ ≥80%)
11. fill in retrospective     — docs/build-prompts/retrospectives/M03.<X>-retrospective.md
                                including the new [END] Coverage holdouts
                                subsection (per RETROSPECTIVE-TEMPLATE.md
                                post-M02 addition)
12. commit (no push)          — exact commit message provided per stage X.6
13. user reviews + approves   — Claude does NOT push without approval
14. push (final stage only)   — Stage F push gates the M03 PR draft per
                                CLAUDE.md §20
```

**Rule:** If a new test passes before implementation, the test is wrong — stop and fix the test (CLAUDE.md §5 hard-fails on missing exports).

**Rule:** Stages are sequential. Stage B does not start until Stage A's commit is on the feature branch (locally is sufficient; push is optional). The parent-milestone PR pushes only at the end of Stage F.

**Rule per CLAUDE.md §8:** Claude does not commit without user approval. After tests pass + retrospective filled, Claude surfaces the diff stat + retrospective + draft commit message. User approves; Claude commits.

**Rule per CLAUDE.md §19:** Each stage produces a retrospective; the final stage also produces an `M03-summary.md` aggregating across stages.

**Rule per CLAUDE.md §20:** Stage F's gap-analysis entry is **immutable** once committed. Future milestones report status updates via their Carry-forward sections; never edit M03's entry after merge.

**Rule per spec §13.5 Dev Logging:** Every Rust binary modified in M03 keeps `tracing_subscriber::fmt::init()` at `main()`. Every Tauri command added in M03 logs entry / error / success. Every renderer `try { await invoke(...) } catch (e) { ... }` block logs `console.error('<context> error:', e)` before `unwrapCmdError(e)` dispatch.

---

<!-- ============================================================ -->
<!-- STAGE A — Build hygiene + carry-forward closures + new deps    -->
<!-- ============================================================ -->

## Stage A — Build hygiene + carry-forward closures + new deps

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens. Per CLAUDE.md §12 web-first rule + `docs/build-prompts/TEMPLATE.md` post-M02 addition. If any claim below is stale, update this section in `M03-live-graph.md` BEFORE pasting Stage A's CLI prompt to the fresh session — never let a fresh session work from a stale snapshot.

- <https://vite.dev/releases> — confirm Vite 7.x is the recommended bump target (Vite 8.0 ships Rolldown March 2026, fresh; M03 stays on Vite 7 for risk control, defers Vite 8 evaluation to M04+)
- <https://crates.io/crates/keyring> — confirm keyring 3.6.x is current 3.x; 4.0 has breaking API surface (M03 stays on 3.6, defers 4.0 to dedicated chore PR)
- <https://www.npmjs.com/package/@xyflow/react> — confirm `@xyflow/react` (renamed from `reactflow`) v12.10.x is current; React 18+ peer dep met
- <https://github.com/pmndrs/zustand/releases> — confirm Zustand v5.0.x is current; React 18+ peer dep met; review v4→v5 migration guide for breaking changes (`createWithEqualityFn` for custom equality, `useShallow` for stable refs, persist middleware behavior, setState replace flag strictness)
- <https://docs.rs/secrecy/latest/secrecy/> — confirm `serde` feature default behavior on 0.10.x: SecretString does NOT serialize via serde by design (security); `Deserialize` impl exists but is unused in M02 codebase. Drop the feature.
- <https://docs.rs/json-typegen/latest/json_typegen/> or <https://github.com/bcherny/json-schema-to-typescript> — confirm a stable Rust crate or Node CLI for emitting TypeScript types from JSON Schema, callable from the existing `crates/xtask` binary, that produces output equivalent to the current hand-mirrored `src/types/agent_event.ts`

### A.1 Problem Statement

M02 Stage E shipped the renderer skeleton. Three classes of carry-forward debt block M03's React Flow work:

1. **Tooling stack is stale or insecure.** Vite 5.4 has a documented esbuild CVE (CVE-2024-23334; arbitrary website can read dev-server responses); the `secrecy/serde` workspace feature is dead weight (SecretString won't serialize via serde even when the feature is on); `keyring 3.6` is fine for now but the 4.0 evaluation is a known carry-forward.
2. **TS types are hand-mirrored from Rust.** `src/types/agent_event.ts` was a pragmatic shortcut in M02 but `CLAUDE.md` §14 forbids hand-rolled types when a schema source-of-truth is available. The risk surfaced concretely in M02.D when `ToolSource` + `AgentSpawned.session_id` were added: the hand-mirrored TS could have silently drifted under any pressure. M03 takes events as schema input (UI consumes them); the schema must be canonical now.
3. **Test infrastructure has known traps.** `crates/runtime-drone/tests/integration*.rs` hard-codes `target/debug/runtime-drone.exe` paths that break under `cargo llvm-cov --workspace` (distinct target dir). Vitest's `--coverage` flag is configured but not run by default, so the 80% threshold is enforced only when someone explicitly passes the flag. Legacy `src/counter.{js,test.js}` files predate `"type": "module"` and are kept alive only by ignore-list entries — risk of reanimation if an ignore entry is dropped.

Stage A closes all three classes before Stage B starts touching React Flow. Net deliverable: a healthy build environment with Vite 7 + React Flow + Zustand deps installed, TS event types generated from a new `schemas/event.v1.json`, drone integration tests using `current_exe()`-derived paths, vitest defaulting to `--coverage`, and the legacy CommonJS files gone.

**One-line success criterion:** `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo xtask regenerate-types --check && npm ci && npm run lint && npm run test && npm audit --audit-level=high` passes locally on Windows + on CI Linux/macOS/Windows × stable + MSRV, with no leftover references to `src/counter.*` and no hand-mirrored TS types.

**New artifacts:**
- `schemas/event.v1.json` — canonical AgentEvent schema (10 variants matching `runtime_core::event::AgentEvent` v0.1; `ToolSource` enum; `AgentSpawned.session_id` field)
- Extended `crates/xtask/src/main.rs` — TS codegen step alongside the existing typify Rust codegen
- Optional: `package.json` `devDependencies` entry for `json-schema-to-typescript` if the codegen calls a Node CLI (decision in A.3)

### A.2 Files to Change

| File | Change |
|---|---|
| `Cargo.toml` (workspace) | **Edited** — bump Vite via package.json (no Cargo change needed for Vite); drop `secrecy = { version = "0.10", features = ["serde"] }` to plain `secrecy = "0.10"` (remove `serde` feature) |
| `package.json` | **Edited** — Vite 5.4.x → ^7.0; add `@xyflow/react ^12.10`; add `zustand ^5.0`; flip `"test": "vitest run"` → `"test": "vitest run --coverage"`; optionally add `json-schema-to-typescript` to devDeps |
| `package-lock.json` | **Regenerated** — `npm install` after package.json edits |
| `crates/xtask/src/main.rs` | **Edited** — add `regenerate_typescript_types()` function alongside the existing typify Rust codegen; emit to `src/types/agent_event.ts` (and any other schema-derived TS files); wire into `regenerate-types` and `regenerate-types --check` subcommands |
| `crates/xtask/Cargo.toml` | **Edited** — add deps for the chosen TS-codegen approach (e.g., `schemars` if generating via Rust, OR a `std::process::Command` shell-out if calling Node `json-schema-to-typescript`) |
| `schemas/event.v1.json` | **New** — canonical AgentEvent schema |
| `src/types/agent_event.ts` | **Regenerated** — replace hand-mirrored content with `regenerate-types` output; add `// GENERATED — do not edit by hand` header (per CLAUDE.md §14) |
| `src/counter.js` | **Delete** |
| `src/counter.test.js` | **Delete** |
| `.prettierignore` | **Edited** — remove `src/counter.*` lines now that the files are gone |
| `eslint.config.js` | **Edited** — remove `src/counter.*` from `ignores` array now that the files are gone |
| `crates/runtime-drone/tests/integration.rs` | **Edited** — replace hard-coded `target/debug/runtime-drone.exe` path derivation with `std::env::current_exe()`-based pattern (mirror `crates/runtime-main/tests/drone_ipc_loopback.rs::drone_binary` archetype per `docs/gotchas.md` #22) |
| `crates/runtime-drone/tests/integration_windows.rs` | **Edited** — same retrofit |
| `vitest.config.ts` | **Edited** (optional) — verify `coverage.thresholds.lines: 80` is honored when `--coverage` is the default; add `runtime` constraint if needed |
| `.github/workflows/ci.yml` | **Edited (small)** — verify the Frontend job's `npm run test` invocation matches the new default; no other change |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` entry under "Added — M03.A" |

### A.3 Detailed Changes

(Stage A's full Find/Replace blocks + new file content land here — TS codegen approach selected, per-file diffs captured, `current_exe()` retrofit shown, etc. Authored verbatim before the Stage A fresh session opens. Surfaced in the next chunk after this PR's Stage A frame is approved.)

> **Author note.** Stage A's A.3 body is intentionally elided in this initial chunk — the surface area is large (xtask TS codegen mechanism choice; per-test-file path retrofit; package.json + Cargo.toml diffs) and surfacing it as separate-file content is cleaner than inlining hundreds of lines. The Stage A chunk's second pass will populate A.3 with concrete content and a TS-codegen-mechanism decision (Rust `schemars` crate vs Node `json-schema-to-typescript` CLI shell-out via `std::process::Command`).

### A.4 Tests

#### Pedantic-pass preflight (for stages adding new modules)

Per `docs/build-prompts/TEMPLATE.md` post-M02 addition + `docs/gotchas.md` #21. Stage A adds `regenerate_typescript_types()` to `xtask` (new module); the preflight applies:

- [ ] `redundant_pub_crate` — used plain `pub` in private modules
- [ ] `derive_partial_eq_without_eq` — N/A (no new types)
- [ ] `unused_async` — N/A (synchronous codegen)
- [ ] `default_trait_access` — explicit type (`HashMap::default()` over `Default::default()`)
- [ ] `match_wildcard_for_single_variants` — explicit binding when single variant remains
- [ ] `cast_precision_loss` / `suboptimal_flops` — N/A
- [ ] `struct_excessive_bools` — N/A
- [ ] `missing_const_for_fn` — pure constructors marked `const fn`
- [ ] `unnecessary_literal_bound` / `doc_markdown` — code identifiers backticked

#### Default test plan for stages adding a new safety primitive

Stage A does not add a safety primitive; xtask is dev-only tooling, not runtime code. The default seam-test pattern still applies for the codegen path:

- 1 unit test for `regenerate_typescript_types_with(schemas: &[Schema], output_dir: &Path)` — testable seam covering the codegen logic without writing real files
- 1 integration test (`crates/xtask/tests/check_drift.rs` extension) that runs `regenerate-types --check` and asserts no drift between committed `src/types/agent_event.ts` and freshly-regenerated output

#### Test plan (Stage A)

- `crates/xtask/tests/check_drift.rs` — extended to cover TS codegen drift in addition to existing Rust-types drift
- `crates/runtime-drone/tests/integration.rs` + `integration_windows.rs` — retrofitted tests still pass; new `drone_binary()` helper using `current_exe()` is exercised
- `tests/unit/` (frontend) — no test change in Stage A (Stage B refactors the test set when EventList → React Flow Canvas)
- `npm run test` (with new default `--coverage`) — passes existing 25 tests; coverage report shows ≥80% on `src/`

#### Coverage target

- Workspace: ≥80% (preserved from M02; xtask is excluded per existing regex)
- runtime-drone: ≥95% (preserved per CLAUDE.md §5; integration test refactor must not regress per-module baselines: `snapshot.rs` 100%, `db.rs` 98.82%, `heartbeat.rs` 98.59%, `command_handler.rs` 97.94%, `ipc.rs` 84.70%)
- runtime-main: ≥95% (preserved with documented exclusions: `providers/anthropic.rs`, `drone_ipc/connection.rs`, `key_store.rs`)
- src-tauri: 50% patch gate (per codecov.yml `tauri-shell` gate from PR #45)
- src/ frontend: ≥80% (Vitest threshold; now triggered by default `--coverage` flag)

**Doc-to-CI invariant.** No new exclusions added in Stage A. If Stage A's xtask refactor reveals a new module that's structurally untestable on CI, update CI workflow regex + CLAUDE.md §5 + this stage's retrospective `[END] Coverage holdouts` subsection in the same commit (per `docs/build-prompts/TEMPLATE.md` post-M02 addition).

### A.5 CLI Prompt

Paste the XML block below into a fresh Claude Code session as the opening message. Per `STAGE-PROMPT-PROTOCOL.md` v1.2 — section-name refs, mandatory `<execution_steps>`, strict reference-first.

```xml
<work_stage_prompt id="M03.A">
  <context>
    Stage A of M03 (Live Graph). Build hygiene + carry-forward closures from
    M02 + new deps for Stages B–F. Absorbs all M02 🟡 Important items so
    Stages B–F focus on the real M03 deliverables (React Flow, node types,
    inspector, VDR, persistence, Tauri E2E). Stage B does not start until
    Stage A's commit is on the milestone branch claude/m03-live-graph.
    First milestone authored on the v1.2 XML stage-prompt protocol.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M03-live-graph.md (Background, Document Structure, Implementation Workflow, Pre-existing legacy file inventory, Stage A sections A.1–A.4)</file>
    <file>agent-runtime-spec.md §0–§0d, §3, §13.5</file>
    <file>docs/MVP-v0.1.md §M3</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md</file>
    <file>docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md</file>
  </read_first>

  <read_reference>
    <file purpose="*_with seam archetype + tracing instrumentation pattern">src-tauri/src/commands.rs</file>
    <file purpose="current_exe()-derived subprocess test path archetype (per gotchas.md #22)">crates/runtime-main/tests/drone_ipc_loopback.rs</file>
    <file purpose="existing typify Rust-type codegen pipeline to extend with TS codegen">crates/xtask/src/main.rs</file>
    <file purpose="hand-mirrored TS shape to replace with generated output">src/types/agent_event.ts</file>
    <file purpose="canonical AgentEvent Rust source-of-truth (10 v0.1 variants + ToolSource + AgentSpawned.session_id)">crates/runtime-core/src/event.rs</file>
  </read_reference>

  <read_prior_milestones>
    <gap_analysis_carry_forward milestone="M02"/>
    <milestone_summary milestone="M02" section="Decisions to apply before the next parent milestone"/>
  </read_prior_milestones>

  <deliverable ref="docs/build-prompts/M03-live-graph.md" section="A.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M03-live-graph.md" section="A.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M03-live-graph.md" section="Key constraints"/>

  <gates milestone="M03"/>

  <self_correction_budget>3</self_correction_budget>

  <gotchas>
    <trap>Stage A's job is to close M02 carry-forward + add deps for B–F, not to start Stage B's React Flow work — resist scope creep into graph rendering even if the React Flow + Zustand deps are now present</trap>
    <trap>secrecy/serde feature drop must not break any existing code path — grep for `secrecy::Serialize` / `serialize_with` / `#[serde(...)]` on SecretString fields before removing the feature; M02 code does NOT serialize SecretString anywhere, but verify before pulling the rug</trap>
    <trap>TS codegen mechanism choice (schemars Rust crate vs json-schema-to-typescript Node CLI) — pick ONE and document the rationale in A.3 before code; avoid mid-stage flip-flop</trap>
    <trap>`cargo xtask regenerate-types --check` must produce zero diff on PR-merged state — write the codegen carefully so output is byte-stable across re-runs (sorted fields, normalized whitespace, deterministic comments)</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT run the M03 smoke test or any browser-based check during Stage A — Stages B+ haven't built the renderer yet; the existing M02 smoke test should still work post-Stage-A, but that's a Stage B verification step, not Stage A</warning>
    <warning>DO NOT bump keyring 3.6 → 4.0 — the API breakage requires a careful audit beyond Stage A's scope; defer to a dedicated chore PR after M03 ships</warning>
    <warning>DO NOT bump Vite 7 → 8 — Vite 8 ships Rolldown (released March 2026, fresh); M03 stays on Vite 7 for risk control; M04+ evaluates Vite 8 once it's seasoned</warning>
  </execution_warnings>

  <time_box estimate_hours="2.5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage B: which Zustand-store shape Stage B will inherit; whether the TS codegen drift check ran cleanly on first try; whether the current_exe() drone test retrofit revealed any other cross-platform path issues; whether the Vite 5→7 bump introduced any unexpected dev-server behavior</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M03-live-graph.md" section="A.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results (each gate, pass/fail, key numbers including frontend coverage now-default)</item>
    <item>retrospective (filled-in [END] section with three-axis scoring + verdict + decisions for Stage B + new [END] Coverage holdouts subsection)</item>
    <item>draft commit message from M03-live-graph.md A.6 Commit Message section (filled with session URL)</item>
    <item>explicit statement: "Stage M03.A is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### A.6 Commit Message

```
chore(workspace): M03 Stage A — build hygiene + carry-forward + new deps

Closes the M02 🟡 Important carry-forward items + adds the deps Stages
B–F need. No React Flow code yet; that lands in Stage B.

Carry-forward closures (per M02-summary Decisions):
- src/counter.{js,test.js}: deleted (legacy CommonJS in `.prettierignore`
  + eslint.config.js ignores; M02.E friction r5).
- crates/runtime-drone/tests/integration*.rs: retrofitted to derive
  binary paths from std::env::current_exe() (per docs/gotchas.md #22;
  M02.D Decisions).
- src/types/agent_event.ts: now generated from schemas/event.v1.json
  via cargo xtask regenerate-types (per CLAUDE.md §14 schemas-as-
  source-of-truth; M02.E Decisions §"Was the IPC TypeScript type sync
  a source of bugs?"). Hand-mirrored content replaced; drift check in
  CI catches future divergence.
- npm run test: now defaults to `vitest run --coverage` so the 80%
  threshold in vitest.config.ts is enforced on every run (M02.E
  Decisions §"Frontend coverage 80% threshold").

Tooling refresh:
- Vite 5.4 → 7.x (esbuild CVE-2024-23334 fix; defers Vite 8 +
  Rolldown evaluation to M04+).
- secrecy/serde feature: dropped (SecretString does not serialize via
  serde by design; the feature was dead weight).
- keyring 3.6: stays (4.0 has breaking API surface; deferred to
  dedicated chore PR per M02 retros).

New deps (no usage yet — Stages B–F):
- @xyflow/react ^12.10 (renamed from `reactflow`; React Flow v12).
- zustand ^5.0 (state management; React 18+ peer dep met).

Schema groundwork:
- schemas/event.v1.json (NEW): canonical AgentEvent schema; 10 v0.1
  variants matching runtime_core::event::AgentEvent + ToolSource enum
  + AgentSpawned.session_id field. Source-of-truth for both Rust
  (typify; existing pipeline) and TypeScript (json-schema-to-
  typescript; new in xtask).
- crates/xtask/src/main.rs: `regenerate_typescript_types` function
  added alongside the existing typify Rust codegen. Wired into
  `cargo xtask regenerate-types` and `--check` subcommands. Drift
  check covers both Rust + TS outputs.

Tests:
- crates/xtask/tests/check_drift.rs: extended to cover TS codegen
  drift (asserts committed src/types/agent_event.ts matches freshly-
  regenerated output).
- 1 unit test for `regenerate_typescript_types_with` (testable seam
  per CLAUDE.md §5 *_with archetype).
- Existing tests preserved; runtime-drone integration tests retrofitted
  but per-module coverage baselines unchanged.

Refs: M03-live-graph.md §A; agent-runtime-spec.md §3 §13.5; CLAUDE.md
§5 §14; docs/gotchas.md #21–#28 (especially #22 current_exe);
M02-summary.md §"Decisions to apply before the next parent milestone";
docs/gap-analysis.md M02 entry §"Carry-forward to M03 prep".

https://claude.ai/code
```

---

<!-- ============================================================ -->
<!-- STAGE B onward — placeholder; authored in subsequent chunks    -->
<!-- ============================================================ -->

## Stage B — React Flow + Zustand foundation + 3 basic node types

*Authored in the next chunk after Stage A is approved.*

## Stage C — Remaining 8 node types + animated edges + color encoding

*Authored in subsequent chunks.*

## Stage D — Click-to-inspect side panel + token-spend visualization + zoom/pan

*Authored in subsequent chunks.*

## Stage E — VDR projection + SQL inspector + graph persistence

*Authored in subsequent chunks.*

## Stage F — Tauri 2.x desktop-shell E2E + Phase Closeout

*Authored in subsequent chunks.*

---

## Summary Table

*Populated in the final chunk before the M03 PR opens.*

## Verification Checklist

*Populated in the final chunk before the M03 PR opens.*
