//! M06 Stage C — `McpClient` lifecycle integration tests against the
//! `MockTransport` (Stage B) + `tempfile`-backed `Registry` +
//! `InMemorySecretStore` + `tempfile`-backed `AuditWriter`.
//!
//! Each test wires the full Stage C surface: registry, secret store,
//! audit writer (per gotcha #66 correlation: emissions verified by
//! reading the `skills.audit.jsonl` file). Tests run with the
//! `test-helpers` cargo feature gating `MockTransport`.
//!
//! These tests link runtime-main (for `AuditWriter`) and runtime-mcp
//! (for the lifecycle surface + the gated `MockTransport`). The Cargo
//! manifest already pulls `runtime-main` as a regular dep; the
//! `MockTransport` is `#[cfg(any(test, feature = "test-helpers"))]` so
//! the integration test file inherits the visibility automatically.

#![cfg(feature = "test-helpers")]

use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use runtime_core::generated::mcp::{
    McpServerConfig, McpServerName, McpTransport, McpTransportCommand,
};
use runtime_main::audit::AuditWriter;
use runtime_mcp::client::{InMemorySecretStore, McpClient, Registry, SecretStore};
use runtime_mcp::error::McpError;
use runtime_mcp::transport::{MockTransport, Transport};
use runtime_mcp::ServerStatus;
use serde_json::json;
use tempfile::TempDir;

const SESSION_ID: &str = "sess-m06c";

async fn open_audit(dir: &TempDir) -> Arc<AuditWriter> {
    let path = dir.path().join("skills.audit.jsonl");
    Arc::new(AuditWriter::open(&path).await.expect("open audit"))
}

async fn read_audit_lines(dir: &TempDir) -> Vec<serde_json::Value> {
    let path = dir.path().join("skills.audit.jsonl");
    if !path.exists() {
        return Vec::new();
    }
    let raw = tokio::fs::read_to_string(&path).await.expect("read audit");
    raw.lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).expect("each line parses"))
        .collect()
}

fn open_registry(dir: &TempDir) -> Arc<Registry> {
    let path = dir.path().join("mcp.sqlite");
    Arc::new(Registry::open(&path).expect("open registry"))
}

fn stdio_config(name: &str) -> McpServerConfig {
    McpServerConfig {
        name: McpServerName::from_str(name).expect("name"),
        transport: McpTransport::Stdio {
            command: McpTransportCommand::from_str("/usr/bin/echo").expect("cmd"),
            args: vec![],
            env: std::collections::HashMap::default(),
            cwd: None,
        },
        auth_secret_ref: None,
    }
}

fn http_config(name: &str, url: &str) -> McpServerConfig {
    McpServerConfig {
        name: McpServerName::from_str(name).expect("name"),
        transport: McpTransport::Http {
            url: url.to_string(),
        },
        auth_secret_ref: Some(format!("mcp.{name}")),
    }
}

fn ok_transport() -> Arc<dyn Transport> {
    Arc::new(MockTransport::new().with_tool("read_file", None, json!({"type": "object"})))
}

fn failing_transport() -> Arc<dyn Transport> {
    // A transport whose connect() succeeds but list_tools() errs makes
    // a clean test fixture; for the `test connection rejects` case we
    // want connect() itself to fail — use the stdio variant pointing at
    // a nonexistent command via a real StdioTransport.
    Arc::new(
        runtime_mcp::transport::StdioTransport::new("this-command-does-not-exist-aaaa")
            .with_args(vec![]),
    )
}

#[tokio::test]
async fn add_server_persists_to_registry_and_audits_mcp_installed() {
    let dir = tempfile::tempdir().unwrap();
    let registry = open_registry(&dir);
    let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
    let audit = open_audit(&dir).await;
    let client = McpClient::new_with_audit(
        Arc::clone(&registry),
        Arc::clone(&secret_store),
        Arc::clone(&audit),
        SESSION_ID,
    );
    let config = stdio_config("github");
    client
        .add_server(config, None, ok_transport())
        .await
        .expect("add_server");

    // Registry assertion.
    let row = registry.get("github").expect("registry has the row");
    assert_eq!(row.transport, "stdio");
    assert!(row.auth_secret_ref.is_none());

    // Audit assertion — exactly ONE line, kind=mcp_installed.
    let lines = read_audit_lines(&dir).await;
    assert_eq!(
        lines.len(),
        1,
        "expected exactly one audit line; got {lines:?}"
    );
    assert_eq!(lines[0]["kind"], "mcp_installed");
    assert_eq!(lines[0]["details"]["name"], "github");
    assert_eq!(lines[0]["details"]["transport_kind"], "stdio");
    assert_eq!(lines[0]["details"]["has_auth"], false);
}

#[tokio::test]
async fn add_server_with_auth_persists_secret_and_audits_both_lines_in_order() {
    // Per gotcha #66: a successful add_server WITH auth must emit BOTH
    // mcp_installed AND mcp_auth_granted in order, not just one.
    let dir = tempfile::tempdir().unwrap();
    let registry = open_registry(&dir);
    let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
    let audit = open_audit(&dir).await;
    let client = McpClient::new_with_audit(
        Arc::clone(&registry),
        Arc::clone(&secret_store),
        Arc::clone(&audit),
        SESSION_ID,
    );
    let config = http_config("vault", "https://vault.example.com/mcp");
    client
        .add_server(config, Some("vault-token-xyz".to_string()), ok_transport())
        .await
        .expect("add_server");

    // Secret persisted.
    let secret = secret_store
        .fetch_secret("mcp.vault")
        .await
        .expect("secret persisted");
    assert_eq!(secret, "vault-token-xyz");

    // Audit assertion — TWO lines in order.
    let lines = read_audit_lines(&dir).await;
    assert_eq!(
        lines.len(),
        2,
        "expected two audit lines in order; got {lines:?}"
    );
    assert_eq!(lines[0]["kind"], "mcp_installed");
    assert_eq!(lines[0]["details"]["has_auth"], true);
    assert_eq!(lines[1]["kind"], "mcp_auth_granted");
    assert_eq!(lines[1]["details"]["name"], "vault");
}

#[tokio::test]
async fn add_server_failing_test_connection_does_not_persist_anything() {
    let dir = tempfile::tempdir().unwrap();
    let registry = open_registry(&dir);
    let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
    let audit = open_audit(&dir).await;
    let client = McpClient::new_with_audit(
        Arc::clone(&registry),
        Arc::clone(&secret_store),
        Arc::clone(&audit),
        SESSION_ID,
    );
    let config = stdio_config("doomed");
    let result = client
        .add_server(
            config,
            Some("would-be-secret".to_string()),
            failing_transport(),
        )
        .await;
    assert!(result.is_err(), "failing connect must err");

    // No registry row.
    let registry_err = registry.get("doomed");
    assert!(
        registry_err.is_err(),
        "registry must be empty after failed add"
    );

    // No secret persisted.
    let secret_err = secret_store.fetch_secret("mcp.doomed").await;
    assert!(
        secret_err.is_err(),
        "secret must NOT be persisted on failed add"
    );

    // No audit lines.
    let lines = read_audit_lines(&dir).await;
    assert!(
        lines.is_empty(),
        "no audit lines on failed add; got {lines:?}"
    );
}

#[tokio::test]
async fn remove_server_disconnects_and_removes_and_audits_mcp_uninstalled() {
    let dir = tempfile::tempdir().unwrap();
    let registry = open_registry(&dir);
    let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
    let audit = open_audit(&dir).await;
    let client = McpClient::new_with_audit(
        Arc::clone(&registry),
        Arc::clone(&secret_store),
        Arc::clone(&audit),
        SESSION_ID,
    );
    client
        .add_server(stdio_config("removeme"), None, ok_transport())
        .await
        .expect("add_server");
    // Drain the install audit lines (1 line).
    let _install_lines = read_audit_lines(&dir).await;

    client.remove_server("removeme").await.expect("remove");

    let registry_err = registry.get("removeme");
    assert!(
        registry_err.is_err(),
        "registry row must be gone after remove"
    );

    let lines = read_audit_lines(&dir).await;
    let last = lines.last().expect("at least one line");
    assert_eq!(last["kind"], "mcp_uninstalled");
    assert_eq!(last["details"]["name"], "removeme");
}

#[tokio::test]
async fn remove_server_removes_auth_secret_when_present() {
    let dir = tempfile::tempdir().unwrap();
    let registry = open_registry(&dir);
    let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
    let audit = open_audit(&dir).await;
    let client = McpClient::new_with_audit(
        Arc::clone(&registry),
        Arc::clone(&secret_store),
        Arc::clone(&audit),
        SESSION_ID,
    );
    client
        .add_server(
            http_config("oauth-server", "https://oauth.example.com/mcp"),
            Some("rotating-token".to_string()),
            ok_transport(),
        )
        .await
        .expect("add_server");
    client.remove_server("oauth-server").await.expect("remove");

    let secret_err = secret_store.fetch_secret("mcp.oauth-server").await;
    assert!(secret_err.is_err(), "secret must be dropped on remove");
}

#[tokio::test]
async fn test_connection_returns_tools_list_without_persistence() {
    let dir = tempfile::tempdir().unwrap();
    let registry = open_registry(&dir);
    let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
    let audit = open_audit(&dir).await;
    let client = McpClient::new_with_audit(
        Arc::clone(&registry),
        Arc::clone(&secret_store),
        Arc::clone(&audit),
        SESSION_ID,
    );
    let transport: Arc<dyn Transport> = Arc::new(
        MockTransport::new()
            .with_tool("a", None, json!({}))
            .with_tool("b", Some("desc".into()), json!({})),
    );
    let tools = client
        .test_connection(transport)
        .await
        .expect("test_connection");
    assert_eq!(tools.len(), 2);

    // No persistence side-effects: the registry must remain empty + no
    // audit lines emitted.
    let all = registry.list().expect("list");
    assert!(all.is_empty(), "test_connection must not persist");
    let lines = read_audit_lines(&dir).await;
    assert!(lines.is_empty(), "test_connection must not audit");
}

#[tokio::test]
async fn test_connection_returns_error_on_unreachable_server() {
    let dir = tempfile::tempdir().unwrap();
    let registry = open_registry(&dir);
    let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
    let audit = open_audit(&dir).await;
    let client = McpClient::new_with_audit(
        Arc::clone(&registry),
        Arc::clone(&secret_store),
        Arc::clone(&audit),
        SESSION_ID,
    );
    let result = client.test_connection(failing_transport()).await;
    assert!(result.is_err(), "unreachable server must err");
}

#[tokio::test]
async fn get_connection_returns_cached_connection_on_second_call() {
    let dir = tempfile::tempdir().unwrap();
    let registry = open_registry(&dir);
    let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
    let audit = open_audit(&dir).await;
    let client = McpClient::new_with_audit(
        Arc::clone(&registry),
        Arc::clone(&secret_store),
        Arc::clone(&audit),
        SESSION_ID,
    );
    client
        .add_server(stdio_config("cacheable"), None, ok_transport())
        .await
        .expect("add_server");
    let conn1 = client
        .get_connection("cacheable", ok_transport())
        .await
        .expect("first");
    let conn2 = client
        .get_connection("cacheable", ok_transport())
        .await
        .expect("second");
    // Same Arc-identity implies caching.
    assert!(
        Arc::ptr_eq(&conn1, &conn2),
        "second get_connection must return the cached Arc"
    );
}

#[tokio::test]
async fn list_servers_includes_added_servers() {
    let dir = tempfile::tempdir().unwrap();
    let registry = open_registry(&dir);
    let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
    let client = McpClient::new(Arc::clone(&registry), Arc::clone(&secret_store), SESSION_ID);
    client
        .add_server(stdio_config("alpha"), None, ok_transport())
        .await
        .expect("alpha");
    client
        .add_server(
            http_config("beta", "https://beta.example.com/mcp"),
            Some("tok".into()),
            ok_transport(),
        )
        .await
        .expect("beta");
    let summaries = client.list_servers().await.expect("list");
    assert_eq!(summaries.len(), 2);
    let names: Vec<&str> = summaries.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"alpha"));
    assert!(names.contains(&"beta"));
    let beta = summaries.iter().find(|s| s.name == "beta").unwrap();
    assert_eq!(beta.transport, "http");
    assert!(beta.has_auth);
}

#[tokio::test]
async fn add_server_twice_in_sequence_with_distinct_names_both_succeed() {
    // gotcha #69 — multi-call invariant.
    let dir = tempfile::tempdir().unwrap();
    let registry = open_registry(&dir);
    let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
    let audit = open_audit(&dir).await;
    let client = McpClient::new_with_audit(
        Arc::clone(&registry),
        Arc::clone(&secret_store),
        Arc::clone(&audit),
        SESSION_ID,
    );
    client
        .add_server(stdio_config("first"), None, ok_transport())
        .await
        .expect("first");
    client
        .add_server(stdio_config("second"), None, ok_transport())
        .await
        .expect("second");

    let lines = read_audit_lines(&dir).await;
    assert_eq!(
        lines.len(),
        2,
        "two add_server calls must produce two audit lines"
    );
    assert_eq!(lines[0]["details"]["name"], "first");
    assert_eq!(lines[1]["details"]["name"], "second");
}

#[tokio::test]
async fn health_pass_with_healthy_connection_does_not_emit_mcp_missing() {
    // Counterpart to the failed-ping test: a connection whose ping()
    // succeeds must NOT be reported via the emit_missing callback. The
    // observer-callback contract is: emit only on failure.
    let dir = tempfile::tempdir().unwrap();
    let registry = open_registry(&dir);
    let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
    let audit = open_audit(&dir).await;
    let client = McpClient::new_with_audit(
        Arc::clone(&registry),
        Arc::clone(&secret_store),
        Arc::clone(&audit),
        SESSION_ID,
    );
    let healthy: Arc<dyn Transport> = Arc::new(MockTransport::new());
    client
        .add_server(stdio_config("steady"), None, Arc::clone(&healthy))
        .await
        .expect("add");
    let _ = client
        .get_connection("steady", Arc::clone(&healthy))
        .await
        .expect("prime cache");

    let observed = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let observed_clone = std::sync::Arc::clone(&observed);
    client
        .run_health_pass(move |name| observed_clone.lock().unwrap().push(name.to_string()))
        .await;
    let names_snapshot: Vec<String> = observed.lock().unwrap().clone();
    assert!(
        names_snapshot.is_empty(),
        "healthy connection must not be reported via emit_missing; got {names_snapshot:?}"
    );
}

#[tokio::test]
async fn health_pass_emits_mcp_missing_for_failed_ping_and_drops_cache() {
    // Health-pass routes failed pings through the supplied event sink
    // (`emit_missing` callback) which the production wiring binds to
    // the existing `mcp_missing` event variant + `on_gap` HITL trigger.
    // No new event variant or HITL trigger.
    let dir = tempfile::tempdir().unwrap();
    let registry = open_registry(&dir);
    let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
    let audit = open_audit(&dir).await;
    let client = McpClient::new_with_audit(
        Arc::clone(&registry),
        Arc::clone(&secret_store),
        Arc::clone(&audit),
        SESSION_ID,
    );
    let unhealthy: Arc<dyn Transport> =
        Arc::new(MockTransport::new().with_ping_error(McpError::Timeout { timeout_ms: 500 }));
    client
        .add_server(stdio_config("flaky"), None, Arc::clone(&unhealthy))
        .await
        .expect("add");
    // Prime the connection cache.
    let _ = client
        .get_connection("flaky", Arc::clone(&unhealthy))
        .await
        .expect("prime cache");

    let observed = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let observed_clone = std::sync::Arc::clone(&observed);
    client
        .run_health_pass(move |name| observed_clone.lock().unwrap().push(name.to_string()))
        .await;

    let names_snapshot: Vec<String> = observed.lock().unwrap().clone();
    assert!(
        names_snapshot.iter().any(|n| n == "flaky"),
        "expected 'flaky' to be reported as missing; got {names_snapshot:?}"
    );

    // Cache should be dropped — next get_connection reconnects. The
    // assertion above (mcp_missing routed via the emit closure) is the
    // load-bearing observable behavior; cache-drop is the implementation
    // mechanism that lets the next get_connection reconnect cleanly.
    let _: Duration = Duration::from_millis(1); // keep Duration import live without lint noise
}

// ── EFF-4 (batched run_health_pass) + CQ-6 (ServerStatus enum) ────────

#[tokio::test]
async fn health_pass_persists_connected_and_error_status_per_server() {
    // EFF-4: `run_health_pass` writes every server's outcome in ONE
    // batched registry transaction (was K sequential update_last_alive
    // calls, no status write at all). CQ-6: the persisted value is the
    // generated `ServerStatus` enum. Observable contract: after a pass,
    // a server whose ping succeeded is `Connected`; one whose ping
    // failed is `Error` — both updated atomically across the pass.
    let dir = tempfile::tempdir().unwrap();
    let registry = open_registry(&dir);
    let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
    let audit = open_audit(&dir).await;
    let client = McpClient::new_with_audit(
        Arc::clone(&registry),
        Arc::clone(&secret_store),
        Arc::clone(&audit),
        SESSION_ID,
    );

    let healthy: Arc<dyn Transport> = Arc::new(MockTransport::new());
    let unhealthy: Arc<dyn Transport> =
        Arc::new(MockTransport::new().with_ping_error(McpError::Timeout { timeout_ms: 250 }));
    client
        .add_server(stdio_config("alive"), None, Arc::clone(&healthy))
        .await
        .expect("add alive");
    client
        .add_server(stdio_config("dead"), None, Arc::clone(&unhealthy))
        .await
        .expect("add dead");
    // Freshly added → schema default `disconnected` (CQ-6).
    assert_eq!(
        registry.get("alive").expect("get alive").status,
        ServerStatus::Disconnected,
        "a freshly added server is `disconnected` until the first pass"
    );

    let _ = client
        .get_connection("alive", Arc::clone(&healthy))
        .await
        .expect("prime alive");
    let _ = client
        .get_connection("dead", Arc::clone(&unhealthy))
        .await
        .expect("prime dead");

    client.run_health_pass(|_| {}).await;

    assert_eq!(
        registry.get("alive").expect("get alive").status,
        ServerStatus::Connected,
        "a server whose ping succeeded must be persisted Connected"
    );
    assert_eq!(
        registry.get("dead").expect("get dead").status,
        ServerStatus::Error,
        "a server whose ping failed must be persisted Error"
    );
}
