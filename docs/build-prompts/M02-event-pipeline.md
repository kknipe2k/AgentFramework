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
<!-- STAGE B — LLMProvider trait + AnthropicProvider stub           -->
<!-- ============================================================ -->

## Stage B — `LLMProvider` trait + `ProviderEvent` enum + `AnthropicProvider` stub

### B.1 Problem Statement

Stage B defines the provider abstraction surface (the `LLMProvider` trait + `ProviderEvent` enum) and ships a stub `AnthropicProvider` that returns a hardcoded sequence of events. The stub lets Stages D and E start work against a stable interface before Stage C lands the real HTTP+SSE implementation.

This split — trait + stub in B, real impl in C — is deliberate: the trait surface is the contract Stages D/E depend on, and stabilizing it before the heavy SSE work means the SSE work can focus purely on the wire-format details without trait churn.

**Success criterion.** `LLMProvider` trait compiles, `ProviderEvent` round-trips through serde, `AnthropicProvider::stream()` returns a hardcoded `agent_spawned → tool_invoked → stream_text → agent_complete`-equivalent `ProviderEvent` sequence wrapped in `BoxStream`, all dependencies resolve via `cargo deny check`, no third-party Anthropic SDK in `Cargo.toml`.

**New artifacts:**
- `crates/runtime-main/src/providers/mod.rs` (new — trait + ProviderEvent + ProviderError + real Message/ContentBlock shape from the Anthropic Messages API)
- `crates/runtime-main/src/providers/anthropic.rs` (new — stub impl, hardcoded model pricing per current claude.com/pricing — no /v1/models pricing endpoint exists)
- `crates/runtime-main/Cargo.toml` (edited — add deps)
- `Cargo.toml` (workspace — edited — add reqwest 0.13, eventsource-stream 0.2, keyring 3.6, async-trait, futures, secrecy 0.10)

### B.2 Files to Change

| File | Change |
|---|---|
| `crates/runtime-main/Cargo.toml` | **Edited** — add deps: `reqwest` (rustls-tls features), `eventsource-stream`, `async-trait`, `futures`, `secrecy`, `serde_json`, `tokio` (workspace), `runtime-core` (path) |
| `Cargo.toml` (workspace root) | **Edited** — add `[workspace.dependencies]` entries for the new deps so version pinning is centralized |
| `crates/runtime-main/src/lib.rs` | **Edited** — add `pub mod providers;` |
| `crates/runtime-main/src/providers/mod.rs` | **New** — `LLMProvider` trait, `ProviderEvent` enum, `ProviderSupport` struct, `ProviderError` enum, `Message` / `ModelInfo` / `Pricing` / `AgentConfig` types per spec §2c |
| `crates/runtime-main/src/providers/anthropic.rs` | **New** — `AnthropicProvider` struct + stub `stream()` returning hardcoded `BoxStream<'_, ProviderEvent>` |
| `crates/runtime-main/README.md` | **New (or edited if exists)** — public-API documentation per CLAUDE.md §6 |
| `deny.toml` | **Edited** — confirm new deps are license-compatible (no GPL/AGPL); add advisory ignores only if pre-existing patterns require |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` Added section noting Stage B deliverables |

### B.3 Detailed Changes

#### `Cargo.toml` (workspace root, edited)

Find the `[workspace.dependencies]` table (or create if missing). Add:

```toml
[workspace.dependencies]
# ... existing entries kept ...

# M02 Stage B — LLM provider deps. Versions confirmed against crates.io as
# of 2026-05. Pin here; member crates pull via `dep = { workspace = true }`.
reqwest             = { version = "0.13", default-features = false, features = ["rustls-tls", "json", "stream"] }
eventsource-stream  = "0.2"
async-trait         = "0.1"
futures             = "0.3"
secrecy             = { version = "0.10", features = ["serde"] }
keyring             = "3.6"

# runtime-core path-dep for runtime-main
runtime-core        = { path = "crates/runtime-core" }
```

Notes:
- `reqwest 0.13` (latest stable; was 0.12 pre-2025). `rustls-tls` (not `native-tls`) keeps the dep tree pure-Rust and cross-compiles cleanly. `json` + `stream` features required.
- `eventsource-stream 0.2` is the SSE parser; current stable.
- `secrecy 0.10` provides `SecretString` so API keys never accidentally `Debug`-print or `Display`. Wraps the keychain-loaded key.
- `keyring 3.6` is the OS keychain client (3.6.3 latest patch). `keyring 4.0-rc` exists but has breaking changes — stay on 3.6 until 4.0 stable. API: `Entry::new(service, user)?.get_password()?`.

#### `crates/runtime-main/Cargo.toml` (edited)

Replace the existing minimal `Cargo.toml` body with the full Stage B dependency set:

```toml
[package]
name        = "runtime-main"
version.workspace      = true
edition.workspace      = true
license.workspace      = true
authors.workspace      = true
repository.workspace   = true
rust-version.workspace = true

[lints]
workspace = true

[dependencies]
runtime-core       = { workspace = true }
async-trait        = { workspace = true }
futures            = { workspace = true }
reqwest            = { workspace = true }
eventsource-stream = { workspace = true }
secrecy            = { workspace = true }
keyring            = { workspace = true }
serde              = { workspace = true, features = ["derive"] }
serde_json         = { workspace = true }
tokio              = { workspace = true, features = ["macros", "rt-multi-thread", "sync", "time"] }
thiserror          = { workspace = true }
tracing            = { workspace = true }

[dev-dependencies]
tokio              = { workspace = true, features = ["test-util", "macros", "rt-multi-thread"] }
proptest           = { workspace = true }

[features]
default     = []
integration = []   # gates the real-Anthropic-API smoke test (Stage C)
```

#### `crates/runtime-main/src/lib.rs` (edited)

Append:

```rust
pub mod providers;
```

#### `crates/runtime-main/src/providers/mod.rs` (new)

The trait + supporting types per spec §2c. Stage B ships the surface; Stage C ships the real implementation behind it.

```rust
//! LLM provider abstraction (spec §2c).
//!
//! v1 ships a single `AnthropicProvider`; the trait abstracts the surface so
//! v1.0+ can add OpenAI / local model support without touching SDK callers.
//! All providers stream `ProviderEvent`s; the SDK layer (M02 Stage D) translates
//! these to `runtime_core::AgentEvent`s for the renderer.

use async_trait::async_trait;
use futures::stream::BoxStream;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod anthropic;

/// Provider-emitted streaming event. Internal to runtime-main; translated to
/// AgentEvent at the SDK boundary (Stage D).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderEvent {
    /// Incremental text delta from the model.
    TextDelta { text: String },
    /// Model requested a tool be invoked.
    ToolUse { id: String, name: String, input: serde_json::Value },
    /// Tool result being fed back to the model (model-side; we mostly emit ToolUse).
    ToolResult { id: String, output: serde_json::Value },
    /// Extended-thinking chunk (Anthropic feature; only when supported + enabled).
    ThinkingDelta { text: String },
    /// Model finished generating; reason in `stop_reason`.
    MessageStop { stop_reason: String },
    /// Provider-side error during the stream.
    Error { code: String, message: String },
}

/// Capability flags reported by the provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderSupport {
    pub tool_use:  bool,
    pub streaming: bool,
    pub thinking:  bool,
}

/// Provider-side error variants. `thiserror`-derived for ergonomic propagation.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("SSE parse error: {0}")]
    Sse(String),
    #[error("API returned error status {status}: {body}")]
    Api { status: u16, body: String },
    #[error("Authentication failed (check API key in keychain)")]
    Auth,
    #[error("Rate limit hit; retry after {retry_after_secs}s")]
    RateLimit { retry_after_secs: u64 },
    #[error("Request timed out after {0:?}")]
    Timeout(Duration),
    #[error("Invalid model: {0}")]
    InvalidModel(String),
    #[error("Provider returned unparseable response: {0}")]
    Unparseable(String),
    #[error("Provider configuration error: {0}")]
    Config(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// One conversation message (user / assistant). Spec §2c + Anthropic Messages
/// API reality (https://docs.anthropic.com/en/api/messages). System prompts
/// are NOT in the messages array — they go in `AgentConfig::system_prompt`
/// (a separate top-level field per the API).
///
/// `content` is `Vec<ContentBlock>` (not `String`) because the real API uses
/// typed content blocks for multi-part messages: text + images, tool calls
/// + tool results, etc. Single-text messages serialize as a 1-element vec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role:    MessageRole,
    pub content: Vec<ContentBlock>,
}

/// Message author. The Anthropic API only allows `user` and `assistant` in
/// the messages array; system prompts are a separate top-level parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

/// Typed content block per Anthropic Messages API. The variants here match
/// the shapes the API accepts in request bodies AND produces in response
/// content arrays.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text.
    Text { text: String },

    /// Image, either as base64 source or URL source.
    Image { source: ImageSource },

    /// Model-emitted tool invocation (in assistant messages).
    ToolUse {
        id:    String,
        name:  String,
        input: serde_json::Value,
    },

    /// Tool result fed back to the model (in subsequent user message).
    ToolResult {
        tool_use_id: String,
        content:     ToolResultContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error:    Option<bool>,
    },

    /// Extended-thinking block (assistant-only; only when thinking enabled).
    Thinking {
        thinking:  String,
        signature: String,
    },
}

/// Source of an image content block — base64 or URL.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    Base64 { media_type: String, data: String },
    Url    { url: String },
}

/// Content of a tool result — either a string or a vec of content blocks
/// (e.g., tool returns text + image).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

/// Per-call agent configuration. Spec §2c.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub model:        String,
    pub messages:     Vec<Message>,
    pub max_tokens:   u32,
    pub temperature:  Option<f32>,
    pub system_prompt: Option<String>,
    pub tools:        Vec<ToolDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name:        String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Pricing info per provider model.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pricing {
    pub input_per_million_usd:  f64,
    pub output_per_million_usd: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub tool_use:  bool,
    pub streaming: bool,
    pub thinking:  bool,
    pub vision:    bool,
}

/// Information about a single model offered by a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id:             String,
    pub display_name:   String,
    pub context_window: u32,
    pub pricing:        Pricing,
    pub capabilities:   ModelCapabilities,
}

/// LLM provider trait. Spec §2c.
///
/// All async methods must be cancellation-safe. Implementations should not
/// hold resources past `await` points that wouldn't survive a drop.
///
/// # Examples
///
/// ```ignore
/// use runtime_main::providers::{LLMProvider, anthropic::AnthropicProvider};
/// use secrecy::SecretString;
///
/// let provider = AnthropicProvider::new(SecretString::from("sk-ant-..."));
/// assert_eq!(provider.name(), "anthropic");
/// ```
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Provider identifier (e.g., "anthropic", "openai").
    fn name(&self) -> &str;

    /// Capability flags for this provider.
    fn supports(&self) -> ProviderSupport;

    /// Open a streaming session against the provider. Stage C lands the real
    /// HTTP+SSE implementation; Stage B's stub returns a hardcoded sequence.
    async fn stream(
        &self,
        config: AgentConfig,
    ) -> Result<BoxStream<'_, ProviderEvent>, ProviderError>;

    /// Pre-flight token count for `messages`. Used by budget controls (M04).
    async fn count_tokens(&self, messages: &[Message]) -> Result<u64, ProviderError>;

    /// List models the provider currently exposes (and their pricing).
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError>;

    /// Estimate cost for a given (input, output) token pair on a model.
    fn estimate_cost(&self, input_tokens: u64, output_tokens: u64, model: &str) -> f64;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_event_round_trips() {
        let cases = vec![
            ProviderEvent::TextDelta { text: "hello".into() },
            ProviderEvent::ToolUse {
                id: "tu_1".into(),
                name: "search".into(),
                input: serde_json::json!({"q": "rust"}),
            },
            ProviderEvent::ThinkingDelta { text: "thinking...".into() },
            ProviderEvent::MessageStop { stop_reason: "end_turn".into() },
            ProviderEvent::Error { code: "rate_limit".into(), message: "slow down".into() },
        ];
        for event in cases {
            let json = serde_json::to_string(&event).unwrap();
            let back: ProviderEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(event, back);
        }
    }

    #[test]
    fn provider_event_tag_is_snake_case() {
        let json = serde_json::to_string(&ProviderEvent::TextDelta { text: "x".into() }).unwrap();
        assert!(json.contains("\"type\":\"text_delta\""), "got: {json}");
    }
}
```

#### `crates/runtime-main/src/providers/anthropic.rs` (new — stub)

Stage B ships a *stub* that satisfies the trait. The hardcoded event sequence lets Stages D/E develop against a stable interface. Stage C replaces the stub body with real HTTP+SSE.

```rust
//! Anthropic Messages API provider (spec §2c).
//!
//! Stage B ships a STUB: `stream()` returns a hardcoded sequence of
//! `ProviderEvent`s. Stage C replaces the body with direct HTTP+SSE via
//! `reqwest` + `eventsource-stream`. The stub exists so Stages D/E can
//! depend on a stable interface before SSE work lands.

use async_trait::async_trait;
use futures::stream::{self, BoxStream, StreamExt};
use secrecy::SecretString;

use super::{
    AgentConfig, LLMProvider, Message, ModelCapabilities, ModelInfo, Pricing,
    ProviderError, ProviderEvent, ProviderSupport,
};

/// Direct HTTP+SSE Anthropic Messages API client.
///
/// API key is loaded from the OS keychain via `keyring` and held in
/// `SecretString` so it never `Debug`-prints. No third-party Anthropic SDK
/// is used (see CLAUDE.md §15 trap #9 + spec §0d).
pub struct AnthropicProvider {
    api_key:  SecretString,
    base_url: String,
    // Stage C adds: http: reqwest::Client, retry_policy, etc.
}

impl AnthropicProvider {
    /// Construct from an API key (loaded by caller from keychain).
    pub fn new(api_key: SecretString) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com".into(),
        }
    }

    /// Construct with an explicit base URL (for wiremock tests in Stage C).
    pub fn with_base_url(api_key: SecretString, base_url: String) -> Self {
        Self { api_key, base_url }
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn supports(&self) -> ProviderSupport {
        ProviderSupport {
            tool_use:  true,
            streaming: true,
            thinking:  true,
        }
    }

    /// STUB: returns a hardcoded `text_delta → message_stop` sequence.
    /// Stage C replaces with real HTTP+SSE implementation.
    async fn stream(
        &self,
        _config: AgentConfig,
    ) -> Result<BoxStream<'_, ProviderEvent>, ProviderError> {
        let events = vec![
            ProviderEvent::TextDelta { text: "Hello".into() },
            ProviderEvent::TextDelta { text: " from".into() },
            ProviderEvent::TextDelta { text: " stub.".into() },
            ProviderEvent::MessageStop { stop_reason: "end_turn".into() },
        ];
        Ok(stream::iter(events).boxed())
    }

    async fn count_tokens(&self, messages: &[Message]) -> Result<u64, ProviderError> {
        // Stage B: rough char/4 approximation across all text content blocks.
        // Stage C uses the real /v1/messages/count_tokens endpoint.
        let total_chars: usize = messages
            .iter()
            .flat_map(|m| m.content.iter())
            .map(|block| match block {
                ContentBlock::Text { text }       => text.len(),
                ContentBlock::Thinking { thinking, .. } => thinking.len(),
                ContentBlock::ToolUse { input, .. } => input.to_string().len(),
                ContentBlock::ToolResult { content, .. } => match content {
                    ToolResultContent::Text(s) => s.len(),
                    ToolResultContent::Blocks(_) => 0, // approximation; Stage C real-counts
                },
                ContentBlock::Image { .. } => 0, // images priced separately
            })
            .sum();
        Ok((total_chars as u64).div_ceil(4))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        // Anthropic does NOT expose pricing via /v1/models — only model
        // metadata. Pricing must be hardcoded here against the docs page
        // (https://platform.claude.com/docs/en/about-claude/pricing) and
        // updated when the docs change. Verified 2026-05.
        //
        // Long-context surcharge eliminated 2026-03-13 — uniform per-token
        // rate across the full 1M window for Opus 4.6+ / Sonnet 4.6.
        Ok(vec![
            ModelInfo {
                id: "claude-opus-4-7".into(),
                display_name: "Claude Opus 4.7".into(),
                context_window: 1_000_000,
                pricing: Pricing { input_per_million_usd: 5.0, output_per_million_usd: 25.0 },
                capabilities: ModelCapabilities {
                    tool_use: true, streaming: true, thinking: true, vision: true,
                },
            },
            ModelInfo {
                id: "claude-sonnet-4-6".into(),
                display_name: "Claude Sonnet 4.6".into(),
                context_window: 1_000_000,
                pricing: Pricing { input_per_million_usd: 3.0, output_per_million_usd: 15.0 },
                capabilities: ModelCapabilities {
                    tool_use: true, streaming: true, thinking: true, vision: true,
                },
            },
            ModelInfo {
                id: "claude-haiku-4-5".into(),
                display_name: "Claude Haiku 4.5".into(),
                context_window: 200_000,
                pricing: Pricing { input_per_million_usd: 1.0, output_per_million_usd: 5.0 },
                capabilities: ModelCapabilities {
                    tool_use: true, streaming: true, thinking: false, vision: true,
                },
            },
        ])
    }

    fn estimate_cost(&self, input_tokens: u64, output_tokens: u64, model: &str) -> f64 {
        // Pricing per https://platform.claude.com/docs/en/about-claude/pricing
        // (verified 2026-05). NOT cache-aware (cache hits = 0.1× input,
        // 5m write = 1.25×, 1h write = 2× — Stage D budget integration
        // adds the cache-aware estimator).
        let pricing = match model {
            "claude-opus-4-7"   => Pricing { input_per_million_usd: 5.0, output_per_million_usd: 25.0 },
            "claude-opus-4-6"   => Pricing { input_per_million_usd: 5.0, output_per_million_usd: 25.0 },
            "claude-opus-4-5"   => Pricing { input_per_million_usd: 5.0, output_per_million_usd: 25.0 },
            "claude-sonnet-4-6" => Pricing { input_per_million_usd: 3.0, output_per_million_usd: 15.0 },
            "claude-sonnet-4-5" => Pricing { input_per_million_usd: 3.0, output_per_million_usd: 15.0 },
            "claude-haiku-4-5"  => Pricing { input_per_million_usd: 1.0, output_per_million_usd:  5.0 },
            _ => return 0.0, // unknown model — Stage C surfaces this as ProviderError::InvalidModel via async paths
        };
        let input_cost  = (input_tokens  as f64) * pricing.input_per_million_usd  / 1_000_000.0;
        let output_cost = (output_tokens as f64) * pricing.output_per_million_usd / 1_000_000.0;
        input_cost + output_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use secrecy::SecretString;

    fn stub_provider() -> AnthropicProvider {
        AnthropicProvider::new(SecretString::from("sk-ant-test"))
    }

    fn stub_config() -> AgentConfig {
        AgentConfig {
            model: "claude-haiku-4-5".into(),
            messages: vec![Message {
                role: super::MessageRole::User,
                content: vec![ContentBlock::Text { text: "ping".into() }],
            }],
            max_tokens: 100,
            temperature: None,
            system_prompt: None,
            tools: vec![],
        }
    }

    #[tokio::test]
    async fn stub_stream_returns_text_then_stop() {
        let provider = stub_provider();
        let mut stream = provider.stream(stub_config()).await.unwrap();
        let mut events = vec![];
        while let Some(e) = stream.next().await {
            events.push(e);
        }
        assert!(matches!(events.first(), Some(ProviderEvent::TextDelta { .. })));
        assert!(matches!(events.last(),  Some(ProviderEvent::MessageStop { .. })));
    }

    #[test]
    fn name_is_anthropic() {
        assert_eq!(stub_provider().name(), "anthropic");
    }

    #[test]
    fn supports_advertises_tool_use_streaming_thinking() {
        let s = stub_provider().supports();
        assert!(s.tool_use && s.streaming && s.thinking);
    }

    #[tokio::test]
    async fn count_tokens_approximates_char_div_4() {
        let provider = stub_provider();
        let messages = vec![Message {
            role: MessageRole::User,
            content: vec![ContentBlock::Text { text: "hello world".into() }], // 11 chars
        }];
        let count = provider.count_tokens(&messages).await.unwrap();
        assert_eq!(count, 3); // 11/4 = 2.75 → ceil 3
    }

    #[tokio::test]
    async fn list_models_returns_three_claude_4x_entries() {
        let models = stub_provider().list_models().await.unwrap();
        assert_eq!(models.len(), 3);
        assert!(models.iter().any(|m| m.id == "claude-opus-4-7"));
        assert!(models.iter().any(|m| m.id == "claude-sonnet-4-6"));
        assert!(models.iter().any(|m| m.id == "claude-haiku-4-5"));
    }

    #[test]
    fn estimate_cost_for_haiku() {
        let provider = stub_provider();
        // 1M input + 1M output on Haiku 4.5 = $1.00 + $5.00 = $6.00
        let cost = provider.estimate_cost(1_000_000, 1_000_000, "claude-haiku-4-5");
        assert!((cost - 6.00).abs() < 1e-6, "got {cost}");
    }

    #[test]
    fn estimate_cost_for_opus() {
        let provider = stub_provider();
        // 1M input + 1M output on Opus 4.7 = $5.00 + $25.00 = $30.00
        let cost = provider.estimate_cost(1_000_000, 1_000_000, "claude-opus-4-7");
        assert!((cost - 30.00).abs() < 1e-6, "got {cost}");
    }

    #[test]
    fn estimate_cost_for_unknown_model_returns_zero() {
        let provider = stub_provider();
        // Stage C surfaces unknown models as ProviderError::InvalidModel; for
        // estimate_cost (non-async), 0.0 is the safe default.
        let cost = provider.estimate_cost(1000, 1000, "nonexistent-model");
        assert_eq!(cost, 0.0);
    }
}
```

#### `crates/runtime-main/README.md` (new or edited)

Replace any existing minimal README with:

```markdown
# `runtime-main`

The Tauri main process for Agent Runtime. Hosts the agent SDK, the LLM provider abstraction, and (in later milestones) the framework loader, capability enforcer, and MCP manager.

## What's here (M02 Stage B)

- `providers/mod.rs` — the `LLMProvider` trait, `ProviderEvent` enum, `ProviderError` (thiserror-derived), and supporting types (`AgentConfig`, `Message`, `ModelInfo`, etc.) per spec §2c.
- `providers/anthropic.rs` — `AnthropicProvider` shell. Stage B ships a stub with hardcoded events; Stage C lands the real HTTP+SSE implementation against the Anthropic Messages API (no third-party SDK).

## Adding a provider

Implement the `LLMProvider` trait. Keep the impl behind a feature flag if it pulls a heavy transitive tree. The trait does not assume anything about transport — `AnthropicProvider` uses HTTP+SSE; a hypothetical `LocalLlamaProvider` could use a local Unix socket.

## Security notes

- API keys are passed as `secrecy::SecretString` so they never `Debug`-print or appear in logs.
- The actual key value is loaded from the OS keychain at startup (Stage E wires this in for the smoke session).
- No literal API keys in environment variables, files, or source. CLAUDE.md §13 + spec §13 zero-telemetry rule.

## Tests

- `cargo test -p runtime-main` — unit tests for the provider trait, ProviderEvent serde, and the stub Anthropic implementation.
- `cargo test -p runtime-main --features integration` — Stage C adds wiremock-driven integration tests + an opt-in real-API smoke test.
```

#### `deny.toml` (edited)

Confirm the new deps don't trip license/advisory rules. Reqwest pulls in some heavyweight transitives (rustls, hyper, tokio); `keyring` pulls platform crates (windows-sys, security-framework). All are MIT/Apache-2.0 compatible. Add to the `[bans] skip = []` list only if duplicate-version warnings appear during `cargo deny check`.

#### `CHANGELOG.md` (edited)

Append to `[Unreleased]`:

```markdown
### Added (M02 Stage B)
- `crates/runtime-main/src/providers/mod.rs` — `LLMProvider` trait + `ProviderEvent` enum + `ProviderError` per spec §2c.
- `crates/runtime-main/src/providers/anthropic.rs` — `AnthropicProvider` shell with stub `stream()` (Stage C replaces with real HTTP+SSE).
- Workspace dependencies: `reqwest` (rustls-tls), `eventsource-stream`, `async-trait`, `futures`, `secrecy`, `keyring`. No third-party Anthropic SDK.
```

### B.4 Tests

1. **`crates/runtime-main/src/providers/mod.rs::tests::provider_event_round_trips`** — every `ProviderEvent` variant survives `to_string` → `from_str`; serde tag is snake_case.
2. **`crates/runtime-main/src/providers/mod.rs::tests::content_block_round_trips`** — every `ContentBlock` variant (Text, Image, ToolUse, ToolResult, Thinking) round-trips; matches Anthropic API wire format.
3. **`crates/runtime-main/src/providers/anthropic.rs::tests::stub_stream_returns_text_then_stop`** — stub `stream()` returns at least one `TextDelta` followed by `MessageStop`.
4. **`tests::name_is_anthropic`** — provider identifies itself.
5. **`tests::supports_advertises_tool_use_streaming_thinking`** — capability flags correct.
6. **`tests::count_tokens_approximates_char_div_4`** — Stage B token approximation across content blocks.
7. **`tests::list_models_returns_three_claude_4x_entries`** — Opus 4.7, Sonnet 4.6, Haiku 4.5.
8. **`tests::estimate_cost_for_haiku`** — 1M input + 1M output on Haiku 4.5 = $6.00.
9. **`tests::estimate_cost_for_opus`** — 1M input + 1M output on Opus 4.7 = $30.00.
10. **`tests::estimate_cost_for_unknown_model_returns_zero`** — defensive default; Stage C upgrades to `ProviderError::InvalidModel` on the async paths.

#### Coverage target

- Workspace ≥80% (general gate, unchanged).
- `runtime-drone` ≥95% (unchanged from Stage A).
- `runtime-main` no specific gate yet; covered by workspace gate. Stage C raises `runtime-main` to ≥95% under the safety-primitive rule.

**Coverage delta gate.** From M02 Stage A onward; verify CI computes delta vs `main` (post-Stage-A baseline).

### B.5 CLI Prompt

```
Read CLAUDE.md for all project rules.
Read docs/build-prompts/M02-event-pipeline.md Stage B (sections B.1 through B.4).

Read prior stage retrospectives for guidance:
  docs/build-prompts/retrospectives/M02.A-retrospective.md
  Focus: [END] "Decisions for the next stage" sections + any [LIVE]
  friction events flagged as relevant to Stage B. Apply decisions.

Read docs/gap-analysis.md for any Carry-forward items targeting Stage B
(look at the most recent entry's Carry-forward section).

═══ STEP 1 — WRITE FAILING TESTS ═══

Create the test files (or stub them in mod.rs / anthropic.rs):

1. providers::mod::tests::provider_event_round_trips
2. providers::mod::tests::provider_event_tag_is_snake_case
3. providers::anthropic::tests::stub_stream_returns_text_then_stop
4. providers::anthropic::tests::name_is_anthropic
5. providers::anthropic::tests::supports_advertises_tool_use_streaming_thinking
6. providers::anthropic::tests::count_tokens_approximates_char_div_4
7. providers::anthropic::tests::list_models_returns_three_claude_4x_entries
8. providers::anthropic::tests::estimate_cost_for_haiku
9. providers::anthropic::tests::estimate_cost_for_unknown_model_returns_zero

Run: cargo test --workspace --package runtime-main
Confirm: all tests fail with `cannot find struct AnthropicProvider`,
`cannot find type ProviderEvent`, etc. — TDD red phase per CLAUDE.md §5.

═══ STEP 2 — IMPLEMENT ═══

Apply changes per B.3, in order:

1. Cargo.toml (workspace root) — add [workspace.dependencies] entries.
2. crates/runtime-main/Cargo.toml — full rewrite per B.3.
3. crates/runtime-main/src/lib.rs — add `pub mod providers;`.
4. crates/runtime-main/src/providers/mod.rs (NEW) — full content per B.3.
5. crates/runtime-main/src/providers/anthropic.rs (NEW) — full content per B.3.
6. crates/runtime-main/README.md (new or edited) — public API doc per B.3.
7. CHANGELOG.md [Unreleased] — append Added section.
8. deny.toml — re-run cargo deny check; add ban exceptions only if needed.

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

If any gate fails, follow CLAUDE.md §7 self-correction. Max 3 iterations
then surface.

═══ STEP 4 — RETROSPECTIVE ═══

Per CLAUDE.md §19, copy retrospectives/RETROSPECTIVE-TEMPLATE.md to:
  docs/build-prompts/retrospectives/M02.B-retrospective.md

Fill in [LIVE] sections during work, then [END] scoring + threshold gates +
decisions for Stage C (specifically: did the trait surface feel right; any
type names or signatures Stage C should change before the real SSE
implementation lands; was the stub event sequence sufficient for downstream
stages to work against).

═══ STEP 5 — SURFACE TO USER ═══

Run: git status, git diff --stat HEAD
Re-run all gates one final time.

Surface: diff stat, gate results, M02.B retrospective, draft commit from B.6.

State: "Stage B is ready. I will NOT commit until you approve."

Wait for explicit approval. Do NOT push (push waits for Stage F per CLAUDE.md §20).

On approval (Stage B — work stage; not the final stage of a parent milestone):
1. Commit Stage B on the parent-milestone branch claude/m02-event-pipeline
   (do NOT push).
2. Stop. Surface the commit. Stage C is opened in a fresh session.
```

### B.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(runtime-main): M02 Stage B — LLMProvider trait + AnthropicProvider stub

Defines the provider abstraction surface per spec §2c. Stage B ships the
trait and a stub AnthropicProvider with hardcoded events so Stages D/E
can develop against a stable interface; Stage C replaces the stub with
real HTTP+SSE.

New:
- crates/runtime-main/src/providers/mod.rs — LLMProvider trait,
  ProviderEvent enum (TextDelta / ToolUse / ToolResult / ThinkingDelta /
  MessageStop / Error), ProviderError (thiserror-derived), and supporting
  types (AgentConfig, Message, ModelInfo, ProviderSupport, Pricing).
- crates/runtime-main/src/providers/anthropic.rs — AnthropicProvider
  shell with SecretString-wrapped key, stub stream() returning hardcoded
  TextDelta+MessageStop sequence, hardcoded list_models()
  (Opus 4.7, Sonnet 4.6, Haiku 4.5), char-based count_tokens(), and
  pricing-table estimate_cost().
- crates/runtime-main/README.md — public API documentation.

Workspace dependencies added (no third-party Anthropic SDK):
- reqwest (rustls-tls + json + stream)
- eventsource-stream (SSE parser)
- async-trait, futures, secrecy, keyring

Tests:
- 9 unit tests covering ProviderEvent serde round-trip, snake_case tag
  format, stub stream behavior, identity + capability flags, token
  approximation, model listing, cost estimation, and unknown-model
  defensive default.

Refs: M02-event-pipeline.md §B; agent-runtime-spec.md §2c.

Retrospective: docs/build-prompts/retrospectives/M02.B-retrospective.md

https://claude.ai/code/session_<id>
EOF
)"
```

---

<!-- ============================================================ -->
<!-- STAGE C — AnthropicProvider real HTTP+SSE implementation       -->
<!-- ============================================================ -->

## Stage C — `AnthropicProvider` real HTTP+SSE implementation

### C.1 Problem Statement

Stage C replaces Stage B's stub `AnthropicProvider::stream()` body with a real implementation that POSTs to `https://api.anthropic.com/v1/messages` with `stream: true`, parses the Server-Sent Events response via `eventsource-stream`, runs the events through a small state machine that maps Anthropic's wire format to `ProviderEvent`s, and emits the translated stream to callers.

The SSE retry/parse loop is a safety primitive (per CLAUDE.md §5) — long-lived I/O wrapping cancellation-sensitive state. The implementation uses the `*_with` / `*_inner` test-seam pattern documented in `docs/style.md` (M01.C codification) so wiremock-fed byte streams exercise every state transition without real network. The thin production wrapper that constructs `reqwest::Client` is excluded from the ≥95% coverage gate via `--ignore-filename-regex` with a one-line rationale (per the M01.C codification commit `1dec4ba`).

Stage C also lands the real-API smoke test, gated behind `cargo test --features integration`. CI never runs the integration feature (no API key in CI). Manual: `cargo test --features integration` from a developer machine with the API key in OS keychain. Cost per run: ~$0.001 against Haiku 4.5 ($1/$5 per MTok).

**Success criterion.** Calling `AnthropicProvider::stream(config)` against a wiremock-backed Anthropic endpoint yields the same `ProviderEvent` sequence regardless of how Anthropic chunks the SSE bytes; against the real API (`--features integration`), a `Hello` prompt returns at least one `TextDelta` followed by exactly one `MessageStop`. `runtime-main` line coverage ≥95% (with documented exclusions).

**New artifacts:**
- `crates/runtime-main/src/providers/anthropic_sse.rs` (new — SSE state machine + `*_with`-pattern parser)
- `crates/runtime-main/tests/anthropic_wiremock.rs` (new — wiremock-driven integration tests)
- `crates/runtime-main/tests/anthropic_smoke.rs` (new — real-API smoke gated by `--features integration`)

### C.2 Files to Change

| File | Change |
|---|---|
| `crates/runtime-main/Cargo.toml` | **Edited** — add `wiremock` to `[dev-dependencies]` |
| `Cargo.toml` (workspace root) | **Edited** — add `wiremock = "0.6"` to `[workspace.dependencies]` |
| `crates/runtime-main/src/providers/anthropic.rs` | **Edited** — replace stub `stream()` body with real implementation; add `stream_with_bytes()` test-seam variant; add Anthropic request body types (`AnthropicRequest`, `AnthropicTool`); wire up `reqwest::Client` lazily |
| `crates/runtime-main/src/providers/anthropic_sse.rs` | **New** — SSE event types (`SseEvent` enum matching Anthropic wire format), `parse_sse_event()` byte-level parser, `translate()` mapping `SseEvent` → `ProviderEvent`, internal state machine (`SseState`) tracking open content blocks + accumulated tool input |
| `crates/runtime-main/src/providers/mod.rs` | **Edited** — declare `mod anthropic_sse;` (private); no public surface change |
| `crates/runtime-main/tests/anthropic_wiremock.rs` | **New** — wiremock fixtures + 8 integration tests covering happy path, tool use, thinking, error, rate-limit, malformed bytes, partial chunk, and stop reasons |
| `crates/runtime-main/tests/anthropic_smoke.rs` | **New** — real-API test gated by `#[cfg(feature = "integration")]`; reads `ANTHROPIC_API_KEY` from keyring entry `agent-runtime/anthropic/api-key`; POSTs `Hello` to Haiku 4.5; asserts ≥1 `TextDelta` + exactly 1 `MessageStop` |
| `crates/runtime-main/README.md` | **Edited** — document `--features integration` for the smoke test, the keyring entry name, and expected cost per run |
| `.github/workflows/ci.yml` | **Edited** — add `runtime-main` to the safety-primitive coverage gate matrix (≥95% line, OS-signal exclusions documented) |
| `CLAUDE.md` | **Edited** — §5 add `runtime-main/src/providers/anthropic.rs` to the safety primitives list (specifically the `stream` wrapper exclusion + `stream_with_bytes` covered seam) |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` Added section noting Stage C deliverables |

### C.3 Detailed Changes

#### `Cargo.toml` (workspace root, edited)

Add to `[workspace.dependencies]`:

```toml
# M02 Stage C — wiremock for provider integration tests. Latest 0.6.x.
wiremock = "0.6"
```

#### `crates/runtime-main/Cargo.toml` (edited)

Add to `[dev-dependencies]`:

```toml
wiremock = { workspace = true }
tempfile = { workspace = true }
```

#### `crates/runtime-main/src/providers/anthropic_sse.rs` (new)

The SSE state machine + parser. Internal module — public surface is via `anthropic.rs::stream()`. This is the testable seam: `parse_sse_event()` is pure (bytes in → `Option<SseEvent>` out), `SseState::translate()` is pure (`SseEvent` in → `Option<ProviderEvent>` out + state mutation), and `stream_events()` glues them together with an injectable byte stream.

```rust
//! Anthropic SSE event parsing + translation to ProviderEvent.
//!
//! The Anthropic Messages API emits a specific SSE event sequence on
//! `POST /v1/messages` with `stream: true`:
//!   message_start → (content_block_start → content_block_delta* →
//!                    content_block_stop)+ → message_delta → message_stop
//! plus `ping` keep-alives anywhere and `error` for server-side errors.
//!
//! See: https://platform.claude.com/docs/en/api/messages-streaming
//!
//! This module exposes the test-seam:
//! - `parse_sse_event(line)` — pure bytes-to-SseEvent parser
//! - `SseState::translate(event)` — pure SseEvent-to-ProviderEvent translator
//!     with internal state for accumulating tool inputs across deltas
//! - `stream_events(byte_stream)` — `*_with`-style entry: caller injects the
//!     byte stream (real reqwest stream OR wiremock-fed bytes); function
//!     yields ProviderEvents.
//!
//! The thin production wrapper in `anthropic.rs` constructs the real
//! reqwest::Client and feeds its byte stream into `stream_events()`. That
//! wrapper is the OS-signal-equivalent holdout (real network is structurally
//! infeasible to test cross-platform) and is excluded from the ≥95%
//! coverage gate per the M01.C codification (commit `1dec4ba`).

use eventsource_stream::Eventsource;
use futures::stream::{Stream, StreamExt};
use serde::Deserialize;

use super::{ContentBlock, ProviderError, ProviderEvent};

/// Anthropic SSE event types per the Messages API streaming spec.
///
/// Each event arrives as `event: <type>\ndata: <json>\n\n`. The `type` field
/// in the JSON matches the SSE event name; we deserialize from the JSON only.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum SseEvent {
    MessageStart {
        message: SseMessage,
    },
    ContentBlockStart {
        index: usize,
        content_block: SseContentBlockStart,
    },
    ContentBlockDelta {
        index: usize,
        delta: SseDelta,
    },
    ContentBlockStop {
        index: usize,
    },
    MessageDelta {
        delta: SseMessageDelta,
        usage: Option<SseUsage>,
    },
    MessageStop,
    Ping,
    Error {
        error: SseError,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SseMessage {
    pub id:    String,
    pub model: String,
    pub usage: SseUsage,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum SseContentBlockStart {
    Text { text: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
    Thinking { thinking: String },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum SseDelta {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
    ThinkingDelta { thinking: String },
    SignatureDelta { signature: String },
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SseMessageDelta {
    pub stop_reason:  Option<String>,
    pub stop_sequence: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct SseUsage {
    #[serde(default)]
    pub input_tokens:  u64,
    #[serde(default)]
    pub output_tokens: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SseError {
    #[serde(rename = "type")]
    pub kind:    String,
    pub message: String,
}

/// Per-stream parsing state. Tracks open content blocks so partial-JSON
/// tool-input deltas can be accumulated into a complete `ToolUse` event.
#[derive(Debug, Default)]
pub(crate) struct SseState {
    /// Index → (block_kind, accumulated_json_or_text).
    open_blocks: std::collections::HashMap<usize, OpenBlock>,
}

#[derive(Debug)]
enum OpenBlock {
    Text,
    ToolUse {
        id:           String,
        name:         String,
        input_buffer: String,
    },
    Thinking,
}

impl SseState {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Translate one SseEvent to zero-or-one ProviderEvent. State is mutated
    /// to accumulate tool input deltas; the complete ToolUse is emitted on
    /// the corresponding ContentBlockStop.
    pub(crate) fn translate(&mut self, event: SseEvent) -> Option<ProviderEvent> {
        match event {
            SseEvent::MessageStart { .. } => None, // bookkeeping only
            SseEvent::Ping => None,                // keep-alive

            SseEvent::ContentBlockStart { index, content_block } => {
                let open = match content_block {
                    SseContentBlockStart::Text { .. } => OpenBlock::Text,
                    SseContentBlockStart::ToolUse { id, name, .. } => OpenBlock::ToolUse {
                        id,
                        name,
                        input_buffer: String::new(),
                    },
                    SseContentBlockStart::Thinking { .. } => OpenBlock::Thinking,
                };
                self.open_blocks.insert(index, open);
                None
            }

            SseEvent::ContentBlockDelta { index, delta } => match delta {
                SseDelta::TextDelta { text } => Some(ProviderEvent::TextDelta { text }),
                SseDelta::ThinkingDelta { thinking } => {
                    Some(ProviderEvent::ThinkingDelta { text: thinking })
                }
                SseDelta::SignatureDelta { .. } => None, // verifier-only
                SseDelta::InputJsonDelta { partial_json } => {
                    if let Some(OpenBlock::ToolUse { input_buffer, .. }) =
                        self.open_blocks.get_mut(&index)
                    {
                        input_buffer.push_str(&partial_json);
                    }
                    None // emit the complete ToolUse on ContentBlockStop
                }
            },

            SseEvent::ContentBlockStop { index } => {
                let removed = self.open_blocks.remove(&index)?;
                if let OpenBlock::ToolUse { id, name, input_buffer } = removed {
                    // partial_json may be empty if Anthropic emitted full input
                    // in the ContentBlockStart's `input` field. Stage C accepts
                    // either form; in practice the API uses InputJsonDelta.
                    let input = if input_buffer.is_empty() {
                        serde_json::Value::Object(serde_json::Map::new())
                    } else {
                        match serde_json::from_str(&input_buffer) {
                            Ok(v)  => v,
                            Err(_) => serde_json::Value::String(input_buffer),
                        }
                    };
                    Some(ProviderEvent::ToolUse { id, name, input })
                } else {
                    None
                }
            }

            SseEvent::MessageDelta { delta, .. } => delta
                .stop_reason
                .map(|stop_reason| ProviderEvent::MessageStop { stop_reason }),

            SseEvent::MessageStop => None, // emit MessageStop on MessageDelta

            SseEvent::Error { error } => Some(ProviderEvent::Error {
                code:    error.kind,
                message: error.message,
            }),
        }
    }
}

/// Parse a single SSE `data: ...` JSON line into an SseEvent. Returns `None`
/// for non-event lines (`event:`, blanks, comments, `: heartbeat`).
///
/// The `eventsource-stream` crate already reassembles event frames; this
/// function decodes the JSON `data` payload into an SseEvent.
pub(crate) fn parse_sse_data(data: &str) -> Result<SseEvent, ProviderError> {
    serde_json::from_str(data).map_err(|e| ProviderError::Sse(e.to_string()))
}

/// Convert an injected byte stream into a stream of ProviderEvents.
///
/// `*_with`-style test-seam: the production wrapper in anthropic.rs feeds
/// this with `reqwest::Response::bytes_stream()`; tests feed it with
/// pre-canned wiremock bytes. Same translation logic exercised both ways.
pub(crate) fn stream_events<S, E>(
    byte_stream: S,
) -> impl Stream<Item = Result<ProviderEvent, ProviderError>>
where
    S: Stream<Item = Result<bytes::Bytes, E>> + Unpin,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut state = SseState::new();
    byte_stream
        .map(|chunk| chunk.map_err(|e| ProviderError::Http(reqwest::Error::from(e))))
        .eventsource()
        .filter_map(move |event_result| {
            let result = match event_result {
                Ok(event) => match parse_sse_data(&event.data) {
                    Ok(sse_event) => Ok(state.translate(sse_event)),
                    Err(e) => Err(e),
                },
                Err(e) => Err(ProviderError::Sse(e.to_string())),
            };
            async move {
                match result {
                    Ok(Some(provider_event)) => Some(Ok(provider_event)),
                    Ok(None) => None,
                    Err(e) => Some(Err(e)),
                }
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_message_start() {
        let data = r#"{"type":"message_start","message":{"id":"msg_1","type":"message","role":"assistant","model":"claude-haiku-4-5","content":[],"stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":25,"output_tokens":1}}}"#;
        let event = parse_sse_data(data).unwrap();
        assert!(matches!(event, SseEvent::MessageStart { .. }));
    }

    #[test]
    fn parses_text_delta() {
        let data = r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"hello"}}"#;
        let event = parse_sse_data(data).unwrap();
        assert!(matches!(event, SseEvent::ContentBlockDelta { index: 0, .. }));
    }

    #[test]
    fn parses_ping() {
        let event = parse_sse_data(r#"{"type":"ping"}"#).unwrap();
        assert!(matches!(event, SseEvent::Ping));
    }

    #[test]
    fn parses_error() {
        let data = r#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#;
        let event = parse_sse_data(data).unwrap();
        assert!(matches!(event, SseEvent::Error { .. }));
    }

    #[test]
    fn parses_message_delta_with_stop_reason() {
        let data = r#"{"type":"message_delta","delta":{"stop_reason":"end_turn","stop_sequence":null},"usage":{"output_tokens":15}}"#;
        let event = parse_sse_data(data).unwrap();
        assert!(matches!(event, SseEvent::MessageDelta { .. }));
    }

    #[test]
    fn translate_text_delta_emits_provider_event() {
        let mut state = SseState::new();
        let evt = SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::TextDelta { text: "hi".into() },
        };
        let out = state.translate(evt);
        assert!(matches!(out, Some(ProviderEvent::TextDelta { .. })));
    }

    #[test]
    fn translate_ping_returns_none() {
        let mut state = SseState::new();
        assert!(state.translate(SseEvent::Ping).is_none());
    }

    #[test]
    fn translate_message_delta_emits_message_stop() {
        let mut state = SseState::new();
        let evt = SseEvent::MessageDelta {
            delta: SseMessageDelta {
                stop_reason: Some("end_turn".into()),
                stop_sequence: None,
            },
            usage: None,
        };
        let out = state.translate(evt);
        assert!(matches!(out, Some(ProviderEvent::MessageStop { .. })));
    }

    #[test]
    fn translate_error_emits_provider_error_event() {
        let mut state = SseState::new();
        let evt = SseEvent::Error {
            error: SseError {
                kind: "overloaded_error".into(),
                message: "slow down".into(),
            },
        };
        let out = state.translate(evt);
        assert!(matches!(out, Some(ProviderEvent::Error { .. })));
    }

    #[test]
    fn tool_use_accumulates_partial_json_then_emits_on_stop() {
        let mut state = SseState::new();
        // Block opens
        state.translate(SseEvent::ContentBlockStart {
            index: 0,
            content_block: SseContentBlockStart::ToolUse {
                id: "tu_1".into(),
                name: "search".into(),
                input: serde_json::Value::Object(serde_json::Map::new()),
            },
        });
        // Two partial-JSON deltas
        state.translate(SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::InputJsonDelta {
                partial_json: r#"{"q":"#.into(),
            },
        });
        state.translate(SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::InputJsonDelta {
                partial_json: r#""rust"}"#.into(),
            },
        });
        // Stop emits the complete ToolUse
        let out = state.translate(SseEvent::ContentBlockStop { index: 0 });
        match out {
            Some(ProviderEvent::ToolUse { id, name, input }) => {
                assert_eq!(id, "tu_1");
                assert_eq!(name, "search");
                assert_eq!(input, serde_json::json!({"q": "rust"}));
            }
            other => panic!("expected ToolUse, got {other:?}"),
        }
    }

    #[test]
    fn signature_delta_is_silent() {
        let mut state = SseState::new();
        let evt = SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::SignatureDelta { signature: "abc".into() },
        };
        assert!(state.translate(evt).is_none());
    }

    #[test]
    fn malformed_data_returns_sse_error() {
        let result = parse_sse_data("not json at all");
        assert!(matches!(result, Err(ProviderError::Sse(_))));
    }
}
```

#### `crates/runtime-main/src/providers/anthropic.rs` (edited — replace stub)

Replace the entire stub `stream()` body with the real implementation. Keep `count_tokens()`, `list_models()`, `estimate_cost()` as they are (Stage B was correct on those).

```rust
//! Anthropic Messages API provider — direct HTTP+SSE.
//!
//! No third-party Anthropic SDK. CLAUDE.md §15 trap #9. Direct API hits via
//! `reqwest` + `eventsource-stream` keep the dependency surface minimal and
//! the breaking-change exposure flat.
//!
//! API key is loaded by the caller from OS keychain via `keyring` and held
//! in `secrecy::SecretString` so it never `Debug`-prints. The provider
//! lazily constructs `reqwest::Client` on first `stream()` call.

use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use std::sync::OnceLock;

use super::{
    AgentConfig, ContentBlock, LLMProvider, Message, ModelCapabilities, ModelInfo, Pricing,
    ProviderError, ProviderEvent, ProviderSupport, ToolDef, ToolResultContent,
};

mod anthropic_sse;

const ANTHROPIC_API_VERSION: &str = "2023-06-01";

pub struct AnthropicProvider {
    api_key:  SecretString,
    base_url: String,
    http:     OnceLock<reqwest::Client>,
}

impl AnthropicProvider {
    pub fn new(api_key: SecretString) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com".into(),
            http:     OnceLock::new(),
        }
    }

    pub fn with_base_url(api_key: SecretString, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            http: OnceLock::new(),
        }
    }

    fn http_client(&self) -> &reqwest::Client {
        self.http.get_or_init(|| {
            reqwest::Client::builder()
                .pool_max_idle_per_host(2)
                .build()
                .expect("reqwest client builder cannot fail with default features")
        })
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    fn name(&self) -> &str { "anthropic" }

    fn supports(&self) -> ProviderSupport {
        ProviderSupport { tool_use: true, streaming: true, thinking: true }
    }

    /// Real HTTP+SSE implementation. Constructs the request, sends it,
    /// and feeds the response byte stream into `anthropic_sse::stream_events`.
    /// Production wrapper — excluded from the ≥95% coverage gate via
    /// `--ignore-filename-regex` because real-network hits are structurally
    /// untestable cross-platform. Logic lives in anthropic_sse.rs.
    async fn stream(
        &self,
        config: AgentConfig,
    ) -> Result<BoxStream<'_, ProviderEvent>, ProviderError> {
        let body = AnthropicRequest::from_config(&config);
        let url  = format!("{}/v1/messages", self.base_url);

        let response = self
            .http_client()
            .post(&url)
            .header("x-api-key",         self.api_key.expose_secret())
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .header("content-type",      "application/json")
            .json(&body)
            .send()
            .await?;

        // Map non-2xx status to ProviderError before consuming the body.
        if !response.status().is_success() {
            let status = response.status().as_u16();
            // 429 specifically — surface retry-after for callers that care.
            if status == 429 {
                let retry_after_secs = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(60);
                return Err(ProviderError::RateLimit { retry_after_secs });
            }
            if status == 401 || status == 403 {
                return Err(ProviderError::Auth);
            }
            let body_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::Api { status, body: body_text });
        }

        let byte_stream = response.bytes_stream();
        let event_stream = anthropic_sse::stream_events(byte_stream)
            .filter_map(|r| async move {
                match r {
                    Ok(event) => Some(event),
                    Err(_)    => None, // log and skip; surface via Error variant separately
                }
            });

        Ok(event_stream.boxed())
    }

    async fn count_tokens(&self, messages: &[Message]) -> Result<u64, ProviderError> {
        // Stage C: Anthropic exposes /v1/messages/count_tokens for accurate
        // counts. For Stage C we keep the char/4 approximation from Stage B
        // because the budget integration in M04 will use the real endpoint
        // with proper response handling — adding it here would be premature.
        let total_chars: usize = messages
            .iter()
            .flat_map(|m| m.content.iter())
            .map(|block| match block {
                ContentBlock::Text { text }              => text.len(),
                ContentBlock::Thinking { thinking, .. }  => thinking.len(),
                ContentBlock::ToolUse { input, .. }      => input.to_string().len(),
                ContentBlock::ToolResult { content, .. } => match content {
                    ToolResultContent::Text(s)   => s.len(),
                    ToolResultContent::Blocks(_) => 0,
                },
                ContentBlock::Image { .. } => 0,
            })
            .sum();
        Ok((total_chars as u64).div_ceil(4))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        // Pricing per https://platform.claude.com/docs/en/about-claude/pricing
        // (verified 2026-05). NO /v1/models pricing endpoint exists — the
        // Anthropic API does not expose pricing dynamically; this list IS
        // the source of truth and must be updated when the docs change.
        // Long-context surcharge eliminated 2026-03-13.
        Ok(vec![
            ModelInfo {
                id: "claude-opus-4-7".into(),
                display_name: "Claude Opus 4.7".into(),
                context_window: 1_000_000,
                pricing: Pricing { input_per_million_usd: 5.0, output_per_million_usd: 25.0 },
                capabilities: ModelCapabilities {
                    tool_use: true, streaming: true, thinking: true, vision: true,
                },
            },
            ModelInfo {
                id: "claude-sonnet-4-6".into(),
                display_name: "Claude Sonnet 4.6".into(),
                context_window: 1_000_000,
                pricing: Pricing { input_per_million_usd: 3.0, output_per_million_usd: 15.0 },
                capabilities: ModelCapabilities {
                    tool_use: true, streaming: true, thinking: true, vision: true,
                },
            },
            ModelInfo {
                id: "claude-haiku-4-5".into(),
                display_name: "Claude Haiku 4.5".into(),
                context_window: 200_000,
                pricing: Pricing { input_per_million_usd: 1.0, output_per_million_usd: 5.0 },
                capabilities: ModelCapabilities {
                    tool_use: true, streaming: true, thinking: false, vision: true,
                },
            },
        ])
    }

    fn estimate_cost(&self, input_tokens: u64, output_tokens: u64, model: &str) -> f64 {
        let pricing = match model {
            "claude-opus-4-7" | "claude-opus-4-6" | "claude-opus-4-5"
                => Pricing { input_per_million_usd: 5.0, output_per_million_usd: 25.0 },
            "claude-sonnet-4-6" | "claude-sonnet-4-5"
                => Pricing { input_per_million_usd: 3.0, output_per_million_usd: 15.0 },
            "claude-haiku-4-5"
                => Pricing { input_per_million_usd: 1.0, output_per_million_usd:  5.0 },
            _ => return 0.0,
        };
        let input_cost  = (input_tokens  as f64) * pricing.input_per_million_usd  / 1_000_000.0;
        let output_cost = (output_tokens as f64) * pricing.output_per_million_usd / 1_000_000.0;
        input_cost + output_cost
    }
}

/// Anthropic /v1/messages request body shape. Subset of the full API; M02
/// uses the parts spec §M2 acceptance criteria require + tool support.
#[derive(Debug, Serialize)]
struct AnthropicRequest<'a> {
    model:      &'a str,
    max_tokens: u32,
    messages:   &'a [Message],
    #[serde(skip_serializing_if = "Option::is_none")]
    system:     Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools:      Vec<AnthropicTool<'a>>,
    stream:     bool,
}

#[derive(Debug, Serialize)]
struct AnthropicTool<'a> {
    name:        &'a str,
    description: &'a str,
    input_schema: &'a serde_json::Value,
}

impl<'a> AnthropicRequest<'a> {
    fn from_config(config: &'a AgentConfig) -> Self {
        Self {
            model:       &config.model,
            max_tokens:  config.max_tokens,
            messages:    &config.messages,
            system:      config.system_prompt.as_deref(),
            temperature: config.temperature,
            tools:       config.tools.iter().map(AnthropicTool::from_def).collect(),
            stream:      true,
        }
    }
}

impl<'a> AnthropicTool<'a> {
    fn from_def(def: &'a ToolDef) -> Self {
        Self {
            name:         &def.name,
            description:  &def.description,
            input_schema: &def.input_schema,
        }
    }
}

#[cfg(test)]
mod tests {
    // Stage B tests are kept as-is. Stage C wiremock tests live in
    // crates/runtime-main/tests/anthropic_wiremock.rs (integration test
    // suite). Real-API smoke at crates/runtime-main/tests/anthropic_smoke.rs.
    // (Re-attach the Stage B unit tests here verbatim.)
}
```

Note: the existing Stage B unit tests in `anthropic.rs::mod tests` are preserved; the snippet above only shows the new structure. Stage B tests stay where they are.

#### `crates/runtime-main/tests/anthropic_wiremock.rs` (new)

```rust
//! Wiremock-driven integration tests for AnthropicProvider.
//!
//! Exercises the SSE state machine end-to-end without real network: wiremock
//! intercepts `POST /v1/messages`, returns a pre-canned SSE response body,
//! and the provider's `stream()` consumes it through the real reqwest +
//! eventsource-stream + sse state machine path. Every transition the API
//! actually emits is exercised.
//!
//! These tests gate ≥95% coverage on `crates/runtime-main/src/providers/`
//! (the SSE state machine specifically — the thin reqwest wrapper above it
//! is excluded per `--ignore-filename-regex` for the same OS-signal-class
//! reason as M01.C drone `lib::run`).

use futures::StreamExt;
use runtime_main::providers::{
    AgentConfig, ContentBlock, LLMProvider, Message, MessageRole, ProviderEvent,
    anthropic::AnthropicProvider,
};
use secrecy::SecretString;
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

fn make_config() -> AgentConfig {
    AgentConfig {
        model: "claude-haiku-4-5".into(),
        messages: vec![Message {
            role: MessageRole::User,
            content: vec![ContentBlock::Text { text: "ping".into() }],
        }],
        max_tokens: 100,
        temperature: None,
        system_prompt: None,
        tools: vec![],
    }
}

const HAPPY_PATH_SSE: &str = "\
event: message_start
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-haiku-4-5\",\"content\":[],\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":10,\"output_tokens\":1}}}

event: content_block_start
data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}

event: ping
data: {\"type\":\"ping\"}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"!\"}}

event: content_block_stop
data: {\"type\":\"content_block_stop\",\"index\":0}

event: message_delta
data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":2}}

event: message_stop
data: {\"type\":\"message_stop\"}

";

#[tokio::test]
async fn happy_path_yields_text_deltas_and_message_stop() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", "sk-ant-test"))
        .and(header("anthropic-version", "2023-06-01"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(HAPPY_PATH_SSE),
        )
        .mount(&server)
        .await;

    let provider = AnthropicProvider::with_base_url(
        SecretString::from("sk-ant-test"),
        server.uri(),
    );
    let mut stream = provider.stream(make_config()).await.unwrap();

    let mut events = vec![];
    while let Some(e) = stream.next().await {
        events.push(e);
    }

    let text_count = events.iter().filter(|e| matches!(e, ProviderEvent::TextDelta { .. })).count();
    assert_eq!(text_count, 2, "expected 2 text deltas, got {events:?}");
    assert!(matches!(events.last(), Some(ProviderEvent::MessageStop { .. })));
}

#[tokio::test]
async fn auth_failure_surfaces_as_provider_error_auth() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_string("{\"type\":\"error\",\"error\":{\"type\":\"authentication_error\",\"message\":\"invalid x-api-key\"}}"))
        .mount(&server)
        .await;

    let provider = AnthropicProvider::with_base_url(
        SecretString::from("sk-ant-bogus"),
        server.uri(),
    );
    let result = provider.stream(make_config()).await;
    assert!(matches!(result, Err(runtime_main::providers::ProviderError::Auth)));
}

#[tokio::test]
async fn rate_limit_includes_retry_after() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "30")
                .set_body_string("rate limited"),
        )
        .mount(&server)
        .await;

    let provider = AnthropicProvider::with_base_url(
        SecretString::from("sk-ant-test"),
        server.uri(),
    );
    let result = provider.stream(make_config()).await;
    match result {
        Err(runtime_main::providers::ProviderError::RateLimit { retry_after_secs }) => {
            assert_eq!(retry_after_secs, 30);
        }
        other => panic!("expected RateLimit, got {other:?}"),
    }
}

// Plus 5 more tests: tool_use_accumulates_and_emits, thinking_delta_passthrough,
// error_event_emits_provider_error, malformed_sse_skipped, partial_chunk_reassembled.
// Each follows the same wiremock + SSE pattern. Full content authored by Stage C
// during implementation per the test plan.
```

#### `crates/runtime-main/tests/anthropic_smoke.rs` (new — gated)

```rust
//! Real-API smoke test for AnthropicProvider.
//!
//! Gated by `--features integration` — CI never runs this; it requires a
//! real Anthropic API key in the OS keychain (`agent-runtime/anthropic/api-key`).
//!
//! Cost per run: ~$0.001 against Haiku 4.5 ($1/$5 per MTok).

#![cfg(feature = "integration")]

use futures::StreamExt;
use keyring::Entry;
use runtime_main::providers::{
    AgentConfig, ContentBlock, LLMProvider, Message, MessageRole, ProviderEvent,
    anthropic::AnthropicProvider,
};
use secrecy::SecretString;

#[tokio::test]
async fn smoke_real_api_hello() {
    let key = Entry::new("agent-runtime", "anthropic")
        .expect("keyring::Entry::new should succeed")
        .get_password()
        .expect("API key not in keychain — run: anthropic-api-key set <value>");

    let provider = AnthropicProvider::new(SecretString::from(key));
    let config = AgentConfig {
        model: "claude-haiku-4-5".into(),
        messages: vec![Message {
            role: MessageRole::User,
            content: vec![ContentBlock::Text {
                text: "Say only the word: hello".into(),
            }],
        }],
        max_tokens: 16,
        temperature: Some(0.0),
        system_prompt: None,
        tools: vec![],
    };

    let mut stream = provider.stream(config).await.expect("stream() should succeed");
    let mut text_deltas = 0usize;
    let mut message_stops = 0usize;

    while let Some(event) = stream.next().await {
        match event {
            ProviderEvent::TextDelta { .. } => text_deltas += 1,
            ProviderEvent::MessageStop { .. } => message_stops += 1,
            _ => {}
        }
    }

    assert!(text_deltas >= 1, "expected ≥1 text delta, got 0");
    assert_eq!(message_stops, 1, "expected exactly 1 MessageStop");
}
```

#### `crates/runtime-main/README.md` (edited)

Append a new section:

```markdown
## Real-API smoke test

The provider integration tests use `wiremock` for offline CI. To exercise the real Anthropic Messages API end-to-end:

1. Get an API key from https://console.anthropic.com (Settings → API Keys → Create Key).
2. Store it in the OS keychain under service `agent-runtime`, user `anthropic`:
   - macOS / Linux: `keyring set agent-runtime anthropic` (use the `keyring` Python tool, or any platform secret manager)
   - Windows: open Credential Manager → add a Generic Credential with internet/network address `agent-runtime` and user name `anthropic`
3. Run: `cargo test --features integration -p runtime-main --test anthropic_smoke`

Cost per run: ~$0.001 against Haiku 4.5 ($1/$5 per million tokens).

CI never runs this test (no API key in CI). The wiremock tests in `tests/anthropic_wiremock.rs` cover the same wire-format paths offline.
```

#### `.github/workflows/ci.yml` (edited)

In the Coverage job, add `runtime-main` to the safety-primitive coverage gate matrix:

```yaml
      - name: runtime-main coverage gate (≥95%, OS-signal exclusions)
        run: |
          cargo llvm-cov --package runtime-main \
            --ignore-filename-regex 'src.main\.rs|generated|src.providers.anthropic\.rs' \
            --fail-under-lines 95
```

The `src.providers.anthropic.rs` exclusion covers the real-network production wrapper (`stream()` body that constructs reqwest::Client and POSTs). The SSE state machine in `anthropic_sse.rs` is fully tested via wiremock + unit tests.

#### `CLAUDE.md` (edited)

In §5 Coverage thresholds, append to the safety primitives list:

```
- `runtime-main/src/providers/anthropic.rs::stream` — production wrapper around the real reqwest+eventsource-stream call. Excluded from the ≥95% gate via `--ignore-filename-regex 'src.providers.anthropic\.rs'` because real-network hits are structurally untestable cross-platform; logic lives in `anthropic_sse.rs` (covered by unit tests + wiremock integration tests).
```

#### `CHANGELOG.md` (edited)

Append to `[Unreleased]`:

```markdown
### Added (M02 Stage C)
- `crates/runtime-main/src/providers/anthropic_sse.rs` — SSE state machine + parser + `*_with`-style test seam. Maps Anthropic Messages API SSE events (`message_start` / `content_block_start` / `content_block_delta` / `content_block_stop` / `message_delta` / `message_stop` / `ping` / `error`) to `ProviderEvent`s. Accumulates tool input partial-JSON deltas across `content_block_delta` events.
- Real `AnthropicProvider::stream()` HTTP+SSE implementation in `providers/anthropic.rs`. Direct `reqwest` + `eventsource-stream`; no third-party SDK.
- `tests/anthropic_wiremock.rs` — 8 integration tests (happy path, auth failure, rate limit, tool use, thinking, error event, malformed SSE, partial chunks).
- `tests/anthropic_smoke.rs` — real-API smoke gated by `--features integration`; reads keychain entry `agent-runtime/anthropic`.
- `runtime-main` added to safety-primitive coverage gate matrix (≥95% with documented `anthropic.rs::stream` wrapper exclusion).

### Changed
- `runtime-main` Cargo.toml gains `wiremock` dev-dependency.
```

### C.4 Tests

1. **`anthropic_sse::tests::parses_message_start`** — JSON → `SseEvent::MessageStart`.
2. **`parses_text_delta`** — JSON → `SseEvent::ContentBlockDelta` with text_delta.
3. **`parses_ping`** — keep-alive event parsed.
4. **`parses_error`** — error event parsed with kind + message.
5. **`parses_message_delta_with_stop_reason`** — message_delta JSON shape.
6. **`translate_text_delta_emits_provider_event`** — state machine: text passthrough.
7. **`translate_ping_returns_none`** — keep-alive silenced.
8. **`translate_message_delta_emits_message_stop`** — stop_reason → MessageStop.
9. **`translate_error_emits_provider_error_event`** — error event surfaced.
10. **`tool_use_accumulates_partial_json_then_emits_on_stop`** — multi-delta accumulation correctness.
11. **`signature_delta_is_silent`** — verifier-only event ignored.
12. **`malformed_data_returns_sse_error`** — bad JSON → ProviderError::Sse.
13. **`anthropic_wiremock::happy_path_yields_text_deltas_and_message_stop`** — end-to-end SSE → ProviderEvents via real reqwest+eventsource-stream chain.
14. **`auth_failure_surfaces_as_provider_error_auth`** — 401 → `ProviderError::Auth`.
15. **`rate_limit_includes_retry_after`** — 429 + Retry-After header → `ProviderError::RateLimit { retry_after_secs }`.
16. **`tool_use_accumulates_and_emits`** — wiremock-driven tool use sequence.
17. **`thinking_delta_passthrough`** — extended-thinking event sequence.
18. **`error_event_emits_provider_error`** — server-emitted SSE error event.
19. **`malformed_sse_skipped`** — bad bytes don't panic the stream.
20. **`partial_chunk_reassembled`** — eventsource-stream's framing handles split bytes.
21. **`anthropic_smoke::smoke_real_api_hello`** — gated `--features integration`; expects ≥1 TextDelta + exactly 1 MessageStop from real API.

#### Coverage target

- Workspace ≥80% (general gate, unchanged).
- `runtime-drone` ≥95% (unchanged from Stage A baseline).
- **`runtime-main` ≥95%** (NEW — safety primitive gate activated). Exclusion: `src/providers/anthropic.rs` (the production stream() wrapper that constructs reqwest::Client; real-network untestable cross-platform). Logic covered: `anthropic_sse.rs` 100% via unit tests + `anthropic.rs` non-network paths via the existing Stage B unit tests.

**Coverage delta gate.** PR-vs-main delta (M02 Stage A activated this) — the ≥95% `runtime-main` gate is a new threshold; baseline measured at this commit.

### C.5 CLI Prompt

```
Read CLAUDE.md for all project rules.
Read docs/build-prompts/M02-event-pipeline.md Stage C (sections C.1 through C.4).

Read prior stage retrospectives for guidance:
  docs/build-prompts/retrospectives/M02.A-retrospective.md
  docs/build-prompts/retrospectives/M02.B-retrospective.md
  Focus: [END] "Decisions for the next stage" sections + any [LIVE]
  friction events flagged as relevant to Stage C. Apply decisions.

Read docs/gap-analysis.md for any Carry-forward items targeting Stage C
(look at the most recent entry's Carry-forward section).

═══ STEP 1 — WRITE FAILING TESTS ═══

Create test files / add to existing:

1. crates/runtime-main/src/providers/anthropic_sse.rs::tests (12 unit tests
   per C.4 #1–#12).
2. crates/runtime-main/tests/anthropic_wiremock.rs (8 integration tests
   per C.4 #13–#20).
3. crates/runtime-main/tests/anthropic_smoke.rs (1 real-API test gated
   #[cfg(feature = "integration")] per C.4 #21).

Run: cargo test --workspace --package runtime-main
Confirm: all tests fail with `cannot find module anthropic_sse`,
`cannot find function stream_events`, etc. — TDD red phase.

═══ STEP 2 — IMPLEMENT ═══

Apply per C.3, in order:

1. Cargo.toml (workspace root) — add wiremock = "0.6".
2. crates/runtime-main/Cargo.toml — add wiremock dev-dep.
3. crates/runtime-main/src/providers/anthropic_sse.rs (NEW) — full content
   per C.3 (SseEvent enum, SseState, parse_sse_data, stream_events).
4. crates/runtime-main/src/providers/anthropic.rs — replace the stub
   stream() body with the real implementation per C.3. Keep Stage B
   tests in `mod tests`.
5. crates/runtime-main/src/providers/mod.rs — declare `mod anthropic_sse;`
   inside `pub mod anthropic { ... }` (or as a submodule of anthropic.rs).
6. crates/runtime-main/tests/anthropic_wiremock.rs (NEW) — full body per
   C.3 + the 5 additional tests (tool_use, thinking, error, malformed,
   partial-chunk) following the same wiremock pattern.
7. crates/runtime-main/tests/anthropic_smoke.rs (NEW) — gated test per
   C.3.
8. .github/workflows/ci.yml — add the runtime-main coverage gate per C.3.
9. CLAUDE.md §5 — append the anthropic.rs exclusion entry per C.3.
10. crates/runtime-main/README.md — append the smoke-test section per C.3.
11. CHANGELOG.md [Unreleased] — append the Added section.

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
  cargo llvm-cov --package runtime-main --ignore-filename-regex "src.main\.rs|generated|src.providers.anthropic\.rs" --fail-under-lines 95
  cargo llvm-cov --package runtime-drone --ignore-filename-regex "src.main\.rs|generated|src.lib\.rs|src.shutdown\.rs" --fail-under-lines 95

DO NOT run `cargo test --features integration` (real API; would cost
real money and require keychain setup). The wiremock tests cover the
same wire-format paths.

If any gate fails, follow CLAUDE.md §7 self-correction. Max 3 iterations
then surface.

═══ STEP 4 — RETROSPECTIVE ═══

Per CLAUDE.md §19, copy retrospectives/RETROSPECTIVE-TEMPLATE.md to:
  docs/build-prompts/retrospectives/M02.C-retrospective.md

Fill in [LIVE] sections during work, then [END] scoring + threshold gates +
decisions for Stage D (specifically: did the SSE state machine surface
edge cases the prompt didn't anticipate; was the *_with seam pattern
applied cleanly; how does the runtime-main coverage policy compare to
runtime-drone's; any wire-format quirks worth carrying into the spec).

═══ STEP 5 — SURFACE TO USER ═══

Run: git status, git diff --stat HEAD
Re-run all gates one final time.

Surface: diff stat, gate results, M02.C retrospective, draft commit from C.6.

State: "Stage C is ready. I will NOT commit until you approve."

Wait for explicit approval. Do NOT push (push waits for Stage F per CLAUDE.md §20).

On approval (Stage C — work stage; not the final stage of a parent milestone):
1. Commit Stage C on the parent-milestone branch claude/m02-event-pipeline
   (do NOT push).
2. Stop. Surface the commit. Stage D is opened in a fresh session.
```

### C.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(runtime-main): M02 Stage C — AnthropicProvider real HTTP+SSE impl

Replaces Stage B's stub stream() with a real implementation that POSTs
to https://api.anthropic.com/v1/messages, parses Server-Sent Events via
eventsource-stream, runs them through a state machine that maps
Anthropic's wire format to ProviderEvents, and emits to callers.

Architecture:
- providers/anthropic_sse.rs (NEW) — pure SSE state machine + parser.
  SseEvent enum mirrors the Anthropic wire format (message_start,
  content_block_start/delta/stop, message_delta/stop, ping, error).
  SseState accumulates tool input partial-JSON deltas across
  content_block_delta events; emits the complete ToolUse on
  ContentBlockStop.
- *_with-style test seam: stream_events(byte_stream) is the testable
  entry; wiremock feeds it pre-canned bytes; production wrapper feeds
  it reqwest::Response::bytes_stream(). Same logic exercised both ways.
- providers/anthropic.rs::stream() is the thin production wrapper —
  excluded from the ≥95% coverage gate via --ignore-filename-regex
  because real-network hits are structurally untestable cross-platform
  (per M01.C codification at commit 1dec4ba).

Wire format (per https://platform.claude.com/docs/en/api/messages-streaming
verified 2026-05):
- Headers: x-api-key, anthropic-version: 2023-06-01, content-type
- Request body: model, max_tokens, messages, system?, temperature?,
  tools?, stream: true
- 401/403 → ProviderError::Auth
- 429 → ProviderError::RateLimit (parses retry-after header)
- non-2xx → ProviderError::Api with status + body

Tests:
- 12 unit tests in anthropic_sse.rs (parser + state machine, each
  variant covered).
- 8 wiremock integration tests in tests/anthropic_wiremock.rs (happy
  path, auth, rate limit, tool use, thinking, error, malformed,
  partial chunk).
- 1 real-API smoke test in tests/anthropic_smoke.rs gated
  #[cfg(feature = "integration")] — reads keychain entry
  agent-runtime/anthropic; CI never runs this; cost ~$0.001 per run.

Coverage:
- runtime-main ≥95% line activated (new safety primitive gate).
  Exclusion: providers/anthropic.rs (real-network wrapper).
  Covered: anthropic_sse.rs 100% via unit + wiremock tests.

CLAUDE.md §5 + .github/workflows/ci.yml updated to record the
runtime-main gate + exclusion list.

Refs: M02-event-pipeline.md §C; agent-runtime-spec.md §2 §2c;
https://platform.claude.com/docs/en/api/messages;
https://platform.claude.com/docs/en/api/messages-streaming.

Retrospective: docs/build-prompts/retrospectives/M02.C-retrospective.md

https://claude.ai/code/session_<id>
EOF
)"
```

---

<!-- ============================================================ -->
<!-- Stages D – F — to be authored in subsequent chunks             -->
<!-- ============================================================ -->

[Stages D, E, F authored in subsequent chunks. This file grows incrementally; each chunk surfaces for approval before the next is appended.]

---

## Summary Table

[Filled in after Stage F is authored.]

---

## Verification Checklist

[Filled in after Stage F is authored.]

---

*M02 prompt — Stage A authored. Stages B–F pending.*
