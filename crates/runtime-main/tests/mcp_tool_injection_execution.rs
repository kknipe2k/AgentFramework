//! M09.D.fix — the canvas-authored MCP tool is SURFACED to the model.
//!
//! The M09.D maintainer IRL disproved the slice: an agent authored with
//! `allowed_tools: ["fs__read_text_file", "Write"]` (validates, canvas-wired,
//! `session_root_agent` set) ran in the Tester with **only the built-in
//! `Write`** in the tool list the model saw. `test_agent_config`
//! (`builder/tester.rs`) builds the model-facing tool list from
//! `builtin_tool_defs(&allowed)` + `request_capability` only — an
//! `allowed_tools` entry that names an MCP tool (`server__tool`) has **no
//! resolver and is silently dropped**, so the model is never told the tool
//! exists, never emits the call, and `try_mcp_dispatch` is never reached.
//! Dispatch was wired (M09.C `build_test_mcp_dispatcher`); the tool's
//! DEFINITION was never injected. (`vertical_slice.e2e.ts` ran tool-free,
//! `builder_mcp_tool.e2e.ts` was store-driven — no test hit this layer; the
//! real-model-meets-real-MCP IRL did — rule 11.)
//!
//! This assembled regression (v1.8 mandate) drives the REAL
//! `run_test_session_with_tools` → `AgentSdk::run_agent` multi-turn loop.
//! The only stubs are the provider (no live Anthropic — CLAUDE.md §10) and
//! the MCP dispatch source; the tool-list builder, the enforcer, the
//! filesystem Write, and the multi-turn feedback are all real. The stub
//! provider models a real model faithfully — it **only calls a tool it was
//! advertised** — so the end-to-end file write is causally gated on the
//! injection: with the def the read runs and the file lands; without it the
//! model calls nothing and no file appears.
//!
//! Grounded-claims (rule 11): the load-bearing assertions are the captured
//! model-facing tool list AND the observable side effect (the file on disk),
//! never an emitted event.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde_json::{json, Value};
use tempfile::TempDir;

use runtime_core::generated::framework::Framework;

use runtime_main::builder::run_test_session_with_tools;
use runtime_main::drone_ipc::DroneClient;
use runtime_main::providers::{
    AgentConfig, CostBreakdown, LLMProvider, Message, ModelInfo, ProviderError, ProviderEvent,
    ProviderSupport, ToolDef,
};
use runtime_main::sdk::{McpDispatchError, McpDispatchOutcome, McpToolDispatch, SessionId};
use runtime_main::tier::Tier;

use std::collections::BTreeMap;

/// The canonical `<server>__<tool>` id the canvas records in `allowed_tools`
/// and `try_mcp_dispatch` resolves; the injected def's name must match it.
const MCP_TOOL: &str = "fs__read_text_file";
/// The content the stub MCP source returns — the marker proving the Write's
/// content traces back to a real MCP dispatch (not a hardcoded constant).
const MCP_CONTENT: &str = "MCP-CONTENT-XYZ";

fn fwd(p: &std::path::Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// A one-agent framework whose `worker` declares the MCP tool + `Write` and a
/// write scope covering `write_glob`. `session_root_agent` is `worker`.
fn fw_with_mcp_tool(write_glob: &str) -> Framework {
    serde_json::from_value(json!({
        "name": "m09-d-fix-mcp-injection",
        "version": "1.0.0",
        "description": "M09.D.fix — author an MCP tool + Write, run it",
        "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
        "agents": [{
            "id": "worker",
            "role": "worker",
            "model": { "provider": "anthropic", "id": "claude-haiku-4-5" },
            "capabilities": {
                "tools_called": [MCP_TOOL],
                "skills_loaded": [],
                "file_access": { "read": [], "write": [write_glob] },
                "network": [],
                "shell": false,
                "spawn_agents": []
            },
            "allowed_tools": [MCP_TOOL, "Write"],
            "allowed_skills": [],
            "spawns": []
        }],
        "tools": [],
        "skills": [],
        "session_root_agent": "worker",
    }))
    .expect("the M09.D.fix fixture framework round-trips through the schema")
}

/// The model-facing definition the run path must inject for the authored
/// `server__tool` — the shape the Tauri shell builds from the connected
/// server's `list_tools` schema.
fn mcp_tool_def() -> ToolDef {
    ToolDef {
        name: MCP_TOOL.to_string(),
        description: "Read a text file from the fs MCP server".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": { "path": { "type": "string" } },
            "required": ["path"],
        }),
    }
}

/// A provider stub modelling a real model: turn 0 captures the advertised
/// tool list and calls the MCP read tool **only if it was advertised**; turn
/// 1 writes the content it read back **only if** the MCP result returned. A
/// model never calls a tool it was not told about, so both effects are gated
/// on the injection under test.
struct McpReadThenWriteStub {
    out_path: String,
    captured_tools: Arc<Mutex<Vec<String>>>,
    turn: Mutex<usize>,
}

#[async_trait]
impl LLMProvider for McpReadThenWriteStub {
    fn name(&self) -> &'static str {
        "m09-d-fix-mcp-stub"
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
        let n = {
            let mut t = self.turn.lock().expect("turn lock");
            let n = *t;
            *t += 1;
            n
        };
        if n == 0 {
            // Capture exactly what the model was told it could call.
            let names: Vec<String> = config.tools.iter().map(|t| t.name.clone()).collect();
            let advertised = names.iter().any(|name| name == MCP_TOOL);
            *self.captured_tools.lock().expect("tools lock") = names;
            if advertised {
                return Ok(Box::pin(futures::stream::iter(vec![
                    ProviderEvent::ToolUse {
                        id: "tu-read".to_string(),
                        name: MCP_TOOL.to_string(),
                        input: json!({ "path": "notes.txt" }),
                    },
                ])));
            }
            // Unadvertised → a real model cannot call it; the run ends here.
            return Ok(Box::pin(futures::stream::iter(vec![
                ProviderEvent::TextDelta {
                    text: "I have no read tool.".to_string(),
                },
                ProviderEvent::MessageStop {
                    stop_reason: "end_turn".to_string(),
                    total_tokens: None,
                },
            ])));
        }
        if n == 1 {
            // Only write if the MCP read actually fed its content back — the
            // marker proves the Write content came from the dispatch, not a
            // constant.
            let history = serde_json::to_string(&config.messages).unwrap_or_default();
            if history.contains(MCP_CONTENT) {
                return Ok(Box::pin(futures::stream::iter(vec![
                    ProviderEvent::ToolUse {
                        id: "tu-write".to_string(),
                        name: "Write".to_string(),
                        input: json!({ "path": self.out_path, "content": MCP_CONTENT }),
                    },
                ])));
            }
        }
        Ok(Box::pin(futures::stream::iter(vec![
            ProviderEvent::TextDelta {
                text: "done".to_string(),
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

/// A stub MCP dispatch source: resolves `fs__read_text_file` to fixed
/// content; everything else is "not an MCP tool" (the run loop falls through
/// to the built-in path). Mirrors the concrete dispatcher's `Invoked` shape.
struct StubMcpRead;

#[async_trait]
impl McpToolDispatch for StubMcpRead {
    async fn dispatch_if_mcp(
        &self,
        _agent_id: &str,
        tool_name: &str,
        _args: Value,
        _aliases: &BTreeMap<String, String>,
    ) -> Option<Result<McpDispatchOutcome, McpDispatchError>> {
        if tool_name == MCP_TOOL {
            return Some(Ok(McpDispatchOutcome::Invoked {
                server: "fs".to_string(),
                tool: "read_text_file".to_string(),
                value: json!(MCP_CONTENT),
            }));
        }
        None
    }
}

/// Scenario: an authored MCP tool's definition is injected into the model's
/// tool list, so the model calls it, the dispatch returns the content, and
/// the built-in Write lands the file from that real MCP data.
#[tokio::test]
async fn authored_mcp_tool_is_surfaced_to_the_model_and_drives_a_real_write() {
    let dir = TempDir::new().expect("tempdir");
    let out = dir.path().join("result.txt");
    let out_arg = format!("{}/result.txt", fwd(dir.path()));
    let write_glob = format!("{}/**", fwd(dir.path()));

    let fw = fw_with_mcp_tool(&write_glob);
    let captured = Arc::new(Mutex::new(Vec::new()));
    let provider = McpReadThenWriteStub {
        out_path: out_arg,
        captured_tools: Arc::clone(&captured),
        turn: Mutex::new(0),
    };

    let db_path = dir.path().join("runtime-tester.sqlite");
    let outcome = run_test_session_with_tools(
        &fw,
        "read notes.txt and write it to result.txt",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        Some(Arc::new(StubMcpRead)),
        SessionId::new(),
        Tier::Promoted,
        vec![mcp_tool_def()],
    )
    .await
    .expect("the assembled run completes");

    // (a) The direct assertion that fails on `main` today: the authored MCP
    // tool reaches the model-facing tool list.
    let tools = captured.lock().expect("tools lock").clone();
    assert!(
        tools.iter().any(|t| t == MCP_TOOL),
        "the authored MCP tool must be advertised to the model; got {tools:?}"
    );

    // (b) End-to-end: the model called it, the dispatch returned the content,
    // and the built-in Write landed the file from that real MCP data (the
    // on-disk side effect — rule 11, not an event).
    assert_eq!(
        std::fs::read_to_string(&out).expect("the in-scope Write produced the file"),
        MCP_CONTENT,
        "the written content traces to the MCP dispatch result"
    );
    assert!(outcome.passed, "an in-scope authored run is a clean pass");
}

/// Causality guard: WITHOUT the injected def the model is told nothing about
/// the MCP tool, so it calls nothing and writes nothing. Proves the injection
/// (not some other path) is what makes the slice run — the mutation the
/// blocking gate must catch.
#[tokio::test]
async fn without_the_injected_def_the_model_never_sees_the_mcp_tool() {
    let dir = TempDir::new().expect("tempdir");
    let out = dir.path().join("result.txt");
    let out_arg = format!("{}/result.txt", fwd(dir.path()));
    let write_glob = format!("{}/**", fwd(dir.path()));

    let fw = fw_with_mcp_tool(&write_glob);
    let captured = Arc::new(Mutex::new(Vec::new()));
    let provider = McpReadThenWriteStub {
        out_path: out_arg,
        captured_tools: Arc::clone(&captured),
        turn: Mutex::new(0),
    };

    let db_path = dir.path().join("runtime-tester.sqlite");
    let _ = run_test_session_with_tools(
        &fw,
        "read notes.txt and write it to result.txt",
        &db_path,
        provider,
        Arc::new(DroneClient::noop()),
        Some(Arc::new(StubMcpRead)),
        SessionId::new(),
        Tier::Promoted,
        Vec::new(), // no MCP tool defs injected — the pre-M09.D.fix reality
    )
    .await
    .expect("the assembled run completes");

    let tools = captured.lock().expect("tools lock").clone();
    assert!(
        !tools.iter().any(|t| t == MCP_TOOL),
        "with no injected def the MCP tool is absent from the model list; got {tools:?}"
    );
    assert!(
        !out.exists(),
        "the model cannot call an unadvertised tool, so no file is written"
    );
}
