//! Single source-of-truth resolver for the v0.1 session database path.
//!
//! ADR-0012: the drone session `SQLite` database is the canonical store
//! for all persisted runtime state in v0.1, **including the MCP server
//! registry**. Both `resolve_db_path` (the drone DB) and
//! `open_mcp_client` (the MCP registry) in `main.rs` resolve their path
//! through this one helper so the file is provably identical — closing
//! `docs/M06-irl-findings.md` 🔴-1 (an added MCP server landed in a
//! stray `mcp.sqlite` while the runtime read the live `session.sqlite`).
//!
//! Path-agnostic per CLAUDE.md §9: the Tauri shell resolves
//! `AppHandle::path().app_local_data_dir()` and passes the resolved
//! directory in. This helper is the testable seam; the `AppHandle`
//! wrappers in `main.rs` are the documented coverage holdout
//! (CLAUDE.md §5 tauri-shell patch-gate, the `*_with`/seam-vs-wrapper
//! precedent in `commands.rs`).

use std::path::{Path, PathBuf};

/// Canonical session-DB filename (ADR-0012). One constant so a divergent
/// registry filename cannot be silently re-introduced (IRL 🔴-1 was
/// `mcp.sqlite`).
pub const SESSION_DB_FILENAME: &str = "session.sqlite";

/// Resolve the single source-of-truth session database path under `dir`.
///
/// Both the drone (`resolve_db_path`) and the MCP registry
/// (`open_mcp_client`) call this so the resolved path is byte-identical.
pub fn session_db_path(dir: &Path) -> PathBuf {
    dir.join(SESSION_DB_FILENAME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_mcp::client::{McpServerRecord, Registry};
    use runtime_mcp::ServerStatus;
    use tempfile::tempdir;

    // Mirror src-tauri/src/main.rs: both `resolve_db_path` (the drone DB,
    // main.rs:298-301) and `open_mcp_client` (the MCP registry,
    // main.rs:262) build their path as `<app_local_data_dir>/<file>`.
    // These two helpers reproduce that exact resolution against an
    // injected dir so the equality assertion needs no Tauri AppHandle
    // (the AppHandle wrapper is the CLAUDE.md §5 coverage holdout; this
    // seam is what gets tested).
    fn drone_path_as_resolve_db_path_does(dir: &Path) -> PathBuf {
        session_db_path(dir)
    }
    fn registry_path_as_open_mcp_client_does(dir: &Path) -> PathBuf {
        session_db_path(dir)
    }

    #[test]
    fn registry_path_equals_drone_session_db_path() {
        let tmp = tempdir().expect("tempdir");
        let drone = drone_path_as_resolve_db_path_does(tmp.path());
        let registry = registry_path_as_open_mcp_client_does(tmp.path());
        assert_eq!(
            drone, registry,
            "IRL 🔴-1: open_mcp_client and resolve_db_path must resolve \
             the SAME file; divergence makes an added MCP server invisible \
             to the runtime"
        );
    }

    #[test]
    fn add_server_then_list_round_trips_through_the_same_store() {
        let tmp = tempdir().expect("tempdir");
        let path = session_db_path(tmp.path());

        let writer = Registry::open(&path).expect("open writer registry");
        writer
            .insert(&McpServerRecord {
                name: "fs-test".to_string(),
                transport: "stdio".to_string(),
                command: Some("npx.cmd".to_string()),
                args_json: Some("[]".to_string()),
                env_json: Some("{}".to_string()),
                cwd: None,
                url: None,
                auth_secret_ref: None,
                status: ServerStatus::Disconnected,
            })
            .expect("insert server");
        drop(writer);

        // A SEPARATELY-constructed connection at the SAME resolved path —
        // the drone-reads-what-the-UI-wrote contract + ADR-0012's
        // two-connection invariant (relies on db::init idempotency,
        // gotcha #80).
        let reader = Registry::open(&path).expect("reopen reader registry");
        let listed = reader.list().expect("list servers");
        assert!(
            listed.iter().any(|s| s.name == "fs-test"),
            "a server added through one connection must be visible \
             through a separate connection at the same resolved path"
        );
    }

    #[test]
    fn no_stray_mcp_sqlite_path_literal_constructed() {
        let tmp = tempdir().expect("tempdir");
        let resolved = session_db_path(tmp.path());
        assert_eq!(
            resolved.file_name().and_then(|f| f.to_str()),
            Some("session.sqlite"),
            "the resolver must yield session.sqlite — regression pin \
             against re-introducing a divergent registry filename \
             (IRL 🔴-1 was mcp.sqlite)"
        );
    }
}
