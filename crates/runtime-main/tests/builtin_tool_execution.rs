//! M08.7.A rung 1 — built-in tool execution assembled regression + unit
//! contract.
//!
//! The cluster-gate close contract (`docs/cluster-pattern.md` §1/§4): the
//! assembled tests drive the REAL `run_test_session_with` →
//! `AgentSdk::run_agent` multi-turn loop against a real `tempfile`
//! workspace fixture. The ONLY stub is the provider (no live Anthropic —
//! CLAUDE.md §10); the executor, the capability enforcer (with grants
//! loaded from the framework's `file_access` — the §1.3-B closure), the
//! filesystem read, and the multi-turn feedback are all real.
//!
//! Grounded-claims (CLAUDE.md §4 rule 11 / gotcha #66): the load-bearing
//! assertion is NOT "a `ToolInvoked` event fired" — it is that the agent's
//! NEXT provider turn RECEIVES the file contents as a `tool_result`
//! (observed on the real `AgentConfig` the loop built) AND the final text
//! quotes `ladder-rung-1`. Injecting a `ToolUse` and asserting the emitted
//! event is the exact trap that hid the built-in-tools-don't-execute gap.

use std::str::FromStr;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde_json::{json, Value};
use tempfile::TempDir;

use runtime_core::event::{AgentEvent, CapabilityKindRef, ToolSource};
use runtime_core::generated::capability::{
    CapabilityDeclaration, CapabilityKind, CapabilityScope, PathPattern, ResourceName,
    SideEffectClass,
};
use runtime_core::generated::framework::Framework;

use runtime_main::builder::run_test_session_with;
use runtime_main::capability::{CapabilityEnforcer, CapabilityError, DenyReason};
use runtime_main::drone_ipc::DroneClient;
use runtime_main::framework_loader::grant_framework_capabilities;
use runtime_main::providers::{
    AgentConfig, ContentBlock, CostBreakdown, LLMProvider, Message, ModelInfo, ProviderError,
    ProviderEvent, ProviderSupport, ToolResultContent,
};
use runtime_main::sdk::builtin_tools::{
    builtin_tool_defs, execute_builtin, is_builtin_tool, BuiltinExecError,
};
use runtime_main::sdk::SessionId;
use runtime_main::tier::Tier;

// ── helpers ───────────────────────────────────────────────────────────

/// Forward-slash a path so the same string is both a valid `std::fs`
/// argument (Windows accepts `/`) and a stable `globset` match target
/// (the existing `scope_contains` tests prove `dir/**` matches
/// `dir/file`).
fn fwd(p: &std::path::Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// A schema-valid one-agent framework whose `worker` agent declares the
/// given `file_access` globs + `allowed_tools`. `session_root_agent` is
/// `worker`, so the run's dispatch agent id is `worker`.
fn one_agent_fw(read: &[&str], write: &[&str], allowed_tools: &[&str]) -> Framework {
    serde_json::from_value(json!({
        "name": "m08-7-a-rung1",
        "version": "1.0.0",
        "description": "M08.7.A rung 1 built-in tool execution fixture",
        "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
        "agents": [{
            "id": "worker",
            "role": "worker",
            "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
            "capabilities": {
                "tools_called": [],
                "skills_loaded": [],
                "file_access": { "read": read, "write": write },
                "network": [],
                "shell": false,
                "spawn_agents": []
            },
            "allowed_tools": allowed_tools,
            "allowed_skills": [],
            "spawns": []
        }],
        "tools": [],
        "skills": [],
        "session_root_agent": "worker",
    }))
    .expect("the rung-1 fixture framework round-trips through the schema")
}

/// Build an enforcer populated from a framework's `file_access` via the
/// production grant pipeline (`grant_framework_capabilities` — the §1.3-B
/// closure), at the given tier.
fn enforcer_for(read: &[&str], write: &[&str], tier: Tier) -> CapabilityEnforcer {
    let fw = one_agent_fw(read, write, &[]);
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(tier);
    grant_framework_capabilities(&mut enforcer, &fw);
    enforcer
}

/// A `Read` request declaration mirroring what the executor builds — the
/// `Path`-scoped shape `subsumes` matches against a `file_access.read`
/// glob grant.
fn read_decl(path: &str) -> CapabilityDeclaration {
    CapabilityDeclaration {
        kind: CapabilityKind::Read,
        resource: ResourceName::from_str("filesystem").expect("non-empty resource"),
        scope: CapabilityScope::Path(PathPattern::from_str(path).expect("non-empty path")),
        side_effect_class: SideEffectClass::Pure,
    }
}

/// The latest `tool_result` text anywhere in a config's message history.
fn latest_tool_result_text(config: &AgentConfig) -> Option<String> {
    config.messages.iter().rev().find_map(|m| {
        m.content.iter().rev().find_map(|b| match b {
            ContentBlock::ToolResult {
                content: ToolResultContent::Text(t),
                ..
            } => Some(t.clone()),
            _ => None,
        })
    })
}

/// The full concatenated `StreamText` the run emitted (the agent's
/// user-visible output).
fn final_text(trace: &[AgentEvent]) -> String {
    trace
        .iter()
        .filter_map(|e| match e {
            AgentEvent::StreamText { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
}

/// The first `CapabilityViolation` in the trace, as `(kind, action)`.
fn first_violation(trace: &[AgentEvent]) -> Option<(CapabilityKindRef, String)> {
    trace.iter().find_map(|e| match e {
        AgentEvent::CapabilityViolation {
            capability_kind,
            requested_action,
            ..
        } => Some((*capability_kind, requested_action.clone())),
        _ => None,
    })
}

// ── provider stub (no live Anthropic) ─────────────────────────────────

/// On turn 1 emits a scripted `ToolUse`; on turn 2+ ECHOES whatever
/// `tool_result` text it receives in `config.messages` back as stream
/// text — a faithful stand-in for a model that quotes the file it was
/// handed. CAPTURES every config it is given so a test can observe what
/// the loop fed back (if the executor never fed a result back, the
/// capture is empty and the echo is empty — the grounded probe).
struct ScriptedToolStub {
    first_tool_name: String,
    first_tool_input: Value,
    seen: Arc<Mutex<Vec<AgentConfig>>>,
    turn: Mutex<usize>,
}

impl ScriptedToolStub {
    fn new(name: &str, input: Value, seen: Arc<Mutex<Vec<AgentConfig>>>) -> Self {
        Self {
            first_tool_name: name.to_string(),
            first_tool_input: input,
            seen,
            turn: Mutex::new(0),
        }
    }
}

#[async_trait]
impl LLMProvider for ScriptedToolStub {
    fn name(&self) -> &'static str {
        "m08-7-a-scripted-stub"
    }
    fn supports(&self) -> ProviderSupport {
        ProviderSupport {
            tool_use: true,
            streaming: true,
            thinking: false,
        }
    }
    async fn stream(
        &self,
        config: AgentConfig,
    ) -> Result<BoxStream<'_, ProviderEvent>, ProviderError> {
        self.seen.lock().expect("seen lock").push(config.clone());
        let n = {
            let mut t = self.turn.lock().expect("turn lock");
            let n = *t;
            *t += 1;
            n
        };
        if n == 0 {
            return Ok(Box::pin(futures::stream::iter(vec![
                ProviderEvent::ToolUse {
                    id: "tu-1".to_string(),
                    name: self.first_tool_name.clone(),
                    input: self.first_tool_input.clone(),
                },
            ])));
        }
        let echoed = latest_tool_result_text(&config).unwrap_or_default();
        Ok(Box::pin(futures::stream::iter(vec![
            ProviderEvent::TextDelta {
                text: format!("The file says: {echoed}"),
            },
            ProviderEvent::MessageStop {
                stop_reason: "end_turn".to_string(),
                total_tokens: None,
            },
        ])))
    }
    async fn count_tokens(&self, _m: &[Message]) -> Result<u64, ProviderError> {
        Ok(0)
    }
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        Ok(Vec::new())
    }
    fn estimate_cost(&self, _b: &CostBreakdown, _m: &str) -> f64 {
        0.0
    }
}

// ── assembled regressions (the §1.4 BDD close contract) ───────────────

/// Scenario: Read feeds file contents back into the agent's turn.
///
/// Given a one-agent framework whose agent has Read in `allowed_tools`,
/// with `file_access.read` covering a workspace file `hello.txt`
/// containing `ladder-rung-1`, when the framework runs, THEN a
/// `ToolInvoked(Read)` is emitted, the agent's NEXT turn receives the
/// contents as a `tool_result`, and the final text contains
/// `ladder-rung-1`.
#[tokio::test]
async fn read_feeds_file_contents_back_and_agent_quotes_them() {
    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("hello.txt");
    std::fs::write(&file, "ladder-rung-1").expect("write fixture file");
    let dir_glob = format!("{}/**", fwd(dir.path()));
    let path_arg = format!("{}/hello.txt", fwd(dir.path()));

    let fw = one_agent_fw(&[&dir_glob], &[], &["Read"]);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let provider = ScriptedToolStub::new("Read", json!({ "path": path_arg }), Arc::clone(&seen));

    let db_path = dir.path().join("runtime-tester.sqlite");
    let outcome = run_test_session_with(
        &fw,
        "read hello.txt and quote it",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
    )
    .await
    .expect("the assembled rung-1 run completes");

    // (a) the built-in actually ran — an agent-correct built-in ToolInvoked.
    assert!(
        outcome.trace.iter().any(|e| matches!(
            e,
            AgentEvent::ToolInvoked { tool_name, source: ToolSource::Builtin, .. }
                if tool_name == "Read"
        )),
        "a built-in Read ToolInvoked must be emitted; trace={:?}",
        outcome.trace
    );

    // (b) LOAD-BEARING (rule 11 / gotcha #66): the NEXT turn received the
    // file contents as a tool_result — observed on the REAL config the
    // multi-turn loop built, not from an emitted event.
    let configs: Vec<AgentConfig> = seen.lock().expect("seen lock").clone();
    assert!(
        configs.len() >= 2,
        "the loop must re-stream a 2nd turn after the tool ran; got {} turn(s)",
        configs.len()
    );
    let fed_back = latest_tool_result_text(&configs[1]).unwrap_or_default();
    assert!(
        fed_back.contains("ladder-rung-1"),
        "turn 2 must receive the file contents as a tool_result; got {fed_back:?}"
    );

    // (c) the agent's final text quotes the file.
    let text = final_text(&outcome.trace);
    assert!(
        text.contains("ladder-rung-1"),
        "the agent's final text must quote the file; got {text:?}"
    );
}

/// A read outside the agent's `file_access` scope is denied through the
/// built-in executor (`capability_kind == Read` — the pre-rung-1
/// `ToolNotFound` fallback emits `Exec`, so this distinguishes the
/// executor path) and never produces a `tool_result`.
#[tokio::test]
async fn read_outside_file_access_scope_is_denied_and_not_executed() {
    let dir = TempDir::new().expect("tempdir");
    std::fs::write(dir.path().join("secret.txt"), "top-secret").expect("write fixture");
    let path_arg = format!("{}/secret.txt", fwd(dir.path()));

    // Grant covers a DIFFERENT subtree — the request path is outside it.
    let fw = one_agent_fw(&["allowed/**"], &[], &["Read"]);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let provider = ScriptedToolStub::new("Read", json!({ "path": path_arg }), Arc::clone(&seen));

    let db_path = dir.path().join("runtime-tester.sqlite");
    let outcome = run_test_session_with(
        &fw,
        "read the secret file",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
    )
    .await
    .expect("the assembled run completes (a denial is a failed test, not Err)");

    let (kind, _action) = first_violation(&outcome.trace)
        .expect("an out-of-scope read must emit CapabilityViolation");
    assert_eq!(
        kind,
        CapabilityKindRef::Read,
        "the violation came from the built-in Read executor (Read), not the ToolNotFound fallback (Exec)"
    );
    assert!(
        !outcome
            .trace
            .iter()
            .any(|e| matches!(e, AgentEvent::ToolResult { .. })),
        "a denied read must NOT produce a ToolResult — it never ran"
    );
    assert!(
        !outcome.passed,
        "a capability violation fails the test outcome"
    );
}

/// A Write under the v0.1 default Novice tier is blocked at L4
/// (`TierViolation`) and creates no file — the capability scope is the
/// boundary; the op never touches disk.
#[tokio::test]
async fn write_under_novice_tier_is_blocked_and_creates_no_file() {
    let dir = TempDir::new().expect("tempdir");
    let target = dir.path().join("out.txt");
    let path_arg = format!("{}/out.txt", fwd(dir.path()));
    let dir_glob = format!("{}/**", fwd(dir.path()));

    // The agent declares write within scope — but Novice forbids Write at
    // L4 before the grant is even consulted.
    let fw = one_agent_fw(&[], &[&dir_glob], &["Write"]);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let provider = ScriptedToolStub::new(
        "Write",
        json!({ "path": path_arg, "content": "should-not-be-written" }),
        Arc::clone(&seen),
    );

    let db_path = dir.path().join("runtime-tester.sqlite");
    let outcome = run_test_session_with(
        &fw,
        "write out.txt",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
    )
    .await
    .expect("the assembled run completes");

    assert!(
        outcome
            .trace
            .iter()
            .any(|e| matches!(e, AgentEvent::TierViolation { .. })),
        "a Novice-tier Write must emit TierViolation; trace={:?}",
        outcome.trace
    );
    assert!(
        !target.exists(),
        "the blocked write must create no file on disk"
    );
}

// ── executor unit contract ────────────────────────────────────────────

#[test]
fn is_builtin_tool_recognizes_read_and_write_only() {
    assert!(is_builtin_tool("Read"));
    assert!(is_builtin_tool("Write"));
    assert!(!is_builtin_tool("Bash"));
    assert!(!is_builtin_tool("fs__read"));
}

#[test]
fn execute_read_inside_scope_returns_file_content() {
    let dir = TempDir::new().expect("tempdir");
    std::fs::write(dir.path().join("hello.txt"), "ladder-rung-1").expect("write");
    let glob = format!("{}/**", fwd(dir.path()));
    let path_arg = format!("{}/hello.txt", fwd(dir.path()));
    let enforcer = enforcer_for(&[&glob], &[], Tier::Novice);

    let out = execute_builtin(&enforcer, "worker", "Read", &json!({ "path": path_arg }))
        .expect("an in-scope read executes");
    assert_eq!(
        out.get("content").and_then(Value::as_str),
        Some("ladder-rung-1"),
        "the read returns the file contents to feed back"
    );
}

#[test]
fn execute_read_outside_scope_is_denied_no_matching_grant() {
    let dir = TempDir::new().expect("tempdir");
    std::fs::write(dir.path().join("hello.txt"), "x").expect("write");
    let path_arg = format!("{}/hello.txt", fwd(dir.path()));
    let enforcer = enforcer_for(&["elsewhere/**"], &[], Tier::Novice);

    let err = execute_builtin(&enforcer, "worker", "Read", &json!({ "path": path_arg }))
        .expect_err("an out-of-scope read is denied, never executed");
    assert!(
        matches!(
            err,
            BuiltinExecError::Capability(CapabilityError::Denied {
                reason: DenyReason::NoMatchingGrant,
                ..
            })
        ),
        "got {err:?}"
    );
}

#[test]
fn execute_read_with_no_grants_is_denied_no_declarations() {
    let dir = TempDir::new().expect("tempdir");
    std::fs::write(dir.path().join("hello.txt"), "x").expect("write");
    let path_arg = format!("{}/hello.txt", fwd(dir.path()));
    let enforcer = enforcer_for(&[], &[], Tier::Novice);

    let err = execute_builtin(&enforcer, "worker", "Read", &json!({ "path": path_arg }))
        .expect_err("an agent with no grants default-denies");
    assert!(
        matches!(
            err,
            BuiltinExecError::Capability(CapabilityError::Denied {
                reason: DenyReason::NoDeclarations,
                ..
            })
        ),
        "got {err:?}"
    );
}

#[test]
fn execute_write_inside_scope_promoted_writes_file() {
    let dir = TempDir::new().expect("tempdir");
    let target = dir.path().join("out.txt");
    let path_arg = format!("{}/out.txt", fwd(dir.path()));
    let glob = format!("{}/**", fwd(dir.path()));
    let enforcer = enforcer_for(&[], &[&glob], Tier::Promoted);

    let out = execute_builtin(
        &enforcer,
        "worker",
        "Write",
        &json!({ "path": path_arg, "content": "written-bytes" }),
    )
    .expect("an in-scope Promoted write executes");
    assert_eq!(out.get("ok").and_then(Value::as_bool), Some(true));
    assert_eq!(
        std::fs::read_to_string(&target).expect("the file was written"),
        "written-bytes",
        "the write produced the file content on disk"
    );
}

#[test]
fn execute_write_under_novice_is_tier_forbidden_and_writes_no_file() {
    let dir = TempDir::new().expect("tempdir");
    let target = dir.path().join("out.txt");
    let path_arg = format!("{}/out.txt", fwd(dir.path()));
    let glob = format!("{}/**", fwd(dir.path()));
    let enforcer = enforcer_for(&[], &[&glob], Tier::Novice);

    let err = execute_builtin(
        &enforcer,
        "worker",
        "Write",
        &json!({ "path": path_arg, "content": "x" }),
    )
    .expect_err("Novice forbids Write at L4");
    assert!(
        matches!(
            err,
            BuiltinExecError::Capability(CapabilityError::TierForbidden { .. })
        ),
        "got {err:?}"
    );
    assert!(
        !target.exists(),
        "a tier-forbidden write must create no file"
    );
}

#[test]
fn execute_missing_path_is_an_op_error() {
    let enforcer = enforcer_for(&["**"], &[], Tier::Novice);
    let err = execute_builtin(&enforcer, "worker", "Read", &json!({}))
        .expect_err("missing 'path' is a malformed-input op error");
    assert!(matches!(err, BuiltinExecError::Op(_)), "got {err:?}");
}

#[test]
fn builtin_tool_defs_advertises_only_builtins() {
    let defs = builtin_tool_defs(&[
        "Read".to_string(),
        "Write".to_string(),
        "fs__list".to_string(),
    ]);
    let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(
        names.len(),
        2,
        "only the in-process built-ins are advertised; got {names:?}"
    );
    assert!(names.contains(&"Read") && names.contains(&"Write"));
    for d in &defs {
        assert!(
            !d.description.is_empty(),
            "each advertised ToolDef carries a description"
        );
        assert_eq!(
            d.input_schema.get("type").and_then(Value::as_str),
            Some("object"),
            "the input_schema is a JSON-Schema object"
        );
    }
}

#[test]
fn builtin_tool_def_shape_is_keyed_per_name_read_vs_write() {
    // Mutation-kill (builtin_tools.rs:165 `name == WRITE_TOOL`): the prior
    // `builtin_tool_defs_advertises_only_builtins` test only asserts the SET
    // of advertised names, so flipping `==`→`!=` (which SWAPS the Read/Write
    // shapes) leaves both names present and survives. This pins each name to
    // its OWN def — a single-name call must return exactly the def for THAT
    // name, with the matching input-schema shape.
    let read = builtin_tool_defs(&["Read".to_string()]);
    assert_eq!(read.len(), 1, "one name in → one def out; got {read:?}");
    assert_eq!(read[0].name, "Read", "Read must advertise the Read def");
    assert!(
        !read[0]
            .input_schema
            .get("required")
            .and_then(Value::as_array)
            .is_some_and(|r| r.iter().any(|v| v.as_str() == Some("content"))),
        "the Read shape takes only `path`, never `content`; got {:?}",
        read[0].input_schema
    );

    let write = builtin_tool_defs(&["Write".to_string()]);
    assert_eq!(write.len(), 1, "one name in → one def out; got {write:?}");
    assert_eq!(write[0].name, "Write", "Write must advertise the Write def");
    assert!(
        write[0]
            .input_schema
            .get("required")
            .and_then(Value::as_array)
            .is_some_and(|r| r.iter().any(|v| v.as_str() == Some("content"))),
        "the Write shape requires `content`; got {:?}",
        write[0].input_schema
    );
}

#[test]
fn grant_framework_capabilities_loads_file_access_into_enforcer() {
    // Finding B closure: the enforcer is empty (default-deny) until grants
    // load. After loading from file_access, an in-scope read passes the
    // L1 check and an out-of-scope read is denied — proving the grant
    // pipeline the built-in executor depends on is wired.
    let enforcer = enforcer_for(&["src/**"], &[], Tier::Novice);
    enforcer
        .check("worker", &read_decl("src/lib.rs"))
        .expect("an in-scope read is granted after capabilities load");
    assert!(
        enforcer.check("worker", &read_decl("docs/x.md")).is_err(),
        "an out-of-scope read is not granted"
    );
}

// ── Op-error path (M08.7.A follow-up: cover the post-check failure arms) ─

#[test]
fn execute_write_missing_content_is_an_op_error() {
    let dir = TempDir::new().expect("tempdir");
    let path_arg = format!("{}/out.txt", fwd(dir.path()));
    let glob = format!("{}/**", fwd(dir.path()));
    let enforcer = enforcer_for(&[], &[&glob], Tier::Promoted);
    let err = execute_builtin(&enforcer, "worker", "Write", &json!({ "path": path_arg }))
        .expect_err("Write without 'content' is a malformed-input op error");
    assert!(matches!(err, BuiltinExecError::Op(_)), "got {err:?}");
}

#[test]
fn execute_empty_path_is_an_op_error() {
    // An empty `path` string passes the `get("path")` presence check but
    // fails PathPattern construction (min length 1) — the file_decl Op arm.
    let enforcer = enforcer_for(&["**"], &[], Tier::Novice);
    let err = execute_builtin(&enforcer, "worker", "Read", &json!({ "path": "" }))
        .expect_err("an empty path is an op error, not a panic");
    assert!(matches!(err, BuiltinExecError::Op(_)), "got {err:?}");
}

#[test]
fn execute_non_builtin_tool_name_is_an_op_error() {
    // execute_builtin is public; a direct call with a non-in-process name
    // (the run loop gates on is_builtin_tool, but the executor defends
    // itself) returns the "not an in-process built-in" op error.
    let enforcer = enforcer_for(&["**"], &[], Tier::Novice);
    let err = execute_builtin(&enforcer, "worker", "Bash", &json!({ "path": "x" }))
        .expect_err("Bash is not an in-process built-in");
    assert!(matches!(err, BuiltinExecError::Op(_)), "got {err:?}");
}

/// A read of a MISSING file inside scope: the capability check passes, the
/// filesystem read fails, and the executor feeds an error `tool_result`
/// back so the multi-turn loop survives (the model can recover) rather
/// than breaking. Covers the `dispatch_builtin` Op arm end-to-end through
/// the real run loop.
#[tokio::test]
async fn read_of_a_missing_file_in_scope_feeds_an_error_result_back() {
    let dir = TempDir::new().expect("tempdir");
    let dir_glob = format!("{}/**", fwd(dir.path()));
    // The path is inside the read scope, but no such file exists on disk.
    let path_arg = format!("{}/ghost.txt", fwd(dir.path()));

    let fw = one_agent_fw(&[&dir_glob], &[], &["Read"]);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let provider = ScriptedToolStub::new("Read", json!({ "path": path_arg }), Arc::clone(&seen));

    let db_path = dir.path().join("runtime-tester.sqlite");
    let outcome = run_test_session_with(
        &fw,
        "read the missing file",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        None,
        SessionId::new(),
    )
    .await
    .expect("the assembled run completes");

    // The op was attempted (ToolInvoked emitted) — the capability check
    // passed; only the filesystem read failed.
    assert!(
        outcome.trace.iter().any(|e| matches!(
            e,
            AgentEvent::ToolInvoked { tool_name, source: ToolSource::Builtin, .. }
                if tool_name == "Read"
        )),
        "an in-scope read attempt emits a built-in ToolInvoked; trace={:?}",
        outcome.trace
    );
    // It was NOT a capability denial.
    assert!(
        !outcome
            .trace
            .iter()
            .any(|e| matches!(e, AgentEvent::CapabilityViolation { .. })),
        "an in-scope read is not a capability violation; trace={:?}",
        outcome.trace
    );
    // An error tool_result was emitted and fed back (the loop re-streamed).
    let error_result = outcome.trace.iter().any(|e| {
        matches!(
            e,
            AgentEvent::ToolResult { output, .. } if output.get("error").is_some()
        )
    });
    assert!(
        error_result,
        "a failed read feeds an error tool_result back; trace={:?}",
        outcome.trace
    );
    let configs = seen.lock().expect("seen lock").len();
    assert!(
        configs >= 2,
        "the loop survives a tool error and re-streams a 2nd turn; got {configs} turn(s)"
    );
}
