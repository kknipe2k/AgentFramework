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

Apply changes in this order. Each block below is either a surgical Find/Replace, full new-file content, or an explicit instruction (with archetype reference) where the implementation is too long to inline.

#### `Cargo.toml` (workspace) — drop `secrecy/serde` feature

**Find:**

```toml
secrecy             = { version = "0.10", features = ["serde"] }
```

**Replace with:**

```toml
secrecy             = "0.10"
```

Rationale: `SecretString` does not serialize via serde by design (security; per `docs.rs/secrecy/0.10.3` + WEBCHECK in this stage's header). The `Deserialize` impl from the `serde` feature is not used in any M02 code path; removing the feature is dead-weight cleanup. Verify the cleanup with `grep -rn "secrecy" crates/ src-tauri/` after the change — every callsite should be `SecretString::from(...)` or `expose_secret()`, never serde-derived.

#### `package.json` — Vite 5→7 + new deps + script change

**Find:**

```json
    "test": "vitest run",
```

**Replace with:**

```json
    "test": "vitest run --coverage",
```

Then bump Vite + add the two new deps. **Find** the `devDependencies` block's `"vite": "^5.4.0"`, **bump** to `"vite": "^7.0.0"` (verify against current 7.x at <https://vite.dev/releases> during WEBCHECK; pick the latest stable 7.x patch).

**Add** to `devDependencies`:

```json
    "json-schema-to-typescript": "^15.0.0",
```

Verify the json-schema-to-typescript major version at <https://www.npmjs.com/package/json-schema-to-typescript> during WEBCHECK; use whichever is current stable (15.x as of late 2025; may be 16.x by M03 author time).

**Add** to `dependencies` (production deps for the renderer):

```json
    "@xyflow/react": "^12.10.0",
    "zustand": "^5.0.0",
```

Verify at <https://www.npmjs.com/package/@xyflow/react> + <https://www.npmjs.com/package/zustand> during WEBCHECK.

After the edits run `npm install` to regenerate `package-lock.json`. Commit the lockfile alongside the manifest. `npm audit --audit-level=high` should pass clean.

#### `crates/xtask/Cargo.toml` — no changes

The TS codegen path shells out to `npx json-schema-to-typescript` via `std::process::Command`; it does not require new Rust deps. The existing `anyhow` + `clap` + `serde_json` deps cover the wiring.

#### `crates/xtask/src/main.rs` — extend `regenerate-types` with TS codegen

The fresh session reads the existing `regenerate_types(check: bool)` function (lines 38–71 of `crates/xtask/src/main.rs`) as the archetype. The TS codegen extension follows the same shape but writes TS to `src/types/`.

**Add** a new function `regenerate_typescript_types(check: bool) -> Result<()>` that:

1. Walks `schemas/` for files matching the M03 codegen target list. Initial list: `event.v1.json` (added in this stage). The list is hardcoded for now (matching the typify schemas array convention); future schemas added via the same function.
2. For each schema, runs `npx --yes json-schema-to-typescript <schemas/X.v1.json>` via `std::process::Command::new("npx")`. Captures stdout (the generated TS); checks exit status (non-zero → bail with the stderr output for diagnostics).
3. Prepends a generated-file header banner identical in shape to the typify Rust files: `// AUTO-GENERATED FILE — do not edit by hand. Regenerate via `cargo xtask regenerate-types`. Source schema: schemas/X.v1.json. Generator: json-schema-to-typescript@<version>.`
4. Writes the result to `src/types/<X>.ts` (e.g., `src/types/agent_event.ts` for `event.v1.json`).
5. In `--check` mode, diffs committed vs regenerated; appends to the existing `all_drift` list rather than maintaining a separate drift list.
6. Returns the merged drift list to the caller.

The testable seam pattern (per CLAUDE.md §5 / `docs/style.md` `*_with` archetype):

```rust
/// Test-seam: regenerate TS types from a caller-supplied list of schemas
/// and runner. Tests inject an in-memory runner; production calls the npx
/// binary via `std::process::Command`.
pub fn regenerate_typescript_types_with<R>(
    schemas: &[(&str, &std::path::Path)],
    output_dir: &std::path::Path,
    runner: R,
) -> Result<Vec<String>>
where
    R: Fn(&std::path::Path) -> Result<String>,
{
    // Iterates schemas, calls runner(schema_path) -> TS source string,
    // prepends header, writes to output_dir/<name>.ts, returns drift list
    // (when check mode is wired by the caller).
}
```

The production wrapper calls the seam with `runner = |schema_path| { run_npx_json_schema_to_typescript(schema_path) }`. The unit test calls the seam with a stub runner that returns a deterministic string, so the test exercises the file-write + header-prepend logic without crossing the npx subprocess boundary.

**Wire the new function into `regenerate_types(check: bool)`:** call `regenerate_typescript_types(check)` after the existing typify loop; merge its drift output into `all_drift` so the single error message in the existing `bail!` branch covers both Rust and TS drift.

**Decision:** Hardcode the schema list (`[("event", "event.v1.json")]`) inside `regenerate_typescript_types` for M03. Future schemas added via line edits to that list. Acceptable because the typify pattern already does the same (`["common", "framework", "skill", "tool", "agent"]` array on line 43 of the existing code).

#### `schemas/event.v1.json` — new

The canonical AgentEvent schema. Source-of-truth: `crates/runtime-core/src/event.rs` (M02-shipped enum). The fresh session reads `event.rs` and emits a JSON Schema Draft 2020-12 document mirroring the AgentEvent enum:

- `$schema`: `https://json-schema.org/draft/2020-12/schema`
- `$id`: `https://schemas.aria-runtime.dev/event.v1.json` (matches the convention in other v1 schemas)
- `title`: `AgentEvent`
- `description`: short paragraph; cite spec §2 + §3.

Body is a `oneOf` of 10 variants matching the v0.1 `AgentEvent` enum:

1. `session_start { session_id, framework, model }`
2. `agent_spawned { agent_id, agent_name, parent_id?, session_id }` — `session_id` is the M02.D addition; ensure it's present in the schema
3. `agent_complete { agent_id, result }`
4. `agent_error { agent_id, error }`
5. `tool_invoked { agent_id, tool_id, input, source, server? }` — `source` is the new `ToolSource` enum (`Builtin | Mcp | Generated`); `server` populated when `source = Mcp`
6. `tool_result { agent_id, tool_id, output, duration_ms? }`
7. `skill_loaded { agent_id, skill_id, mode? }`
8. `stream_text { agent_id, text }`
9. `stream_thinking { agent_id, text }`
10. `decision_record { agent_id, decision, rationale, confidence? }`

Use `serde(tag = "type", rename_all = "snake_case")` shape: each variant has a `type` discriminator field plus the listed payload fields. Reference: spec §2 Event Types subsection (line 836+ of `agent-runtime-spec.md`). The TypeScript discriminated-union output is exactly what `src/types/agent_event.ts` should look like post-regeneration.

**Verify byte-level parity:** after the schema lands and the fresh session runs `cargo xtask regenerate-types`, the generated `src/types/agent_event.ts` must match the existing hand-mirrored content semantically. Mismatch is acceptable on whitespace / comment placement; field shapes + variant names + enum values must match. If regeneration produces a different shape than the hand-mirrored version, the schema is wrong (not the hand-mirrored version) — fix the schema until parity holds.

#### `src/types/agent_event.ts` — regenerate

After `schemas/event.v1.json` lands, run `cargo xtask regenerate-types`. The hand-mirrored content is replaced by generator output. The header banner makes it obvious to future readers that this file is generated, not authored.

Diff against the pre-Stage-A content. Spot-check: `ToolSource` enum values match (`Builtin | Mcp | Generated`); `AgentSpawned.session_id` present and required; `tool_invoked` discriminator's `source` + optional `server` fields present.

#### `src/counter.js` + `src/counter.test.js` — delete

```bash
rm src/counter.js src/counter.test.js
```

Stage the deletions.

#### `.prettierignore` — remove counter.* entries

**Find:**

```
src/counter.js
src/counter.test.js
```

**Delete those two lines.** Note: lines 24–25 in current main; surrounding context preserved.

#### `eslint.config.js` — remove counter.* entries

**Find:**

```javascript
      'src/counter.js',
      'src/counter.test.js',
```

**Delete those two lines.** Note: lines 24–25 of current `ignores` array; trailing comma and surrounding entries preserved.

#### `crates/runtime-drone/tests/integration.rs` — `current_exe()` retrofit

The existing `drone_binary()` helper (line 13 of current main) hard-codes `target/debug/runtime-drone.exe` paths. Refactor to use the M02.D archetype at `crates/runtime-main/tests/drone_ipc_loopback.rs::drone_binary` (per `docs/gotchas.md` #22).

The archetype derives the path from `std::env::current_exe()`, which works under both `cargo test` (debug target dir) and `cargo llvm-cov --workspace` (instrumented target dir). The fresh session reads the archetype function and copies its shape into `runtime-drone`'s `integration.rs`:

```rust
/// Locate the runtime-drone binary alongside the test binary.
///
/// Per `docs/gotchas.md` #22: `cargo test` puts the test binary under
/// `target/debug/deps/integration-<hash>` while `cargo llvm-cov --workspace`
/// uses a distinct target dir (`target/llvm-cov-target/...`). Hard-coding
/// `target/debug/runtime-drone` breaks under coverage runs. Deriving from
/// `std::env::current_exe()` works for both.
fn drone_binary() -> std::path::PathBuf {
    let test_exe = std::env::current_exe().expect("current_exe");
    // test_exe = .../target/<profile>/deps/integration-<hash>(.exe)
    // step up to .../target/<profile>/deps/, then to .../target/<profile>/
    let deps_dir = test_exe.parent().expect("parent of test exe");
    let target_profile = deps_dir.parent().expect("parent of deps");
    let mut p = target_profile.join("runtime-drone");
    if cfg!(windows) {
        p.set_extension("exe");
    }
    p
}
```

Same refactor in `crates/runtime-drone/tests/integration_windows.rs` if it has its own copy of the helper. If `integration_windows.rs` already calls `drone_binary()` from `integration.rs` via a shared module, the single retrofit covers both.

**After the retrofit, run** `cargo test --package runtime-drone` (verifies the helper still finds the binary at default debug profile) **and** `cargo llvm-cov --package runtime-drone` (verifies the helper finds the binary under the instrumented profile).

#### `crates/xtask/tests/check_drift.rs` — extend for TS codegen drift

The current test file exercises three drift cases for typify Rust output (`crates/runtime-core/src/generated/common.rs`). Extend with a fourth case for TS:

```rust
    // === Case 4: --check detects TS codegen drift ===
    {
        use std::fs;
        let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let target = workspace_root.join("src/types/agent_event.ts");
        let original = fs::read_to_string(&target).expect("read original");

        // Mutate: append a comment.
        fs::write(&target, format!("{original}\n// drift-test\n")).expect("write mutation");

        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["regenerate-types", "--check"])
            .output()
            .expect("run xtask --check");

        // Restore BEFORE asserting (so a panicking assertion doesn't leave the file dirty).
        fs::write(&target, &original).expect("restore");

        assert!(
            !output.status.success(),
            "drift check should detect TS mutation. stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
```

This case asserts that `cargo xtask regenerate-types --check` exits non-zero when the committed TS file diverges from regenerated output, mirroring the existing Case 3 for Rust output.

#### `vitest.config.ts` — verify coverage threshold honored

After the `package.json` script change (`"test": "vitest run --coverage"`), `vitest.config.ts`'s `coverage.thresholds.lines: 80` (from M02 Stage E) actually runs on every `npm run test`. No edit needed unless the existing config has an unrelated bug — verify by running `npm run test` after Stage A's edits and confirming the threshold is enforced (test output should include coverage table; if coverage is below 80% the run fails).

If the existing `vitest.config.ts` does not enable a `coverage` block at all, **add** the block per Vitest 2.x docs (Vitest is at 2.1.x in the M02-pinned package.json; Vite 7's peer dep allows Vitest 2.x to keep working). Reference at <https://vitest.dev/config/#coverage>.

#### `CHANGELOG.md` — `[Unreleased]` entry

**Add** a section under `[Unreleased]` matching the shape of the existing M02.E entry:

```markdown
### Added — M03.A (Build hygiene + carry-forward closures + new deps)

- ... (paste content matching A.6 Commit Message body, formatted as a
  bulleted list per Keep-a-Changelog conventions; cross-reference
  M02-summary Decisions, gotchas.md #22 + #29-31, spec §13.5)
```

The fresh Stage A session writes the bulleted version of A.6's commit-message body into CHANGELOG.


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
<!-- STAGE B — React Flow + Zustand foundation + 3 basic node types -->
<!-- ============================================================ -->

## Stage B — React Flow + Zustand foundation + 3 basic node types

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens. If any claim is stale, update this section in `M03-live-graph.md` BEFORE pasting Stage B's CLI prompt to the fresh session.

- <https://reactflow.dev/learn/customization/custom-nodes> — confirm the `@xyflow/react` v12 custom-node API: `Handle` + `Position` + `NodeProps<T>` generic typing; `nodeTypes` prop on `<ReactFlow>` is a stable-reference map (memoize or define outside the component to avoid re-renders)
- <https://reactflow.dev/api-reference/react-flow#fit-view> — confirm the `fitView` prop and `fitViewOptions` for v12; behavior on initial mount when `nodes` is empty
- <https://zustand.docs.pmnd.rs/guides/typescript> — confirm Zustand v5 TypeScript patterns (`create<T>()(set => ({...}))` shape; selector-based subscriptions for re-render minimization)
- <https://zustand.docs.pmnd.rs/integrations/persisting-store-data> — confirm v5's persist middleware behavior (M03 Stage E adds persistence; Stage B does NOT persist yet, but document the seam so Stage E's addition is non-breaking)
- <https://reactflow.dev/api-reference/types/node> + <https://reactflow.dev/api-reference/types/edge> — confirm v12 `Node` + `Edge` type shapes; specifically the `data` field's generic typing

### B.1 Problem Statement

M02 ships a flat `<ul>` event list. Spec §3 declares the live graph as the user-facing surface: agents are nodes, skill loads are edges, gaps are nodes. Stage B lays the foundation — React Flow + Zustand + three of the eleven spec §3 node types — without trying to ship every node type at once. Three node types (`AgentNode`, `ToolNode`, `SkillNode`) are enough to cover the M02 smoke-test event flow (agent_spawned → tool_invoked → skill_loaded → tool_result → agent_complete) end-to-end as a graph. The remaining eight types (MCPNode, GapNode, HITLNode, PlanNode, TaskNode, VerifyNode, HookNode, FrameworkNode) land in Stage C.

The cardinal architectural decision Stage B locks: **Zustand store, not Tauri-event-side event handling, drives the graph state.** The IPC layer (M02's `subscribeAgentEvents`) still fires on every AgentEvent; the new `applyEvent(event)` action on the Zustand store is the single entry point that translates events into node + edge mutations. Components subscribe to the store via selector hooks; React Flow renders nodes + edges from the store's snapshot.

The second decision: **the three node components are React Flow custom nodes**, not generic divs styled to look like nodes. Custom nodes get React Flow's `Handle` + `Position` primitives, participate in edge calculation correctly, and benefit from the v12 SSR-safe layout work without renderer-side hacks.

The third decision: **the store is event-driven, not snapshot-driven.** The store's state is computed by replaying every received `AgentEvent` through `applyEvent`. This is forward-compatible with Stage E's persistence: replay the persisted event log to reconstruct the graph at session-reload time. Stage B's tests use this property to verify reducer-shaped invariants (idempotence on duplicate events, ordering independence for non-causal events) without integration tests.

**One-line success criterion:** clicking "Run smoke test" in the M02 SetupPanel UI shows the smoke session as a small graph (1 AgentNode for `smoke-agent`, 0 tool/skill nodes since the smoke prompt doesn't invoke any), with the AgentNode transitioning from `active` (blue) to `complete` (green) via React Flow color encoding, and the renderer Vitest coverage on `src/lib/graphStore.ts` and the three node components is ≥80%.

**New artifacts:**
- `src/lib/graphStore.ts` — Zustand store; the canonical event-driven graph reducer
- `src/components/GraphCanvas.tsx` — `<ReactFlow>` wrapper subscribed to the store
- `src/components/nodes/AgentNode.tsx` — custom node for spec §3 AgentNode
- `src/components/nodes/ToolNode.tsx` — custom node for spec §3 ToolNode
- `src/components/nodes/SkillNode.tsx` — custom node for spec §3 SkillNode
- `tests/unit/graphStore.test.ts` — Zustand store action tests
- `tests/unit/nodes/AgentNode.test.tsx` + `ToolNode.test.tsx` + `SkillNode.test.tsx` — component render tests

### B.2 Files to Change

| File | Change |
|---|---|
| `src/lib/graphStore.ts` | **New** — Zustand store with `nodes`, `edges`, `selectedNodeId` state + `applyEvent`, `clear`, `selectNode` actions |
| `src/lib/eventReducer.ts` | **Delete** — replaced by graphStore |
| `tests/unit/eventReducer.test.ts` | **Delete** — replaced by graphStore.test.ts |
| `tests/unit/graphStore.test.ts` | **New** — covers all 6 AgentEvent variants Stage B handles, edge cases (orphan events, duplicates) |
| `src/components/GraphCanvas.tsx` | **New** — wraps `<ReactFlow>` from `@xyflow/react`; subscribes to store via selectors; defines stable `nodeTypes` map |
| `src/components/EventList.tsx` | **Delete** — replaced by GraphCanvas |
| `src/components/nodes/AgentNode.tsx` | **New** — React Flow custom node for spec §3 AgentNode (status + name) |
| `src/components/nodes/ToolNode.tsx` | **New** — React Flow custom node for spec §3 ToolNode |
| `src/components/nodes/SkillNode.tsx` | **New** — React Flow custom node for spec §3 SkillNode |
| `tests/unit/nodes/AgentNode.test.tsx` | **New** |
| `tests/unit/nodes/ToolNode.test.tsx` | **New** |
| `tests/unit/nodes/SkillNode.test.tsx` | **New** |
| `src/App.tsx` | **Edited** — replace `<EventList events={state.events} />` with `<GraphCanvas />`; remove the `useReducer(reducer, initialState)` hook (the Zustand store handles state); preserve SetupPanel + SmokeButton + handleSetKey + handleSmoke; preserve `subscribeAgentEvents` but call `applyEvent` from the store instead of `dispatch({ type: 'event_received', event })` |
| `tests/unit/App.test.tsx` | **Edited** — refactor to assert on Zustand store state (read via `useGraphStore.getState()` in test) instead of `state.events` count; preserve happy-path + error-path scenarios |
| `tests/unit/components.test.tsx` | **Edited** — drop EventList variant render tests (8 tests); preserve SetupPanel password-input + save-button-min-key-length + SmokeButton disabled-state tests; add a "renders empty GraphCanvas before any events arrive" smoke test |
| `tests/unit/ipc.test.ts` | **No change** — IPC layer is preserved as-is |
| `src/lib/ipc.ts` | **No change** — `subscribeAgentEvents` + `unwrapCmdError` already correct |
| `src/styles.css` | **Edited** — add graph-canvas styles + node-type styles (.agent-node, .tool-node, .skill-node) per spec §3 visual design (dark bg, color encoding, animated dashed edges); preserve existing M02 component styles |

### B.3 Detailed Changes

#### `src/lib/graphStore.ts` — new

```typescript
import { create } from 'zustand';
import type { Edge, Node } from '@xyflow/react';
import type { AgentEvent } from '../types/agent_event';

/**
 * Status field shared by every spec §3 node type. Drives color encoding
 * (per spec §3 Visual Design: blue=active, green=complete, red=error).
 */
export type NodeStatus = 'active' | 'complete' | 'error';

/**
 * Data attached to AgentNode instances in the React Flow graph.
 */
export interface AgentNodeData {
  agentId: string;
  agentName: string;
  status: NodeStatus;
  parentAgentId: string | null;
}

/**
 * Data attached to ToolNode instances. Stage B handles the basic shape;
 * Stage C extends with `source` ("builtin" | "mcp" | "generated") +
 * `server` for MCP tools.
 */
export interface ToolNodeData {
  toolId: string;
  toolName: string;
  agentId: string; // parent agent
  status: NodeStatus;
  durationMs: number | null;
}

/**
 * Data attached to SkillNode instances. Skills are loaded into context
 * (not called); the edge from the parent agent is dashed (no flow
 * animation per spec §3 Behavior).
 */
export interface SkillNodeData {
  skillId: string;
  skillName: string;
  agentId: string; // parent agent that loaded the skill
  mode: string | null; // mode-variant section selector if applicable
}

/**
 * Discriminated union over the three Stage B node types. Stage C extends
 * with the remaining eight spec §3 types.
 */
export type GraphNode =
  | Node<AgentNodeData, 'agent'>
  | Node<ToolNodeData, 'tool'>
  | Node<SkillNodeData, 'skill'>;

/**
 * Edge variants Stage B emits. Stage C adds animated active-call edges
 * + dashed skill-load edges per spec §3 Behavior.
 */
export type GraphEdge = Edge;

interface GraphState {
  nodes: GraphNode[];
  edges: GraphEdge[];
  selectedNodeId: string | null;

  /**
   * Single entry point for translating AgentEvent into node + edge
   * mutations. Idempotent on duplicate events (asserts in test); order-
   * independent for non-causal events (e.g., two unrelated agent_spawned
   * events).
   */
  applyEvent: (event: AgentEvent) => void;

  /**
   * Clear all nodes + edges. Called when user clicks "Run smoke test"
   * (renderer dispatches `clear` before the new session begins; M02's
   * dispatch({type:'clear'}) shape preserved).
   */
  clear: () => void;

  /**
   * Set the currently-selected node. Stage D's inspector panel uses this.
   */
  selectNode: (id: string | null) => void;
}

export const useGraphStore = create<GraphState>((set) => ({
  nodes: [],
  edges: [],
  selectedNodeId: null,

  applyEvent: (event) =>
    set((state) => {
      switch (event.type) {
        case 'session_start':
          // No node added at session_start; Stage E may render a
          // FrameworkNode root here, but Stage B has no FrameworkNode.
          return state;

        case 'agent_spawned':
          // Skip duplicate (idempotence: same agent_id appearing twice
          // is a bug upstream but the store should not crash on it).
          if (state.nodes.some((n) => n.id === `agent:${event.agentId}`)) {
            return state;
          }
          return {
            ...state,
            nodes: [
              ...state.nodes,
              {
                id: `agent:${event.agentId}`,
                type: 'agent',
                position: nextAgentPosition(state.nodes),
                data: {
                  agentId: event.agentId,
                  agentName: event.agentName,
                  status: 'active',
                  parentAgentId: event.parentId ?? null,
                },
              } satisfies Node<AgentNodeData, 'agent'>,
            ],
            edges: event.parentId
              ? [
                  ...state.edges,
                  {
                    id: `edge:${event.parentId}->${event.agentId}`,
                    source: `agent:${event.parentId}`,
                    target: `agent:${event.agentId}`,
                  },
                ]
              : state.edges,
          };

        case 'tool_invoked':
          // ... (per-event mutation; full body in the fresh session)
          return state;

        case 'tool_result':
          // Update existing ToolNode's status to 'complete' + durationMs;
          // if no matching ToolNode (out-of-order arrival), no-op.
          return state;

        case 'skill_loaded':
          // Add SkillNode + dashed edge from agent.
          return state;

        case 'agent_complete':
          return updateAgentStatus(state, event.agentId, 'complete');

        case 'agent_error':
          return updateAgentStatus(state, event.agentId, 'error');

        // M02 emits these too; Stage B treats them as no-op (no node
        // representation). Stage D's inspector panel surfaces stream
        // text + decision records.
        case 'stream_text':
        case 'stream_thinking':
        case 'decision_record':
          return state;

        default: {
          // Exhaustiveness check — TS narrows event to `never` here.
          const _exhaustive: never = event;
          return state;
        }
      }
    }),

  clear: () => set({ nodes: [], edges: [], selectedNodeId: null }),

  selectNode: (id) => set({ selectedNodeId: id }),
}));

// ---- helpers ----

function nextAgentPosition(existing: GraphNode[]): { x: number; y: number } {
  // Naive layout for Stage B: stagger horizontally. Stage D adds a real
  // layout algorithm (probably dagre). The position is the React Flow
  // default coordinate space (px); React Flow's `fitView` re-centers
  // on mount.
  const agentCount = existing.filter((n) => n.type === 'agent').length;
  return { x: agentCount * 220, y: 0 };
}

function updateAgentStatus(
  state: { nodes: GraphNode[]; edges: GraphEdge[]; selectedNodeId: string | null },
  agentId: string,
  status: NodeStatus,
): GraphState {
  return {
    ...state,
    nodes: state.nodes.map((n) =>
      n.id === `agent:${agentId}` && n.type === 'agent'
        ? { ...n, data: { ...n.data, status } }
        : n,
    ),
  } as unknown as GraphState; // narrow the discriminated union after the map
}
```

The full `tool_invoked`, `tool_result`, `skill_loaded` bodies are authored verbatim by the fresh session per the patterns shown above (idempotence + parent-edge wiring). Each follows the same shape: lookup by stable id (`tool:<id>` / `skill:<id>`), early-return on duplicate, append node + edge.

#### `src/components/nodes/AgentNode.tsx` — new

```typescript
import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { AgentNodeData } from '../../lib/graphStore';

export function AgentNode({ data }: NodeProps<AgentNodeData>) {
  return (
    <div
      className={`agent-node agent-node--${data.status}`}
      data-testid={`agent-node-${data.agentId}`}
      data-status={data.status}
      aria-label={`agent ${data.agentName} (${data.status})`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="agent-node__name">{data.agentName}</div>
      <div className="agent-node__id">{data.agentId.slice(0, 8)}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
```

`ToolNode.tsx` and `SkillNode.tsx` follow the same shape with their respective data types. SkillNode's wrapper `div` gets a `skill-node--dashed` modifier so the spec §3 dashed-edge / no-flow-animation styling applies (see CSS section).

#### `src/components/GraphCanvas.tsx` — new

```typescript
import { ReactFlow, Background, Controls } from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { useGraphStore } from '../lib/graphStore';
import { AgentNode } from './nodes/AgentNode';
import { ToolNode } from './nodes/ToolNode';
import { SkillNode } from './nodes/SkillNode';

// Defined OUTSIDE the component per @xyflow/react v12 docs: nodeTypes is
// a stable-reference map; redefining it on each render forces React Flow
// to re-mount every node, which kills the streaming UX.
const nodeTypes = {
  agent: AgentNode,
  tool: ToolNode,
  skill: SkillNode,
};

export function GraphCanvas() {
  const nodes = useGraphStore((s) => s.nodes);
  const edges = useGraphStore((s) => s.edges);
  const selectNode = useGraphStore((s) => s.selectNode);

  return (
    <div className="graph-canvas" data-testid="graph-canvas">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        fitView
        onNodeClick={(_, node) => selectNode(node.id)}
        onPaneClick={() => selectNode(null)}
      >
        <Background />
        <Controls />
      </ReactFlow>
    </div>
  );
}
```

#### `src/App.tsx` — replace EventList with GraphCanvas

**Find:**

```typescript
import { useEffect, useReducer, useState } from 'react';
import { initialState, reducer } from './lib/eventReducer';
```

**Replace with:**

```typescript
import { useEffect, useState } from 'react';
import { useGraphStore } from './lib/graphStore';
```

**Find:**

```typescript
import { EventList } from './components/EventList';
```

**Replace with:**

```typescript
import { GraphCanvas } from './components/GraphCanvas';
```

**Find** the `useReducer` line and replace with `useGraphStore` selectors. The `dispatch({ type: 'event_received', event })` call inside `subscribeAgentEvents` becomes `useGraphStore.getState().applyEvent(event)`. The `dispatch({ type: 'clear' })` call at the start of `handleSmoke` becomes `useGraphStore.getState().clear()`. The `<EventList events={state.events} />` JSX becomes `<GraphCanvas />`.

**Preserve verbatim:** SetupPanel + SmokeButton + handleSetKey + handleSmoke (including the `console.error` + `unwrapCmdError(e)` from PR #45). The state-machine semantics (clear → started → completed/error) shift from the deleted `eventReducer` to a new minimal state hook (`const [running, setRunning] = useState(false);` + `state.error` derived from a separate `useState(null)`). Full `App.tsx` body produced by the fresh session.

#### `src/styles.css` — graph + node styles

Append the graph styles. Preserve M02's existing component styles. Per spec §3 Visual Design Principles:

- Dark background, high-contrast labels
- Color encoding: `active` = blue, `complete` = green, `error` = red, `gap` (Stage C) = amber, `hitl` (Stage C) = white/bright
- Edges animated dashes during active calls (Stage C); solid when complete
- SkillNode dashed outline; no flow animation

```css
.graph-canvas {
  width: 100%;
  height: 70vh;
  background: #0e1014;
}

.agent-node {
  padding: 8px 12px;
  border-radius: 6px;
  font-family: system-ui, sans-serif;
  font-size: 13px;
  border: 2px solid #2a3045;
  background: #15192a;
  color: #e6e6e6;
  min-width: 140px;
}
.agent-node--active   { border-color: #4a90e2; }
.agent-node--complete { border-color: #4caf50; }
.agent-node--error    { border-color: #e53935; }

.tool-node { /* same shape with .tool-node--{active,complete,error} */ }
.skill-node {
  border-style: dashed;
  /* no animation; skill is loaded-into-context, not called */
}
```

Stage C extends with edge animation styles (`.react-flow__edge--animated` overrides) + the remaining eight node types' base styles. Stage D extends with token-spend-driven `font-size` scaling on AgentNode.

### B.4 Tests

#### Pedantic-pass preflight

Stage B introduces several new modules (graphStore, GraphCanvas, AgentNode, ToolNode, SkillNode). Apply the checklist from `docs/gotchas.md` #21:

- [ ] No clippy traps (Stage B is TS only — N/A for the Rust pedantic list)
- [ ] No TS strict-mode violations: `tsc --noEmit` clean
- [ ] No ESLint flat-config violations: `eslint .` clean
- [ ] No prettier violations: `prettier --check .` clean

Frontend-specific traps (per `docs/gotchas.md` #25, #26, #27):

- [ ] Vitest+RTL DOM-ref staleness: any `await waitFor(...)` followed by element interaction must re-query (`findByLabelText`) — never reuse the pre-await ref (per #27)
- [ ] React Flow `nodeTypes` map MUST be defined outside the component (per WEBCHECK / React Flow v12 docs); inline definition causes re-mounts on every render
- [ ] Zustand v5 selector patterns: components subscribe via `useGraphStore((s) => s.<field>)`, not `useGraphStore()` (the latter triggers re-renders on any state change; the selector form re-renders only when the selected slice changes)

#### Default test plan for stages adding a new safety primitive

The Zustand store is the closest thing to a safety primitive in Stage B (single source of graph state; bug here breaks the entire UI). Apply the seam-test pattern (per `docs/build-prompts/TEMPLATE.md` post-M02 addition):

- N unit tests for the store actions (testable seam: `applyEvent` is pure-function-shaped; tests dispatch events directly and assert on the resulting state without touching React Flow)
- M integration tests (Vitest+RTL) for the GraphCanvas component rendering nodes/edges from the store

#### Test plan (Stage B)

`tests/unit/graphStore.test.ts` — Zustand store action tests:

1. **agent_spawned: adds AgentNode with active status** — apply event, assert `state.nodes` has one AgentNode with id `agent:<id>` and `status: 'active'`
2. **agent_spawned with parentId: adds parent edge** — apply parent + child agent_spawned, assert `state.edges` contains the parent→child edge
3. **agent_spawned: idempotent on duplicate** — apply same event twice, assert nodes count is 1
4. **agent_complete: updates AgentNode status to complete** — spawn agent + complete, assert status transition
5. **agent_error: updates AgentNode status to error** — same shape
6. **tool_invoked: adds ToolNode + edge from agent** — assert node + edge correctly wired
7. **tool_result: updates ToolNode status to complete + durationMs** — assert update
8. **skill_loaded: adds SkillNode + dashed edge from agent** — assert node added; edge data field marks it dashed
9. **stream_text / stream_thinking / decision_record: no-op** — assert state unchanged after these events
10. **clear: empties nodes + edges + selectedNodeId** — populate then clear, assert empty state
11. **selectNode: sets selectedNodeId** — assert
12. **applyEvent on session_start: no-op (Stage E adds FrameworkNode)** — assert state unchanged

Coverage target on `src/lib/graphStore.ts`: ≥95% line (treat as primitive — every branch in `applyEvent` is covered by an explicit test above).

`tests/unit/nodes/AgentNode.test.tsx` — render tests:

1. **renders agent name + truncated agent_id** — assert
2. **applies status class for active|complete|error** — three render assertions with each status
3. **has accessible aria-label** — assert label contains agent name + status
4. **has data-testid + data-status attributes** — for E2E selectability (Stage F)
5. **renders source + target Handles** — verify React Flow handle elements present

Same shape for ToolNode + SkillNode (~5 tests each).

`tests/unit/App.test.tsx` — refactored:

1. **save-key + run-smoke happy path: AgentNode appears in graph** — mock `subscribeAgentEvents` to fire an `agent_spawned`, assert `useGraphStore.getState().nodes` contains the AgentNode
2. **command-error surface: state.error rendered, no node added** — fire an invoke rejection, assert error path

`tests/unit/components.test.tsx` — drop EventList variant tests; keep:

- SetupPanel password-input invariant
- SetupPanel save-button-min-key-length
- SmokeButton disabled-state
- Add: GraphCanvas renders empty before any events arrive (data-testid="graph-canvas" present; no nodes)

#### Coverage target

- Workspace Rust: ≥80% (preserved; no Rust change in Stage B)
- runtime-drone: ≥95% (preserved; no Rust change)
- runtime-main: ≥95% (preserved; no Rust change)
- src-tauri: 50% patch gate (preserved; no Rust change)
- **src/ frontend: ≥80%** with **graphStore.ts ≥95%** (treat as primitive; the store is the single source of truth for graph state)

**Doc-to-CI invariant.** No new exclusions in Stage B. The graphStore + components are pure TS, no OS-call wrappers; everything is testable. If Stage B somehow surfaces an OS-call holdout (unexpected; React Flow + Zustand are pure renderer code), update CI workflow regex + CLAUDE.md §5 + the retro `[END] Coverage holdouts` subsection in the same commit.

### B.5 CLI Prompt

```xml
<work_stage_prompt id="M03.B">
  <context>
    Stage B of M03 (Live Graph). React Flow + Zustand foundation; three
    basic node types (Agent, Tool, Skill); replace the M02 EventList with
    a GraphCanvas component. Builds on Stage A's @xyflow/react + zustand
    deps + regenerated agent_event.ts schema. Stage C does not start
    until Stage B's commit is on the milestone branch.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M03-live-graph.md (Stage B sections B.1–B.4)</file>
    <file>agent-runtime-spec.md §3</file>
    <file>docs/MVP-v0.1.md §M3</file>
    <file>docs/style.md</file>
    <file>docs/gotchas.md (especially #21 #25 #26 #27)</file>
  </read_first>

  <read_reference>
    <file purpose="event-driven-store archetype + applyEvent shape; Stage A regenerated this">src/types/agent_event.ts</file>
    <file purpose="M02 useReducer pattern being replaced">src/lib/eventReducer.ts</file>
    <file purpose="component test idiom (RTL queries, mock @tauri-apps/api)">tests/unit/components.test.tsx</file>
    <file purpose="App.tsx state-machine pattern to preserve (handleSetKey + handleSmoke + console.error)">src/App.tsx</file>
    <file purpose="renderer-side IPC wrappers (preserved as-is)">src/lib/ipc.ts</file>
  </read_reference>

  <read_prior_stages>
    <retrospective section="[END] Decisions for the next stage">docs/build-prompts/retrospectives/M03.A-retrospective.md</retrospective>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M03-live-graph.md" section="B.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M03-live-graph.md" section="B.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M03-live-graph.md" section="Key constraints"/>

  <gates milestone="M03"/>

  <self_correction_budget>3</self_correction_budget>

  <gotchas>
    <trap>Stage B's job is the foundation + 3 basic node types, NOT all 11. Resist scope creep; MCPNode/GapNode/HITLNode/PlanNode/TaskNode/VerifyNode/HookNode/FrameworkNode all land in Stage C.</trap>
    <trap>React Flow v12 nodeTypes map MUST be defined outside the component (per WEBCHECK reference). Inline definition forces React Flow to re-mount every node on every render — kills streaming UX. Define `nodeTypes` at module level (per the GraphCanvas.tsx pattern in B.3).</trap>
    <trap>Zustand v5 selector pattern matters: `useGraphStore((s) => s.nodes)` re-renders only on nodes changes; bare `useGraphStore()` re-renders on any state change. Components must use selector form (per WEBCHECK reference).</trap>
    <trap>Vitest+RTL DOM-ref staleness (per docs/gotchas.md #27): any test that awaits a state change must re-query the DOM via `findByLabelText` etc. before interacting with elements. Capturing pre-await refs is broken even when the await resolves.</trap>
    <trap>The `graphStore.ts` `applyEvent` function MUST handle the discriminated union exhaustively (the `_exhaustive: never` check in the default branch). Adding a new AgentEvent variant later that the store doesn't handle is a TS compile error, not a silent no-op.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT bump @xyflow/react or zustand mid-stage — Stage A pinned them; Stage B uses what Stage A installed. Bumping mid-stage means re-verifying every test.</warning>
    <warning>DO NOT add edge animation in Stage B — Stage C handles animated edges + dashed-edge styling. Stage B ships with React Flow's default edge rendering.</warning>
    <warning>DO NOT touch SetupPanel, SmokeButton, or ipc.ts unless absolutely required — they're preserved verbatim from M02.</warning>
  </execution_warnings>

  <time_box estimate_hours="5.5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage C: which edge-animation pattern Stage C will use; whether the @xyflow/react `Background` + `Controls` toolbar feels right for spec §3 visual design; whether the AgentNode/ToolNode/SkillNode CSS structure can be reused for the remaining 8 node types or needs a refactor; whether the Zustand store's discriminated-union exhaustiveness check held up against the new variants Stage C adds.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M03-live-graph.md" section="B.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results (each gate, pass/fail, key numbers including frontend coverage now ≥80% with graphStore ≥95%)</item>
    <item>retrospective (filled-in [END] section with three-axis scoring + verdict + decisions for Stage C + new [END] Coverage holdouts subsection)</item>
    <item>draft commit message from M03-live-graph.md B.6 Commit Message section (filled with session URL)</item>
    <item>screenshot or paste of the rendered graph after a successful smoke-test run, showing the AgentNode in active → complete state</item>
    <item>explicit statement: "Stage M03.B is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### B.6 Commit Message

```
feat(renderer): M03 Stage B — React Flow + Zustand foundation + 3 basic nodes

Lays the foundation for the live graph. Replaces M02's flat <ul>
event list with a React Flow canvas backed by a Zustand store.
Three of the eleven spec §3 node types ship: AgentNode, ToolNode,
SkillNode. The remaining eight (MCP, Gap, HITL, Plan, Task, Verify,
Hook, Framework) land in Stage C.

Architecture:
- Zustand store (src/lib/graphStore.ts) is the single source of
  graph state. The `applyEvent(event)` action is the canonical
  AgentEvent → node/edge translation (idempotent on duplicates;
  exhaustive over the discriminated union via TS `never` check).
- React Flow custom nodes (src/components/nodes/{Agent,Tool,Skill}
  Node.tsx) use Handle + Position + NodeProps<T> primitives. CSS
  per spec §3 visual design (dark bg, color encoding for status).
- GraphCanvas component (src/components/GraphCanvas.tsx) wraps
  <ReactFlow>; nodeTypes defined at module level (per @xyflow/react
  v12 docs; inline definition causes per-render remounts).
- App.tsx refactored to subscribe to the Zustand store via selectors
  instead of useReducer; SetupPanel + SmokeButton + handleSmoke +
  unwrapCmdError preserved verbatim from M02.

Deletions:
- src/lib/eventReducer.ts (replaced by graphStore.ts)
- src/components/EventList.tsx (replaced by GraphCanvas)
- tests/unit/eventReducer.test.ts (replaced by graphStore.test.ts)

Tests (new):
- tests/unit/graphStore.test.ts — 12 tests covering each AgentEvent
  variant Stage B handles + idempotence + clear/select actions.
  Coverage on graphStore.ts: ≥95% line (treated as primitive).
- tests/unit/nodes/{Agent,Tool,Skill}Node.test.tsx — 5 tests each
  covering render + status classes + accessibility + handles.
- tests/unit/components.test.tsx — refactored: dropped EventList
  variant tests; added GraphCanvas empty-state smoke test.
- tests/unit/App.test.tsx — refactored: asserts on Zustand store
  state instead of state.events.

Per-stage decisions (per Stage B retro):
- Naive horizontal-stagger layout for AgentNodes; Stage D adds a
  proper layout algorithm (probably dagre).
- Edge animation deferred to Stage C; Stage B ships with React Flow's
  default static edges.
- Token-spend node weight deferred to Stage D.

Refs: M03-live-graph.md §B; agent-runtime-spec.md §3; CLAUDE.md §5
*_with archetype (graphStore.applyEvent is the seam); docs/gotchas.md
#21 (clippy traps), #25 (Vite root), #26 (serde tag-shape — N/A for
TS), #27 (Vitest+RTL DOM-ref staleness).

https://claude.ai/code
```

---
<!-- ============================================================ -->
<!-- STAGE C — 8 remaining node types + animated edges + colors     -->
<!-- ============================================================ -->

## Stage C — Remaining 8 node types + animated edges + color encoding

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens. If any claim is stale, update this section in `M03-live-graph.md` BEFORE pasting Stage C's CLI prompt to the fresh session.

- <https://reactflow.dev/api-reference/types/edge> — confirm `Edge` type's `animated` field semantics in `@xyflow/react` v12; class-name conventions for animated edges (`react-flow__edge--animated`)
- <https://reactflow.dev/learn/customization/custom-edges> — confirm v12's custom-edge API; needed if Stage C decides to use a custom Edge component for the dash-flow visual instead of pure CSS
- <https://reactflow.dev/api-reference/types/node-props> — confirm v12's `NodeProps<T>` generic for typed `data`; the eight new components consume this via the same pattern as Stage B's three
- <https://www.w3.org/WAI/ARIA/apg/patterns/> — confirm ARIA pattern recommendations for graph nodes (especially for HITLNode which presents a blocking-on-input affordance — does it need `role="alert"` / `aria-live="assertive"`?)

### C.1 Problem Statement

Stage B shipped three of eleven node types (Agent, Tool, Skill). Stage C adds the remaining eight: **MCPNode**, **GapNode**, **HITLNode**, **PlanNode**, **TaskNode**, **VerifyNode**, **HookNode**, **FrameworkNode**. Plus the spec §3 visual layer: animated edges during active tool calls, dashed edges for skill loads (Stage B's foundation extended consistently), and the full color-encoding palette across every node type's status states (active blue, complete green, error red, **gap amber**, **hitl white/bright**).

The cardinal scoping decision: **Stage C ships components + the graphStore wiring for events that already exist; M4+ wires the rest.** Per MVP §M3 out-of-scope ("plan/task nodes wired to data → M4; MCP node connecting to a real server → M6"), Stage C ships **eight component files** with full styling and accessibility, but the graphStore's `applyEvent` only adds new wiring for events the v0.1 AgentEvent schema actually emits:

- **`session_start` → spawn FrameworkNode** as the graph's root (already in v0.1 schema; not yet handled by Stage B's store)
- **`tool_invoked` with `source: 'mcp'` + `server: '<name>'` → lazily spawn MCPNode** as the parent of the ToolNode (Stage B treats every tool the same; Stage C splits MCP-hosted tools into their own MCPNode parent group)

The remaining six components (Gap, HITL, Plan, Task, Verify, Hook) ship as **renderable components without store wiring**. Their `applyEvent` cases land at M4 (plan/task/verify/hook events) and M5 (gap events). Stage C tests them with **synthetic store fixtures** — manually populated state passed to `<GraphCanvas>` — to verify render correctness without manufacturing events the schema doesn't yet emit.

The second decision: **animated edges via Edge.animated + CSS, not custom edge components.** React Flow v12's built-in `animated: true` field on `Edge` triggers the `react-flow__edge--animated` CSS class; pure CSS handles the dash-flow visual. Custom edge components are reserved for M4+ if a need surfaces (e.g., per-edge-type token spend visualization).

The third decision: **edge state lifecycle.** When `tool_invoked` fires, the agent → tool edge is created with `animated: true`. When `tool_result` fires, the same edge transitions to `animated: false` + a `complete` color class. When `agent_error` fires, downstream edges (where source is the failed agent) transition to `error` color. Stage C codifies this state machine in graphStore.

The fourth decision: **GapNode visual prominence.** Per spec §3 Behavior ("GapNode appears immediately on `tool_missing` ... HITLNode blocks the graph visually, dims non-relevant nodes"), GapNode + HITLNode get prominence styling — pulsing borders or higher z-index. Stage C ships the styles; the actual gap-flow integration with the agent loop is M5.

**One-line success criterion:** every node type renders correctly in isolation (Vitest); the M02 smoke test (single AgentNode + zero ToolNodes since the prompt invokes none) now also displays a FrameworkNode root with an edge to the smoke-agent; CSS color encoding verified visually + via component-test class assertions; renderer Vitest coverage on the 8 new components ≥80%; graphStore.ts coverage stays ≥95%.

**New artifacts:**
- `src/components/nodes/MCPNode.tsx` + 7 sibling files (one per remaining node type)
- 8 new test files at `tests/unit/nodes/<NodeName>.test.tsx`
- Extended `src/lib/graphStore.ts` (FrameworkNode + MCPNode wiring; edge-animation lifecycle)
- Extended `src/styles.css` (8 new node-type styles + animated-edge styles + dashed-edge styles)
- Extended `src/components/GraphCanvas.tsx` (`nodeTypes` map with 11 entries)

### C.2 Files to Change

| File | Change |
|---|---|
| `src/components/nodes/MCPNode.tsx` | **New** — React Flow custom node for spec §3 MCPNode; renders MCP server name + tool count badge |
| `src/components/nodes/GapNode.tsx` | **New** — spec §3 GapNode; amber color + prominent styling per Visual Design Principles |
| `src/components/nodes/HITLNode.tsx` | **New** — spec §3 HITLNode; bright/white styling + `role="alert"` + `aria-live="assertive"` |
| `src/components/nodes/PlanNode.tsx` | **New** — spec §3 PlanNode; renders plan name + progress (placeholder data; M4 wires real plan-state data) |
| `src/components/nodes/TaskNode.tsx` | **New** — spec §3 TaskNode; renders task name + status + HITL flag |
| `src/components/nodes/VerifyNode.tsx` | **New** — spec §3 VerifyNode; pass/fail state visualization |
| `src/components/nodes/HookNode.tsx` | **New** — spec §3 HookNode; hook name + category + outcome |
| `src/components/nodes/FrameworkNode.tsx` | **New** — spec §3 FrameworkNode; framework name + active model |
| `tests/unit/nodes/MCPNode.test.tsx` | **New** — 5 tests: render + status classes + tool-count + a11y + handles |
| `tests/unit/nodes/GapNode.test.tsx` | **New** — same shape; emphasize amber color class assertion |
| `tests/unit/nodes/HITLNode.test.tsx` | **New** — same shape; assert `role="alert"` + `aria-live` attrs |
| `tests/unit/nodes/PlanNode.test.tsx` | **New** |
| `tests/unit/nodes/TaskNode.test.tsx` | **New** |
| `tests/unit/nodes/VerifyNode.test.tsx` | **New** |
| `tests/unit/nodes/HookNode.test.tsx` | **New** |
| `tests/unit/nodes/FrameworkNode.test.tsx` | **New** |
| `src/lib/graphStore.ts` | **Edited** — extend `GraphNode` discriminated union with 8 new variants; add data interfaces (MCPNodeData, GapNodeData, etc.); extend `applyEvent` for `session_start` → FrameworkNode + `tool_invoked.source='mcp'` → lazy MCPNode + animated-edge lifecycle; the six remaining variants get type definitions but no `applyEvent` handlers (M4+ adds those) |
| `tests/unit/graphStore.test.ts` | **Edited** — add 2 new tests: `session_start: spawns FrameworkNode at root` + `tool_invoked with source mcp: lazily spawns parent MCPNode`; update existing `agent_spawned` test to assert FrameworkNode is the parent when no parentId is provided |
| `src/components/GraphCanvas.tsx` | **Edited** — extend `nodeTypes` map from 3 entries to 11 (one per spec §3 node type) |
| `src/styles.css` | **Edited** — add 8 new node-type styles (each with `--active`, `--complete`, `--error` modifiers per spec §3 color encoding); add `--gap` (amber) modifier on GapNode; add `--hitl` (bright/white) modifier on HITLNode; add `.react-flow__edge--animated` dash-flow keyframes; add `.react-flow__edge--dashed` style for skill-load edges |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` entry under "Added — M03.C" |

### C.3 Detailed Changes

#### `src/lib/graphStore.ts` — extend types + applyEvent

Add eight new data interfaces + extend the `GraphNode` discriminated union. Pattern (one shown; rest follow same shape):

```typescript
export interface MCPNodeData {
  serverId: string;
  serverName: string;
  status: NodeStatus;
  discoveredToolCount: number | null;
}

export interface GapNodeData {
  gapId: string;
  kind: 'tool_missing' | 'skill_missing';
  missingName: string;
  status: NodeStatus; // 'gap' is rendered via amber color class on the component side
}

export interface HITLNodeData {
  hitlId: string;
  prompt: string;
  resolved: boolean;
}

// PlanNodeData, TaskNodeData, VerifyNodeData, HookNodeData,
// FrameworkNodeData follow the same pattern. All have a `status: NodeStatus`
// or equivalent state field. Full content authored by the fresh session
// per spec §3 + the existing AgentNodeData/ToolNodeData/SkillNodeData
// shapes from Stage B.
```

Extend the `GraphNode` discriminated union:

```typescript
export type GraphNode =
  | Node<AgentNodeData,     'agent'>
  | Node<ToolNodeData,      'tool'>
  | Node<SkillNodeData,     'skill'>
  | Node<MCPNodeData,       'mcp'>
  | Node<GapNodeData,       'gap'>
  | Node<HITLNodeData,      'hitl'>
  | Node<PlanNodeData,      'plan'>
  | Node<TaskNodeData,      'task'>
  | Node<VerifyNodeData,    'verify'>
  | Node<HookNodeData,      'hook'>
  | Node<FrameworkNodeData, 'framework'>;
```

Extend `applyEvent` for the two new event-driven cases:

```typescript
case 'session_start':
  // Spawn FrameworkNode as the graph's root if not already present.
  // Idempotent (same session_id arriving twice no-ops).
  if (state.nodes.some((n) => n.id === `framework:${event.framework}`)) {
    return state;
  }
  return {
    ...state,
    nodes: [
      ...state.nodes,
      {
        id: `framework:${event.framework}`,
        type: 'framework',
        position: { x: -200, y: -150 },
        data: {
          frameworkName: event.framework,
          model: event.model,
          status: 'active' as NodeStatus,
        },
      } satisfies Node<FrameworkNodeData, 'framework'>,
    ],
  };

case 'tool_invoked':
  // Stage B's existing handler is extended: when source === 'mcp', lazily
  // spawn the MCPNode parent (if not already present), then add the
  // ToolNode + edge from MCP → tool. When source !== 'mcp', existing
  // Stage B behavior holds (edge from agent → tool directly).
  // Animated-edge lifecycle: edges created here have `animated: true`;
  // tool_result handler turns this off.
  // Full body authored by the fresh session.
  return /* extended state */;

case 'tool_result':
  // Update existing ToolNode's status to 'complete' + durationMs;
  // turn off animated flag on the agent → tool (or MCP → tool) edge.
  return /* updated state */;

// The six remaining variants from spec §3 (gap_added, hitl_requested,
// plan_created, task_started, verify_completed, hook_fired) are NOT
// handled in Stage C — schemas/event.v1.json only declares the 10 v0.1
// variants Stage A added. M4+ extends the schema and adds these handlers.
// The TS exhaustiveness check on `applyEvent`'s default branch enforces
// this: adding a new variant to the schema later is a compile error
// until the handler lands.
```

The wiring for Gap, HITL, Plan, Task, Verify, Hook in `applyEvent` is **deliberately deferred to M4+**. Stage C tests these components by passing **synthetic graph state** to `<GraphCanvas>` directly in unit tests:

```typescript
// tests/unit/nodes/GapNode.test.tsx — example of synthetic-state testing
import { render } from '@testing-library/react';
import { ReactFlow } from '@xyflow/react';
import { GapNode } from '../../../src/components/nodes/GapNode';

const nodeTypes = { gap: GapNode };

test('GapNode renders with amber color when kind is tool_missing', () => {
  const nodes = [{
    id: 'gap:tool-missing-foo',
    type: 'gap',
    position: { x: 0, y: 0 },
    data: { gapId: 'tool-missing-foo', kind: 'tool_missing', missingName: 'foo', status: 'gap' },
  }];
  const { getByTestId } = render(<ReactFlow nodes={nodes} edges={[]} nodeTypes={nodeTypes} />);
  const node = getByTestId('gap-node-tool-missing-foo');
  expect(node).toHaveClass('gap-node--gap');
  expect(node).toHaveAttribute('data-kind', 'tool_missing');
});
```

This pattern verifies render correctness without forcing the store to handle events that don't yet exist in the schema.

#### `src/components/nodes/<NodeName>.tsx` — eight new files

Each follows the AgentNode pattern from Stage B with type-appropriate data:

```typescript
// src/components/nodes/FrameworkNode.tsx — example
import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { FrameworkNodeData } from '../../lib/graphStore';

export function FrameworkNode({ data }: NodeProps<FrameworkNodeData>) {
  return (
    <div
      className={`framework-node framework-node--${data.status}`}
      data-testid={`framework-node-${data.frameworkName}`}
      data-status={data.status}
      aria-label={`framework ${data.frameworkName} on ${data.model}`}
    >
      <div className="framework-node__name">{data.frameworkName}</div>
      <div className="framework-node__model">{data.model}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
```

The seven other node components follow the same shape with their respective data fields. Two get specialized markup:

- **GapNode** — extra `data-kind` attribute (`tool_missing` | `skill_missing`); body lists the missing artifact name; amber border + pulsing animation modifier (CSS handles the pulse via `@keyframes`)
- **HITLNode** — `role="alert"` + `aria-live="assertive"` per ARIA APG patterns for blocking inputs; renders `data.prompt` so the user sees what's being asked

The fresh Stage C session reads `src/components/nodes/AgentNode.tsx` (Stage B archetype) + this section for the per-component specializations and produces the eight files in one pass.

#### `src/components/GraphCanvas.tsx` — extend nodeTypes

**Find:**

```typescript
const nodeTypes = {
  agent: AgentNode,
  tool: ToolNode,
  skill: SkillNode,
};
```

**Replace with:**

```typescript
const nodeTypes = {
  agent: AgentNode,
  tool: ToolNode,
  skill: SkillNode,
  mcp: MCPNode,
  gap: GapNode,
  hitl: HITLNode,
  plan: PlanNode,
  task: TaskNode,
  verify: VerifyNode,
  hook: HookNode,
  framework: FrameworkNode,
};
```

Plus the matching imports at the top of the file.

#### `src/styles.css` — 8 new node styles + edge animation

Append the new node-type styles (one per node, with `--active`/`--complete`/`--error` modifiers; GapNode adds `--gap` amber; HITLNode adds `--hitl` bright). Spec §3 Visual Design Principles drive the colors:

```css
/* Color encoding palette (spec §3 Visual Design Principles) */
:root {
  --node-active:   #4a90e2;
  --node-complete: #4caf50;
  --node-error:    #e53935;
  --node-gap:      #ffa726;
  --node-hitl:     #ffffff;
}

/* MCPNode — server hosting tools */
.mcp-node {
  padding: 8px 12px;
  border-radius: 6px;
  font-family: system-ui, sans-serif;
  font-size: 13px;
  border: 2px solid #2a3045;
  background: #15192a;
  color: #e6e6e6;
  min-width: 160px;
}
.mcp-node--active   { border-color: var(--node-active); }
.mcp-node--complete { border-color: var(--node-complete); }
.mcp-node--error    { border-color: var(--node-error); }
.mcp-node__tool-count {
  font-size: 11px;
  color: #aaa;
  margin-top: 2px;
}

/* GapNode — missing tool/skill (amber per spec §3) */
.gap-node {
  /* base styles */
}
.gap-node--gap {
  border-color: var(--node-gap);
  animation: gap-pulse 1.4s ease-in-out infinite;
}
@keyframes gap-pulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(255,167,38,0.4); }
  50%      { box-shadow: 0 0 0 8px rgba(255,167,38,0); }
}

/* HITLNode — blocking on human input (bright/white per spec §3) */
.hitl-node {
  /* base styles */
}
.hitl-node--hitl {
  border-color: var(--node-hitl);
  background: rgba(255,255,255,0.08);
}

/* Plan/Task/Verify/Hook/FrameworkNode — base + status modifiers, same pattern */

/* Animated edges (active tool calls per spec §3 Behavior) */
.react-flow__edge.animated .react-flow__edge-path {
  stroke-dasharray: 5 5;
  animation: dash-flow 1s linear infinite;
}
@keyframes dash-flow {
  to { stroke-dashoffset: -10; }
}

/* Dashed edges for skill loads (no animation; loaded skill stays in context) */
.react-flow__edge--dashed .react-flow__edge-path {
  stroke-dasharray: 3 3;
}
```

Full CSS body produced by the fresh session per the patterns above. Verify visually after the renderer compiles by running `npm run tauri dev` and triggering the M02 smoke test — the AgentNode now has a FrameworkNode parent edge.

### C.4 Tests

#### Pedantic-pass preflight

Same checklist as Stage B (TS strict, ESLint flat-config, prettier). Per `docs/gotchas.md` #21 and the Stage B traps (nodeTypes stability, Zustand selector pattern, Vitest+RTL DOM-ref staleness, exhaustive applyEvent). New trap class for Stage C:

- [ ] `tsc --noEmit` clean across the new discriminated-union variants
- [ ] No `as any` / `@ts-ignore` to bypass narrowing in the 8 new component files
- [ ] No `any` in test fixtures — use the data interfaces from `graphStore.ts`

#### Default test plan for stages adding a new safety primitive

`graphStore.ts` is still the primitive being protected. Stage C adds 2 store tests + 8 component tests:

- 2 new graphStore tests for the FrameworkNode + lazy MCPNode wiring
- 5 component tests per new node type × 8 = 40 new component tests
- Test for the animated-edge state machine: `tool_invoked` creates animated edge; `tool_result` transitions to non-animated + complete

Total Stage C new tests: ~43.

#### Test plan (Stage C)

`tests/unit/graphStore.test.ts` — **edited** (add):

1. **session_start: spawns FrameworkNode at root** — apply event, assert FrameworkNode with `framework:<name>` id is in `state.nodes`
2. **session_start: idempotent** — apply twice, assert one FrameworkNode
3. **tool_invoked with source 'mcp': lazily spawns parent MCPNode** — apply event with `source: 'mcp', server: 'github-mcp'`, assert MCPNode is in nodes + ToolNode is in nodes + edge MCP → Tool is in edges (NOT agent → Tool directly)
4. **tool_invoked with source 'mcp': second tool from same server reuses MCPNode** — apply two tool_invoked events with same `server`, assert one MCPNode + two ToolNodes + two edges from the single MCPNode
5. **tool_invoked: edge created with animated=true** — assert the edge in state.edges has `animated: true`
6. **tool_result: turns off animated flag** — apply tool_invoked then tool_result, assert the edge's `animated` is now false (or undefined)

`tests/unit/nodes/<NodeName>.test.tsx` — **new** (8 files, 5 tests each):

For each component:
1. **renders with all required data fields** — assert the test-id, name, secondary fields all rendered
2. **applies status class for active|complete|error** — three render cases
3. **has accessible aria-label** — assert label content
4. **has data-testid + data-status attributes** — for E2E selectability (Stage F)
5. **renders source/target Handles per node type** — FrameworkNode has source only; GapNode has target only; HITLNode has target only (input affordance — graph blocks here); PlanNode/TaskNode have both; etc.

Special cases:
- **GapNode test**: assert `data-kind` attribute distinguishes `tool_missing` vs `skill_missing`; assert `gap-pulse` keyframe class is applied
- **HITLNode test**: assert `role="alert"` + `aria-live="assertive"` attributes present
- **FrameworkNode test**: assert renders both framework name + model

#### Coverage target

- Workspace Rust: ≥80% (preserved)
- runtime-drone: ≥95% (preserved)
- runtime-main: ≥95% (preserved)
- src-tauri: 50% patch gate (preserved)
- src/ frontend: ≥80% with **graphStore.ts ≥95%** (preserved primitive treatment); each new node component ≥80% line via the 5-tests-per-component pattern

**Doc-to-CI invariant.** Stage C does not add OS-call wrappers; coverage is end-to-end testable. No new exclusions to the codecov path-based override map. If component-test coverage falls below 80% during Stage C, surface in the retro before reducing the gate.

### C.5 CLI Prompt

```xml
<work_stage_prompt id="M03.C">
  <context>
    Stage C of M03 (Live Graph). Eight remaining spec §3 node types
    (MCP, Gap, HITL, Plan, Task, Verify, Hook, Framework) + animated
    edges (tool calls) + dashed edges (skill loads) + full color-
    encoding palette. graphStore wiring extended only for events the
    v0.1 schema emits (session_start → FrameworkNode; tool_invoked
    with source='mcp' → lazy MCPNode); Gap/HITL/Plan/Task/Verify/Hook
    components ship as renderable but their applyEvent handlers land
    at M4+ when those events join the schema. Builds on Stage B's
    React Flow + Zustand foundation. Stage D does not start until
    Stage C's commit is on the milestone branch.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M03-live-graph.md (Stage C sections C.1–C.4)</file>
    <file>agent-runtime-spec.md §3 (Node Types + Behavior + Visual Design)</file>
    <file>docs/MVP-v0.1.md §M3</file>
    <file>docs/gotchas.md (#21 #25 #26 #27)</file>
  </read_first>

  <read_reference>
    <file purpose="Stage B AgentNode archetype to mirror for the 8 new node components">src/components/nodes/AgentNode.tsx</file>
    <file purpose="Zustand store + applyEvent shape from Stage B (extending here)">src/lib/graphStore.ts</file>
    <file purpose="GraphCanvas nodeTypes map being extended (3 → 11)">src/components/GraphCanvas.tsx</file>
    <file purpose="Stage B's CSS color encoding for AgentNode — pattern to extend across 8 new types">src/styles.css</file>
    <file purpose="component test pattern from Stage B (5 tests per node)">tests/unit/nodes/AgentNode.test.tsx</file>
    <file purpose="graphStore test fixtures from Stage B (synthetic-state pattern for event-less node types)">tests/unit/graphStore.test.ts</file>
  </read_reference>

  <read_prior_stages>
    <retrospective section="[END] Decisions for the next stage">docs/build-prompts/retrospectives/M03.A-retrospective.md</retrospective>
    <retrospective section="[END] Decisions for the next stage">docs/build-prompts/retrospectives/M03.B-retrospective.md</retrospective>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M03-live-graph.md" section="C.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M03-live-graph.md" section="C.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M03-live-graph.md" section="Key constraints"/>

  <gates milestone="M03"/>

  <self_correction_budget>3</self_correction_budget>

  <gotchas>
    <trap>Stage C ships 8 components + edge animation + 2 graphStore wirings (FrameworkNode + lazy MCPNode). Resist scope creep into M4 territory: the six event-less node types (Gap, HITL, Plan, Task, Verify, Hook) ship as components only. Their applyEvent handlers land at M4 (plan/task/verify/hook events) and M5 (gap events) when the schema gains those events.</trap>
    <trap>Synthetic-state testing pattern for the six event-less components: Vitest tests pass populated state directly into `<ReactFlow nodes={...}>` rather than dispatching events through the store. This verifies render correctness without manufacturing events the schema doesn't yet emit. The pattern is locked here so M4+ extends without fighting the test infrastructure.</trap>
    <trap>Exhaustive `applyEvent` discriminated union: TS's `_exhaustive: never` check at the default branch protects against silent drift. When M4+ adds new event variants, the compile error in graphStore.ts is the forcing function. Don't suppress with `as never` or `@ts-ignore`.</trap>
    <trap>nodeTypes map (in GraphCanvas.tsx) MUST stay defined at module level even after growing to 11 entries. The Stage B trap re-applies: inline definition triggers per-render remount of every node type — kills the streaming UX especially noticeable now with 11 types.</trap>
    <trap>HITLNode + GapNode have ARIA-specific requirements per WAI APG: HITLNode is `role="alert"` + `aria-live="assertive"` (blocking input affordance); GapNode pulses via CSS keyframe (visual) but doesn't need its own ARIA role beyond standard graph-node a11y. Don't apply alert role to GapNode (it's not a request for response — it's a gap visualization).</trap>
    <trap>Color encoding palette is in `src/styles.css` `:root` CSS custom properties (--node-active, --node-complete, --node-error, --node-gap, --node-hitl). Don't hardcode the hex values per-component-rule — use `var(--node-...)`. This keeps the palette consistent and Stage D's token-spend visualization (which adjusts per-node visual weight, not color) doesn't have to fight per-component overrides.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT extend `schemas/event.v1.json` with M4+ event variants in Stage C. Stage A locked the v0.1 schema at 10 variants; M4 owns the schema bump (per schemas/README.md minor in-place bump policy). Stage C touches only the renderer.</warning>
    <warning>DO NOT add custom edge components in Stage C. React Flow v12's built-in `Edge.animated: true` + CSS keyframes is the chosen mechanism (per Decision #2 in C.1). Custom edge components are deferred to M4+ if a per-edge-type need surfaces.</warning>
    <warning>DO NOT wire PlanNode/TaskNode to plan-state data in Stage C. M4 ships the plan + task primitives; until then, component tests use synthetic placeholder data.</warning>
    <warning>DO NOT touch SetupPanel, SmokeButton, ipc.ts, or App.tsx unless absolutely required — preserved verbatim from Stage B (which itself preserved them from M02).</warning>
  </execution_warnings>

  <time_box estimate_hours="5.5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage D: which inspector-panel layout (right-rail vs bottom-drawer); whether the 11-node-type CSS color palette holds up against Stage D's token-spend node-weight scaling; whether the synthetic-state testing pattern is durable for the six event-less components or needs refactor before M4 wires events; whether any of the eight new components need refactor for reuse vs duplication of base styles.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M03-live-graph.md" section="C.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results (each gate, pass/fail, key numbers including frontend coverage on the 8 new components ≥80%)</item>
    <item>retrospective (filled-in [END] section with three-axis scoring + verdict + decisions for Stage D + [END] Coverage holdouts subsection)</item>
    <item>draft commit message from M03-live-graph.md C.6 Commit Message section</item>
    <item>screenshot or paste of the rendered graph after smoke-test run, showing FrameworkNode root + AgentNode (with color encoding active → complete transition)</item>
    <item>explicit statement: "Stage M03.C is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### C.6 Commit Message

```
feat(renderer): M03 Stage C — 8 remaining node types + animated edges + colors

Lights up the rest of spec §3's node-type set. After Stage C, all 11
node types ship as renderable components: AgentNode, ToolNode,
SkillNode (Stage B) + MCPNode, GapNode, HITLNode, PlanNode, TaskNode,
VerifyNode, HookNode, FrameworkNode (this stage). graphStore.applyEvent
extended for the two events that already exist in the v0.1 schema:
session_start → FrameworkNode (graph root); tool_invoked with
source='mcp' → lazy parent MCPNode. The remaining six components
(Gap, HITL, Plan, Task, Verify, Hook) ship as renderable but their
event-driven wiring lands at M4 (plan/task/verify/hook events) and
M5 (gap events) when the schema gains those variants.

8 new components in src/components/nodes/. Each follows the Stage B
AgentNode archetype with type-appropriate data + Handle/Position +
ARIA + data-testid pattern. Two specialize:
- HITLNode: role="alert" + aria-live="assertive" (per WAI APG;
  blocking input affordance).
- GapNode: data-kind attribute + CSS gap-pulse keyframe per spec §3
  Behavior ("GapNode appears immediately on tool_missing").

Edge layer:
- React Flow v12's built-in Edge.animated: true + CSS keyframes
  (no custom Edge component; deferred to M4+ if per-edge-type need
  surfaces).
- Animated dashed flow on tool_invoked → tool_result transitions;
  static dashed edges on skill_loaded.
- Edge state machine codified in graphStore: animated=true on
  tool_invoked; cleared on tool_result.

CSS palette (spec §3 Visual Design Principles):
- :root CSS custom properties --node-{active,complete,error,gap,hitl}
  for consistent color encoding across all 11 node types.
- gap-pulse keyframe for GapNode visual prominence.
- Dash-flow keyframe for animated edges; dashed-edge style for skills.

Tests (~43 new):
- 2 new graphStore tests (FrameworkNode + lazy MCPNode + animated-
  edge lifecycle).
- 5 component tests per new node × 8 = 40 component tests.
- Synthetic-state testing pattern for the 6 event-less components
  (Gap, HITL, Plan, Task, Verify, Hook): tests pass populated state
  directly into <ReactFlow> rather than dispatching events. Pattern
  is locked here so M4+ extends without fighting the infrastructure.

Coverage:
- graphStore.ts stays ≥95% (primitive treatment per CLAUDE.md §5).
- Each new node component ≥80% line via 5-tests-per-component pattern.
- Workspace + runtime-drone + runtime-main + src-tauri gates all
  preserved (no Rust changes in Stage C).

Refs: M03-live-graph.md §C; agent-runtime-spec.md §3 Node Types +
Behavior + Visual Design; docs/MVP-v0.1.md §M3; docs/gotchas.md
#21 + #27 (Vitest+RTL DOM-ref staleness, applies to component tests
that await render).

https://claude.ai/code
```

---
<!-- ============================================================ -->
<!-- STAGE D — Inspector + token-spend node weight + zoom/pan      -->
<!-- ============================================================ -->

## Stage D — Click-to-inspect side panel + token-spend visualization + zoom/pan

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens. If any claim is stale, update this section in `M03-live-graph.md` BEFORE pasting Stage D's CLI prompt to the fresh session.

- <https://github.com/dagrejs/dagre> — confirm `@dagrejs/dagre` is the maintained fork (vs the unmaintained `dagre` package); current stable version; React Flow v12 layout-integration pattern (use `dagre.graphlib.Graph` to compute positions, then assign back to React Flow `Node.position`)
- <https://reactflow.dev/learn/layouting> — confirm v12's layouting API + `useNodesState` / `useEdgesState` hooks for layout integration; whether layout should be in the store (Zustand) or computed via React Flow hooks
- <https://reactflow.dev/api-reference/components/minimap> — confirm `<MiniMap>` props in v12 (color-encoding callback, position, click-to-pan behavior); a11y options
- <https://reactflow.dev/api-reference/components/controls> — confirm `<Controls>` (already in Stage B's GraphCanvas) supports keyboard navigation, fit-view, zoom-in/out
- <https://docs.anthropic.com/en/api/messages-streaming#message_delta> — confirm Anthropic's `message_delta` SSE event payload includes `usage.input_tokens` + `usage.output_tokens` (the source of token counts the AgentSdk surfaces; M02 already consumed this in `anthropic_sse.rs`, but Stage D wires the count forward to AgentEvent variants)
- <https://www.w3.org/WAI/ARIA/apg/patterns/dialog/> — confirm ARIA dialog pattern recommendations for the InspectorPanel (focus trap when open, ESC to close, role="dialog", aria-modal)

### D.1 Problem Statement

Stages B + C ship the graph as a renderable surface — 11 node types, animated edges, color encoding. The user can see the runtime as it happens but cannot drill into any node. Stage D adds three pieces that make the graph **interactive**:

1. **Click-to-inspect side panel.** Per spec §3 Behavior ("Click any node for full VDR trace, input/output, timing"). The panel renders alongside `<GraphCanvas>` (right-rail layout); shows the selected node's data + a chronological list of events that contributed to that node's state. Stage E's VDR projection populates the "decision history" subsection; Stage D's panel handles M02-shipped event data.

2. **Token-spend visualization (node weight).** Per spec §3 Visual Design ("Token spend shown as node weight — larger spend = visually larger node"). Each AgentNode and ToolNode tracks cumulative input + output tokens; CSS-based scaling makes higher-spend nodes visually larger. Triggers a Rust-side schema bump: `schemas/event.v1.json` extends `tool_result` with optional `tokens_in?` + `tokens_out?` fields and `agent_complete` with `tokens_total?`; `runtime_core::AgentEvent` matches; AgentSdk's `EventPipeline` populates them from the existing `ProviderEvent` token data (M02.B already consumes Anthropic's `message_delta.usage`; Stage D surfaces it through the SDK→renderer pipeline).

3. **Zoom/pan/select controls + layout.** Stage B added React Flow's built-in `<Controls>`; Stage D extends with `<MiniMap>` (necessary at 11 node types and growing) plus a proper layout algorithm (`@dagrejs/dagre`) that replaces Stage B's naive horizontal-stagger. Layout runs after every store-state change but is debounced — re-running dagre on every event would freeze the UI.

The cardinal architectural decision: **the InspectorPanel reads from the store, never holds its own state.** Selected-node id is in the store (`selectedNodeId` from Stage B); the panel subscribes to the selected node's data slice via Zustand selectors. Closing the panel clears `selectedNodeId`. This matches the Stage B pattern (single source of truth, components subscribe via selectors) and forward-compatible with Stage E's persistence (selected-node state replays from the event log).

The second decision: **layout runs in `useEffect` inside `<GraphCanvas>`, not inside the store.** Layout is a *visualization concern*, not state — the store's nodes have logical positions (the staggered defaults from Stage B); the layout pass overlays presentation positions. Storing layout positions in Zustand would conflate concerns and complicate persistence. Per React Flow's layouting guide, the recommended pattern is `useEffect(() => { layoutNodes(nodes, edges); setNodes(...); }, [nodes.length])` — layout reruns on node-count changes, debounced to coalesce rapid event bursts.

The third decision: **token weight via CSS `transform: scale()`, not JS-computed font-size.** Per spec §3 Visual Design ("larger spend = visually larger node"). Pure CSS keeps the renderer fast; the scale factor is computed from cumulative tokens (`scale = clamp(0.8, 1 + tokens/1000, 1.5)`). Disabled in tests via a `data-token-scale-disabled` attribute set by Vitest setup so visual regressions don't trip the test suite.

The fourth decision: **schema bump strategy.** `schemas/event.v1.json` is currently the only `v1.x` schema in the runtime that the Rust side does NOT generate from. Stage A added it to the TS codegen list (`xtask` regenerates `src/types/agent_event.ts`); the Rust `AgentEvent` enum stays hand-written for M03. Stage D's schema bump adds optional fields to existing variants; both the Rust enum and the TS regenerated types must match. The xtask drift check enforces TS parity; the Rust side relies on hand-written maintenance — flag in Stage E retro for "extend Rust typify codegen to include event.v1.json" carry-forward to M04.

**One-line success criterion:** the M02 smoke test produces a graph where (a) clicking the AgentNode opens an InspectorPanel showing the smoke-agent's full data + the events received, (b) the AgentNode visually scales after `agent_complete` fires (Haiku 4.5 reports ~10–20 tokens for the smoke prompt; barely visible scaling but the mechanism is wired), (c) `<MiniMap>` renders in the corner with a click-to-pan affordance, (d) layout via dagre produces a coherent root → smoke-agent edge instead of Stage B's hardcoded position; renderer Vitest coverage on `InspectorPanel.tsx` ≥80%, `layout.ts` ≥95% (treated as primitive).

**New artifacts:**
- `src/components/InspectorPanel.tsx` — right-rail panel; subscribes to `selectedNodeId` + node-data selectors; renders ARIA-compliant dialog
- `src/lib/layout.ts` — dagre wrapper; `layoutGraph(nodes, edges) => { nodes }` pure function
- 2 new test files at `tests/unit/components/InspectorPanel.test.tsx` and `tests/unit/lib/layout.test.ts`

### D.2 Files to Change

| File | Change |
|---|---|
| `src/components/InspectorPanel.tsx` | **New** — right-rail panel; ARIA dialog; renders selected node's full data via Zustand selectors; ESC-to-close; focus trap |
| `tests/unit/components/InspectorPanel.test.tsx` | **New** — render selected/unselected states; ESC closes; ARIA attrs; focus trap behavior |
| `src/lib/layout.ts` | **New** — `layoutGraph(nodes, edges) => Node[]` pure function over `@dagrejs/dagre`; takes the GraphNode + GraphEdge types from graphStore; returns nodes with computed `position` |
| `tests/unit/lib/layout.test.ts` | **New** — empty graph returns []; single-node graph returns one positioned node; parent-child edge produces top-down layout; throws cleanly on cycles (dagre handles cycles but our schema doesn't have them; test surfaces the contract) |
| `src/lib/graphStore.ts` | **Edited** — extend `AgentNodeData` + `ToolNodeData` with `tokensIn: number` + `tokensOut: number` (default 0); extend `applyEvent` for `tool_result.tokens_in` + `tool_result.tokens_out` + `agent_complete.tokens_total` (additive — events without the optional fields no-op the token tracking) |
| `tests/unit/graphStore.test.ts` | **Edited** — add 4 new tests: tool_result with tokens updates AgentNode + ToolNode token totals; agent_complete with tokens_total matches sum of contributing tool_results; missing token fields don't crash; cumulative token totals across multiple tool_results |
| `src/components/GraphCanvas.tsx` | **Edited** — add `<MiniMap>` from `@xyflow/react`; wire `useEffect` layout pass via `layoutGraph(nodes, edges)`; render alongside `<InspectorPanel>` (side-by-side via flexbox in `<App>`) |
| `src/components/nodes/AgentNode.tsx` | **Edited** — add `style={{ transform: `scale(${tokenScale(data.tokensIn + data.tokensOut)})` }}` (or via inline CSS custom property `--token-scale`); add unit-test setup-time disable attr |
| `src/components/nodes/ToolNode.tsx` | **Edited** — same scaling pattern |
| `src/styles.css` | **Edited** — add `.inspector-panel` styles (right-rail layout; ARIA visible-on-focus); add `transform-origin: center` for scaling stability; add MiniMap container override; the `@keyframes` already in Stage C are preserved |
| `src/App.tsx` | **Edited** — wrap `<GraphCanvas>` + `<InspectorPanel>` in a flexbox container so they sit side-by-side; preserve `<SetupPanel>` + `<SmokeButton>` above |
| `tests/unit/App.test.tsx` | **Edited** — add 2 tests: clicking an AgentNode opens InspectorPanel; ESC closes it |
| `package.json` | **Edited** — add `@dagrejs/dagre ^1.x` to `dependencies` |
| `package-lock.json` | **Regenerated** via `npm install` |
| `schemas/event.v1.json` | **Edited** — extend `tool_result` with optional `tokens_in?: integer` + `tokens_out?: integer`; extend `agent_complete` with optional `tokens_total?: integer`. Additive minor in-place bump per `schemas/README.md` versioning policy; `$id` URL unchanged |
| `src/types/agent_event.ts` | **Regenerated** via `cargo xtask regenerate-types` after schema edit lands; the new optional fields appear |
| `crates/runtime-core/src/event.rs` | **Edited** — extend the `AgentEvent::ToolResult` variant with `tokens_in: Option<u64>` + `tokens_out: Option<u64>`; extend `AgentEvent::AgentComplete` with `tokens_total: Option<u64>`; update serde derives to skip-if-none on output (`#[serde(skip_serializing_if = "Option::is_none")]`) |
| `crates/runtime-main/src/sdk/event_pipeline.rs` | **Edited** — populate the new fields from `ProviderEvent` data (M02.B already consumes Anthropic's `message_delta.usage` into `CostBreakdown`; Stage D surfaces the per-call counts forward to `AgentEvent::ToolResult` and `AgentEvent::AgentComplete`) |
| `crates/runtime-main/tests/sdk_event_translation.rs` | **Edited** — extend translation tests to assert tokens_in/tokens_out flow through correctly |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` entry under "Added — M03.D" |

### D.3 Detailed Changes

#### `schemas/event.v1.json` — additive token fields

Schema bump: `tool_result` gains optional `tokens_in?: integer` + `tokens_out?: integer`; `agent_complete` gains optional `tokens_total?: integer`. Per `schemas/README.md`: additive optional fields are a minor in-place bump (no `$id` change). Stage A's xtask drift check + the new `cargo xtask regenerate-types` will produce updated `src/types/agent_event.ts` after this edit.

```json
// schemas/event.v1.json — relevant variants (additive fields shown)
"oneOf": [
  // ... other variants unchanged ...
  {
    "type": "object",
    "required": ["type", "agentId", "toolId", "output"],
    "properties": {
      "type":     { "const": "tool_result" },
      "agentId":  { "type": "string" },
      "toolId":   { "type": "string" },
      "output":   { "type": "object" },
      "durationMs": { "type": "integer", "minimum": 0 },
      "tokensIn":  { "type": "integer", "minimum": 0 },
      "tokensOut": { "type": "integer", "minimum": 0 }
    }
  },
  {
    "type": "object",
    "required": ["type", "agentId", "result"],
    "properties": {
      "type":         { "const": "agent_complete" },
      "agentId":      { "type": "string" },
      "result":       { "type": "string" },
      "tokensTotal":  { "type": "integer", "minimum": 0 }
    }
  }
]
```

#### `crates/runtime-core/src/event.rs` — match the schema

Hand-edit (event.rs is hand-written; not in the typify codegen list). Extend the variants with the new optional fields:

```rust
pub enum AgentEvent {
    // ...
    ToolResult {
        agent_id: String,
        tool_id: String,
        output: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tokens_in: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tokens_out: Option<u64>,
    },
    AgentComplete {
        agent_id: String,
        result: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tokens_total: Option<u64>,
    },
    // ...
}
```

The serde `skip_serializing_if = "Option::is_none"` keeps backward-compatible JSON output: M02-era event consumers see the same shape; M03-era consumers see the new fields when populated.

#### `crates/runtime-main/src/sdk/event_pipeline.rs` — surface tokens

The `EventPipeline::translate` function currently produces `AgentEvent::ToolResult` from `ProviderEvent::ToolResult`. Extend to populate the token fields from the existing per-call cost-breakdown tracking. The fresh session reads the existing `CostBreakdown` accumulator usage in `agent_sdk.rs` (Stage M02.B) and threads the per-tool-call counts forward.

For `AgentEvent::AgentComplete`: M02's `agent_sdk.rs` already aggregates total tokens per session (for cost reporting). Surface that aggregate in the `tokens_total` field at completion time.

#### `src/lib/layout.ts` — dagre wrapper

```typescript
import dagre from '@dagrejs/dagre';
import type { GraphNode, GraphEdge } from './graphStore';

const NODE_WIDTH = 180;
const NODE_HEIGHT = 60;

/**
 * Run dagre layout over a snapshot of nodes + edges; return new nodes
 * with computed positions. Pure function — no side effects, deterministic
 * for a given input.
 *
 * Stage D ships top-down hierarchical layout (rankdir='TB'); Stage E may
 * extend with per-graph-shape selectors.
 */
export function layoutGraph(
  nodes: GraphNode[],
  edges: GraphEdge[],
): GraphNode[] {
  if (nodes.length === 0) {
    return nodes;
  }

  const g = new dagre.graphlib.Graph();
  g.setDefaultEdgeLabel(() => ({}));
  g.setGraph({ rankdir: 'TB', nodesep: 50, ranksep: 80 });

  for (const node of nodes) {
    g.setNode(node.id, { width: NODE_WIDTH, height: NODE_HEIGHT });
  }
  for (const edge of edges) {
    g.setEdge(edge.source, edge.target);
  }

  dagre.layout(g);

  return nodes.map((node) => {
    const layoutNode = g.node(node.id);
    return {
      ...node,
      position: { x: layoutNode.x - NODE_WIDTH / 2, y: layoutNode.y - NODE_HEIGHT / 2 },
    };
  });
}
```

Tested via 4 unit tests in `tests/unit/lib/layout.test.ts` (per D.4).

#### `src/components/InspectorPanel.tsx` — ARIA dialog right-rail

```typescript
import { useEffect, useRef } from 'react';
import { useGraphStore } from '../lib/graphStore';

export function InspectorPanel() {
  const selectedNodeId = useGraphStore((s) => s.selectedNodeId);
  const selectedNode = useGraphStore((s) =>
    s.selectedNodeId ? s.nodes.find((n) => n.id === s.selectedNodeId) : null,
  );
  const selectNode = useGraphStore((s) => s.selectNode);
  const panelRef = useRef<HTMLDivElement>(null);

  // Focus management per WAI APG dialog pattern.
  useEffect(() => {
    if (selectedNodeId && panelRef.current) {
      panelRef.current.focus();
    }
  }, [selectedNodeId]);

  // ESC closes per ARIA dialog pattern.
  useEffect(() => {
    if (!selectedNodeId) return undefined;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') selectNode(null);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [selectedNodeId, selectNode]);

  if (!selectedNodeId || !selectedNode) {
    return null;
  }

  return (
    <aside
      ref={panelRef}
      className="inspector-panel"
      role="dialog"
      aria-modal="false"
      aria-label="node inspector"
      tabIndex={-1}
      data-testid="inspector-panel"
    >
      <header className="inspector-panel__header">
        <h2>{selectedNode.type} node</h2>
        <button
          type="button"
          onClick={() => selectNode(null)}
          aria-label="close inspector"
        >
          ×
        </button>
      </header>
      <pre className="inspector-panel__data">
        {JSON.stringify(selectedNode.data, null, 2)}
      </pre>
      {/* Stage E adds VDR-correlated decision history here. */}
    </aside>
  );
}
```

Tested via 6 tests (per D.4).

#### `src/components/GraphCanvas.tsx` — MiniMap + layout effect

Extend Stage B/C's GraphCanvas:

```typescript
import { ReactFlow, Background, Controls, MiniMap, useNodesState } from '@xyflow/react';
import { useEffect, useMemo } from 'react';
import { useGraphStore } from '../lib/graphStore';
import { layoutGraph } from '../lib/layout';
// ... existing imports for nodeTypes ...

export function GraphCanvas() {
  const storeNodes = useGraphStore((s) => s.nodes);
  const edges = useGraphStore((s) => s.edges);
  const selectNode = useGraphStore((s) => s.selectNode);

  // Run dagre layout when node count changes. Debounce for rapid event bursts.
  const layouted = useMemo(() => layoutGraph(storeNodes, edges), [storeNodes.length, edges.length]);

  return (
    <div className="graph-canvas" data-testid="graph-canvas">
      <ReactFlow
        nodes={layouted}
        edges={edges}
        nodeTypes={nodeTypes}
        fitView
        onNodeClick={(_, node) => selectNode(node.id)}
        onPaneClick={() => selectNode(null)}
      >
        <Background />
        <Controls />
        <MiniMap nodeStrokeWidth={3} pannable zoomable />
      </ReactFlow>
    </div>
  );
}
```

#### `src/components/nodes/AgentNode.tsx` + `ToolNode.tsx` — token weight

Extend Stage B/C's nodes with token-weight scaling:

```typescript
function tokenScale(totalTokens: number): number {
  // clamp(0.8, 1 + tokens/1000, 1.5)
  return Math.max(0.8, Math.min(1.5, 1 + totalTokens / 1000));
}

export function AgentNode({ data }: NodeProps<AgentNodeData>) {
  const scale = tokenScale(data.tokensIn + data.tokensOut);
  return (
    <div
      className={`agent-node agent-node--${data.status}`}
      data-testid={`agent-node-${data.agentId}`}
      data-status={data.status}
      style={{ transform: `scale(${scale})`, transformOrigin: 'center' }}
      aria-label={`agent ${data.agentName} (${data.status})`}
    >
      {/* ... existing markup ... */}
    </div>
  );
}
```

ToolNode follows the same pattern. Tests verify the `transform` style attr applies; visual regression is out-of-scope for unit tests (covered by Stage F E2E if needed).

#### `src/App.tsx` — side-by-side layout

Extend Stage B's App.tsx:

```typescript
return (
  <main>
    <h1>Agent Runtime — M03 live graph</h1>
    <SetupPanel onSave={handleSetKey} />
    <SmokeButton disabled={!hasKey || running} onClick={handleSmoke} />
    {error && <p className="error">{error}</p>}
    <div className="graph-layout">
      <GraphCanvas />
      <InspectorPanel />
    </div>
  </main>
);
```

CSS for `.graph-layout`: flexbox row; GraphCanvas takes `flex: 1`; InspectorPanel takes `flex: 0 0 360px` when visible (collapses when no node selected via `null` return).

### D.4 Tests

#### Pedantic-pass preflight

Same checklist as Stage B + C. Stage D adds Rust changes (event.rs + event_pipeline.rs); apply `docs/gotchas.md` #21 clippy traps to those files specifically:

- [ ] `derive_partial_eq_without_eq` — `serde_json::Value` containment (the `output` field on `ToolResult`); already has `#[allow]` from M02; preserve
- [ ] `unused_async` — N/A (event translation is sync)
- [ ] `default_trait_access` — N/A
- [ ] `match_wildcard_for_single_variants` — N/A
- [ ] `cast_precision_loss` / `suboptimal_flops` — token scaling is u64 → f64; Stage D's `tokenScale` is in TS not Rust, but verify any Rust-side aggregation uses `u64::saturating_add` for safety
- [ ] `struct_excessive_bools` — N/A
- [ ] `missing_const_for_fn` — N/A

#### Default test plan for stages adding a new safety primitive

`layout.ts` is treated as a primitive (≥95% line) — single source of layout truth; bug here breaks the visual layer. `InspectorPanel.tsx` is normal-coverage (≥80%).

Rust-side: `event_pipeline.rs` extension is exercised by `crates/runtime-main/tests/sdk_event_translation.rs` extensions; runtime-main coverage gate (≥95%) preserved.

#### Test plan (Stage D)

`tests/unit/lib/layout.test.ts` — 4 tests:

1. **empty graph returns empty** — `layoutGraph([], [])` returns `[]`
2. **single-node graph returns one positioned node** — single node gets a position (any non-NaN coords)
3. **parent-child edge produces top-down layout** — assert child's `y > parent.y`
4. **deterministic for same input** — call twice, assert identical positions

`tests/unit/components/InspectorPanel.test.tsx` — 6 tests:

1. **returns null when no selectedNodeId** — `render(...)` produces empty document
2. **renders panel when selectedNodeId is set** — populate store, render, assert `data-testid="inspector-panel"` present
3. **renders selected node's data as JSON** — assert `<pre>` content includes node data fields
4. **ESC key clears selectedNodeId** — fire ESC keydown, assert `selectNode(null)` was called (via store assertion)
5. **close button clears selectedNodeId** — click `aria-label="close inspector"`, assert
6. **has ARIA dialog attributes** — assert `role="dialog"`, `aria-label`, `aria-modal="false"` (non-modal — graph stays interactable behind the panel)

`tests/unit/graphStore.test.ts` — **edited** (4 new tests):

1. **tool_result with tokens updates ToolNode's tokensIn + tokensOut** — apply tool_result with tokens, assert ToolNode data fields
2. **tool_result with tokens accumulates AgentNode totals** — apply tool_result, assert parent AgentNode's tokensIn/tokensOut += tool_result counts
3. **agent_complete with tokens_total updates AgentNode totals** — final aggregate
4. **events without optional token fields don't crash** — apply tool_result without tokens fields, assert state unchanged in token slots (defaults to 0)

`tests/unit/App.test.tsx` — **edited** (2 new tests):

1. **clicking AgentNode opens InspectorPanel** — render App, fire `agent_spawned`, click the node (find by data-testid), assert panel appears
2. **ESC closes InspectorPanel** — extend the previous test, fire ESC, assert panel hides

`crates/runtime-main/tests/sdk_event_translation.rs` — **edited** (2 new tests):

1. **tool_result translation surfaces tokens_in + tokens_out** — feed a `ProviderEvent::ToolResult` with token data; assert the resulting `AgentEvent::ToolResult` has the matching fields
2. **agent_complete translation surfaces tokens_total** — feed completion event; assert tokens_total flows through

#### Coverage target

- Workspace Rust: ≥80% (preserved)
- runtime-drone: ≥95% (preserved; no Rust change in Stage D's drone scope)
- runtime-main: ≥95% (preserved; the event_pipeline.rs extension is small + covered by the 2 new translation tests)
- src-tauri: 50% patch gate (preserved)
- src/ frontend: ≥80% with **graphStore.ts ≥95%** (preserved primitive treatment) + **layout.ts ≥95%** (new primitive); InspectorPanel.tsx ≥80%

**Doc-to-CI invariant.** Stage D does not add OS-call wrappers; coverage stays end-to-end testable. No new exclusions to the codecov path-based override map.

### D.5 CLI Prompt

```xml
<work_stage_prompt id="M03.D">
  <context>
    Stage D of M03 (Live Graph). Click-to-inspect side panel + token-
    spend visualization (node weight) + zoom/pan + dagre layout. Adds
    InspectorPanel + layout.ts; bumps schemas/event.v1.json with optional
    token fields; extends runtime-core::AgentEvent + runtime-main's
    EventPipeline to surface tokens from M02.B's existing ProviderEvent
    cost tracking. Builds on Stages B + C's React Flow + Zustand
    foundation + 11 node types. Stage E does not start until Stage D's
    commit is on the milestone branch.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M03-live-graph.md (Stage D sections D.1–D.4)</file>
    <file>agent-runtime-spec.md §3 (Behavior + Visual Design Principles)</file>
    <file>docs/MVP-v0.1.md §M3</file>
    <file>schemas/README.md (additive minor in-place bump policy)</file>
    <file>docs/gotchas.md (#21 #25 #27)</file>
  </read_first>

  <read_reference>
    <file purpose="Stage B/C graphStore + applyEvent shape (extending here for token tracking)">src/lib/graphStore.ts</file>
    <file purpose="Stage B/C GraphCanvas (extending with MiniMap + layout effect)">src/components/GraphCanvas.tsx</file>
    <file purpose="Stage B AgentNode archetype (extending with token-weight scaling)">src/components/nodes/AgentNode.tsx</file>
    <file purpose="M02 event_pipeline.rs translation pattern (extending to surface token fields)">crates/runtime-main/src/sdk/event_pipeline.rs</file>
    <file purpose="M02 AgentSdk's CostBreakdown aggregation (the existing source of token data)">crates/runtime-main/src/sdk/agent_sdk.rs</file>
    <file purpose="ARIA dialog pattern reference for InspectorPanel">https://www.w3.org/WAI/ARIA/apg/patterns/dialog/</file>
  </read_reference>

  <read_prior_stages>
    <retrospective section="[END] Decisions for the next stage">docs/build-prompts/retrospectives/M03.A-retrospective.md</retrospective>
    <retrospective section="[END] Decisions for the next stage">docs/build-prompts/retrospectives/M03.B-retrospective.md</retrospective>
    <retrospective section="[END] Decisions for the next stage">docs/build-prompts/retrospectives/M03.C-retrospective.md</retrospective>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M03-live-graph.md" section="D.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M03-live-graph.md" section="D.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M03-live-graph.md" section="Key constraints"/>

  <gates milestone="M03"/>

  <self_correction_budget>3</self_correction_budget>

  <gotchas>
    <trap>Stage D's schema bump on schemas/event.v1.json must run xtask regenerate-types AFTER editing schemas + event.rs. Verify drift check passes (`cargo xtask regenerate-types --check` exits zero) before commit. Without this, src/types/agent_event.ts diverges from the schema and CI fails.</trap>
    <trap>schemas/event.v1.json is NOT in the typify Rust codegen list (only the TS list per Stage A). The Rust event.rs is hand-written and must be manually updated to match the schema bump. Flag in retro Decisions for M04: extend xtask Rust typify list to include `event` so this drift class can't recur.</trap>
    <trap>Layout in useEffect, NOT in the store. React Flow's layouting guide (per WEBCHECK) says layout is a visualization concern; storing computed positions in Zustand conflates concerns and breaks Stage E's persistence. Pure-function `layoutGraph` + `useMemo` keeps the store clean.</trap>
    <trap>Token-weight scaling via CSS transform: scale(), NOT JS-computed font-size. CSS keeps the renderer fast; the scale factor is computed inline. Tests use `data-token-scale-disabled` setup attr to avoid visual-regression flakes.</trap>
    <trap>InspectorPanel is `aria-modal="false"` (non-modal) — graph stays interactable behind the panel. Per WAI APG dialog pattern, modal dialogs trap focus + dim the background; M03's inspector is informational-not-blocking. Don't make it modal.</trap>
    <trap>Existing graphStore tests must pass unchanged after the token-tracking edits. The new token fields are ADDITIVE on AgentNodeData/ToolNodeData; existing tests that don't populate them rely on default 0. Verify by running the original 12 graphStore tests post-edit; failures here indicate non-additive changes.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT switch to elk-js or another layout library. @dagrejs/dagre is the chosen library per Decision #4 in D.1; Stage D's layout.ts pattern is locked. M04+ may revisit if dagre proves insufficient.</warning>
    <warning>DO NOT make InspectorPanel modal (aria-modal="true" + focus trap that prevents tabbing out). The graph stays interactable; the panel is informational. Per WAI APG dialog pattern + Decision #1 in D.1.</warning>
    <warning>DO NOT extend the schema with non-token fields in Stage D. The Stage D schema bump scope is exactly two variants × three new optional fields. Other extensions (e.g., MCP server-id surfacing on tool_invoked) wait for M06 which owns MCP integration.</warning>
    <warning>DO NOT touch SetupPanel, SmokeButton, ipc.ts, or the M02 keychain code — preserved verbatim.</warning>
  </execution_warnings>

  <time_box estimate_hours="4.5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage E: whether the dagre layout's per-event recomputation is performant enough for Stage E's persistence-replay (rendering 100+ nodes at session reload); whether the InspectorPanel data-rendering pattern needs refactor for Stage E's VDR row addition; whether the token-scaling clamp(0.8, ..., 1.5) range works for the smoke test or needs adjustment; whether the schema bump → hand-edit Rust event.rs flow is brittle enough to push the M04 carry-forward (extend Rust typify list to include event.v1.json) into a hotfix PR before M04.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M03-live-graph.md" section="D.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results (each gate, pass/fail, key numbers including layout.ts ≥95% + InspectorPanel ≥80% + Rust drift check passing on the schema bump)</item>
    <item>retrospective (filled-in [END] section with three-axis scoring + verdict + decisions for Stage E + [END] Coverage holdouts subsection)</item>
    <item>draft commit message from M03-live-graph.md D.6 Commit Message section</item>
    <item>screenshot or paste of the rendered graph after smoke-test run, showing: (a) clicking AgentNode opens InspectorPanel; (b) MiniMap visible in corner; (c) AgentNode visually scaled per token count (subtle for ~20 tokens but the mechanism wired); (d) dagre layout produces top-down hierarchy</item>
    <item>explicit statement: "Stage M03.D is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### D.6 Commit Message

```
feat(renderer): M03 Stage D — inspector panel + token weight + dagre layout

Three pieces that make the graph interactive:
- Click-to-inspect side panel (per spec §3 Behavior). Right-rail
  layout; ARIA-compliant dialog (aria-modal="false" — non-modal so
  graph stays interactable); ESC + close-button to dismiss; subscribes
  to selectedNodeId via Zustand selectors (single source of truth
  pattern from Stage B preserved).
- Token-spend visualization (per spec §3 Visual Design). Each
  AgentNode + ToolNode tracks cumulative tokensIn + tokensOut; CSS
  transform: scale() applies via clamp(0.8, 1 + tokens/1000, 1.5).
  Triggers schema bump: tool_result + agent_complete gain optional
  token fields; runtime-core::AgentEvent matches; AgentSdk's
  EventPipeline surfaces tokens from M02.B's existing ProviderEvent
  cost tracking.
- Zoom/pan/select extension. <MiniMap> from @xyflow/react alongside
  Stage B's <Controls>; @dagrejs/dagre layout replaces Stage B's
  naive horizontal-stagger via pure-function layoutGraph(nodes, edges)
  in src/lib/layout.ts, called from useEffect in <GraphCanvas>.

Schema:
- schemas/event.v1.json bumped (additive minor in-place per
  schemas/README.md): tool_result gains optional tokens_in +
  tokens_out; agent_complete gains optional tokens_total. $id
  unchanged. xtask regenerate-types updates src/types/agent_event.ts.
- crates/runtime-core/src/event.rs hand-edited to match (event.rs
  is NOT in the typify codegen list per Stage A; flagged in retro
  Decisions as M04 carry-forward to extend the typify list).
- crates/runtime-main/src/sdk/event_pipeline.rs extended to surface
  tokens from M02.B's CostBreakdown into the new AgentEvent fields.

12 files in scope: 4 NEW (InspectorPanel + InspectorPanel.test +
layout.ts + layout.test), 8 EDIT (graphStore + GraphCanvas +
AgentNode + ToolNode + App + App.test + styles + schemas/event.v1.json
+ event.rs + event_pipeline.rs + sdk_event_translation.rs +
package.json + CHANGELOG).

~16 new tests: 4 layout tests; 6 InspectorPanel tests; 4 graphStore
token-tracking tests; 2 App-level interaction tests; 2 Rust event-
translation tests.

Coverage:
- layout.ts ≥95% line (new primitive — pure function).
- InspectorPanel.tsx ≥80% line (component-test pattern).
- graphStore.ts stays ≥95% (token-tracking added without dropping
  existing branches).
- runtime-main stays ≥95% (event_pipeline.rs extension covered by
  the 2 new translation tests).

Per-stage decisions (per Stage D retro):
- @dagrejs/dagre chosen over elk-js (smaller bundle; v0.1 sufficient).
- Layout in useEffect not in store (visualization vs state separation).
- CSS transform: scale() for token weight (perf > JS-computed sizing).
- aria-modal="false" on InspectorPanel (informational, not blocking).

Refs: M03-live-graph.md §D; agent-runtime-spec.md §3 Behavior + Visual
Design; schemas/README.md additive in-place bump; docs/gotchas.md
#21 + #27 + new entry M04 carry-forward (extend Rust typify list to
include event.v1.json).

https://claude.ai/code
```

---
<!-- ============================================================ -->
<!-- STAGE E — VDR projection + SQL inspector + graph persistence  -->
<!-- ============================================================ -->

## Stage E — VDR projection + SQL inspector + graph persistence

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens. If any claim is stale, update this section in `M03-live-graph.md` BEFORE pasting Stage E's CLI prompt to the fresh session.

- <https://www.sqlite.org/lang_select.html> — confirm SQLite SELECT statement syntax + restrictions; needed for the SQL inspector's allow-list validation (only SELECT permitted; DDL/DML/PRAGMA rejected)
- <https://github.com/rusqlite/rusqlite> — confirm rusqlite v0.32 (M02 pin) prepared-statement + column-iteration API; needed for the read-only query path that returns rows as JSON
- <https://www.sqlite.org/lang_keywords.html> — confirm reserved keywords (used in SQL parsing for the SELECT-only validator)
- <https://docs.rs/serde_rusqlite/latest/serde_rusqlite/> — evaluate whether the `serde_rusqlite` crate adds enough value over hand-rolled `Row → serde_json::Value` conversion for ~50 lines of read-only query handling. Decision: hand-roll (per CLAUDE.md §6 "no third-party dependencies without `cargo deny check` passing"; one-off conversion logic doesn't justify a new dep)
- <https://v2.tauri.app/develop/calling-rust/> — confirm Tauri 2.x command pattern for commands that take a `String` argument and return `Vec<Value>`; needed for the new `query_session_db` + `replay_session` commands

### E.1 Problem Statement

Stages B–D ship a graph that lives **only as long as the session window stays open**. Per spec §3 Behavior ("Graph state is persisted per session — reopen a session and the graph reconstructs") + MVP §M3 acceptance criteria ("Graph reconstructs after page reload (state from SQLite)"), the graph must persist. Plus per spec §2b ("VDR is a projection of signals 4 (decision) + 5 (verify)") and MVP §M3 ("populated table + simple SQL inspector"), Stage E delivers the VDR projection + a renderer-side SQL inspector for ad-hoc query of session state.

Three pieces, one stage:

1. **VDR projection (drone-side, continuous).** A new `runtime-drone::vdr` module reads signals from the existing `signals` table (M01-shipped) and writes correlated VDR rows to the `vdr` table. Idempotent: re-running over the same signals produces the same VDR row count (no duplicates). Triggered after each signal write (drone-internal — no IPC roundtrip per signal). Per spec §2b's projection model: signals are the source of truth, VDR is the read-optimized projection.

2. **SQL inspector (renderer + IPC).** A new `<SqlInspector>` component lets the user paste a SELECT statement and view results as a table. Renderer calls a new Tauri command `query_session_db(sql: String) -> Vec<JsonRow>`; main proxies to drone via a new `DroneCommand::QuerySessionDb`; drone validates the SQL is **SELECT-only** (no DDL/DML/PRAGMA — read-only is the security boundary), executes, returns rows as JSON. Per spec §13 zero-telemetry: the inspector is dev-mode + user-initiated; nothing leaves the machine.

3. **Graph persistence (replay-from-signals on app start).** When the renderer mounts, if a previous session's `session_id` is in `localStorage` or the URL hash, main fires a new `replay_session(session_id)` Tauri command. Main reads signals via drone IPC, translates each signal → AgentEvent (the inverse of the M02 EventPipeline), and emits them through the existing `app.emit("agent_event", ...)` channel. The renderer's `graphStore.applyEvent` consumes them in order — same code path as live events; idempotency from Stage B/C tests guarantees correctness.

The cardinal architectural decision: **VDR is a drone-internal projection, not an IPC-driven one.** The drone has direct SQLite access; running the projector on every signal-write costs O(1) per signal (decision/verify signals only; ~5 ms per row on M01's hardware baseline). Pushing the projection to main would require IPC roundtrips on every write. Drone-internal is the right shape per spec §1 (drone owns persistence) + §2b (VDR is a read-optimized projection of signals).

The second decision: **graph persistence is replay-from-signals, not snapshot-the-store.** Snapshotting the Zustand store would be simpler short-term but conflicts with spec's append-only model (signals are immutable; VDR is derived; replay is the canonical reconstruction path). Replay also tests `graphStore.applyEvent`'s idempotency property end-to-end — every event flows through the same code path whether live or replayed.

The third decision: **SELECT-only validation via parser, not regex.** Regex-matching `^SELECT` is trivially bypassed (`SELECT 1; DROP TABLE foo`). Drone uses `sqlite3_prepare_v2` to compile the SQL, then walks the parsed statement to assert it's a single `SELECT` (no compound statements via semicolons; no `WITH RECURSIVE` modifying CTEs; no `pragma_table_info`-shape pragma calls). Future M03+ work may extend with `EXPLAIN` + parameterized queries; v0.1 is `SELECT * FROM ... WHERE ...` shape.

The fourth decision: **session-id discovery via the existing M02 keychain pattern.** No new persistence for "which session was last open"; the renderer reads from `localStorage.lastSessionId` (set on session start). This avoids a fourth Tauri command for "give me the most-recent session"; main just reads what the renderer sends.

**One-line success criterion:** after running the M02 smoke test once, the user can: (a) open the SQL inspector and run `SELECT * FROM signals WHERE session_id = ?` to see the live signal log; (b) reload the app — the graph reconstructs identically (same nodes, same edges, same selected-node state) from the persisted signal log; (c) the `vdr` table populates with at least one row from the `decision_record` event the smoke test emits (M02.D's heuristic decision-extractor produces these); (d) renderer Vitest coverage on `SqlInspector.tsx` ≥80%; drone-main runtime coverage stays ≥95% on the new vdr module.

**New artifacts:**
- `crates/runtime-drone/src/vdr.rs` — VDR projection engine (drone-internal continuous projector)
- `crates/runtime-drone/tests/vdr_projection.rs` — projection idempotence + correlation correctness tests
- `crates/runtime-main/src/sdk/replay.rs` — signal-log → AgentEvent translator (inverse of EventPipeline)
- `crates/runtime-main/tests/sdk_replay.rs` — replay translation tests
- `src/components/SqlInspector.tsx` — renderer-side SQL inspector
- `tests/unit/components/SqlInspector.test.tsx` — render + execute + error-path tests
- New `DroneCommand` variants in `runtime-core::drone`: `QuerySessionDb { sql: String }`, `ReadSignals { session_id: String }`
- New Tauri commands in `src-tauri/src/commands.rs`: `query_session_db`, `replay_session`

### E.2 Files to Change

| File | Change |
|---|---|
| `crates/runtime-drone/src/vdr.rs` | **New** — VDR projection module; `project_signal(conn, signal)` for the per-signal projection path; `project_session(conn, session_id)` for full-session replay (used at SQL-inspector load time) |
| `crates/runtime-drone/src/db.rs` | **Edited** — add `vdr_insert(conn, vdr_row)` + `vdr_query_for_signal(conn, signal_id)` helpers + `signals_for_session(conn, session_id)` helper |
| `crates/runtime-drone/src/command_handler.rs` | **Edited** — handle two new DroneCommand variants: `QuerySessionDb { sql }` (read-only SQL execution; SELECT-only validation; rows-as-JSON response) + `ReadSignals { session_id }` (returns all signals for the session as JSON) |
| `crates/runtime-drone/src/heartbeat.rs` | **Edited** — wire the VDR projector to fire after each signal write (drone-internal — no IPC) |
| `crates/runtime-drone/tests/vdr_projection.rs` | **New** — 6 tests: projection produces row for decision signal; idempotent on re-run; correlates with tool signal; rejects non-SELECT SQL; SELECT returns rows as JSON; replay-full-session reproduces VDR table |
| `crates/runtime-drone/tests/integration.rs` | **Edited** — extend with QuerySessionDb + ReadSignals roundtrip integration tests |
| `crates/runtime-core/src/drone.rs` | **Edited** — add `QuerySessionDb { sql: String }` + `ReadSignals { session_id: String }` to `DroneCommand` enum; matching `QueryResult { rows: Vec<JsonValue> }` + `SignalLog { signals: Vec<JsonValue> }` to `DroneEvent` enum (or `DroneResponse` if a response type exists) |
| `crates/runtime-main/src/drone_ipc/client.rs` | **Edited** — add `query_session_db(sql: String) -> Result<Vec<Value>>` + `read_signals(session_id: String) -> Result<Vec<Value>>` methods on `DroneClient`; both wrap the underlying `send_with_reconnect` with the new command shapes |
| `crates/runtime-main/src/sdk/replay.rs` | **New** — `replay_signals_to_events(signals: Vec<Value>) -> Vec<AgentEvent>` pure-function translator; reverse of M02.D's EventPipeline |
| `crates/runtime-main/tests/sdk_replay.rs` | **New** — 4 tests: each signal type translates correctly; ordering preserved; missing fields don't crash; large signal log (~100 entries) translates without OOM |
| `src-tauri/src/commands.rs` | **Edited** — add `query_session_db(sql: String) -> Result<Vec<Value>, CmdError>` + `replay_session(session_id: String) -> Result<(), CmdError>` Tauri commands; both with `*_with` testable seams (per CLAUDE.md §5 archetype) |
| `src-tauri/src/main.rs` | **Edited** — register the two new commands in the `tauri::Builder` |
| `src/components/SqlInspector.tsx` | **New** — text area for SQL input + Execute button + results table; ARIA-compliant; debounced execute (avoid spamming drone on every keystroke) |
| `tests/unit/components/SqlInspector.test.tsx` | **New** — 5 tests: renders empty state; user types SQL + clicks Execute → invoke called with the SQL; renders results table on success; renders error state on rejection; debounces rapid clicks |
| `src/lib/ipc.ts` | **Edited** — add `invokeQuerySessionDb(sql: string) -> Promise<Row[]>` + `invokeReplaySession(sessionId: string) -> Promise<void>` wrappers |
| `src/App.tsx` | **Edited** — add `<SqlInspector>` to the layout (below the graph + inspector panel); add `useEffect` on mount that reads `localStorage.lastSessionId` and calls `invokeReplaySession` if present; persist `localStorage.lastSessionId` on session-start |
| `tests/unit/App.test.tsx` | **Edited** — add 2 tests: replay-on-mount calls invokeReplaySession with stored session id; replay populates the graph (subscribes to events fired by main during replay) |
| `tests/unit/ipc.test.ts` | **Edited** — add tests for the two new wrappers |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` entry under "Added — M03.E" |

### E.3 Detailed Changes

#### `crates/runtime-drone/src/vdr.rs` — projection engine

```rust
//! VDR projection. Reads signals 4 (decision) + 5 (verify) per spec §2b
//! and produces correlated VDR rows. Drone-internal continuous projector;
//! triggered after each signal write (no IPC roundtrip per signal).
//!
//! Idempotence: re-running `project_signal(conn, signal)` over the same
//! signal twice produces the same VDR row (uses `INSERT OR IGNORE` plus
//! a UNIQUE constraint on `vdr.contributing_signal_id`).

use rusqlite::{params, Connection, Result};
use serde_json::Value;

/// Project a single signal into the VDR table. Returns the number of rows
/// inserted (0 if signal type does not produce VDR rows or the row already
/// exists; 1 if a new row was inserted).
pub fn project_signal(conn: &Connection, signal: &Signal) -> Result<usize> {
    match signal.signal_type.as_str() {
        "decision" | "verify" => {
            let row = build_vdr_row(conn, signal)?;
            conn.execute(
                "INSERT OR IGNORE INTO vdr (
                    id, session_id, contributing_signal_id, kind,
                    decision_text, rationale, alternatives_considered,
                    confidence, tool_id, tool_input, tool_output,
                    outcome, token_cost, snapshot_id, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    row.id, row.session_id, row.contributing_signal_id, row.kind,
                    row.decision_text, row.rationale, row.alternatives_considered,
                    row.confidence, row.tool_id, row.tool_input, row.tool_output,
                    row.outcome, row.token_cost, row.snapshot_id, row.created_at,
                ],
            )
        }
        _ => Ok(0), // non-decision/verify signals do not produce VDR rows
    }
}

/// Project an entire session's signals. Used at SQL-inspector load time
/// + as a fallback consistency check. O(N) over signal count.
pub fn project_session(conn: &Connection, session_id: &str) -> Result<usize> {
    let signals = signals_for_session(conn, session_id)?;
    let mut total = 0;
    for signal in signals {
        total += project_signal(conn, &signal)?;
    }
    Ok(total)
}

fn build_vdr_row(conn: &Connection, signal: &Signal) -> Result<VdrRow> {
    // For decision signals: extract decision_text + rationale + confidence
    // from signal.payload_json. For verify signals: extract pass/fail +
    // failing_items. Correlate with the tool signal via signal.pre_signal_id
    // (decision-followed-by-tool-call) or signal.parent_signal_id (verify-
    // ran-on-tool-result). Token cost: sum from contributing tool signals'
    // payload_json.tokens_in + tokens_out (M03 Stage D).
    todo!("implementation per spec §2b VDR row shape; full body authored by fresh session")
}

// ... helper functions: signals_for_session, etc. — the fresh session
// fills these in following the existing db.rs query pattern (M01.C archetype) ...
```

**Schema migration:** the existing `vdr` table (M01) needs a `contributing_signal_id` column with a UNIQUE constraint for the idempotence guarantee. Stage E adds a migration in `db.rs::init_schema` (idempotent — `ALTER TABLE ... ADD COLUMN IF NOT EXISTS` semantics + `CREATE UNIQUE INDEX IF NOT EXISTS`).

#### `crates/runtime-drone/src/command_handler.rs` — new IPC commands

Extend the existing `dispatch_command` switch with two new variants:

```rust
DroneCommand::QuerySessionDb { sql } => {
    if !is_select_only(&sql) {
        return Err(DroneError::SqlValidation(format!(
            "only SELECT statements permitted; got: {}",
            &sql[..sql.len().min(80)]
        )));
    }
    let rows = execute_select(&conn, &sql)?;
    Ok(DroneResponse::QueryResult { rows })
}

DroneCommand::ReadSignals { session_id } => {
    let signals = signals_for_session_as_json(&conn, &session_id)?;
    Ok(DroneResponse::SignalLog { signals })
}
```

The `is_select_only` validator is **parser-based, not regex-based**. Per Decision #3 in E.1: regex-matching `^SELECT` is trivially bypassed. Use `sqlite3_prepare_v2` to compile the SQL; check the resulting prepared statement's `sql()` for compound statements via semicolons + verify `column_count() > 0` (DDL/DML have zero output columns) + reject `pragma_*` and `WITH RECURSIVE` modifying CTEs explicitly.

```rust
fn is_select_only(sql: &str) -> bool {
    // Compile the SQL; if it doesn't compile as a single statement,
    // reject. If it produces zero output columns, it's not a SELECT
    // (DDL/DML have zero columns). Reject pragma calls explicitly.
    let trimmed = sql.trim();
    if trimmed.contains(';') {
        // Compound statements via semicolons — reject (one statement only).
        // SQLite ignores trailing whitespace + a single trailing semicolon
        // but multiple semicolons indicate compound. Conservative: reject.
        let stripped = trimmed.trim_end_matches(';').trim_end();
        if stripped.contains(';') {
            return false;
        }
    }
    let lower = trimmed.to_lowercase();
    if lower.starts_with("pragma ") {
        return false;
    }
    // Use rusqlite's column_count() check after prepare:
    // SELECT and EXPLAIN return columns; everything else returns 0.
    let dummy_conn = rusqlite::Connection::open_in_memory().unwrap_or_else(|_| panic!("in-memory open"));
    match dummy_conn.prepare(sql) {
        Ok(stmt) => stmt.column_count() > 0 && lower.starts_with("select"),
        Err(_) => false,
    }
}
```

Tested via the new `vdr_projection.rs::rejects_non_select_sql` test (per E.4).

#### `crates/runtime-main/src/sdk/replay.rs` — signal-to-event translator

```rust
//! Replay: signal log → AgentEvent stream. Inverse of M02.D's EventPipeline.
//! Used when the renderer mounts with a known session_id; main reads
//! signals via drone IPC, translates them here, emits to the renderer
//! via the existing `app.emit("agent_event", ...)` channel.

use runtime_core::event::AgentEvent;
use serde_json::Value;

/// Pure-function translator. Each signal becomes one AgentEvent.
/// Ordering is preserved (signals are sorted by timestamp at the drone).
/// Idempotent at the renderer side: graphStore.applyEvent's idempotence
/// property guarantees correctness when called twice.
pub fn replay_signals_to_events(signals: &[Value]) -> Vec<AgentEvent> {
    signals.iter().filter_map(signal_to_event).collect()
}

fn signal_to_event(signal: &Value) -> Option<AgentEvent> {
    let signal_type = signal.get("type")?.as_str()?;
    let payload = signal.get("payload_json")?;
    match signal_type {
        "agent" => {
            // payload has event="spawned" | "complete" | "error"
            let event = payload.get("event")?.as_str()?;
            match event {
                "spawned" => Some(AgentEvent::AgentSpawned { /* ... */ }),
                "complete" => Some(AgentEvent::AgentComplete { /* ... */ }),
                "error" => Some(AgentEvent::AgentError { /* ... */ }),
                _ => None,
            }
        }
        "tool" => Some(AgentEvent::ToolInvoked { /* ... */ }),
        "skill" => Some(AgentEvent::SkillLoaded { /* ... */ }),
        "decision" => Some(AgentEvent::DecisionRecord { /* ... */ }),
        "session" => {
            let event = payload.get("event")?.as_str()?;
            match event {
                "start" => Some(AgentEvent::SessionStart { /* ... */ }),
                _ => None, // session_end is not in v0.1 AgentEvent schema
            }
        }
        _ => None, // unknown signal type — skip silently per spec §2b's "more types may exist"
    }
}
```

The fresh session fills in the `/* ... */` extraction from each signal's `payload_json` shape per M02 Signal Schema v2 (see `.aria/docs/SIGNAL-SCHEMA-V2.md` if the reference is needed). Tested via the 4 tests in `crates/runtime-main/tests/sdk_replay.rs` per E.4.

#### `src/components/SqlInspector.tsx` — renderer SQL inspector

```typescript
import { useState } from 'react';
import { invokeQuerySessionDb } from '../lib/ipc';

export function SqlInspector() {
  const [sql, setSql] = useState('SELECT * FROM signals LIMIT 10;');
  const [rows, setRows] = useState<Record<string, unknown>[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleExecute(): Promise<void> {
    setLoading(true);
    setError(null);
    try {
      const result = await invokeQuerySessionDb(sql);
      setRows(result);
    } catch (e) {
      console.error('SQL inspector error:', e);
      setError(unwrapCmdError(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <section aria-label="SQL inspector" className="sql-inspector" data-testid="sql-inspector">
      <textarea
        value={sql}
        onChange={(e) => setSql(e.target.value)}
        rows={3}
        aria-label="SQL query"
        disabled={loading}
      />
      <button type="button" onClick={() => void handleExecute()} disabled={loading || sql.trim().length === 0}>
        {loading ? 'Executing…' : 'Execute'}
      </button>
      {error && <p className="error" role="alert">{error}</p>}
      {rows.length > 0 && (
        <table className="sql-results">
          <thead>
            <tr>{Object.keys(rows[0]).map((k) => <th key={k}>{k}</th>)}</tr>
          </thead>
          <tbody>
            {rows.map((row, i) => (
              <tr key={i}>
                {Object.values(row).map((v, j) => <td key={j}>{String(v)}</td>)}
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
```

#### `src/App.tsx` — replay-on-mount + persist session-id

Extend the existing `useEffect` block:

```typescript
useEffect(() => {
  // Replay-on-mount: if a previous session-id is in localStorage,
  // request main to read its signal log and re-emit AgentEvents.
  // graphStore.applyEvent processes them in order — same code path
  // as live events; idempotence property guarantees correctness.
  const lastSessionId = localStorage.getItem('lastSessionId');
  if (lastSessionId) {
    void invokeReplaySession(lastSessionId).catch((e) => {
      console.error('Replay session error:', e);
    });
  }

  // ... existing subscribeAgentEvents wiring ...

  return () => {
    // ... existing cleanup ...
  };
}, []);
```

When `agent_event` arrives with `type: 'session_start'`, write `event.session_id` to `localStorage.lastSessionId` so the next mount can replay.

### E.4 Tests

#### Pedantic-pass preflight

Stage E adds new Rust modules (vdr.rs, replay.rs) and TS components (SqlInspector). Apply `docs/gotchas.md` #21 + the existing M02 traps. Stage E specifics:

- [ ] `derive_partial_eq_without_eq` on VDR row types (likely contain serde_json::Value) — needs `#[allow]` with rationale
- [ ] `unused_async` — drone command handlers may be sync (the existing M01 dispatch is sync); keep new variants sync too
- [ ] SELECT-only validator is **parser-based, not regex-based** (per Decision #3); regex-only attempts will fail security review

Vitest+RTL trap (per `docs/gotchas.md` #27) applies to SqlInspector tests: any `await waitFor(() => loading toggle)` followed by interaction must re-query the elements via `findByLabelText`.

#### Default test plan for stages adding a new safety primitive

`vdr.rs` is a new safety primitive — bug here corrupts the VDR table. Treat as ≥95% line per the runtime-drone gate. `replay.rs` is normal coverage (≥80%) — pure function with deterministic translation; less risk because it's read-only over the persisted log.

#### Test plan (Stage E)

`crates/runtime-drone/tests/vdr_projection.rs` — **new** (6 tests):

1. **decision signal produces VDR row** — write a decision signal, call project_signal, assert vdr table has 1 row with matching contributing_signal_id
2. **verify signal produces VDR row** — same shape for verify type
3. **non-decision/non-verify signal produces nothing** — write a tool signal, call project_signal, assert vdr table count unchanged
4. **idempotent on re-run** — call project_signal twice on the same signal, assert vdr count is 1 (UNIQUE constraint enforces)
5. **project_session reproduces VDR table** — write 5 signals (mix of types), call project_session, assert correct subset projected
6. **rejects non-SELECT SQL** — call is_select_only with 6 attack vectors (DROP, DELETE, INSERT, UPDATE, PRAGMA, compound semicolons); assert all rejected

`crates/runtime-drone/tests/integration.rs` — **edited** (2 new tests):

1. **QuerySessionDb roundtrip** — drone subprocess receives QuerySessionDb command with valid SELECT; returns rows; main parses; assert correctness
2. **ReadSignals roundtrip** — drone returns signals for a populated session; main parses; assert ordering

`crates/runtime-main/tests/sdk_replay.rs` — **new** (4 tests):

1. **each signal type translates to expected AgentEvent** — feed mock signals of each type; assert AgentEvent variants match
2. **ordering preserved** — feed signals out of timestamp order (drone returns sorted; test simulates raw); assert output ordering matches input order
3. **missing fields don't crash** — feed signal with missing payload_json fields; assert filtered out, not panicked
4. **large signal log translates without OOM** — feed 100-signal log; assert duration < 100ms + memory bounded (per `docs/gotchas.md` #28 bounded streams)

`tests/unit/components/SqlInspector.test.tsx` — **new** (5 tests):

1. **renders default SQL placeholder + Execute button** — assert visible
2. **user types SQL + clicks Execute → invokeQuerySessionDb called with the SQL** — mock the IPC; assert call shape
3. **renders results table on success** — IPC returns rows; assert table headers + cells
4. **renders error state on rejection** — IPC rejects with CmdError; assert error message via `unwrapCmdError`
5. **debounces rapid Execute clicks** — fire 3 clicks within 100ms; assert IPC called once

`tests/unit/App.test.tsx` — **edited** (2 new tests):

1. **mount replays session if localStorage has lastSessionId** — set localStorage, render App, assert invokeReplaySession called with that id
2. **session_start event persists session_id to localStorage** — fire session_start through subscribeAgentEvents, assert localStorage.lastSessionId set

#### Coverage target

- Workspace Rust: ≥80% (preserved)
- runtime-drone: ≥95% (preserved; new vdr.rs adds ~150 lines covered by 6 tests)
- runtime-main: ≥95% (preserved; new replay.rs covered by 4 tests)
- src-tauri: 50% patch gate (new commands.rs functions; testable seams cover the logic — wrappers are excluded by the `tauri-shell` codecov path)
- src/ frontend: ≥80% with **graphStore.ts ≥95%** (preserved) + **layout.ts ≥95%** (preserved); SqlInspector.tsx ≥80%

**Doc-to-CI invariant.** Stage E adds runtime-main `replay.rs` (testable) — no new exclusion. Stage E adds runtime-drone `vdr.rs` (testable) — no new exclusion. SqlInspector renders results from a Tauri command — the `*_with` testable seam in `src-tauri/src/commands.rs` keeps the new commands' coverage in the seam, the wrapper-only lines counted under the 50% `tauri-shell` gate.

### E.5 CLI Prompt

```xml
<work_stage_prompt id="M03.E">
  <context>
    Stage E of M03 (Live Graph). VDR projection (drone-internal continuous
    projector reading signals → vdr table) + SQL inspector (renderer-side
    SELECT-only query UI) + graph persistence (replay-from-signals on app
    mount). Largest stage in M03 — touches drone Rust, runtime-main Rust,
    src-tauri Rust, and the renderer. Builds on Stage D's inspector panel
    + token tracking. Stage F does not start until Stage E's commit is
    on the milestone branch.
  </context>

  <read_first>
    <file>CLAUDE.md</file>
    <file>STAGE-PROMPT-PROTOCOL.md</file>
    <file>docs/build-prompts/M03-live-graph.md (Stage E sections E.1–E.4)</file>
    <file>agent-runtime-spec.md §1 (drone) §2b (signals + VDR) §3 (graph behavior)</file>
    <file>docs/MVP-v0.1.md §M3</file>
    <file>docs/gotchas.md (especially #21 #27 #28)</file>
  </read_first>

  <read_reference>
    <file purpose="M01 drone db.rs schema + WAL pragma archetype to extend with VDR migration">crates/runtime-drone/src/db.rs</file>
    <file purpose="M01 drone command_handler.rs dispatch pattern to extend with QuerySessionDb + ReadSignals">crates/runtime-drone/src/command_handler.rs</file>
    <file purpose="M02.D EventPipeline shape — replay.rs is the inverse direction">crates/runtime-main/src/sdk/event_pipeline.rs</file>
    <file purpose="M02.D drone IPC client + reconnect pattern to extend with new command methods">crates/runtime-main/src/drone_ipc/client.rs</file>
    <file purpose="M02 Tauri command archetype + *_with testable seam pattern">src-tauri/src/commands.rs</file>
    <file purpose="Stage D InspectorPanel pattern (sibling component for SqlInspector)">src/components/InspectorPanel.tsx</file>
    <file purpose="Stage A xtask drift check pattern (no schema changes Stage E but pattern is referenced)">crates/xtask/tests/check_drift.rs</file>
  </read_reference>

  <read_prior_stages>
    <retrospective section="[END] Decisions for the next stage">docs/build-prompts/retrospectives/M03.A-retrospective.md</retrospective>
    <retrospective section="[END] Decisions for the next stage">docs/build-prompts/retrospectives/M03.B-retrospective.md</retrospective>
    <retrospective section="[END] Decisions for the next stage">docs/build-prompts/retrospectives/M03.C-retrospective.md</retrospective>
    <retrospective section="[END] Decisions for the next stage">docs/build-prompts/retrospectives/M03.D-retrospective.md</retrospective>
  </read_prior_stages>

  <deliverable ref="docs/build-prompts/M03-live-graph.md" section="E.3 Detailed Changes"/>

  <test_plan_required>true</test_plan_required>

  <execution_steps>
    <step name="write_failing_tests" budget="1"/>
    <step name="implement" budget="1"/>
    <step name="verify_gates" budget_iterations="3"/>
    <step name="fill_retrospective"/>
    <step name="surface"/>
  </execution_steps>

  <acceptance_criteria ref="docs/build-prompts/M03-live-graph.md" section="E.4 Tests"/>

  <scope_locks ref="docs/build-prompts/M03-live-graph.md" section="Key constraints"/>

  <gates milestone="M03"/>

  <self_correction_budget>3</self_correction_budget>

  <gotchas>
    <trap>SELECT-only SQL validation MUST be parser-based, not regex-based. Regex-matching ^SELECT is trivially bypassed (`SELECT 1; DROP TABLE foo` slips through). Use `rusqlite::Connection::prepare` + `column_count() > 0` + reject compound semicolons + reject `pragma_*`. Per Decision #3 in E.1.</trap>
    <trap>VDR projection is drone-INTERNAL (continuous, post-signal-write), not main-side via IPC. Pushing to main means IPC roundtrips per signal — wrong shape. Per Decision #1 in E.1.</trap>
    <trap>Graph persistence is replay-FROM-SIGNALS, not snapshot-the-store. Snapshotting Zustand directly would conflict with spec's append-only model. Replay also tests applyEvent's idempotency end-to-end. Per Decision #2 in E.1.</trap>
    <trap>VDR idempotence: `INSERT OR IGNORE` + UNIQUE constraint on contributing_signal_id. Without the constraint, re-running project_signal duplicates rows. Schema migration in db.rs MUST add the constraint via `CREATE UNIQUE INDEX IF NOT EXISTS`.</trap>
    <trap>Signal-log replay can be large (100+ signals on a real session). The test for OOM behavior (per docs/gotchas.md #28) bounds the input + asserts memory-bounded translation. Don't use unbounded streaming patterns.</trap>
    <trap>The new Tauri commands (`query_session_db`, `replay_session`) MUST have `*_with` testable seams (per CLAUDE.md §5 archetype). Without seams, the wrapper lines fall under the 50% tauri-shell codecov gate AND the testable logic stays in the seam at 80%+.</trap>
    <trap>localStorage is NOT shared across Tauri instances or sessions (it's webview-scoped). For M03, localStorage.lastSessionId is sufficient; M04+ may persist last-session-id in the SQLite database itself if cross-instance state is needed. Don't over-engineer for M03.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT add new schema changes to schemas/event.v1.json in Stage E. Stage E is signal-log replay + VDR — no new event variants. Stage D's schema bump is the last AgentEvent schema change in M03.</warning>
    <warning>DO NOT add `serde_rusqlite` or other third-party crates without `cargo deny check` passing per CLAUDE.md §6 hard rule. Hand-roll the Row → JSON conversion (~50 lines) instead.</warning>
    <warning>DO NOT extend the SQL inspector with parameterized queries / EXPLAIN / WITH RECURSIVE in Stage E. v0.1 is `SELECT * FROM ... WHERE ...` shape. M03+ may extend.</warning>
    <warning>DO NOT touch SetupPanel, SmokeButton, ipc.ts (beyond the two new wrappers), or M02 keychain code — preserved verbatim. Stage E adds; doesn't refactor existing surfaces.</warning>
  </execution_warnings>

  <time_box estimate_hours="5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Decisions for Stage F: whether the SQL inspector + graph replay UX feels right or needs refactor for the Tauri-driver E2E (Stage F's E2E suite must navigate to the inspector + verify replay reconstructs); whether the VDR projection performance is acceptable with 100+ signals (drone-internal projector should be <10ms per signal); whether the localStorage-based session-id persistence will hold up against M04's multi-session expectations or needs an early SQLite-side persistence; whether the SELECT-only validator's rejection of compound statements is too strict for legitimate use cases.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M03-live-graph.md" section="E.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD) — Stage E is the largest stage; expect ~30+ files</item>
    <item>gate results (each gate, pass/fail, key numbers including new vdr.rs ≥95% + replay.rs ≥80% + SqlInspector.tsx ≥80% + drift check + drone integration tests)</item>
    <item>retrospective (filled-in [END] section with three-axis scoring + verdict + decisions for Stage F + [END] Coverage holdouts subsection)</item>
    <item>draft commit message from M03-live-graph.md E.6 Commit Message section</item>
    <item>screenshot or paste of the rendered app showing: (a) graph live-rendered after smoke-test run; (b) SQL inspector below with `SELECT * FROM signals` results visible; (c) reload demonstration — close the app, reopen, graph reconstructs identically</item>
    <item>explicit statement: "Stage M03.E is ready. I will not commit until you approve."</item>
  </approval_surface>
</work_stage_prompt>
```

### E.6 Commit Message

```
feat(runtime+renderer): M03 Stage E — VDR projection + SQL inspector + replay

Largest stage in M03. Three pieces, one stage:

- VDR projection: drone-internal continuous projector reads signals
  4 (decision) + 5 (verify) per spec §2b → writes correlated vdr
  table rows. New `runtime_drone::vdr` module. Idempotent via
  UNIQUE constraint on contributing_signal_id (schema migration in
  db.rs::init_schema). Triggered after each signal write — no IPC
  roundtrip per signal.
- SQL inspector: new <SqlInspector> renderer component + new Tauri
  command query_session_db(sql) → main proxies to drone via new
  DroneCommand::QuerySessionDb. Drone validates SELECT-only via
  parser-based rusqlite::Connection::prepare + column_count() > 0
  (NOT regex-based — regex is trivially bypassable). Returns rows
  as JSON.
- Graph persistence: replay-from-signals on app mount. New
  invokeReplaySession(session_id) Tauri command; main reads signals
  via new DroneCommand::ReadSignals; new runtime-main::sdk::replay
  module translates signal log → AgentEvent stream (inverse of
  M02.D EventPipeline); main re-emits via existing app.emit
  ("agent_event") channel. graphStore.applyEvent's idempotence
  (Stage B/C tested) guarantees correctness.

Four architectural decisions locked:
- VDR is drone-internal projection, not IPC-driven (perf + spec
  alignment).
- Persistence is replay-from-signals, not snapshot-the-store
  (spec append-only model + tests applyEvent idempotence end-to-end).
- SELECT-only validation is parser-based, not regex-based (security).
- Session-id discovery via localStorage.lastSessionId (M03 sufficient;
  M04+ may persist in SQLite for cross-instance).

~30 files in scope: 4 NEW Rust modules (vdr.rs + vdr_projection.rs
test + replay.rs + sdk_replay.rs test), 2 NEW renderer files
(SqlInspector + test), ~12 EDIT (drone db + command_handler +
heartbeat + integration test, runtime-core::drone + runtime-main
client + sdk-mod, src-tauri commands + main, src App + ipc + tests).

~17 new tests: 6 vdr_projection (decision/verify/non-match/
idempotence/full-session/SELECT-only-rejection); 2 drone integration
roundtrips; 4 replay (per-type/ordering/missing-fields/100-signal-OOM);
5 SqlInspector (default/execute/results/error/debounce); 2 App-level
(replay-on-mount + session-id-persist).

Coverage: vdr.rs ≥95% line (new primitive — bug corrupts the VDR
table); replay.rs ≥80% (pure function); SqlInspector ≥80%;
runtime-drone stays ≥95%; runtime-main stays ≥95%.

Refs: M03-live-graph.md §E; agent-runtime-spec.md §1 §2b §3 §13;
docs/MVP-v0.1.md §M3 acceptance criteria #2 (graph reconstructs
after page reload) + #6 (VDR populated table); CLAUDE.md §5 *_with
archetype + §6 cargo deny no-new-deps; docs/gotchas.md #21 #27 #28.

https://claude.ai/code
```

---
<!-- ============================================================ -->
<!-- STAGE F — Tauri-driver E2E + Phase Closeout                  -->
<!-- ============================================================ -->

## Stage F — Tauri 2.x desktop-shell E2E + Phase Closeout

**WEBCHECK:** verify each URL against this stage's prompt body **before** the fresh session opens. If any claim is stale, update this section in `M03-live-graph.md` BEFORE pasting Stage F's CLI prompt to the fresh session.

- <https://v2.tauri.app/develop/tests/webdriver/> — confirm Tauri 2.x desktop-shell E2E uses `tauri-driver` + WebdriverIO; macOS unsupported (no WKWebView WebDriver tool); Linux + Windows officially supported
- <https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/> — confirm WebdriverIO config shape (`wdio.conf.ts`) for Tauri 2.x; `tauri-driver` startup as a service; window-handle attachment pattern
- <https://v2.tauri.app/develop/tests/webdriver/ci/> — confirm GitHub Actions CI setup for Linux (webkit2gtk + tauri-driver via apt) + Windows (msedgedriver auto-on-PATH for windows-latest runners)
- <https://www.npmjs.com/package/@crabnebula/tauri-driver> — confirm current stable version of the npm package; alternative is `tauri-driver` cargo binary (older path); the npm package is the v2 official distribution
- <https://docs.crabnebula.dev/plugins/tauri-e2e-tests/> — confirm any additional setup or configuration for end-to-end Tauri tests
- <https://webdriver.io/docs/configurationfile> — confirm WebdriverIO v9 config + capabilities format (Tauri 2.x ecosystem expects v9+; older v8 configs may not work)

### F.1 Problem Statement

Stage F closes M03 with two workstreams that share a single commit:

**Part 1 — Full Tauri 2.x desktop-shell E2E.** M02.E shipped four `test.skip()` Playwright entries with rationale stating that full desktop-shell E2E requires `tauri-driver` + WebdriverIO per the official Tauri docs (Playwright's `_electron` API is Electron-specific and won't drive Tauri's WebView2 / WebKitGTK windows). Stage F lights this up: adds `@crabnebula/tauri-driver` + WebdriverIO v9 + a `wdio.conf.ts` configured for Tauri 2.x, ships 6 E2E tests covering renderer-load + smoke happy path + graph rendering + inspector panel + SQL inspector + replay reconstruction, wires a new `e2e-tauri-driver` CI job into Linux + Windows runners (macOS unsupported per tauri-driver upstream — deferred indefinitely per MVP §M3 out-of-scope), and **deletes** the four `test.skip()` carry-forwards from `tests/e2e/smoke.spec.ts`. The renderer-level Playwright tests stay — they cover a different layer (Vite dev server, fast feedback) and complement the desktop-shell tests.

**Part 2 — M03 Phase Closeout.** Per `CLAUDE.md` §20, every parent milestone produces a Gap Analysis entry **separate from per-stage retrospectives**. Retrospectives evaluate the build *process*; gap analysis evaluates the build *product* (does code match spec, what did spec get wrong, prioritized fix backlog). The M03 entry lands at Stage F as the **final commit on the parent-milestone branch**, gating the PR push. Six required sections per `CLAUDE.md` §20 (codebase deep dive cumulative across M01+M02+M03; adherence to spec with file:line cites; spec review forward-looking; fix backlog 🔴/🟡/🟢; carry-forward from prior milestones; sign-off). Plus the new v1.2 protocol's `<gotchas_graduation>` subsection auditing per-stage `<gotchas>` across A–F (disposition: kept | graduated | resolved | expired). Plus `M03-summary.md` per `SUMMARY-TEMPLATE.md` aggregating per-stage axis scores. The commit is **immutable once landed** — future milestones report status updates via their Carry-forward sections only; Stage F's gap-analysis entry never gets edited.

The cardinal architectural decision: **Tauri-driver E2E tests are a SEPARATE test type, not an extension of the existing Playwright suite.** They live at `tests/e2e-tauri/` (new directory) with their own `wdio.conf.ts`. The existing `tests/e2e/` Playwright tests remain and exercise the Vite-dev-server renderer (fast feedback, no Tauri compile required); the new `tests/e2e-tauri/` exercises the actual built Tauri app (slow feedback, full integration). Two test types, two CI jobs, two layers of regression detection.

The second decision: **CI matrix is Linux + Windows only; macOS is documented out-of-scope.** Per `tauri-driver`'s official limitations + MVP §M3 out-of-scope; the existing macOS Rust matrix job continues to run unit + integration tests but skips the new `e2e-tauri-driver` job. This is a one-line CI config — `if: matrix.os != 'macos-latest'` on the new job.

The third decision: **the four `test.skip()` carry-forwards are DELETED, not migrated.** The skipped entries in `tests/e2e/smoke.spec.ts` were rationale-only placeholders for Stage F to address; their replacement isn't a 1:1 port to the new test file (the WebdriverIO + tauri-driver test setup is genuinely different). Stage F authors fresh tests in `tests/e2e-tauri/smoke.e2e.ts` covering the same intent + new M03-specific scenarios.

The fourth decision: **gap-analysis entry's `<gotchas_graduation>` audits all 28+ per-stage gotchas** (Stage A through Stage F authoring + execution combined). Each gets a disposition. This is the v1.2 protocol's structural improvement over M01/M02 (which used informal disposition discussion in retros); Stage F is the first milestone to ship the formal subsection.

**One-line success criterion:** the new `e2e-tauri-driver` CI job runs green on Linux + Windows; six E2E tests cover renderer-load + save-key + smoke + graph-render + inspector-panel + sql-inspector + replay; `M03-summary.md` lands with all axis scores ≥3 across all stages (no hard-gate violations); `docs/gap-analysis.md` gains an immutable M03 entry with six populated sections + a `<gotchas_graduation>` subsection covering all per-stage gotchas; the M03 PR is pushed and ready for three-artifact review.

**New artifacts:**
- `wdio.conf.ts` — WebdriverIO v9 config for Tauri 2.x desktop-shell E2E
- `tests/e2e-tauri/` directory + `tests/e2e-tauri/smoke.e2e.ts`
- `docs/build-prompts/retrospectives/M03-summary.md`
- M03 entry appended to `docs/gap-analysis.md` (immutable)

### F.2 Files to Change

| File | Change |
|---|---|
| `package.json` | **Edited** — add `@crabnebula/tauri-driver`, `@wdio/cli ^9`, `@wdio/local-runner ^9`, `@wdio/mocha-framework ^9`, `@wdio/spec-reporter ^9`, `webdriverio ^9`, `@types/mocha` to `devDependencies`. Add new script `"test:e2e:tauri": "wdio run wdio.conf.ts"` |
| `package-lock.json` | **Regenerated** via `npm install` |
| `wdio.conf.ts` | **New** — WebdriverIO v9 config; tauri-driver as a service; window-handle attachment + base URL = `tauri://localhost`; spec-reporter; capabilities for chromedriver-on-WebKitGTK (Linux) + msedgedriver-on-WebView2 (Windows); excludes macOS via `process.platform !== 'darwin'` guard at config-load time |
| `tests/e2e-tauri/smoke.e2e.ts` | **New** — 6 tests using `@wdio/mocha-framework` + chai assertions (per WebdriverIO v9 default): (1) app launches with SetupPanel visible, (2) save-key flow (paste valid format → click save → ✓ checkmark appears), (3) run-smoke happy path (after key saved, click Run smoke test → AgentNode appears in graph), (4) click AgentNode → InspectorPanel opens with node data, (5) SQL inspector executes `SELECT * FROM signals LIMIT 5` and renders rows, (6) reload (close + relaunch) → graph reconstructs from persisted signals |
| `tests/e2e/smoke.spec.ts` | **Edited** — delete the 4 `test.skip()` entries (the renderer-level Playwright happy-path tests STAY; only the skip-with-rationale entries that pointed to Stage F leave) |
| `.github/workflows/ci.yml` | **Edited** — add new `e2e-tauri-driver` job: matrix Linux + Windows; needs Rust toolchain + npm + tauri-driver setup + cargo tauri build; runs `npm run test:e2e:tauri`. Linux runner installs tauri-driver via npm; Windows runner uses msedgedriver auto-on-PATH (windows-latest has it). Job uses the existing Tauri Linux deps `apt-get install` block; macOS skipped via `if:` guard |
| `docs/build-prompts/retrospectives/M03-summary.md` | **New** — per `docs/build-prompts/retrospectives/SUMMARY-TEMPLATE.md`: aggregate three-axis scores across A–F; cross-stage trends; pattern-level wins + surprises; hard-gate-violation summary (none, or list); time-box accuracy (calibrated 25–31h vs actual; M02 ratio was 0.7×); 8–12 explicit decisions to apply before M04.1 authoring; verdict (Sound | Sound-but-rough | Friction-heavy | Not-ready); user-review notes |
| `docs/gap-analysis.md` | **Edited (append-only)** — append the M03 entry per `CLAUDE.md` §20: six required sections (codebase deep dive cumulative; adherence to spec ✅/⚠️/❌ with file:line cites; spec review forward-looking; fix backlog 🔴/🟡/🟢; carry-forward from M01/M02; sign-off) plus `<gotchas_graduation>` subsection per v1.2 STAGE-PROMPT-PROTOCOL.md (each per-stage gotcha gets disposition: kept | graduated | resolved | expired). **APPEND-ONLY HARD RULE per §20: prior M01 + M02 entries MUST NOT be edited, reordered, or deleted.** CI's `gap-analysis.md append-only check` job (M01.E-shipped) verifies. |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` entry under "Added — M03.F (Tauri-shell E2E + Phase Closeout)" |

### F.3 Detailed Changes

#### `wdio.conf.ts` — WebdriverIO v9 + tauri-driver config

```typescript
import type { Options } from '@wdio/types';
import { spawn, type ChildProcess } from 'node:child_process';
import { resolve } from 'node:path';

if (process.platform === 'darwin') {
  // macOS unsupported by tauri-driver upstream per
  // https://v2.tauri.app/develop/tests/webdriver/. The wdio.conf.ts
  // exits cleanly so `npm run test:e2e:tauri` is a no-op on macOS
  // rather than a hard failure.
  process.stdout.write('tauri-driver E2E skipped on macOS (unsupported)\n');
  process.exit(0);
}

const TAURI_DRIVER_PORT = 4444;
const APP_BIN_NAME = process.platform === 'win32' ? 'agent-runtime.exe' : 'agent-runtime';
const APP_BIN_PATH = resolve(__dirname, 'src-tauri/target/release', APP_BIN_NAME);

let tauriDriverProc: ChildProcess | undefined;

export const config: Options.Testrunner = {
  runner: 'local',
  framework: 'mocha',
  mochaOpts: { ui: 'bdd', timeout: 60_000 },
  reporters: ['spec'],
  specs: ['./tests/e2e-tauri/**/*.e2e.ts'],
  maxInstances: 1, // tauri-driver does not parallelize within a single host
  capabilities: [
    {
      maxInstances: 1,
      // Per the official Tauri 2.x WebDriver example
      // (https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/develop/Tests/WebDriver/Example/webdriverio.mdx),
      // capabilities intentionally OMIT `browserName` — tauri-driver constructs
      // the native value when proxying to the platform driver (WebKitWebDriver
      // on Linux, msedgedriver on Windows). Setting it explicitly breaks
      // the session-creation handshake on both platforms.
      'tauri:options': {
        application: APP_BIN_PATH,
      },
    } as WebdriverIO.Capabilities,
  ],
  hostname: '127.0.0.1',
  port: TAURI_DRIVER_PORT,
  logLevel: 'info',

  // Lifecycle hooks: spawn tauri-driver before WebdriverIO connects;
  // tear down after. Per https://v2.tauri.app/develop/tests/webdriver/.
  onPrepare() {
    tauriDriverProc = spawn('tauri-driver', ['--port', String(TAURI_DRIVER_PORT)], {
      stdio: 'inherit',
    });
  },

  onComplete() {
    tauriDriverProc?.kill('SIGTERM');
  },
};
```

#### `tests/e2e-tauri/smoke.e2e.ts` — 6 tests covering M03's user-facing surfaces

```typescript
import { expect } from 'chai';

describe('Tauri shell E2E — M03 live graph', () => {
  it('app launches with SetupPanel visible', async () => {
    const setupPanel = await $('section[aria-label="api key setup"]');
    await setupPanel.waitForDisplayed({ timeout: 10_000 });
    expect(await setupPanel.isDisplayed()).to.be.true;
  });

  it('save-key flow: paste key → save → ✓ stored in OS keychain', async () => {
    const input = await $('input[type="password"]');
    await input.setValue('sk-ant-test-1234567890123456'); // length ≥ 10 to enable button
    const saveButton = await $('button*=Save key');
    await saveButton.click();
    const savedIndicator = await $('span[aria-label="saved"]');
    await savedIndicator.waitForDisplayed({ timeout: 5_000 });
    expect(await savedIndicator.getText()).to.include('stored in OS keychain');
  });

  it('graph renders after smoke test (real Anthropic API call)', async () => {
    // Requires real ANTHROPIC_API_KEY in the OS keychain.
    // CI runs this with a test key from secrets; locally requires user-set key.
    const smokeButton = await $('button*=Run smoke test');
    await smokeButton.click();
    const agentNode = await $('[data-testid^="agent-node-"]');
    await agentNode.waitForDisplayed({ timeout: 30_000 });
    expect(await agentNode.getAttribute('data-status')).to.equal('active');
    // Wait for completion (Haiku 4.5 ~3s for the smoke prompt).
    await browser.waitUntil(
      async () => (await agentNode.getAttribute('data-status')) === 'complete',
      { timeout: 30_000 },
    );
  });

  it('click AgentNode → InspectorPanel opens with node data', async () => {
    const agentNode = await $('[data-testid^="agent-node-"]');
    await agentNode.click();
    const inspector = await $('[data-testid="inspector-panel"]');
    await inspector.waitForDisplayed({ timeout: 5_000 });
    const json = await $('.inspector-panel__data');
    const text = await json.getText();
    expect(text).to.include('agentId');
    expect(text).to.include('status');
  });

  it('SQL inspector executes SELECT * FROM signals LIMIT 5', async () => {
    const inspectorTextarea = await $('textarea[aria-label="SQL query"]');
    await inspectorTextarea.setValue('SELECT * FROM signals LIMIT 5;');
    const executeButton = await $('button*=Execute');
    await executeButton.click();
    const resultsTable = await $('table.sql-results');
    await resultsTable.waitForDisplayed({ timeout: 5_000 });
    const rows = await $$('table.sql-results tbody tr');
    expect(rows.length).to.be.greaterThan(0);
  });

  it('reload reconstructs the graph from persisted signals', async () => {
    // Close the app, relaunch via tauri-driver's restart capability.
    await browser.reloadSession();
    // After reload, the AgentNode from the previous smoke test should
    // re-appear via replay-from-signals.
    const agentNode = await $('[data-testid^="agent-node-"]');
    await agentNode.waitForDisplayed({ timeout: 15_000 });
    expect(await agentNode.getAttribute('data-status')).to.equal('complete');
  });
});
```

The fresh Stage F session adapts these tests to the actual selectors + timing observed during local Tauri-driver runs; the patterns are the load-bearing structure.

#### `.github/workflows/ci.yml` — new `e2e-tauri-driver` job

Add a new job after the existing `e2e` (Playwright renderer) job:

```yaml
e2e-tauri-driver:
  name: E2E (Tauri-driver desktop shell)
  needs: [detect-cargo, frontend]
  if: needs.detect-cargo.outputs.has_cargo == 'true'
  runs-on: ${{ matrix.os }}
  strategy:
    fail-fast: false
    matrix:
      os: [ubuntu-latest, windows-latest]
      # macos-latest skipped — tauri-driver does not support macOS per
      # https://v2.tauri.app/develop/tests/webdriver/. Renderer-level
      # Playwright tests cover macOS via the existing `e2e` job.
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: actions/setup-node@v4
      with:
        node-version: 22
        cache: npm
    - name: Install Tauri Linux system deps
      if: matrix.os == 'ubuntu-latest'
      run: |
        sudo apt-get update
        sudo apt-get install -y \
          libwebkit2gtk-4.1-dev \
          libgtk-3-dev \
          libayatana-appindicator3-dev \
          librsvg2-dev \
          libxdo-dev \
          libssl-dev \
          webkit2gtk-driver \
          xvfb
    - run: npm ci
    - name: Install tauri-driver
      run: npm install -g @crabnebula/tauri-driver
    - name: Build Tauri app (release mode)
      run: cargo tauri build
      env:
        ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_TEST_KEY }}
    - name: Run Tauri-driver E2E (Linux with Xvfb)
      if: matrix.os == 'ubuntu-latest'
      run: xvfb-run --auto-servernum npm run test:e2e:tauri
    - name: Run Tauri-driver E2E (Windows native)
      if: matrix.os == 'windows-latest'
      run: npm run test:e2e:tauri
```

Linux uses `xvfb-run` to provide a virtual display for the WebKitGTK window. Windows runs natively (windows-latest has Edge WebView2 + msedgedriver pre-installed).

`secrets.ANTHROPIC_TEST_KEY` is a low-budget test key; the smoke test costs ~$0.001 per run against Haiku 4.5.

#### `tests/e2e/smoke.spec.ts` — delete `test.skip()` entries

Delete the four `test.skip()` blocks that pointed to Stage F. The remaining renderer-level Playwright tests (3 active tests covering page-loads + password-input-type + smoke-disabled-without-key) STAY. They cover a different layer than the new Tauri-shell tests.

#### `docs/build-prompts/retrospectives/M03-summary.md` — Phase Closeout summary

Per `docs/build-prompts/retrospectives/SUMMARY-TEMPLATE.md`. Sections (full template; fresh session populates):

- **Stage trail** (commits across A–F)
- **Aggregate scoring** (Process / Product / Pattern axes — mean across stages with std dev)
- **Cross-stage trends** (friction patterns, pattern wins, surprises, hard-gate violations)
- **Time-box accuracy** (calibrated 25–31h vs actual; M02 ratio was 0.7×)
- **Decisions to apply before next parent milestone** (CLAUDE.md updates; TEMPLATE.md updates; M04 stage prompts known constraints)
- **Verdict** (Sound | Sound-but-rough | Friction-heavy | Not-ready)
- **User-review notes**
- **Sign-off**

The Stage F fresh session has access to all five prior-stage retrospectives (`M03.A` through `M03.E-retrospective.md`) and aggregates them into the summary.

#### `docs/gap-analysis.md` — append M03 entry (immutable on commit)

Per `CLAUDE.md` §20 + the new v1.2 `<gotchas_graduation>` subsection requirement. Six required sections + the new subsection:

```markdown
## M03 — Live Graph (2026-MM-DD, commit `<sha>`)

### 1. Codebase deep dive (cumulative across M01 + M02 + M03)

[200–500 words. M01 drone foundation; M02 event pipeline; M03 live graph
+ VDR projection + persistence + Tauri-shell E2E. What's structurally
sound, what's accumulated debt, what M03 added that previous milestones
couldn't have shipped without.]

### 2. Adherence to spec

✅ items: [list with file:line cites — e.g., `crates/runtime-drone/src/vdr.rs::project_signal:42 — VDR projection per spec §2b`]
⚠️ items: [list with rationale]
❌ items: [list — should be empty for M03]

### 3. Spec review forward-looking

Missing: [things spec doesn't say but should]
Contradicted: [things spec says one way but code does differently and code is right]
Ambiguous: [things spec leaves open that need clarifying for M04+]

### 4. Fix backlog

🔴 Critical: [list — should be empty for M03]
🟡 Important: [list with target milestone]
🟢 Nice-to-have: [list]

### 5. Carry-forward from prior milestones

[Status updates on M01 + M02 entries' open items. Format per CLAUDE.md §20:
"M01 🟡 'X' — RESOLVED at <commit>" or "M02 🟡 'X' — STILL OPEN, target M0Y"]

### 6. Gotchas graduation (v1.2 protocol addition)

Per STAGE-PROMPT-PROTOCOL.md v1.2 — every per-stage <gotchas> entry
across A–F gets a disposition.

| Stage | Gotcha | Disposition | Target/Resolution |
|---|---|---|---|
| A | secrecy/serde feature drop | RESOLVED | M03.A commit <sha> dropped feature |
| A | TS codegen mechanism choice | RESOLVED | M03.A chose Node CLI shell-out |
| ... | (all 28+ gotchas across stages) | ... | ... |

### 7. Sign-off

Hard gates G1–G5 cleared in all six M03 stages; <count> Critical-severity
findings (target: zero). Per CLAUDE.md §20 this entry is **immutable**
once committed.

— Claude (Stage F fresh session); user-approved <date>.
```

The fresh Stage F session populates this from all six per-stage retrospectives + the actual M03 codebase state. Pre-commit verification: `git show origin/main:docs/gap-analysis.md` produces the prior content; `head -n <prior-line-count> docs/gap-analysis.md` MUST match byte-for-byte (after CRLF/LF normalization). The new content is APPEND-ONLY at the bottom.

CI's `gap-analysis.md append-only check` job (M01.E-shipped) verifies on every PR push.

#### Aspirational note (post-PR #47)

The Tauri 2.x WebDriver setup specified above is non-trivial against wdio v9. PR #47 disabled the `e2e-tauri-driver` CI job after three iterations on the capabilities object (`'edge' / 'webkit2gtk'` → `'wry'` → omit per the official docs) failed to clear both Linux and Windows for independent reasons (Linux: tauri-driver could not exec the built app binary; Windows: msedgedriver not on PATH). Upstream compat between wdio v9 and tauri-driver 2.x is tracked in tauri-apps/tauri#10670 and tauri-apps/tauri#9203; the only confirmed-working community example pins wdio@7. Future readers should treat §F.3 as **aspirational** — check the current status of those issues (and consider downgrading to wdio@7 if they remain open) before re-enabling the CI job in M04.

### F.4 Tests

#### Pedantic-pass preflight

Stage F adds new TypeScript files (`wdio.conf.ts`, `tests/e2e-tauri/*.e2e.ts`). Apply `docs/gotchas.md` #21 + Stage B/C/D/E TS traps. Stage F specifics:

- [ ] `tsc --noEmit` clean across the new test files
- [ ] No `as any` in WebdriverIO capability casts (use `WebdriverIO.Capabilities` typed cast where needed)
- [ ] No `@ts-ignore` to suppress mocha global types — use `@types/mocha` properly
- [ ] WebdriverIO v9 config uses the v9 `Options.Testrunner` type (NOT v8's `WebdriverIO.Config`)

ARIA + a11y on the E2E tests:
- [ ] Selectors prefer `aria-label` / `role` queries over `data-testid` when possible (matches the existing Vitest+RTL test idiom; future a11y refactor doesn't break tests)
- [ ] No tests rely on visual styling alone (color encoding for status — assert via `data-status` attribute, not CSS color)

#### Default test plan for stages adding a new safety primitive

Stage F doesn't add a new safety primitive in code — the closeout artifacts (gap-analysis entry + M03-summary) are documentation. The Tauri-driver E2E suite is a new test type, not a primitive; coverage targets don't apply.

#### Test plan (Stage F)

`tests/e2e-tauri/smoke.e2e.ts` — 6 E2E tests (per F.3 above):

1. App launches + SetupPanel visible
2. Save-key flow + ✓ indicator
3. Graph renders after smoke (real Anthropic API call)
4. Click AgentNode → InspectorPanel opens
5. SQL inspector executes SELECT
6. Reload reconstructs graph

Test data dependencies:
- Test 3 needs `ANTHROPIC_TEST_KEY` in CI secrets (low-budget; ~$0.001 per CI run × 2 OS = ~$0.002 per PR run)
- Test 6 depends on Test 3 having populated the signal log — ordering matters; tests run in declared order via mocha's default

Local development (without CI secrets):
- Tests 1, 2, 4 (with seeded data), 5 (with seeded data) work without an API key
- Test 3 + 6 require a real key in keychain; manual run only

Skipped on macOS via the `wdio.conf.ts` `process.platform === 'darwin'` early-exit. The CI `e2e-tauri-driver` job skips macOS-latest entirely.

#### Coverage target

Stage F doesn't change coverage gates:

- Workspace Rust: ≥80% (preserved)
- runtime-drone: ≥95% (preserved)
- runtime-main: ≥95% (preserved)
- src-tauri: 50% patch gate (preserved)
- src/ frontend: ≥80% (preserved)
- E2E tests are NOT measured for coverage (they exercise the built binary; coverage tools don't instrument the release build)

**Doc-to-CI invariant.** Stage F adds a new CI job (`e2e-tauri-driver`) but no new code-coverage gates. The new job's pass/fail is the gate — no coverage threshold applies.

### F.5 CLI Prompt

Stage F is the closeout — uses `<closeout_stage_prompt>`, not `<work_stage_prompt>`. Per STAGE-PROMPT-PROTOCOL.md v1.2 §8.

```xml
<closeout_stage_prompt id="M03.F">
  <context>
    Closeout stage of M03 (Live Graph). Stages A–E have committed on the
    milestone branch claude/m03-live-graph-authoring (then renamed
    claude/m03-live-graph at execution-branch creation). This stage
    produces TWO things in ONE commit:

    1. Tauri 2.x desktop-shell E2E suite (tauri-driver + WebdriverIO v9;
       Linux + Windows matrix; macOS unsupported per upstream + MVP §M3
       out-of-scope). Six E2E tests covering renderer-load + save-key +
       smoke + graph-render + inspector + sql-inspector + replay
       reconstruction. Deletes the four test.skip() carry-forwards from
       M02.E's tests/e2e/smoke.spec.ts.

    2. M03 Phase Closeout artifacts per CLAUDE.md §20: M03-summary.md
       aggregating per-stage retrospectives + docs/gap-analysis.md
       APPEND-ONLY entry with six required sections + the new v1.2
       protocol's <gotchas_graduation> subsection (each per-stage
       gotcha across A–F gets disposition: kept | graduated | resolved
       | expired).

    The gap-analysis commit is the FINAL commit on the parent-milestone
    branch and gates the PR push. The entry is IMMUTABLE once committed.
  </context>

  <read_first>
    <file>CLAUDE.md (especially §3.4 Gap Analysis Entry + §4.7 Do-Not-Commit Rule + §20)</file>
    <file>STAGE-PROMPT-PROTOCOL.md (v1.2 — closeout-only tags + gotchas_graduation requirement)</file>
    <file>docs/build-prompts/M03-live-graph.md (Stage F sections F.1–F.4)</file>
    <file>agent-runtime-spec.md §3 + §13 (privacy — no telemetry in CI E2E)</file>
    <file>docs/MVP-v0.1.md §M3 acceptance criteria</file>
    <file>docs/gotchas.md (especially #23 tauri-driver + WebdriverIO matrix; #25 Vite root; #27 Vitest+RTL — applies analogously to WebdriverIO selectors)</file>
    <file>docs/build-prompts/retrospectives/SUMMARY-TEMPLATE.md</file>
    <file>docs/gap-analysis.md (read existing M01 + M02 entries; verify their content survives untouched in your final diff)</file>
  </read_first>

  <read_reference>
    <file purpose="Tauri 2.x WebDriver setup canonical reference">https://v2.tauri.app/develop/tests/webdriver/</file>
    <file purpose="WebdriverIO v9 + Tauri example config reference">https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/</file>
    <file purpose="Tauri WebDriver CI configuration reference">https://v2.tauri.app/develop/tests/webdriver/ci/</file>
    <file purpose="M02.E test.skip() rationale entries to delete">tests/e2e/smoke.spec.ts</file>
    <file purpose="renderer-level Playwright config + smoke test (preserved unchanged)">playwright.config.ts</file>
    <file purpose="existing CI workflow with frontend + e2e jobs to extend">.github/workflows/ci.yml</file>
    <file purpose="M01-summary archetype for the M03-summary.md shape">docs/build-prompts/retrospectives/M01-summary.md</file>
    <file purpose="M02-summary archetype for the M03-summary.md shape">docs/build-prompts/retrospectives/M02-summary.md</file>
    <file purpose="M01 + M02 gap-analysis entries — APPEND BELOW; do not modify">docs/gap-analysis.md</file>
  </read_reference>

  <cumulative_reads>
    <codebase>entire shipped codebase to date (M01.A–M01.D + M02.A–M02.F + M03.A–M03.E commits on the milestone branch + this stage's E2E additions)</codebase>
    <spec>spec/agent-runtime-spec.md (end-to-end, focus on §3 Live Graph + §2b VDR + §1d IPC + §13 Privacy that M03 touches)</spec>
    <gap_analysis>docs/gap-analysis.md (M01 + M02 entries; M03.F appends the third)</gap_analysis>
    <retrospectives>docs/build-prompts/retrospectives/M03.A-retrospective.md, M03.B-, M03.C-, M03.D-, M03.E-retrospective.md (all five work stages)</retrospectives>
  </cumulative_reads>

  <deliverables>
    <milestone_summary>docs/build-prompts/retrospectives/M03-summary.md (aggregates per-stage retrospectives; scores axes across stages; marks verdict; ~250 lines per M01-summary archetype)</milestone_summary>
    <gap_analysis_entry>docs/gap-analysis.md (append M03 entry; six required sections + new <gotchas_graduation> subsection per v1.2; APPEND-ONLY; ~200 lines)</gap_analysis_entry>
    <e2e_suite>tests/e2e-tauri/smoke.e2e.ts + wdio.conf.ts + .github/workflows/ci.yml e2e-tauri-driver job (Linux + Windows; macOS skipped)</e2e_suite>
    <pr_description>draft only; PR opens only on explicit human ask after approval</pr_description>
  </deliverables>

  <gap_analysis_requirements ref="CLAUDE.md" section="20. Gap Analysis Protocol (append-only, per-milestone)">
    <gotchas_graduation>
      <stage_review id="A">
        <gotcha>scope creep into Stage B's React Flow work — resist</gotcha>
        <disposition>resolved</disposition>
        <target>M03.A commit (held the line; A landed without React Flow code)</target>
      </stage_review>
      <stage_review id="A">
        <gotcha>secrecy/serde feature drop side effects — verify no callsite breaks</gotcha>
        <disposition>resolved</disposition>
        <target>M03.A commit (grep verified zero callsites; feature dropped cleanly)</target>
      </stage_review>
      <stage_review id="A">
        <gotcha>TS codegen drift — output must be byte-stable across re-runs</gotcha>
        <disposition>kept</disposition>
        <target>per-stage <gotchas> in M04+ when xtask Rust list extends to event.v1.json (deferred from Stage D)</target>
      </stage_review>
      <stage_review id="B">
        <gotcha>nodeTypes map MUST be defined outside the component (re-render trap)</gotcha>
        <disposition>graduated</disposition>
        <target>docs/gotchas.md §23 (or new entry — check Stage F authoring)</target>
      </stage_review>
      <stage_review id="B">
        <gotcha>Zustand v5 selector pattern (use selectors, not bare hook)</gotcha>
        <disposition>graduated</disposition>
        <target>docs/gotchas.md §X — new entry per protocol-iteration session</target>
      </stage_review>
      <!-- ... full graduation matrix populated by the fresh session ...
           28+ entries covering all per-stage gotchas A–F plus this Stage F's
           own gotchas. Each is one of: kept (still applies forward, stays in
           per-stage <gotchas>), graduated (recurred 2+ times, promoted to
           docs/gotchas.md), resolved (fixed by code change, cite commit),
           expired (stage-local with no forward applicability, cite rationale). -->
    </gotchas_graduation>
  </gap_analysis_requirements>

  <append_only_verification>
    <local_check>prior content of docs/gap-analysis.md (M01 + M02 entries; lines 1–<count>) MUST be a literal prefix of HEAD before commit. Verify via: `git show origin/main:docs/gap-analysis.md > /tmp/prior.md && head -n <count> docs/gap-analysis.md | tr -d '\r' > /tmp/head.md && diff -q /tmp/prior.md /tmp/head.md` — must report identical.</local_check>
    <ci_check name="gap-analysis.md append-only check">added in M01.E; verifies on every PR push that prior entries remain byte-identical to merged main.</ci_check>
  </append_only_verification>

  <three_artifact_review>
    <artifact>code diff (cumulative M03.A through M03.F)</artifact>
    <artifact>per-stage retrospectives (M03.A–E) + M03 milestone summary (M03-summary.md, new in F)</artifact>
    <artifact>new docs/gap-analysis.md M03 entry — flagged "IMMUTABLE once committed"</artifact>
    <pushback_blocks_pr>true</pushback_blocks_pr>
  </three_artifact_review>

  <scope_locks>
    <lock>Append-only is a hard rule (CLAUDE.md §4.1, §4.6, §20) — no editing M01 or M02 prior entries, ever. CI enforces.</lock>
    <lock>The <gotchas_graduation> subsection MUST list every prior stage of M03 (A–F), even those whose <gotchas> were "None observed." Per STAGE-PROMPT-PROTOCOL.md v1.2 validator rules.</lock>
    <lock>tauri-driver does not support macOS — DO NOT add macos-latest to the new e2e-tauri-driver CI matrix. The renderer-level Playwright tests cover macOS via the existing e2e job; that's the cross-OS coverage path for v0.1.</lock>
    <lock>M03 PR push happens AFTER user approves this Stage F commit. Per CLAUDE.md §8, no push without user approval; per §20, the gap-analysis commit is the FINAL commit on the branch and gates the push.</lock>
  </scope_locks>

  <gates milestone="M03"/>

  <self_correction_budget>3</self_correction_budget>

  <gotchas>
    <trap>The "Carry-forward" section is required even when empty — write "None observed." rather than omit (CLAUDE.md §3.4). Same for "Spec review forward-looking" sections (Missing/Contradicted/Ambiguous).</trap>
    <trap>Severity is non-elastic — if M03 has a pile of 🔴 Criticals in the fix backlog, the milestone shouldn't ship; surface this rather than rationalize down to 🟡. The user's review verdict on this entry is the milestone's go/no-go gate.</trap>
    <trap>The <gotchas_graduation> disposition `expired` requires rationale beyond bare `n/a` per STAGE-PROMPT-PROTOCOL.md v1.2 — explain WHY the gotcha doesn't apply forward (validator checks length).</trap>
    <trap>tauri-driver's tests/e2e-tauri/ directory MUST NOT be confused with tests/e2e/ (Playwright). Separate test types, separate CI jobs, separate config files. Don't move or merge them.</trap>
    <trap>The wdio.conf.ts macOS early-exit (process.exit(0)) ensures `npm run test:e2e:tauri` is a no-op on macOS rather than a hard failure. Without this guard, macOS local development breaks. CI's macos-latest skips the whole e2e-tauri-driver job, so the guard is for local dev.</trap>
    <trap>Three-artifact review pushback BLOCKS the PR push (pushback_blocks_pr: true). If the user pushes back on the gap-analysis content during review, revise; do NOT push the branch until all three artifacts have user approval.</trap>
  </gotchas>

  <execution_warnings>
    <warning>DO NOT edit prior gap-analysis.md entries (M01 or M02). Append at the bottom only. CI's append-only check fails the PR if prior content shifts even by one byte (after CRLF/LF normalization).</warning>
    <warning>DO NOT skip the &lt;gotchas_graduation&gt; subsection. The v1.2 protocol validator (when shipped) will reject the entry without it. Manual authoring per the table format in F.3 above.</warning>
    <warning>DO NOT push the branch in the same approval round as the Stage F commit. CLAUDE.md §8 + §20 require: commit Stage F (user approves) → user reviews three artifacts (code + retros/summary + gap-analysis) → user approves push → branch pushed → PR draft surfaced for SEPARATE re-approval.</warning>
    <warning>DO NOT add new code in Stage F beyond the E2E suite + wdio.conf.ts + CI workflow extension. The closeout is doc-heavy by design; new code in Stage F sneaks past the per-stage retro discipline.</warning>
  </execution_warnings>

  <time_box estimate_hours="5.5"/>

  <retrospective_requirements ref="docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md" section="M[NN].&lt;X&gt; — Stage Retrospective">
    <special_log>Stage F retro is the M03.F-retrospective.md AND the M03-summary.md is created in this stage too — but they're DIFFERENT files. M03.F-retrospective.md covers ONLY this closeout stage's process (was the tauri-driver setup smooth, did the gap-analysis authoring hit any walls, was the CI workflow extension straightforward); M03-summary.md aggregates ALL six stages. Don't conflate them.</special_log>
  </retrospective_requirements>

  <commit_protocol ref="CLAUDE.md" section="8. PR + commit workflow (CRITICAL — read carefully)"/>
  <commit_message ref="docs/build-prompts/M03-live-graph.md" section="F.6 Commit Message"/>

  <approval_surface>
    <item>diff stat (git diff --stat HEAD)</item>
    <item>gate results (each gate, pass/fail, key numbers including new e2e-tauri-driver CI job results — green on Linux + Windows)</item>
    <item>M03.F-retrospective.md filled in (Stage F retro — covers ONLY this closeout stage)</item>
    <item>M03-summary.md filled in (aggregate across all six stages)</item>
    <item>gap-analysis.md M03 entry full text (six sections + gotchas_graduation subsection)</item>
    <item>append-only verification output (diff -q on prior content; CI check status)</item>
    <item>draft commit message from M03-live-graph.md F.6 Commit Message section</item>
    <item>screenshot or paste of: (a) successful e2e-tauri-driver CI run; (b) the rendered graph + inspector + SQL inspector + replay reconstruction in the built Tauri app; (c) M03 PR description draft (will be opened only after user re-approval)</item>
    <item>explicit flag: "This gap-analysis entry is IMMUTABLE once committed. Please review the M03 entry text carefully — once the commit lands and the PR pushes, future milestones can only report status via Carry-forward sections; the M03 entry itself is locked forever."</item>
    <item>explicit statement: "M03 closeout is ready. I will not commit until you approve. After commit, I will not push until you re-approve. After push, I will not open the PR until you ask."</item>
  </approval_surface>
</closeout_stage_prompt>
```

### F.6 Commit Message

```
docs+test+ci(m03): Stage F — Tauri-driver E2E + Phase Closeout

Final commit on the M03 milestone branch. Two workstreams in one commit:

Tauri 2.x desktop-shell E2E suite:
- New tests/e2e-tauri/ directory; 6 tests covering renderer-load,
  save-key, smoke happy-path (real Anthropic API call), click-to-
  inspect, SQL inspector execute, reload reconstructs.
- New wdio.conf.ts (WebdriverIO v9; tauri-driver as service; Linux
  webkit2gtk + Windows edge capabilities; macOS early-exit since
  tauri-driver is upstream-unsupported on macOS per
  v2.tauri.app/develop/tests/webdriver/).
- New CI job e2e-tauri-driver (Linux + Windows matrix; xvfb on
  Linux for headless WebKitGTK; uses ANTHROPIC_TEST_KEY secret
  for the smoke test costing ~$0.001 per CI run × 2 OS).
- Deletes the four test.skip() carry-forwards from M02.E in
  tests/e2e/smoke.spec.ts (the renderer-level Playwright tests
  stay; they cover a different layer with faster feedback).

M03 Phase Closeout per CLAUDE.md §20:
- New docs/build-prompts/retrospectives/M03-summary.md per
  SUMMARY-TEMPLATE.md (aggregate axis scores across A–F; cross-
  stage trends; pattern wins + surprises; time-box accuracy
  vs the calibrated 25–31h estimate; 8–12 explicit decisions for
  M04.1).
- Append M03 entry to docs/gap-analysis.md per CLAUDE.md §20.
  Six required sections (codebase deep dive cumulative across
  M01+M02+M03; adherence ✅/⚠️/❌ with file:line; spec review
  forward-looking; fix backlog 🔴/🟡/🟢; carry-forward from M01+M02;
  sign-off) plus the new v1.2 protocol's <gotchas_graduation>
  subsection auditing all 28+ per-stage gotchas across A–F with
  disposition (kept | graduated | resolved | expired).

This commit is the FINAL commit on claude/m03-live-graph per
CLAUDE.md §20. The gap-analysis entry is IMMUTABLE once committed.
Push gates the M03 PR (separate user approval per CLAUDE.md §8 +
§20).

Refs: M03-live-graph.md §F; agent-runtime-spec.md §3 + §13;
CLAUDE.md §20; STAGE-PROMPT-PROTOCOL.md v1.2 (closeout schema +
gotchas_graduation subsection); docs/gotchas.md #23 (tauri-driver
matrix); MVP-v0.1.md §M3 acceptance criteria.

https://claude.ai/code
```

---

## Summary Table

| Stage | Title | Calibrated effort | Key deliverables |
|---|---|---|---|
| A | Build hygiene + carry-forward closures + new deps | ~2–3h | Vite 5→7 bump; secrecy/serde drop; @xyflow/react + zustand deps; src/counter.{js,test.js} delete; current_exe() drone test retrofit; schemas/event.v1.json + xtask TS codegen; vitest --coverage default |
| B | React Flow + Zustand foundation + 3 basic node types | ~5–6h | graphStore.ts (Zustand store with applyEvent reducer); GraphCanvas wrapper; AgentNode + ToolNode + SkillNode custom node components; ~30 tests |
| C | Remaining 8 node types + animated edges + color encoding | ~5–6h | MCPNode + GapNode + HITLNode + PlanNode + TaskNode + VerifyNode + HookNode + FrameworkNode + Edge.animated lifecycle + spec §3 color palette via CSS custom properties; ~43 tests; synthetic-state testing pattern locked |
| D | Click-to-inspect side panel + token-spend visualization + zoom/pan | ~4–5h | InspectorPanel (ARIA dialog) + layout.ts (dagre) + token-weight CSS scaling + MiniMap + schemas/event.v1.json bump for tokens + Rust event.rs + event_pipeline.rs token surfacing; ~16 tests |
| E | VDR projection + SQL inspector + graph persistence | ~5–6h | Drone-internal vdr.rs continuous projector + SqlInspector renderer component + parser-based SELECT-only validation + replay-from-signals on app mount + replay.rs (signal→event translator); ~17 tests |
| F | Tauri 2.x desktop-shell E2E + Phase Closeout | ~5–6h | tauri-driver + WebdriverIO v9 + 6 E2E tests + new e2e-tauri-driver CI job (Linux + Windows; macOS skipped) + M03-summary.md + immutable gap-analysis.md M03 entry with <gotchas_graduation> |

Total calibrated: 25–31h. Expected actual at M02's 0.7× ratio: 17–22h.

## Verification Checklist

Per CLAUDE.md §16 session-start checklist, applied at M03 PR open time. The user verifies before merging:

- [ ] All 6 stages committed on `claude/m03-live-graph` (or whatever branch the user opens with at M03 execution time)
- [ ] All 6 per-stage retrospectives present at `docs/build-prompts/retrospectives/M03.{A,B,C,D,E,F}-retrospective.md`
- [ ] M03-summary.md present at `docs/build-prompts/retrospectives/M03-summary.md`
- [ ] M03 gap-analysis entry appended to `docs/gap-analysis.md` (CI append-only check passes)
- [ ] All 11 spec §3 node types render correctly in unit tests
- [ ] graphStore.ts + layout.ts + vdr.rs + replay.rs all ≥95% line coverage (new primitives)
- [ ] Frontend coverage ≥80% on src/
- [ ] Workspace Rust coverage ≥80%; runtime-drone + runtime-main ≥95%; src-tauri 50% patch gate
- [ ] CI green across all 3 OS targets × stable + MSRV (Rust matrix); frontend job; e2e renderer job; e2e-tauri-driver job (Linux + Windows; macOS skipped)
- [ ] Codecov delta gates pass (project + patch + per-flag)
- [ ] cargo audit + cargo deny clean
- [ ] No new third-party deps without ADR per CLAUDE.md §6 (Stage A added @xyflow/react + zustand + json-schema-to-typescript; Stage D added @dagrejs/dagre; Stage F added @crabnebula/tauri-driver + @wdio/* — all justified inline)
- [ ] schemas/event.v1.json bumped (Stage D) + xtask drift check passes
- [ ] No telemetry / phone-home additions; no secrets in commits
- [ ] M02 smoke test still works end-to-end (regression check); now produces a graph instead of an event list
- [ ] Three-artifact review by user: code diff + retros/summary + gap-analysis entry — pushback on any blocks merge

---

## Summary Table

*Populated in the final chunk before the M03 PR opens.*

## Verification Checklist

*Populated in the final chunk before the M03 PR opens.*
