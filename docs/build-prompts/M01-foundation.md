# M01 — Foundation

> **Milestone:** M1 of M11 in `docs/MVP-v0.1.md`
> **Estimated effort:** ~30–50 hours Claude execution + ~10 hours human direction; **2 weeks elapsed** at sustained pace
> **Branch:** `claude/m01-foundation` (off `main`)
> **Prerequisite milestones:** none — root milestone

---

## Read first

Before writing any code, read in this order:

1. **`CLAUDE.md`** (repo root) — protocol, hard rules, quality gates, PR workflow, anti-patterns. You should already have this auto-loaded; confirm by stating the §4 Hard Rules at the top of your first response.
2. **`agent-runtime-spec.md`** — read these sections in full:
   - §0 Project Positioning, §0a Capability Matrix, §0b Three Concepts, §0c Dev Loop, §0d Release Scope (always)
   - **§1 Phase 1: The Drone** — the entire section, including subsections §1b Recovery Semantics, §1c Multi-Session & SQLite Concurrency, §1d IPC Channels
   - **§12 Engineering Charter** — quality gates, ADR rules, CI matrix
   - **Persistence Layer** section (the SQLite schema)
   - **Project Structure** section (Cargo workspace layout)
   - **Starting Prompt for Claude Code** (use as cross-check on this milestone's deliverables)
3. **`docs/MVP-v0.1.md`** — read **§M1 Foundation** in full + the Milestone Overview table at the top
4. **`docs/adr/`** — read these ADRs:
   - **ADR-0001** ARIA as Archetype — for context on what the project is and isn't
   - **ADR-0002** Tauri + Rust over Electron — for the stack rationale; critical because M1 sets up the Cargo workspace
   - **ADR-0003** Engineering Charter adoption — for the quality gates and process this milestone must respect
5. **`schemas/`** — read these schemas (they drive type generation in `runtime-core`):
   - `schemas/README.md` — versioning policy, type-generation pipeline
   - `schemas/common.v1.json` — shared types (Capabilities, Provenance, HookRef, Hook, JSONLogicExpression, ModelRef, SemVer, FileGlobList, Tier)
   - `schemas/framework.v1.json` — top-level framework
   - `schemas/skill.v1.json`, `schemas/tool.v1.json`, `schemas/agent.v1.json` — artifact schemas
6. **`examples/aria/`** — skim these reference artifacts as a sanity check that the generated types you produce in `runtime-core` actually validate the existing examples:
   - `examples/aria/framework.json`
   - `examples/aria/skills/planning.md` (frontmatter only)
   - `examples/aria/agents/orchestrator.md` (frontmatter only)
   - `examples/aria/tools/aria_verify.md` (frontmatter only)
7. **`.github/workflows/ci.yml`** — read the existing CI workflow. It already has schema-validation jobs that pass; you'll be adding/activating the Rust jobs that currently gate on `Cargo.toml` existing.

After reading, **state in 1–3 sentences what M1 delivers and the test plan in 5–8 bullets**. Wait for confirmation before writing code.

---

## Problem statement

The project has a complete specification (`agent-runtime-spec.md`), schemas, two reference frameworks, and OSS scaffolding. **It has no code.** The runtime binary doesn't exist; the Cargo workspace doesn't exist; the type system in `runtime-core` doesn't exist; CI's Rust jobs are stubbed out waiting for `Cargo.toml` to land.

M1 unblocks every subsequent milestone by:

1. Establishing the **Cargo workspace skeleton** with the four crates (`runtime-core`, `runtime-main`, `runtime-drone`, `runtime-sandbox`) and the Tauri wrapper (`src-tauri/`). Most are stubs at M1; only `runtime-core` and `runtime-drone` get real implementation.
2. Setting up **type generation from schemas** — `runtime-core` types are generated from `schemas/*.v1.json` via `typify`, never hand-written. CI fails if committed types differ from regenerated types.
3. Implementing **Phase 1: The Drone** — the survival layer. Heartbeat, snapshots, recovery, IPC. This is what keeps a session resumable across crashes and is the foundation every later milestone depends on.
4. Making **CI green on Linux/macOS/Windows** with all `cargo` gates active (fmt, clippy, test, doc, audit, deny, coverage).

After M1, the next session (M2) builds the SDK event pipeline on top of a working drone. Without M1's drone, there's nothing to snapshot to; without `runtime-core` types, there's no shared type vocabulary; without CI green, every subsequent commit lands without quality gates.

There is no user-facing behavior to demo at the end of M1 — that's M2's threshold. M1's success is "the foundation works and stays working."

---

## Scope

### In scope (deliver these)

**Workspace skeleton:**
- `Cargo.toml` at repo root declaring the workspace and member crates
- `Cargo.lock` (committed)
- `rust-toolchain.toml` pinning Rust to the chosen stable version (e.g., 1.80; pick the latest stable as of M1 start and pin)
- `crates/runtime-core/` — real implementation: types generated from schemas, `AgentEvent` union, `DroneEvent`/`DroneCommand` enums, `Plan`/`Task`/`Capability`/`Hook` types, error types via `thiserror`
- `crates/runtime-drone/` — real implementation per §1 of the spec: CLI args, SQLite WAL setup, heartbeat task, snapshot writer, graceful shutdown, SIGTERM emergency snapshot, IPC server (Unix socket / Windows named pipe with framed JSON)
- `crates/runtime-main/` — **stub crate** with just `lib.rs` and `Cargo.toml`. M2 implements the SDK pipeline.
- `crates/runtime-sandbox/` — **stub crate** with just `lib.rs` and `Cargo.toml`. M5 implements the sandbox host.
- `src-tauri/` — **stub Tauri config** with `tauri.conf.json` (allowlist starts empty + minimal commands), `src/main.rs` that just opens an empty webview, and `Cargo.toml`. M2 wires up the first real command.

**Type generation pipeline:**
- `xtask` workspace member at `crates/xtask/` with a `regenerate-types` subcommand
- Generates Rust types from `schemas/*.v1.json` via `typify`
- Output written into `crates/runtime-core/src/generated/` (committed)
- CI step that runs `cargo xtask regenerate-types --check` and fails if output differs from committed (drift detection)

**Drone implementation (the meat of M1):**
- All eight `DroneEvent` variants and all six `DroneCommand` variants per §1d of the spec — exact serde tags
- SQLite schema: `sessions`, `snapshots`, `signals`, `vdr`, `token_usage`, `skills` tables per the Persistence Layer section of the spec (only `sessions` and `snapshots` are actively used by drone in M1; the others get the schema but no insert paths yet)
- WAL mode with the four required pragmas in order (`journal_mode = WAL`, `synchronous = NORMAL`, `busy_timeout = 5000`, `foreign_keys = ON`)
- Framed JSON-newline IPC over Unix domain socket (Linux/macOS) and Windows named pipe (Windows). Use `tokio_util` codec.
- Heartbeat task: 5-second interval, writes `heartbeats` row to SQLite, emits `DroneEvent::Heartbeat` on socket
- `SnapshotNow` command: serialize provided state to `snapshots` table; emit `SnapshotWritten` event
- `GracefulShutdown` command: flush pending writes, close socket, exit 0 within `timeout_ms`
- `RevertToSnapshot` command: load named snapshot blob, return it on the socket; do not actually mutate filesystem (that's M4)
- SIGTERM/SIGINT handler via `tokio::signal`: emergency snapshot then exit; use `select!` against the main event loop

**CI activation:**
- Existing `.github/workflows/ci.yml` already has the `detect-cargo` job and the gated `rust`, `cargo-audit`, `cargo-deny`, `coverage` jobs. They activate when `Cargo.toml` exists at repo root. M1 makes this happen.
- Add `cargo xtask regenerate-types --check` as a CI step
- All Rust gates green on Linux, macOS, Windows × stable + MSRV (1.80)

### Out of scope (do NOT deliver these)

- **Anything in `runtime-main` beyond a stub `lib.rs`.** SDK pipeline, providers, MCP, framework loader — all M2+.
- **Anything in `runtime-sandbox` beyond a stub.** OS-level sandboxing is M5+ scope.
- **Anything in `src-tauri` beyond an empty webview.** Real commands are M2+.
- **Frontend code.** No `package.json` work in M1 unless you discover the Tauri stub needs a minimal `index.html`; the renderer is M2+.
- **MCP, generators, builder canvas, plan model, gap detection, capability enforcement, registry.** All later milestones. The drone serializes whatever state blob it's given; it doesn't know about plans or capabilities yet.
- **The actual production-ready drone snapshot semantics for plan/task/capability state.** M1's drone accepts an opaque `state_json` blob; M4 starts populating it with real plan state. Don't pre-design the blob structure here.
- **Auto-update, code-signing, signed releases.** All M11+ scope.
- **Multi-session.** v0.1 is single-session per §0d; M1's drone supports one session per process.

If you find yourself wanting to deliver any of the above, **stop and ask** — never silently expand.

---

## TDD plan

Write these tests in this order. Each fails when first written. The production code that makes it pass is the implementation.

### Unit tests (Rust — `cargo test`)

In `crates/runtime-core`:

1. **`framework_v1_round_trip`** — load `examples/aria/framework.json` via `serde_json::from_str` into the generated `Framework` type, then re-serialize. Assert the round-trip is structurally equivalent (parse to `serde_json::Value` on both sides; assert equal). Drives: typify generation correctness.
2. **`skill_frontmatter_round_trip`** — same for `examples/aria/skills/planning.md` frontmatter (parsed via `serde_yaml`). Drives: `Skill` type matches `schemas/skill.v1.json`.
3. **`drone_event_serde_round_trip`** — `DroneEvent` variants serialize to the exact JSON shape the spec §1d declares (snake_case tags). Drives: tag attributes correct.
4. **`drone_command_serde_round_trip`** — same for `DroneCommand`.
5. **`activity_state_transitions_valid`** — typestate-style test that valid transitions (Active → Idle → Stalled → TimedOut → UserAborted, etc.) compile and round-trip; invalid state names fail to deserialize.

In `crates/runtime-drone`:

6. **`heartbeat_fires_at_interval`** — using `tokio::time::pause()` + `advance(Duration::from_secs(5))`, the heartbeat task produces exactly one `DroneEvent::Heartbeat` per 5-second window. Drives: heartbeat loop correctness without wall-clock dependence.
7. **`snapshot_now_writes_correct_row`** — given a `DroneCommand::SnapshotNow { reason }` with a state blob, the `snapshots` table contains a row with the correct `session_id`, `timestamp`, `reason`, and `state_json`. Drives: snapshot writer + SQL.
8. **`graceful_shutdown_flushes_within_timeout`** — given pending writes in the queue and `DroneCommand::GracefulShutdown { timeout_ms: 1000 }`, the drone exits 0 within 1000ms AND all pending writes are present in SQLite at exit. Drives: shutdown ordering.
9. **`sigterm_triggers_emergency_snapshot`** — spawn drone in a sub-process, send SIGTERM, verify (a) drone exits within grace period and (b) an emergency snapshot exists in SQLite with `reason: 'sigterm'`. (On Windows: equivalent via `CTRL_BREAK_EVENT`.)
10. **`wal_pragmas_set_in_correct_order`** — open the SQLite database via the drone's connection setup, query `PRAGMA journal_mode`, `PRAGMA synchronous`, `PRAGMA busy_timeout`, `PRAGMA foreign_keys`, assert `WAL`, `NORMAL`, `5000`, `1`. Drives: pragma initialization.
11. **`revert_to_snapshot_returns_blob`** — given a snapshot exists, `DroneCommand::RevertToSnapshot { snapshot_id }` returns a `DroneEvent` containing the original state_json. Does NOT mutate filesystem (M4 scope).

### Property tests (Rust — `proptest`)

12. **`drone_event_json_round_trip`** — for any valid `DroneEvent` (generated via proptest strategy), `serde_json::to_string` then `from_str` is identity.
13. **`drone_command_json_round_trip`** — same for `DroneCommand`.
14. **`framed_codec_round_trip`** — for any sequence of (`DroneEvent` | `DroneCommand`) framed via the `tokio_util` codec, encode then decode yields the original sequence. Drives: IPC framing correctness across message boundaries.
15. **`snapshot_state_hash_deduplication`** — for two snapshots with byte-identical `state_json`, the computed `state_hash` is identical. (Hash collision detection at read time; M1 just computes the hash on write.)

### Fuzz harnesses (Rust — `cargo-fuzz`)

16. **`fuzz_target_drone_command_decode`** — fuzz the IPC frame decoder with arbitrary bytes; must not panic, must not deserialize to a `DroneCommand` that bypasses validation. Run 30 seconds in CI per PR; nightly runs 1 hour.

### Integration tests (Rust — `cargo test --features integration`)

17. **`drone_lifecycle_end_to_end`** — spawn drone as a subprocess, connect to its IPC socket, send a sequence of commands (`SnapshotNow`, `RevertToSnapshot`, `GracefulShutdown`), verify all events received in the right order. Drives: process-spawn + IPC + lifecycle in one test.

### Frontend / E2E

**N/A — frontend doesn't exist yet at M1; renderer is a stub. M3 introduces Vitest tests; Playwright comes when the renderer can run a session.**

### Doc tests

Every `pub` type in `runtime-core` and every `pub` function in `runtime-drone` gets a doc comment with at least one example. `cargo test --doc` runs them; `cargo doc --no-deps -- -D rustdoc::missing_docs` enforces presence.

### Coverage target

- `runtime-core`: ≥80% line coverage
- `runtime-drone`: **100% line coverage** (it's a safety primitive per CLAUDE.md §5)
- `xtask`: ≥80% line coverage on the type-generation logic
- Stub crates (`runtime-main`, `runtime-sandbox`): no coverage requirement (they're empty)

---

## Acceptance criteria

The milestone is "done" only when every criterion below is checked:

### Build & types

- [ ] `Cargo.toml` at repo root declares the workspace with members `crates/runtime-core`, `crates/runtime-main`, `crates/runtime-drone`, `crates/runtime-sandbox`, `crates/xtask`, `src-tauri`
- [ ] `rust-toolchain.toml` pins to the chosen Rust stable version
- [ ] `Cargo.lock` committed
- [ ] `cargo build --workspace` succeeds on Linux, macOS, Windows
- [ ] `cargo build --workspace --release` succeeds (catches release-only issues early)
- [ ] `runtime-core/src/generated/` exists and contains types generated from schemas via `typify`
- [ ] `cargo xtask regenerate-types --check` succeeds (no drift between generated and committed)
- [ ] Generated types correctly deserialize `examples/aria/framework.json` and `examples/ralph/framework.json` (verified by tests #1 above)

### Drone behavior

- [ ] `runtime-drone --session-id test --db-path /tmp/d.sqlite --ipc-socket /tmp/d.sock` starts and is reachable on the socket
- [ ] All eight `DroneEvent` variants and six `DroneCommand` variants implemented per spec §1d (verified by tests #3, #4, #12, #13)
- [ ] SQLite WAL pragmas set in correct order (verified by test #10)
- [ ] Heartbeat fires every 5 seconds and writes to SQLite (verified by test #6)
- [ ] `SnapshotNow` writes correct row (verified by test #7)
- [ ] `GracefulShutdown` flushes within `timeout_ms` (verified by test #8)
- [ ] SIGTERM triggers emergency snapshot then exits (verified by test #9)
- [ ] `RevertToSnapshot` returns the snapshot blob without filesystem mutation (verified by test #11)
- [ ] Drone-as-subprocess lifecycle works end-to-end (verified by test #17)

### Quality gates (the must-pass list per CLAUDE.md §6)

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes (with `clippy::pedantic` + `clippy::nursery` lints active per §12)
- [ ] `cargo test --workspace` passes
- [ ] `cargo test --workspace --doc` passes
- [ ] `cargo doc --workspace --no-deps -- -D rustdoc::missing_docs -D rustdoc::broken_intra_doc_links` passes
- [ ] `cargo audit` clean (no high/critical advisories)
- [ ] `cargo deny check` passing
- [ ] `cargo llvm-cov --workspace` shows coverage ≥80% lines on `runtime-core` and ≥100% on `runtime-drone`
- [ ] `cargo xtask regenerate-types --check` passes (drift detection)
- [ ] Fuzz harness compiles and runs for 30 seconds without panic (`cargo fuzz run drone_command_decode -- -max_total_time=30`)
- [ ] CI green on Linux, macOS, Windows × stable + MSRV (1.80) — verify by inspecting the CI run after push, before drafting the PR

### Workspace lint configuration

- [ ] `Cargo.toml` workspace section sets `lints.rust = { unsafe_code = "forbid" }` for all crates EXCEPT `runtime-sandbox` (which uses `unsafe_code = "warn"` and has no `unsafe` blocks yet at M1; M5 adds them with `// SAFETY:` comments)
- [ ] `Cargo.toml` workspace section sets `lints.clippy.pedantic = { level = "warn", priority = -1 }` and `lints.clippy.nursery = { level = "warn", priority = -1 }`

### Documentation

- [ ] Every `pub` type in `runtime-core` has a doc comment with at least one example
- [ ] Every `pub` function in `runtime-drone` has a doc comment with at least one example
- [ ] `crates/runtime-drone/README.md` documents the CLI args, IPC protocol, and how to test locally
- [ ] `crates/runtime-core/README.md` documents the type-generation pipeline and how to regenerate
- [ ] `CHANGELOG.md` has an `[Unreleased]` entry naming what M1 delivered

---

## Code expectations

### File / module layout

```
.
├── Cargo.toml                          # Workspace root
├── Cargo.lock                          # Committed
├── rust-toolchain.toml                 # Pin stable
│
├── crates/
│   ├── runtime-core/
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   ├── src/
│   │   │   ├── lib.rs                  # Re-exports + module tree
│   │   │   ├── generated/              # typify output (committed)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── common.rs           # from schemas/common.v1.json
│   │   │   │   ├── framework.rs        # from schemas/framework.v1.json
│   │   │   │   ├── skill.rs
│   │   │   │   ├── tool.rs
│   │   │   │   └── agent.rs
│   │   │   ├── event.rs                # AgentEvent (hand-curated; see note below)
│   │   │   ├── drone.rs                # DroneEvent / DroneCommand
│   │   │   ├── error.rs                # thiserror types
│   │   │   └── tests/                  # Integration tests for round-trips
│   │   └── tests/                      # Cross-crate integration if any
│   │
│   ├── runtime-drone/
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   ├── src/
│   │   │   ├── main.rs                 # Binary entry: clap CLI args, tokio runtime
│   │   │   ├── lib.rs                  # Library exports for testing
│   │   │   ├── ipc.rs                  # Unix socket / named pipe + framed JSON codec
│   │   │   ├── heartbeat.rs            # Tokio task: 5s interval, emits + writes
│   │   │   ├── snapshot.rs             # Snapshot writer + state_hash
│   │   │   ├── shutdown.rs             # Graceful + SIGTERM emergency
│   │   │   ├── db.rs                   # SQLite open + WAL pragmas + schema init
│   │   │   └── command_handler.rs      # Dispatches DroneCommand variants
│   │   ├── tests/                      # Integration tests (subprocess spawn)
│   │   └── fuzz/                       # cargo-fuzz harnesses
│   │       └── fuzz_targets/
│   │           └── drone_command_decode.rs
│   │
│   ├── runtime-main/                   # STUB — only lib.rs with `pub fn placeholder() {}`
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   │
│   ├── runtime-sandbox/                # STUB — only lib.rs
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   │
│   └── xtask/
│       ├── Cargo.toml
│       └── src/main.rs                 # `regenerate-types` subcommand calls typify
│
├── src-tauri/                          # STUB — minimal Tauri config
│   ├── Cargo.toml
│   ├── tauri.conf.json                 # Empty allowlist; no commands yet
│   ├── build.rs
│   └── src/main.rs                     # tauri::Builder::default().run()
```

### A note on `AgentEvent`

`AgentEvent` is the typed event union the runtime emits. Per the spec it has many variants spanning multiple subsystems (agent lifecycle, tool/skill, gap, plan/task, hook, rail, budget, mode, capability, etc.). At M1, only a handful of variants are *needed* by the drone (none directly — the drone is below the AgentEvent layer; AgentEvent comes in M2). However, you should **define the full enum at M1** so it's stable across milestones. Variants that aren't yet emitted by anything are still valid Rust, just unused.

`AgentEvent` is **hand-curated, not generated** — it's a tagged enum where the variant names matter for downstream code. Generated types from schemas cover the *artifact shapes*; the *event taxonomy* lives in `event.rs` and is curated to match spec §2 + §2a + §2b + §3a + §3b + §4a + §4b + §6a + §8.security exactly.

If a variant is added in a later milestone, it's an additive minor change to `runtime-core` (semver-minor). Removing or restricting a variant is a breaking change requiring an ADR.

### Patterns to match

- **`Cargo.toml` style:** workspace dependencies in the root `Cargo.toml`'s `[workspace.dependencies]`; member crates reference them with `dep.workspace = true`. Avoids version drift across crates.
- **Error types:** `thiserror::Error` at library boundaries; `anyhow::Result` only in `main.rs` of binaries (`runtime-drone`, `xtask`). No `anyhow` in library crates.
- **Logging:** `tracing` crate with `tracing-subscriber` for the binary. JSON-format logs in production (Windows file output via `tracing-appender`). Never use `println!` for anything but explicit user-facing output (which the drone has none of — its output is the IPC socket).
- **CLI args:** `clap` with derive macro. Subcommands not needed in M1 (drone is one binary with flat args).
- **Async:** `#[tokio::main(flavor = "multi_thread", worker_threads = 4)]` for the drone binary. Library code is generic over a runtime where reasonable; concrete `tokio::spawn` is fine for tasks the drone owns.
- **Time:** `tokio::time::sleep` and `tokio::time::interval`. Tests use `tokio::time::pause()` + `advance()`. Never `std::thread::sleep` in async code.

### Naming for this milestone

- Crate names: kebab-case (`runtime-core`, `runtime-drone`, `runtime-main`, `runtime-sandbox`, `xtask`).
- Module names: snake_case, short (`ipc`, `heartbeat`, `snapshot`, `shutdown`, `db`).
- Type names: as in spec — `DroneEvent`, `DroneCommand`, `ActivityState`, `StopReason`, `ProcessType`, `ProcessConfig`, `AlertLevel`, `RevertReason`. CamelCase.
- Generated module names: match the schema file names (`common.rs`, `framework.rs`, `skill.rs`, `tool.rs`, `agent.rs`).

### What NOT to write

- **Async drama.** `tokio::spawn` is fine; don't reach for `tokio::task::spawn_blocking` unless you have a CPU-bound or blocking-IO operation. SQLite via `rusqlite` is technically blocking, but the drone is low-throughput (~1 op/sec); a single mutex-guarded connection is fine. M2's main process may need a dedicated thread; M1's drone doesn't.
- **Custom serialization.** Use `serde` derives. Don't hand-write `Serialize`/`Deserialize` impls for anything in M1.
- **Custom IPC framing.** Use `tokio_util::codec::LengthDelimitedCodec` or `LinesCodec` with JSON. Don't roll your own.
- **Premature optimization.** Connection pooling, write batching, snapshot compression — none of it in M1. Single connection, one write per command, raw JSON. Optimize at M3+ if profiling justifies it.
- **Tauri commands that aren't `placeholder`.** The Tauri stub is empty by design. Real commands land in M2.

---

## Verification commands

Run these in order. All must pass before the milestone is considered done.

```bash
# 1. Format check
cargo fmt --all -- --check
# Expected: no output, exit 0
# On failure: run `cargo fmt --all` and re-run check

# 2. Lint
cargo clippy --workspace --all-targets -- -D warnings
# Expected: no warnings, exit 0
# On failure: read each warning; fix or document with `#[allow(clippy::...)]`
# + linked issue per CLAUDE.md anti-patterns

# 3. Build (debug + release)
cargo build --workspace
cargo build --workspace --release
# Expected: succeeds on Linux, macOS, Windows
# On failure: most likely platform-specific dep issue (e.g., named pipe vs unix
# socket cfg gates) or missing tokio feature flags

# 4. Type-generation drift check
cargo xtask regenerate-types --check
# Expected: exit 0, no diff between regenerated and committed types
# On failure: run `cargo xtask regenerate-types` and commit the result; this
# means a schema changed without the types being regenerated

# 5. Tests
cargo test --workspace
# Expected: all pass
# On failure: read the specific test; check if it's a setup issue
# (`tokio::time::pause` not enabled, etc.) before assuming production code

# 6. Doc tests
cargo test --workspace --doc
# Expected: all pass
# On failure: a doc-comment example doesn't compile or its assertion failed

# 7. Doc build (strict)
cargo doc --workspace --no-deps -- -D rustdoc::missing_docs -D rustdoc::broken_intra_doc_links
# Expected: succeeds with no warnings
# On failure: a `pub` item is missing a doc comment, or an intra-doc link is broken

# 8. Coverage
cargo llvm-cov --workspace --html --output-dir target/coverage/html
cargo llvm-cov report --workspace --fail-under-lines 80
# Then verify drone-specific coverage:
cargo llvm-cov report --workspace --package runtime-drone --fail-under-lines 100
# Expected: workspace ≥80%, runtime-drone ≥100%
# On failure: identify uncovered lines via the HTML report; add tests

# 9. Audit
cargo audit
# Expected: no high or critical advisories
# On failure: update the affected dependency; if no fix exists, file an issue
# and add to `cargo-deny` ignores with linked rationale

# 10. Deny (license + duplicate-version + unmaintained)
cargo deny check
# Expected: passing (deny.toml committed at repo root)
# On failure: see deny.toml output for which rule failed; usually a license
# conflict (block GPL/AGPL) or a duplicate major version of a transitive dep

# 11. Fuzz harness compile + 30s smoke
cargo fuzz run drone_command_decode -- -max_total_time=30
# Expected: no panics or hangs in 30s
# On failure: read the panicking input from the corpus; fix the decoder

# 12. Manual drone smoke test (validates the integration test setup)
mkdir -p /tmp/drone-smoke
cargo run --bin runtime-drone -- \
  --session-id smoke \
  --db-path /tmp/drone-smoke/d.sqlite \
  --ipc-socket /tmp/drone-smoke/d.sock &
DRONE_PID=$!
sleep 6  # let one heartbeat fire
kill -TERM $DRONE_PID
wait $DRONE_PID
# Expected: drone exits cleanly with emergency snapshot in DB
# Check: sqlite3 /tmp/drone-smoke/d.sqlite "SELECT reason FROM snapshots;"
# should include 'sigterm'
```

---

## Self-correction guidance

### Likely failure modes for this milestone

| Failure | Likely cause | First thing to check |
|---|---|---|
| `cargo build` fails on Windows only | Unix-socket `cfg` gate is wrong; named-pipe path not handled | `crates/runtime-drone/src/ipc.rs` — `#[cfg(unix)]` vs `#[cfg(windows)]` |
| Heartbeat test hangs | `tokio::time::pause()` not in scope OR `interval` was created before `pause` | Test setup order — `pause()` must be called before any `tokio::time::*` resource is constructed |
| Snapshot test sees no row | SQLite connection wasn't committed; transaction not closed | `crates/runtime-drone/src/snapshot.rs` — confirm explicit commit; check `synchronous = NORMAL` is set |
| WAL pragma test fails | Pragmas executed in wrong order or against the wrong connection | `crates/runtime-drone/src/db.rs` — order matters: `journal_mode = WAL` must be first; `busy_timeout` after that |
| SIGTERM test exits 1 not 0 | Emergency snapshot path panicked instead of returning Result | `crates/runtime-drone/src/shutdown.rs` — wrap snapshot in `if let Err(_) = ... { tracing::error!(...) }` and exit 0 anyway; emergency snapshot is best-effort |
| `cargo test --doc` fails | Doc example imports a path not exposed by `pub use` | Check `lib.rs` re-exports; doc examples run as if from a downstream crate |
| `cargo deny check` fails on duplicate version | Two crates depend on different majors of same dep | Read `cargo deny check` output; usually fixable via `[patch.crates-io]` or by aligning a dep version |
| Coverage on `runtime-drone` is <100% | A branch (e.g., the SIGTERM emergency-snapshot path) has no test | Open the HTML coverage report; identify red lines; add a targeted test |
| `cargo xtask regenerate-types --check` fails | Schema was edited without regenerating types, OR typify config differs from committed types | Run `cargo xtask regenerate-types` and inspect the diff. If diff is intentional → commit. If unintentional → revert schema change. |
| CI green locally but red on Windows runner | Path separators (`/` vs `\`), file casing, or `tokio` feature flag missing on Windows | Read the Windows CI log; reproduce locally with `cargo build --target x86_64-pc-windows-gnu` if cross-compile available; otherwise rely on CI iteration |

### Escalate if

- After 3 self-correction iterations, any gate is still failing
- A schema change is required for a type to compile correctly (this is an ADR-required event)
- A dependency not listed in spec or ADR is needed (M1 should not need any beyond what's in spec)
- The Windows-only `runtime-drone` IPC path needs a non-trivial workaround (named-pipe semantics differ from Unix socket semantics; some divergence is acceptable but document it)

Per CLAUDE.md §12, escalation surfaces:
- What you tried (1 line per attempt)
- Current failures (full output, not summarized)
- Best current hypothesis
- What you would try next, if anything

---

## Deliverables

After M1, these files exist and are committed:

**Workspace + tooling:**
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`
- `deny.toml` (cargo-deny config)
- `crates/xtask/` (full implementation: `regenerate-types` subcommand)

**Real implementations:**
- `crates/runtime-core/` — `lib.rs`, `event.rs`, `drone.rs`, `error.rs`, `generated/*.rs`, `README.md`
- `crates/runtime-drone/` — `main.rs`, `lib.rs`, `ipc.rs`, `heartbeat.rs`, `snapshot.rs`, `shutdown.rs`, `db.rs`, `command_handler.rs`, `tests/`, `fuzz/`, `README.md`

**Stubs (intentional):**
- `crates/runtime-main/` — empty `lib.rs`
- `crates/runtime-sandbox/` — empty `lib.rs`
- `src-tauri/` — minimal Tauri config + empty webview

**Tests:**
- All 17 tests listed in the TDD plan, passing
- Fuzz harness for `drone_command_decode`

These files are updated:

- `CHANGELOG.md` — `[Unreleased]` entry
- `.github/workflows/ci.yml` — already has gated jobs; M1 makes them activate. No edits needed unless an M1-specific step is required (e.g., `cargo xtask regenerate-types --check` if not already present).
- `docs/MVP-v0.1.md` — mark M1 acceptance criteria as checked

---

## PR + commit rule

Per **`CLAUDE.md` §8 PR + commit workflow** — Claude does not commit until the user explicitly approves.

When all acceptance criteria are checked and all gates pass:

1. Run a final `git status` and `git diff --stat HEAD`.
2. Re-run all quality gates and capture exact results.
3. Draft the PR description following `.github/PULL_REQUEST_TEMPLATE.md`. Include all required sections.
4. **Surface to the user** — PR title, PR description (markdown), diff stat, gate results.
5. State explicitly: *"I will not commit until you approve."*
6. Wait for explicit approval before any `git commit` or `git push`.

PR notes specific to M1:

- This milestone produces a **lot** of files (~30+ new files, several thousand lines). Consider splitting into 3–5 logical commits within the same branch:
  1. Workspace skeleton (`Cargo.toml`, `rust-toolchain.toml`, empty crates)
  2. `runtime-core` types + `xtask regenerate-types`
  3. `runtime-drone` Phase 1 implementation
  4. Tests + fuzz harnesses
  5. Documentation + CHANGELOG
- Use **merge-commit** (not squash) when merging this PR — the per-commit history is valuable for understanding how the foundation was built. The maintainer can override.
- This PR will trigger CI for the first time on a real codebase. **Inspect the CI run** before drafting the merge — confirm Linux/macOS/Windows × stable/MSRV all pass. If any cell fails, fix before drafting (don't surface a PR with red CI).
- DCO sign-off mandatory: every commit `git commit -s -m "..."`.

---

## Milestone-specific gotchas

1. **WAL pragma order matters.** Set `journal_mode = WAL` first, then `synchronous = NORMAL`, `busy_timeout = 5000`, `foreign_keys = ON`. Out-of-order can cause silent failures (e.g., `busy_timeout` set before WAL mode is applied to a transaction-mode connection).
2. **`busy_timeout = 5000` is the magic number.** Skipping it causes flaky tests under contention. Don't omit it. Don't lower it.
3. **`tokio::time::pause()` must be called BEFORE any `tokio::time::*` resource is constructed.** A `Interval` created before `pause()` keeps real-time semantics. Test setup ordering matters.
4. **SIGTERM on Windows isn't a thing.** Windows uses `CTRL_BREAK_EVENT` / `CTRL_C_EVENT`. Use `tokio::signal::windows::ctrl_break()` and `ctrl_c()` for Windows; `tokio::signal::unix::SignalKind::terminate()` and `interrupt()` for Unix. Test #9 needs a Windows variant.
5. **Unix domain socket vs named pipe semantics differ.** Unix sockets are filesystem objects with permissions; named pipes are kernel objects with security descriptors. The `crates/runtime-drone/src/ipc.rs` module needs platform-specific code paths gated on `#[cfg(unix)]` and `#[cfg(windows)]`. Don't try to abstract them as if they're identical.
6. **`typify` generates types from schemas, but it's not perfect.** Some schema constructs (`oneOf` with discriminator, `$ref` to external files, etc.) may need post-processing. If the generated type doesn't compile or doesn't round-trip, your options are: (a) adjust the schema to be more typify-friendly, (b) post-process the generated output in `xtask`, or (c) add a `#[serde(...)]` annotation by hand-editing the generated file (last resort — leaves a maintenance trap).
7. **Hand-edited generated types fail the drift check.** If you must edit a generated file, the drift check fails on the next regeneration. Either: (a) commit the edit and accept the drift (escalate to user), or (b) fix the generation in `xtask` so the edit is unnecessary.
8. **The `AgentEvent` variants seem like overkill at M1.** They are; M1 doesn't emit any of them. But defining the full enum now means M2+ doesn't have to add variants behind feature flags or via breaking changes. Define it; comment-link variants to the spec sections that introduce them.
9. **Sub-process integration tests (test #17) are flaky by default.** Use a `tempfile`-backed socket path; capture stdout/stderr; assert on specific stderr lines for liveness rather than `sleep`. If a test must wait for a heartbeat, use `tokio::time::sleep(Duration::from_millis(5500))` once and check; don't spin-wait.
10. **`cargo-fuzz` requires nightly Rust** for the `cargo +nightly fuzz` invocation. CI's fuzz job uses a nightly toolchain just for that step; the rest stays on stable. The `rust-toolchain.toml` doesn't need to specify nightly; the fuzz CI step does it inline (`uses: dtolnay/rust-toolchain@nightly`).

---

## Milestone-specific anti-patterns

- **Implementing Phase 2+ logic in `runtime-main`.** It's a stub for M1. If you find yourself adding more than `pub fn placeholder() {}`, you're scope-creeping into M2.
- **Hand-writing types that should come from schemas.** If `runtime-core/src/framework.rs` has a manually-typed struct, that's a bug. All types from schemas live in `runtime-core/src/generated/`.
- **Using `println!` or `eprintln!` for drone logging.** Use `tracing`. The drone's stdout/stderr are reserved for log output (per spec §1d), not IPC.
- **Skipping the fuzz harness "because the parser is simple."** The IPC frame parser handles untrusted bytes from another process. Even if it's "just JSON," fuzz it. 30 seconds in CI is cheap.
- **Adding a custom `Drop` impl on the drone connection.** SQLite's `rusqlite::Connection` already handles cleanup. Custom `Drop` introduces ordering bugs.
- **Tests that depend on real time.** Use `tokio::time::pause()`. Wall-clock tests are flaky on CI runners.
- **Fixing a CI Windows-only failure by adding `#[cfg(unix)]` to skip the test.** That's hiding the bug. Either the test should be platform-conditional in a meaningful way (e.g., a Unix-socket test that doesn't apply to Windows) or it should pass on both platforms. Skipping silently is the bad pattern.

---

## Time-box (soft)

- **Reading + planning:** 30–45 minutes (re-read CLAUDE.md, spec sections, schemas; state deliverable + test plan; wait for confirmation)
- **Workspace skeleton + xtask + type generation:** 4–6 hours (the boring-but-load-bearing work)
- **TDD red phase (write all 17 tests + fuzz harness):** 6–8 hours
- **TDD green phase (implement drone + IPC + heartbeat + snapshot + shutdown):** 12–18 hours
- **Refactor + polish + doc comments + READMEs:** 4–6 hours
- **CI iteration (cross-OS green):** 2–4 hours (often the longest tail; budget for it)
- **Gate verification + PR drafting:** 30–45 minutes

**Total estimated:** 30–50 hours of Claude execution. ~10 hours of human direction (clarifications, mid-flight scope checks, PR review).

If actual time exceeds 2× the estimate, surface it. Likely cause: a typify edge case requiring schema rework, or a Windows-only IPC issue requiring platform-specific design.

---

*End of M01 prompt.*
