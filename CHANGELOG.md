# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
