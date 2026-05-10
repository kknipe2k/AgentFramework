//! Migration runner integration tests — M04 Stage B.
//!
//! Verify the architecture works as advertised: applies pending
//! migrations in order, tracks them in `_migrations`, idempotent across
//! re-runs.

use runtime_drone::db;
use rusqlite::Connection;
use tempfile::TempDir;

fn fresh_db() -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("migrations-test.sqlite");
    (dir, path)
}

#[test]
fn fresh_init_applies_all_migrations() {
    let (_dir, path) = fresh_db();
    let _conn = db::init(&path).expect("init");

    let conn = Connection::open(&path).expect("reopen");
    let versions: Vec<i64> = conn
        .prepare("SELECT version FROM _migrations ORDER BY version")
        .unwrap()
        .query_map([], |r| r.get::<_, i64>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(versions, vec![0, 1]);
}

#[test]
fn re_init_is_idempotent_no_extra_migration_rows() {
    let (_dir, path) = fresh_db();
    let _c1 = db::init(&path).expect("first");
    let _c2 = db::init(&path).expect("second");
    let _c3 = db::init(&path).expect("third");

    let conn = Connection::open(&path).expect("reopen");
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM _migrations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 2, "each migration should appear exactly once");
}

#[test]
fn migration_table_records_name_and_applied_at() {
    let (_dir, path) = fresh_db();
    let _c = db::init(&path).expect("init");
    let conn = Connection::open(&path).expect("reopen");
    let rows: Vec<(i64, String, i64)> = conn
        .prepare("SELECT version, name, applied_at FROM _migrations ORDER BY version")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].1, "initial");
    assert_eq!(rows[1].1, "plans_tasks");
    assert!(rows[0].2 > 0, "applied_at must be a real unix ms timestamp");
    assert!(rows[1].2 >= rows[0].2);
}

#[test]
fn migration_001_adds_plans_and_tasks_tables() {
    let (_dir, path) = fresh_db();
    let conn = db::init(&path).expect("init");

    let table_exists = |name: &str| -> bool {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [name],
                |r| r.get(0),
            )
            .unwrap();
        count == 1
    };
    assert!(table_exists("plans"));
    assert!(table_exists("tasks"));
    assert!(table_exists("_migrations"));
}

#[test]
fn migration_000_preserves_m01_baseline_tables() {
    let (_dir, path) = fresh_db();
    let conn = db::init(&path).expect("init");
    for t in [
        "sessions",
        "snapshots",
        "signals",
        "heartbeats",
        "vdr",
        "token_usage",
        "skills",
        "mcp_servers",
    ] {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [t],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "M01 baseline table missing: {t}");
    }
}

#[test]
fn run_migrations_on_existing_connection_is_idempotent() {
    // Mirror init_in_existing path — pre-seed test fixtures may call
    // run_migrations on a Connection they've already opened.
    let (_dir, path) = fresh_db();
    let conn = Connection::open(&path).expect("open raw");
    db::run_migrations(&conn).expect("first run");
    db::run_migrations(&conn).expect("second run");
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM _migrations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 2);
}

#[test]
fn plans_table_check_constraints_reject_invalid_status() {
    let (_dir, path) = fresh_db();
    let conn = db::init(&path).expect("init");
    conn.execute(
        "INSERT INTO sessions (id, status) VALUES ('s1', 'active')",
        [],
    )
    .unwrap();
    let bad = conn.execute(
        "INSERT INTO plans (id, session_id, title, status, approval_required, loop_policy, created_at) \
         VALUES ('p1', 's1', 'T', 'INVALID_STATUS', 0, 'fresh_context_per_task', 0)",
        [],
    );
    assert!(bad.is_err(), "invalid status must violate CHECK");
}

#[test]
fn tasks_table_check_constraints_reject_invalid_status() {
    let (_dir, path) = fresh_db();
    let conn = db::init(&path).expect("init");
    conn.execute(
        "INSERT INTO sessions (id, status) VALUES ('s1', 'active')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO plans (id, session_id, title, status, approval_required, loop_policy, created_at) \
         VALUES ('p1', 's1', 'T', 'approved', 0, 'fresh_context_per_task', 0)",
        [],
    )
    .unwrap();
    let bad = conn.execute(
        "INSERT INTO tasks (id, plan_id, title, status, created_at) \
         VALUES ('t1', 'p1', 'T', 'INVALID_STATUS', 0)",
        [],
    );
    assert!(bad.is_err(), "invalid task status must violate CHECK");
}
