//! VDR projection tests — Stage E (M03).
//!
//! Exercises `runtime_drone::vdr::project_signal` + `project_session`
//! against a temp `SQLite` DB seeded with synthetic signals. Per spec
//! §2b the VDR is a read-optimized projection of signals 4 (decision)
//! + 5 (verify); other signal types do not produce VDR rows.

use runtime_drone::db;
use runtime_drone::vdr::{is_select_only, project_session, project_signal};
use rusqlite::{params, Connection};
use serde_json::json;
use tempfile::TempDir;

fn open() -> (TempDir, Connection) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("d.sqlite");
    let conn = db::init(&path).expect("init");
    conn.execute(
        "INSERT INTO sessions (id, status) VALUES ('s1', 'active')",
        [],
    )
    .expect("seed session");
    (dir, conn)
}

fn insert_signal(
    conn: &Connection,
    id: &str,
    session_id: &str,
    sig_type: &str,
    event_name: &str,
    payload: &serde_json::Value,
) {
    conn.execute(
        "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
         VALUES (?1, ?2, ?3, ?4, '0', ?5, ?6)",
        params![
            id,
            session_id,
            sig_type,
            event_name,
            payload.to_string(),
            "agent_loop",
        ],
    )
    .expect("insert signal");
}

fn vdr_count(conn: &Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM vdr", [], |r| r.get(0))
        .expect("count vdr")
}

#[test]
fn decision_signal_produces_vdr_row() {
    let (_dir, conn) = open();
    insert_signal(
        &conn,
        "sig-d1",
        "s1",
        "decision",
        "decision",
        &json!({
            "agent_id": "a1",
            "decision": "pick haiku",
            "rationale": "cost",
            "tool_used": "estimate_cost"
        }),
    );

    let inserted = project_signal(&conn, "sig-d1").expect("project");
    assert_eq!(inserted, 1, "decision signal must produce one VDR row");
    assert_eq!(vdr_count(&conn), 1);

    let (sig_id, decision, rationale): (String, String, String) = conn
        .query_row(
            "SELECT contributing_signal_id, decision, rationale FROM vdr WHERE contributing_signal_id = ?1",
            ["sig-d1"],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .expect("query vdr");
    assert_eq!(sig_id, "sig-d1");
    assert_eq!(decision, "pick haiku");
    assert_eq!(rationale, "cost");
}

#[test]
fn verify_signal_produces_vdr_row() {
    let (_dir, conn) = open();
    insert_signal(
        &conn,
        "sig-v1",
        "s1",
        "verify",
        "verify",
        &json!({
            "agent_id": "a1",
            "hook_id": "test-suite",
            "passed": true,
        }),
    );

    let inserted = project_signal(&conn, "sig-v1").expect("project");
    assert_eq!(inserted, 1, "verify signal must produce one VDR row");
    let outcome: String = conn
        .query_row(
            "SELECT outcome FROM vdr WHERE contributing_signal_id = 'sig-v1'",
            [],
            |r| r.get(0),
        )
        .expect("query vdr");
    assert_eq!(outcome, "pass");
}

#[test]
fn non_decision_signal_produces_nothing() {
    let (_dir, conn) = open();
    insert_signal(
        &conn,
        "sig-t1",
        "s1",
        "tool",
        "invoked",
        &json!({"agent_id": "a1", "tool_name": "search"}),
    );

    let inserted = project_signal(&conn, "sig-t1").expect("project");
    assert_eq!(inserted, 0, "tool signal must not produce VDR rows");
    assert_eq!(vdr_count(&conn), 0);
}

#[test]
fn project_signal_is_idempotent() {
    let (_dir, conn) = open();
    insert_signal(
        &conn,
        "sig-d1",
        "s1",
        "decision",
        "decision",
        &json!({"agent_id": "a1", "decision": "x", "rationale": "y"}),
    );

    let first = project_signal(&conn, "sig-d1").expect("project 1");
    let second = project_signal(&conn, "sig-d1").expect("project 2");
    assert_eq!(first, 1);
    assert_eq!(second, 0, "second projection must be a no-op");
    assert_eq!(vdr_count(&conn), 1, "UNIQUE constraint enforces single row");
}

#[test]
fn project_session_reproduces_vdr_for_decision_and_verify_only() {
    let (_dir, conn) = open();
    insert_signal(
        &conn,
        "sig-1",
        "s1",
        "decision",
        "decision",
        &json!({"agent_id": "a1", "decision": "d1", "rationale": "r1"}),
    );
    insert_signal(
        &conn,
        "sig-2",
        "s1",
        "tool",
        "invoked",
        &json!({"agent_id": "a1", "tool_name": "search"}),
    );
    insert_signal(
        &conn,
        "sig-3",
        "s1",
        "verify",
        "verify",
        &json!({"agent_id": "a1", "hook_id": "h1", "passed": false}),
    );
    insert_signal(
        &conn,
        "sig-4",
        "s1",
        "agent",
        "spawned",
        &json!({"agent_id": "a1"}),
    );
    insert_signal(
        &conn,
        "sig-5",
        "s1",
        "decision",
        "decision",
        &json!({"agent_id": "a1", "decision": "d2", "rationale": "r2"}),
    );

    let inserted = project_session(&conn, "s1").expect("project_session");
    assert_eq!(
        inserted, 3,
        "two decisions + one verify must produce 3 VDR rows"
    );
    assert_eq!(vdr_count(&conn), 3);
}

#[test]
fn is_select_only_rejects_attack_vectors() {
    assert!(is_select_only("SELECT * FROM signals"));
    assert!(is_select_only("SELECT id FROM signals WHERE id = 'x'"));
    assert!(is_select_only("SELECT * FROM signals;"));
    assert!(
        !is_select_only("DROP TABLE signals"),
        "DROP must be rejected"
    );
    assert!(
        !is_select_only("DELETE FROM signals"),
        "DELETE must be rejected"
    );
    assert!(
        !is_select_only("INSERT INTO signals (id) VALUES ('x')"),
        "INSERT must be rejected"
    );
    assert!(
        !is_select_only("UPDATE signals SET id = 'x'"),
        "UPDATE must be rejected"
    );
    assert!(
        !is_select_only("PRAGMA table_info(signals)"),
        "PRAGMA must be rejected"
    );
    assert!(
        !is_select_only("SELECT 1; DROP TABLE signals"),
        "compound semicolons must be rejected"
    );
    assert!(!is_select_only(""), "empty string must be rejected");
    assert!(!is_select_only("not sql at all"));
}
