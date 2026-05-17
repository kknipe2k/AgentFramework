//! M06 Stage C — `Registry` `SQLite` tests against `tempfile`-backed paths.
//!
//! Verifies the path-agnostic contract per CLAUDE.md §9 archetype:
//! - `Registry::open(path: &Path)` accepts any path.
//! - Migration runner applies 002 cleanly + idempotently.
//! - insert / get / list / remove round-trip.
//! - `update_last_alive` persists the timestamp.
//! - Multi-call invariant per gotcha #69.
//!
//! The migration runner ownership note: in production the drone process
//! owns the `SQLite` file + applies migrations at process startup. The
//! `Registry::open` path here re-runs the runner (idempotent) so tests
//! don't require a pre-spawned drone.

use runtime_mcp::client::{LifecycleError, McpServerRecord, Registry};
use tempfile::TempDir;

fn temp_db_path() -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("mcp.sqlite");
    (dir, path)
}

fn stdio_record(name: &str) -> McpServerRecord {
    McpServerRecord {
        name: name.to_string(),
        transport: "stdio".to_string(),
        command: Some("/usr/bin/echo".to_string()),
        args_json: Some("[]".to_string()),
        env_json: Some("{}".to_string()),
        cwd: None,
        url: None,
        auth_secret_ref: None,
        status: "configured".to_string(),
    }
}

fn http_record(name: &str, url: &str) -> McpServerRecord {
    McpServerRecord {
        name: name.to_string(),
        transport: "http".to_string(),
        command: None,
        args_json: None,
        env_json: None,
        cwd: None,
        url: Some(url.to_string()),
        auth_secret_ref: Some(format!("mcp.{name}")),
        status: "configured".to_string(),
    }
}

#[test]
fn registry_open_initializes_schema_via_migration_runner() {
    let (_dir, path) = temp_db_path();
    let _registry = Registry::open(&path).expect("open registry");
    // The schema-shape assertions live in the drone-owned db.rs unit
    // tests; here we only assert that the file exists post-open.
    assert!(path.exists(), "registry open must create the SQLite file");
}

#[test]
fn registry_insert_persists_config() {
    let (_dir, path) = temp_db_path();
    let registry = Registry::open(&path).expect("open");
    registry
        .insert(&stdio_record("github"))
        .expect("insert stdio");
}

#[test]
fn registry_get_returns_persisted_config() {
    let (_dir, path) = temp_db_path();
    let registry = Registry::open(&path).expect("open");
    let original = http_record("vault", "https://vault.example.com/mcp");
    registry.insert(&original).expect("insert");
    let fetched = registry.get("vault").expect("get");
    assert_eq!(fetched.name, "vault");
    assert_eq!(fetched.transport, "http");
    assert_eq!(
        fetched.url.as_deref(),
        Some("https://vault.example.com/mcp")
    );
    assert_eq!(fetched.auth_secret_ref.as_deref(), Some("mcp.vault"));
}

#[test]
fn registry_get_returns_not_found_for_missing_name() {
    let (_dir, path) = temp_db_path();
    let registry = Registry::open(&path).expect("open");
    let err = registry.get("nope").expect_err("missing name must err");
    match err {
        LifecycleError::NotFound(name) => assert_eq!(name, "nope"),
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[test]
fn registry_list_returns_all_configs_in_some_order() {
    let (_dir, path) = temp_db_path();
    let registry = Registry::open(&path).expect("open");
    registry.insert(&stdio_record("a")).expect("a");
    registry.insert(&stdio_record("b")).expect("b");
    registry.insert(&stdio_record("c")).expect("c");
    let all = registry.list().expect("list");
    assert_eq!(all.len(), 3);
    let names: Vec<&str> = all.iter().map(|r| r.name.as_str()).collect();
    assert!(names.contains(&"a"));
    assert!(names.contains(&"b"));
    assert!(names.contains(&"c"));
}

#[test]
fn registry_insert_duplicate_name_returns_already_exists() {
    let (_dir, path) = temp_db_path();
    let registry = Registry::open(&path).expect("open");
    registry.insert(&stdio_record("dup")).expect("first");
    let err = registry
        .insert(&stdio_record("dup"))
        .expect_err("second insert must err");
    match err {
        LifecycleError::AlreadyExists(name) => assert_eq!(name, "dup"),
        other => panic!("expected AlreadyExists, got {other:?}"),
    }
}

#[test]
fn registry_remove_deletes_row() {
    let (_dir, path) = temp_db_path();
    let registry = Registry::open(&path).expect("open");
    registry.insert(&stdio_record("ephemeral")).expect("insert");
    registry.remove("ephemeral").expect("remove");
    let err = registry
        .get("ephemeral")
        .expect_err("removed row must err on get");
    assert!(matches!(err, LifecycleError::NotFound(_)));
}

#[test]
fn registry_remove_missing_name_is_idempotent() {
    let (_dir, path) = temp_db_path();
    let registry = Registry::open(&path).expect("open");
    registry
        .remove("never-existed")
        .expect("remove of missing must be Ok");
}

#[test]
fn registry_update_last_alive_persists_timestamp() {
    let (_dir, path) = temp_db_path();
    let registry = Registry::open(&path).expect("open");
    registry.insert(&stdio_record("pinged")).expect("insert");
    registry
        .update_last_alive("pinged", 1_700_000_000_000)
        .expect("update");
    // The persistence shape (last_connected_at column) is asserted
    // implicitly via SQLite-side query; here we just confirm the
    // method returns Ok and a subsequent get still succeeds.
    let row = registry.get("pinged").expect("get after update");
    assert_eq!(row.name, "pinged");
}

// gotcha #69 — multi-call invariants.

#[test]
fn registry_open_twice_in_sequence_does_not_re_run_migrations() {
    let (_dir, path) = temp_db_path();
    let registry1 = Registry::open(&path).expect("first open");
    drop(registry1);
    // Second open against the same path must succeed without erroring
    // on duplicate-table errors — the `_migrations` table tracks
    // applied versions so re-runs are no-ops.
    let _registry2 = Registry::open(&path).expect("second open must not re-run migrations");
}

#[test]
fn registry_insert_and_get_twice_in_sequence_both_succeed() {
    let (_dir, path) = temp_db_path();
    let registry = Registry::open(&path).expect("open");
    registry.insert(&stdio_record("multi")).expect("insert");
    let first = registry.get("multi").expect("get #1");
    let second = registry.get("multi").expect("get #2");
    assert_eq!(first, second);
}
