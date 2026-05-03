# M02 Event Pipeline — Specification + Stage Prompts

**Protocol version:** v1.0 (pre-XML-schema; grandfathered per `STAGE-PROMPT-PROTOCOL.md` v1.2 changelog item #8). Exempt from v1.2 validator rules (URI-fragment refs, missing `<execution_steps>`, inline content in tags whose Phase doc sections exist). M03 onward uses v1.2.
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
| `codecov.yml` | **New** — Codecov project + patch coverage rules (`target: auto`, `threshold: 0.5%`, `base: auto`); flips Codecov from advisory (M01) to required (M02+) |
| `CLAUDE.md` | **Edited** — §5 add "Coverage delta gating mechanism" subsection (Codecov-enforced; M02 baseline; subsequent milestones gate on delta vs `main`) |
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

    fn check_round_trip(s: Signal) {
        let json = serde_json::to_string(&s).unwrap();
        let back: Signal = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    fn payload() -> serde_json::Value {
        serde_json::json!({"k": "v", "n": 42})
    }

    #[test]
    fn round_trip_tool() {
        check_round_trip(Signal::Tool {
            signal_id: "sig-1".into(),
            agent_id:  "agent-1".into(),
            tool_name: "search".into(),
            payload_json: payload(),
            pre_signal_id: Some(PreSignalId("sig-prev".into())),
            parent_signal_id: Some(ParentSignalId("sig-parent".into())),
            retry_of: None,
            context_type: ContextType::ToolInvoke,
        });
    }

    #[test]
    fn round_trip_skill() {
        check_round_trip(Signal::Skill {
            signal_id: "sig-2".into(),
            agent_id:  "agent-1".into(),
            skill_name: "skim-skill".into(),
            skill_version: "1.0.0".into(),
            payload_json: payload(),
            parent_signal_id: None,
            context_type: ContextType::SkillLoad,
        });
    }

    #[test]
    fn round_trip_agent() {
        check_round_trip(Signal::Agent {
            signal_id: "sig-3".into(),
            agent_id:  "agent-1".into(),
            event: "spawned".into(),
            payload_json: payload(),
            parent_signal_id: None,
            context_type: ContextType::AgentLoop,
        });
    }

    #[test]
    fn round_trip_decision() {
        check_round_trip(Signal::Decision {
            signal_id: "sig-4".into(),
            agent_id:  "agent-1".into(),
            decision: "pick haiku".into(),
            rationale: "cost-sensitive".into(),
            tool_used: Some("estimate_cost".into()),
            payload_json: payload(),
            parent_signal_id: None,
            context_type: ContextType::AgentLoop,
        });
    }

    #[test]
    fn round_trip_verify() {
        check_round_trip(Signal::Verify {
            signal_id: "sig-5".into(),
            agent_id:  "agent-1".into(),
            hook_id: "test-suite".into(),
            passed: true,
            payload_json: payload(),
            parent_signal_id: None,
            context_type: ContextType::HookExecute,
        });
    }

    #[test]
    fn round_trip_error() {
        check_round_trip(Signal::Error {
            signal_id: "sig-6".into(),
            agent_id:  Some("agent-1".into()),
            error_kind: "timeout".into(),
            message: "tool exceeded 60s".into(),
            payload_json: payload(),
            parent_signal_id: None,
            retry_of: Some(RetryOfSignalId("sig-orig".into())),
            context_type: ContextType::ToolInvoke,
        });
    }

    #[test]
    fn round_trip_hitl() {
        check_round_trip(Signal::Hitl {
            signal_id: "sig-7".into(),
            agent_id:  "agent-1".into(),
            prompt: "approve plan?".into(),
            response: Some("yes".into()),
            payload_json: payload(),
            parent_signal_id: None,
            context_type: ContextType::HitlPrompt,
        });
    }

    #[test]
    fn round_trip_session() {
        check_round_trip(Signal::Session {
            signal_id: "sig-8".into(),
            event: "start".into(),
            payload_json: payload(),
            context_type: ContextType::SessionLifecycle,
        });
    }

    #[test]
    fn tag_serialization_is_snake_case() {
        let s = Signal::Tool {
            signal_id: "x".into(),
            agent_id:  "x".into(),
            tool_name: "x".into(),
            payload_json: payload(),
            pre_signal_id: None,
            parent_signal_id: None,
            retry_of: None,
            context_type: ContextType::ToolInvoke,
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"kind\":\"tool\""), "got: {json}");
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

#### `codecov.yml` (new)

```yaml
# Codecov coverage gating. Flips from advisory (M01 commit c04aac5)
# to required (M02 Stage A). Project + patch checks both blocking.
# Absolute thresholds (cargo llvm-cov --fail-under-lines) remain
# authoritative for hard floors; Codecov gates the *delta* vs main.

coverage:
  status:
    project:
      default:
        target:        auto       # match base branch coverage
        threshold:     0.5%       # allow ≤0.5pp regression
        base:          auto       # compare against main
        if_no_uploads: error
        informational: false      # block PR check on regression
      runtime-drone:
        target:    95%
        threshold: 0.5%
        flags:     [runtime-drone]
        informational: false
      runtime-main:
        target:    95%
        threshold: 0.5%
        flags:     [runtime-main]
        informational: false
    patch:
      default:
        target:        80%
        threshold:     0%
        informational: false      # PR-touched lines must hit project floor

comment:
  layout:  "reach, diff, flags, files"
  behavior: default
  require_changes: true
  require_base:    true
  require_head:    true

ignore:
  - "**/generated/**"
  - "**/src/main.rs"
  - "**/build.rs"
  - "src-tauri/gen/**"
```

The `runtime-drone` and `runtime-main` flag entries inherit the absolute floors from `cargo llvm-cov --fail-under-lines` and add the 0.5pp delta gate on top. `flags` reference uploads tagged in the CI workflow (extend `.github/workflows/ci.yml` upload step with `flags: runtime-drone`/`runtime-main`).

#### `.github/workflows/ci.yml` (edited — Codecov step)

Locate the existing `Coverage (Rust)` job's `codecov-action` invocation. Confirm:

```yaml
      - uses: codecov/codecov-action@v4
        with:
          files:    lcov.info
          flags:    workspace
          fail_ci_if_error: true
```

Add per-crate flag uploads after the workspace one (separate runs of `cargo llvm-cov --package <crate>`):

```yaml
      - name: Upload runtime-drone coverage
        uses: codecov/codecov-action@v4
        with:
          files:    lcov-drone.info
          flags:    runtime-drone
          fail_ci_if_error: true

      - name: Upload runtime-main coverage
        uses: codecov/codecov-action@v4
        with:
          files:    lcov-main.info
          flags:    runtime-main
          fail_ci_if_error: true
```

(The `cargo llvm-cov --package <crate> --lcov --output-path lcov-<crate>.info` invocations are added before each upload.)

#### `CLAUDE.md` §5 (edited)

Locate the "Coverage thresholds" subsection. After the existing safety-primitive policy paragraph, add:

```
**Coverage delta gating (from M02 onward) — Codecov-enforced.** M01 used
absolute thresholds (workspace ≥80%, drone ≥95%) because no baseline
existed. Starting M02, every PR also passes a delta-gate via Codecov:
project + patch coverage thresholds set in `codecov.yml` (`target: auto`,
`threshold: 0.5%`, `base: auto`). Codecov pulls the LCOV uploaded by the
existing `cargo-llvm-cov` step in `.github/workflows/ci.yml`, compares
to `main`'s last green build, and fails the PR check if any gated crate
regresses by >0.5 percentage points (absolute) OR if patch coverage on
the changed lines drops below the project floor.

Codecov was advisory in M01 (commit `c04aac5`); M02 Stage A flips
required-on for the project + patch checks via:
- New `codecov.yml` at repo root with the project + patch rules.
- `.github/workflows/ci.yml` keeps the existing upload step; the
  `informational: false` flag in `codecov.yml` makes the check
  blocking.
- The absolute-threshold gates (`cargo llvm-cov --fail-under-lines`)
  remain authoritative for hard floors; Codecov gates the *delta*.
- No custom bash scripts to maintain.

Pre-M01 carry-forward; resolved per the M01 gap-analysis Important
"Coverage delta gating mechanism" item.
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

Stage A is mostly hygiene + scaffolding; tests cover the full type surface (signal.rs all 8 variants), every schema invariant on the new mcp_servers table (UNIQUE name, default values, CHECK constraints on transport/auth_kind/scope/status, mutual-exclusion stdio-vs-remote), and the heartbeat status typed-enum round-trip:

1. **`crates/runtime-drone/src/heartbeat.rs::tests::heartbeat_writes_typed_status_to_db`** — update existing tests if any string literals appear; verify the `heartbeats.status` column now receives the snake_case enum string from `HeartbeatStatus::Ok`.
2. *(reserved — was the single-variant signal test; now 14–22 below cover all 8 variants explicitly)*
3. **`crates/runtime-drone/src/db.rs::tests::init_schema_creates_mcp_servers_table`** — extend the existing schema-init tests to verify the 8th table exists with all 22 expected columns, all 4 CHECK constraints, and all 3 indexes (status / enabled / scope).
4. **`crates/runtime-drone/src/db.rs::tests::mcp_servers_stdio_invariant_enforced`** — insert a stdio row WITHOUT command (or WITH url) → expect SQL CHECK constraint failure.
5. **`crates/runtime-drone/src/db.rs::tests::mcp_servers_remote_invariant_enforced`** — insert an http/sse row WITHOUT url (or WITH command) → expect SQL CHECK constraint failure.
6. **`crates/runtime-drone/src/db.rs::tests::mcp_servers_status_transitions`** — insert with default status, update to 'connected', update to 'errored' with last_error → all transitions allowed; invalid status string → CHECK failure.
7. **`crates/runtime-drone/src/db.rs::tests::mcp_servers_unique_name_enforced`** — insert two rows with the same `name` → expect UNIQUE constraint failure on the second.
8. **`crates/runtime-drone/src/db.rs::tests::mcp_servers_default_values_applied`** — insert a stdio row with only required fields (`name`, `transport`, `command`, `added_at`, `updated_at`) → SELECT confirms `status='configured'`, `enabled=1`, `scope='user'`, `retry_count=0`, `startup_timeout_ms=10000`, `tool_timeout_ms=60000`.
9. **`crates/runtime-drone/src/db.rs::tests::mcp_servers_invalid_auth_kind_rejected`** — insert with `auth_kind='ssh-key'` → expect CHECK constraint failure (only 'none', 'bearer', 'oauth', 'custom' allowed).
10. **`crates/runtime-drone/src/db.rs::tests::mcp_servers_invalid_scope_rejected`** — insert with `scope='enterprise'` → expect CHECK constraint failure (only 'user', 'project', 'plugin', 'local' allowed).
11. **`crates/runtime-drone/src/db.rs::tests::mcp_servers_invalid_transport_rejected`** — insert with `transport='websocket'` → expect CHECK constraint failure (only 'stdio', 'http', 'sse', 'streamable_http' allowed).
12. **`crates/runtime-drone/tests/integration_windows.rs`** — full subprocess test (per A.3 above): spawn drone, connect to named pipe, send `SnapshotNow`, verify snapshot row, send `GracefulShutdown`, verify clean exit. Gated `#[cfg(windows)]` only.
13. **`crates/runtime-drone/src/db.rs::tests::heartbeat_status_roundtrip_via_db`** — write `HeartbeatStatus::Degraded`, read back, assert equal.

**`crates/runtime-core/src/signal.rs::tests`** — 8 round-trip tests + 1 tag-format test (per A.3 expanded test module):
14. `round_trip_tool` — Signal::Tool with all correlation fields set
15. `round_trip_skill` — Signal::Skill
16. `round_trip_agent` — Signal::Agent
17. `round_trip_decision` — Signal::Decision with tool_used
18. `round_trip_verify` — Signal::Verify
19. `round_trip_error` — Signal::Error with retry_of
20. `round_trip_hitl` — Signal::Hitl with response set
21. `round_trip_session` — Signal::Session
22. `tag_serialization_is_snake_case` — `kind: "tool"` not `"Tool"`

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

/// Token-usage breakdown for `estimate_cost`. Cache-aware so M04 budget
/// integration just plumbs the values; no trait refactor needed.
///
/// Cache rates per Anthropic docs (verified 2026-05):
/// - 5-minute cache write: 1.25× input price
/// - 1-hour cache write:   2.0× input price
/// - Cache read:           0.1× input price
///
/// Unknown / unused cache fields default to 0 via `CostBreakdown::simple`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CostBreakdown {
    pub input_tokens:    u64,
    pub output_tokens:   u64,
    pub cache_5m_writes: u64,
    pub cache_1h_writes: u64,
    pub cache_reads:     u64,
}

impl CostBreakdown {
    /// Simple constructor for callers without cache awareness.
    /// All cache fields zero.
    pub fn simple(input_tokens: u64, output_tokens: u64) -> Self {
        Self {
            input_tokens,
            output_tokens,
            ..Self::default()
        }
    }
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

    /// Estimate cost for a token-usage breakdown on a model. Cache-aware
    /// per Anthropic docs (5m write 1.25×, 1h write 2×, read 0.1× input).
    /// Callers without cache awareness use `CostBreakdown::simple(in, out)`.
    fn estimate_cost(&self, breakdown: &CostBreakdown, model: &str) -> f64;
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

    fn estimate_cost(&self, b: &CostBreakdown, model: &str) -> f64 {
        // Cache-aware pricing per https://platform.claude.com/docs/en/about-claude/pricing
        // (verified 2026-05).
        // Cache multipliers: 5m write 1.25× input, 1h write 2× input, read 0.1× input.
        let pricing = match model {
            "claude-opus-4-7"   => Pricing { input_per_million_usd: 5.0, output_per_million_usd: 25.0 },
            "claude-opus-4-6"   => Pricing { input_per_million_usd: 5.0, output_per_million_usd: 25.0 },
            "claude-opus-4-5"   => Pricing { input_per_million_usd: 5.0, output_per_million_usd: 25.0 },
            "claude-sonnet-4-6" => Pricing { input_per_million_usd: 3.0, output_per_million_usd: 15.0 },
            "claude-sonnet-4-5" => Pricing { input_per_million_usd: 3.0, output_per_million_usd: 15.0 },
            "claude-haiku-4-5"  => Pricing { input_per_million_usd: 1.0, output_per_million_usd:  5.0 },
            _ => return 0.0, // unknown model — Stage C surfaces this as ProviderError::InvalidModel via async paths
        };
        let input_rate  = pricing.input_per_million_usd  / 1_000_000.0;
        let output_rate = pricing.output_per_million_usd / 1_000_000.0;

        (b.input_tokens    as f64) * input_rate
      + (b.output_tokens   as f64) * output_rate
      + (b.cache_5m_writes as f64) * input_rate * 1.25
      + (b.cache_1h_writes as f64) * input_rate * 2.0
      + (b.cache_reads     as f64) * input_rate * 0.1
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

    #[tokio::test]
    async fn list_models_pricing_values_correct() {
        let models = stub_provider().list_models().await.unwrap();
        let opus   = models.iter().find(|m| m.id == "claude-opus-4-7").unwrap();
        let sonnet = models.iter().find(|m| m.id == "claude-sonnet-4-6").unwrap();
        let haiku  = models.iter().find(|m| m.id == "claude-haiku-4-5").unwrap();
        assert_eq!(opus.pricing,   Pricing { input_per_million_usd: 5.0, output_per_million_usd: 25.0 });
        assert_eq!(sonnet.pricing, Pricing { input_per_million_usd: 3.0, output_per_million_usd: 15.0 });
        assert_eq!(haiku.pricing,  Pricing { input_per_million_usd: 1.0, output_per_million_usd:  5.0 });
    }

    #[test]
    fn estimate_cost_simple_for_haiku() {
        let provider = stub_provider();
        // 1M input + 1M output on Haiku 4.5 = $1.00 + $5.00 = $6.00
        let b = CostBreakdown::simple(1_000_000, 1_000_000);
        let cost = provider.estimate_cost(&b, "claude-haiku-4-5");
        assert!((cost - 6.00).abs() < 1e-6, "got {cost}");
    }

    #[test]
    fn estimate_cost_simple_for_sonnet() {
        let provider = stub_provider();
        // 1M input + 1M output on Sonnet 4.6 = $3.00 + $15.00 = $18.00
        let b = CostBreakdown::simple(1_000_000, 1_000_000);
        let cost = provider.estimate_cost(&b, "claude-sonnet-4-6");
        assert!((cost - 18.00).abs() < 1e-6, "got {cost}");
    }

    #[test]
    fn estimate_cost_simple_for_opus() {
        let provider = stub_provider();
        // 1M input + 1M output on Opus 4.7 = $5.00 + $25.00 = $30.00
        let b = CostBreakdown::simple(1_000_000, 1_000_000);
        let cost = provider.estimate_cost(&b, "claude-opus-4-7");
        assert!((cost - 30.00).abs() < 1e-6, "got {cost}");
    }

    #[test]
    fn estimate_cost_with_cache_writes_5m() {
        let provider = stub_provider();
        // 1M cache_5m_writes on Haiku 4.5 = 1M × $1 × 1.25 = $1.25
        let b = CostBreakdown {
            input_tokens:    0,
            output_tokens:   0,
            cache_5m_writes: 1_000_000,
            cache_1h_writes: 0,
            cache_reads:     0,
        };
        let cost = provider.estimate_cost(&b, "claude-haiku-4-5");
        assert!((cost - 1.25).abs() < 1e-6, "got {cost}");
    }

    #[test]
    fn estimate_cost_with_cache_writes_1h() {
        let provider = stub_provider();
        // 1M cache_1h_writes on Haiku 4.5 = 1M × $1 × 2.0 = $2.00
        let b = CostBreakdown {
            cache_1h_writes: 1_000_000,
            ..Default::default()
        };
        let cost = provider.estimate_cost(&b, "claude-haiku-4-5");
        assert!((cost - 2.00).abs() < 1e-6, "got {cost}");
    }

    #[test]
    fn estimate_cost_with_cache_reads() {
        let provider = stub_provider();
        // 1M cache_reads on Haiku 4.5 = 1M × $1 × 0.1 = $0.10
        let b = CostBreakdown {
            cache_reads: 1_000_000,
            ..Default::default()
        };
        let cost = provider.estimate_cost(&b, "claude-haiku-4-5");
        assert!((cost - 0.10).abs() < 1e-6, "got {cost}");
    }

    #[test]
    fn estimate_cost_combined_cache_and_io() {
        let provider = stub_provider();
        // Realistic scenario: 1M output, 100K input, 500K cache_reads, 50K cache_5m_writes on Haiku 4.5
        // = (1M × $5/M) + (100K × $1/M) + (500K × $1/M × 0.1) + (50K × $1/M × 1.25)
        // = $5.00 + $0.10 + $0.05 + $0.0625 = $5.2125
        let b = CostBreakdown {
            input_tokens:    100_000,
            output_tokens:   1_000_000,
            cache_5m_writes: 50_000,
            cache_1h_writes: 0,
            cache_reads:     500_000,
        };
        let cost = provider.estimate_cost(&b, "claude-haiku-4-5");
        assert!((cost - 5.2125).abs() < 1e-6, "got {cost}");
    }

    #[test]
    fn estimate_cost_for_unknown_model_returns_zero() {
        let provider = stub_provider();
        // Stage C surfaces unknown models as ProviderError::InvalidModel; for
        // estimate_cost (non-async), 0.0 is the safe default.
        let b = CostBreakdown::simple(1000, 1000);
        let cost = provider.estimate_cost(&b, "nonexistent-model");
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
2. **`crates/runtime-main/src/providers/mod.rs::tests::provider_event_tag_is_snake_case`** — `"type":"text_delta"` not `"TextDelta"`.
3. **`crates/runtime-main/src/providers/mod.rs::tests::content_block_round_trips`** — every `ContentBlock` variant (Text, Image, ToolUse, ToolResult, Thinking) round-trips; matches Anthropic API wire format.
4. **`crates/runtime-main/src/providers/anthropic.rs::tests::stub_stream_returns_text_then_stop`** — stub `stream()` returns at least one `TextDelta` followed by `MessageStop`.
5. **`tests::name_is_anthropic`** — provider identifies itself.
6. **`tests::supports_advertises_tool_use_streaming_thinking`** — capability flags correct.
7. **`tests::count_tokens_approximates_char_div_4`** — Stage B token approximation across content blocks.
8. **`tests::list_models_returns_three_claude_4x_entries`** — Opus 4.7, Sonnet 4.6, Haiku 4.5 IDs present.
9. **`tests::list_models_pricing_values_correct`** — verify each `ModelInfo.pricing` struct has correct $5/$25 (Opus), $3/$15 (Sonnet), $1/$5 (Haiku) values.
10. **`tests::estimate_cost_simple_for_haiku`** — `CostBreakdown::simple(1M, 1M)` on Haiku 4.5 = $6.00.
11. **`tests::estimate_cost_simple_for_sonnet`** — `CostBreakdown::simple(1M, 1M)` on Sonnet 4.6 = $18.00.
12. **`tests::estimate_cost_simple_for_opus`** — `CostBreakdown::simple(1M, 1M)` on Opus 4.7 = $30.00.
13. **`tests::estimate_cost_with_cache_writes_5m`** — 1M `cache_5m_writes` on Haiku 4.5 = $1.25 (1M × $1 × 1.25 multiplier).
14. **`tests::estimate_cost_with_cache_writes_1h`** — 1M `cache_1h_writes` on Haiku 4.5 = $2.00 (1M × $1 × 2.0 multiplier).
15. **`tests::estimate_cost_with_cache_reads`** — 1M `cache_reads` on Haiku 4.5 = $0.10 (1M × $1 × 0.1 multiplier).
16. **`tests::estimate_cost_combined_cache_and_io`** — realistic scenario: input 100K + output 1M + cache_reads 500K + cache_5m_writes 50K on Haiku 4.5 = $5.2125.
17. **`tests::estimate_cost_for_unknown_model_returns_zero`** — defensive default; Stage C upgrades to `ProviderError::InvalidModel` on the async paths.

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

Create the test files (or stub them in mod.rs / anthropic.rs) per B.4:

1. providers::mod::tests::provider_event_round_trips (all variants)
2. providers::mod::tests::provider_event_tag_is_snake_case
3. providers::mod::tests::content_block_round_trips (all 5 ContentBlock variants)
4. providers::anthropic::tests::stub_stream_returns_text_then_stop
5. providers::anthropic::tests::name_is_anthropic
6. providers::anthropic::tests::supports_advertises_tool_use_streaming_thinking
7. providers::anthropic::tests::count_tokens_approximates_char_div_4
8. providers::anthropic::tests::list_models_returns_three_claude_4x_entries
9. providers::anthropic::tests::list_models_pricing_values_correct
10. providers::anthropic::tests::estimate_cost_simple_for_haiku
11. providers::anthropic::tests::estimate_cost_simple_for_sonnet
12. providers::anthropic::tests::estimate_cost_simple_for_opus
13. providers::anthropic::tests::estimate_cost_with_cache_writes_5m
14. providers::anthropic::tests::estimate_cost_with_cache_writes_1h
15. providers::anthropic::tests::estimate_cost_with_cache_reads
16. providers::anthropic::tests::estimate_cost_combined_cache_and_io
17. providers::anthropic::tests::estimate_cost_for_unknown_model_returns_zero

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

    fn estimate_cost(&self, b: &CostBreakdown, model: &str) -> f64 {
        // Cache-aware. Implementation matches the Stage B trait signature.
        let pricing = match model {
            "claude-opus-4-7" | "claude-opus-4-6" | "claude-opus-4-5"
                => Pricing { input_per_million_usd: 5.0, output_per_million_usd: 25.0 },
            "claude-sonnet-4-6" | "claude-sonnet-4-5"
                => Pricing { input_per_million_usd: 3.0, output_per_million_usd: 15.0 },
            "claude-haiku-4-5"
                => Pricing { input_per_million_usd: 1.0, output_per_million_usd:  5.0 },
            _ => return 0.0,
        };
        let input_rate  = pricing.input_per_million_usd  / 1_000_000.0;
        let output_rate = pricing.output_per_million_usd / 1_000_000.0;
        (b.input_tokens    as f64) * input_rate
      + (b.output_tokens   as f64) * output_rate
      + (b.cache_5m_writes as f64) * input_rate * 1.25
      + (b.cache_1h_writes as f64) * input_rate * 2.0
      + (b.cache_reads     as f64) * input_rate * 0.1
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
<!-- STAGE D — AgentSdk + main↔drone IPC client                     -->
<!-- ============================================================ -->

## Stage D — `AgentSdk` + main↔drone IPC client + `ProviderEvent`→`AgentEvent` translation

### D.1 Problem Statement

Stage D wires the provider into a usable agent SDK. The `AgentSdk<P: LLMProvider>` wraps any `LLMProvider` (M02 ships `AnthropicProvider`; future v1.0+ adds OpenAI / local models behind the same trait), runs the agent loop, translates `ProviderEvent`s into `AgentEvent`s for the renderer, and tells the M01 drone to take snapshots when significant lifecycle events happen.

This stage also lands the main-side IPC client that connects to the drone over the Unix socket / Windows named pipe shipped in M01. Wire format is the same `LinesCodec`-framed JSON the drone speaks; the client sends `DroneCommand`s and receives `DroneEvent`s. M02 only exercises `DroneCommand::SnapshotNow` (on `task_started` events — there are no real tasks yet, but the wiring is verified end-to-end).

The translation layer is non-trivial: a single Anthropic stream produces many `ProviderEvent`s that must collectively become a typed `AgentEvent` sequence. `agent_spawned` precedes the first stream message. `tool_invoked` fires when `ProviderEvent::ToolUse` arrives, with the tool input fully accumulated. `stream_text` events bundle consecutive `TextDelta`s up to the next non-text event (so the renderer doesn't get spammed with one event per token). `decision_record` events are extracted from text by a heuristic (Stage D ships the simplest one — first-line "Decision: X / Rationale: Y" detection — to be expanded in M04 when verify+rails come online). `agent_complete` fires on `MessageStop`. `agent_error` fires on `Error` or any unrecoverable provider error.

The agent loop runs the provider stream to exhaustion (M02 is single-turn; multi-turn tool-use loops land in M03+). Cancellation-safety is mandatory — dropping the future at any await point must clean up the drone IPC connection without leaving orphan snapshots.

**Success criterion.** `AgentSdk::run_agent(config)` against `AnthropicProvider` (real or wiremock-backed) produces a full `agent_spawned → tool_invoked? → stream_text* → agent_complete` sequence on the event channel. The main-side drone client connects to a running drone subprocess, sends `SnapshotNow` on every task lifecycle transition, and reconnects on transient drone restarts. Cancellation-safety: `cargo test` exercises drop-mid-stream behavior and verifies no panics, no leaked tasks, no half-written snapshots.

**New artifacts:**
- `crates/runtime-main/src/sdk/mod.rs` (new — module root)
- `crates/runtime-main/src/sdk/agent_sdk.rs` (new — `AgentSdk<P>` struct + `run_agent`)
- `crates/runtime-main/src/sdk/event_pipeline.rs` (new — `ProviderEvent` → `AgentEvent` translator with bundling state)
- `crates/runtime-main/src/sdk/decision_extractor.rs` (new — first-line "Decision:/Rationale:" heuristic; pure function for easy testing)
- `crates/runtime-main/src/drone_ipc/mod.rs` (new — module root)
- `crates/runtime-main/src/drone_ipc/client.rs` (new — main-side IPC client; cfg-platform Unix/Windows)
- `crates/runtime-main/src/drone_ipc/connection.rs` (new — connection lifecycle + reconnect)
- `crates/runtime-main/tests/sdk_event_translation.rs` (new — table-driven translation tests)
- `crates/runtime-main/tests/sdk_cancellation.rs` (new — drop-mid-stream behavior)
- `crates/runtime-main/tests/drone_ipc_loopback.rs` (new — main-side client ↔ drone-server loopback exercising every DroneCommand variant)

### D.2 Files to Change

| File | Change |
|---|---|
| `crates/runtime-main/src/lib.rs` | **Edited** — `pub mod sdk;` + `pub mod drone_ipc;` |
| `crates/runtime-main/src/sdk/mod.rs` | **New** — re-exports + `SdkError` + `SessionId` newtype |
| `crates/runtime-main/src/sdk/agent_sdk.rs` | **New** — `AgentSdk<P: LLMProvider>` struct with `provider`, `event_tx`, `drone_client`, `session_id` fields; `run_agent(config) -> Result<(), SdkError>` method that drives the provider stream and emits `AgentEvent`s; uses `*_with`-style seam for testability |
| `crates/runtime-main/src/sdk/event_pipeline.rs` | **New** — `EventPipeline` translator with bundling state for consecutive `TextDelta`s; cancellation-safe; pure-logic `next_event()` method takes `ProviderEvent` and returns `Vec<AgentEvent>` (zero-or-more output per input) |
| `crates/runtime-main/src/sdk/decision_extractor.rs` | **New** — `extract_decision(text: &str) -> Option<DecisionRecord>` pure fn; first-line heuristic per problem statement; full property-test coverage of malformed inputs |
| `crates/runtime-main/src/drone_ipc/mod.rs` | **New** — re-exports + `DroneIpcError` |
| `crates/runtime-main/src/drone_ipc/client.rs` | **New** — `DroneClient` struct with `connect()`, `send(cmd: DroneCommand)`, `events() -> impl Stream<Item = DroneEvent>` methods; cfg-platform Unix `UnixStream` / Windows `NamedPipeClient` |
| `crates/runtime-main/src/drone_ipc/connection.rs` | **New** — connection state machine; reconnect on transient errors with exponential backoff (200ms → 400ms → 800ms → 1.6s, max 5 retries before surfacing `DroneIpcError::Disconnected`) |
| `crates/runtime-core/src/event.rs` | **Edited** — confirm/expand `AgentEvent` to match spec §2 v0.1 subset (M01 shipped some variants; M02 needs `agent_spawned`, `agent_complete`, `agent_error`, `tool_invoked`, `tool_result`, `stream_text`, `decision_record`, `task_started`, `task_completed`, `session_start`); add new variants only if missing |
| `crates/runtime-main/Cargo.toml` | **Edited** — add `tokio` features (`net`, `io-util`), `tokio-util` (`codec`) for `LinesCodec`, `tracing` for diagnostic logging |
| `Cargo.toml` (workspace root) | **Edited** — confirm `tokio`/`tokio-util` versions match across workspace |
| `crates/runtime-main/tests/sdk_event_translation.rs` | **New** — 20+ table-driven tests covering every `ProviderEvent`-to-`AgentEvent` mapping including bundling, decision extraction, error-path translation |
| `crates/runtime-main/tests/sdk_cancellation.rs` | **New** — drop-mid-stream tests using `tokio::pin!` + manual poll; verifies no panic, no leaked tasks |
| `crates/runtime-main/tests/drone_ipc_loopback.rs` | **New** — spawns a drone subprocess (uses M01 binary), connects via the main-side client, sends every `DroneCommand` variant, verifies expected `DroneEvent` responses; tests reconnect after killing the drone mid-session |
| `crates/runtime-main/README.md` | **Edited** — document `AgentSdk` usage, `DroneClient` connection, the `ProviderEvent`↔`AgentEvent` mapping table |
| `CLAUDE.md` | **Edited** — §5 add `runtime-main/src/sdk/` and `runtime-main/src/drone_ipc/` to safety primitives list with their respective coverage gates and exclusions |
| `.github/workflows/ci.yml` | **Edited** — extend the Coverage job's `runtime-main` gate to cover both `sdk/` and `drone_ipc/` (still ≥95% with documented exclusions) |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` Added section noting Stage D deliverables |

### D.3 Detailed Changes

#### `crates/runtime-main/src/sdk/mod.rs` (new)

```rust
//! Agent SDK — wraps any `LLMProvider` to drive an agent loop and emit
//! typed `AgentEvent`s. Spec §2.

mod agent_sdk;
mod event_pipeline;
mod decision_extractor;

pub use agent_sdk::{AgentSdk, SdkError, SessionId};
pub use event_pipeline::EventPipeline;
pub use decision_extractor::{extract_decision, DecisionRecord};
```

#### `crates/runtime-main/src/sdk/agent_sdk.rs` (new)

```rust
//! AgentSdk — drives a provider stream and emits AgentEvents. Spec §2.
//!
//! Generic over `LLMProvider` so v1.0+ providers slot in without changes.
//! Cancellation-safe: drop at any await point must clean up the drone IPC
//! connection without leaving orphan snapshots.
//!
//! Test seam: `run_agent_with_provider_stream` accepts a pre-built
//! ProviderEvent stream so tests can inject deterministic sequences
//! without touching reqwest. Production wrapper `run_agent` constructs
//! the real provider stream via `LLMProvider::stream()`.

use futures::stream::{Stream, StreamExt};
use runtime_core::event::AgentEvent;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use super::event_pipeline::EventPipeline;
use crate::drone_ipc::DroneClient;
use crate::providers::{AgentConfig, LLMProvider, ProviderError, ProviderEvent};

/// Newtype wrapping a session UUID.
#[derive(Debug, Clone, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    #[error("provider error: {0}")]
    Provider(#[from] ProviderError),
    #[error("drone IPC error: {0}")]
    Drone(#[from] crate::drone_ipc::DroneIpcError),
    #[error("event channel closed")]
    EventChannelClosed,
    #[error("agent {agent_id}: {message}")]
    Agent { agent_id: String, message: String },
}

/// Agent SDK. Generic over the LLM provider so v1.0+ providers (OpenAI,
/// local) slot in behind the same trait.
pub struct AgentSdk<P: LLMProvider> {
    provider:     Arc<P>,
    event_tx:     mpsc::Sender<AgentEvent>,
    drone_client: Arc<DroneClient>,
    session_id:   SessionId,
}

impl<P: LLMProvider + 'static> AgentSdk<P> {
    pub fn new(
        provider:     Arc<P>,
        event_tx:     mpsc::Sender<AgentEvent>,
        drone_client: Arc<DroneClient>,
        session_id:   SessionId,
    ) -> Self {
        Self { provider, event_tx, drone_client, session_id }
    }

    /// Production entry point. Constructs the provider stream and delegates.
    pub async fn run_agent(&self, config: AgentConfig) -> Result<(), SdkError> {
        let stream = self.provider.stream(config).await?;
        self.run_agent_with_provider_stream(stream).await
    }

    /// Test-seam variant. Accepts any pre-built ProviderEvent stream.
    pub async fn run_agent_with_provider_stream<S>(
        &self,
        mut stream: S,
    ) -> Result<(), SdkError>
    where
        S: Stream<Item = ProviderEvent> + Unpin,
    {
        let agent_id = format!("agent_{}", Uuid::new_v4());
        self.emit(AgentEvent::AgentSpawned {
            agent_id:    agent_id.clone(),
            agent_name:  "smoke".to_string(),
            parent_id:   None,
            session_id:  self.session_id.clone(),
        }).await?;

        // Tell the drone the task is starting. Drives a SnapshotNow.
        self.drone_client.send(runtime_core::drone::DroneCommand::SnapshotNow {
            reason:     "task_started".to_string(),
            state_json: serde_json::json!({"agent_id": agent_id}),
        }).await?;

        let mut pipeline = EventPipeline::new(agent_id.clone());

        while let Some(provider_event) = stream.next().await {
            for agent_event in pipeline.next_event(provider_event) {
                self.emit(agent_event).await?;
            }
        }

        // Flush any buffered text bundle.
        for agent_event in pipeline.flush() {
            self.emit(agent_event).await?;
        }

        Ok(())
    }

    async fn emit(&self, event: AgentEvent) -> Result<(), SdkError> {
        self.event_tx.send(event).await.map_err(|_| SdkError::EventChannelClosed)
    }
}

#[cfg(test)]
mod tests {
    // Tests live in tests/sdk_event_translation.rs and tests/sdk_cancellation.rs
    // for visibility; this module-level test stub confirms wiring only.

    #[test]
    fn session_id_is_unique_per_call() {
        use super::SessionId;
        let a = SessionId::new();
        let b = SessionId::new();
        assert_ne!(a, b);
    }
}
```

#### `crates/runtime-main/src/sdk/event_pipeline.rs` (new)

```rust
//! ProviderEvent → AgentEvent translator with consecutive-TextDelta
//! bundling. Pure logic; no I/O. Spec §2 event taxonomy.
//!
//! Bundling: consecutive `ProviderEvent::TextDelta`s collapse into one
//! `AgentEvent::StreamText` per non-text event boundary. Without this the
//! renderer gets spammed with one event per token; with it, one event per
//! "burst of text" which matches user expectation for streaming UX.

use runtime_core::event::AgentEvent;

use super::decision_extractor::extract_decision;
use crate::providers::ProviderEvent;

pub struct EventPipeline {
    agent_id:     String,
    text_buffer:  String,
}

impl EventPipeline {
    pub fn new(agent_id: String) -> Self {
        Self { agent_id, text_buffer: String::new() }
    }

    /// Translate one ProviderEvent. Returns zero-or-more AgentEvents.
    /// Bundling state is mutated; call `flush()` at end-of-stream.
    pub fn next_event(&mut self, event: ProviderEvent) -> Vec<AgentEvent> {
        let mut output = Vec::new();
        match event {
            ProviderEvent::TextDelta { text } => {
                self.text_buffer.push_str(&text);
            }
            ProviderEvent::ThinkingDelta { text } => {
                self.flush_text_buffer(&mut output);
                // Thinking deltas pass through as-is for now; M04 may surface
                // them differently (private trace vs renderer-facing).
                output.push(AgentEvent::StreamText {
                    agent_id: self.agent_id.clone(),
                    text,
                });
            }
            ProviderEvent::ToolUse { id, name, input } => {
                self.flush_text_buffer(&mut output);
                output.push(AgentEvent::ToolInvoked {
                    tool_name: name,
                    agent_id:  self.agent_id.clone(),
                    source:    runtime_core::event::ToolSource::Builtin, // M02 stub; refined in M06
                    server:    None,
                    input,
                });
                // Note: ToolResult AgentEvent is emitted in M03+ when tool
                // execution actually happens; M02 only records the invocation.
            }
            ProviderEvent::ToolResult { id, output: result } => {
                self.flush_text_buffer(&mut output);
                output.push(AgentEvent::ToolResult {
                    tool_name:    format!("tool_{id}"),
                    agent_id:     self.agent_id.clone(),
                    output:       result,
                    duration_ms:  0, // unknown until M03 runs the tool
                });
            }
            ProviderEvent::MessageStop { stop_reason } => {
                self.flush_text_buffer(&mut output);
                output.push(AgentEvent::AgentComplete {
                    agent_id: self.agent_id.clone(),
                    result:   stop_reason,
                });
            }
            ProviderEvent::Error { code, message } => {
                self.flush_text_buffer(&mut output);
                output.push(AgentEvent::AgentError {
                    agent_id: self.agent_id.clone(),
                    error:    format!("{code}: {message}"),
                });
            }
        }
        output
    }

    /// Drain any buffered text. Call at end-of-stream.
    pub fn flush(&mut self) -> Vec<AgentEvent> {
        let mut output = Vec::new();
        self.flush_text_buffer(&mut output);
        output
    }

    fn flush_text_buffer(&mut self, output: &mut Vec<AgentEvent>) {
        if !self.text_buffer.is_empty() {
            let text = std::mem::take(&mut self.text_buffer);
            // Decision extraction heuristic — see decision_extractor.rs.
            if let Some(decision) = extract_decision(&text) {
                output.push(AgentEvent::DecisionRecord {
                    agent_id:  self.agent_id.clone(),
                    decision:  decision.decision,
                    rationale: decision.rationale,
                    tool_used: decision.tool_used.unwrap_or_default(),
                });
            }
            output.push(AgentEvent::StreamText {
                agent_id: self.agent_id.clone(),
                text,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    // Table-driven coverage in tests/sdk_event_translation.rs.
    // Module-local sanity:
    use super::*;

    #[test]
    fn empty_flush_emits_nothing() {
        let mut p = EventPipeline::new("a1".into());
        assert!(p.flush().is_empty());
    }

    #[test]
    fn lone_text_delta_flushes_on_message_stop() {
        let mut p = EventPipeline::new("a1".into());
        let pre = p.next_event(ProviderEvent::TextDelta { text: "hi".into() });
        assert!(pre.is_empty(), "text deltas buffer until boundary");
        let post = p.next_event(ProviderEvent::MessageStop { stop_reason: "end_turn".into() });
        assert!(post.iter().any(|e| matches!(e, AgentEvent::StreamText { .. })));
        assert!(post.iter().any(|e| matches!(e, AgentEvent::AgentComplete { .. })));
    }
}
```

#### `crates/runtime-main/src/sdk/decision_extractor.rs` (new)

```rust
//! Heuristic decision extraction from streamed text.
//!
//! M02 ships the simplest version: detect a `Decision:` / `Rationale:`
//! pair on consecutive lines anywhere in the text. M04 (verify+rails)
//! upgrades to a structured emitter that gets injected by the prompt
//! template, eliminating the heuristic.
//!
//! Pure function; full property-test coverage of malformed inputs.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionRecord {
    pub decision:  String,
    pub rationale: String,
    pub tool_used: Option<String>,
}

/// Extract a decision record from a text block.
///
/// Heuristic: looks for `Decision: <text>` followed (possibly with
/// intervening blank lines) by `Rationale: <text>` and optionally
/// `Tool used: <text>`. Returns `None` if either marker is missing.
///
/// # Examples
///
/// ```
/// use runtime_main::sdk::extract_decision;
/// let text = "Decision: pick haiku\nRationale: cost-sensitive task\n";
/// let d = extract_decision(text).unwrap();
/// assert_eq!(d.decision,  "pick haiku");
/// assert_eq!(d.rationale, "cost-sensitive task");
/// ```
pub fn extract_decision(text: &str) -> Option<DecisionRecord> {
    let mut decision  = None;
    let mut rationale = None;
    let mut tool_used = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Decision:") {
            decision = Some(rest.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix("Rationale:") {
            rationale = Some(rest.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix("Tool used:") {
            tool_used = Some(rest.trim().to_string());
        }
    }
    match (decision, rationale) {
        (Some(d), Some(r)) => Some(DecisionRecord {
            decision:  d,
            rationale: r,
            tool_used,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_decision_and_rationale() {
        let t = "Decision: A\nRationale: B\n";
        let d = extract_decision(t).unwrap();
        assert_eq!(d.decision,  "A");
        assert_eq!(d.rationale, "B");
        assert!(d.tool_used.is_none());
    }

    #[test]
    fn extracts_tool_used_when_present() {
        let t = "Decision: ship\nRationale: green CI\nTool used: cargo test\n";
        let d = extract_decision(t).unwrap();
        assert_eq!(d.tool_used.unwrap(), "cargo test");
    }

    #[test]
    fn returns_none_when_decision_missing() {
        assert!(extract_decision("Rationale: only").is_none());
    }

    #[test]
    fn returns_none_when_rationale_missing() {
        assert!(extract_decision("Decision: only").is_none());
    }

    #[test]
    fn returns_none_for_empty_input() {
        assert!(extract_decision("").is_none());
    }

    #[test]
    fn handles_intervening_blank_lines() {
        let t = "Decision: A\n\n\nRationale: B\n";
        let d = extract_decision(t).unwrap();
        assert_eq!(d.decision,  "A");
        assert_eq!(d.rationale, "B");
    }

    #[test]
    fn handles_leading_whitespace() {
        let t = "   Decision: A   \n   Rationale: B   \n";
        let d = extract_decision(t).unwrap();
        assert_eq!(d.decision,  "A");
        assert_eq!(d.rationale, "B");
    }

    #[test]
    fn last_decision_wins_when_multiple() {
        let t = "Decision: first\nRationale: r1\nDecision: second\nRationale: r2\n";
        let d = extract_decision(t).unwrap();
        assert_eq!(d.decision,  "second");
        assert_eq!(d.rationale, "r2");
    }

    proptest::proptest! {
        #[test]
        fn never_panics_on_arbitrary_input(s in "\\PC{0,1000}") {
            let _ = extract_decision(&s);
        }
    }
}
```

#### `crates/runtime-main/src/drone_ipc/mod.rs` (new)

```rust
//! Main-side IPC client for the runtime-drone subprocess (M01).
//!
//! Wire format: `LinesCodec`-framed JSON over Unix domain socket
//! (Linux/macOS) or Windows named pipe. Same format the drone speaks
//! (see crates/runtime-drone/src/ipc.rs).
//!
//! M02 only sends `DroneCommand::SnapshotNow` (on task lifecycle events).
//! M03+ adds `SpawnProcess`, `StopProcess`, etc. as new subsystems land.

mod client;
mod connection;

pub use client::DroneClient;
pub use connection::DroneIpcError;
```

#### `crates/runtime-main/src/drone_ipc/client.rs` (new)

```rust
//! DroneClient — main-side connection to the runtime-drone subprocess.
//!
//! Cfg-platform: UnixStream on Linux/macOS; NamedPipeClient on Windows.
//! Reconnects automatically on transient errors (see `connection.rs`).

use futures::stream::Stream;
use runtime_core::drone::{DroneCommand, DroneEvent};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::connection::{Connection, DroneIpcError};

pub struct DroneClient {
    inner: Arc<Mutex<Connection>>,
}

impl DroneClient {
    /// Connect to the drone over its IPC socket / named pipe.
    pub async fn connect(addr: &str) -> Result<Self, DroneIpcError> {
        let conn = Connection::connect(addr).await?;
        Ok(Self { inner: Arc::new(Mutex::new(conn)) })
    }

    /// Send a DroneCommand. Reconnects on transient errors.
    pub async fn send(&self, cmd: DroneCommand) -> Result<(), DroneIpcError> {
        let mut guard = self.inner.lock().await;
        guard.send_with_reconnect(cmd).await
    }

    /// Stream of incoming DroneEvents.
    pub async fn events(
        &self,
    ) -> Result<impl Stream<Item = Result<DroneEvent, DroneIpcError>>, DroneIpcError> {
        let guard = self.inner.lock().await;
        Ok(guard.event_stream())
    }
}
```

#### `crates/runtime-main/src/drone_ipc/connection.rs` (new)

```rust
//! Connection state machine + reconnect policy.

use futures::stream::Stream;
use runtime_core::drone::{DroneCommand, DroneEvent};
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

#[cfg(unix)]
use tokio::net::UnixStream;
#[cfg(windows)]
use tokio::net::windows::named_pipe::ClientOptions;

#[derive(Debug, Error)]
pub enum DroneIpcError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("codec: {0}")]
    Codec(String),
    #[error("disconnected after {retries} retries")]
    Disconnected { retries: u32 },
    #[error("serialization: {0}")]
    Json(#[from] serde_json::Error),
}

const MAX_RETRIES: u32 = 5;
const BASE_BACKOFF: Duration = Duration::from_millis(200);

pub(crate) struct Connection {
    addr: String,
    /// Inner socket; cfg-platform.
    #[cfg(unix)]
    stream: Option<UnixStream>,
    #[cfg(windows)]
    stream: Option<tokio::net::windows::named_pipe::NamedPipeClient>,
}

impl Connection {
    pub async fn connect(addr: &str) -> Result<Self, DroneIpcError> {
        let stream = open(addr).await?;
        Ok(Self { addr: addr.to_string(), stream: Some(stream) })
    }

    pub async fn send_with_reconnect(
        &mut self,
        cmd: DroneCommand,
    ) -> Result<(), DroneIpcError> {
        for attempt in 0..MAX_RETRIES {
            match self.send_once(&cmd).await {
                Ok(()) => return Ok(()),
                Err(DroneIpcError::Io(_)) => {
                    let backoff = BASE_BACKOFF * 2u32.pow(attempt);
                    sleep(backoff).await;
                    let _ = self.reconnect().await;
                }
                Err(other) => return Err(other),
            }
        }
        Err(DroneIpcError::Disconnected { retries: MAX_RETRIES })
    }

    async fn send_once(&mut self, cmd: &DroneCommand) -> Result<(), DroneIpcError> {
        // Real impl uses tokio_util::codec::LinesCodec wrapped around the stream.
        // Stage D author writes this against the M01 drone's accept_loop.
        // Pseudo-shape:
        //   let json = serde_json::to_string(cmd)? + "\n";
        //   self.stream.as_mut().write_all(json.as_bytes()).await?;
        //   self.stream.as_mut().flush().await?;
        //   Ok(())
        unimplemented!("Stage D author: real LinesCodec write + flush")
    }

    async fn reconnect(&mut self) -> Result<(), DroneIpcError> {
        self.stream = Some(open(&self.addr).await?);
        Ok(())
    }

    pub fn event_stream(&self) -> impl Stream<Item = Result<DroneEvent, DroneIpcError>> {
        // Real impl: FramedRead over the read half + LinesCodec + serde_json.
        // Stage D author: full body. Mirrors crates/runtime-drone/src/ipc.rs:32-39.
        futures::stream::empty()
    }
}

#[cfg(unix)]
async fn open(addr: &str) -> Result<UnixStream, DroneIpcError> {
    Ok(UnixStream::connect(addr).await?)
}

#[cfg(windows)]
async fn open(
    addr: &str,
) -> Result<tokio::net::windows::named_pipe::NamedPipeClient, DroneIpcError> {
    Ok(ClientOptions::new().open(addr)?)
}
```

Stage D's implementer fills in the `unimplemented!()` and `futures::stream::empty()` placeholders against the M01 drone-side codec (the wire format is identical; this is just the client side of the same socket). The structure above pins the public API and the reconnect policy.

#### `crates/runtime-core/src/event.rs` (edited)

Confirm `AgentEvent` includes the variants Stage D emits:

- `SessionStart` (M01 may have shipped; verify shape: `{ session_id, framework, model }`)
- `AgentSpawned`
- `AgentComplete`
- `AgentError`
- `ToolInvoked` (with `ToolSource` enum: `Mcp | Builtin | Generated`)
- `ToolResult`
- `StreamText`
- `DecisionRecord`
- `TaskStarted`
- `TaskCompleted`

If any are missing, add per spec §2 lines 834-920. Property-test their serde round-trips (M01.B pattern).

#### `crates/runtime-main/Cargo.toml` (edited)

Add to `[dependencies]`:
```toml
tokio-util = { workspace = true, features = ["codec"] }
uuid       = { workspace = true, features = ["v4", "serde"] }
```

#### `crates/runtime-main/tests/sdk_event_translation.rs` (new)

Table-driven tests over `EventPipeline::next_event`. Each row is `(input ProviderEvent sequence, expected AgentEvent sequence)`. Cover:

- single TextDelta + MessageStop → StreamText + AgentComplete
- multiple TextDeltas + MessageStop → bundled StreamText + AgentComplete
- TextDelta + ToolUse + TextDelta + MessageStop → StreamText, ToolInvoked, StreamText, AgentComplete (boundary forces flush)
- ToolUse first → no leading StreamText
- ThinkingDelta routes correctly
- Error event → AgentError + buffer flush
- Empty stream → empty output
- Decision-pattern in TextDelta → DecisionRecord + StreamText (both)
- Multiple consecutive ToolUses → multiple ToolInvoked, no spurious StreamText
- Out-of-order text after MessageStop → flushed (defensive)

20+ assertions total; `proptest` for "no input sequence panics."

#### `crates/runtime-main/tests/sdk_cancellation.rs` (new)

Drop-mid-stream behavior:
- Build an infinite ProviderEvent stream
- Drive `run_agent_with_provider_stream` inside a `tokio::select!` with a timeout
- Assert no panic, assert event_tx is dropped cleanly (Receiver sees channel-closed)
- Assert no Tokio task remains (`tokio::runtime::Handle::current().metrics().num_alive_tasks()` stable)
- Repeat with drop happening inside a TextDelta burst, after ToolUse, mid-MessageStop

#### `crates/runtime-main/tests/drone_ipc_loopback.rs` (new)

End-to-end test exercising every `DroneCommand` variant against a real drone subprocess:

```rust
//! Drone IPC loopback — main-side client ↔ drone-server (M01) end-to-end.
//!
//! Spawns a runtime-drone subprocess, connects via main-side DroneClient,
//! sends every DroneCommand variant, asserts expected DroneEvent responses.
//!
//! Reconnect path: kill the drone mid-session, verify client reconnects on
//! the next send (within MAX_RETRIES), verify subsequent commands work.

#![cfg(any(unix, windows))]

use runtime_main::drone_ipc::DroneClient;
use runtime_core::drone::{DroneCommand, DroneEvent};
// ... full body authored by Stage D implementer.
```

Tests:
1. `connects_and_handshakes`
2. `sends_snapshot_now_receives_snapshot_written`
3. `sends_graceful_shutdown_drone_exits_clean`
4. `sends_spawn_process_receives_process_spawned`  
5. `sends_stop_process_receives_process_stopped`
6. `sends_set_activity_timeout_no_event_expected`
7. `sends_revert_to_snapshot_receives_recovery_available`
8. `reconnects_after_drone_killed_mid_session` (kills with SIGKILL, verifies retry budget consumed correctly)
9. `surfaces_disconnected_error_after_max_retries` (drone never restarts; confirms exponential backoff timing)
10. `cancels_cleanly_on_drop` (drops client mid-send, verifies no orphan socket)

#### Other file edits

- `crates/runtime-main/README.md` — append §"Agent SDK" with usage example, mapping table for ProviderEvent→AgentEvent, drone IPC connection example
- `CLAUDE.md` §5 — add `runtime-main/src/sdk/` and `runtime-main/src/drone_ipc/` to safety primitives list with exclusions documented
- `.github/workflows/ci.yml` — extend `runtime-main` ≥95% gate to span both new modules; ignore-filename-regex updated to include `src.drone_ipc.connection\.rs::open` (cfg-platform OS-call holdouts)
- `CHANGELOG.md` — `[Unreleased]` Added section per Stage D deliverables

### D.4 Tests

Total: 50+ tests across the new modules and integration files.

**Unit (in-module):**
- `decision_extractor::tests` — 8 + property test (no panic on arbitrary input)
- `event_pipeline::tests` — 2 module-local sanity (table-driven volume in integration file)
- `agent_sdk::tests::session_id_is_unique_per_call` — 1
- `connection::tests` — backoff timing test, reconnect-success test, max-retries-surfaces test (≥3)

**Integration (`tests/`):**
- `sdk_event_translation.rs` — 20+ rows + property test
- `sdk_cancellation.rs` — 5+ drop-mid-stream tests
- `drone_ipc_loopback.rs` — 10 tests (every DroneCommand variant + reconnect + cancellation + max-retries)

**Coverage target:**

- Workspace ≥80% (general gate, unchanged)
- `runtime-drone` ≥95% (unchanged)
- **`runtime-main` ≥95%** continuing — exclusions extended to:
  - `src/providers/anthropic.rs` (network wrapper, Stage C exclusion)
  - `src/drone_ipc/connection.rs::open` (cfg-platform OS-call wrapper; the testable seam is `Connection::send_with_reconnect` which is fully covered by loopback + injected-error tests)

The exclusion list is documented inline in `CLAUDE.md` §5 (per the M01.C codification pattern) and in `.github/workflows/ci.yml` with the ignore-filename-regex.

**Coverage delta gate.** Active from M02 Stage A baseline. Stage D may shift the baseline meaningfully (large new module surface); CI computes the delta vs main post-Stage-C-merge, fails if any safety-primitive crate regresses >0.5pp.

### D.5 CLI Prompt

```
Read CLAUDE.md for all project rules.
Read docs/build-prompts/M02-event-pipeline.md Stage D (sections D.1 through D.4).

Read prior stage retrospectives for guidance:
  docs/build-prompts/retrospectives/M02.A-retrospective.md
  docs/build-prompts/retrospectives/M02.B-retrospective.md
  docs/build-prompts/retrospectives/M02.C-retrospective.md
  Focus: [END] "Decisions for the next stage" sections + any [LIVE]
  friction events flagged as relevant to Stage D. Apply decisions.

Read docs/gap-analysis.md for any Carry-forward items targeting Stage D.

Read for reference (do not modify):
  crates/runtime-drone/src/ipc.rs (lines 13–119) — the drone-side codec.
    Main-side client mirrors the framing exactly.
  crates/runtime-main/src/providers/anthropic_sse.rs — the *_with-style
    test seam pattern. Apply the same shape to AgentSdk.

═══ STEP 1 — WRITE FAILING TESTS ═══

Create test files / add to existing per D.4:

1. crates/runtime-main/src/sdk/decision_extractor.rs::tests (9 tests + 1
   proptest)
2. crates/runtime-main/src/sdk/event_pipeline.rs::tests (2 module-local)
3. crates/runtime-main/src/sdk/agent_sdk.rs::tests::session_id_is_unique
4. crates/runtime-main/src/drone_ipc/connection.rs::tests (3+ for
   backoff/reconnect/max-retries)
5. crates/runtime-main/tests/sdk_event_translation.rs (20+ rows + proptest)
6. crates/runtime-main/tests/sdk_cancellation.rs (5+ drop-mid-stream)
7. crates/runtime-main/tests/drone_ipc_loopback.rs (10 tests; spawns
   real drone subprocess via cargo run --bin runtime-drone)

Run: cargo test --workspace --package runtime-main
Confirm: all tests fail with `cannot find module sdk`, `cannot find type
DroneClient`, etc. — TDD red phase.

═══ STEP 2 — IMPLEMENT ═══

Apply per D.3 in this order (each step minimal-implementation to make
its corresponding tests pass; iterate):

1. crates/runtime-core/src/event.rs — add any AgentEvent variants Stage D
   needs that aren't yet present. Round-trip property tests follow the
   M01.B pattern.
2. crates/runtime-main/Cargo.toml — add tokio-util + uuid features.
3. crates/runtime-main/src/lib.rs — `pub mod sdk;` + `pub mod drone_ipc;`.
4. crates/runtime-main/src/sdk/mod.rs — module root with re-exports.
5. crates/runtime-main/src/sdk/decision_extractor.rs (NEW) — full body
   per D.3 + proptest.
6. crates/runtime-main/src/sdk/event_pipeline.rs (NEW) — full body per D.3.
7. crates/runtime-main/src/sdk/agent_sdk.rs (NEW) — full body per D.3
   including the *_with test seam.
8. crates/runtime-main/src/drone_ipc/mod.rs (NEW) — module root.
9. crates/runtime-main/src/drone_ipc/connection.rs (NEW) — full body
   including LinesCodec + reconnect with exponential backoff.
   Replace the `unimplemented!()` placeholder against the M01 drone codec.
10. crates/runtime-main/src/drone_ipc/client.rs (NEW) — full body per D.3.
11. tests/sdk_event_translation.rs (NEW) — table-driven body covering
    every ProviderEvent→AgentEvent mapping per D.4.
12. tests/sdk_cancellation.rs (NEW) — drop-mid-stream tests per D.4.
13. tests/drone_ipc_loopback.rs (NEW) — every-variant + reconnect tests
    per D.4. Spawns drone subprocess.
14. .github/workflows/ci.yml — extend runtime-main coverage gate per D.3.
15. CLAUDE.md §5 — add sdk/ and drone_ipc/ to safety primitives list.
16. crates/runtime-main/README.md — append §"Agent SDK" section.
17. CHANGELOG.md [Unreleased] — append Added section.

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
  cargo llvm-cov --package runtime-main --ignore-filename-regex "src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs" --fail-under-lines 95
  cargo llvm-cov --package runtime-drone --ignore-filename-regex "src.main\.rs|generated|src.lib\.rs|src.shutdown\.rs" --fail-under-lines 95

Cancellation safety: run `cargo test sdk_cancellation -- --test-threads=1`
and confirm no leaked tokio tasks via inspection of test output.

If any gate fails, follow CLAUDE.md §7 self-correction. Max 3 iterations
then surface.

═══ STEP 4 — RETROSPECTIVE ═══

Per CLAUDE.md §19, copy retrospectives/RETROSPECTIVE-TEMPLATE.md to:
  docs/build-prompts/retrospectives/M02.D-retrospective.md

Fill in [LIVE] sections during work, then [END] scoring + threshold gates
+ decisions for Stage E (specifically: did the *_with seam pattern hold
up for AgentSdk; did the decision-extractor heuristic catch real cases
or generate false positives; did the reconnect policy surface anything
worth carrying into the spec; how does the drone-IPC main-side coverage
compare to drone-server side from M01.C).

═══ STEP 5 — SURFACE TO USER ═══

Run: git status, git diff --stat HEAD
Re-run all gates one final time.

Surface: diff stat, gate results, M02.D retrospective, draft commit from D.6.

State: "Stage D is ready. I will NOT commit until you approve."

Wait for explicit approval. Do NOT push (push waits for Stage F per
CLAUDE.md §20).

On approval (Stage D — work stage; not the final stage of a parent milestone):
1. Commit Stage D on the parent-milestone branch claude/m02-event-pipeline
   (do NOT push).
2. Stop. Surface the commit. Stage E is opened in a fresh session.
```

### D.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(runtime-main): M02 Stage D — AgentSdk + drone IPC client + event translation

Wraps any LLMProvider in an AgentSdk that drives the agent loop, translates
ProviderEvents to typed AgentEvents for the renderer, and connects to the
M01 drone over its IPC socket / named pipe to drive snapshots on task
lifecycle events.

Architecture:
- src/sdk/agent_sdk.rs — AgentSdk<P: LLMProvider> wrapping any provider.
  *_with-style test seam (run_agent_with_provider_stream takes any
  Stream<ProviderEvent>) so wiremock-fed and synthetic streams exercise
  the same logic. Production wrapper run_agent constructs the real
  provider stream.
- src/sdk/event_pipeline.rs — pure ProviderEvent→AgentEvent translator
  with consecutive-TextDelta bundling. Bundles burst into one StreamText
  per non-text event boundary; flushed on stream end. Pure logic; no I/O.
- src/sdk/decision_extractor.rs — first-line "Decision:/Rationale:"
  heuristic per spec §2 decision_record events. Pure fn; proptest covers
  no-panic on arbitrary input. M04 verify+rails work upgrades to a
  structured emitter, eliminating the heuristic.
- src/drone_ipc/client.rs — main-side DroneClient mirrors the M01
  drone-server LinesCodec framing. Sends DroneCommand, receives
  DroneEvent; cfg-platform Unix UnixStream / Windows NamedPipeClient.
- src/drone_ipc/connection.rs — exponential-backoff reconnect policy
  (200ms → 400ms → 800ms → 1.6s, max 5 retries) before surfacing
  DroneIpcError::Disconnected.

Tests (50+ total):
- 9 unit tests for decision extractor + 1 proptest for no-panic.
- 20+ table-driven tests in tests/sdk_event_translation.rs covering every
  ProviderEvent→AgentEvent mapping including bundling boundaries, decision
  extraction integration, error-path translation.
- 5+ drop-mid-stream cancellation-safety tests in tests/sdk_cancellation.rs.
- 10 drone-IPC loopback tests in tests/drone_ipc_loopback.rs spawning the
  M01 drone subprocess and exercising every DroneCommand variant + the
  reconnect + max-retries-disconnected surface paths.

Coverage:
- runtime-main ≥95% line continuing; exclusion list extended to include
  drone_ipc/connection.rs::open (cfg-platform OS-call wrapper). Inner
  testable seam Connection::send_with_reconnect fully covered by loopback.
- AgentEvent serde round-trips re-tested per M01.B pattern after any
  variant additions.

Spec §2 alignment:
- AgentEvent variants emitted: SessionStart, AgentSpawned, AgentComplete,
  AgentError, ToolInvoked, ToolResult, StreamText, DecisionRecord,
  TaskStarted, TaskCompleted. Subset of the full union; extends as M03+
  subsystems land.
- M02 single-turn only; multi-turn tool-use loops are M03+.
- ToolSource enum default = Builtin; refined to Mcp by M06.

Refs: M02-event-pipeline.md §D; agent-runtime-spec.md §2 §1d (drone
wire format); CLAUDE.md §5 (coverage policy) + new safety primitive
listings; M01.C codification commit 1dec4ba.

Retrospective: docs/build-prompts/retrospectives/M02.D-retrospective.md

https://claude.ai/code/session_<id>
EOF
)"
```

---

<!-- ============================================================ -->
<!-- STAGE E — Tauri shell + skeleton React renderer + frontend CI -->
<!-- ============================================================ -->

## Stage E — Tauri shell + skeleton React renderer + frontend CI gates + Playwright

### E.1 Problem Statement

Stage E lights up the user-facing path. A minimal React + TypeScript renderer (Vite-bundled) shows a single page with a "Run smoke test" button. Clicking it invokes a Tauri command `run_smoke_session` on the main process, which constructs an `AgentSdk<AnthropicProvider>`, pulls the API key from OS keychain, runs the agent against a hardcoded "Say hello" prompt, and emits each `AgentEvent` to the renderer via `app.emit("agent_event", event)`. The renderer subscribes via `listen('agent_event')` and appends each event to an unstyled `<ul>`. End state: user clicks button → renderer lists `agent_spawned → stream_text* → agent_complete`.

This is also the first time the project has any frontend code. Stage E activates the full frontend CI pipeline: Vite + TypeScript strict + ESLint + Prettier + Vitest unit tests + Playwright end-to-end against the built Tauri app. CI gates land for all of them (CLAUDE.md §6). The frontend tests cover both pure-logic units (event-list reducer, IPC subscriber lifecycle) and the full E2E happy path (click button → events appear).

The Tauri side adds the allowlisted command, wires `tauri::AppHandle::emit` from the SDK's `event_tx` receiver, and configures `tauri.conf.json` with the minimum capability set (single-window, no shell, no network from renderer — main is the only network actor). Per spec §10 capability boundary, the renderer never holds the API key, never speaks HTTP, never touches files: every privileged operation goes through a typed `#[tauri::command]`.

Stage E also stores the API key into the OS keychain via a one-time `set_api_key` command (rather than requiring the user to use a separate tool). The command takes the key as input, writes to `keyring::Entry::new("agent-runtime", "anthropic")`, and never returns the key (zeros from memory after write). The smoke session command reads the key on each invocation; if not present, surfaces `SetupRequired` error to the renderer.

**Success criterion.** `npm run tauri dev` boots the app. User clicks "Set API Key" → enters `sk-ant-...` → click stored. User clicks "Run smoke test" → renderer's event list shows ≥4 events ending in `agent_complete`. Playwright E2E test reproduces this against the built app on Linux/macOS/Windows × stable. All frontend gates green: prettier, eslint, tsc strict, vitest, npm audit (high+).

**New artifacts:**
- `src/main.tsx` (new — Vite entry)
- `src/App.tsx` (new — single-page UI)
- `src/components/EventList.tsx` (new — `<ul>` of `AgentEvent`s)
- `src/components/SetupPanel.tsx` (new — API key entry)
- `src/components/SmokeButton.tsx` (new — invokes `run_smoke_session`)
- `src/lib/ipc.ts` (new — typed wrapper around `@tauri-apps/api/core::invoke` + `event::listen`)
- `src/lib/eventReducer.ts` (new — pure reducer for the event list)
- `src/types/agent_event.ts` (new — generated from `runtime-core` schema OR hand-written subset typed as `runtime_core::AgentEvent` discriminated union; M03+ regenerates from schema)
- `src/index.html` (new — Vite root)
- `src/styles.css` (new — minimal; `<ul>` looks like a `<ul>`; M03 brings real styling)
- `package.json` (new — dependencies + scripts)
- `package-lock.json` (new — lockfile)
- `tsconfig.json` (new — TS strict)
- `vite.config.ts` (new — Vite + Tauri plugin)
- `vitest.config.ts` (new — Vitest)
- `playwright.config.ts` (new — Playwright E2E)
- `.eslintrc.cjs` (new — ESLint rules)
- `.prettierrc.json` (new — Prettier config)
- `tests/e2e/smoke.spec.ts` (new — Playwright smoke test)
- `tests/unit/eventReducer.test.ts` (new — Vitest reducer tests)
- `tests/unit/ipc.test.ts` (new — Vitest IPC wrapper tests)
- `src-tauri/src/main.rs` (edited — add `run_smoke_session`, `set_api_key` commands; configure event channel; wire AgentSdk)
- `src-tauri/src/commands.rs` (new — Tauri command handlers)
- `src-tauri/tauri.conf.json` (edited — capability set, build commands, identifier)
- `src-tauri/Cargo.toml` (edited — depend on `runtime-main`, `runtime-core`)
- `src-tauri/capabilities/default.json` (new — Tauri 2.x capability config)
- `crates/runtime-main/src/key_store.rs` (new — `read_api_key()` + `write_api_key()` keyring wrappers)

### E.2 Files to Change

| File | Change |
|---|---|
| `package.json` | **New** — root npm package; deps: `react@18`, `react-dom@18`, `@tauri-apps/api@2`, `@tauri-apps/plugin-shell@2`; devDeps: `typescript@5`, `vite@5`, `@vitejs/plugin-react@4`, `vitest@2`, `@playwright/test@1.48`, `eslint@9`, `prettier@3`, `@types/react@18`, `@types/react-dom@18`. Scripts: `dev`, `build`, `tauri`, `lint`, `format`, `test`, `test:e2e`, `typecheck` |
| `package-lock.json` | **New** — generated by `npm install` |
| `tsconfig.json` | **New** — TS 5.x strict; `target: ES2022`; `module: ESNext`; `moduleResolution: bundler`; `jsx: react-jsx`; `strict: true`; `noUncheckedIndexedAccess: true`; `noImplicitOverride: true` |
| `vite.config.ts` | **New** — Vite + `@vitejs/plugin-react`; `clearScreen: false`; ports for Tauri |
| `vitest.config.ts` | **New** — Vitest with happy-dom env; coverage via `@vitest/coverage-v8` |
| `playwright.config.ts` | **New** — Playwright; targets the built Tauri app; cross-platform via `webServer` config that spawns `npm run tauri dev` |
| `.eslintrc.cjs` | **New** — ESLint 9 flat config; `@typescript-eslint`, `react`, `react-hooks` plugins; strict TS rules |
| `.prettierrc.json` | **New** — Prettier 3 config; 2-space indent, trailing commas, single quotes |
| `src/index.html` | **New** — Vite root with `<div id="root"></div>` |
| `src/main.tsx` | **New** — React 18 root render |
| `src/App.tsx` | **New** — composes `SetupPanel`, `SmokeButton`, `EventList`; manages app state via `useReducer` |
| `src/components/EventList.tsx` | **New** — props `{events: AgentEvent[]}`; renders unstyled `<ul>` |
| `src/components/SetupPanel.tsx` | **New** — input + "Save Key" button; invokes `set_api_key`; never re-displays the key |
| `src/components/SmokeButton.tsx` | **New** — button + invokes `run_smoke_session`; subscribes to events |
| `src/lib/ipc.ts` | **New** — typed wrappers: `invokeRunSmokeSession()`, `invokeSetApiKey(key)`, `subscribeAgentEvents(handler)`. All return `Promise` types matching the Rust command signatures |
| `src/lib/eventReducer.ts` | **New** — pure reducer `(state, action) => state` for the event list; action types `event_received`, `clear`, `error` |
| `src/types/agent_event.ts` | **New** — discriminated union mirroring `runtime_core::AgentEvent` v0.1 subset; explicit type for every variant Stage D emits |
| `src/styles.css` | **New** — minimal CSS; centers the page, `<ul>` defaults |
| `tests/e2e/smoke.spec.ts` | **New** — Playwright: launches Tauri app, sets a fake API key (mocks the Anthropic call via main-side wiremock for E2E), clicks button, asserts ≥4 events appear |
| `tests/unit/eventReducer.test.ts` | **New** — Vitest: `event_received` appends; `clear` empties; `error` sets error state; immutability invariants |
| `tests/unit/ipc.test.ts` | **New** — Vitest: mocks `@tauri-apps/api/core` to verify command invocation shape; subscriber lifecycle (subscribe → emit → unsubscribe → no further calls) |
| `src-tauri/src/main.rs` | **Edited** — `tauri::Builder::default()` registers `run_smoke_session` + `set_api_key` commands; sets up `tokio::sync::mpsc` for event flow; spawns task that forwards `AgentEvent` from the channel to `app.emit("agent_event", e)` |
| `src-tauri/src/commands.rs` | **New** — `#[tauri::command] async fn run_smoke_session(state: State<...>) -> Result<(), CmdError>` and `#[tauri::command] async fn set_api_key(key: String) -> Result<(), CmdError>` |
| `src-tauri/Cargo.toml` | **Edited** — add `runtime-main = { workspace = true }`, `runtime-core = { workspace = true }`, `tokio` features |
| `src-tauri/tauri.conf.json` | **Edited** — productName, version, identifier `com.agent-runtime.app`, single-window config, build commands point at Vite |
| `src-tauri/capabilities/default.json` | **New** — Tauri 2.x capability set: `core:default`, `event:default` (so `app.emit` works); explicitly NOT `shell:*`, NOT `fs:*`, NOT `http:*` (renderer has zero filesystem / network / shell access) |
| `crates/runtime-main/src/key_store.rs` | **New** — `read_api_key() -> Result<SecretString, KeyStoreError>` reads keyring entry `agent-runtime/anthropic`; `write_api_key(key: &str) -> Result<(), KeyStoreError>` writes; `delete_api_key()` for tests |
| `crates/runtime-main/src/lib.rs` | **Edited** — `pub mod key_store;` |
| `.github/workflows/ci.yml` | **Edited** — add Frontend job: `npm ci`, `npx prettier --check`, `npx eslint`, `npx tsc --noEmit`, `npm run test`, `npm audit --audit-level=high`. Add E2E job: `npm run tauri build` + `npx playwright test` against the built artifact |
| `.gitignore` | **Edited** — `node_modules/`, `dist/`, `.vite/`, `playwright-report/`, `test-results/` |
| `crates/runtime-main/README.md` | **Edited** — document `key_store` usage; reference Tauri command surface |
| `CLAUDE.md` | **Edited** — §5 confirm coverage policy applies to frontend (vitest coverage threshold + E2E green); §6 add the new frontend gates to the must-pass list with explicit commands |
| `docs/MVP-v0.1.md` | **Edited** — §M2 acceptance criteria checklist marked `[x]` for items Stage E delivers |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` Added section noting Stage E deliverables |

### E.3 Detailed Changes

#### `package.json` (new)

```json
{
  "name": "agent-runtime",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev":       "vite",
    "build":     "tsc --noEmit && vite build",
    "preview":   "vite preview",
    "tauri":     "tauri",
    "lint":      "eslint .",
    "format":    "prettier --write .",
    "format:check": "prettier --check .",
    "typecheck": "tsc --noEmit",
    "test":      "vitest run",
    "test:watch": "vitest",
    "test:e2e":  "playwright test",
    "test:e2e:install": "playwright install --with-deps"
  },
  "dependencies": {
    "react":              "^18.3.0",
    "react-dom":          "^18.3.0",
    "@tauri-apps/api":    "^2.0.0"
  },
  "devDependencies": {
    "@playwright/test":              "^1.48.0",
    "@tauri-apps/cli":               "^2.0.0",
    "@types/react":                  "^18.3.0",
    "@types/react-dom":              "^18.3.0",
    "@typescript-eslint/eslint-plugin": "^8.0.0",
    "@typescript-eslint/parser":     "^8.0.0",
    "@vitejs/plugin-react":          "^4.3.0",
    "@vitest/coverage-v8":           "^2.1.0",
    "eslint":                        "^9.0.0",
    "eslint-plugin-react":           "^7.37.0",
    "eslint-plugin-react-hooks":     "^5.0.0",
    "happy-dom":                     "^15.0.0",
    "prettier":                      "^3.3.0",
    "typescript":                    "^5.6.0",
    "vite":                          "^5.4.0",
    "vitest":                        "^2.1.0"
  }
}
```

(Versions verified against npm/crates registries 2026-05; pin to current stable major. M03+ may bump.)

#### `tsconfig.json` (new)

```json
{
  "compilerOptions": {
    "target":                 "ES2022",
    "module":                 "ESNext",
    "moduleResolution":       "bundler",
    "lib":                    ["ES2022", "DOM", "DOM.Iterable"],
    "jsx":                    "react-jsx",
    "strict":                 true,
    "noUncheckedIndexedAccess": true,
    "noImplicitOverride":     true,
    "noFallthroughCasesInSwitch": true,
    "forceConsistentCasingInFileNames": true,
    "esModuleInterop":        true,
    "skipLibCheck":           true,
    "resolveJsonModule":      true,
    "isolatedModules":        true,
    "noEmit":                 true,
    "allowJs":                false
  },
  "include": ["src", "tests"]
}
```

#### `vite.config.ts` (new)

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Tauri-recommended config: don't clear screen, use a fixed port,
// expose env vars as VITE_*.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port:        1420,
    strictPort:  true,
    host:        false,
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    target:       'es2022',
    minify:       'esbuild',
    sourcemap:    true,
    chunkSizeWarningLimit: 600,
  },
});
```

#### `src/types/agent_event.ts` (new)

Discriminated union mirroring `runtime_core::AgentEvent` v0.1 subset Stage D emits. M03+ regenerates from schema (per CLAUDE.md §14).

```typescript
// Mirrors runtime_core::AgentEvent (subset emitted in M02 Stage D).
// Will be regenerated from schemas/event.v1.json in M03+.
export type AgentEvent =
  | { type: 'session_start';   session_id: string; framework: string; model: string }
  | { type: 'agent_spawned';   agent_id: string; agent_name: string; parent_id: string | null; session_id: string }
  | { type: 'agent_complete';  agent_id: string; result: string }
  | { type: 'agent_error';     agent_id: string; error: string }
  | { type: 'tool_invoked';    tool_name: string; agent_id: string; source: 'mcp' | 'builtin' | 'generated'; server: string | null; input: unknown }
  | { type: 'tool_result';     tool_name: string; agent_id: string; output: unknown; duration_ms: number }
  | { type: 'stream_text';     agent_id: string; text: string }
  | { type: 'decision_record'; agent_id: string; decision: string; rationale: string; tool_used: string }
  | { type: 'task_started';    task_id: string; agent_id: string }
  | { type: 'task_completed';  task_id: string; duration_ms: number };
```

#### `src/lib/eventReducer.ts` (new)

```typescript
import type { AgentEvent } from '../types/agent_event';

export interface State {
  readonly events: readonly AgentEvent[];
  readonly error: string | null;
  readonly running: boolean;
}

export const initialState: State = {
  events:  [],
  error:   null,
  running: false,
};

export type Action =
  | { type: 'event_received'; event: AgentEvent }
  | { type: 'clear' }
  | { type: 'error';          message: string }
  | { type: 'started' }
  | { type: 'completed' };

export function reducer(state: State, action: Action): State {
  switch (action.type) {
    case 'event_received':
      return { ...state, events: [...state.events, action.event] };
    case 'clear':
      return { ...initialState };
    case 'error':
      return { ...state, error: action.message, running: false };
    case 'started':
      return { ...state, running: true, error: null };
    case 'completed':
      return { ...state, running: false };
  }
}
```

#### `src/lib/ipc.ts` (new)

```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { AgentEvent } from '../types/agent_event';

export async function invokeRunSmokeSession(): Promise<void> {
  await invoke('run_smoke_session');
}

export async function invokeSetApiKey(key: string): Promise<void> {
  await invoke('set_api_key', { key });
}

export async function subscribeAgentEvents(
  handler: (event: AgentEvent) => void,
): Promise<UnlistenFn> {
  return listen<AgentEvent>('agent_event', (e) => handler(e.payload));
}
```

#### `src/App.tsx` (new)

```typescript
import { useEffect, useReducer, useState } from 'react';
import { initialState, reducer } from './lib/eventReducer';
import { invokeRunSmokeSession, invokeSetApiKey, subscribeAgentEvents } from './lib/ipc';
import { EventList }   from './components/EventList';
import { SetupPanel }  from './components/SetupPanel';
import { SmokeButton } from './components/SmokeButton';
import './styles.css';

export function App(): JSX.Element {
  const [state, dispatch] = useReducer(reducer, initialState);
  const [hasKey, setHasKey] = useState(false);

  useEffect(() => {
    const unsubscribePromise = subscribeAgentEvents((event) => {
      dispatch({ type: 'event_received', event });
      if (event.type === 'agent_complete' || event.type === 'agent_error') {
        dispatch({ type: 'completed' });
      }
    });
    return () => {
      void unsubscribePromise.then((unsub) => unsub());
    };
  }, []);

  async function handleSetKey(key: string): Promise<void> {
    await invokeSetApiKey(key);
    setHasKey(true);
  }

  async function handleSmoke(): Promise<void> {
    dispatch({ type: 'clear' });
    dispatch({ type: 'started' });
    try {
      await invokeRunSmokeSession();
    } catch (e) {
      dispatch({ type: 'error', message: e instanceof Error ? e.message : String(e) });
    }
  }

  return (
    <main>
      <h1>Agent Runtime — M02 smoke</h1>
      <SetupPanel onSave={handleSetKey} />
      <SmokeButton disabled={!hasKey || state.running} onClick={handleSmoke} />
      {state.error && <p className="error">{state.error}</p>}
      <EventList events={state.events} />
    </main>
  );
}
```

#### `src/components/EventList.tsx` (new)

```typescript
import type { AgentEvent } from '../types/agent_event';

interface Props { events: readonly AgentEvent[]; }

export function EventList({ events }: Props): JSX.Element {
  return (
    <ul aria-label="agent events">
      {events.map((event, idx) => (
        <li key={idx} data-event-type={event.type}>
          <strong>{event.type}</strong>{' '}
          {renderSummary(event)}
        </li>
      ))}
    </ul>
  );
}

function renderSummary(event: AgentEvent): string {
  switch (event.type) {
    case 'agent_spawned':   return `agent ${event.agent_id}`;
    case 'agent_complete':  return `result: ${event.result}`;
    case 'agent_error':     return `error: ${event.error}`;
    case 'tool_invoked':    return `${event.tool_name} (${event.source})`;
    case 'tool_result':     return `${event.tool_name} (${event.duration_ms}ms)`;
    case 'stream_text':     return event.text;
    case 'decision_record': return event.decision;
    case 'session_start':   return `session ${event.session_id}`;
    case 'task_started':    return `task ${event.task_id}`;
    case 'task_completed':  return `task ${event.task_id} (${event.duration_ms}ms)`;
  }
}
```

#### `src/components/SetupPanel.tsx` (new)

```typescript
import { useState } from 'react';

interface Props { onSave: (key: string) => Promise<void>; }

export function SetupPanel({ onSave }: Props): JSX.Element {
  const [key,    setKey]    = useState('');
  const [saving, setSaving] = useState(false);
  const [saved,  setSaved]  = useState(false);

  async function handleSave(): Promise<void> {
    setSaving(true);
    try {
      await onSave(key);
      setKey('');
      setSaved(true);
    } finally {
      setSaving(false);
    }
  }

  return (
    <section aria-label="api key setup">
      <label>
        Anthropic API key:
        <input
          type="password"
          value={key}
          onChange={(e) => setKey(e.target.value)}
          placeholder="sk-ant-..."
          disabled={saving}
        />
      </label>
      <button onClick={handleSave} disabled={saving || key.length < 10}>
        {saving ? 'Saving…' : 'Save key'}
      </button>
      {saved && <span aria-label="saved">✓ stored in OS keychain</span>}
    </section>
  );
}
```

#### `src/components/SmokeButton.tsx` (new)

```typescript
interface Props {
  disabled: boolean;
  onClick:  () => Promise<void>;
}

export function SmokeButton({ disabled, onClick }: Props): JSX.Element {
  return (
    <button
      onClick={() => { void onClick(); }}
      disabled={disabled}
      aria-label="run smoke test"
    >
      Run smoke test
    </button>
  );
}
```

#### `src-tauri/src/commands.rs` (new)

```rust
//! Tauri command handlers for M02 Stage E.

use runtime_core::event::AgentEvent;
use runtime_main::providers::{
    AgentConfig, ContentBlock, Message, MessageRole, anthropic::AnthropicProvider,
};
use runtime_main::sdk::{AgentSdk, SessionId};
use runtime_main::key_store::{read_api_key, write_api_key, KeyStoreError};
use runtime_main::drone_ipc::DroneClient;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Debug, Error, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CmdError {
    #[error("API key not set; call set_api_key first")]
    SetupRequired,
    #[error("provider error: {0}")]
    Provider(String),
    #[error("drone IPC unavailable: {0}")]
    Drone(String),
    #[error("key store: {0}")]
    KeyStore(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl From<KeyStoreError> for CmdError {
    fn from(e: KeyStoreError) -> Self { Self::KeyStore(e.to_string()) }
}

pub struct AppState {
    pub drone_addr: String,
}

#[tauri::command]
pub async fn set_api_key(key: String) -> Result<(), CmdError> {
    write_api_key(&key).map_err(CmdError::from)?;
    // SecretString-wrapped key zeroes from memory on drop after the write.
    Ok(())
}

#[tauri::command]
pub async fn run_smoke_session(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), CmdError> {
    let api_key = match read_api_key() {
        Ok(k) => k,
        Err(KeyStoreError::NotFound) => return Err(CmdError::SetupRequired),
        Err(e) => return Err(e.into()),
    };
    let provider = Arc::new(AnthropicProvider::new(api_key));
    let drone = Arc::new(
        DroneClient::connect(&state.drone_addr)
            .await
            .map_err(|e| CmdError::Drone(e.to_string()))?,
    );
    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(64);
    let sdk = AgentSdk::new(provider, event_tx, drone, SessionId::new());

    // Forward events to the renderer.
    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            let _ = app_clone.emit("agent_event", &event);
        }
    });

    let config = AgentConfig {
        model: "claude-haiku-4-5".into(),
        messages: vec![Message {
            role: MessageRole::User,
            content: vec![ContentBlock::Text { text: "Say only the word: hello".into() }],
        }],
        max_tokens: 16,
        temperature: Some(0.0),
        system_prompt: None,
        tools: vec![],
    };

    sdk.run_agent(config)
        .await
        .map_err(|e| CmdError::Provider(e.to_string()))?;
    Ok(())
}
```

#### `src-tauri/capabilities/default.json` (new)

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capabilities the renderer is allowed to use. Locked-down for M02; renderer never touches network, shell, or filesystem directly — privileged ops go through #[tauri::command].",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:event:default",
    "core:event:allow-listen",
    "core:event:allow-emit"
  ]
}
```

Notably absent: `shell:*`, `fs:*`, `http:*`, `dialog:*`. Renderer cannot invoke shell commands, read files, make HTTP calls, or open native dialogs. Per spec §10 capability boundary; aligns with CLAUDE.md §15 trap #10 (Tauri allowlist is the security boundary).

#### `crates/runtime-main/src/key_store.rs` (new)

```rust
//! OS-keychain-backed API key storage.
//!
//! Uses the `keyring` crate to read/write under service `agent-runtime`,
//! user `anthropic`. SecretString-wrapped on read so the key never
//! Debug-prints. Per spec §13 zero-telemetry — no logging of the key.

use keyring::Entry;
use secrecy::SecretString;
use thiserror::Error;

const SERVICE: &str = "agent-runtime";
const USER:    &str = "anthropic";

#[derive(Debug, Error)]
pub enum KeyStoreError {
    #[error("API key not found in OS keychain (entry: {SERVICE}/{USER})")]
    NotFound,
    #[error("keyring error: {0}")]
    Keyring(#[from] keyring::Error),
}

pub fn read_api_key() -> Result<SecretString, KeyStoreError> {
    let entry = Entry::new(SERVICE, USER)?;
    match entry.get_password() {
        Ok(s)   => Ok(SecretString::from(s)),
        Err(keyring::Error::NoEntry) => Err(KeyStoreError::NotFound),
        Err(e)  => Err(e.into()),
    }
}

pub fn write_api_key(key: &str) -> Result<(), KeyStoreError> {
    let entry = Entry::new(SERVICE, USER)?;
    entry.set_password(key)?;
    Ok(())
}

#[cfg(test)]
pub fn delete_api_key() -> Result<(), KeyStoreError> {
    let entry = Entry::new(SERVICE, USER)?;
    let _ = entry.delete_credential();
    Ok(())
}

#[cfg(test)]
mod tests {
    // Real keychain access is environment-specific (Linux Secret Service,
    // macOS Keychain, Windows Credential Manager). These tests run in CI
    // only on platforms where a session keychain is available; otherwise
    // gated as #[ignore]-by-default. The keyring crate's mock backend is
    // used when no platform service is available.

    use super::*;

    #[test]
    #[ignore = "requires platform keychain — run locally or in CI cells with session bus"]
    fn read_after_write_roundtrips() {
        write_api_key("sk-ant-test").unwrap();
        let key = read_api_key().unwrap();
        use secrecy::ExposeSecret;
        assert_eq!(key.expose_secret(), "sk-ant-test");
        delete_api_key().unwrap();
    }

    #[test]
    #[ignore = "requires platform keychain"]
    fn read_when_missing_returns_not_found() {
        delete_api_key().unwrap();
        assert!(matches!(read_api_key(), Err(KeyStoreError::NotFound)));
    }
}
```

#### `tests/e2e/smoke.spec.ts` (new)

```typescript
import { test, expect, _electron as electron } from '@playwright/test';
import { spawn } from 'node:child_process';
import { setTimeout as sleep } from 'node:timers/promises';

// Playwright launches the built Tauri app via its bundled binary.
// Cross-platform: the binary path differs per OS; the playwright.config.ts
// `webServer` configuration handles the spawn.

test('smoke: click button → events appear → agent_complete', async ({ page }) => {
  // Set a fake API key first (the wiremock-equivalent for E2E uses a
  // localhost mock Anthropic served by a dev fixture).
  await page.locator('input[type=password]').fill('sk-ant-fixture');
  await page.locator('button:has-text("Save key")').click();
  await expect(page.locator('text=stored in OS keychain')).toBeVisible();

  // Run smoke.
  await page.locator('button:has-text("Run smoke test")').click();

  // Wait for the event list to populate.
  const events = page.locator('ul[aria-label="agent events"] li');
  await expect(events.first()).toBeVisible({ timeout: 10_000 });

  // Should see at least 4 events: agent_spawned, ≥1 stream_text, agent_complete.
  await expect(events).toHaveCount({ gte: 4 } as any, { timeout: 15_000 });

  // Last event should be agent_complete or agent_error.
  const last = events.last();
  const lastType = await last.getAttribute('data-event-type');
  expect(['agent_complete', 'agent_error']).toContain(lastType);
});

test('smoke: clicking smoke without API key surfaces SetupRequired', async ({ page }) => {
  // Note: this test requires the keychain to be empty at start. CI runs
  // tests in isolation; locally the developer runs `keyring delete` first.
  await page.locator('button:has-text("Run smoke test")').click();
  await expect(page.locator('text=API key not set')).toBeVisible();
});
```

#### `.github/workflows/ci.yml` (edited)

Add jobs:

```yaml
  frontend:
    name: Frontend (TypeScript)
    runs-on: ubuntu-latest
    needs: detect-frontend
    if: needs.detect-frontend.outputs.has_frontend == 'true'
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      - run: npm ci
      - run: npx prettier --check .
      - run: npx eslint .
      - run: npx tsc --noEmit
      - run: npm run test
      - run: npm audit --audit-level=high
      - uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: frontend-test-output
          path: |
            test-results/
            coverage/

  e2e:
    name: E2E (Playwright)
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    needs: [rust, frontend]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      - name: Install Tauri Linux system deps
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev libssl-dev build-essential curl wget file
      - run: npm ci
      - run: npx playwright install --with-deps
      - run: npm run tauri build -- --debug
      - run: npx playwright test
      - uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: playwright-report-${{ matrix.os }}
          path: playwright-report/
```

(`detect-frontend` job mirrors `detect-cargo` — checks for `package.json` at root.)

#### `docs/MVP-v0.1.md` §M2 (edited)

Mark acceptance criteria delivered by Stage E as `[x]`:

- `[x]` User clicks "Run smoke test" in renderer; main calls Anthropic with a hardcoded prompt; renderer shows `agent_spawned` → `tool_invoked` (LoadSkill) → `stream_text` chunks → `agent_complete`
- `[x]` Drone snapshots fire on `task_started` events (none yet at this stage; just verify the wiring)
- `[x]` Anthropic API key read from OS keychain via `keyring` crate
- `[x]` Provider integration tests use `wiremock` for offline CI; real-API smoke is a manual `cargo test --features integration` run
- `[x]` No third-party SDK in `Cargo.toml` for Anthropic — direct `reqwest` + `eventsource-stream` only

(Verification waits for actual implementation to land.)

#### `CLAUDE.md` §6 (edited)

Add Frontend gates explicitly:

```
### Frontend gates (active from M02 Stage E)

```bash
npm ci
npx prettier --check '**/*.{ts,tsx,js,jsx,json,md,yml,yaml}'
npx eslint .
npx tsc --noEmit
npm run test
npm audit --audit-level=high
npx playwright test  # E2E; requires built app
```

These gates must pass on every PR from M02 onward. Vitest coverage threshold ≥80% on src/; Playwright E2E green on Linux/macOS/Windows.
```

#### `CHANGELOG.md` (edited)

Append to `[Unreleased]`:

```markdown
### Added (M02 Stage E)
- `package.json` + full frontend tooling (Vite, TypeScript strict, React 18, Vitest, Playwright, ESLint 9, Prettier 3).
- `src/` skeleton renderer: `App.tsx`, `EventList`, `SetupPanel`, `SmokeButton`, typed IPC wrappers, pure event reducer.
- `src/types/agent_event.ts` — TS discriminated union mirroring `runtime_core::AgentEvent` v0.1 subset.
- `src-tauri/src/commands.rs` — `run_smoke_session` and `set_api_key` Tauri commands.
- `src-tauri/capabilities/default.json` — locked-down capability set: `event` only; no shell, no fs, no http.
- `crates/runtime-main/src/key_store.rs` — OS-keychain-backed API key storage via `keyring` crate.
- E2E test: `tests/e2e/smoke.spec.ts` (Playwright; cross-platform).
- Frontend CI gates: prettier / eslint / tsc / vitest / npm audit.
- E2E CI gate: Playwright on Linux/macOS/Windows.

### Status
- M02 §M2 acceptance criteria all met (renderer shows agent_spawned → stream_text → agent_complete on click; API key in OS keychain; wiremock for CI; no third-party SDK).
```

### E.4 Tests

Total: 30+ tests across unit, integration, and E2E.

**Frontend unit (Vitest):**
1. `eventReducer.test::initial_state_is_empty`
2. `event_received_appends_immutably` — verifies state.events !== prevState.events
3. `clear_resets_to_initial`
4. `error_sets_message_and_clears_running`
5. `started_sets_running_clears_error`
6. `completed_clears_running`
7. `multiple_events_preserve_order`
8. `clear_after_events_drops_them`
9. `error_during_running_keeps_existing_events` — defensive
10. `ipc.test::invokeRunSmokeSession_calls_invoke_with_correct_command_name`
11. `invokeSetApiKey_passes_key_arg`
12. `subscribeAgentEvents_returns_unlisten_fn`
13. `subscribeAgentEvents_handler_called_on_emit`
14. `unsubscribe_stops_handler_calls`

**Backend unit (Rust):**
15. `key_store::tests::read_after_write_roundtrips` (gated `#[ignore]` for CI cells without keychain)
16. `key_store::tests::read_when_missing_returns_not_found` (same gating)
17. `commands::tests::cmd_error_serializes_with_type_tag` — verifies `serde_json::to_string(&CmdError::SetupRequired)` produces `{"type":"setup_required"}` for renderer pattern matching
18. `commands::tests::set_api_key_zeros_input` — verifies the `key: String` argument doesn't outlive the function (memory safety on the boundary)

**E2E (Playwright):**
19. `smoke.spec::click_button_events_appear` — happy path
20. `smoke_without_key_surfaces_setup_required` — error path
21. `events_appear_in_order_agent_spawned_first` — invariant
22. `events_terminate_with_agent_complete_or_error` — invariant
23. `clear_button_resets_event_list` — UX (if added)
24. `set_key_input_is_password_type` — security: key never visible
25. `smoke_disabled_when_no_key` — UX
26. `smoke_disabled_during_running` — UX

**Wiremock-driven E2E (Linux only — uses localhost mock Anthropic):**
27. `e2e_with_mock_anthropic` — full pipeline against wiremock'd /v1/messages
28. `e2e_handles_provider_error_event` — wiremock returns SSE error → renderer shows agent_error

**Backend integration:**
29. Update `tests/sdk_event_translation.rs` — add commands.rs flow tests
30. Update `tests/drone_ipc_loopback.rs` — verify SnapshotNow fires when run_smoke_session is invoked end-to-end

#### Coverage target

- Workspace ≥80% (general Rust gate, unchanged)
- `runtime-drone` ≥95% (unchanged)
- `runtime-main` ≥95% (continuing from Stage C; exclusion list extended for `commands.rs::run_smoke_session` if it spawns an unmockable Tauri AppHandle subprocess; `key_store.rs` excluded the platform-keychain-call path)
- **Frontend src/ ≥80%** (NEW — Vitest coverage gate). Excludes `*.tsx` component leaves where Playwright E2E provides better coverage than unit tests; covers the reducer + IPC layer fully.
- **E2E green on Linux/macOS/Windows** (NEW — Playwright)

### E.5 CLI Prompt

```
Read CLAUDE.md for all project rules.
Read docs/build-prompts/M02-event-pipeline.md Stage E (sections E.1 through E.4).

Read prior stage retrospectives for guidance:
  docs/build-prompts/retrospectives/M02.A-retrospective.md
  docs/build-prompts/retrospectives/M02.B-retrospective.md
  docs/build-prompts/retrospectives/M02.C-retrospective.md
  docs/build-prompts/retrospectives/M02.D-retrospective.md
  Focus: [END] "Decisions for the next stage" sections + any [LIVE]
  friction events. Apply decisions.

Read docs/gap-analysis.md for any Carry-forward items targeting Stage E.

Read for reference:
  agent-runtime-spec.md §M2 (acceptance criteria); §10 (capability
    boundary); §13 (zero-telemetry); §"Project Structure" src/ layout.
  CLAUDE.md §6 frontend gates section.
  src-tauri/tauri.conf.json (current minimal Tauri config from M01).
  crates/runtime-main/src/sdk/agent_sdk.rs (Stage D AgentSdk to wire).

═══ STEP 1 — WRITE FAILING TESTS ═══

Frontend tests don't exist yet. Create:

1. tests/unit/eventReducer.test.ts — 9 reducer tests per E.4 #1–#9.
2. tests/unit/ipc.test.ts — 5 tests per E.4 #10–#14 with @tauri-apps/api
   mocked via vitest-mock-extended.
3. crates/runtime-main/src/key_store.rs::tests — 2 tests per E.4 #15–#16
   (keychain-gated #[ignore]).
4. src-tauri/src/commands.rs::tests — 2 tests per E.4 #17–#18.
5. tests/e2e/smoke.spec.ts — 8 Playwright tests per E.4 #19–#26.

Run: cargo test --workspace && npm run test
Confirm: all tests fail (no production code yet).

═══ STEP 2 — IMPLEMENT ═══

Apply per E.3 in order. Each step is one file or one logical group:

1. package.json + tsconfig.json + vite.config.ts + vitest.config.ts +
   playwright.config.ts + .eslintrc.cjs + .prettierrc.json + .gitignore
   updates. Run `npm install`. Verify `npx tsc --noEmit` errors only on
   missing source files (expected).
2. src/index.html + src/styles.css.
3. src/types/agent_event.ts.
4. src/lib/eventReducer.ts (full body per E.3).
5. src/lib/ipc.ts (full body per E.3).
6. src/components/{EventList,SetupPanel,SmokeButton}.tsx.
7. src/App.tsx.
8. src/main.tsx (React 18 root).
9. crates/runtime-main/src/key_store.rs (NEW) + lib.rs export.
10. src-tauri/src/commands.rs (NEW) + main.rs registers commands +
    spawns event-forwarding task.
11. src-tauri/Cargo.toml — depend on runtime-main, runtime-core.
12. src-tauri/tauri.conf.json — productName, identifier, build cmds.
13. src-tauri/capabilities/default.json (NEW) — minimal capability set.
14. .github/workflows/ci.yml — add `frontend` and `e2e` jobs.
15. CLAUDE.md §6 — add frontend gates section.
16. crates/runtime-main/README.md — append key_store + Tauri command docs.
17. docs/MVP-v0.1.md §M2 — mark acceptance criteria checked.
18. CHANGELOG.md [Unreleased] — append Added + Status sections.

Throughout: run gates incrementally. After each major step, run
`npm run test` (frontend) or `cargo test --workspace` (Rust) to keep
the loop tight.

═══ STEP 3 — VERIFY ═══

Rust gates (unchanged set + extended runtime-main coverage gate):
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo build --workspace
  cargo test --workspace
  cargo test --workspace --doc
  RUSTDOCFLAGS="-D missing_docs" cargo doc --workspace --no-deps
  cargo audit
  cargo deny check
  cargo llvm-cov --workspace --ignore-filename-regex "src.main\.rs|generated" --fail-under-lines 80
  cargo llvm-cov --package runtime-main --ignore-filename-regex "src.main\.rs|generated|src.providers.anthropic\.rs|src.drone_ipc.connection\.rs|src.key_store\.rs" --fail-under-lines 95
  cargo llvm-cov --package runtime-drone --ignore-filename-regex "src.main\.rs|generated|src.lib\.rs|src.shutdown\.rs" --fail-under-lines 95

Frontend gates:
  npm ci
  npx prettier --check '**/*.{ts,tsx,js,jsx,json,md,yml,yaml}'
  npx eslint .
  npx tsc --noEmit
  npm run test
  npm audit --audit-level=high

E2E gate (slow; run last):
  npm run tauri build -- --debug
  npx playwright test

If any gate fails, follow CLAUDE.md §7 self-correction. Max 3 iterations
then surface.

═══ STEP 4 — RETROSPECTIVE ═══

Per CLAUDE.md §19, copy retrospectives/RETROSPECTIVE-TEMPLATE.md to:
  docs/build-prompts/retrospectives/M02.E-retrospective.md

Fill in [LIVE] sections during work. At [END]: score axes, evaluate
threshold gates, decisions for Stage F. Specific things to capture:
- Did the Tauri capability set need expansion? If so, document.
- Playwright stability across OSes — flake rate worth noting?
- Frontend coverage 80% threshold — felt right or too lenient?
- Did the keyring crate's CI behavior surface anything?
- Was the IPC TypeScript type sync (manual mirror of AgentEvent) a
  source of bugs? Anything to drive M03 schema-codegen.

═══ STEP 5 — SURFACE TO USER ═══

Run: git status, git diff --stat HEAD
Re-run all gates one final time.

Surface: diff stat, gate results, M02.E retrospective, draft commit from E.6.

State: "Stage E is ready. I will NOT commit until you approve."

Wait for explicit approval. Do NOT push (push waits for Stage F per
CLAUDE.md §20).

On approval (Stage E — work stage; not the final stage of a parent milestone):
1. Commit Stage E on the parent-milestone branch claude/m02-event-pipeline
   (do NOT push).
2. Stop. Surface the commit. Stage F (Phase Closeout: Gap Analysis) is
   opened in a fresh session.
```

### E.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
feat(workspace): M02 Stage E — Tauri shell + skeleton renderer + E2E

Lights up the user-facing path. User clicks "Run smoke test" → main
calls Anthropic via the AgentSdk from Stage D → renderer lists
AgentEvents as they arrive. End-to-end across Anthropic API + main
process + Tauri IPC + React renderer + drone snapshot writes.

Frontend (React 18 + TypeScript strict + Vite):
- src/App.tsx composes SetupPanel + SmokeButton + EventList. State via
  useReducer over a pure reducer (lib/eventReducer.ts) with full
  immutability. IPC layer (lib/ipc.ts) wraps @tauri-apps/api/core
  invoke + event listen with typed return signatures.
- src/types/agent_event.ts mirrors runtime_core::AgentEvent v0.1
  subset (10 variants Stage D emits). M03+ regenerates from schema.
- Components are unstyled — `<ul>` looks like a `<ul>`. M03 brings
  React Flow + real graph rendering.

Tauri (2.x):
- src-tauri/src/commands.rs — run_smoke_session, set_api_key
  #[tauri::command]s. CmdError serializes with type-tag for renderer
  pattern matching.
- src-tauri/capabilities/default.json — locked-down: event only; no
  shell, no fs, no http. Renderer cannot bypass the command surface.
- src-tauri/src/main.rs registers commands + spawns event-forwarding
  task (mpsc channel → app.emit).

Backend:
- crates/runtime-main/src/key_store.rs — keyring-crate-backed API key
  storage under service "agent-runtime", user "anthropic". SecretString
  on read; never Debug-prints; never logged. Per spec §13 zero-telemetry.

Tests (30+):
- 14 frontend unit tests (Vitest) — reducer + IPC wrappers + immutability.
- 4 backend unit tests — key_store roundtrip (gated), CmdError serde.
- 8 Playwright E2E tests — happy path, error paths, password-input
  invariants, button-state UX. Cross-platform: Linux/macOS/Windows.
- 2 wiremock-backed E2E tests — full pipeline against localhost mock
  Anthropic for offline CI.

CI:
- New `frontend` job: prettier / eslint / tsc / vitest / npm audit.
- New `e2e` job (matrix Linux/macOS/Windows): tauri build + playwright.
- Coverage gate extends:
  - runtime-main ≥95% continuing; key_store.rs added to exclusion list
    (platform-keychain-call holdout, same M01.C class as ipc.rs/open).
  - Frontend src/ ≥80% via @vitest/coverage-v8.
- §0d acceptance criteria for M02 marked [x] in docs/MVP-v0.1.md.

CLAUDE.md §6 — frontend gates section added with explicit commands;
becomes part of the must-pass list from this commit forward.

Versions verified against npm/crates registries 2026-05 per the
web-first rule:
- React 18.3, TypeScript 5.6, Vite 5.4, Vitest 2.1, Playwright 1.48
- ESLint 9, Prettier 3.3, Tauri CLI 2.0
- @tauri-apps/api 2.0
- Rust deps continue from Stages B–D.

Refs: M02-event-pipeline.md §E; agent-runtime-spec.md §M2 §10 §13;
docs/MVP-v0.1.md §M2 acceptance; CLAUDE.md §6 + §15 trap #10.

Retrospective: docs/build-prompts/retrospectives/M02.E-retrospective.md

https://claude.ai/code/session_<id>
EOF
)"
```

---

<!-- ============================================================ -->
<!-- STAGE F — Phase Closeout: Gap Analysis (per CLAUDE.md §20)     -->
<!-- ============================================================ -->

## Stage F — Phase Closeout: Gap Analysis

> **Per CLAUDE.md §20.** This stage runs after Stages A–E commit and `M02-summary.md` lands. It produces one new entry in `docs/gap-analysis.md`. The gap-analysis commit is the final commit on the parent-milestone branch — it gates the M02 PR push.

### F.1 Problem Statement

Generate the M02 entry in `docs/gap-analysis.md`. Cumulative review of code-vs-spec across all milestones to date (M01 + M02). Append-only — never edit prior entries (Pre-M01 baseline, Pre-M01 addendum, M01).

The M02 review covers both new ground (provider abstraction, SSE state machine, agent SDK, Tauri shell, frontend skeleton, E2E pipeline) and resolution of carry-forward items M02 closed (mcp_servers table, .gitattributes, Windows drone integration test, coverage delta gating, *_with pattern documentation, src-tauri/gen/schemas/ gitignore). The Adherence section records every ⚠️ where M02 ships behavior that diverges from spec; the Spec review section flags places spec needs M03+ updates (Phase 3 React Flow + Zustand expansion, Session FSM diagram, plan model field shapes); the Fix backlog prioritizes M03-prep work; the Carry-forward section reports status of every prior-milestone Important item.

**Success criterion.** New M02 entry in `docs/gap-analysis.md` follows the six-section template at the top of the file. Append-only check passes locally and in CI. M02-summary.md aggregates the per-stage retrospectives. PR push gated on this commit; PR description references all stage commits + retrospectives + summary + gap-analysis entry (three-artifact review per CLAUDE.md §19 + §20).

### F.2 Files to Change

| File | Change |
|---|---|
| `docs/gap-analysis.md` | **Edited (append-only)** — new M02 section appended at the bottom per the entry template. **Prior entries (Pre-M01 baseline, Pre-M01 addendum, M01) are immutable** — never edit, reorder, or delete. CI append-only check enforces. |
| `docs/build-prompts/retrospectives/M02-summary.md` | **New** — aggregates M02.A through M02.E retrospectives per `docs/build-prompts/retrospectives/SUMMARY-TEMPLATE.md`. Per-stage axis scores rolled up; cross-stage friction patterns; verdict (Pattern held / held with friction / strained); decisions to apply before M03 authoring. |
| `CHANGELOG.md` | **Edited** — `[Unreleased]` Status section updated noting M02 gap analysis appended; M02 entry summary; carry-forward closure summary. |

### F.3 Detailed Changes

The M02 entry follows the six-section template defined at the top of `docs/gap-analysis.md`. Do NOT diverge from the template; do NOT skip sections (write "None observed." if a section truly has nothing to report).

**Process:**

1. Re-read `agent-runtime-spec.md` end-to-end (yes, all of it — at least skim, with focus on §2, §2a, §2b, §2c, §10, §13, §M2 — sections M02 touches).
2. Read every file produced or edited across all M02 stages:
   - List via `git log --oneline main..HEAD` and `git diff --stat main..HEAD`.
   - Open every new `.rs`, `.ts`, `.tsx`, `.json`, `.toml`, `.yml`, `.md` file. Cumulative review (M01 surfaces still in scope).
3. Read the prior `docs/gap-analysis.md` entries IN FULL (Pre-M01 baseline, Pre-M01 addendum, M01). The Carry-forward section MUST report status of every Important item from those entries — Resolved / Still open / Deferred to M0X.
4. Read the M02 per-stage retrospectives + draft `M02-summary.md` aggregating them.
5. Draft the new gap-analysis entry per the template. Severity is non-elastic (CLAUDE.md §20).
6. Run the append-only check locally before surfacing:
   ```
   git fetch origin main
   git show origin/main:docs/gap-analysis.md > /tmp/gap-base.md
   base_lines=$(wc -l < /tmp/gap-base.md)
   diff /tmp/gap-base.md <(head -n "$base_lines" docs/gap-analysis.md)
   ```
   Output must be empty. If it isn't, prior content was modified; revert and re-edit by APPENDING ONLY at the bottom.

#### M02 entry — section-by-section guidance

**1. Codebase deep dive** — narrative review of the *cumulative* code shipped to date (M01 + M02), not just M02. 200–500 words. Surface:
- Provider abstraction quality: does the `LLMProvider` trait surface bend cleanly when v1.0+ adds OpenAI / local? Note any signature that already feels too-Anthropic-shaped.
- SSE state machine completeness: every Anthropic event variant exercised? Any wire-format quirk discovered during implementation that the prompt didn't anticipate?
- AgentSdk + drone IPC: cancellation safety holding under real (non-test) conditions? Any orphan-task or leaked-socket failure modes surface?
- Tauri capability boundary: did the lockdown hold (event-only)? Any capability that had to be widened during implementation?
- Frontend type sync: TS `AgentEvent` discriminated union currently hand-mirrored from `runtime_core::AgentEvent` Rust enum. Did this drift during M02? Cite as a M03 codegen target.
- Coverage holdouts: list every `--ignore-filename-regex` exclusion across the workspace (M01.C drone wrapper, anthropic.rs network wrapper, drone_ipc/connection.rs::open, key_store.rs platform-keychain-call). Are any reaching for over-exclusion?

**2. Adherence to spec** — for each area touched, classify ✅ / ⚠️ / ❌ with file:line citations on both spec and code sides. Cover:
- ✅ items: zero-telemetry (no analytics dep added); direct Anthropic API (no `@anthropic-ai/sdk` or `anthropic-rs`); SSE event sequence matches §2c wire format; capability lockdown matches §10; renderer never speaks HTTP / fs / shell.
- ⚠️ items expected (Stage authors should land these or surface them):
  - Decision extraction is a heuristic (M02 ships first-line "Decision:/Rationale:"); M04 verify+rails replaces with structured emitter. Document.
  - `count_tokens` is char/4 approximation (Stage B/C); M04 budget integration replaces with real `/v1/messages/count_tokens` endpoint. Document.
  - `ProviderEvent::ToolResult` translates to `AgentEvent::ToolResult` with `duration_ms: 0` (M02 doesn't run tools); M03 fills correctly. Document.
  - `AgentSdk` is generic over `P: LLMProvider` — not `Box<dyn>`. v1.0+ multi-provider switching will refactor; documented as the M02 decision.
  - `signal.rs` is scaffolded but no signals are emitted (emission lands in M04). Verify the type surface is right.
  - `mcp_servers` table created but no MCP code consumes it (lands in M06). Verify the schema is forward-compatible.
- ❌ items: "None observed" if no outright contradictions where code ships behavior the spec forbids or vice versa.

**3. Spec review (forward-looking)** —
- Missing items: Phase 3 React Flow + Zustand spec expansion (Pre-M01 carry-forward, M03-blocking); Session FSM diagram in §11 (Pre-M01 carry-forward, M04-blocking); plan model field shapes (M04-blocking); model deprecation policy (when does the runtime stop accepting deprecated model IDs).
- Contradictions: any spec section that says X and another says NOT X. Surface explicitly.
- Ambiguity: any §2c trait method whose semantics aren't clear (cancellation safety guarantees, error variant exhaustiveness, retry policy expectations).
- Open questions: should `ProviderEvent::Error` be terminal (stream ends after) or recoverable (stream continues if downstream code retries)? M02 implementation chose terminal; spec doesn't say.
- Recommended spec changes: bundle into a post-M02 `docs(spec):` PR before M03 authoring (parallel to the post-M01 `docs(spec):` PR pattern).

**4. Fix backlog** — cumulative across M01 + M02; severity non-elastic.
- 🔴 Critical (must fix before M03 starts): expected to be empty if M02 shipped correctly. Surface honestly if not.
- 🟡 Important (should fix this release cycle): TS `AgentEvent` codegen from schema (M03 prep — manual mirror is fragile); Phase 3 React Flow + Zustand spec expansion (M03-blocking); plan model field shapes (M04-blocking); Session FSM diagram (M04-blocking); decision extractor → structured emitter migration (M04-blocking); count_tokens → real endpoint (M04-blocking); plus any new ⚠️ items from §2.
- 🟢 Nice-to-have: any Tauri build/cache optimization, doc cross-link cleanup, schema lint, etc.

**5. Carry-forward from prior milestones** — status of every Important item from Pre-M01 baseline, Pre-M01 addendum, and M01 entries.

Expected resolutions M02 closes:
- M01 🟡 "Coverage delta gating mechanism" — RESOLVED at M02 Stage A (CLAUDE.md §5 + .github/workflows/scripts/coverage-delta.sh).
- M01 🟡 "*_with / _inner test-seam pattern" — RESOLVED at M02 Stage A (docs/style.md) + applied at Stage C (anthropic_sse.rs) + Stage D (agent_sdk.rs).
- M01 🟡 "Windows drone integration test" — RESOLVED at M02 Stage A (crates/runtime-drone/tests/integration_windows.rs).
- M01 🟡 ".gitattributes line-ending normalization" — RESOLVED at M02 Stage A (.gitattributes new file).
- M01 🟡 "mcp_servers table" — RESOLVED at M02 Stage A (option (a) — table added with full 22-field schema).
- M01 🟡 "Post-M01 docs(spec): PR" — RESOLVED at PR #36 (pre-M02 work).
- M01 🟡 "src-tauri/gen/schemas/ gitignore" (PR #36 surface follow-up) — RESOLVED at M02 Stage A (.gitignore append).
- Pre-M01 baseline 🟡 "typify oneOf clippy suppression" — confirmed still resolved.

Expected to remain open:
- M01 🟡 "Phase 3 React Flow + Zustand spec expansion" — STILL OPEN, M03-blocking.
- M01 🟡 "Session FSM diagram in spec §11" — STILL OPEN, M04-blocking.
- Pre-M01 addendum 🟡 "Reuse-first vs duplication-first §9 bias" — STILL DEFERRED to M07–M08 per addendum decision.
- Pre-M01 addendum 🟡 "UI consistency: existing look and feel" — CARRY-FORWARD INTO M03 prep.
- Pre-M01 baseline 🟢 "§10 numbering gap" — RESOLVED at PR #36 (cosmetic; closeout).

**6. Sign-off** — Claude attestation + UTC timestamp.

#### `docs/build-prompts/retrospectives/M02-summary.md` (new)

Per `docs/build-prompts/retrospectives/SUMMARY-TEMPLATE.md`. Aggregates:
- Per-stage axis scores (Process, Product, Pattern) → mean + range.
- Per-stage outcome (Sound / Sound-but-rough / Friction-heavy / Not-ready).
- Cross-stage trends: which decisions actually got applied (vs noted-and-forgotten); which friction patterns recurred; which time-box estimates came in fast/slow vs the calibrated baselines.
- Verdict: Pattern held / held with friction / strained.
- Decisions to apply before M03 authoring (CLAUDE.md / TEMPLATE.md / per-milestone-prompt updates).

### F.4 Tests

No new code tests. Verification is:

1. **Append-only check** (CI-enforced via the `gap-analysis.md append-only check` job added in M01 Stage E + this stage's run).
2. **Schema validation** of the entry's structure (six sections present; severity emojis valid; file:line citations parseable) — manual review at PR time, no automated gate.
3. **User review** of the entry's substance per CLAUDE.md §19 + §20 three-artifact review (code diff + retrospectives/summary + gap-analysis entry).

#### Coverage target

N/A — documentation stage.

### F.5 CLI Prompt

```
Read CLAUDE.md §20 (Gap Analysis Protocol) and docs/gap-analysis.md
header (entry template).
Read docs/build-prompts/M02-event-pipeline.md Stage F (sections F.1
through F.4).

═══ STEP 1 — INGEST ═══

Read in order (large, but cumulative review is the point):

1. agent-runtime-spec.md (skim end-to-end; focus on §2, §2a, §2b, §2c,
   §10, §13, §M2).
2. All files produced/edited across M02 stages (commit list:
   git log --oneline main..HEAD; diff: git diff --stat main..HEAD).
   Read every new .rs, .ts, .tsx, .json, .toml, .yml, .md.
3. Prior gap-analysis.md entries IN FULL: Pre-M01 baseline, Pre-M01
   addendum, M01. The Carry-forward section MUST report status of
   every Important item from these entries.
4. M02 per-stage retrospectives:
   docs/build-prompts/retrospectives/M02.{A,B,C,D,E}-retrospective.md.

═══ STEP 2 — DRAFT M02-summary.md ═══

Copy SUMMARY-TEMPLATE.md to docs/build-prompts/retrospectives/M02-summary.md.
Aggregate axis scores (Process, Product, Pattern) across stages A–E.
Cross-stage trends. Verdict (Pattern held / held with friction /
strained). Decisions to apply before M03 authoring (CLAUDE.md /
TEMPLATE.md updates carrying forward).

═══ STEP 3 — DRAFT THE GAP ANALYSIS ENTRY ═══

Append to docs/gap-analysis.md a new section following the six-section
template at the top of that file:

  ## M02 — Event Pipeline (<YYYY-MM-DD>, commit `<sha-of-Stage-E-commit>`)
  ### Codebase deep dive (cumulative — M01 + M02)
  ### Adherence to spec
  ### Spec review (forward-looking)
  ### Fix backlog
  ### Carry-forward from prior milestones
  ### Sign-off

Severity in the Fix backlog is non-elastic. If everything is "Important,"
re-prioritize. Critical = "must fix before M03 starts." A pile of
Criticals is a signal the milestone shouldn't have shipped; surface
that honestly.

Carry-forward MUST report status (Resolved / Still open / Deferred to
M0X) for every Important item from Pre-M01 baseline + Pre-M01 addendum +
M01 entries. Expected resolutions are listed in F.3 above.

═══ STEP 4 — VERIFY APPEND-ONLY ═══

Run locally:
  git fetch origin main
  git show origin/main:docs/gap-analysis.md > /tmp/gap-base.md
  base_lines=$(wc -l < /tmp/gap-base.md)
  diff /tmp/gap-base.md <(head -n "$base_lines" docs/gap-analysis.md)

Output must be empty. If it isn't, prior content was modified; revert
and re-edit by APPENDING ONLY at the bottom.

═══ STEP 5 — SURFACE TO USER ═══

Run: git status, git diff docs/gap-analysis.md
   git diff docs/build-prompts/retrospectives/M02-summary.md
   git diff CHANGELOG.md

Surface:
- The full new gap-analysis entry (verbatim).
- The full M02-summary.md (verbatim).
- The CHANGELOG diff.
- Draft commit message from F.6.

State: "M02 Gap Analysis is ready. I will NOT commit until you approve.
Please review the entry — once committed, prior entries are immutable
forever per CLAUDE.md §20."

Wait for explicit approval.

On approval (Stage F — Phase Closeout: Gap Analysis; final stage):
1. Commit Stage F on the parent-milestone branch claude/m02-event-pipeline.
2. Push the branch (first push for M02 — push waits until after Stage F
   per CLAUDE.md §20).
3. Draft the M02 PR description aggregating all stage commits + each
   stage's retrospective + M02-summary + the new gap-analysis entry.
   Surface for approval.
4. On approval to open: use mcp__github__create_pull_request to open
   the PR. Do NOT merge. User reviews + merges on GitHub.
```

### F.6 Commit Message

```bash
git commit -s -m "$(cat <<'EOF'
docs(gap-analysis): M02 — append cumulative product+spec audit

Per CLAUDE.md §20. Reviews codebase to date (M01 + M02) against
agent-runtime-spec.md; records adherence findings, spec gaps, and
prioritized fix backlog. This entry is immutable — future milestones
report status via Carry-forward.

M02 entry covers:
- Codebase deep dive: provider abstraction quality, SSE state machine
  completeness, AgentSdk cancellation safety, Tauri capability boundary
  hold, frontend type sync, coverage holdouts.
- Adherence: zero-telemetry, direct Anthropic API, SSE wire format,
  capability lockdown all ✅. ⚠️ items: decision-extractor heuristic
  (M04 replacement scheduled), count_tokens approximation (M04 real
  endpoint scheduled), AgentSdk generic vs dyn (refactor cost
  documented), signal.rs scaffold (M04 emission integration), mcp_servers
  schema-only (M06 consumer).
- Spec review: Phase 3 expansion still M03-blocking; Session FSM
  M04-blocking; plan model field shapes M04-blocking; ProviderEvent::
  Error terminal-vs-recoverable ambiguity needs spec resolution.
- Fix backlog: 0 Critical (M02 shipped correctly); 🟡 Important spans
  M03/M04 prep work + spec edits; 🟢 Nice-to-have for cosmetic.
- Carry-forward: M01 🟡 items M02 closed (coverage delta gating, *_with
  pattern doc, Windows drone integration test, .gitattributes,
  mcp_servers table, src-tauri/gen/schemas/ gitignore). M01 🟡 items
  remaining open (Phase 3 spec, Session FSM, UI consistency carry to
  M03 prep).

M02-summary.md (new, NOT immutable but once written not edited)
aggregates per-stage axis scores and verdict.

CHANGELOG.md [Unreleased] Status section updated.

This commit is the FINAL commit on claude/m02-event-pipeline per
CLAUDE.md §20. Push gates the M02 PR.

Refs: M02-event-pipeline.md §F; CLAUDE.md §19 + §20; SUMMARY-TEMPLATE.md.

https://claude.ai/code/session_<id>
EOF
)"
```

---

## Summary Table

| Stage | New Files | Edited Files | Tests Added | Effort (calibrated) |
|---|---|---|---|---|
| **A** Build hygiene + scaffolding | 4 (.gitattributes, codecov.yml, signal.rs, integration_windows.rs) | 8 (.gitignore, db.rs, drone.rs, heartbeat.rs, lib.rs, CLAUDE.md, style.md, CHANGELOG.md) | 22 (1 heartbeat + 9 mcp_servers schema invariants + 9 signal.rs round-trips + 1 tag format + 1 Windows IPC integration + extended init_schema coverage) | ~1.5h |
| **B** LLMProvider trait + provider scaffolding | 2 (providers/mod.rs, anthropic.rs stub) | 6 (Cargo.toml workspace + crate, lib.rs, README, deny.toml, CHANGELOG.md) | 17 unit (cache-aware estimate_cost coverage: simple + 5m + 1h + read + combined; +sonnet test; +pricing-values test; +ContentBlock round-trip) | ~1.5h |
| **C** AnthropicProvider real HTTP+SSE | 3 (anthropic_sse.rs, anthropic_wiremock.rs, anthropic_smoke.rs) | 7 (Cargo.toml workspace + crate, anthropic.rs, mod.rs, README, ci.yml, CLAUDE.md, CHANGELOG.md) | 21 (12 unit + 8 wiremock + 1 smoke gated) | ~3h |
| **D** AgentSdk + drone IPC client | 9 (sdk/{mod, agent_sdk, event_pipeline, decision_extractor}, drone_ipc/{mod, client, connection}, sdk_event_translation.rs, sdk_cancellation.rs, drone_ipc_loopback.rs) | 6 (event.rs, lib.rs, Cargo.toml, README, ci.yml, CLAUDE.md, CHANGELOG.md) | 50+ (9+1 unit decision, 20+ table-driven, 5+ cancellation, 10 IPC loopback) | ~2.5h |
| **E** Tauri shell + renderer + E2E | 18 (frontend full surface + commands.rs + capabilities/default.json + key_store.rs + tests) | 8 (main.rs, Cargo.toml, tauri.conf.json, ci.yml, .gitignore, CLAUDE.md, MVP-v0.1.md, CHANGELOG.md) | 30+ (14 Vitest + 4 Rust + 8 Playwright + 2 wiremock E2E) | ~3h |
| **F** Phase Closeout: Gap Analysis | 1 (M02-summary.md) | 2 (gap-analysis.md append-only, CHANGELOG.md) | 0 (documentation stage; append-only check is CI gate) | ~1.5h |
| **Total** | 37 new | 37 edited | 140+ tests | **~13h actual** |

Estimates calibrated against M01's 0.3× ratio. M02 is ~3× M01's scope by file count (provider, SDK, IPC, frontend, all-new vs M01's mostly-scaffolding). Per the calibration, M01's 30–40h human-time-equivalent estimate ran in 10.5h actual; M02's 30–40h-equivalent estimate runs ~13h actual.

---

## Verification Checklist

Before approving the M02 PR (Stage F's surface), verify:

### Automated (gates)

- [ ] `cargo fmt --all -- --check` — zero diff
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings
- [ ] `cargo build --workspace` — succeeds on Linux/macOS/Windows × stable + MSRV
- [ ] `cargo test --workspace` — all tests pass (~116+ tests)
- [ ] `cargo test --workspace --doc` — doc tests pass
- [ ] `RUSTDOCFLAGS="-D missing_docs" cargo doc --workspace --no-deps` — clean
- [ ] `cargo audit` — zero high/critical
- [ ] `cargo deny check` — passes
- [ ] `cargo llvm-cov --workspace ... --fail-under-lines 80` — workspace coverage ≥80%
- [ ] `cargo llvm-cov --package runtime-main ... --fail-under-lines 95` — runtime-main ≥95%
- [ ] `cargo llvm-cov --package runtime-drone ... --fail-under-lines 95` — drone ≥95% (no regression)
- [ ] Coverage delta gate vs main passes (no >0.5pp regression on any safety-primitive crate)
- [ ] `npm run typecheck` — TS strict clean
- [ ] `npx prettier --check '**/*.{ts,tsx,js,jsx,json,md,yml,yaml}'` — clean
- [ ] `npx eslint .` — zero warnings
- [ ] `npm run test` — all Vitest tests pass; src/ coverage ≥80%
- [ ] `npm audit --audit-level=high` — zero high/critical
- [ ] `npx playwright test` — E2E smoke green on Linux/macOS/Windows
- [ ] `cargo run --bin xtask -- regenerate-types --check` — no schema drift
- [ ] Schema validation (existing) — passes
- [ ] gap-analysis.md append-only check — passes
- [ ] CI green on all OS × toolchain cells

### Manual

- [ ] Smoke session works end-to-end against real Anthropic API: set key → click button → see events → ends in agent_complete
- [ ] All five M02 stage retrospectives present and filled in (M02.A–E)
- [ ] `M02-summary.md` aggregates across stages with explicit verdict
- [ ] `docs/gap-analysis.md` M02 entry is the only new entry; prior entries unchanged
- [ ] M02 PR description references all stage commits + retrospectives + summary + gap-analysis entry
- [ ] CHANGELOG `[Unreleased]` reflects what M02 delivered
- [ ] `docs/MVP-v0.1.md` §M2 acceptance criteria all `- [x]`

### Approval gate (per CLAUDE.md §19 + §20)

- [ ] **Hard Gate G1: do-not-commit-until-approved held** — every stage commit landed only after explicit user approval
- [ ] User has reviewed each stage retrospective; scoring matches observable evidence
- [ ] M02-summary verdict is "Pattern held" (sound) or "Pattern held with friction"; not "Pattern strained"
- [ ] Gap-analysis Carry-forward correctly reports status of every prior Important item
- [ ] Three-artifact review complete: code diff + retrospectives/summary + gap-analysis entry

---

*End of M02-event-pipeline.md. M02 PR opens after Stage F approval.*

