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
    NewAmbiguity,
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

/// A `ConnectionResolver` mapping each server name to its own scripted
/// `MockTransport` — exercises the §5a re-resolution driver, which must
/// snapshot the *per-server* tool list (not one shared transport).
struct MultiServerResolver {
    servers: BTreeMap<String, MockTransport>,
}

#[async_trait]
impl ConnectionResolver for MultiServerResolver {
    async fn connection(&self, server: &str) -> Result<Arc<dyn Connection>, McpError> {
        let t = self
            .servers
            .get(server)
            .ok_or_else(|| McpError::connect_failed(format!("unknown server {server}")))?;
        Ok(Arc::from(t.connect().await?))
    }
}

fn multi(pairs: &[(&str, &[&str])]) -> Arc<MultiServerResolver> {
    let servers = pairs
        .iter()
        .map(|(s, tools)| {
            let mut t = MockTransport::new();
            for tool in *tools {
                t = t
                    .with_tool(*tool, None, json!({"type": "object"}))
                    .with_tool_result(*tool, json!({"ok": true}));
            }
            ((*s).to_string(), t)
        })
        .collect();
    Arc::new(MultiServerResolver { servers })
}

fn empty_resolver() -> Arc<RwLock<NamespaceResolver>> {
    Arc::new(RwLock::new(NamespaceResolver::new(BTreeMap::new())))
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

const fn no_aliases() -> BTreeMap<String, String> {
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

// ── ADR-0011 (b) — §5a re-resolution-on-connect driver (M06.V 🟡 #1) ──
//
// The resolver lives in `McpDispatcher` (ADR-0010), so the §5a step-5
// re-resolution driver is authored against `McpDispatcher`, NOT
// `McpClient` (which only *impls* `ConnectionResolver`) — the M06.V
// Dec-6 `<wire_trace_vs_adr_reconcile>` #6 reconciliation. M06 shipped
// the `NamespaceResolver` re-resolution primitive delivered + tested but
// with no production driver; these tests pin the driver `McpDispatcher`
// now exposes.

#[tokio::test]
async fn on_server_connected_returns_new_ambiguity_when_two_servers_share_a_short_name() {
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(Tier::Promoted);
    enforcer.grant("worker", mcp_tool_capability("pdf-mcp", "read"));

    // Empty resolver: nothing is connected at construction (the
    // src-tauri ctor builds it `NamespaceResolver::new(BTreeMap::new())`
    // — the resolver is populated by the connect driver, not the ctor).
    let dispatcher = McpDispatcher::new(
        empty_resolver(),
        Arc::new(enforcer),
        multi(&[("pdf-mcp", &["read"]), ("img-mcp", &["read"])]),
        None,
        "sess-rere",
    );

    // First connect: no collision yet.
    let first = dispatcher
        .on_server_connected("pdf-mcp")
        .await
        .expect("connect pdf-mcp");
    assert!(
        first.is_empty(),
        "first server connect cannot be ambiguous, got {first:?}"
    );

    // Second connect exposing the same short name → newly ambiguous.
    let second = dispatcher
        .on_server_connected("img-mcp")
        .await
        .expect("connect img-mcp");
    assert_eq!(
        second,
        vec![NewAmbiguity {
            short_name: "read".to_string(),
            candidates: vec!["img-mcp__read".to_string(), "pdf-mcp__read".to_string()],
        }],
        "the short name 'read' became ambiguous across the two connected servers"
    );

    // The resolver state actually changed: the short name now resolves
    // Ambiguous through the live dispatch path.
    let outcome = dispatcher
        .dispatch_if_mcp("worker", "read", json!({}), &no_aliases())
        .await
        .expect("resolved → Some")
        .expect("ambiguity is a resolved outcome");
    assert!(
        matches!(outcome, McpDispatchOutcome::Ambiguous { ref name, .. } if name == "read"),
        "post-connect, 'read' must resolve Ambiguous, got {outcome:?}"
    );
}

#[tokio::test]
async fn on_server_disconnected_re_resolves_and_clears_ambiguity() {
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_tier(Tier::Promoted);
    enforcer.grant("worker", mcp_tool_capability("pdf-mcp", "read"));

    let dispatcher = McpDispatcher::new(
        empty_resolver(),
        Arc::new(enforcer),
        multi(&[("pdf-mcp", &["read"]), ("img-mcp", &["read"])]),
        None,
        "sess-rere",
    );
    dispatcher.on_server_connected("pdf-mcp").await.expect("c1");
    dispatcher.on_server_connected("img-mcp").await.expect("c2");

    // Disconnect one side → the short name is unambiguous again and
    // dispatch routes to the single remaining server.
    dispatcher.on_server_disconnected("img-mcp").await;

    let outcome = dispatcher
        .dispatch_if_mcp("worker", "read", json!({}), &no_aliases())
        .await
        .expect("resolved → Some")
        .expect("dispatch did not error");
    match outcome {
        McpDispatchOutcome::Invoked { server, tool, .. } => {
            assert_eq!(server, "pdf-mcp");
            assert_eq!(tool, "read");
        }
        other => panic!("post-disconnect 'read' must Invoke on pdf-mcp, got {other:?}"),
    }
}

#[tokio::test]
async fn on_server_connected_twice_for_same_server_is_idempotent() {
    // gotcha #69 multi-call invariant: re-snapshotting the same server
    // with the same tool list yields no *new* ambiguity the second time.
    let dispatcher = McpDispatcher::new(
        empty_resolver(),
        Arc::new(CapabilityEnforcer::new()),
        multi(&[("pdf-mcp", &["read"]), ("img-mcp", &["read"])]),
        None,
        "sess-rere",
    );
    dispatcher.on_server_connected("pdf-mcp").await.expect("c1");
    let amb1 = dispatcher.on_server_connected("img-mcp").await.expect("c2");
    let amb2 = dispatcher
        .on_server_connected("img-mcp")
        .await
        .expect("c2 again");
    assert_eq!(
        amb1.len(),
        1,
        "first img-mcp connect surfaces the ambiguity"
    );
    assert!(
        amb2.is_empty(),
        "re-connecting the same server with the same tools is not *newly* ambiguous, got {amb2:?}"
    );
}
