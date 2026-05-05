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
