# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
