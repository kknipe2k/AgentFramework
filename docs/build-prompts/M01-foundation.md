# M01 Foundation — Specification + Stage Prompts

**Date:** 2026-04-28
**Status:** Design approved — implement one stage at a time, in order
**Scope:** Cargo workspace + empty crates + Tauri stub + type-generation pipeline + drone Phase 1 + fuzz harness + 100% drone coverage + per-crate READMEs + CI green on Linux/macOS/Windows × stable + MSRV.

This document combines the M01 specification with the stage prompts that drive its implementation. It follows the per-milestone-as-PR pattern: stages A–D commit to a single feature branch (`claude/m01-foundation`); one PR for the parent milestone; each stage produces a per-stage retrospective per `CLAUDE.md` §19.

---

## Background and Design Decision

**Problem:** The repo has a complete spec, schemas, examples, and OSS scaffolding. It has zero Rust code. Every later milestone depends on a working drone (the survival layer), runtime-core types from schemas (the shared vocabulary), CI green (the quality gate), and the Cargo workspace skeleton (the structural foundation).

**Solution:** Land the foundation as four sequential stages on one feature branch, each its own commit, all surfacing one PR at the end:

- **Stage A — Workspace Skeleton.** Cargo workspace, empty crates, Tauri stub, lint configuration, CI activation. Validates the build pipeline before any meaningful code exists.
- **Stage B — Type Generation Pipeline.** `xtask regenerate-types` using `typify`; types from `schemas/*.v1.json` written to `crates/runtime-core/src/generated/`; hand-curated event taxonomy (`AgentEvent`, `DroneEvent`, `DroneCommand`); CI drift check.
- **Stage C — Drone Phase 1 Implementation.** `runtime-drone` per spec §1: heartbeat + snapshot + IPC + SIGTERM. **100% line coverage** on the safety primitive.
- **Stage D — Fuzz, Coverage, Polish.** `cargo-fuzz` harness for the IPC frame decoder; nightly fuzz workflow; per-crate READMEs; cross-OS CI verification; CHANGELOG re-organized; M1 acceptance criteria fully checked.

**Why one PR for the parent milestone (not one PR per stage):** Sub-milestone-as-PR was over-engineering. Stages-as-commits-on-one-branch gives the same incremental discipline (each stage commits cleanly before the next starts) without quadrupling PR ceremony. User reviews one M01 PR with four commits + four per-stage retrospectives.

**Why stages, not a single 540-line prompt:** Per `TEMPLATE.md` scope-split rule, milestones >250 prompt-lines or >12h work get staged. The original M01 prompt was 540 lines — too much for a fresh-session opening message. Stages A–D each have their own X.5 CLI Prompt (~80–150 lines) that gets pasted into a fresh Claude Code session; the rest of this document is the spec the prompts reference.

**Key constraints:**
- v0.1 scope per spec §0d: Windows-first, single-session, STANDARD mode hardcoded, `fresh_context_per_task` only, Anthropic-only, no telemetry, Novice + Promoted tiers.
- Drone is a safety primitive per `CLAUDE.md` §5: 100% line coverage on `crates/runtime-drone`.
- Schemas are the source of truth per `CLAUDE.md` §14: hand-written types in `runtime-core/src/generated/` are forbidden; CI fails if committed types differ from regenerated.
- Drone IPC is platform-specific (Unix domain socket vs Windows named pipe) per spec §1d.

**License:** Apache 2.0; DCO sign-off on every commit (`git commit -s`).

**Existing patterns to mirror:**
- `agent-runtime-spec.md` §1 + §1c + §1d for drone behavior, SQLite WAL, IPC framing.
- `schemas/README.md` for the type-generation pipeline.
- `examples/aria/framework.json` and frontmatter files for round-trip validation in tests.
- `CLAUDE.md` §6 for the must-pass gates list; §7 for self-correction; §8 for PR + commit workflow; §9 for style + anti-patterns; §15 for common gotchas; §19 for retrospective protocol.

---

## Document Structure

| Stage | Summary | Estimated effort |
|---|---|---|
| **A** | Workspace skeleton + Tauri stub + CI green | ~5–8h |
| **B** | xtask + typify + runtime-core types from schemas + drift check | ~6–10h |
| **C** | Drone Phase 1 implementation (heartbeat + snapshot + IPC + SIGTERM) + 100% coverage | ~12–18h |
| **D** | Fuzz harness + workspace coverage + per-crate READMEs + cross-OS verification | ~4–6h |

Total: 27–42 hours Claude execution. ~10 hours human direction.

---

## Implementation Workflow

Each stage runs through this exact cycle:

```
1. /clear                     — fresh context (only between stages)
2. Paste CLI Prompt below     — Claude writes failing tests first, then implements
3. cargo test --workspace     — confirm new tests fail before any production code
4. implement                  — Claude makes production changes
5. cargo test --workspace     — all tests green
6. cargo clippy + fmt + audit — zero warnings
7. cargo llvm-cov             — coverage threshold met
8. cargo +nightly fuzz (D only) — fuzz harness runs cleanly
9. fill in retrospective      — docs/build-prompts/retrospectives/M01.[A-D]-retrospective.md
10. commit (no push)          — exact commit message provided per stage
11. user reviews + approves   — Claude does NOT push without approval
12. push + continue           — to next stage on same branch
```

**Rule:** If a new test passes before implementation, the test is wrong — stop and fix the test.

**Rule:** Stages are sequential. Stage B does not start until Stage A's commit is on the feature branch (locally is sufficient; push is optional). Stage D does not push the PR until D is committed.

**Rule per `CLAUDE.md` §8:** Claude does not commit without user approval. After tests pass + retrospective filled, Claude surfaces the diff stat + retrospective + draft commit message. User approves; Claude commits.

---

## Stage A — Workspace Skeleton

### A.1 Problem Statement

Establish the Cargo workspace with empty member crates and a Tauri stub. CI activates the gated Rust jobs (currently skipped because `Cargo.toml` doesn't exist). No real implementation in any crate — each has only `pub fn placeholder()` and a single trivial test confirming the test runner executes.

The success criterion is "CI is green on a real Cargo workspace and Stage B can begin."

**New artifacts:**
- `Cargo.toml` at repo root (workspace root)
- `Cargo.lock` (committed)
- `rust-toolchain.toml` pinning current stable Rust
- `deny.toml` (cargo-deny policy)
- `crates/runtime-core/`, `crates/runtime-main/`, `crates/runtime-drone/`, `crates/runtime-sandbox/`, `crates/xtask/` — each with `Cargo.toml` + `src/lib.rs` (and `src/main.rs` for crates that have a binary)
- `src-tauri/` — Tauri config + empty webview entry

### A.2 Files to Change

| File | Change |
|---|---|
| `Cargo.toml` | **New** — workspace root with members + workspace lints |
| `Cargo.lock` | **New** — committed |
| `rust-toolchain.toml` | **New** — pin stable |
| `deny.toml` | **New** — cargo-deny policy |
| `crates/runtime-core/Cargo.toml` | **New** — package manifest |
| `crates/runtime-core/src/lib.rs` | **New** — placeholder + test |
| `crates/runtime-main/Cargo.toml` | **New** |
| `crates/runtime-main/src/lib.rs` | **New** — placeholder + test |
| `crates/runtime-drone/Cargo.toml` | **New** |
| `crates/runtime-drone/src/lib.rs` | **New** — placeholder + test |
| `crates/runtime-drone/src/main.rs` | **New** — `println!("not yet implemented"); std::process::exit(0)` |
| `crates/runtime-sandbox/Cargo.toml` | **New** |
| `crates/runtime-sandbox/src/lib.rs` | **New** — placeholder + test (overrides workspace `unsafe_code = "warn"`) |
| `crates/xtask/Cargo.toml` | **New** |
| `crates/xtask/src/main.rs` | **New** — placeholder; real implementation in Stage B |
| `lefthook.yml` | **New** — pre-commit hook config (resolves CLAUDE.md §12 TBD) |
| `src-tauri/Cargo.toml` | **New** |
| `src-tauri/tauri.conf.json` | **New** — empty allowlist, minimal config |
| `src-tauri/build.rs` | **New** — Tauri build script |
| `src-tauri/src/main.rs` | **New** — `tauri::Builder::default().run(...)` |
| `src-tauri/icons/` | **New** — placeholder icon |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` entry naming Stage A |

### A.3 Detailed Changes

#### Workspace `Cargo.toml`

```toml
[workspace]
resolver = "2"
members = [
    "crates/runtime-core",
    "crates/runtime-main",
    "crates/runtime-drone",
    "crates/runtime-sandbox",
    "crates/xtask",
    "src-tauri",
]

[workspace.package]
version      = "0.1.0"
edition      = "2021"
license      = "Apache-2.0"
repository   = "https://github.com/kknipe2k/AgentFramework"
rust-version = "1.80"

# Shared dependencies — populated by later stages; empty in Stage A.
[workspace.dependencies]

[workspace.lints.rust]
unsafe_code = "forbid"
warnings    = "deny"

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
nursery  = { level = "warn", priority = -1 }
```

#### Per-crate `Cargo.toml` template

```toml
[package]
name         = "<crate-name>"           # runtime-core, runtime-main, etc.
version      = { workspace = true }
edition      = { workspace = true }
license      = { workspace = true }
repository   = { workspace = true }
rust-version = { workspace = true }

[lints]
workspace = true
```

#### `runtime-sandbox` lint override

`crates/runtime-sandbox/Cargo.toml` ends with:

```toml
[lints]
workspace = true

# Sandbox needs unsafe for seccomp / landlock / Job Objects in M5+.
# v0.1 has no unsafe blocks yet; warn-only keeps the door open.
[lints.rust]
unsafe_code = "warn"
```

#### `lib.rs` template (per crate)

```rust
//! Runtime <crate-name> placeholder.
//!
//! Real implementation lands in subsequent stages of M01 and later milestones.

/// Returns the string `"ok"`. Placeholder for Stage A; real exports come later.
///
/// # Examples
///
/// ```
/// assert_eq!(<crate_name>::placeholder(), "ok");
/// ```
#[must_use]
pub fn placeholder() -> &'static str {
    "ok"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_returns_ok() {
        assert_eq!(placeholder(), "ok");
    }
}
```

Replace `<crate-name>` and `<crate_name>` with the actual crate name (e.g., `runtime-core` and `runtime_core`). Five lib.rs files total.

#### `runtime-drone/src/main.rs` (placeholder binary)

```rust
//! runtime-drone binary placeholder.
//!
//! Real implementation in Stage C.

fn main() {
    println!("runtime-drone: not yet implemented (Stage C)");
}
```

#### `xtask/src/main.rs` (placeholder)

```rust
//! xtask placeholder.
//!
//! `regenerate-types` subcommand lands in Stage B.

fn main() {
    eprintln!("xtask: subcommands land in Stage B");
    std::process::exit(0);
}
```

#### `rust-toolchain.toml`

```toml
[toolchain]
channel    = "1.80"          # MSRV pin
components = ["rustfmt", "clippy", "llvm-tools-preview"]
profile    = "minimal"
```

(Update `channel` to the latest stable as of work-start; document the choice in the PR description.)

#### `deny.toml` (cargo-deny policy)

```toml
[graph]
all-features = true

[advisories]
db-urls    = ["https://github.com/rustsec/advisory-db"]
yanked     = "deny"
ignore     = []

[licenses]
allow                       = ["Apache-2.0", "MIT", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-DFS-2016", "Zlib"]
confidence-threshold        = 0.9
unused-allowed-license      = "allow"

[bans]
multiple-versions = "warn"
wildcards         = "deny"
deny              = []
```

GPL/AGPL/LGPL are denied by default (not in `allow` list). Adjust `allow` in subsequent stages only if a dependency requires it (with PR rationale).

#### `src-tauri/tauri.conf.json` (empty allowlist)

```json
{
  "$schema": "https://schema.tauri.app/config/2.0",
  "productName": "agent-runtime",
  "version": "0.1.0",
  "identifier": "dev.aria-runtime.app",
  "build": {
    "beforeDevCommand":   "",
    "beforeBuildCommand": "",
    "devUrl":             "http://localhost:1420",
    "frontendDist":       "../dist"
  },
  "app": {
    "windows": [
      {
        "label":  "main",
        "title":  "Agent Runtime",
        "width":  1280,
        "height": 800
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active":     true,
    "targets":    ["msi"],
    "icon":       ["icons/icon.png"]
  }
}
```

#### `src-tauri/src/main.rs`

```rust
fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### `src-tauri/build.rs`

```rust
fn main() {
    tauri_build::build();
}
```

#### `src-tauri/icons/icon.png`

Use Tauri's default placeholder icon (32×32 PNG). The `cargo tauri build` invocation tolerates a 32px icon for v0.1; replace with branded assets at M11.

> **Implementer note:** Tauri's `cargo tauri init` scaffolding generates appropriate icons. Easiest path: run `cargo install create-tauri-app && cargo create-tauri-app` in a temp directory, copy the generated `src-tauri/icons/` to ours.

#### `lefthook.yml` (resolves CLAUDE.md §12 pre-commit TBD)

`lefthook` is the chosen pre-commit hook tool — single Go binary, no Python dependency, no language-specific runtime required (vs. `pre-commit` framework). Stage A wires `cargo fmt --check` and `cargo clippy` only; Stage B wires the schema drift-check; Stage D wires the full gate set.

```yaml
# Pre-commit hook configuration. Install once locally: `lefthook install`.
# CI mirrors the same gates so `--no-verify` is never a shortcut.
pre-commit:
  parallel: true
  commands:
    fmt:
      glob: "*.rs"
      run: cargo fmt --all -- --check
    clippy:
      glob: "*.rs"
      run: cargo clippy --workspace --all-targets -- -D warnings
```

Install instruction (added to repo root README in a later stage):

```bash
# Once per clone:
lefthook install
```

#### `CHANGELOG.md` `[Unreleased]` entry

Append under existing `[Unreleased] / Added`:

```markdown
- M01 Stage A: Cargo workspace skeleton — five member crates (runtime-core, runtime-main, runtime-drone, runtime-sandbox, xtask), Tauri stub (src-tauri/), workspace lints (deny warnings, forbid unsafe except sandbox, clippy pedantic + nursery), cargo-deny policy, lefthook pre-commit config. CI's gated Rust jobs activate. No real implementation; trivial placeholder per crate. (Stage B onward adds real code.)
```

### A.4 Tests

Five trivial tests, one per member crate. Test infrastructure validation only; coverage gates activate at Stage B+.

Each crate's `lib.rs` already contains the test (see `lib.rs` template in A.3):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_returns_ok() {
        assert_eq!(placeholder(), "ok");
    }
}
```

`cargo test --workspace` should find and run all five tests; all five should pass.

**No coverage gate** at Stage A — `cargo llvm-cov` is *runnable* but doesn't yet need to *pass* a threshold. Stage B introduces the first real coverage requirement.

### A.5 CLI Prompt

```
Read CLAUDE.md for all project rules.
Read docs/build-prompts/M01-foundation.md Stage A (sections A.1 through A.4).

═══ STEP 1 — WRITE FAILING TESTS ═══

Create the five lib.rs files (one per crate: runtime-core, runtime-main,
runtime-drone, runtime-sandbox, xtask) using the lib.rs template in section A.3.

The placeholder() function does not exist yet. The cargo workspace does not
exist yet. Therefore:

Run: cargo test --workspace
Confirm: command FAILS with "could not find Cargo.toml" — this is the right
failure for Stage A's pre-flight.

After creating the lib.rs files (with their tests but WITHOUT the workspace
infrastructure yet — no Cargo.toml at root, no per-crate Cargo.toml, no
rust-toolchain.toml), the tests cannot even compile. That's the correct
"red" state for Stage A.

═══ STEP 2 — IMPLEMENT ═══

1. Create Cargo.toml at repo root with the workspace declaration from A.3
   (members + lints + workspace dependencies stub).

2. Create rust-toolchain.toml with the toolchain pin from A.3 (channel set
   to current latest stable as of work-start; record the version in the PR
   description).

3. Create deny.toml with the cargo-deny policy from A.3.

4. Create per-crate Cargo.toml files using the template in A.3:
   - crates/runtime-core/Cargo.toml
   - crates/runtime-main/Cargo.toml
   - crates/runtime-drone/Cargo.toml      (with [[bin]] entry for runtime-drone)
   - crates/runtime-sandbox/Cargo.toml    (with the unsafe_code = "warn" override)
   - crates/xtask/Cargo.toml

5. Create the five lib.rs files from the template (already done in STEP 1
   if test-first was followed; otherwise create them now).

6. Create crates/runtime-drone/src/main.rs with the placeholder binary.
7. Create crates/xtask/src/main.rs with the placeholder.

8. Set up src-tauri/:
   - Cargo.toml (Tauri 2.x; package name "agent-runtime")
   - tauri.conf.json from A.3
   - build.rs from A.3
   - src/main.rs from A.3
   - icons/icon.png (use Tauri's default scaffolding icon; see implementer note in A.3)

9. Run cargo build --workspace to download dependencies and confirm everything
   compiles. Cargo.lock is created at this step; commit it.

10. Add to CHANGELOG.md [Unreleased] / Added the entry from A.3.

═══ STEP 3 — VERIFY ═══
Run each gate; all must pass:
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo build --workspace
  cargo build --workspace --release
  cargo test --workspace               (5 trivial tests pass)
  cargo test --workspace --doc         (5 doc-test examples compile and pass)
  cargo doc --workspace --no-deps -- -D rustdoc::missing_docs
  cargo audit
  cargo deny check

If any gate fails, follow the self-correction loop in CLAUDE.md §7 (max 3
iterations, then surface). Do NOT bypass with --no-verify or #[allow(...)].

═══ STEP 4 — RETROSPECTIVE ═══

Per CLAUDE.md §19, copy retrospectives/RETROSPECTIVE-TEMPLATE.md to:
  docs/build-prompts/retrospectives/M01.A-retrospective.md

Fill in:
- Header (date, branch, starting commit)
- [LIVE] sections — friction/ambiguity/surface/protocol-drift/surprise events
  encountered during this stage
- [END] three-axis scoring (Process / Product / Pattern)
- Threshold gate evaluation (G1–G5; S1–S5)
- Decisions for Stage B (what to update in CLAUDE.md / TEMPLATE.md if any)

═══ STEP 5 — SURFACE TO USER ═══

Run: git status, git diff --stat HEAD
Re-run all gates one final time. Capture exact results.

Surface to user:
- Diff stat
- Gate results (each gate, pass/fail)
- Stage A retrospective (M01.A-retrospective.md)
- Draft commit message (from A.6 below)

State explicitly: "Stage A is ready. I will NOT commit until you approve.
Please review the retrospective and the diff."

Wait for explicit approval before any git commit. Do NOT push (push waits
until Stage D is approved; the entire M01 PR pushes together).
```

### A.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(workspace): M01 Stage A — Cargo workspace skeleton + CI activation

Establishes the Cargo workspace with five member crates (runtime-core,
runtime-main, runtime-drone, runtime-sandbox, xtask) and the Tauri stub
(src-tauri/). Workspace lints configured: deny warnings, forbid unsafe
(warn-only on runtime-sandbox where M5+ adds unsafe), clippy pedantic +
nursery. cargo-deny policy committed at deny.toml. rust-toolchain.toml
pins the toolchain.

Each crate has a placeholder() function with one trivial test confirming
the test runner executes; real implementation in subsequent stages.

CI's gated Rust jobs (cargo fmt/clippy/test/audit/deny/coverage) now
activate because Cargo.toml exists at root.

Stage B (type generation) starts on the same branch.

Refs: M01-foundation.md §A
Retrospective: docs/build-prompts/retrospectives/M01.A-retrospective.md

https://claude.ai/code/session_<id>
EOF
)"
```

---

## Stage B — Type Generation Pipeline

### B.1 Problem Statement

Make `runtime-core` the curated type vocabulary:

1. **Generated types** from `schemas/*.v1.json` via `typify` — `Framework`, `Skill`, `Tool`, `Agent`, `Capabilities`, `Provenance`, `HookRef`, `Hook`, `JSONLogicExpression`, etc. Source of truth lives in `schemas/`; CI fails if committed types differ from regenerated.

2. **Hand-curated event taxonomy** — `AgentEvent`, `DroneEvent`, `DroneCommand`, plus supporting enums (`ActivityState`, `StopReason`, `ProcessType`, `AlertLevel`, `RevertReason`). NOT generated; the contract every later milestone evolves. Variants that aren't yet emitted are still defined at Stage B; later milestones use them without breaking changes.

3. **`xtask regenerate-types` subcommand** — build tool that runs typify against schemas and writes to `crates/runtime-core/src/generated/`. `--check` mode for CI drift detection.

After Stage B, Stage C's drone serializes `DroneEvent` and deserializes `DroneCommand` against canonical types.

### B.2 Files to Change

| File | Change |
|---|---|
| `crates/runtime-core/Cargo.toml` | **Edited** — add deps: serde, serde_json, serde_yaml, thiserror, schemars (if needed) |
| `crates/runtime-core/src/lib.rs` | **Edited** — re-export from `event`, `drone`, `error`, `generated/` |
| `crates/runtime-core/src/generated/mod.rs` | **New** — re-export submodules |
| `crates/runtime-core/src/generated/common.rs` | **New** (auto-generated) — types from `schemas/common.v1.json` |
| `crates/runtime-core/src/generated/framework.rs` | **New** (auto-generated) |
| `crates/runtime-core/src/generated/skill.rs` | **New** (auto-generated) |
| `crates/runtime-core/src/generated/tool.rs` | **New** (auto-generated) |
| `crates/runtime-core/src/generated/agent.rs` | **New** (auto-generated) |
| `crates/runtime-core/src/event.rs` | **New** — hand-curated `AgentEvent` enum (full variant list per spec) |
| `crates/runtime-core/src/drone.rs` | **New** — hand-curated `DroneEvent`, `DroneCommand`, supporting enums per spec §1d |
| `crates/runtime-core/src/error.rs` | **New** — `RuntimeError` via thiserror |
| `crates/runtime-core/Cargo.toml` | (above) |
| `crates/xtask/Cargo.toml` | **Edited** — add deps: clap, typify, serde_json, anyhow, similar (for diff-check), regex |
| `crates/xtask/src/main.rs` | **Edited** — replace placeholder; add `regenerate-types` subcommand with `--check` flag |
| `.github/workflows/ci.yml` | **Edited** — add `cargo xtask regenerate-types --check` step gated on Cargo.toml existing |
| `crates/runtime-core/README.md` | **New** (basic; expanded in Stage D) — documents type-generation pipeline |
| `Cargo.toml` | **Edited** — add to `[workspace.dependencies]`: serde, serde_json, thiserror (shared), and others as needed |
| `tests/round_trip.rs` (in runtime-core) | **New** — integration tests for round-trip serialization of examples/aria + examples/ralph artifacts |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` entry naming Stage B |

### B.3 Detailed Changes

#### `crates/runtime-core/Cargo.toml` (deps added)

```toml
[package]
name         = "runtime-core"
# ... (workspace inherits unchanged)

[lints]
workspace = true

[dependencies]
serde       = { workspace = true, features = ["derive"] }
serde_json  = { workspace = true }
serde_yaml  = { workspace = true }
thiserror   = { workspace = true }

[dev-dependencies]
proptest    = { workspace = true }
```

Add to root `Cargo.toml` `[workspace.dependencies]`:

```toml
serde       = { version = "1.0", default-features = false }
serde_json  = "1.0"
serde_yaml  = "0.9"
thiserror   = "1.0"
proptest    = "1.4"
```

#### `crates/runtime-core/src/lib.rs` (replace Stage A placeholder)

```rust
//! Runtime core: shared types for the agent runtime.
//!
//! Types in `generated/` are emitted by `cargo xtask regenerate-types` from
//! `schemas/*.v1.json` — do not hand-edit them. Types in `event.rs`, `drone.rs`,
//! and `error.rs` are hand-curated; they are the contract every later milestone
//! evolves.

pub mod drone;
pub mod error;
pub mod event;
pub mod generated;

pub use drone::{ActivityState, AlertLevel, DroneCommand, DroneEvent, ProcessConfig, ProcessType, RevertReason, StopReason};
pub use error::RuntimeError;
pub use event::AgentEvent;
pub use generated::*;
```

#### `crates/runtime-core/src/generated/mod.rs` (header for the directory)

```rust
// Re-exports the typify-generated modules.
//
// All files in this directory are AUTO-GENERATED by `cargo xtask regenerate-types`
// from schemas/*.v1.json. Do not hand-edit.
//
// CI runs `cargo xtask regenerate-types --check` to enforce drift detection.

pub mod agent;
pub mod common;
pub mod framework;
pub mod skill;
pub mod tool;
```

#### `crates/runtime-core/src/generated/<schema>.rs` (auto-generated header)

Each generated file starts with:

```rust
// AUTO-GENERATED FILE — DO NOT EDIT
//
// Regenerate with: `cargo xtask regenerate-types`
// Source schema:   schemas/<name>.v1.json
// Generated by:    typify <version>
//
// Drift detection runs in CI via `cargo xtask regenerate-types --check`.

#![allow(clippy::pedantic, clippy::nursery)]  // typify output is opinionated; lints don't add value here

use serde::{Deserialize, Serialize};

// <typify output>
```

The lint `allow` is intentional — generated code shouldn't trigger pedantic lints meant for hand-written code.

#### `crates/runtime-core/src/event.rs` (hand-curated, full variant list)

The `AgentEvent` enum contains every variant the runtime emits across all milestones. v0.1 implements only a subset; the rest are valid Rust at Stage B but emitted by later milestones.

Reference variants (from spec §2 + §2a + §2b + §3a + §3b + §4a + §4b + §6a + §8.security):

```rust
//! AgentEvent — canonical event union emitted by the runtime.
//!
//! Variants span the spec sections they belong to; cross-references in
//! variant doc comments link back to the originating spec section.
//!
//! Variants that aren't yet emitted at v0.1 are still defined here so later
//! milestones extend the union additively (semver-minor, no breaking change).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    // ── Session lifecycle (spec §2) ──
    SessionStart {
        session_id: String,
        framework:  String,
        model:      String,
    },
    SessionEnd {
        session_id:   String,
        duration_ms:  u64,
        end_reason:   String,
    },

    // ── Agent lifecycle (spec §2) ──
    AgentSpawned {
        agent_id:   String,
        agent_name: String,
        parent_id:  Option<String>,
    },
    AgentComplete {
        agent_id: String,
        result:   String,
    },
    AgentError {
        agent_id: String,
        error:    String,
    },

    // ── Tool / Skill (spec §0b + §2) ──
    ToolInvoked {
        agent_id:  String,
        tool_name: String,
        input:     serde_json::Value,
    },
    ToolResult {
        agent_id:    String,
        tool_name:   String,
        output:      serde_json::Value,
        duration_ms: u64,
    },
    ToolError {
        agent_id:  String,
        tool_name: String,
        error:     String,
    },
    SkillLoaded {
        agent_id:   String,
        skill_name: String,
        mode:       Option<String>,
    },

    // ── Plan / Task lifecycle (spec §3a) ──
    PlanCreated   { plan_id: String, task_count: u32 },
    PlanApproved  { plan_id: String },
    PlanRejected  { plan_id: String, reason: String },
    TaskStarted   { plan_id: String, task_id: String, agent_id: String },
    TaskCompleted { plan_id: String, task_id: String, duration_ms: u64 },
    TaskFailed    { plan_id: String, task_id: String, error: String, failure_count: u32 },
    TaskRolledBack { plan_id: String, task_id: String, snapshot_id: String },
    TaskEscalated { plan_id: String, task_id: String, reason: String },

    // ── Mode (spec §3b) ──
    ModeChanged { from: String, to: String, reason: String },

    // ── Verify + Rails (spec §4a) ──
    VerifyStarted   { hook_id: String, level: String },
    VerifyPassed    { hook_id: String, duration_ms: u64 },
    VerifyFailed    { hook_id: String, error: String },
    RailTriggered   { rail_id: String, severity: String, message: String },

    // ── Gap detection (spec §4b) ──
    SkillMissing  { agent_id: String, skill_name: String, severity: String },
    ToolMissing   { agent_id: String, tool_name: String, severity: String },
    GapResolved   { agent_id: String, capability: String, kind: String },

    // ── HITL (spec §6a) ──
    HitlRequested  { agent_id: String, prompt: String, hitl_kind: String },
    HitlResolved   { agent_id: String, response: String, duration_ms: u64 },

    // ── Capability enforcement (spec §8.security) ──
    CapabilityViolation { agent_id: String, declared: String, attempted: String },
    CapabilityGrant     { agent_id: String, capability: String, scope: String },

    // ── Budget (spec §2a) ──
    BudgetWarn       { spent_usd: f64, cap_usd: f64, percent: u32 },
    BudgetDownshift  { from_model: String, to_model: String, reason: String },
    BudgetSuspended  { spent_usd: f64, cap_usd: f64 },
    BudgetExceeded   { spent_usd: f64, cap_usd: f64 },

    // ── Stream + decision trace (spec §2 + §2b) ──
    StreamText      { agent_id: String, text: String },
    DecisionRecord  { agent_id: String, decision: String, rationale: String, tool_used: Option<String> },
    TokenUsage      { input: u64, output: u64, model: String, cost_usd: f64 },
}
```

> **Implementer note:** This is the v0.1 "AgentEvent" surface. If you discover a variant the spec implies but isn't listed above (cross-reference §2 through §8.security), surface it before adding — variant additions need spec backing. The variant list will grow slightly in later milestones; that's expected and additive.

#### `crates/runtime-core/src/drone.rs` (hand-curated per spec §1d)

```rust
//! Drone IPC types — DroneEvent / DroneCommand and supporting enums.
//!
//! Specified in `agent-runtime-spec.md` §1d. These types are the wire format
//! for main↔drone IPC over Unix domain socket / Windows named pipe.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DroneEvent {
    Heartbeat { status: String, timestamp: i64 },
    SnapshotWritten { snapshot_id: String, session_id: String, reason: String, timestamp: i64 },
    ActivityStateChange { from: ActivityState, to: ActivityState },
    ProcessSpawned { pid: u32, process_type: ProcessType },
    ProcessStopped { pid: u32, reason: StopReason },
    RecoveryAvailable { session_id: String, snapshot_id: String },
    Alert { level: AlertLevel, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DroneCommand {
    SnapshotNow      { reason: String, state_json: serde_json::Value },
    GracefulShutdown { timeout_ms: u64 },
    SpawnProcess     { process_type: ProcessType, config: ProcessConfig },
    StopProcess      { pid: u32, force: bool },
    SetActivityTimeout { ms: u64 },
    RevertToSnapshot { snapshot_id: String, reason: RevertReason },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActivityState { Active, Idle, Stalled, TimedOut, UserAborted, Recovering }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason { Graceful, Crash, Timeout, UserAbort, ForceKill }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessType { Agent, Mcp, SkillSandbox }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlertLevel { Warn, Critical }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RevertReason { HookRollback, UserRollback, GapRecovery }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessConfig {
    pub command: String,
    pub args:    Vec<String>,
    pub env:     std::collections::HashMap<String, String>,
}
```

#### `crates/runtime-core/src/error.rs`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("schema validation failed: {0}")]
    SchemaValidation(String),

    #[error("type generation drift detected: {0}")]
    TypeDrift(String),

    #[error("invalid event payload: {0}")]
    InvalidEvent(String),

    #[error("invalid drone command: {0}")]
    InvalidCommand(String),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

#### `crates/xtask/src/main.rs` (replaces Stage A placeholder)

```rust
//! xtask: project-wide build/maintenance tasks.
//!
//! Subcommands:
//!   regenerate-types         — run typify against schemas/*.v1.json
//!                              and write to crates/runtime-core/src/generated/
//!   regenerate-types --check — regenerate to a temp dir, diff against
//!                              committed; non-zero exit on drift

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "xtask", about = "project build tasks")]
struct Args {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Regenerate Rust types from JSON schemas via typify.
    RegenerateTypes {
        /// Verify committed types match regenerated; exit non-zero on drift.
        #[arg(long)]
        check: bool,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Cmd::RegenerateTypes { check } => regenerate_types(check),
    }
}

fn regenerate_types(check: bool) -> Result<()> {
    use std::fs;
    let workspace_root = workspace_root()?;
    let schemas_dir    = workspace_root.join("schemas");
    let target_dir     = workspace_root.join("crates/runtime-core/src/generated");

    let schemas = ["common", "framework", "skill", "tool", "agent"];
    let mut all_drift = Vec::new();

    for name in schemas {
        let schema_path = schemas_dir.join(format!("{name}.v1.json"));
        let target_path = target_dir.join(format!("{name}.rs"));
        let generated   = generate_one(&schema_path, name)?;

        if check {
            let committed = fs::read_to_string(&target_path)
                .with_context(|| format!("read committed: {target_path:?}"))?;
            if committed != generated {
                all_drift.push(name.to_string());
            }
        } else {
            fs::write(&target_path, &generated)
                .with_context(|| format!("write {target_path:?}"))?;
        }
    }

    if check && !all_drift.is_empty() {
        anyhow::bail!(
            "type generation drift in: {}\nrun `cargo xtask regenerate-types` and commit the result",
            all_drift.join(", ")
        );
    }
    Ok(())
}

fn generate_one(schema_path: &std::path::Path, name: &str) -> Result<String> {
    let schema_text = std::fs::read_to_string(schema_path)
        .with_context(|| format!("read schema: {schema_path:?}"))?;
    let schema: typify::SchemaObject = serde_json::from_str(&schema_text)?;

    let mut type_space = typify::TypeSpace::new(&typify::TypeSpaceSettings::default());
    type_space.add_root_schema(schema)?;

    let header = format!(
        "// AUTO-GENERATED FILE — DO NOT EDIT\n\
         //\n\
         // Regenerate with: `cargo xtask regenerate-types`\n\
         // Source schema:   schemas/{name}.v1.json\n\
         // Generated by:    typify\n\
         //\n\
         // Drift detection runs in CI via `cargo xtask regenerate-types --check`.\n\
         \n\
         #![allow(clippy::pedantic, clippy::nursery)]\n\
         \n\
         use serde::{{Deserialize, Serialize}};\n\
         \n",
    );

    let body = type_space.to_stream().to_string();
    Ok(format!("{header}{body}\n"))
}

fn workspace_root() -> Result<PathBuf> {
    let metadata = std::process::Command::new("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .arg("--no-deps")
        .output()
        .context("cargo metadata")?;
    let json: serde_json::Value = serde_json::from_slice(&metadata.stdout)?;
    let workspace = json["workspace_root"]
        .as_str()
        .context("workspace_root in cargo metadata output")?;
    Ok(PathBuf::from(workspace))
}
```

> **Implementer note:** The exact typify API may shift slightly between versions. The critical invariants: (a) read schema, (b) emit Rust to a string, (c) compare-or-write against `generated/<name>.rs`, (d) on `--check` and drift, exit non-zero with the drifted names. If typify version pins matter, document them in `crates/xtask/Cargo.toml` with rationale.

#### `crates/xtask/Cargo.toml`

```toml
[package]
name         = "xtask"
version      = { workspace = true }
edition      = { workspace = true }
license      = { workspace = true }
repository   = { workspace = true }
rust-version = { workspace = true }

[lints]
workspace = true

[dependencies]
anyhow      = "1.0"
clap        = { version = "4", features = ["derive"] }
serde       = { workspace = true, features = ["derive"] }
serde_json  = { workspace = true }
typify      = "0.1"   # pin to specific version; document upgrades via PR
```

> Pin a specific typify version — different versions can produce different output, falsely failing the drift check. Upgrades are intentional and PR'd.

#### `.github/workflows/ci.yml` — add drift-check step

In the existing `rust:` job (gated on `Cargo.toml` existing), add after `cargo test --workspace`:

```yaml
      - name: Type generation drift check
        run: cargo xtask regenerate-types --check
        if: matrix.os == 'ubuntu-latest' && matrix.toolchain == 'stable'
        # Drift check runs once (not per OS/toolchain) — generated output is
        # platform-independent.
```

#### `crates/runtime-core/README.md` (basic; expanded in Stage D)

```markdown
# runtime-core

Shared types for the agent runtime.

## Type generation pipeline

Types in `src/generated/` are emitted by `cargo xtask regenerate-types` from
`schemas/*.v1.json`. **Do not hand-edit them** — CI's drift check (`cargo
xtask regenerate-types --check`) fails if committed types differ from
regenerated.

To change a type:

1. Edit `schemas/<name>.v1.json` (with ADR per `CLAUDE.md` §11 if it's a
   primitive change).
2. Run `cargo xtask regenerate-types`.
3. Commit the schema + generated output together.

## Hand-curated types

`event.rs` (`AgentEvent`), `drone.rs` (`DroneEvent`, `DroneCommand`), and
`error.rs` (`RuntimeError`) are NOT generated. They're the contract every
later milestone evolves. Adding a variant is semver-minor; removing or
restructuring is a breaking change requiring an ADR.

## License

Apache-2.0. See repo root `LICENSE` and `NOTICE`.
```

### B.4 Tests

Embedded as exact code (15 tests across runtime-core + xtask).

#### `crates/runtime-core/tests/round_trip.rs` (8 round-trip + property tests)

```rust
//! Round-trip serialization tests for runtime-core types.

use runtime_core::{AgentEvent, DroneCommand, DroneEvent};
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .to_path_buf()
}

#[test]
fn framework_v1_round_trip() {
    let path = workspace_root().join("examples/aria/framework.json");
    let text = std::fs::read_to_string(&path).expect("read examples/aria/framework.json");
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("parse aria framework");
    let into_typed: runtime_core::generated::framework::Framework = serde_json::from_value(parsed.clone())
        .expect("deserialize aria framework into typed Framework");
    let back: serde_json::Value = serde_json::to_value(&into_typed).expect("re-serialize");
    // Structural equality: don't compare strings (formatting may differ); compare parsed JSON.
    assert_eq!(parsed, back, "round-trip should be structurally identical");
}

#[test]
fn framework_ralph_round_trip() {
    let path = workspace_root().join("examples/ralph/framework.json");
    let text = std::fs::read_to_string(&path).expect("read examples/ralph/framework.json");
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("parse ralph framework");
    let into_typed: runtime_core::generated::framework::Framework = serde_json::from_value(parsed.clone())
        .expect("deserialize ralph framework");
    let back: serde_json::Value = serde_json::to_value(&into_typed).expect("re-serialize");
    assert_eq!(parsed, back);
}

#[test]
fn drone_event_serde_tags_correct() {
    let event = DroneEvent::Heartbeat { status: "ok".into(), timestamp: 1234567890 };
    let json = serde_json::to_value(&event).expect("serialize");
    assert_eq!(json["type"], "heartbeat", "tag must be snake_case 'heartbeat'");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["timestamp"], 1234567890i64);
}

#[test]
fn drone_command_serde_tags_correct() {
    let cmd = DroneCommand::SnapshotNow { reason: "task_boundary".into(), state_json: serde_json::json!({}) };
    let json = serde_json::to_value(&cmd).expect("serialize");
    assert_eq!(json["type"], "snapshot_now", "tag must be 'snapshot_now'");
}

#[test]
fn agent_event_session_start_round_trip() {
    let event = AgentEvent::SessionStart {
        session_id: "s1".into(),
        framework:  "examples/aria".into(),
        model:      "claude-sonnet-4-6".into(),
    };
    let json: serde_json::Value = serde_json::to_value(&event).unwrap();
    let back: AgentEvent = serde_json::from_value(json).unwrap();
    assert_eq!(event, back);
}

#[test]
fn agent_event_capability_violation_round_trip() {
    let event = AgentEvent::CapabilityViolation {
        agent_id:  "a1".into(),
        declared:  "tools_called: [Read]".into(),
        attempted: "Bash".into(),
    };
    let json: serde_json::Value = serde_json::to_value(&event).unwrap();
    let back: AgentEvent = serde_json::from_value(json).unwrap();
    assert_eq!(event, back);
}

#[test]
fn skill_planning_frontmatter_round_trip() {
    let path = workspace_root().join("examples/aria/skills/planning.md");
    let text = std::fs::read_to_string(&path).expect("read planning.md");

    // Extract frontmatter (between leading --- and second ---).
    let parts: Vec<&str> = text.splitn(3, "---\n").collect();
    assert!(parts.len() >= 3, "expected frontmatter delimited by ---");
    let frontmatter_yaml = parts[1];

    let parsed: serde_yaml::Value = serde_yaml::from_str(frontmatter_yaml).expect("parse frontmatter");
    let typed: runtime_core::generated::skill::Skill = serde_yaml::from_value(parsed.clone())
        .expect("deserialize into Skill");
    let back: serde_yaml::Value = serde_yaml::to_value(&typed).expect("re-serialize");
    // YAML round-trip is strict on shape but tolerant of key ordering.
    assert_eq!(parsed, back);
}

#[test]
fn drone_event_variant_count_matches_spec() {
    // If this test fails, you removed a variant — that's a breaking change.
    // Update this count if you added a variant in a later milestone (additive
    // changes are fine; keep this assertion in sync with spec §1d).
    let variants_in_drone_event = 7; // Heartbeat, SnapshotWritten, ActivityStateChange, ProcessSpawned, ProcessStopped, RecoveryAvailable, Alert
    let _check = match DroneEvent::Heartbeat { status: "".into(), timestamp: 0 } {
        DroneEvent::Heartbeat { .. }            => 1,
        DroneEvent::SnapshotWritten { .. }      => 2,
        DroneEvent::ActivityStateChange { .. }  => 3,
        DroneEvent::ProcessSpawned { .. }       => 4,
        DroneEvent::ProcessStopped { .. }       => 5,
        DroneEvent::RecoveryAvailable { .. }    => 6,
        DroneEvent::Alert { .. }                => 7,
    };
    let _ = variants_in_drone_event;
    // Test passes if it compiles — the match exhaustiveness check enforces variant count.
}
```

#### `crates/runtime-core/src/event.rs` — add proptest module

Append to `event.rs`:

```rust
#[cfg(test)]
mod proptest_round_trip {
    use super::*;
    use proptest::prelude::*;

    fn arb_session_start() -> impl Strategy<Value = AgentEvent> {
        (any::<String>(), any::<String>(), any::<String>())
            .prop_map(|(session_id, framework, model)| AgentEvent::SessionStart {
                session_id,
                framework,
                model,
            })
    }

    fn arb_tool_invoked() -> impl Strategy<Value = AgentEvent> {
        (any::<String>(), any::<String>())
            .prop_map(|(agent_id, tool_name)| AgentEvent::ToolInvoked {
                agent_id,
                tool_name,
                input: serde_json::json!({"key": "value"}),
            })
    }

    proptest! {
        #[test]
        fn session_start_round_trips(event in arb_session_start()) {
            let json: serde_json::Value = serde_json::to_value(&event).unwrap();
            let back: AgentEvent = serde_json::from_value(json).unwrap();
            prop_assert_eq!(event, back);
        }

        #[test]
        fn tool_invoked_round_trips(event in arb_tool_invoked()) {
            let json: serde_json::Value = serde_json::to_value(&event).unwrap();
            let back: AgentEvent = serde_json::from_value(json).unwrap();
            prop_assert_eq!(event, back);
        }
    }
}
```

#### `crates/runtime-core/src/drone.rs` — add proptest module

Same pattern: arbitrary strategies for `DroneEvent::Heartbeat`, `DroneCommand::SnapshotNow`, etc., with `proptest!` round-trip blocks.

#### `crates/xtask/tests/check_drift.rs` (3 xtask tests)

```rust
//! xtask drift-detection integration tests.
//! Requires the workspace to be in a state where committed types match generated
//! (i.e., this test runs from a clean checkout after Stage B's implementation).

use std::process::Command;

#[test]
fn regenerate_types_check_passes_when_in_sync() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["regenerate-types", "--check"])
        .output()
        .expect("run xtask");
    assert!(
        output.status.success(),
        "drift check should pass on a clean checkout. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn regenerate_types_writes_files() {
    // Run regenerate-types (without --check) and verify the generated/*.rs
    // files have the expected header.
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["regenerate-types"])
        .output()
        .expect("run xtask regenerate-types");
    assert!(output.status.success(), "regenerate-types should succeed");

    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .to_path_buf();
    let common_rs = workspace_root.join("crates/runtime-core/src/generated/common.rs");
    let text = std::fs::read_to_string(&common_rs).expect("read generated common.rs");
    assert!(text.contains("AUTO-GENERATED FILE"), "generated file should have auto-gen header");
    assert!(text.contains("typify"), "generated file should reference typify in header");
}

#[test]
fn regenerate_types_check_detects_drift() {
    // Read a generated file, mutate it temporarily, run --check, expect failure,
    // then restore. (Use a separate temp file to avoid actually mutating the
    // committed source if the test panics.)
    use std::fs;
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .to_path_buf();
    let target = workspace_root.join("crates/runtime-core/src/generated/common.rs");
    let original = fs::read_to_string(&target).expect("read original");

    // Mutate: append a comment.
    fs::write(&target, format!("{}\n// drift-test\n", original)).expect("write mutation");

    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["regenerate-types", "--check"])
        .output()
        .expect("run xtask --check");

    // Restore BEFORE asserting (so a panicking assertion doesn't leave the file dirty).
    fs::write(&target, original).expect("restore");

    assert!(
        !output.status.success(),
        "drift check should detect the mutation. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
```

#### `CHANGELOG.md` `[Unreleased]` entry

Append under `Added`:

```markdown
- M01 Stage B: Type-generation pipeline. xtask `regenerate-types` subcommand reads schemas/*.v1.json via typify and writes to crates/runtime-core/src/generated/. Hand-curated AgentEvent + DroneEvent + DroneCommand enums in event.rs / drone.rs (full variant lists per spec §2 + §1d). RuntimeError via thiserror. CI drift check active. examples/aria/framework.json and examples/ralph/framework.json round-trip through generated Framework type.
```

### B.5 CLI Prompt

```
Read CLAUDE.md for all project rules.
Read docs/build-prompts/M01-foundation.md Stage B (sections B.1 through B.4).
Stage A must be committed on this feature branch before starting Stage B.
Run `git log --oneline -1` and confirm the previous commit is Stage A.

═══ STEP 1 — WRITE FAILING TESTS ═══

Create the test files exactly as specified in B.4:
  - crates/runtime-core/tests/round_trip.rs (8 tests)
  - crates/runtime-core/src/event.rs proptest module
  - crates/runtime-core/src/drone.rs proptest module
  - crates/xtask/tests/check_drift.rs (3 tests)

Run: cargo test --workspace
Confirm: tests FAIL because:
  - generated/*.rs doesn't exist yet (round-trip tests)
  - event.rs / drone.rs don't have full enums yet (proptest tests)
  - xtask doesn't have regenerate-types subcommand yet (drift tests)

If any test passes before implementation, the test is wrong — stop and fix it.

═══ STEP 2 — IMPLEMENT ═══

1. Update workspace Cargo.toml [workspace.dependencies]: add serde, serde_json,
   serde_yaml, thiserror, proptest with the versions in B.3.

2. Update crates/runtime-core/Cargo.toml with the deps in B.3 (replacing the
   placeholder Stage A version).

3. Replace crates/runtime-core/src/lib.rs with the version in B.3 (re-exports
   from event, drone, error, generated/*).

4. Create crates/runtime-core/src/error.rs with the RuntimeError enum from B.3.

5. Create crates/runtime-core/src/event.rs with the full AgentEvent enum from
   B.3 (variants spanning §2 + §2a + §2b + §3a + §3b + §4a + §4b + §6a +
   §8.security). Include the proptest module at the bottom.

6. Create crates/runtime-core/src/drone.rs with DroneEvent + DroneCommand +
   supporting enums per spec §1d, exactly as in B.3. Include the proptest
   module at the bottom.

7. Create crates/runtime-core/src/generated/mod.rs from B.3.

8. Update crates/xtask/Cargo.toml with the deps in B.3 (anyhow, clap, serde,
   typify pinned).

9. Replace crates/xtask/src/main.rs with the implementation in B.3 (clap
   subcommand parser + regenerate_types fn + workspace_root helper).

10. Run `cargo run --bin xtask -- regenerate-types` to populate
    crates/runtime-core/src/generated/{common,framework,skill,tool,agent}.rs
    with typify output. Inspect each file:
    - Has the AUTO-GENERATED header per B.3 template
    - Compiles when included in lib.rs
    - Round-trips example artifacts (verified by tests below)

11. Update .github/workflows/ci.yml to add the drift-check step from B.3
    (gated on Cargo.toml + matrix.os == 'ubuntu-latest' && stable).

12. Create crates/runtime-core/README.md with the basic content from B.3.

13. Add CHANGELOG.md [Unreleased] entry from B.4.

═══ STEP 3 — VERIFY ═══
Run each gate; all must pass:
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo build --workspace
  cargo test --workspace               (all 11+ tests pass: 8 round-trip + proptests + 3 xtask)
  cargo test --workspace --doc
  cargo doc --workspace --no-deps -- -D rustdoc::missing_docs
  cargo xtask regenerate-types --check (no drift)
  cargo audit
  cargo deny check
  cargo llvm-cov report --workspace --fail-under-lines 80

If any gate fails, follow CLAUDE.md §7 self-correction loop. Max 3 iterations
then surface.

═══ STEP 4 — RETROSPECTIVE ═══

Per CLAUDE.md §19, copy retrospectives/RETROSPECTIVE-TEMPLATE.md to:
  docs/build-prompts/retrospectives/M01.B-retrospective.md

Fill in [LIVE] sections (typify-related friction is likely; record it),
[END] three-axis scoring, threshold gates, decisions for Stage C.

═══ STEP 5 — SURFACE TO USER ═══

Run: git status, git diff --stat HEAD
Re-run all gates one final time. Capture exact results.

Surface: diff stat, gate results, M01.B retrospective, draft commit message
from B.6.

State: "Stage B is ready. I will NOT commit until you approve."
Do NOT push. Wait for approval. Push waits until Stage D.
```

### B.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(runtime-core,xtask): M01 Stage B — type-generation pipeline + curated event taxonomy

Adds the type-generation pipeline:
- xtask regenerate-types reads schemas/*.v1.json via typify and writes
  to crates/runtime-core/src/generated/{common,framework,skill,tool,agent}.rs
- xtask regenerate-types --check fails CI on drift between schemas and
  committed generated types
- CI step active on ubuntu-latest stable

Adds the hand-curated event taxonomy in runtime-core:
- AgentEvent (full variant list per spec §2 + §2a + §2b + §3a + §3b +
  §4a + §4b + §6a + §8.security; later milestones extend additively)
- DroneEvent + DroneCommand + ActivityState/StopReason/ProcessType/
  AlertLevel/RevertReason per spec §1d
- RuntimeError via thiserror

Tests:
- examples/aria/framework.json and examples/ralph/framework.json round-trip
  through generated Framework type
- skill/agent/tool frontmatter round-trips for representative examples
- proptest round-trips for AgentEvent and DroneEvent variants
- xtask drift-check positive and negative cases

Stage C (drone Phase 1 implementation) starts next on this branch.

Refs: M01-foundation.md §B
Retrospective: docs/build-prompts/retrospectives/M01.B-retrospective.md

https://claude.ai/code/session_<id>
EOF
)"
```

---

## Stage C — Drone Phase 1 Implementation

### C.1 Problem Statement

Implement spec §1 The Drone in `crates/runtime-drone`. This is the survival layer; every later milestone depends on it. **100% line coverage** required per `CLAUDE.md` §5 (drone is a safety primitive).

The drone:
1. Runs as a child process spawned by the (future M02+) main process
2. Maintains a heartbeat to SQLite every 5 seconds
3. Accepts `DroneCommand` messages on its IPC channel and emits `DroneEvent` messages
4. Takes append-only snapshots and stores them with `state_hash` = SHA-256 of `state_json`
5. Survives `SIGTERM`/`SIGINT` (Unix) or `CTRL_BREAK_EVENT` (Windows) with an emergency snapshot before exiting
6. Initializes SQLite with the four required pragmas in the right order: `journal_mode = WAL`, `synchronous = NORMAL`, `busy_timeout = 5000`, `foreign_keys = ON`

CLI: `runtime-drone --session-id <id> --db-path <path> --ipc-socket <path>`.

### C.2 Files to Change

| File | Change |
|---|---|
| `crates/runtime-drone/Cargo.toml` | **Edited** — add deps: tokio, tokio_util, rusqlite, serde, serde_json, sha2, thiserror, tracing, tracing-subscriber, clap |
| `crates/runtime-drone/src/main.rs` | **Edited** — replace placeholder with real binary entry |
| `crates/runtime-drone/src/lib.rs` | **Edited** — library exports for testing |
| `crates/runtime-drone/src/db.rs` | **New** — SQLite open + WAL pragmas + schema init |
| `crates/runtime-drone/src/ipc.rs` | **New** — Unix domain socket / Windows named pipe + framed JSON codec |
| `crates/runtime-drone/src/heartbeat.rs` | **New** — tokio task: 5s interval, emits + writes |
| `crates/runtime-drone/src/snapshot.rs` | **New** — snapshot writer + state_hash |
| `crates/runtime-drone/src/shutdown.rs` | **New** — graceful shutdown + SIGTERM emergency |
| `crates/runtime-drone/src/command_handler.rs` | **New** — dispatches DroneCommand variants |
| `crates/runtime-drone/tests/integration.rs` | **New** — subprocess-spawn end-to-end test |
| `crates/runtime-drone/README.md` | **New** (basic; expanded in Stage D) |
| `Cargo.toml` | **Edited** — add to `[workspace.dependencies]`: tokio, tokio_util, rusqlite, sha2, tracing |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` entry naming Stage C |

### C.3 Detailed Changes

> **Implementer note:** Stage C produces the bulk of M01's code (~1500–2000 lines across 7 modules + integration test). Code patterns, types, and module interfaces are specified below; the exact production code is generated during the TDD cycle (write failing tests first, then implement minimum to pass).

#### `crates/runtime-drone/Cargo.toml` deps

Add to `[dependencies]`: `runtime-core` (path), `tokio` (full features), `tokio-util` (codec features), `rusqlite` (bundled), `serde` (derive), `serde_json`, `sha2`, `thiserror`, `tracing`, `tracing-subscriber` (env-filter, json), `clap` (derive), `futures`, `bytes`, `uuid` (v4).

Add to `[dev-dependencies]`: `proptest`, `tempfile`, `tokio` (test-util), and `nix` (for SIGTERM in integration tests, Unix only).

Update root `Cargo.toml` `[workspace.dependencies]` accordingly.

#### Module interfaces

**`db.rs`** — SQLite setup. Function `pub fn init(path: &Path) -> Result<Connection, DbError>`. Sets the four pragmas in the order spec §1c requires; calls `init_schema(&conn)?` to create all 7 tables (sessions, snapshots, signals, vdr, token_usage, skills, heartbeats). Tests cover pragma values + schema completeness.

**`snapshot.rs`** — Append-only snapshot writer. Function `pub fn write(conn, session_id, reason, state) -> Result<String, SnapshotError>`. Computes `state_hash = SHA-256(state_json)`; inserts a new row (never UPDATE); returns the new snapshot UUID. Tests cover correct row written, state_hash deterministic, append-not-update.

**`heartbeat.rs`** — Tokio task. Function `pub async fn run(conn, event_tx) -> Result<(), HeartbeatError>`. Uses `tokio::time::interval(Duration::from_secs(5))`; on each tick, writes to `heartbeats` table + sends `DroneEvent::Heartbeat` on `event_tx`. Tests use `tokio::time::pause()` (`#[tokio::test(start_paused = true)]`).

**`ipc.rs`** — IPC server. Function `pub async fn serve(socket_path, cmd_tx, event_tx) -> Result<(), IpcError>`. Platform-specific via `#[cfg(unix)]` / `#[cfg(windows)]`. Framed JSON-newline using `tokio_util::codec::Framed` with `LinesCodec`. Each connection spawns a read-half (decode `DroneCommand` → `cmd_tx`) and write-half (encode `DroneEvent` from a per-connection event subscriber). Tests cover encode/decode round-trip, malformed JSON → `Alert`, server keeps running after malformed input.

**`shutdown.rs`** — Signal handler. Function `pub async fn wait_and_handle(conn, session_id, event_tx) -> Result<(), ShutdownError>`. Uses `tokio::signal::unix::SignalKind::terminate()` and `interrupt()` on Unix; `tokio::signal::windows::ctrl_break()` and `ctrl_c()` on Windows. On signal: attempt emergency snapshot (best-effort: log error and exit 0 anyway). Tests are subprocess-based (in `tests/integration.rs`) — can't reliably signal the test binary itself.

**`command_handler.rs`** — Command dispatch. Function `pub async fn run(conn, session_id, cmd_rx, event_tx) -> Result<(), CommandError>`. Loop on `cmd_rx.recv().await`; match on `DroneCommand` variant; call `snapshot::write` for SnapshotNow; signal shutdown coordinator for GracefulShutdown; query+return blob for RevertToSnapshot; emit `Alert` for unknown variants or errors.

#### `crates/runtime-drone/src/lib.rs` (replaces Stage A placeholder)

```rust
//! Drone library — exposes orchestration for binary + tests.

pub mod command_handler;
pub mod db;
pub mod heartbeat;
pub mod ipc;
pub mod shutdown;
pub mod snapshot;

use std::path::PathBuf;
use thiserror::Error;

/// Run the drone main loop until shutdown or fatal error.
pub async fn run(
    session_id: String,
    db_path: PathBuf,
    ipc_socket: PathBuf,
) -> Result<(), DroneError> {
    let conn = db::init(&db_path)?;
    let conn = std::sync::Arc::new(std::sync::Mutex::new(conn));

    let (event_tx, _event_rx) = tokio::sync::broadcast::channel(64);
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(64);

    let hb_handle  = tokio::spawn(heartbeat::run(conn.clone(), event_tx.clone()));
    let ipc_handle = tokio::spawn(ipc::serve(ipc_socket, cmd_tx, event_tx.clone()));
    let ch_handle  = tokio::spawn(command_handler::run(conn.clone(), session_id.clone(), cmd_rx, event_tx.clone()));

    shutdown::wait_and_handle(conn, session_id, event_tx).await?;

    hb_handle.abort();
    ipc_handle.abort();
    ch_handle.abort();
    Ok(())
}

#[derive(Error, Debug)]
pub enum DroneError {
    #[error(transparent)]
    Db(#[from] db::DbError),

    #[error(transparent)]
    Ipc(#[from] ipc::IpcError),

    #[error(transparent)]
    Snapshot(#[from] snapshot::SnapshotError),

    #[error(transparent)]
    Shutdown(#[from] shutdown::ShutdownError),
}
```

#### `crates/runtime-drone/src/main.rs` (replaces Stage A placeholder)

```rust
use clap::Parser;
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(name = "runtime-drone", version)]
struct Args {
    #[arg(long)] session_id: String,
    #[arg(long)] db_path:    PathBuf,
    #[arg(long)] ipc_socket: PathBuf,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    let args = Args::parse();
    info!(session_id = %args.session_id, "drone starting");
    if let Err(e) = runtime_drone::run(args.session_id, args.db_path, args.ipc_socket).await {
        error!(error = %e, "drone exited with error");
        std::process::exit(1);
    }
    info!("drone exited cleanly");
    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};
    let env = EnvFilter::try_from_env("RUNTIME_DRONE_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(env).with_target(false).json().init();
}
```

#### `crates/runtime-drone/README.md` (basic; expanded in Stage D)

Same as the version drafted in C.3 of the prior round (CLI usage, IPC protocol, manual smoke, coverage requirement, license).

#### `CHANGELOG.md` `[Unreleased]` entry

Append under `Added`:

```markdown
- M01 Stage C: Drone Phase 1 implementation per spec §1. Heartbeat (5s interval), snapshot writer with SHA-256 state_hash (append-only), platform-specific IPC (Unix domain socket / Windows named pipe) with framed JSON-newline via tokio_util::codec, SIGTERM/SIGINT/CTRL_BREAK emergency-snapshot handling. SQLite WAL pragmas in correct order. All 7 DroneEvent variants and 6 DroneCommand variants handled. 100% line coverage on runtime-drone.
```

### C.4 Tests

Test plan (write all as failing tests in STEP 1 of the CLI prompt; implement to make them pass in STEP 2):

#### Per-module unit tests

**`db.rs`** (3 tests):
1. `pragmas_set_in_correct_order` — open DB, query `PRAGMA journal_mode/synchronous/busy_timeout/foreign_keys`, assert `WAL`/1/5000/1.
2. `schema_creates_all_tables` — assert `sqlite_master` lists all 7 tables.
3. `init_idempotent` — call `init` twice on same path, no error.

**`snapshot.rs`** (3 tests):
1. `snapshot_writes_correct_row` — write, query, assert all fields match (id, session_id, timestamp, reason, state_json, state_hash).
2. `snapshot_state_hash_is_sha256` — known input has known hash.
3. `snapshot_appends_does_not_update` — two writes produce two rows.

**`heartbeat.rs`** (2 tests, `tokio::time::pause`):
1. `heartbeat_fires_at_5_second_interval` — advance 5s, exactly 1 heartbeat received; advance 10s, exactly 2.
2. `heartbeat_writes_row_to_db` — heartbeats table has row corresponding to emitted event.

**`ipc.rs`** (3 tests):
1. `server_decodes_command` — client writes JSON `DroneCommand::SnapshotNow`, server's `cmd_rx` receives it.
2. `server_encodes_event` — server emits `DroneEvent::Heartbeat`, client reads JSON line.
3. `malformed_json_emits_alert` — client writes invalid JSON, server keeps running, `Alert` emitted.

**`command_handler.rs`** (3 tests):
1. `graceful_shutdown_flushes_within_timeout` — pending writes + `GracefulShutdown { timeout_ms: 1000 }` → exits ≤1s, all writes committed.
2. `revert_to_snapshot_returns_blob` — pre-existing snapshot, `RevertToSnapshot { snapshot_id }` returns its `state_json` via event.
3. `unknown_snapshot_id_emits_alert` — invalid snapshot_id → `DroneEvent::Alert { level: Critical, ... }`.

#### Property tests

Append to `event.rs` and `drone.rs` proptest modules from Stage B:
- `drone_event_codec_round_trip_proptest` — encode through `LinesCodec`, decode, identity.
- `drone_command_codec_round_trip_proptest` — same.

#### Integration test (`tests/integration.rs`)

`drone_lifecycle_end_to_end` — spawn drone subprocess, wait for socket, send SIGTERM (Unix) or skip (Windows v0.1), assert clean exit and emergency snapshot row in SQLite.

#### Coverage

- `runtime-drone`: **100% line coverage** (`cargo llvm-cov report --package runtime-drone --fail-under-lines 100`)
- Workspace: ≥80%

### C.5 CLI Prompt

```
Read CLAUDE.md for all project rules — especially §5 (TDD discipline),
§7 (self-correction loop), §15 gotcha #11–#15 (drone-specific traps).

Read docs/build-prompts/M01-foundation.md Stage C (sections C.1 through C.4).
Read agent-runtime-spec.md §1 (Drone), §1c (Multi-session), §1d (IPC).

Stages A and B must already be committed on this feature branch.
Run: git log --oneline -2  → previous two commits should be Stage A and Stage B.

═══ STEP 1 — WRITE FAILING TESTS ═══

Create test bodies in each module's #[cfg(test)] mod tests block per C.4
(11 unit tests across db.rs/snapshot.rs/heartbeat.rs/ipc.rs/command_handler.rs).
Create crates/runtime-drone/tests/integration.rs with the lifecycle test.
Create proptest blocks in event.rs and drone.rs (extending Stage B's modules).

The implementations don't exist yet. Run:
  cargo test --workspace
Expected: failures because production code doesn't compile / functions don't
exist. That's the correct red state for Stage C.

If any test passes before implementation, the test is wrong — stop and fix it.

═══ STEP 2 — IMPLEMENT ═══

Implement modules in this order (lowest to highest dependency):

1. crates/runtime-drone/Cargo.toml — add deps from C.3 (tokio, tokio_util,
   rusqlite, sha2, uuid, etc.). Update workspace [workspace.dependencies].

2. crates/runtime-drone/src/db.rs — full implementation (init fn, init_schema,
   pragmas in correct order). Tests pass: db::tests.

3. crates/runtime-drone/src/snapshot.rs — full implementation (write fn with
   state_hash via sha2). Tests pass: snapshot::tests.

4. crates/runtime-drone/src/heartbeat.rs — tokio task with 5s interval. Tests
   use tokio::time::pause(). Tests pass: heartbeat::tests.

5. crates/runtime-drone/src/ipc.rs — platform-specific server. Use
   #[cfg(unix)] for UnixListener and #[cfg(windows)] for NamedPipeServer.
   tokio_util LinesCodec for framing. Tests pass: ipc::tests.

6. crates/runtime-drone/src/shutdown.rs — signal handler with emergency
   snapshot. Subprocess-spawn integration test exercises this in tests/.

7. crates/runtime-drone/src/command_handler.rs — DroneCommand dispatch loop.
   Tests pass: command_handler::tests.

8. crates/runtime-drone/src/lib.rs — replace Stage A placeholder with the
   run() fn from C.3 (orchestrates heartbeat + ipc + command_handler +
   shutdown via tokio::spawn).

9. crates/runtime-drone/src/main.rs — replace Stage A placeholder with binary
   entry (clap CLI, tracing init, call lib::run).

10. crates/runtime-drone/tests/integration.rs — subprocess-spawn lifecycle
    test from C.4. Use tempfile for socket path; nix for SIGTERM (Unix-only;
    skip on Windows for v0.1).

11. crates/runtime-drone/README.md — basic content from C.3.

12. CHANGELOG.md [Unreleased] entry from C.3.

═══ STEP 3 — VERIFY ═══

Run each gate; all must pass:
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo build --workspace
  cargo test --workspace
  cargo test --workspace --features integration
  cargo test --workspace --doc
  cargo doc --workspace --no-deps -- -D rustdoc::missing_docs
  cargo audit
  cargo deny check
  cargo llvm-cov report --package runtime-drone --fail-under-lines 100
  cargo llvm-cov report --workspace --fail-under-lines 80

Manual smoke (Linux/macOS):
  mkdir -p /tmp/drone-smoke
  cargo run --bin runtime-drone -- \
    --session-id smoke --db-path /tmp/drone-smoke/d.sqlite --ipc-socket /tmp/drone-smoke/d.sock &
  DRONE_PID=$!; sleep 6; kill -TERM $DRONE_PID; wait $DRONE_PID
  sqlite3 /tmp/drone-smoke/d.sqlite "SELECT reason FROM snapshots;"
  # Expected: includes 'sigterm' or 'emergency'

If any gate fails, follow CLAUDE.md §7 self-correction. Max 3 iterations
then surface.

═══ STEP 4 — RETROSPECTIVE ═══

Per CLAUDE.md §19, copy retrospectives/RETROSPECTIVE-TEMPLATE.md to:
  docs/build-prompts/retrospectives/M01.C-retrospective.md

Fill in [LIVE] (drone implementation has many edges — record WAL pragma
issues, IPC platform quirks, signal handling), [END] scoring, threshold
gates, decisions for Stage D.

100% drone coverage often requires 1–2 self-correction rounds; document
each holdout in the retrospective.

═══ STEP 5 — SURFACE TO USER ═══

Run: git status, git diff --stat HEAD, full gate verification.
Surface: diff stat, gate results, M01.C retrospective, draft commit from C.6.
State: "Stage C is ready. I will NOT commit until you approve."
Do NOT push. Push waits until Stage D is approved.
```

### C.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(runtime-drone): M01 Stage C — Phase 1 drone (heartbeat + snapshot + IPC + SIGTERM)

Implements the survival layer per spec §1:
- Heartbeat task (5s interval) — DroneEvent::Heartbeat + heartbeats table
- Append-only snapshot writer with SHA-256 state_hash
- SQLite WAL pragmas in correct order (journal_mode, synchronous,
  busy_timeout, foreign_keys)
- Platform-specific IPC: Unix domain socket / Windows named pipe;
  framed JSON-newline via tokio_util::codec
- SIGTERM/SIGINT/CTRL_BREAK handler: best-effort emergency snapshot
  before clean exit
- All 6 DroneCommand variants + all 7 DroneEvent variants

Coverage: 100% line on runtime-drone (safety primitive per CLAUDE.md §5);
≥80% workspace.

Stage D (fuzz harness + cross-OS verification + READMEs polish) starts next.

Refs: M01-foundation.md §C, agent-runtime-spec.md §1 §1c §1d
Retrospective: docs/build-prompts/retrospectives/M01.C-retrospective.md

https://claude.ai/code/session_<id>
EOF
)"
```

---

## Stage D — Fuzz, Coverage, Polish

### D.1 Problem Statement

Close M01 cleanly. Stage A landed the workspace; Stage B added types; Stage C produced the working drone with 100% coverage. Stage D adds the fuzz harness for the IPC frame decoder, sets up the nightly fuzz workflow, polishes per-crate READMEs, organizes the CHANGELOG, and verifies CI green across Linux/macOS/Windows × stable + MSRV.

After Stage D's PR merges, M01 is complete and M02 can start.

### D.2 Files to Change

| File | Change |
|---|---|
| `crates/runtime-drone/fuzz/Cargo.toml` | **New** (separate package, not a workspace member) |
| `crates/runtime-drone/fuzz/fuzz_targets/drone_command_decode.rs` | **New** — fuzz target for IPC frame decoder |
| `crates/runtime-drone/fuzz/corpus/drone_command_decode/` | **New** — seed corpus (one valid `DroneCommand` JSON per variant) |
| `.github/workflows/ci.yml` | **Edited** — add `fuzz-smoke` job (30s on PR) |
| `.github/workflows/fuzz-nightly.yml` | **New** — scheduled 1-hour fuzz on main |
| `crates/runtime-core/README.md` | **Edited** — expanded from Stage B basic |
| `crates/runtime-drone/README.md` | **Edited** — expanded from Stage C basic |
| `crates/xtask/README.md` | **New** — subcommand reference |
| `CHANGELOG.md` | **Edited** — re-organized `[Unreleased]` into Keep-a-Changelog subsections |
| `docs/MVP-v0.1.md` | **Edited** — mark M1 acceptance criteria complete |

### D.3 Detailed Changes

#### `crates/runtime-drone/fuzz/Cargo.toml`

```toml
[package]
name        = "runtime-drone-fuzz"
version     = "0.0.0"
publish     = false
edition     = "2021"

[package.metadata]
cargo-fuzz = true

[dependencies]
libfuzzer-sys = "0.4"
serde_json    = "1"
runtime-core  = { path = "../../runtime-core" }

[[bin]]
name = "drone_command_decode"
path = "fuzz_targets/drone_command_decode.rs"
test = false
doc  = false

[profile.release]
debug = 1
```

> Not a workspace member — cargo-fuzz convention is a sibling package.

#### `crates/runtime-drone/fuzz/fuzz_targets/drone_command_decode.rs`

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use runtime_core::DroneCommand;

// Fuzz the IPC frame decoder with arbitrary bytes.
//
// Invariants:
//   1. Must not panic on any input.
//   2. If the input deserializes to a DroneCommand, the variant must be
//      one of the spec §1d variants — serde + tagged enum enforces this.
//   3. Untrusted bytes through this path must not bypass validation.
//
// Run for 30s in CI on PRs; 1 hour nightly on main.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Mimics the LinesCodec path: each newline-delimited frame is parsed.
        for line in s.lines() {
            let _: Result<DroneCommand, _> = serde_json::from_str(line);
        }
    }
});
```

#### `crates/runtime-drone/fuzz/corpus/drone_command_decode/`

Seed files, one per `DroneCommand` variant, named `seed_<variant>.json`. Examples:

```
seed_snapshot_now.json:        {"type":"snapshot_now","reason":"task_boundary","state_json":{}}
seed_graceful_shutdown.json:   {"type":"graceful_shutdown","timeout_ms":5000}
seed_revert_to_snapshot.json:  {"type":"revert_to_snapshot","snapshot_id":"abc","reason":"hook_rollback"}
seed_set_activity_timeout.json:{"type":"set_activity_timeout","ms":30000}
seed_spawn_process.json:       {"type":"spawn_process","process_type":"agent","config":{"command":"sh","args":[],"env":{}}}
seed_stop_process.json:        {"type":"stop_process","pid":1234,"force":false}
```

Without seeds, fuzzing starts cold and finds shallow bugs slowly. Six seeds = one per variant.

#### `.github/workflows/ci.yml` — add `fuzz-smoke` job

```yaml
  fuzz-smoke:
    name: Fuzz smoke (30s)
    needs: detect-cargo
    if: needs.detect-cargo.outputs.has_cargo == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - uses: Swatinem/rust-cache@v2
      - run: cargo install cargo-fuzz --locked
      - run: cargo +nightly fuzz run drone_command_decode -- -max_total_time=30
        working-directory: crates/runtime-drone
```

#### `.github/workflows/fuzz-nightly.yml` (new)

```yaml
name: Fuzz Nightly
on:
  schedule:
    - cron: '0 4 * * *'  # 04:00 UTC daily
  workflow_dispatch:
permissions:
  contents: read

jobs:
  fuzz:
    runs-on: ubuntu-latest
    timeout-minutes: 70
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - uses: Swatinem/rust-cache@v2
      - run: cargo install cargo-fuzz --locked
      - name: Fuzz drone_command_decode for 1 hour
        run: cargo +nightly fuzz run drone_command_decode -- -max_total_time=3600
        working-directory: crates/runtime-drone
      - name: Upload corpus on failure
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: fuzz-corpus
          path: crates/runtime-drone/fuzz/corpus
```

#### Per-crate README expansions

**`crates/runtime-core/README.md`** (~80 lines): expand on Stage B basic. Sections: Overview, Type Generation Pipeline, Hand-Curated Types, Cross-References (to spec sections), License.

**`crates/runtime-drone/README.md`** (~120 lines): expand on Stage C basic. Sections: CLI, IPC Protocol (with example commands and events), SQLite Schema, Manual Smoke Test, Platform-Specific Notes (Unix socket vs Windows named pipe), Coverage Requirement, License.

**`crates/xtask/README.md`** (~50 lines): subcommand reference. Currently only `regenerate-types`; document inputs, outputs, drift behavior, when to run.

#### `CHANGELOG.md` reorganized

Replace the four ad-hoc M01.A/B/C/D entries from prior stages with one consolidated `[Unreleased]` section organized per Keep-a-Changelog:

```markdown
## [Unreleased]

### Added
- M01 Foundation milestone: Cargo workspace with five member crates
  (runtime-core, runtime-main, runtime-drone, runtime-sandbox, xtask) +
  Tauri stub (src-tauri/) + workspace lints (deny warnings, forbid unsafe
  except sandbox, clippy pedantic + nursery) + cargo-deny policy.
- Type-generation pipeline: xtask `regenerate-types` reads schemas/*.v1.json
  via typify and writes to crates/runtime-core/src/generated/. CI drift check
  active.
- Hand-curated event taxonomy in runtime-core: AgentEvent (full variant list
  per spec §2 + §2a + §2b + §3a + §3b + §4a + §4b + §6a + §8.security),
  DroneEvent + DroneCommand per spec §1d, RuntimeError via thiserror.
- Drone Phase 1 (`runtime-drone`): heartbeat + append-only snapshot writer
  with SHA-256 state_hash + platform-specific IPC (Unix socket / Windows
  named pipe) with framed JSON-newline + SIGTERM/SIGINT/CTRL_BREAK
  emergency-snapshot handling. SQLite WAL pragmas in correct order.
- 100% line coverage on runtime-drone; ≥80% workspace coverage.
- Fuzz harness for IPC frame decoder (`drone_command_decode`); CI runs
  30s on PRs; nightly workflow runs 1 hour on main.
- Per-crate READMEs documenting the public API surface.

### Tests
- Schema round-trip tests: examples/aria/framework.json,
  examples/ralph/framework.json, and 19 skill/agent/tool frontmatter files
  all round-trip through generated runtime-core types.
- Property tests for AgentEvent, DroneEvent, DroneCommand JSON round-trips.
- xtask drift-check positive and negative cases.
- Drone unit tests: WAL pragmas, schema, snapshot append-only, heartbeat
  interval, IPC encode/decode, command dispatch, malformed input → Alert.
- Subprocess-spawn integration test: drone responds to SIGTERM with
  emergency snapshot.

### Documentation
- Per-crate READMEs (runtime-core, runtime-drone, xtask).
- M01 Foundation specification + stage prompts at
  docs/build-prompts/M01-foundation.md.
- Per-stage retrospectives at
  docs/build-prompts/retrospectives/M01.{A,B,C,D}-retrospective.md
  + parent-milestone summary at M01-summary.md (per CLAUDE.md §19).

### Status
M01 Foundation milestone complete. M02 (event pipeline + AnthropicProvider
+ Tauri shell + AgentEvent flow) is the next milestone.
```

#### `docs/MVP-v0.1.md` §M1 — mark acceptance criteria complete

Convert each `- [ ]` line in the M1 section to `- [x]`. Add a closing line at end of M1 section: "**Status: Complete** (PR #N merged YYYY-MM-DD)."

### D.4 Tests

#### Fuzz harness validation

1. **`fuzz_target_compiles`** — `cargo +nightly fuzz build` succeeds. CI runs on every PR.
2. **`fuzz_target_runs_30s_no_panic`** — `cargo +nightly fuzz run drone_command_decode -- -max_total_time=30` exits 0 with no panics. CI runs on every PR.
3. **`fuzz_corpus_seeded`** — initial corpus contains at least 6 valid seeds (one per `DroneCommand` variant) at `crates/runtime-drone/fuzz/corpus/drone_command_decode/`.

These three are validated at gate-verification time in STEP 3 of the CLI prompt; no separate test file needed because `cargo fuzz build/run` are themselves the assertions.

#### M01 acceptance criteria final pass

Walk through `docs/MVP-v0.1.md` §M1 acceptance criteria. Each `- [ ]` should be verifiable by a specific gate; flip to `- [x]` only when verified.

### D.5 CLI Prompt

```
Read CLAUDE.md for all project rules.
Read docs/build-prompts/M01-foundation.md Stage D (sections D.1 through D.4).
Stages A, B, C must already be committed on this feature branch.
Run: git log --oneline -3  → previous three commits should be Stage A, B, C.

═══ STEP 1 — WRITE FAILING/MISSING ARTIFACTS ═══

The fuzz harness package, nightly workflow, and per-crate READMEs don't
exist yet. There's no "test" to write that fails before they exist —
their absence IS the failing state.

Run `cargo +nightly fuzz build` and confirm it fails with "no fuzz package
configured" (or similar). That's the right starting state.

═══ STEP 2 — IMPLEMENT ═══

1. Create crates/runtime-drone/fuzz/Cargo.toml from D.3 (NOT a workspace
   member — cargo-fuzz convention is sibling package).

2. Create crates/runtime-drone/fuzz/fuzz_targets/drone_command_decode.rs
   from D.3.

3. Create crates/runtime-drone/fuzz/corpus/drone_command_decode/ and seed
   it with 6 files (one per DroneCommand variant) per D.3.

4. Update .github/workflows/ci.yml: add the `fuzz-smoke` job from D.3
   (gated on detect-cargo.has_cargo == 'true').

5. Create .github/workflows/fuzz-nightly.yml from D.3 (scheduled 04:00 UTC
   daily; 70-minute timeout; uploads corpus on failure).

6. Expand crates/runtime-core/README.md to ~80 lines per D.3 (Overview,
   Type Generation, Hand-Curated Types, Cross-References, License).

7. Expand crates/runtime-drone/README.md to ~120 lines per D.3 (CLI, IPC
   Protocol with examples, SQLite Schema, Manual Smoke, Platform-Specific
   Notes, Coverage Requirement, License).

8. Create crates/xtask/README.md (~50 lines) per D.3 (subcommand reference).

9. Reorganize CHANGELOG.md [Unreleased] section per D.3 (Keep-a-Changelog
   subsections: Added / Tests / Documentation / Status).

10. In docs/MVP-v0.1.md §M1, flip all `- [ ]` acceptance criteria to `- [x]`
    after verifying each gate. Add closing line: "**Status: Complete**
    (M01 PR merged YYYY-MM-DD)" — leave date as YYYY-MM-DD; user fills
    in at merge time.

═══ STEP 3 — VERIFY ═══

Run each gate; all must pass:
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo build --workspace
  cargo test --workspace
  cargo test --workspace --features integration
  cargo test --workspace --doc
  cargo doc --workspace --no-deps -- -D rustdoc::missing_docs
  cargo audit
  cargo deny check
  cargo llvm-cov report --package runtime-drone --fail-under-lines 100
  cargo llvm-cov report --workspace --fail-under-lines 80

Fuzz validation:
  cd crates/runtime-drone
  cargo +nightly fuzz build                                    (compiles)
  cargo +nightly fuzz run drone_command_decode -- -max_total_time=30
                                                                (no panic in 30s)
  ls fuzz/corpus/drone_command_decode/ | wc -l                  (≥6 seed files)

CI inspection — check the GitHub Actions run after pushing this stage's
commit; confirm Linux/macOS/Windows × stable + MSRV all green; confirm
fuzz-smoke step passes; confirm fuzz-nightly workflow appears in the
Actions tab (it won't run until 04:00 UTC).

═══ STEP 4 — RETROSPECTIVE + SUMMARY ═══

Per CLAUDE.md §19:

1. Copy retrospectives/RETROSPECTIVE-TEMPLATE.md to:
     docs/build-prompts/retrospectives/M01.D-retrospective.md
   Fill in [LIVE], [END] scoring, threshold gates, decisions for M02.

2. Copy retrospectives/SUMMARY-TEMPLATE.md to:
     docs/build-prompts/retrospectives/M01-summary.md
   Aggregate findings across M01.A, M01.B, M01.C, M01.D retrospectives.
   Score means, time-box accuracy, friction patterns that recurred,
   hard-gate violations (none expected; if any, this milestone wouldn't
   be ready to merge).

═══ STEP 5 — SURFACE M01 PR DRAFT ═══

This is the moment the M01 PR draft surfaces.

Run: git status, git diff --stat HEAD
Re-run all gates one final time. Capture exact results.

Draft the PR description per .github/PULL_REQUEST_TEMPLATE.md. Include
in the PR description:
  - Summary of M01 (what shipped across all 4 stages)
  - All 4 stage commits in the commit list
  - Quality gate results (each gate, pass/fail with key numbers)
  - Coverage numbers (drone 100%, workspace ≥80%)
  - Cross-OS CI status (link to CI run)
  - Links to all 4 stage retrospectives + the M01-summary.md
  - AI-assistance disclosure
  - DCO sign-off plan

Surface to user:
  - Diff stat
  - All 4 stage retrospectives
  - M01-summary.md
  - PR description
  - Draft commit message from D.6

State explicitly: "M01 is ready. I will NOT commit Stage D, push, or open
the PR until you approve. Please review:
  1. The diff stat
  2. M01.D-retrospective.md
  3. M01-summary.md
  4. The PR description
The other 3 stages (A, B, C) are already committed on this branch from
prior approval rounds."

Wait for explicit approval. On approval:
  1. Commit Stage D with the message from D.6
  2. Push the branch
  3. Open the PR (only if user explicitly asked for one in their approval)
```

### D.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(workspace): M01 Stage D — fuzz harness + per-crate READMEs + CHANGELOG + M1 closeout

Closes M01 Foundation:
- cargo-fuzz harness for drone_command_decode (IPC frame decoder)
- CI fuzz-smoke job (30s on every PR)
- Nightly fuzz workflow (1h scheduled at 04:00 UTC) — uploads corpus on failure
- Per-crate READMEs expanded:
  - runtime-core: type-generation pipeline + hand-curated types
  - runtime-drone: CLI + IPC protocol + SQLite schema + manual smoke +
    platform notes + coverage requirement
  - xtask: subcommand reference
- CHANGELOG.md [Unreleased] reorganized per Keep-a-Changelog
- docs/MVP-v0.1.md §M1 acceptance criteria fully checked

M01 implementation work complete. Stage E (Phase Closeout: Gap Analysis,
per CLAUDE.md §20) is the final commit on this branch before the PR is
drafted. After PR merges, M02 (event pipeline + AnthropicProvider + Tauri
shell) starts on a fresh branch.

Per-stage retrospectives:
- docs/build-prompts/retrospectives/M01.A-retrospective.md
- docs/build-prompts/retrospectives/M01.B-retrospective.md
- docs/build-prompts/retrospectives/M01.C-retrospective.md
- docs/build-prompts/retrospectives/M01.D-retrospective.md
- docs/build-prompts/retrospectives/M01-summary.md

Refs: M01-foundation.md §D, agent-runtime-spec.md §1 §1c §1d §12

https://claude.ai/code/session_<id>
EOF
)"
```

---

## Stage E — Phase Closeout: Gap Analysis

> **Per CLAUDE.md §20.** Final stage of M01. Runs after Stage D commits and the parent-milestone summary lands. Produces one new entry in `docs/gap-analysis.md`. **Append-only** — no prior entry may be edited. The gap analysis commit is the final commit on the parent-milestone branch — it gates the PR push.

### E.1 Problem Statement

Generate the M01 entry in `docs/gap-analysis.md` per the entry template at the top of that file. Cumulative review of code-vs-spec across all M01 work to date. Six sections, all required: codebase deep dive, adherence to spec, spec review (forward-looking), fix backlog (🔴/🟡/🟢), carry-forward (N/A — first milestone), sign-off.

This is the first gap analysis entry. M01 sets the precedent for honesty, specificity, and prioritization. If everything is rated "Important," the prioritization is meaningless.

### E.2 Files to Change

| File | Change |
|---|---|
| `docs/gap-analysis.md` | **Edited (append-only)** — new section appended at the bottom; placeholder line `*No entries yet. M01 will be the first.*` removed and replaced with the M01 entry |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` notes that the M01 gap analysis entry was added |

### E.3 Detailed Changes

The entry follows the six-section template defined at the top of `docs/gap-analysis.md`. Do NOT diverge from the template. Do NOT skip a section — write **"None observed."** if a section truly has nothing to report.

**Process:**

1. Re-read `agent-runtime-spec.md` end-to-end (skim with focus on §1, §1b, §1c, §1d, §2 + §11 — the sections M01 actually touched).
2. Read every file produced or edited across M01 stages A → D (use `git log --oneline main..HEAD` to enumerate commits, then `git show --stat` per commit).
3. Read M01.A / M01.B / M01.C / M01.D retrospectives + M01-summary.
4. Draft the entry against the template.
5. Run the append-only check locally (see E.4 Tests).
6. Surface for user review per E.5.

**Header for the new entry:**

```markdown
## M01 — Foundation (<YYYY-MM-DD>, commit `<sha-of-stage-D-commit>`)

> Author: Claude (per `CLAUDE.md` §20)
> Stages aggregated: M01.A (workspace), M01.B (types), M01.C (drone), M01.D (fuzz + polish)
> Reviewed against: agent-runtime-spec.md §1 §1b §1c §1d §2 §11 §12, schemas/*.v1.json, M01.A–D retrospectives + M01-summary.
```

The placeholder line `*No entries yet. M01 will be the first.*` at the bottom of `docs/gap-analysis.md` is removed and replaced with the M01 entry. This is the only "edit" to existing content allowed — it's a placeholder removal, not a finding revision.

**`CHANGELOG.md` `[Unreleased]` addition:**

```markdown
- M01 Phase Closeout: cumulative gap analysis appended to docs/gap-analysis.md
  per CLAUDE.md §20 (append-only living document). Gates the M01 PR.
```

### E.4 Tests

No new code tests. Verification is the **append-only check** plus user review of the entry's substance.

#### Append-only check (run locally before surfacing)

```bash
# Compare gap-analysis.md against the PR base (origin/main).
# Prior content must be a literal prefix of the new file.
git fetch origin main
git show origin/main:docs/gap-analysis.md > /tmp/gap-base.md 2>/dev/null
if [ -s /tmp/gap-base.md ]; then
  base_lines=$(wc -l < /tmp/gap-base.md)
  if ! diff -q /tmp/gap-base.md <(head -n "$base_lines" docs/gap-analysis.md) > /dev/null; then
    echo "FAIL: prior content was modified."
    diff /tmp/gap-base.md <(head -n "$base_lines" docs/gap-analysis.md)
    exit 1
  fi
  echo "OK: append-only invariant holds."
else
  echo "Note: gap-analysis.md is new on this branch (first entry); skipping append-only check."
fi
```

#### CI append-only gate (added to `.github/workflows/ci.yml` in this stage)

```yaml
  gap-analysis-append-only:
    name: gap-analysis.md append-only check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Verify prior content is unchanged
        run: |
          set -euo pipefail
          BASE_REF="${{ github.event.pull_request.base.ref || 'main' }}"
          git fetch origin "$BASE_REF":"$BASE_REF" 2>/dev/null || git fetch origin "$BASE_REF"
          if git show "$BASE_REF":docs/gap-analysis.md > /tmp/gap-base.md 2>/dev/null; then
            base_lines=$(wc -l < /tmp/gap-base.md)
            if [ "$base_lines" -gt 0 ]; then
              if ! diff -q /tmp/gap-base.md <(head -n "$base_lines" docs/gap-analysis.md) > /dev/null; then
                echo "::error::docs/gap-analysis.md is append-only per CLAUDE.md §20."
                echo "Prior content was modified or deleted. Diff:"
                diff /tmp/gap-base.md <(head -n "$base_lines" docs/gap-analysis.md) || true
                exit 1
              fi
            fi
          fi
          echo "Append-only invariant holds."
```

This job is added to the same `ci.yml` Stage D edits (the fuzz-smoke job's neighbor). It runs on every PR regardless of which files changed; it's cheap (no toolchain install).

#### Coverage target

N/A — documentation stage; no Rust code changes.

### E.5 CLI Prompt

```
Read CLAUDE.md (focus: §10 Don't-touch zones, §17 Reference Index, §20 Gap
Analysis Protocol).
Read docs/gap-analysis.md header in full (entry template + append-only rule).
Read docs/build-prompts/M01-foundation.md Stage E sections E.1 through E.4.

═══ STEP 1 — INGEST ═══

Read in this order:

  1. agent-runtime-spec.md — full skim, focused read on §1, §1b, §1c, §1d,
     §2, §11, §12 (the sections M01 touched)
  2. Every file changed in M01 — enumerate commits:
       git log --oneline main..HEAD
     For each commit:
       git show --stat <sha>
  3. Per-stage retrospectives:
       docs/build-prompts/retrospectives/M01.A-retrospective.md
       docs/build-prompts/retrospectives/M01.B-retrospective.md
       docs/build-prompts/retrospectives/M01.C-retrospective.md
       docs/build-prompts/retrospectives/M01.D-retrospective.md
       docs/build-prompts/retrospectives/M01-summary.md
  4. schemas/*.v1.json — confirm what's been generated vs hand-curated
  5. docs/gap-analysis.md — header + entry template (currently no prior
     entries; M01 is the first)

═══ STEP 2 — DRAFT THE ENTRY ═══

Append a new section to docs/gap-analysis.md immediately above the
placeholder line:

  *No entries yet. M01 will be the first.*

Remove that placeholder line as part of the same edit. Use the exact six-
section template from the file's header. Header:

  ## M01 — Foundation (<YYYY-MM-DD>, commit `<sha-of-stage-D-commit>`)

  > Author: Claude (per `CLAUDE.md` §20)
  > Stages aggregated: M01.A, M01.B, M01.C, M01.D
  > Reviewed against: agent-runtime-spec.md §1 §1b §1c §1d §2 §11 §12,
  > schemas/*.v1.json, M01.A–D retrospectives + M01-summary.

Then the six required sections. Specifics:

- **Codebase deep dive** (200–500 words): cumulative narrative. What's
  solid, what's structurally weak, what surprised. Specific files.
- **Adherence to spec**: ✅ / ⚠️ / ❌ with file:line on BOTH sides
  (spec section + crate/file.rs:line).
- **Spec review (forward-looking)**: missing items, contradictions,
  ambiguity, open questions, recommended spec changes — each with
  file:line where the gap surfaces. Re-read prior sections with fresh
  eyes; surface things the M01 audit caught even if they don't bite
  M01 directly.
- **Fix backlog**: 🔴 Critical / 🟡 Important / 🟢 Nice-to-have. Be
  honest. If you're tempted to mark everything Important, re-prioritize.
- **Carry-forward**: write "N/A — first milestone." (M01 is the first
  entry; nothing to carry forward yet.)
- **Sign-off**: timestamp + Claude attestation per template.

═══ STEP 3 — ADD CI APPEND-ONLY GATE ═══

Edit .github/workflows/ci.yml: add the gap-analysis-append-only job per
E.4. Place it after the existing fuzz-smoke job. No new toolchain install
required.

═══ STEP 4 — VERIFY APPEND-ONLY (LOCAL) ═══

Note: on M01 the file is NEW on this branch (no entry on main yet), so
the local check will print "skipping append-only check" — that's expected.
For M02+ it will be a hard check.

Run:
  git fetch origin main
  git show origin/main:docs/gap-analysis.md > /tmp/gap-base.md 2>/dev/null
  if [ -s /tmp/gap-base.md ]; then
    base_lines=$(wc -l < /tmp/gap-base.md)
    diff -q /tmp/gap-base.md <(head -n "$base_lines" docs/gap-analysis.md) \
      && echo "OK: append-only holds." \
      || (echo "FAIL"; diff /tmp/gap-base.md <(head -n "$base_lines" docs/gap-analysis.md); exit 1)
  else
    echo "First entry on this branch — append-only check skipped."
  fi

═══ STEP 5 — UPDATE CHANGELOG ═══

Append to CHANGELOG.md [Unreleased] / Added per E.3.

═══ STEP 6 — SURFACE TO USER ═══

Run: git status, git diff docs/gap-analysis.md
Print the full new entry text (so the user can read it without scrolling
git diff output).

State explicitly:

  "M01 Stage E (Gap Analysis) is ready. I will NOT commit until you
  approve. Per CLAUDE.md §20, once committed this entry is IMMUTABLE —
  future milestones may only update its status via their Carry-forward
  sections. Please review the entry's substance carefully before
  approving."

Wait for explicit approval. After approval:
  1. Commit per E.6.
  2. Push the parent-milestone branch (origin/claude/m01-foundation),
     which now contains stage A/B/C/D commits + this Stage E commit.
  3. Draft the M01 PR description per CLAUDE.md §8 (do NOT open the PR
     unless explicitly asked — surface the description for user review).
```

### E.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
docs(gap-analysis): M01 — append cumulative product+spec audit

Per CLAUDE.md §20. First entry in docs/gap-analysis.md. Reviews codebase
shipped across M01.A–D against agent-runtime-spec.md; records adherence
findings, spec gaps, and prioritized fix backlog (🔴 Critical / 🟡
Important / 🟢 Nice-to-have).

Also adds the CI gap-analysis-append-only job that enforces the
immutability of prior entries on every PR.

This entry is immutable. Future milestones report status updates via
their Carry-forward sections; M01's findings stay as written.

Refs: M01-foundation.md §E, CLAUDE.md §20

https://claude.ai/code/session_<id>
EOF
)"
```

---

## Summary Table

| Stage | New Files | Edited Files | Tests Added | Effort |
|---|---|---|---|---|
| **A** Workspace skeleton | 5 crate Cargo.tomls + 5 lib.rs + 2 main.rs + Cargo.toml + rust-toolchain.toml + deny.toml + 4 src-tauri files | `CHANGELOG.md` | 5 trivial (one per crate) | ~5–8h |
| **B** Type generation | xtask main + 5 generated/*.rs + event.rs + drone.rs + error.rs + runtime-core README + xtask Cargo.toml + tests/round_trip.rs | `Cargo.toml`, `runtime-core/Cargo.toml`, `runtime-core/lib.rs`, `xtask/Cargo.toml`, `.github/workflows/ci.yml`, `CHANGELOG.md` | 11+ (8 round-trip + proptests + 3 xtask drift) | ~6–10h |
| **C** Drone Phase 1 | 7 drone modules (db/snapshot/heartbeat/ipc/shutdown/command_handler/lib.rs replacement) + integration.rs + drone README | `Cargo.toml`, `runtime-drone/Cargo.toml`, `runtime-drone/main.rs`, `CHANGELOG.md` | 14+ unit + 2 proptest + 1 integration | ~12–18h |
| **D** Fuzz + polish | Fuzz package + fuzz target + 6 corpus seeds + nightly workflow + xtask README | `runtime-core/README.md`, `runtime-drone/README.md`, `.github/workflows/ci.yml`, `CHANGELOG.md`, `MVP-v0.1.md` | Fuzz harness (3 implicit checks at gate time) | ~4–6h |
| **E** Phase Closeout: Gap Analysis | — | `docs/gap-analysis.md` (append M01 entry + remove placeholder), `.github/workflows/ci.yml` (add append-only gate), `CHANGELOG.md` | CI append-only gate (1 new job) | ~2–4h |
| **Total** | **~45 new files** | **~16 edited files** | **30+ tests + fuzz harness + append-only gate** | **~29–46h** |

---

## Verification Checklist

Before approving the M01 PR (Stage D's surface), verify:

### Automated (gates)

- [ ] `cargo fmt --all -- --check` — zero diff
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings
- [ ] `cargo build --workspace` — succeeds on Linux/macOS/Windows × stable + MSRV (per CI matrix)
- [ ] `cargo build --workspace --release` — same
- [ ] `cargo test --workspace` — all unit + property tests pass
- [ ] `cargo test --workspace --features integration` — drone subprocess test passes (Linux/macOS)
- [ ] `cargo test --workspace --doc` — all doc-test examples compile and pass
- [ ] `cargo doc --workspace --no-deps -- -D rustdoc::missing_docs -D rustdoc::broken_intra_doc_links` — clean
- [ ] `cargo audit` — zero high/critical
- [ ] `cargo deny check` — passing
- [ ] `cargo xtask regenerate-types --check` — no drift
- [ ] `cargo llvm-cov report --package runtime-drone --fail-under-lines 100` — drone at 100%
- [ ] `cargo llvm-cov report --workspace --fail-under-lines 80` — workspace ≥80%
- [ ] `cargo +nightly fuzz run drone_command_decode -- -max_total_time=30` — no panic
- [ ] `gap-analysis-append-only` CI job — passes (Stage E adds; first run on Stage E's commit)
- [ ] CI green on all OS × toolchain cells (visually inspect after push)

### Manual

- [ ] Manual drone smoke (Linux/macOS): drone starts, heartbeat fires, SIGTERM produces emergency snapshot row in SQLite
- [ ] Tauri stub builds: `cargo tauri build --no-bundle` succeeds (full bundle deferred to M11)
- [ ] All 4 stage retrospectives present and filled in (M01.A–D)
- [ ] `M01-summary.md` aggregates across the 4 work stages with verdict
- [ ] **`docs/gap-analysis.md` M01 entry appended (Stage E)** — six sections complete, none omitted; severity levels honestly applied
- [ ] M01 PR description references all 5 stage commits (A/B/C/D + E) + retrospectives + gap-analysis entry
- [ ] CHANGELOG `[Unreleased]` reflects what M01 actually delivered
- [ ] `docs/MVP-v0.1.md` §M1 acceptance criteria all `- [x]`
- [ ] No leftover TODOs without linked issues
- [ ] No `#[allow(...)]` without comments + linked issues

### Approval gate (per CLAUDE.md §19)

- [ ] **Hard Gate G1: do-not-commit-until-approved held** — all 4 stage commits happened only after explicit user approval (verify against git log + retrospective claims)
- [ ] User has reviewed each stage retrospective; scoring matches observable evidence
- [ ] M01-summary verdict is "Pattern held" (sound) or "Pattern held with friction" (apply soft-gate fixes); not "Pattern strained"

If all checked, the M01 PR is ready to merge.

---

*End of M01 Foundation specification + stage prompts.*

