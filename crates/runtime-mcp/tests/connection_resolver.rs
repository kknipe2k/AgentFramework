//! ADR-0011 (a) — `impl ConnectionResolver for McpClient` (M07.D1).
//!
//! M06 shipped the `ConnectionResolver` trait (`McpDispatcher` consumes
//! it) but NO production impl: `dispatch.rs` only has a test mock and
//! `src-tauri` could not construct a concrete `McpDispatcher` (ADR-0011
//! Context #1). D1 closes that: `McpClient` — the process-wide MCP
//! lifecycle manager that already owns the registry + connection cache —
//! implements `ConnectionResolver` by deriving the transport from the
//! persisted registry record and delegating to `get_connection`.
//!
//! The live-connect happy path needs a real MCP server and is the
//! mandatory Stage V `--features integration` reference-server smoke
//! (the seam↔concrete OS-call holdout per ADR-0011's named negative
//! consequence). These tests pin the trait wiring + the
//! registry-`NotFound` → `McpError` mapping + the gotcha #69 multi-call
//! invariant — the parts that ARE unit-observable.

use std::sync::Arc;

use runtime_mcp::client::{InMemorySecretStore, McpClient, Registry, SecretStore};
use runtime_mcp::{ConnectionResolver, McpError};
use tempfile::TempDir;

fn client_over_empty_registry() -> (TempDir, Arc<McpClient>) {
    let dir = TempDir::new().expect("tempdir");
    let registry = Arc::new(Registry::open(&dir.path().join("mcp.sqlite")).expect("open registry"));
    let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
    let client = Arc::new(McpClient::new(registry, secret_store, "sess-cr"));
    (dir, client)
}

#[tokio::test]
async fn mcp_client_is_usable_as_a_dyn_connection_resolver() {
    // ADR-0011 (a): the concrete `McpDispatcher` ctor in `src-tauri`
    // injects `Arc<McpClient>` as `Arc<dyn ConnectionResolver>`. This
    // compiles ONLY when the production impl exists (M06 had none).
    let (_dir, client) = client_over_empty_registry();
    let resolver: Arc<dyn ConnectionResolver> = client;
    // Object-safe + reachable through the trait object (not just the
    // inherent `get_connection`). `Arc<dyn Connection>` is not `Debug`,
    // so unwrap via `match` rather than `expect_err`.
    let Err(err) = resolver.connection("ghost").await else {
        panic!("an unregistered server cannot yield a connection");
    };
    // Registry `NotFound` must surface as a stable `McpError` (the
    // dispatch path only knows `McpError`, not `LifecycleError`), and
    // it must name the offending server for the audit log + renderer.
    assert!(
        err.to_string().contains("ghost"),
        "mapped error must name the server; got {err}"
    );
}

#[tokio::test]
async fn connection_for_unregistered_server_maps_registry_not_found_to_connect_failed() {
    let (_dir, client) = client_over_empty_registry();
    let Err(err) = client.connection("never-added").await else {
        panic!("missing registry row must not yield a connection");
    };
    // A missing server is a connect-time failure class (it is not a
    // mid-session transport blip) so Stage C lifecycle's retry-vs-
    // surface policy classifies it correctly.
    assert!(
        matches!(err, McpError::ConnectFailed(_)),
        "registry NotFound must map to ConnectFailed, got {err:?}"
    );
}

#[tokio::test]
async fn connection_twice_in_sequence_both_err_without_poisoning() {
    // gotcha #69 — a first failed resolve must not leave the client in
    // a state where the second call panics / behaves differently.
    let (_dir, client) = client_over_empty_registry();
    let first = client.connection("ghost").await;
    let second = client.connection("ghost").await;
    assert!(first.is_err(), "call #1 errs");
    assert!(
        second.is_err(),
        "call #2 errs identically (no poisoned cache)"
    );
}
