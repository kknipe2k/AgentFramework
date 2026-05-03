# M02 Event Pipeline — Specification + Stage Prompts

**Date:** 2026-05-02
**Status:** Design approved — implement one stage at a time, in order
**Scope:** Bring the event pipeline alive. Stand up `runtime-main` with the `LLMProvider` trait + `AnthropicProvider` (direct HTTP+SSE), wire it through an `AgentSdk` that emits `AgentEvent`s, connect main↔drone IPC over the Unix socket / Windows named pipe shipped in M01, and surface events to a skeleton React renderer through Tauri's typed IPC. End state: user clicks "Run smoke test" in the renderer, main calls Anthropic with a hardcoded prompt, the renderer lists `agent_spawned → tool_invoked → stream_text → agent_complete` events as they arrive. Spec §2 + §M2 acceptance criteria.

---

## Background and Design Decision

**Problem.** M01 shipped the persistence + recovery substrate (drone) and the type-generation pipeline. There's nothing yet that produces events to persist, nothing that calls an LLM, nothing the user can click. M02 is the smallest possible end-to-end slice that proves the architecture works: renderer → main → provider → API → events back through main → renderer, with the drone watching from the side.

**Solution.** Six stages on one feature branch (`claude/m02-event-pipeline`), each a fresh Claude Code session. Stage A absorbs all M01 carry-forward Important items (build hygiene + scaffolds) so Stages B–E focus on the real M02 deliverables. Stage B defines the `LLMProvider` trait and a stub `AnthropicProvider` (so Stages D–E can depend on it). Stage C lands the real `AnthropicProvider` HTTP+SSE implementation. Stage D wraps it in `AgentSdk` and connects to the M01 drone. Stage E adds the Tauri shell + minimal renderer + frontend CI gates. Stage F runs the Phase Closeout gap analysis per CLAUDE.md §20.

**Why one PR for the parent milestone (not one PR per stage).** Same as M01 — stages-as-commits-on-one-branch gives incremental discipline (each stage is reviewable, each stage retrospective surfaces friction early) without the overhead of six PR reviews for one logical milestone. Consistent with the per-milestone-as-PR pattern in `docs/build-prompts/README.md`.

**Why six stages, not four.** The original B (LLMProvider + AnthropicProvider in one stage) estimated ~4h, at the upper end of the safe single-session budget. Splitting into B (trait + stub) + C (real impl) keeps every stage under 4h actual, which protects against context compaction risk mid-stage and means the trait surface is stable before the real HTTP code lands. Phase Closeout as Stage F follows the corrected TEMPLATE.md sequence (Stage E in M01's pattern was Phase Closeout; M02 has one extra work stage so Phase Closeout is F).

**Key constraints.**
- §0d Release Scope Matrix — M02 is in scope; out-of-scope items (graph, plan, HITL, budget, MCP) stay deferred to M03+.
- Direct Anthropic API only — no `@anthropic-ai/sdk`, no `anthropic-rs`. `reqwest` + `eventsource-stream`. CLAUDE.md §15 trap #9.
- Zero telemetry — no analytics SDK, no crash reporter, no phone-home. Spec §13 + CLAUDE.md §4 hard rule #4.
- API key from OS keychain via `keyring` — never env vars, never files. Spec §M2 acceptance.
- `AnthropicProvider`'s SSE retry loop is a safety primitive — ≥95% coverage gate per the M01.C codification (`*_with` / `_inner` test-seam pattern, OS-signal-orchestrator-class exclusions documented). CLAUDE.md §5.
- Frontend lands skeleton-only — single page listing events as `<ul>`. No React Flow, no Tailwind, no graph rendering. Those land in M03.
- Real-API smoke test gated behind `cargo test --features integration` — CI uses wiremock only; no API key in CI.

**License.** Apache 2.0; DCO sign-off (`git commit -s`) on every commit.

**Existing patterns to mirror.**
- `docs/build-prompts/M01-foundation.md` — overall stage structure, surface-and-wait pattern, six-section per-stage shape.
- `docs/build-prompts/TEMPLATE.md` — Stage D/E sequence (Stage <work> = commit only; Phase Closeout = commit + push + PR), estimation calibration paragraph, safety primitive coverage gate dual-policy.
- `crates/runtime-drone/src/{lib,shutdown}.rs` — the `*_with` / `*_inner` test-seam pattern for OS-signal-driven async (M01.C archetype). Stage C replicates this for the SSE retry loop.
- `crates/runtime-drone/src/ipc.rs` — Unix socket + Windows named pipe pattern. Stage D's main-side client mirrors the cfg-platform structure.
- `crates/runtime-drone/src/db.rs::init_schema` — table creation pattern. Stage A extends with `mcp_servers`.
- `docs/gap-analysis.md` M01 entry "Adherence to spec" + Carry-forward — Stage A's work list comes from the 🟡 Important items targeting M02.
- `docs/build-prompts/retrospectives/M01-summary.md` Decisions section — Stage A applies the protocol-doc decisions.

---

## Document Structure

| Stage | Summary | Estimated effort |
|---|---|---|
| **A** | Build hygiene + scaffolding (M01 carry-forward + signal.rs + HeartbeatStatus + mcp_servers table) | ~1.5h |
| **B** | `LLMProvider` trait + `ProviderEvent` enum + `AnthropicProvider` stub + deps | ~1.5h |
| **C** | `AnthropicProvider` real HTTP+SSE impl + wiremock tests + `*_with` SSE pattern + ≥95% coverage | ~3h |
| **D** | `AgentSdk` + main↔drone IPC client + `ProviderEvent`→`AgentEvent` translation | ~2.5h |
| **E** | Tauri shell + skeleton React renderer + frontend CI gates + Playwright smoke | ~3h |
| **F** | Phase Closeout: Gap Analysis (per CLAUDE.md §20) | ~1.5h |

**Total: ~13h actual.** Calibrated from M01's 0.3× ratio (M01 estimated 26–40h, ran 10.5h). Each stage under 4h to protect against context compaction.

**Estimation calibration.** Per TEMPLATE.md and M01-summary.md decisions: M01's method overestimated by ~3×. M02 estimates above are already calibrated. Stage B is the riskiest estimate (LLM provider work has more unknowns than workspace setup); flag at Stage B retrospective if it ran significantly long. For each stage, the analogous M01 stage and complexity multiplier:

- **A** ~ M01.A workspace skeleton (2h actual) × 0.75 (smaller scope, mostly file edits not new crates) = 1.5h.
- **B** ~ M01.B type generation pipeline (4h actual) × 0.4 (trait + stub is simpler than typify integration) = 1.5h.
- **C** ~ M01.C drone Phase 1 (3h actual) × 1.0 (similar surface — async I/O + cfg-platform; lifted by the existing `*_with` archetype) = 3h.
- **D** ~ M01.C drone Phase 1 (3h actual) × 0.85 (smaller scope; drone server already done, only client side new) = 2.5h.
- **E** ~ M01.A workspace skeleton (2h actual) × 1.5 (frontend tooling is new — first npm/Vite/Vitest/Playwright integration) = 3h.
- **F** ~ M01 Stage E Phase Closeout (estimated 1.5h, will measure) = 1.5h.

---

## Implementation Workflow

Each stage runs through this exact cycle:

```
1. /clear                     — fresh context (between stages)
2. cd /d C:\agent-runtime && git pull origin main
                              — pull any prior-stage merges + protocol updates
3. Paste CLI Prompt below     — Claude reads CLAUDE.md, prior retros, stage X.1–X.4
4. Failing tests first        — Claude writes tests; cargo test --workspace shows red
5. Implement                  — Claude makes production changes
6. Gates green                — fmt + clippy + test + doc + audit + deny + llvm-cov + xtask
7. Fill in retrospective      — docs/build-prompts/retrospectives/M02.<X>-retrospective.md
8. Commit (no push, except final stage)  — exact commit message provided per stage
9. User reviews + approves    — Claude does NOT push without approval
10. Push + PR                 — only on Stage F (Phase Closeout) approval
```

**Rule:** If a new test passes before implementation, the test is wrong — stop and fix the test.

**Rule:** Stages are sequential. Stage B does not start until Stage A's commit is on the feature branch.

**Rule per CLAUDE.md §8:** Claude does not commit without user approval. Surface diff stat + retrospective + draft commit message; wait for explicit approval; commit.

**Rule per CLAUDE.md §19:** Each stage produces a retrospective; Stage F also produces `M02-summary.md` aggregating across stages.

**Rule per CLAUDE.md §20 (Stage F-specific):** Phase Closeout is the final commit on the parent-milestone branch. Push + PR happen only after Stage F approval.

---

<!-- ============================================================ -->
<!-- STAGE A — Build hygiene + scaffolding                          -->
<!-- ============================================================ -->

## Stage A — Build hygiene + scaffolding

### A.1 Problem Statement

M01 shipped with a backlog of Important carry-forward items in `docs/gap-analysis.md` M01 entry — some are spec-side (resolved by PR #36), some are protocol-doc (resolved by PR #37), and the rest are code-side cleanups that need to land before M02's real work begins. Stage A absorbs all of them in a single hygiene-and-scaffolding pass so Stages B–E can focus on the actual event pipeline without distraction.

Stage A also stands up two scaffolds M02 work depends on: `runtime-core/src/signal.rs` (the Signal Schema v2 type definitions per spec §2b — actual signal emission integration is M04+ work, but the types need to exist for runtime-main to import) and `runtime-core/src/drone.rs::HeartbeatStatus` typed-enum adoption (replacing the current `String` field per the spec definition added in PR #36).

**Success criterion.** Working tree is clean across Linux/macOS/Windows; CI green; the eight backlog items are closed; the M02 type/scaffolding surface is in place for Stage B to import.

**New artifacts:**
- `.gitattributes` (new)
- `crates/runtime-core/src/signal.rs` (new)
- `crates/runtime-drone/tests/integration_windows.rs` (new) OR cfg-extended `tests/integration.rs`

### A.2 Files to Change

| File | Change |
|---|---|
| `.gitattributes` | **New** — line-ending normalization (`*.rs text eol=lf`, `*.json text eol=lf`, `*.md text eol=lf`) |
| `.gitignore` | **Edited** — add `src-tauri/gen/schemas/` (Tauri-generated build artifacts) |
| `crates/runtime-drone/src/db.rs` | **Edited** — add `mcp_servers` table to `init_schema()` per spec §11:2435-2444 |
| `crates/runtime-drone/tests/integration.rs` | **Edited** — split into Unix + Windows variants OR add sibling `tests/integration_windows.rs` exercising `ipc::accept_loop` on `cfg(windows)` |
| `crates/runtime-core/src/signal.rs` | **New** — Signal Schema v2 types (8-variant `Signal` enum + correlation field types); scaffold only, no emission yet |
| `crates/runtime-core/src/lib.rs` | **Edited** — add `pub mod signal;` |
| `crates/runtime-core/src/drone.rs` | **Edited** — replace `Heartbeat { status: String, ... }` with `Heartbeat { status: HeartbeatStatus, ... }`; define `HeartbeatStatus` enum per spec §1d |
| `crates/runtime-drone/src/heartbeat.rs` | **Edited** — emit `HeartbeatStatus::Ok` (typed) instead of `"ok"` (string); update tests |
| `CLAUDE.md` | **Edited** — §5 add "Coverage delta gating mechanism" subsection (M02 baseline; subsequent milestones gate on delta vs `main`) |
| `docs/style.md` | **Edited** — add "*_with / _inner test-seam pattern" subsection under "Function design"; cite M01.C `lib.rs` + `shutdown.rs` archetype |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` Added/Tests/Documentation sections noting Stage A deliverables |

### A.3 Detailed Changes

#### `.gitattributes` (new)

```
# Force LF line endings for all text files. Avoids Windows CRLF/LF noise on
# cross-platform clones (M01 carry-forward — see docs/gap-analysis.md M01 entry
# Important "*.gitattributes line-ending normalization*").

* text=auto eol=lf

*.rs   text eol=lf
*.toml text eol=lf
*.json text eol=lf
*.md   text eol=lf
*.yml  text eol=lf
*.yaml text eol=lf
*.sh   text eol=lf

# Binary files — never normalize.
*.png binary
*.ico binary
*.jpg binary
*.gif binary
```

After committing, run `git add --renormalize .` once to apply to existing files. Commit any resulting line-ending changes as a separate commit (or include in Stage A commit; either works since it's noise-only).

#### `.gitignore` (edited)

Append the following block at the end of the file:

```
# Tauri build-time generated schemas. Regenerated on every cargo build of
# src-tauri; not source-of-truth and platform-specific. M01 carry-forward —
# see docs/gap-analysis.md M01 entry Important "src-tauri/gen/schemas/ should
# be gitignored to prevent future drift" (surfaced in PR #36 surface).
src-tauri/gen/schemas/
```

After adding, untrack the existing committed copies:
```
git rm --cached src-tauri/gen/schemas/desktop-schema.json
git rm --cached src-tauri/gen/schemas/windows-schema.json
git rm --cached src-tauri/gen/schemas/acl-manifests.json
git rm --cached src-tauri/gen/schemas/capabilities.json
```

The files stay on disk (regenerated by cargo build); they just no longer track in git.

#### `crates/runtime-drone/src/db.rs` (edited)

Find the `init_schema` function. After the existing 7 table creation statements (sessions, snapshots, signals, heartbeats, vdr, token_usage, skills), add the 8th per spec §11:2435-2444 + MCP-client best practice (Claude Code, Claude Desktop, VS Code).

The schema is comprehensive (22 fields) because MCP servers vary substantially: stdio servers spawn local processes (need command + args + env); HTTP/SSE servers connect remote (need URL + headers + auth); both need connection-state tracking, retry counts, configurable timeouts, OAuth state, and capability caching. Schema cost now is zero; migration cost when M06 lands is non-trivial. Also: NEVER store literal secrets — `auth_token_ref`, `env_json` values, and OAuth tokens are *references to OS keychain entries* enforced by the Rust constructor layer.

```rust
// 8th table — mcp_servers. Per spec §11:2435-2444 + MCP best practice
// (Claude Code, Claude Desktop, VS Code MCP client schemas). Fields cover:
// identity, transport-specific config (stdio: command/args/env; remote:
// url/headers), authentication (with keychain refs — NEVER literal secrets),
// connection lifecycle, timeouts, scope tracking (user/project/plugin/local),
// and capability caching. Mutual-exclusion CHECK enforces stdio-vs-remote
// invariant at the SQL level. M01 gap-analysis Important resolved via
// option (a) — schema is stable; no MCP code yet, lands in M06.
conn.execute(
    "CREATE TABLE IF NOT EXISTS mcp_servers (
        id                          INTEGER PRIMARY KEY AUTOINCREMENT,

        -- Identity
        name                        TEXT NOT NULL UNIQUE,
        transport                   TEXT NOT NULL
                                    CHECK (transport IN ('stdio', 'http', 'sse', 'streamable_http')),

        -- stdio transport (NULL for non-stdio)
        command                     TEXT,
        args_json                   TEXT,
        env_json                    TEXT,

        -- http / sse / streamable_http transport (NULL for stdio)
        url                         TEXT,
        headers_json                TEXT,

        -- Authentication (any transport)
        auth_kind                   TEXT
                                    CHECK (auth_kind IN ('none', 'bearer', 'oauth', 'custom') OR auth_kind IS NULL),
        auth_token_ref              TEXT,
        oauth_state_json            TEXT,

        -- Connection lifecycle
        status                      TEXT NOT NULL DEFAULT 'configured'
                                    CHECK (status IN ('configured', 'connected', 'errored', 'disabled', 'failed')),
        last_error                  TEXT,
        last_connected_at           INTEGER,
        retry_count                 INTEGER NOT NULL DEFAULT 0,

        -- Timeouts (Claude Code defaults; per-server override)
        startup_timeout_ms          INTEGER NOT NULL DEFAULT 10000,
        tool_timeout_ms             INTEGER NOT NULL DEFAULT 60000,

        -- Configuration metadata
        enabled                     BOOLEAN NOT NULL DEFAULT 1,
        scope                       TEXT NOT NULL DEFAULT 'user'
                                    CHECK (scope IN ('user', 'project', 'plugin', 'local')),
        plugin_id                   TEXT,

        -- Capabilities cache (populated on first successful connect)
        discovered_tool_count       INTEGER,
        last_capabilities_refresh   INTEGER,

        -- Audit timestamps
        added_at                    INTEGER NOT NULL,
        updated_at                  INTEGER NOT NULL,

        -- Mutual-exclusion invariant (stdio vs remote)
        CHECK (
            (transport = 'stdio' AND command IS NOT NULL AND url IS NULL)
            OR
            (transport IN ('http', 'sse', 'streamable_http') AND url IS NOT NULL AND command IS NULL)
        )
    )",
    [],
)?;

conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_mcp_servers_status ON mcp_servers(status)",
    [],
)?;
conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_mcp_servers_enabled ON mcp_servers(enabled)",
    [],
)?;
conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_mcp_servers_scope ON mcp_servers(scope)",
    [],
)?;
```

**Security invariant** (Rust-layer enforcement, not schema):
- The `auth_token_ref` column stores keychain entry names (e.g., `agent-runtime/mcp/github/token`), NEVER the literal token.
- `env_json` map values are either non-secret literals (e.g., `RUST_LOG=info`) or keychain refs prefixed with `keychain://` (e.g., `keychain://agent-runtime/mcp/db/password`).
- `oauth_state_json` contains `{access_token_ref, refresh_token_ref, expires_at, scopes[]}` — all token fields are refs.
- The Rust insert/update path validates this invariant; tests cover the negative case (literal-looking secrets in `auth_token_ref` panic in debug, log error in release).

#### `crates/runtime-drone/tests/integration.rs` (edited or split)

Current: `#[cfg(unix)]` only — exercises SIGTERM lifecycle via subprocess. Spec §0d says v0.1 is Windows-only; the integration test must also run on Windows.

Two options. Pick whichever makes for cleaner test code:

**Option A — split into two files.** Move existing test to `tests/integration_unix.rs` with `#[cfg(unix)]`. Add new `tests/integration_windows.rs` with `#[cfg(windows)]` that exercises:
- Drone subprocess startup
- Connect to Windows named pipe
- Send `DroneCommand::SnapshotNow` over the pipe
- Verify snapshot row appears in SQLite
- Send `DroneCommand::GracefulShutdown { timeout_ms: 1000 }`
- Verify drone exits cleanly within timeout

**Option B — single file with both cfg gates.** Keep `tests/integration.rs`; gate one helper per OS; use a single `#[test]` with platform-conditional inner code.

Recommended: Option A. Cleaner separation; easier to maintain. Each file is one cfg block.

Test target: lift `crates/runtime-drone/src/ipc.rs` Windows-platform coverage above 84.70% baseline. Verify via `cargo llvm-cov --package runtime-drone` on Windows-CI cell.

#### `crates/runtime-core/src/signal.rs` (new)

Per spec §2b. Signal Schema v2 types — 8-variant `Signal` enum with correlation fields. Scaffold only; actual emission integration lands in M04 (when verify/HITL/plan come online).

```rust
//! Signal Schema v2 — forensic event log types (spec §2b).
//!
//! Signals are write-heavy operational forensics. The VDR projection layer
//! consumes them and produces decision-focused rows (`vdr` table). This
//! module defines the type surface; emission integration is M04+ work.

use serde::{Deserialize, Serialize};

/// Reference to a prior signal in a causal chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreSignalId(pub String);

/// Reference to a parent signal (e.g., the agent that triggered this).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParentSignalId(pub String);

/// Reference to a signal this is a retry of.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryOfSignalId(pub String);

/// What kind of context this signal was produced in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextType {
    AgentLoop,
    SkillLoad,
    ToolInvoke,
    HookExecute,
    PlanCreate,
    HitlPrompt,
    SessionLifecycle,
}

/// Signal — forensic event. 8 kinds per spec §2b.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Signal {
    Tool {
        signal_id: String,
        agent_id: String,
        tool_name: String,
        payload_json: serde_json::Value,
        pre_signal_id: Option<PreSignalId>,
        parent_signal_id: Option<ParentSignalId>,
        retry_of: Option<RetryOfSignalId>,
        context_type: ContextType,
    },
    Skill {
        signal_id: String,
        agent_id: String,
        skill_name: String,
        skill_version: String,
        payload_json: serde_json::Value,
        parent_signal_id: Option<ParentSignalId>,
        context_type: ContextType,
    },
    Agent {
        signal_id: String,
        agent_id: String,
        event: String, // spawned | complete | error
        payload_json: serde_json::Value,
        parent_signal_id: Option<ParentSignalId>,
        context_type: ContextType,
    },
    Decision {
        signal_id: String,
        agent_id: String,
        decision: String,
        rationale: String,
        tool_used: Option<String>,
        payload_json: serde_json::Value,
        parent_signal_id: Option<ParentSignalId>,
        context_type: ContextType,
    },
    Verify {
        signal_id: String,
        agent_id: String,
        hook_id: String,
        passed: bool,
        payload_json: serde_json::Value,
        parent_signal_id: Option<ParentSignalId>,
        context_type: ContextType,
    },
    Error {
        signal_id: String,
        agent_id: Option<String>,
        error_kind: String,
        message: String,
        payload_json: serde_json::Value,
        parent_signal_id: Option<ParentSignalId>,
        retry_of: Option<RetryOfSignalId>,
        context_type: ContextType,
    },
    Hitl {
        signal_id: String,
        agent_id: String,
        prompt: String,
        response: Option<String>,
        payload_json: serde_json::Value,
        parent_signal_id: Option<ParentSignalId>,
        context_type: ContextType,
    },
    Session {
        signal_id: String,
        event: String, // start | suspend | resume | end
        payload_json: serde_json::Value,
        context_type: ContextType,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_round_trip_preserves_all_fields() {
        // Round-trip test for one variant; expand in M04 when emission integrates.
        let s = Signal::Session {
            signal_id: "sig-1".into(),
            event: "start".into(),
            payload_json: serde_json::json!({"session_id": "s-1"}),
            context_type: ContextType::SessionLifecycle,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: Signal = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
```

#### `crates/runtime-core/src/lib.rs` (edited)

Add the module declaration in the appropriate location (alphabetical order with existing `pub mod` lines):

```rust
pub mod signal;
```

#### `crates/runtime-core/src/drone.rs` (edited)

Locate the `DroneEvent::Heartbeat` variant. Currently:

```rust
Heartbeat { status: String, timestamp: i64 },
```

Replace with:

```rust
Heartbeat { status: HeartbeatStatus, timestamp: i64 },
```

Add the new enum (place near other public types in the file):

```rust
/// Status reported in `DroneEvent::Heartbeat`. Matches the strings written by
/// `runtime-drone::heartbeat::run` to the `heartbeats.status` SQLite column.
/// Defined in spec §1d (post-M01 docs(spec): PR #36 closeout).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeartbeatStatus {
    Ok,
    Degraded,
    Stalled,
}
```

#### `crates/runtime-drone/src/heartbeat.rs` (edited)

Find every site that constructs `DroneEvent::Heartbeat { status: "ok".to_string(), ... }` (or similar string literal) and replace with `DroneEvent::Heartbeat { status: HeartbeatStatus::Ok, ... }`. Same pattern for "degraded" and "stalled" if present.

Also update SQLite column writes — `heartbeats.status` is `TEXT`, so serialize via `serde_json::to_string(&status)?` or call `.as_str()` if you add a helper. Simpler: derive `Display` on `HeartbeatStatus` matching the snake_case rename.

Update existing unit tests in `heartbeat.rs` to use the typed enum.

#### `CLAUDE.md` §5 (edited)

Locate the "Coverage thresholds" subsection. After the existing safety-primitive policy paragraph, add:

```
**Coverage delta gating (from M02 onward).** M01 used absolute thresholds
(workspace ≥80%, drone ≥95%) because no baseline existed. Starting M02, every
PR also passes a delta-gate: workspace and per-safety-primitive coverage must
not regress vs `main`'s last green build. CI computes the delta via
`cargo-llvm-cov --json` for both PR HEAD and `origin/main`, fails the job if
any gated crate's line coverage drops by >0.5 percentage points (absolute).
The script lives at `.github/workflows/scripts/coverage-delta.sh` (added in
M02 Stage A). Pre-M01 carry-forward; resolved per the M01 gap-analysis
Important "Coverage delta gating mechanism" item.
```

#### `docs/style.md` (edited)

In the "Function design" section, after the existing four bullets, add a new subsection:

```
### `*_with` / `*_inner` test-seam pattern (OS-driven async functions)

When wrapping an OS-driven async future (signal handlers, long-lived I/O like
SSE streams, OS-timer cleanups), structure the function so the testable logic
lives in a separate `*_inner` (no parameter injection) or `*_with` (caller
injects the future) variant, with the production function being a thin
wrapper that constructs the OS future and delegates.

The pattern lifted M01.C drone coverage from 87% → 95% by making `lib::run`
and `shutdown::handle_emergency` testable without firing real OS signals
cross-platform. Cite `crates/runtime-drone/src/lib.rs::{run, run_inner}` and
`crates/runtime-drone/src/shutdown.rs::{wait_and_handle, wait_and_handle_with}`
as the archetype.

```rust
// Production wrapper — thin; delegates to *_with variant.
pub async fn wait_and_handle(state: &State) -> Result<()> {
    let signal = tokio::signal::ctrl_c();
    wait_and_handle_with(state, signal).await
}

// Testable variant — caller injects the signal future.
pub async fn wait_and_handle_with<F>(state: &State, signal: F) -> Result<()>
where
    F: Future<Output = std::io::Result<()>>,
{
    signal.await?;
    state.persist().await
}

// In tests: pass `async { Ok(()) }` for an immediate signal; pass
// `tokio::time::sleep(Duration::from_millis(50)).map(|_| Ok(()))` for
// timed-arrival behavior; etc.
```

The `_with` form is preferred when callers may legitimately want to inject
non-OS signals (e.g., a `oneshot::Receiver` for testing or a timer for
non-signal-driven shutdown). The `_inner` form is preferred when the function
just needs the testable seam without expanding the public API.

When excluding the production wrapper from coverage (because it can only be
exercised by firing real OS signals cross-platform), document it inline with
a one-line rationale per excluded function — see M01.C codification commit
`1dec4ba`.
```

#### `CHANGELOG.md` (edited)

In the `[Unreleased]` section, under existing categories, append:

```markdown
### Added
- `crates/runtime-core/src/signal.rs` — Signal Schema v2 type scaffold per spec §2b (8-variant `Signal` enum + correlation field types). Emission integration is M04+ work.
- `crates/runtime-core/src/drone.rs::HeartbeatStatus` typed enum (replaces `String`); spec §1d adoption per PR #36.
- `crates/runtime-drone/src/db.rs::init_schema` — 8th table `mcp_servers` per spec §11. Schema only; MCP code lands in M06.
- `crates/runtime-drone/tests/integration_windows.rs` — Windows-platform integration test exercising `ipc::accept_loop` over named pipe (lifts §0d Windows-only coverage gap).

### Changed
- Workspace coverage gate adds delta-gating (M02 baseline; CI fails on >0.5pp regression vs `main`). Documented in `CLAUDE.md` §5.

### Documentation
- `docs/style.md` — `*_with` / `*_inner` test-seam pattern documented as canonical TDD-friendly approach to OS-driven async functions.
- `.gitattributes` — line-ending normalization (`*.rs text eol=lf`, etc.) — closes M01 gap-analysis Important "line-ending normalization".
- `.gitignore` — `src-tauri/gen/schemas/` excluded — Tauri build artifacts, not source-of-truth.
```

### A.4 Tests

Stage A is mostly hygiene + scaffolding; new tests cover schema invariants and type round-trips:

1. **`crates/runtime-core/src/signal.rs::tests::signal_round_trip_preserves_all_fields`** — round-trip for one `Signal` variant (the others are M04 emission-integration concerns; testing all 8 here would be premature).
2. **`crates/runtime-drone/src/heartbeat.rs::tests::heartbeat_writes_typed_status_to_db`** — update existing tests if any string literals appear; verify the `heartbeats.status` column now receives the snake_case enum string from `HeartbeatStatus::Ok`.
3. **`crates/runtime-drone/src/db.rs::tests::init_schema_creates_mcp_servers_table`** — extend the existing schema-init tests to verify the 8th table exists with all 22 expected columns, check constraints, and indexes.
4. **`crates/runtime-drone/src/db.rs::tests::mcp_servers_stdio_invariant_enforced`** — insert a stdio row WITHOUT command (or WITH url) → expect SQL CHECK constraint failure.
5. **`crates/runtime-drone/src/db.rs::tests::mcp_servers_remote_invariant_enforced`** — insert an http/sse row WITHOUT url (or WITH command) → expect SQL CHECK constraint failure.
6. **`crates/runtime-drone/src/db.rs::tests::mcp_servers_status_transitions`** — insert with default status, update to 'connected', update to 'errored' with last_error → all transitions allowed; invalid status string → CHECK failure.
7. **`crates/runtime-drone/tests/integration_windows.rs`** — full subprocess test (per A.3 above): spawn drone, connect to named pipe, send `SnapshotNow`, verify snapshot row, send `GracefulShutdown`, verify clean exit. Gated `#[cfg(windows)]` only.
8. **`crates/runtime-drone/src/db.rs::tests::heartbeat_status_roundtrip_via_db`** — write `HeartbeatStatus::Degraded`, read back, assert equal.

#### Coverage target

- Workspace ≥80% (general gate, unchanged).
- `runtime-drone` ≥95% (with the documented OS-signal-orchestrator exclusions per `1dec4ba` codification, unchanged).
- `runtime-core` no specific gate (covered by workspace gate).
- New `signal.rs` module: ≥90% line on the test suite added (one variant exercised; others are M04 work).

**Safety primitive coverage gate.** Stage A doesn't add a new safety primitive — it edits the existing drone primitive. The dual-policy applies (workspace ≥80% + drone ≥95%); already in CI from M01.

**Coverage delta gate.** Stage A activates the new delta-gating mechanism documented in CLAUDE.md §5. Verify CI computes the delta vs `main` and passes (no regression expected since this is additive).

### A.5 CLI Prompt

```
Read CLAUDE.md for all project rules.
Read docs/build-prompts/M02-event-pipeline.md Stage A (sections A.1 through A.4).

[Stage A is the first stage of M02 — no prior M02 retrospectives exist yet.
Skip the prior-retrospective read step that Stage B+ will use.]

Read docs/gap-analysis.md M01 entry — Carry-forward section + every 🟡
Important item. Stage A's job is to close the M02-bound items. Apply.

Read docs/build-prompts/retrospectives/M01-summary.md "Decisions to apply
before the next parent milestone" section. Apply the items marked as M02-prep.

═══ STEP 1 — WRITE FAILING TESTS ═══

Create the new test files (or extend existing ones) per A.4:

1. crates/runtime-core/src/signal.rs::tests::signal_round_trip_preserves_all_fields
2. crates/runtime-drone/src/db.rs::tests::init_schema_creates_mcp_servers_table
3. crates/runtime-drone/tests/integration_windows.rs (full body per A.3)
4. crates/runtime-drone/src/db.rs::tests::heartbeat_status_roundtrip_via_db

Run: cargo test --workspace
Confirm: all 4 new tests fail with `unresolved import`, `cannot find type
HeartbeatStatus`, `cannot find table mcp_servers`, etc. — i.e., they fail
because the production code doesn't exist yet (TDD red phase).

If any new test passes before implementation, the test is wrong — stop and
fix it (per CLAUDE.md §5 "hard-fails on missing exports").

═══ STEP 2 — IMPLEMENT ═══

Apply each change per A.3, in this order:

1. .gitattributes (new) — content per A.3.
2. git add --renormalize . (apply to existing files; commit any line-ending
   diffs as part of Stage A).
3. .gitignore (edited) — append Tauri schema block per A.3.
4. git rm --cached src-tauri/gen/schemas/* (untrack but keep on disk).
5. crates/runtime-drone/src/db.rs — add mcp_servers table per A.3.
6. crates/runtime-drone/tests/integration_windows.rs (or split per Option A) —
   full Windows integration test per A.3.
7. crates/runtime-core/src/signal.rs (new) — full content per A.3.
8. crates/runtime-core/src/lib.rs — add `pub mod signal;`.
9. crates/runtime-core/src/drone.rs — add HeartbeatStatus enum, change the
   Heartbeat variant per A.3.
10. crates/runtime-drone/src/heartbeat.rs — update string literals to typed
    enum + Display impl as needed.
11. CLAUDE.md §5 — add "Coverage delta gating" subsection per A.3.
12. docs/style.md — add "*_with / _inner test-seam pattern" subsection per A.3.
13. CHANGELOG.md [Unreleased] — append Added/Changed/Documentation sections.

For step 11 (coverage delta gating), the .github/workflows/scripts/coverage-
delta.sh script also needs creation. Implement as a small bash script that:
- Runs `cargo llvm-cov --workspace --json --output-path /tmp/cov-pr.json`
- Checks out `origin/main`, runs the same, outputs to `/tmp/cov-main.json`
- Diffs per-crate line coverage; fails if any gated crate regresses >0.5pp
- Wire into the existing Coverage CI job (`.github/workflows/ci.yml`)

═══ STEP 3 — VERIFY ═══

Run each gate; all must pass:

  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo build --workspace
  cargo test --workspace
  cargo test --workspace --doc
  RUSTDOCFLAGS="-D missing_docs" cargo doc --workspace --no-deps
  cargo audit
  cargo deny check
  cargo llvm-cov --workspace --ignore-filename-regex "src.main\.rs|generated" --fail-under-lines 80
  cargo llvm-cov --package runtime-drone --ignore-filename-regex "src.main\.rs|generated|src.lib\.rs|src.shutdown\.rs" --fail-under-lines 95
  cargo run --bin xtask -- regenerate-types --check

Verify locally on Windows that the new integration_windows test passes:
  cargo test --package runtime-drone --test integration_windows

If any gate fails, follow CLAUDE.md §7 self-correction. Max 3 iterations
then surface.

═══ STEP 4 — RETROSPECTIVE ═══

Per CLAUDE.md §19, copy retrospectives/RETROSPECTIVE-TEMPLATE.md to:
  docs/build-prompts/retrospectives/M02.A-retrospective.md

Fill in [LIVE] sections during work, then [END] scoring + threshold gates +
decisions for Stage B (specifically: which scaffolds Stage B will import
from runtime-core; whether the *_with pattern doc landed in the right form;
whether the coverage delta gate ran cleanly on first try).

═══ STEP 5 — SURFACE TO USER ═══

Run: git status, git diff --stat HEAD
Re-run all gates one final time.

Surface: diff stat, gate results, M02.A retrospective, draft commit from A.6.

State: "Stage A is ready. I will NOT commit until you approve."

Wait for explicit approval. Do NOT push (push waits for Stage F per CLAUDE.md §20).

On approval (Stage A — work stage; not the final stage of a parent milestone):
1. Commit Stage A on the parent-milestone branch claude/m02-event-pipeline
   (do NOT push).
2. Stop. Surface the commit. Stage B is opened in a fresh session.
```

### A.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(workspace): M02 Stage A — build hygiene + scaffolding

Closes M01 carry-forward Important items + scaffolds for M02 Stages B–E.

Hygiene:
- .gitattributes — LF normalization for *.rs/*.toml/*.json/*.md/*.yml/*.sh
  (closes M01 gap-analysis "line-ending normalization")
- .gitignore — exclude src-tauri/gen/schemas/ (Tauri-autogenerated; not
  source-of-truth; closes follow-up from PR #36 surface)
- runtime-drone db.rs — add 8th SQLite table mcp_servers per spec §11
  (resolves M01 gap-analysis Important via option (a) — schema is stable;
  no MCP code yet, lands in M06)
- runtime-drone tests/integration_windows.rs — Windows-platform integration
  test exercising ipc::accept_loop over named pipe (lifts §0d Windows-only
  coverage gap)

Scaffolds:
- runtime-core/src/signal.rs (NEW) — Signal Schema v2 types per spec §2b
  (8-variant Signal enum + correlation field types). Emission integration
  is M04+ work.
- runtime-core/src/drone.rs — HeartbeatStatus typed enum replaces String
  per spec §1d adoption (post-M01 docs(spec): PR #36).
- runtime-drone/src/heartbeat.rs — emit typed HeartbeatStatus to SQLite
  via Display impl.

Process:
- CLAUDE.md §5 — Coverage delta gating mechanism documented (M02 baseline;
  CI fails on >0.5pp regression vs main).
- docs/style.md — *_with / _inner test-seam pattern documented as canonical
  TDD approach to OS-driven async fns. Cites M01.C archetype at
  crates/runtime-drone/src/{lib,shutdown}.rs and codification commit 1dec4ba.

CHANGELOG.md [Unreleased] reflects Added/Changed/Documentation deltas.

Refs: M02-event-pipeline.md §A; docs/gap-analysis.md M01 entry "Adherence
to spec" + Carry-forward; M01-summary.md Decisions section.

Retrospective: docs/build-prompts/retrospectives/M02.A-retrospective.md

https://claude.ai/code/session_<id>
EOF
)"
```

---

<!-- ============================================================ -->
<!-- Stages B – F — to be authored in subsequent chunks             -->
<!-- ============================================================ -->

[Stages B, C, D, E, F authored in subsequent chunks per the parent-conversation chunked-authoring protocol. This file grows incrementally; each chunk surfaces for approval before the next is appended.]

---

## Summary Table

[Filled in after Stage F is authored.]

---

## Verification Checklist

[Filled in after Stage F is authored.]

---

*M02 prompt — Stage A authored. Stages B–F pending.*
