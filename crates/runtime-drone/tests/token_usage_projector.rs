//! `token_usage` projection tests — M07.D2 (ADR-0011 d; closes the
//! M06.5 `token_usage = 0` open finding).
//!
//! Mirrors `vdr_projection.rs`: exercises
//! `runtime_drone::token_usage::project_signal` against a temp SQLite
//! DB seeded with synthetic signals. Per spec §2c.3 + §937 the
//! `token_usage` signal carries `{ input, output, model, cost_usd }`;
//! the projector is the third drone projector (parallel to vdr +
//! plan_projector — same `handle_write_signal` transaction, NOT a new
//! DroneCommand).

use runtime_drone::db;
use runtime_drone::token_usage::project_signal;
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
    sig_type: &str,
    event_name: &str,
    payload: &serde_json::Value,
) {
    conn.execute(
        "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
         VALUES (?1, 's1', ?2, ?3, '0', ?4, 'agent_loop')",
        params![id, sig_type, event_name, payload.to_string()],
    )
    .expect("insert signal");
}

fn token_rows(conn: &Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM token_usage", [], |r| r.get(0))
        .expect("count token_usage")
}

#[test]
fn token_usage_signal_produces_one_token_usage_row() {
    let (_dir, conn) = open();
    insert_signal(
        &conn,
        "sig-tu1",
        "agent",
        "token_usage",
        &json!({
            "type": "token_usage",
            "input": 1234,
            "output": 56,
            "model": "claude-haiku-4-5",
            "cost_usd": 0.0021
        }),
    );

    let inserted = project_signal(&conn, "sig-tu1").expect("project");
    assert_eq!(inserted, 1, "a token_usage signal produces one row");
    assert_eq!(token_rows(&conn), 1);

    let (input, output, model, cost): (i64, i64, String, f64) = conn
        .query_row(
            "SELECT input_tokens, output_tokens, model, cost_usd FROM token_usage",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .expect("query token_usage");
    assert_eq!(input, 1234);
    assert_eq!(output, 56);
    assert_eq!(model, "claude-haiku-4-5");
    assert!((cost - 0.0021).abs() < f64::EPSILON);
}

#[test]
fn non_token_usage_signal_produces_no_row() {
    let (_dir, conn) = open();
    insert_signal(
        &conn,
        "sig-a1",
        "agent",
        "agent_complete",
        &json!({ "type": "agent_complete", "agent_id": "a1", "result": "ok" }),
    );
    let inserted = project_signal(&conn, "sig-a1").expect("project");
    assert_eq!(inserted, 0, "a non-token_usage signal is not projected");
    assert_eq!(token_rows(&conn), 0);
}

#[test]
fn projection_is_idempotent_for_the_same_signal_id() {
    let (_dir, conn) = open();
    insert_signal(
        &conn,
        "sig-tu2",
        "agent",
        "token_usage",
        &json!({
            "type": "token_usage",
            "input": 10,
            "output": 20,
            "model": "claude-haiku-4-5",
            "cost_usd": 0.0
        }),
    );
    assert_eq!(project_signal(&conn, "sig-tu2").expect("first"), 1);
    assert_eq!(
        project_signal(&conn, "sig-tu2").expect("second"),
        0,
        "re-projecting the same signal id inserts zero rows (idempotent)"
    );
    assert_eq!(token_rows(&conn), 1, "exactly one row after re-projection");
}

#[test]
fn missing_signal_id_is_not_found() {
    let (_dir, conn) = open();
    let err = project_signal(&conn, "nope").expect_err("missing → error");
    assert!(
        err.to_string().contains("nope"),
        "SignalNotFound names the missing id; got: {err}"
    );
}
