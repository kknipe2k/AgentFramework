//! M06 Stage C — `SecretStore` trait tests against `InMemorySecretStore`.
//!
//! Verifies the trait's contract surface: store + fetch round-trip, fetch
//! on missing ref returns `NotFound`, remove + fetch returns `NotFound`,
//! multi-call invariant per gotcha #69.

use runtime_mcp::client::{InMemorySecretStore, LifecycleError, SecretStore};

#[tokio::test]
async fn secret_store_round_trip_via_in_memory_fake() {
    let store = InMemorySecretStore::new();
    store
        .store_secret("mcp.github", "ghp_abc123")
        .await
        .expect("store");
    let secret = store.fetch_secret("mcp.github").await.expect("fetch");
    assert_eq!(secret, "ghp_abc123");
}

#[tokio::test]
async fn secret_store_fetch_returns_not_found_for_missing_ref() {
    let store = InMemorySecretStore::new();
    let err = store
        .fetch_secret("mcp.never-stored")
        .await
        .expect_err("missing ref must err");
    match err {
        LifecycleError::NotFound(name) => assert_eq!(name, "mcp.never-stored"),
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[tokio::test]
async fn secret_store_remove_then_fetch_returns_not_found() {
    let store = InMemorySecretStore::new();
    store
        .store_secret("mcp.gitlab", "glpat-xyz")
        .await
        .expect("store");
    store.remove_secret("mcp.gitlab").await.expect("remove");
    let err = store
        .fetch_secret("mcp.gitlab")
        .await
        .expect_err("removed ref must err on fetch");
    assert!(matches!(err, LifecycleError::NotFound(_)));
}

#[tokio::test]
async fn secret_store_remove_missing_ref_is_idempotent() {
    // Per the trait contract: remove() is idempotent so test setup +
    // teardown can run without ordering constraints. Removing a missing
    // ref must NOT err.
    let store = InMemorySecretStore::new();
    store
        .remove_secret("mcp.never-existed")
        .await
        .expect("remove of missing ref must be Ok");
}

#[tokio::test]
async fn secret_store_store_overwrites_prior_value() {
    let store = InMemorySecretStore::new();
    store.store_secret("mcp.s3", "first").await.expect("first");
    store
        .store_secret("mcp.s3", "second")
        .await
        .expect("second");
    let v = store.fetch_secret("mcp.s3").await.expect("fetch");
    assert_eq!(v, "second", "second store must overwrite first");
}

// gotcha #69 — multi-call invariant on every public method.

#[tokio::test]
async fn secret_store_store_then_fetch_twice_in_sequence_returns_same_secret() {
    let store = InMemorySecretStore::new();
    store
        .store_secret("mcp.kafka", "auth-tok")
        .await
        .expect("store");
    let first = store.fetch_secret("mcp.kafka").await.expect("fetch #1");
    let second = store.fetch_secret("mcp.kafka").await.expect("fetch #2");
    assert_eq!(first, "auth-tok");
    assert_eq!(first, second);
}

#[tokio::test]
async fn secret_store_two_distinct_refs_isolate_correctly() {
    // Inserting one ref must not bleed into another. Defends against an
    // accidental "single-slot" implementation.
    let store = InMemorySecretStore::new();
    store.store_secret("mcp.a", "secret-a").await.expect("a");
    store.store_secret("mcp.b", "secret-b").await.expect("b");
    assert_eq!(store.fetch_secret("mcp.a").await.unwrap(), "secret-a");
    assert_eq!(store.fetch_secret("mcp.b").await.unwrap(), "secret-b");
}
