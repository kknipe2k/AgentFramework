//! End-to-end MCP dispatch through the §5a resolver + L1+L4 capability
//! gates + audit — M06.D (ADR-0010 concrete-impl side).
//!
//! Per gotcha #66: the deny test asserts BOTH the returned outcome AND
//! the audit-log file content (file inspection, not just a method-call
//! assertion). Per gotcha #69: a dispatch-twice multi-call invariant.

#![cfg(feature = "test-helpers")]

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use runtime_main::audit::AuditWriter;
use runtime_main::capability::CapabilityEnforcer;
use runtime_main::sdk::{McpDispatchOutcome, McpToolDispatch};
use runtime_main::tier::Tier;
use runtime_mcp::transport::{Connection, MockTransport, Transport};
use runtime_mcp::{
    mcp_tool_capability, ConnectionResolver, McpDispatcher, McpError, NamespaceResolver,
};
use serde_json::json;
use tempfile::tempdir;
use tokio::sync::RwLock;

/// A `ConnectionResolver` backed by one scripted `MockTransport`.
struct MockResolver {
    transport: MockTransport,
}

#[async_trait]
impl ConnectionResolver for MockResolver {
    async fn connection(&self, _server: &str) -> Result<Arc<dyn Connection>, McpError> {
        Ok(Arc::from(self.transport.connect().await?))
    }
}

fn resolver_with(pairs: &[(&str, &[&str])]) -> Arc<RwLock<NamespaceResolver>> {
    let connected: BTreeMap<String, Vec<String>> = pairs
        .iter()
        .map(|(s, tools)| {
            (
                (*s).to_string(),
                tools.iter().map(|t| (*t).to_string()).collect(),
            )
        })
        .collect();
    Arc::new(RwLock::new(NamespaceResolver::new(connected)))
}

fn no_aliases() -> BTreeMap<String, String> {
    BTreeMap::new()
}

#[tokio::test]
async fn mcp_tool_dispatch_with_valid_grant_succeeds_and_emits_tool_invoked() {
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(Tier::Promoted); // Exec passes L4 under Promoted.
    enforcer.grant("worker", mcp_tool_capability("pdf-mcp", "extract_text"));

    let transport = MockTransport::new()
        .with_tool("extract_text", None, json!({"type": "object"}))
        .with_tool_result("extract_text", json!({"text": "hello"}));

    let dispatcher = McpDispatcher::new(
        resolver_with(&[("pdf-mcp", &["extract_text"])]),
        Arc::new(enforcer),
        Arc::new(MockResolver { transport }),
        None,
        "sess-1",
    );

    let outcome = dispatcher
        .dispatch_if_mcp("worker", "pdf-mcp__extract_text", json!({}), &no_aliases())
        .await
        .expect("MCP tool resolved → Some")
        .expect("dispatch did not error");

    match outcome {
        McpDispatchOutcome::Invoked {
            server,
            tool,
            value,
        } => {
            assert_eq!(server, "pdf-mcp");
            assert_eq!(tool, "extract_text");
            assert_eq!(value, json!({"text": "hello"}));
        }
        other => panic!("expected Invoked, got {other:?}"),
    }
}

#[tokio::test]
async fn mcp_tool_dispatch_missing_grant_emits_capability_violation_and_mcp_request_blocked() {
    // No grant for "worker" → default-deny. Outcome is Blocked carrying
    // the resolved server + tool context.
    let enforcer = CapabilityEnforcer::new();
    let transport = MockTransport::new().with_tool("extract_text", None, json!({}));

    let dispatcher = McpDispatcher::new(
        resolver_with(&[("pdf-mcp", &["extract_text"])]),
        Arc::new(enforcer),
        Arc::new(MockResolver { transport }),
        None,
        "sess-1",
    );

    let outcome = dispatcher
        .dispatch_if_mcp("worker", "pdf-mcp__extract_text", json!({}), &no_aliases())
        .await
        .expect("MCP tool resolved → Some")
        .expect("a capability deny is a resolved outcome, not a dispatch error");

    match outcome {
        McpDispatchOutcome::Blocked {
            agent_id,
            server,
            tool,
            reason,
        } => {
            assert_eq!(agent_id, "worker");
            assert_eq!(server, "pdf-mcp");
            assert_eq!(tool, "extract_text");
            assert!(!reason.is_empty(), "deny reason must be human-readable");
        }
        other => panic!("expected Blocked, got {other:?}"),
    }
}

#[tokio::test]
async fn mcp_tool_dispatch_ambiguous_short_name_emits_tool_alias_ambiguous() {
    let enforcer = CapabilityEnforcer::new();
    let transport = MockTransport::new().with_tool("extract_text", None, json!({}));

    let dispatcher = McpDispatcher::new(
        resolver_with(&[
            ("pdf-mcp", &["extract_text"]),
            ("image-mcp", &["extract_text"]),
        ]),
        Arc::new(enforcer),
        Arc::new(MockResolver { transport }),
        None,
        "sess-1",
    );

    let outcome = dispatcher
        .dispatch_if_mcp("worker", "extract_text", json!({}), &no_aliases())
        .await
        .expect("ambiguous IS an MCP tool → Some")
        .expect("ambiguity is a resolved outcome, not a dispatch error");

    match outcome {
        McpDispatchOutcome::Ambiguous {
            name,
            mut candidates,
        } => {
            assert_eq!(name, "extract_text");
            candidates.sort();
            assert_eq!(
                candidates,
                vec![
                    "image-mcp__extract_text".to_string(),
                    "pdf-mcp__extract_text".to_string(),
                ]
            );
        }
        other => panic!("expected Ambiguous, got {other:?}"),
    }
}

#[tokio::test]
async fn mcp_tool_dispatch_with_alias_succeeds() {
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(Tier::Promoted);
    enforcer.grant("worker", mcp_tool_capability("pdf-mcp", "extract_text"));

    let transport = MockTransport::new()
        .with_tool("extract_text", None, json!({}))
        .with_tool_result("extract_text", json!({"ok": true}));

    let dispatcher = McpDispatcher::new(
        resolver_with(&[
            ("pdf-mcp", &["extract_text"]),
            ("image-mcp", &["extract_text"]),
        ]),
        Arc::new(enforcer),
        Arc::new(MockResolver { transport }),
        None,
        "sess-1",
    );

    let mut aliases = BTreeMap::new();
    aliases.insert(
        "extract_text".to_string(),
        "pdf-mcp__extract_text".to_string(),
    );

    let outcome = dispatcher
        .dispatch_if_mcp("worker", "extract_text", json!({}), &aliases)
        .await
        .expect("alias resolves → Some")
        .expect("dispatch did not error");

    match outcome {
        McpDispatchOutcome::Invoked { server, tool, .. } => {
            assert_eq!(server, "pdf-mcp");
            assert_eq!(tool, "extract_text");
        }
        other => panic!("expected Invoked via alias, got {other:?}"),
    }
}

#[tokio::test]
async fn non_mcp_tool_falls_through_to_default_dispatch_path() {
    // "Read" is a builtin, not an MCP tool. dispatch_if_mcp returns
    // None so the SDK falls through to the Stage A non-MCP L1 path.
    let enforcer = CapabilityEnforcer::new();
    let transport = MockTransport::new().with_tool("extract_text", None, json!({}));

    let dispatcher = McpDispatcher::new(
        resolver_with(&[("pdf-mcp", &["extract_text"])]),
        Arc::new(enforcer),
        Arc::new(MockResolver { transport }),
        None,
        "sess-1",
    );

    let result = dispatcher
        .dispatch_if_mcp(
            "worker",
            "Read",
            json!({"path": "src/lib.rs"}),
            &no_aliases(),
        )
        .await;
    assert!(
        result.is_none(),
        "a non-MCP tool must return None (fall-through), got {result:?}"
    );
}

#[tokio::test]
async fn mcp_tool_dispatch_audits_mcp_request_blocked_on_capability_deny() {
    // Gotcha #66: assert the AUDIT FILE content, not just the outcome.
    let dir = tempdir().unwrap();
    let path = dir.path().join("skills.audit.jsonl");
    let writer = Arc::new(AuditWriter::open(&path).await.expect("open audit"));

    let enforcer = CapabilityEnforcer::new(); // no grant → deny

    let transport = MockTransport::new().with_tool("extract_text", None, json!({}));
    let dispatcher = McpDispatcher::new(
        resolver_with(&[("pdf-mcp", &["extract_text"])]),
        Arc::new(enforcer),
        Arc::new(MockResolver { transport }),
        Some(Arc::clone(&writer)),
        "sess-audit",
    );

    let outcome = dispatcher
        .dispatch_if_mcp("worker", "pdf-mcp__extract_text", json!({}), &no_aliases())
        .await
        .expect("Some")
        .expect("deny is a resolved outcome");
    assert!(matches!(outcome, McpDispatchOutcome::Blocked { .. }));

    // File inspection — exactly one mcp_request_blocked line carrying
    // the MCP context.
    let contents = tokio::fs::read_to_string(&path).await.expect("read audit");
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines.len(), 1, "exactly one audit line, got: {contents}");
    let parsed: serde_json::Value =
        serde_json::from_str(lines[0]).expect("audit line parses as JSON");
    assert_eq!(parsed["kind"], "mcp_request_blocked");
    assert_eq!(parsed["session_id"], "sess-audit");
    assert_eq!(parsed["details"]["agent_id"], "worker");
    assert_eq!(parsed["details"]["server"], "pdf-mcp");
    assert_eq!(parsed["details"]["tool"], "extract_text");
    assert!(
        parsed["details"]["reason"].is_string(),
        "reason recorded in audit details"
    );
}

#[tokio::test]
async fn mcp_tool_dispatch_twice_in_sequence_both_succeed() {
    // Gotcha #69: two sequential dispatches against the same dispatcher
    // must both succeed; first-call mutation must not break the second.
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(Tier::Promoted);
    enforcer.grant("worker", mcp_tool_capability("pdf-mcp", "extract_text"));

    let transport = MockTransport::new()
        .with_tool("extract_text", None, json!({}))
        .with_tool_result("extract_text", json!({"n": 1}));

    let dispatcher = McpDispatcher::new(
        resolver_with(&[("pdf-mcp", &["extract_text"])]),
        Arc::new(enforcer),
        Arc::new(MockResolver { transport }),
        None,
        "sess-1",
    );

    for call in 1..=2 {
        let outcome = dispatcher
            .dispatch_if_mcp("worker", "pdf-mcp__extract_text", json!({}), &no_aliases())
            .await
            .unwrap_or_else(|| panic!("call {call}: Some"))
            .unwrap_or_else(|e| panic!("call {call}: no error, got {e:?}"));
        assert!(
            matches!(outcome, McpDispatchOutcome::Invoked { .. }),
            "call {call} must Invoke"
        );
    }
}
