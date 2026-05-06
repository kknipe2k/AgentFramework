# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — M03.C (Remaining 8 node types + animated edges + color encoding)

Lights up the rest of spec §3's node-type set. After Stage C, all 11 node types ship as renderable components: AgentNode, ToolNode, SkillNode (Stage B) + MCPNode, GapNode, HITLNode, PlanNode, TaskNode, VerifyNode, HookNode, FrameworkNode (this stage). graphStore.applyEvent extended for the two events that already exist in the v0.1 schema: `session_start` → FrameworkNode (graph root); `tool_invoked` with `source='mcp'` → lazy parent MCPNode. The remaining six components (Gap, HITL, Plan, Task, Verify, Hook) ship as renderable but their event-driven wiring lands at M4 (plan/task/verify/hook events) and M5 (gap events) when the schema gains those variants.

- **`src/components/nodes/{MCPNode,GapNode,HITLNode,PlanNode,TaskNode,VerifyNode,HookNode,FrameworkNode}.tsx` (NEW)** — eight new React Flow custom-node components mirroring the Stage B AgentNode archetype (`Handle` + `Position` + `data-testid` + `data-status` + ARIA label). Two specialize: HITLNode is `role="alert"` + `aria-live="assertive"` per WAI APG (blocking input affordance); GapNode adds `data-kind` (`tool_missing` / `skill_missing`) + the `gap-node--gap` class drives the `@keyframes gap-pulse` animation per spec §3 Behavior ("GapNode appears immediately on tool_missing"). FrameworkNode is the graph root — source handle only (no upstream parent in v0.1).
- **`src/lib/graphStore.ts`** — extended:
  - Eight new data interfaces (`MCPNodeData`, `GapNodeData`, `HITLNodeData`, `PlanNodeData`, `TaskNodeData`, `VerifyNodeData`, `HookNodeData`, `FrameworkNodeData`) plus typed `Node<...>` aliases.
  - `GraphNode` discriminated union grown from 3 → 11 variants. `EdgeData.kind` enum gains `'agent-mcp'`.
  - `applyEvent('session_start')` promoted from no-op to spawn FrameworkNode at root with id `framework:<name>`. Idempotent on duplicate session_start.
  - `applyEvent('tool_invoked')` extended: `source: 'mcp'` + `server` set lazily spawns an MCPNode with id `mcp:<server>` and wires agent → MCP + MCP → tool edges (NOT agent → tool); same MCP server reused across multiple tools (one MCPNode + one agent→MCP edge + one MCP→tool edge per tool). Non-MCP tools keep Stage B's agent → tool routing.
  - Animated-edge state machine: every `tool_invoked`-created edge has `animated: true`; `tool_result` clears the flag on the inbound edge (matches by target so both agent→tool and MCP→tool shapes resolve uniformly).
  - Coverage: 96.37%+ preserved on the safety primitive.
- **`src/components/GraphCanvas.tsx`** — `nodeTypes` map grown from 3 → 11 entries (one per spec §3 node type). Map definition kept at module level per @xyflow/react v12 docs (Stage B trap re-applies with 11 types — inline definition triggers per-render remount).
- **`src/styles.css`** — extended:
  - Spec §3 Visual Design palette in `:root` CSS custom properties (`--node-active`, `--node-complete`, `--node-error`, `--node-gap`, `--node-hitl` + base bg/border/fg). Existing AgentNode + ToolNode rules refactored to use `var(--node-...)` so future stages adjust the palette in one place.
  - Eight new node-type style blocks each with `--<status>` modifiers (Plan/Task/Verify use type-specific status enums per spec §3a + §4a; MCP/Hook/Framework use the shared `active/complete/error` palette).
  - GapNode `gap-pulse` keyframe (1.4s amber pulse) + HITLNode bright/white modifier per spec §3.
  - `.react-flow__edge.animated` keyframe (`dash-flow` 1s linear) for active-call animation; `.react-flow__edge--dashed` static dashed style for skill-load edges.
- **Tests** — `tests/unit/graphStore.test.ts` (7 new tests: `session_start_spawns_FrameworkNode_at_root` + idempotent; MCP lazy spawn + reuse across tools; animated-edge lifecycle on `tool_invoked`/`tool_result` for both agent→tool and MCP→tool shapes); `tests/unit/nodes/{MCP,Gap,HITL,Plan,Task,Verify,Hook,Framework}Node.test.tsx` (5 tests each = 40 new component tests; HITLNode + GapNode + FrameworkNode have specialized assertions per their spec §3 specializations); `tests/unit/App.test.tsx` updated to assert FrameworkNode lands when `session_start` arrives in the smoke happy-path.
- **Synthetic-state testing pattern locked.** Tests for the six event-less components (Gap, HITL, Plan, Task, Verify, Hook) pass populated state directly to `<NodeComponent>` rather than dispatching events through the store. M4+ wires events to these components without renderer-test churn.

Refs: `docs/build-prompts/M03-live-graph.md` §C; `agent-runtime-spec.md` §3 (Node Types + Behavior + Visual Design); `docs/MVP-v0.1.md` §M3; `docs/gotchas.md` #21 + #27.

### Added — M03.B (React Flow + Zustand foundation + 3 basic node types)

Lays the foundation for the live graph. Replaces M02's flat `<ul>` event list with a React Flow canvas backed by a Zustand store. Three of the eleven spec §3 node types ship: AgentNode, ToolNode, SkillNode. The remaining eight (MCP, Gap, HITL, Plan, Task, Verify, Hook, Framework) land in Stage C.

- **`src/lib/graphStore.ts` (NEW)** — Zustand v5 store; the canonical source of graph state. Exports `applyEvent(event)`, `clear()`, `selectNode(id)` actions plus `nodes` / `edges` / `selectedNodeId` slices. `applyEvent` is the single entry point for translating `AgentEvent` into node + edge mutations. Idempotent on duplicate events; exhaustive over the 36-variant discriminated union via TS `_exhaustive: never` check. Stage B handles 6 variants as render mutations (`agent_spawned` + parent edge; `agent_complete`/`agent_error` status flips; `tool_invoked` + edge; `tool_result` complete + duration; `skill_loaded` + dashed edge); the remaining 30 are explicit no-ops Stage C/D/M4+ light up. Coverage: 96.37% line.
- **`src/components/nodes/AgentNode.tsx` (NEW)** — React Flow v12 custom node with `Handle` + `Position` primitives. Renders agent name + 8-char-truncated id + status class. ARIA-labeled. `data-testid` + `data-status` for E2E selectability (Stage F).
- **`src/components/nodes/ToolNode.tsx` (NEW)** — same shape, renders tool name + duration (when complete).
- **`src/components/nodes/SkillNode.tsx` (NEW)** — dashed outline (`skill-node--dashed` class) per spec §3 Behavior; no flow animation. Renders skill name + mode-variant (when present).
- **`src/components/GraphCanvas.tsx` (NEW)** — wraps `<ReactFlow>` from `@xyflow/react`; subscribes to the store via Zustand selectors (`useGraphStore((s) => s.nodes)` form) so re-renders trigger only on the relevant slice change. `nodeTypes` map defined at module level per @xyflow/react v12 docs (inline definition forces per-render remounts and kills the streaming UX). Includes `<Background />` + `<Controls />`. `onNodeClick` / `onPaneClick` wired to `selectNode` for Stage D's inspector seam.
- **`src/App.tsx`** — refactored: Zustand store replaces the M02 `useReducer`. SetupPanel + SmokeButton + handleSetKey + handleSmoke + `console.error` + `unwrapCmdError` preserved verbatim. Heading flipped from "M02 smoke" to "M03 live graph".
- **`src/styles.css`** — appended graph canvas + 3 node-type styles per spec §3 Visual Design (dark background, color-encoded status: blue=active, green=complete, red=error; dashed SkillNode outline). M02 component styles preserved.
- **Tests** — `tests/unit/graphStore.test.ts` (13 tests covering each Stage B AgentEvent branch + idempotence + clear/select + an exhaustive no-op coverage test for the other 27 schema variants); `tests/unit/nodes/{Agent,Tool,Skill}Node.test.tsx` (5 tests each: render + status classes + accessibility + handles); `tests/unit/App.test.tsx` refactored to assert on `useGraphStore.getState().nodes` instead of listitem count; `tests/unit/components.test.tsx` refactored — dropped EventList tests, added GraphCanvas empty-state smoke. 41 frontend tests pass; coverage 93.47% global, 96.37% on graphStore primitive.
- **Deletions** — `src/lib/eventReducer.ts`, `src/components/EventList.tsx`, `tests/unit/eventReducer.test.ts` (replaced by graphStore architecture).

Refs: `docs/build-prompts/M03-live-graph.md` §B; `agent-runtime-spec.md` §3; `CLAUDE.md` §5 (TDD discipline) §14 (schemas-as-source-of-truth — schema's snake_case field names used throughout); `docs/gotchas.md` #21 (clippy traps — N/A for TS), #25 (Vite root — preserved), #26 (serde tag-shape — N/A for TS), #27 (Vitest+RTL DOM-ref staleness — observed via `act()` wrap of synchronous Zustand dispatch in App.test.tsx).

### Added — M03.A (Build hygiene + carry-forward closures + new deps)

Closes the M02 🟡 Important carry-forward items + adds the deps Stages B–F need. No React Flow code yet; that lands in Stage B. Per `docs/gap-analysis.md` M02 entry §"Carry-forward to M03 prep" + `M02-summary.md` §"Decisions to apply before the next parent milestone".

- **`schemas/event.v1.json` (NEW)** — canonical AgentEvent schema covering all variants of `runtime_core::event::AgentEvent` (session/agent/tool/skill/plan/task/mode/verify/rails/gap/HITL/capability/budget/stream/decision/token + `ToolSource` enum). Source-of-truth for renderer TypeScript types per `CLAUDE.md` §14 schemas-as-source-of-truth. Replaces hand-mirrored `src/types/agent_event.ts`.
- **`crates/xtask/src/main.rs`** — extends `regenerate-types` + `regenerate-types --check` to also generate TypeScript types via `npx --yes json-schema-to-typescript`. New testable seam `regenerate_typescript_types_with(schemas, output_dir, runner, check)` mirrors the M01.C / M02.C / M02.D / M02.E `*_with` archetype; production wires `runner = run_npx_json_schema_to_typescript`. Drift list merges with the existing typify Rust-codegen drift list so a single bail message covers both Rust and TS regressions.
- **`src/types/agent_event.ts`** — regenerated. Hand-mirrored content replaced by `cargo xtask regenerate-types` output. Header banner makes the generated nature explicit; `.prettierignore` + `eslint.config.js` exclude the path so prettier/eslint don't fight the codegen formatter. The drift check in CI catches future divergence between schema and generated TS.
- **`crates/xtask/tests/check_drift.rs`** — Case 4 added: mutates `src/types/agent_event.ts`, runs `regenerate-types --check`, asserts non-zero exit, restores. Mirrors existing Case 3 for Rust drift.
- **`crates/runtime-drone/tests/integration.rs` + `integration_windows.rs`** — `drone_binary()` retrofitted to derive paths from `std::env::current_exe()` instead of `CARGO_MANIFEST_DIR`-relative `target/debug/runtime-drone`. Per `docs/gotchas.md` #22: `cargo llvm-cov --workspace` uses a distinct target dir that breaks hard-coded paths. Archetype: `crates/runtime-main/tests/drone_ipc_loopback.rs::drone_binary`.
- **`package.json`** — `"test"` script flipped from `vitest run` → `vitest run --coverage` so the 80% threshold in `vitest.config.ts` is enforced on every run (M02.E carry-forward — the threshold was configured but only triggered when `--coverage` was passed explicitly).
- **`src/counter.{js,test.js}`** — deleted. Legacy CommonJS files predating the M02 `"type": "module"` flip; were carried forward via `.prettierignore` + `eslint.config.js ignores`. The ignore-list entries are now removed.
- **Workspace `Cargo.toml`** — `secrecy` dropped the `serde` feature. Per `docs.rs/secrecy/0.10`: `SecretString` does NOT serialize via serde by default (the feature requires the `SerializableSecret` marker trait, which no M02 code implements). The feature was dead weight; verified by grep on `secrecy::Serialize` / `serialize_with` / `Deserialize` over `SecretString`.
- **`package.json`** — Vite 5.4 → ^7.1.0 (the dev-server esbuild advisory in 5.x is in the moderate-vulns chain that `npm audit --audit-level=high` already filters out, but the bump closes the M02.E surprise event 4 carry-forward). Vite 8 (Rolldown) is GA but out-of-scope per the M03 stage prompt's `<execution_warnings>`; defer to M04+.
- **`package.json`** — added `@xyflow/react ^12.10.0` + `zustand ^5.0.0` (production deps for Stages B–F React Flow + state management) + `json-schema-to-typescript ^15.0.0` (devDep used by the new xtask TS codegen). `keyring 3.6` stays per the M03 stage prompt's `<execution_warnings>` — 4.0 has breaking API surface and is deferred to a dedicated chore PR after M03 ships.

Refs: `docs/build-prompts/M03-live-graph.md` §A; `agent-runtime-spec.md` §3 §13.5; `CLAUDE.md` §5 §14; `docs/gotchas.md` #21–#28 (especially #22 `current_exe`); `M02-summary.md` §"Decisions to apply before the next parent milestone"; `docs/gap-analysis.md` M02 entry §"Carry-forward to M03 prep".

### Fixed — Post-M02 smoke-test live debugging

Live debugging a "[object Object]" smoke-test failure in the M02 desktop app surfaced four overlapping issues. All four are fixed here in one PR; the underlying spec/process gap (dev-logging discipline) is locked into the spec so future milestones don't repeat the silent-stub trap.

- **`Cargo.toml` — keyring 3.x platform features.** Bare `keyring = "3.6"` ships NO platform backend by default; the workspace dep was missing the `apple-native` / `windows-native` / `sync-secret-service` features. Result: the keyring crate compiled but used a stub backend that silently succeeded on writes and returned `NoEntry` on reads. Symptoms in M02 dev: "Save key ✓ stored in OS keychain" then `setup_required` on smoke test. Fix opts into all three OS backends (one-line change to the workspace dep). Per `docs/gotchas.md` #29.
- **`src/lib/ipc.ts` — typed `unwrapCmdError` helper.** Tauri renderer's `catch(e)` receives a serde-tagged JS object (e.g., `{type: "setup_required"}` or `{type: "provider", message: "..."}`); `e instanceof Error` is `false`; `String(e)` yields `"[object Object]"`. The new `unwrapCmdError(e: unknown): string` helper exhaustively handles `Error` instances, `CmdError` shape (with type + optional message), generic objects with `message`, and falls back to `String(e)`. Exported so M03+ command surfaces reuse it. Type definition matches the actual `CmdError` enum in `src-tauri/src/commands.rs`. Per `docs/gotchas.md` #30.
- **`src/App.tsx` — error logging at every `catch`.** Both `handleSetKey` and `handleSmoke` now `console.error('<context> error:', e)` before user-facing dispatch. Critical for diagnostics: without this, structured errors collapse to `"[object Object]"` in the UI with zero signal in the DevTools console. The change pairs with the `unwrapCmdError` helper — together they ensure every renderer-side error has a console log AND a user-readable string.
- **`src-tauri/src/main.rs` — `tracing_subscriber::fmt::init()`.** M02 wired `tracing::info!` / `tracing::error!` calls inside Tauri commands but never initialized the subscriber, so the calls emitted to a null sink. The fix adds an `init_tracing()` function called at the top of `main()` with `EnvFilter`-based level config (default `info` globally, `debug` for project crates; `RUST_LOG` overrides). Adds `tracing` + `tracing-subscriber` (with `env-filter`, `fmt` features) to `src-tauri/Cargo.toml`. Per `docs/gotchas.md` #31.
- **`src-tauri/src/commands.rs` — minimum-viable command-level instrumentation.** `set_api_key` and `run_smoke_session` now log entry (`info!`), failure paths (`error!` with `error = %e` + which sub-step), and success (`info!`). API key VALUES are never logged (only `key_len` for `set_api_key`); `SecretString` wrapping ensures `Debug` output is `[REDACTED]`. Per `agent-runtime-spec.md` §13.5 (new this PR).

### Documentation — Spec §13.5 + gotchas #29–#31

Locks the dev-logging discipline that the M02 live debugging exposed as a structural gap.

- **`agent-runtime-spec.md` §13.5 Dev Logging** — new subsection inside §13 Privacy & Telemetry. Documents the dev/release boundary (zero-telemetry remains in force), the `tracing_subscriber::fmt::init()` requirement at every Rust binary's `main()`, the per-Tauri-command instrumentation pattern (entry / success / error logs), the renderer-side `console.error` + `unwrapCmdError` pattern, the secrets-redaction invariant (`SecretString` for API keys; structural-only logging for user content), what release mode does differently (JSON formatter, log files at `$DATA_DIR/logs/{date}/`), and what dev mode does NOT do (no telemetry, no automatic diagnostics, no phone-home-on-crash). Includes the per-milestone logging-requirements gate that §13.5 reviews land in closeout stages.
- **`docs/gotchas.md`** — three new entries (#29, #30, #31) consolidating the M02 live debugging traps:
  - **#29** keyring 3.x stub backend (no platform features by default)
  - **#30** Tauri renderer's `catch(e)` gets non-Error objects from serde-tagged enums
  - **#31** Tauri main process binary needs `tracing_subscriber::fmt::init()` in `main()`

### Documentation — Post-M02 spec lock + ADR-0006

Per `M02-summary.md` Decisions + `docs/gap-analysis.md` M02 entry Fix
Backlog. Locks the M02 architectural decisions into the spec so M03+
implementations don't have to re-decide. Pairs with the
post-M02-protocol-iteration PR (gotchas + retrospective + template
carry-forwards) — both PRs are pre-M03 housekeeping.

- **`agent-runtime-spec.md` §2c LLMProvider Abstraction** — two new
  subsections locking the M02 SSE wire-format + ProviderEvent semantics:
  - **§2c.1 Anthropic SSE wire format** — full event-set table with
    payload + ProviderEvent mapping; specific call-out for
    `signature_delta` (verifier-only; consumed silently) and `ping` (SSE
    keep-alive; consumed silently). Pre-M02 spec drafts didn't document
    these; M02 implementation discovered them live and they tripped
    fresh implementations as "unknown event type" warnings.
  - **§2c.2 ProviderEvent::Error semantics** — locks `Error` as
    **terminal**: stream yields Error then terminates without
    MessageStop. Retry logic lives in AgentSdk task layer, not provider
    layer (cost-runaway + correctness rationale documented). Adds
    cancellation-safety language: provider stream is cancellation-safe;
    dropping mid-burst drops the underlying reqwest::Response.
- **`agent-runtime-spec.md` §2b Signals & VDR Projection** — adds a
  ⚠️ note flagging the `signal::ContextType` enum's divergence from
  spec's `context.type ∈ {skill, framework, code, search, verify,
  commit, subagent}` set. M02's runtime scaffold uses operation-context
  variants (`AgentLoop / SkillLoad / ToolInvoke / HookExecute /
  PlanCreate / HitlPrompt / SessionLifecycle`); reconciliation deferred
  to M04 closeout when emission integration provides evidence on which
  shape is correct.
- **`agent-runtime-spec.md` §1d IPC Channels** — new "Reconnect
  semantics" subsection documenting the 5-attempt 200ms→3.2s
  exponential backoff M02.D landed in `DroneClient::send_with_reconnect`,
  the `*_with` testable seam pattern, and the open M03-blocking
  question on long-lived events() subscription survival across
  reconnect.
- **`agent-runtime-spec.md` §10 Persistence Layer** — adds ⚠️ note
  alongside the `mcp_servers` table definition flagging the divergence
  from the documented 7-field shape (the shipped table is 22 fields)
  and pointing readers to ADR-0006 for the full rationale.
- **`agent-runtime-spec.md` Project Structure** — runtime-main module
  listing updated to reflect M02 actuals (sdk/, providers/, drone_ipc/,
  key_store.rs, etc.) plus per-file milestone tags (M02 / M04 / M05 /
  M06 / M06+ / M07 / M09) so readers can see what's shipped vs what's
  forward-looking.
- **`docs/adr/0006-mcp-servers-schema.md`** — new ADR (Accepted)
  documenting the 22-field `mcp_servers` schema's divergence from spec
  §10's 7-field shape. Per-field rationale table covers transport set
  (stdio/http/sse/streamable_http), stdio-vs-remote mutual-exclusion
  CHECK, OAuth refresh state persistence, capability discovery cache,
  scope/plugin_id, retry+timeout policy, lifecycle audit fields. Four
  alternatives rejected (match 7-field exactly; split tables; single
  JSON column; defer to M06 Stage A) with explicit reasoning. Target
  was "before M06 Stage A"; landed during post-M02 housekeeping.
- **`docs/MVP-v0.1.md` §M2 / §M3** — Tauri 2.x E2E framework note.
  §M2 Out-of-scope clarifies M02 ships renderer-level Playwright
  against Vite dev server (`@tauri-apps/api` module-mocked); full
  desktop-shell E2E is M03 carry-forward. §M3 deliverable adds
  `tauri-driver` + WebdriverIO matrix (Linux + Windows; macOS
  unsupported), wires the four `test.skip()` carry-forward Playwright
  tests, adds CI E2E acceptance criterion. §M3 out-of-scope adds
  "macOS Tauri-shell E2E (tauri-driver does not support macOS —
  deferred indefinitely)".

### Documentation — Post-M02 protocol iteration

Per `CLAUDE.md` §19 + `M02-summary.md` Verdict ("Pattern held but with
friction") prescribed protocol-iteration session before M03 authoring opens.
Lands the carry-forward decisions from M02.A–E retrospectives into the
shared protocol docs so M03 stages don't re-discover the same friction.

- **`docs/gotchas.md`** — eight new entries (#21–#28) consolidating M02
  carry-forwards: clippy pedantic+nursery patterns (compound entry covering
  9 sub-patterns), `current_exe()`-derived subprocess test paths, Tauri 2.x
  E2E uses `tauri-driver` + WebdriverIO (not Playwright `_electron`), ESLint
  9 flat-config default, Vite root convention, `serde(tag = "type")` requires
  struct-shape variants, Vitest+RTL DOM-ref-staleness pattern, bound
  test-fixture streams.
- **`docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md`** — new
  `[END] Coverage holdouts` subsection between Threshold evaluation and
  Decisions for next stage. Records workspace + per-package coverage
  actuals, exclusions added this stage, current exclusion list, per-module
  baselines (preserved-or-improved invariant per CLAUDE.md §5), and a
  doc-to-CI drift check. Replaces the historical scatter across CLAUDE.md
  §5 + per-stage `[END] Decisions`.
- **`docs/build-prompts/TEMPLATE.md`** — four additions to the milestone
  prompt template:
  - **`WEBCHECK:` header** at each stage's title block — required when the
    stage touches fast-moving tooling surfaces (npm / Tauri / esbuild /
    Vite / etc.). Lists authoritative URLs to web-verify against the prompt
    body before the fresh session opens. Per CLAUDE.md §12 web-first rule.
  - **Pre-existing legacy file inventory** subsection in milestone-level
    Background — required when this milestone touches a tree a prior
    milestone created. Lists tracked-but-orphaned files that prettier /
    eslint will scan with disposition (delete / preserve / refactor).
  - **Pedantic-pass preflight** checklist in the Stage X.4 Tests section
    template — clippy pedantic+nursery patterns to verify against new
    modules before writing the test plan. Cross-references gotchas.md #21.
  - **Default test plan for stages adding a new safety primitive** — codifies
    the M01.C / M02.A / M02.C / M02.D / M02.E pattern: "N unit tests for
    the testable seam + M integration tests for end-to-end behavior."
  - **Doc-to-CI invariant** addition under Safety primitive coverage gate —
    requires updating CI workflow + CLAUDE.md §5 + per-stage retro
    Coverage holdouts subsection in the SAME commit when adding a new
    coverage exclusion. Cites the M02.E `key_store.rs` drift bug as the
    cautionary tale.

### Added — M02.E (Tauri shell + skeleton renderer + frontend CI gates + Playwright)

- `package.json` + full frontend tooling (Vite 5.4, TypeScript 5.6 strict,
  React 18.3, Vitest 2.1, Playwright 1.48, ESLint 9 flat-config, Prettier 3,
  `@testing-library/react`, `@testing-library/user-event`,
  `@testing-library/jest-dom`, `happy-dom` 20.x).
- `src/` skeleton renderer:
  - `App.tsx` composes `SetupPanel` + `SmokeButton` + `EventList`; state via
    `useReducer` over a pure reducer (`lib/eventReducer.ts`) with full
    immutability.
  - `lib/ipc.ts` — typed wrappers over `@tauri-apps/api/core::invoke` and
    `@tauri-apps/api/event::listen`.
  - `lib/eventReducer.ts` — pure reducer + `Action` discriminated union.
  - `types/agent_event.ts` — TypeScript discriminated union mirroring
    `runtime_core::AgentEvent` v0.1 subset (10 variants Stage D emits +
    `ToolSource` enum). M03+ regenerates from `schemas/event.v1.json` via
    `cargo xtask regenerate-types` per CLAUDE.md §14.
  - `components/{EventList,SetupPanel,SmokeButton}.tsx` — minimal
    accessible markup; password-input invariant for the API-key field.
  - `index.html` + `styles.css` + `main.tsx` (React 18 root).
- `crates/runtime-main/src/key_store.rs` — OS-keychain-backed API key
  storage via the `keyring` crate. Reads return `SecretString` so the key
  never `Debug`-prints; `delete_api_key()` is idempotent (treats `NoEntry`
  as success). 2 unit tests + 2 keychain-gated `#[ignore]` tests for
  read-after-write round-trip + missing-entry → `NotFound` mapping.
- `src-tauri/src/commands.rs` — `set_api_key` and `run_smoke_session`
  Tauri commands. `CmdError` serializes with `serde(tag = "type")` for
  renderer pattern-matching. The testable seam
  `run_smoke_session_with(provider, event_tx, config)` (M01.C / M02.C /
  M02.D `*_with` archetype) accepts an injectable `LLMProvider` + channel
  so unit tests exercise the SDK→event flow without crossing reqwest or
  the Tauri `AppHandle`.
- `src-tauri/capabilities/default.json` — locked-down capability set:
  `core:default` + `core:event:{default,allow-listen,allow-emit}` only.
  No `shell:*`, no `fs:*`, no `http:*`, no `dialog:*`. Per spec §10
  capability boundary; CLAUDE.md §15 trap #10 (Tauri allowlist is the
  security boundary).
- Tests:
  - 14 frontend unit tests across `tests/unit/eventReducer.test.ts` (10)
    + `tests/unit/ipc.test.ts` (5) — pure reducer immutability + IPC
    wrapper call-shape + subscriber lifecycle.
  - 11 component tests across `tests/unit/components.test.tsx` (8)
    covering password-input invariant, save-button enabled-when-key-min-
    length, EventList aria-label + data-event-type attrs, all 10
    `AgentEvent` variant render paths.
  - 2 App-level state-machine tests in `tests/unit/App.test.tsx` —
    save-key-then-run-smoke happy path + command-error surface (mocks
    `@tauri-apps/api` at the module level).
  - 4 Playwright renderer-level E2E tests in `tests/e2e/smoke.spec.ts`:
    renderer-loads-with-setup-visible, password-input-type, smoke-
    disabled-without-key, save-key-then-run-disables-button-during-run.
  - 4 Rust unit tests in `src-tauri/src/commands.rs::tests` —
    `cmd_error_serializes_with_type_tag`, `from_keystore_not_found_maps_
    to_setup_required`, `run_smoke_session_with_emits_events_to_channel`,
    `smoke_config_targets_haiku_with_tight_budget`.
  - 2 unit tests + 2 keychain-gated tests in
    `crates/runtime-main/src/key_store.rs::tests`.
- CI:
  - `frontend` job (existed from M02.A) updated to run prettier on
    `**/*.{ts,tsx,js,jsx,json}` (markdown + YAML excluded via
    `.prettierignore` since markdown is checked by the existing
    `markdown-lint` job and YAML is structurally validated by the
    Actions runner).
  - `e2e` job added — installs Playwright + Chromium, runs
    `npm run test:e2e` against the Vite dev server.
  - `runtime-main` coverage gate exclusion list extended to add
    `src/key_store.rs` (the keychain-call paths are platform-bound and
    `#[ignore]`-gated). runtime-main remains at 99.37% line; workspace
    at 94.51% line. CLAUDE.md §5 + §6 updated.

### Documentation — M02.E

- `CLAUDE.md` §6 — frontend gates section made authoritative; E2E gates
  subsection added with the Tauri 2.x platform note (full desktop-shell
  E2E requires `tauri-driver` + WebdriverIO per the official Tauri docs;
  Stage E ships renderer-level Playwright against the dev server).
- `crates/runtime-main/README.md` — `key_store` module documented;
  Tauri command surface (`set_api_key` + `run_smoke_session` +
  `CmdError` shape) documented with the testable-seam pattern note.
- `docs/MVP-v0.1.md` §M2 — acceptance criteria all marked `[x]`; the
  `tool_invoked (LoadSkill)` sub-criterion noted as M03+ work since
  skills don't exist at M02.

### Status — M02.E

Stage E is the final implementation stage of M02. Stage F (Phase Closeout:
Gap Analysis) follows in a fresh session per CLAUDE.md §20.

### Documentation — M02.F (Phase Closeout: Gap Analysis)

- `docs/gap-analysis.md` — M02 entry appended (commit `4bd809a`, Stage E).
  Cumulative product+spec audit across M01 + M02 per CLAUDE.md §20. Six
  sections: codebase deep dive, adherence to spec (1 ❌ "None observed";
  multiple ✅ holds for zero-telemetry, direct Anthropic API, SSE wire
  format, Tauri capability lockdown, schemas-as-source-of-truth, `*_with`
  test-seam pattern, coverage delta gating, mcp_servers + Windows IPC test
  + .gitattributes carry-forwards closed; ⚠️ items for the M04-deferred
  decision-extractor heuristic, count_tokens approximation, EventPipeline
  ToolResult duration_ms placeholder, ContextType enum diverging from spec
  §2b's value set (M04 closeout reconciles — direction undetermined),
  mcp_servers schema deliberately richer than spec §11 (ADR before M06),
  vitest threshold not yet enforced by default, Tauri 2.x desktop-shell
  E2E deferred to M03, hand-mirrored TS AgentEvent), spec review
  (forward-looking — signature_delta + ping events, IPC reconnect surface,
  ProviderEvent::Error terminal semantics, Phase 3 spec expansion, Session
  FSM diagram, plan model field shapes, model deprecation policy, error.v1
  schema), fix backlog (0 Critical; ~17 🟡 Important spanning M03/M04 prep
  + post-M02 docs(spec) PR + CLAUDE.md+TEMPLATE.md consolidation; ~5 🟢
  Nice-to-have including vite 5→7 bump and counter.{js,test.js} cleanup),
  carry-forward (M01 🟡 mcp_servers / coverage delta gating / *_with /
  Windows drone integration test / .gitattributes / post-M01 docs(spec)
  PR all RESOLVED; M01 🟡 Phase 3 spec expansion / Session FSM diagram /
  UI consistency STILL OPEN per their target milestone). Append-only
  invariant verified locally and via `git diff origin/main` (line 706+ is
  pure addition). Per CLAUDE.md §20 the entry is **immutable** once
  committed.
- `docs/build-prompts/retrospectives/M02-summary.md` — new. Per-parent-
  milestone roll-up aggregating M02.A–E retrospectives. Mean Process
  38.6/40, Product 39.4/40, Pattern 29.2/35; verdict **"Pattern held but
  with friction"** (all hard gates passed; Stage E sound-but-rough due to
  8 Sev-2-or-3 prompt-drift items including Tauri 2.x E2E framework, ESLint
  flat-config, Vite root convention, serde tag-shape, Vitest+RTL idiom).
  Decisions to apply before M03.1 authoring documented.

### Status — M02.F

Stage F is the final stage of M02 per CLAUDE.md §20. The Stage F commit is
the last on `claude/m02-event-pipeline`; the M02 PR push is gated on this
commit. The M02 PR aggregates all stage commits + per-stage retrospectives
+ M02-summary.md + the new gap-analysis entry for three-artifact review
per CLAUDE.md §19 + §20.

### Added — M02.D (AgentSdk + drone IPC client + event translation)

- `crates/runtime-main/src/sdk/agent_sdk.rs` — `AgentSdk<P: LLMProvider>`
  agent loop. Generic over the provider trait so v1.0+ providers slot in
  unchanged. Constructs the provider stream in `run_agent(config)`; the
  test-seam variant `run_agent_with_provider_stream(stream)` accepts any
  pre-built `Stream<ProviderEvent>` (mirrors the M01.C / M02.C `*_with`
  archetype). Emits `AgentSpawned` first, drives the `EventPipeline` to
  exhaustion, flushes buffered text. `SessionId` newtype wraps `Uuid`.
- `crates/runtime-main/src/sdk/event_pipeline.rs` — pure
  `ProviderEvent` → `AgentEvent` translator. Consecutive `TextDelta`s
  bundle into a single `StreamText` per non-text event boundary;
  flushed on `ThinkingDelta`, `ToolUse`, `ToolResult`, `MessageStop`,
  `Error`, and end-of-stream. Decision extraction runs at every flush
  and prepends a `DecisionRecord` when matching markers are present.
- `crates/runtime-main/src/sdk/decision_extractor.rs` — first-line
  `Decision:`/`Rationale:` heuristic per spec §2 `decision_record`.
  Pure function; line-by-line scan tolerates intervening blank lines
  and leading whitespace; last `Decision:`/`Rationale:` pair wins.
  Optional `Tool used:` capture. Property test verifies no panic on
  arbitrary input. M04 verify+rails replaces the heuristic with a
  structured emitter.
- `crates/runtime-main/src/drone_ipc/client.rs` — `DroneClient` main-
  side IPC client. Connects via `DroneClient::connect(addr)` (cfg-
  platform Unix `UnixStream` / Windows `NamedPipeClient`). Test-only
  `DroneClient::noop()` short-circuits all sends. `events()` returns a
  single-consumer stream of `Result<DroneEvent, DroneIpcError>`.
- `crates/runtime-main/src/drone_ipc/connection.rs` — connection state
  machine + reconnect policy. Exponential backoff: 200ms → 400ms →
  800ms → 1.6s (4 sleeps for 5 attempts; no trailing sleep). Surfaces
  `DroneIpcError::Disconnected { retries }` on exhaustion.
  `Connection::from_streams` is the testable seam taking already-opened
  read+write halves; the `open()` cfg-platform OS-call wrapper is the
  coverage holdout.
- `crates/runtime-core/src/event.rs` — `ToolSource { Builtin, Mcp,
  Generated }` enum added; `AgentEvent::ToolInvoked` gains `source` +
  `server` fields; `AgentEvent::AgentSpawned` gains `session_id`.
  Property tests round-trip the new shape per the M01.B pattern.
- `crates/runtime-main/tests/sdk_event_translation.rs` — 20 table-
  driven translation tests + 1 proptest covering bundling boundaries,
  decision extraction, error-path translation, multi-tool sequencing,
  buffer drain semantics, agent-id propagation.
- `crates/runtime-main/tests/sdk_cancellation.rs` — 5 drop-mid-stream
  cancellation-safety tests using `tokio::time::timeout` +
  `futures::stream::iter` patterns. Verifies no panic on drop, channel
  drains to `Closed`, back-pressure does not panic.
- `crates/runtime-main/tests/drone_ipc_loopback.rs` — 10 end-to-end
  tests spawning the M01 `runtime-drone` binary, exercising every
  `DroneCommand` variant, the `SnapshotWritten` event surface, and the
  reconnect / disconnect surface paths.
- `runtime-main` safety-primitive coverage gate extended to span
  `sdk/` and `drone_ipc/`. Exclusions: `providers/anthropic.rs` (Stage
  C real-network wrapper) plus `drone_ipc/connection.rs::open` (cfg-
  platform OS-call holdout); the testable seam
  `Connection::send_with_reconnect` is fully covered. CI gate +
  `CLAUDE.md` §5 updated.

### Changed — M02.D

- `crates/runtime-main/Cargo.toml` — add `tokio-util` (`codec`
  feature), `uuid` (`v4` + `serde`), `tempfile` + `rusqlite[bundled]`
  (dev-deps for the loopback test).
- `crates/runtime-main/src/lib.rs` — top-level module declarations
  (`pub mod sdk; pub mod drone_ipc;`).
- `crates/runtime-main/README.md` — appended §"Agent SDK" with the
  `ProviderEvent` ↔ `AgentEvent` mapping table and `DroneClient`
  reconnect policy notes.

### Added — M02.C (AnthropicProvider real HTTP+SSE)

- `crates/runtime-main/src/providers/anthropic_sse.rs` — SSE state
  machine + parser. `SseEvent` enum mirrors the Anthropic Messages API
  wire format (`message_start`, `content_block_start/delta/stop`,
  `message_delta/stop`, `ping`, `error`). `SseState` accumulates tool
  input partial-JSON deltas across `content_block_delta` events; emits
  the complete `ToolUse` on `ContentBlockStop`. `signature_delta` is
  parsed and silently dropped (verifier-only payload).
- `crates/runtime-main/src/providers/anthropic.rs::stream` — real
  HTTP+SSE implementation. Direct `reqwest` + `eventsource-stream`; no
  third-party Anthropic SDK. Lazy `OnceLock<reqwest::Client>` per
  provider instance. Maps non-2xx responses: 401/403 → `Auth`; 429 →
  `RateLimit { retry_after_secs }` (parsed from the `retry-after`
  header, default 60); other → `Api { status, body }`.
- `crates/runtime-main/tests/anthropic_wiremock.rs` — 8 wiremock-driven
  integration tests covering happy path, auth failure, rate limit, tool
  use accumulation, thinking + signature passthrough, server-emitted
  error, malformed bytes skipped, and partial-chunk reassembly.
- `crates/runtime-main/tests/anthropic_smoke.rs` — real-API smoke gated
  by `--features integration`; reads keychain entry
  `agent-runtime/anthropic`. CI never runs this; cost ~$0.001 per run
  against Haiku 4.5.
- `runtime-main` added to the safety-primitive coverage gate matrix
  (≥95% line) with `src/providers/anthropic.rs` excluded as the
  real-network production wrapper. CI gate + CLAUDE.md §5 updated.

### Changed — M02.C

- `crates/runtime-main/Cargo.toml` — add `bytes` dep (direct, was
  transitive via reqwest) for the SSE state machine's stream type
  bound, and `wiremock` dev-dep for the integration tests.
- Workspace `Cargo.toml` — pin `wiremock = "0.6"` in
  `[workspace.dependencies]`.
- `crates/runtime-main/README.md` — add real-API smoke-test section
  with platform-specific keychain setup notes.
- `crates/runtime-main/src/providers/anthropic.rs` — remove the now-
  obsolete `stub_stream_returns_text_then_stop` test (the wiremock
  `happy_path_yields_text_deltas_and_message_stop` covers the same
  end-to-end shape against the real HTTP+SSE pipeline).

### Added — M02.B (LLMProvider trait + AnthropicProvider stub)

- `crates/runtime-main/src/providers/mod.rs` — `LLMProvider` trait,
  `ProviderEvent` enum (`TextDelta` / `ToolUse` / `ToolResult` /
  `ThinkingDelta` / `MessageStop` / `Error`), `ProviderError`
  (thiserror-derived), and supporting types (`AgentConfig`, `Message`,
  `ContentBlock`, `ImageSource`, `ToolResultContent`, `ModelInfo`,
  `Pricing`, `CostBreakdown`, `ProviderSupport`, `ModelCapabilities`)
  per spec §2c.
- `crates/runtime-main/src/providers/anthropic.rs` — `AnthropicProvider`
  shell. `SecretString`-wrapped API key; stub `stream()` returning
  hardcoded `TextDelta + MessageStop` sequence; hardcoded `list_models()`
  (Opus 4.7, Sonnet 4.6, Haiku 4.5); char-based `count_tokens()`;
  cache-aware `estimate_cost()` (5m write 1.25× / 1h write 2× / read
  0.1× input). Stage C replaces the stub body with real HTTP+SSE.
- `crates/runtime-main/README.md` — public API documentation per
  CLAUDE.md §6.
- Workspace dependencies (no third-party Anthropic SDK): `reqwest`
  (rustls-tls + json + stream), `eventsource-stream`, `async-trait`,
  `secrecy`, `keyring`, plus a path-dep entry for `runtime-core`.

### Added — M02.A (Build hygiene + scaffolding)

- `crates/runtime-core/src/signal.rs` — Signal Schema v2 type scaffold per
  spec §2b (8-variant `Signal` enum + `ContextType` + correlation field
  types `PreSignalId` / `ParentSignalId` / `RetryOfSignalId`). Emission
  integration is M04+ work; M02.A ships the type surface so M03+ work can
  import without churn.
- `crates/runtime-core/src/drone.rs::HeartbeatStatus` typed enum
  (`Ok`/`Degraded`/`Stalled`) replaces the prior `String`. Implements
  `Display` + `FromStr` so SQLite text storage round-trips through the
  enum. Closes M01 gap-analysis Important "HeartbeatStatus typed enum"
  per spec §1d (PR #36 closeout).
- `crates/runtime-drone/src/db.rs::init_schema` — 8th SQLite table
  `mcp_servers` per spec §11:2435-2444 + MCP best-practice (Claude Code
  / Claude Desktop / VS Code MCP client schemas). 22 columns covering
  identity, transport-specific config (stdio/http/sse/streamable_http),
  authentication (keychain refs, never literal secrets), connection
  lifecycle, timeouts, scope tracking, capability cache; SQL CHECK
  constraints enforce the stdio-vs-remote mutual exclusion. Schema only;
  MCP client lands in M06.
- `crates/runtime-drone/tests/integration_windows.rs` — Windows-platform
  end-to-end test exercising `ipc::accept_loop` over named pipe: spawns
  drone, sends `SnapshotNow`, verifies SQLite row, sends
  `GracefulShutdown`, verifies clean exit. Sister to the existing
  `tests/integration.rs` Unix SIGTERM lifecycle test; together they
  cover §0d Windows-only release scope.

### Changed — M02.A

- `crates/runtime-drone/src/command_handler.rs::run` accepts an optional
  `oneshot::Sender<&'static str>` and signals it on `GracefulShutdown`,
  driving full drone-process exit through the IPC channel. `run_inner`
  selects between the OS-signal future and the IPC-shutdown future to
  unify cross-platform graceful shutdown.
- Workspace coverage gate adds delta-gating from M02 onward (Codecov
  project: `target: auto`, `threshold: 0.5%`; patch: `target: 80%`).
  Per-crate Codecov flag uploads added for `runtime-drone` and
  `runtime-main`. Documented in `CLAUDE.md` §5 "Coverage delta gating
  (from M02 onward)".

### Documentation — M02.A

- `docs/style.md` — `*_with` / `*_inner` test-seam pattern documented
  as the canonical TDD-friendly approach to OS-driven async functions.
  Cites M01.C archetype at `crates/runtime-drone/src/{lib,shutdown}.rs`
  and codification commit `1dec4ba`.
- `.gitattributes` — explicit LF normalization for `*.rs`, `*.toml`,
  `*.json`, `*.md`, `*.yml`, `*.sh`, `*.bash`, `*.py`, `*.html`, `*.css`,
  `*.js`. Closes M01 gap-analysis Important "line-ending normalization".
- `.gitignore` — `src-tauri/gen/schemas/` excluded; the four
  Tauri-generated files (`acl-manifests.json`, `capabilities.json`,
  `desktop-schema.json`, `windows-schema.json`) untracked but kept on
  disk. Closes M01 PR #36 follow-up "src-tauri/gen/schemas/ should be
  gitignored to prevent future drift".

### Added

- **Spec §15 Sharing & Distribution + ADR-0005** — three sharing tiers
  declared (runtime-to-runtime in v0.1 via M07; headless CLI
  `agent-runtime-cli` in v1.0; WASM in v2.0+); cross-OS portability
  rules (POSIX-only paths, `compatible_os` declaration); the "Share It"
  module forward-declared as v1.0 deliverable in M08+. Four additive
  optional fields in `schemas/framework.v1.json` (`requires_secrets`,
  `runtime_dependency_class`, `compatible_os`, `share_provenance`)
  ship as v0.1 schema groundwork so M03–M07 frameworks are
  forward-compatible with the v1.0 headless CLI and Share It module
  without schema migration. Minor in-place schema bump per
  `schemas/README.md` versioning policy; `$id` unchanged. MVP-v0.1.md
  §M07 updated to emit `share_provenance` on export and validate the
  four fields on import; §M08 forward-declares the Share It module.
  Generated Rust types (`crates/runtime-core/src/generated/framework.rs`)
  and TypeScript types (`src/types/framework.ts`) **must be regenerated
  via `cargo xtask regenerate-types` before this changeset's PR merges**
  — the type-drift CI gate (per CLAUDE.md §14) blocks merge otherwise.
  Regen happens on a Rust-capable machine (Windows / macOS / Linux);
  the agent environment that authored the schema/spec/ADR changes does
  not have a usable cargo toolchain.
- **M01 Foundation milestone** — Cargo workspace with five member crates
  (`runtime-core`, `runtime-main`, `runtime-drone`, `runtime-sandbox`,
  `xtask`) plus Tauri stub at `src-tauri/`, workspace lints (deny
  warnings, forbid unsafe except sandbox, clippy pedantic + nursery),
  and a `cargo-deny` policy. `rust-toolchain.toml` pins channel to
  `stable`; MSRV enforcement lives in workspace `Cargo.toml`.
- **Type-generation pipeline** — `cargo xtask regenerate-types` reads
  `schemas/*.v1.json` via [`typify`](https://crates.io/crates/typify)
  and writes to `crates/runtime-core/src/generated/`. CI runs
  `--check` on every PR to fail on any drift between committed types
  and freshly regenerated output.
- **Hand-curated event taxonomy in `runtime-core`** — `AgentEvent`
  (full variant list per spec §2 + §2a + §2b + §3a + §3b + §4a + §4b
  + §6a + §8.security), `DroneEvent` + `DroneCommand` per spec §1d,
  `RuntimeError` via `thiserror`.
- **Drone Phase 1 (`runtime-drone`)** — heartbeat task (5s tokio
  interval) writing `heartbeats` rows and emitting
  `DroneEvent::Heartbeat`; append-only snapshot writer with SHA-256
  `state_hash`; platform-specific IPC server (Unix domain socket on
  Linux/macOS, Windows named pipe via `tokio::net::windows::named_pipe`)
  with framed JSON-newline via `tokio_util::codec::LinesCodec` and
  malformed-input tolerance (emits `Alert`, keeps server alive);
  SIGTERM / SIGINT / CTRL_BREAK / CTRL_C handler with best-effort
  emergency snapshot before exit. SQLite WAL pragmas applied in correct
  order (`journal_mode → synchronous → busy_timeout → foreign_keys`);
  7-table schema (`sessions`, `snapshots`, `signals`, `heartbeats`,
  `vdr`, `token_usage`, `skills`).
- **Runtime-drone safety-primitive coverage gate** — ≥95% line with
  `lib.rs` + `shutdown.rs` excluded (OS-signal orchestrators exercised
  end-to-end by the Unix subprocess integration test). Per-module
  baseline (M01.C measured): `snapshot.rs` 100%, `db.rs` 98.82%,
  `heartbeat.rs` 98.59%, `command_handler.rs` 97.94%, `ipc.rs` 84.70%.
  Workspace coverage gate: ≥80% line, generated code and binary stubs
  excluded.
- **Fuzz harness** — cargo-fuzz `drone_command_decode` target for the
  IPC frame decoder with 6 seed corpus entries (one per
  `DroneCommand` variant). CI fuzz-smoke job runs 30s on every PR;
  scheduled `fuzz-nightly.yml` workflow runs 1 hour at 04:00 UTC and
  uploads the corpus on failure.
- **Per-crate READMEs** — `runtime-core`, `runtime-drone`, and `xtask`
  document the public API surface, IPC protocol, SQLite schema,
  manual smoke procedure, platform-specific details, and the
  coverage requirement.

### Tests

- **Schema round-trip tests** — `examples/aria/framework.json`,
  `examples/ralph/framework.json`, and 19 skill / agent / tool
  frontmatter files all round-trip through generated `runtime-core`
  types via the serialize-deserialize-serialize stability check.
- **Property tests** — `proptest` round trips for `AgentEvent`,
  `DroneEvent`, `DroneCommand`, including the newline-delimited JSON
  codec wire format.
- **Drift-check positive and negative cases** in `xtask`.
- **Drone unit tests** (22 total) — WAL pragmas, schema, snapshot
  append-only and SHA-256 hash, heartbeat interval, IPC encode /
  decode, command dispatch, malformed-input → `Alert`, broadcast
  lagged path.
- **Subprocess-spawn integration test** (`tests/integration.rs`,
  `#[cfg(unix)]`) — drone responds to SIGTERM with an emergency
  snapshot.
- **Fuzz target compiles and runs** — `cargo +nightly fuzz build`
  succeeds on Linux/macOS/Windows; `cargo +nightly fuzz run … -- 
  -max_total_time=30` exits 0 with no panics on Linux CI.

### Documentation

- Per-crate READMEs (`runtime-core`, `runtime-drone`, `xtask`).
- M01 Foundation specification + per-stage prompts at
  `docs/build-prompts/M01-foundation.md` (Stages A through E).
- M01 Phase Closeout: cumulative gap analysis appended to
  `docs/gap-analysis.md` per `CLAUDE.md` §20 (append-only living
  document). Gates the M01 PR. CI gains a `gap-analysis-append-only`
  job that enforces the immutability of prior entries on every PR.
- Per-stage retrospectives at
  `docs/build-prompts/retrospectives/M01.{A,B,C,D}-retrospective.md`
  + parent-milestone summary at `M01-summary.md` (per `CLAUDE.md` §19).
- Comprehensive product specification (`agent-runtime-spec.md`)
  covering project positioning, capability matrix, three-concept model
  (Tool/Skill/Agent), dev loop, release scope matrix, drone, recovery,
  multi-session, IPC, event pipeline, budget, signals/VDR,
  LLMProvider abstraction, live graph, plan/task primitive,
  mode/sizing, gap detection, verify/rails, MCP manager, framework
  loader, HITL policy, registry, generators with 5-layer security,
  builder canvas, persistence, secrets vault, reconciliation/degraded
  modes, engineering charter, privacy/telemetry, first-run UX.
- JSON Schema source-of-truth files in `schemas/` (Draft 2020-12):
  `common.v1.json`, `skill.v1.json`, `tool.v1.json`, `agent.v1.json`,
  `framework.v1.json`. All 19 example artifacts validate.
- `examples/aria/` reference framework reconstructing every row of
  the capability matrix.
- `examples/ralph/` sibling framework demonstrating the
  `loop_policy: continuous` variant; reuses `aria/` tools and skills
  via `source: external`.
- `docs/MVP-v0.1.md` build checklist (11 milestones; novice-and-
  experienced two-path success criterion).
- Engineering Charter in spec §12; Privacy & Telemetry in §13
  (zero telemetry by default); First-Run UX state machine in §14.
- ADR template + ADRs 0001–0004 (ARIA-as-archetype, Tauri-over-
  Electron, Engineering Charter adoption, defer paid code-signing).
- OSS scaffolding: `LICENSE` (Apache 2.0), `NOTICE`,
  `CODE_OF_CONDUCT.md`, `SECURITY.md`, `CONTRIBUTING.md`.

### Changed

- **Code-signing posture for v0.1: deferred** (per ADR-0004). v0.1
  ships unsigned `.msi` with SHA-256 checksums and Sigstore provenance
  attestations via GitHub Actions OIDC. Paid Windows EV code-signing
  revisited at v0.5+ when adoption is proven. Affects:
  `docs/MVP-v0.1.md` M11 acceptance + risk register R4;
  `docs/README-v0.1.md` install instructions (SmartScreen-warning
  explainer + checksum/cosign verification steps);
  `.github/workflows/release.yml` (drops signing secrets, adds
  SHA-256 generation + `actions/attest-build-provenance@v1`);
  spec §0d distribution row.

### Status

M01 Foundation milestone complete. M02 (event pipeline +
`AnthropicProvider` + Tauri shell + `AgentEvent` flow) is the next
milestone.

---

## Versioning

- **0.x** — pre-stable. Schemas may change; APIs are not guaranteed compatible across 0.x versions.
- **1.0+** — semver strict. Breaking changes to framework JSON schema, AgentEvent union, or any `pub` Rust API require a major bump.

## Release artifacts

Once releases begin (v0.1.0 Windows Preview), each release will include:
- Signed Windows installer (`.msi`) at v0.1; macOS `.dmg` and Linux AppImage from v1.0.
- SBOM in CycloneDX format.
- Source tarball.
- SLSA Level 3 provenance attestations from v1.0.
