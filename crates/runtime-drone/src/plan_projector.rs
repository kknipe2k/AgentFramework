//! Plan + Task projection — drone-internal continuous projector.
//!
//! Reads plan/task `AgentEvent` payloads and UPSERTs `plans` + `tasks`.
//!
//! Per spec §2b: signals are the source of truth, the relational tables
//! are read-optimized projections. M04 Stage B authored this projector
//! parallel to [`crate::vdr::project_signal`] (M03.E archetype).
//!
//! ## Idempotence
//!
//! Every projection path uses `INSERT ... ON CONFLICT(id) DO UPDATE`
//! semantics. Re-running `project_signal` on the same signal id
//! produces no observable state change beyond timestamp re-writes
//! (which themselves are idempotent — the snapshot logic doesn't depend
//! on monotonically-increasing applied_at).
//!
//! ## Out-of-order projection
//!
//! When a `task_completed` event arrives before its `task_started`
//! (replay path; live execution never produces this order), the projector
//! UPSERTs the row with `status='done'`. A subsequent `task_started`
//! arrival also UPSERTs but does NOT downgrade a terminal status — the
//! projector preserves terminal state (`done`, `skipped`, `escalated`).
//!
//! ## Triggering
//!
//! Stage B wires the call into [`crate::command_handler`]'s `WriteSignal`
//! arm. Drone-internal: the runtime-main process never directly invokes
//! the projector; it goes through the IPC `WriteSignal` command.
//!
//! Safety primitive: ≥95% coverage gate per CLAUDE.md §5.

use rusqlite::{params, Connection};
use serde_json::Value;
use thiserror::Error;

/// Errors raised by the plan projector.
#[derive(Debug, Error)]
pub enum PlanProjectorError {
    /// Signal id not found in the `signals` table.
    #[error("signal not found: {0}")]
    SignalNotFound(String),
    /// Underlying rusqlite error.
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    /// JSON parse error reading the signal's `payload_json`.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Payload missing a required field for its event type.
    #[error("invalid payload for {event_type}: missing field {field}")]
    InvalidPayload {
        /// Event type (e.g., `plan_created`).
        event_type: String,
        /// Missing field name.
        field: &'static str,
    },
}

/// Project the signal identified by `signal_id` into the `plans` /
/// `tasks` tables.
///
/// Returns the number of rows mutated by the projection. Returns `0` for
/// signal kinds whose payloads aren't plan/task events (idempotent
/// no-op; mirrors the [`crate::vdr::project_signal`] convention).
///
/// # Errors
///
/// - [`PlanProjectorError::SignalNotFound`] if `signal_id` isn't in the
///   signals table.
/// - [`PlanProjectorError::Sqlite`] on database errors.
/// - [`PlanProjectorError::Json`] on malformed `payload_json`.
/// - [`PlanProjectorError::InvalidPayload`] on missing required fields
///   inside an otherwise-well-formed payload.
pub fn project_signal(conn: &Connection, signal_id: &str) -> Result<usize, PlanProjectorError> {
    let row = read_signal_row(conn, signal_id)?;
    let payload: Value = if row.payload_json.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&row.payload_json)?
    };

    let event_type = payload
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    match event_type.as_str() {
        "plan_created" => insert_plan(conn, &row.session_id, &payload),
        "plan_approved" => update_plan_status(conn, &payload, "approved", true),
        "plan_aborted" => update_plan_status(conn, &payload, "aborted", false),
        "plan_complete" => update_plan_status(conn, &payload, "complete", false),
        "plan_revised" => update_plan_status(conn, &payload, "pending_approval", false),
        "task_started" => upsert_task_started(conn, &payload),
        "task_completed" => update_task_terminal(conn, &payload, "done"),
        "task_failed" => update_task_failed(conn, &payload),
        "task_skipped" => update_task_terminal(conn, &payload, "skipped"),
        "task_escalated" => update_task_escalated(conn, &payload),
        "task_rolled_back" => update_task_terminal(conn, &payload, "failed"),
        _ => Ok(0),
    }
}

struct SignalRow {
    session_id: String,
    payload_json: String,
}

fn read_signal_row(conn: &Connection, signal_id: &str) -> Result<SignalRow, PlanProjectorError> {
    conn.query_row(
        "SELECT session_id, payload_json FROM signals WHERE id = ?1",
        params![signal_id],
        |r| {
            Ok(SignalRow {
                session_id: r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                payload_json: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            PlanProjectorError::SignalNotFound(signal_id.to_string())
        }
        other => PlanProjectorError::from(other),
    })
}

fn require_str<'a>(
    payload: &'a Value,
    field: &'static str,
    event_type: &str,
) -> Result<&'a str, PlanProjectorError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| PlanProjectorError::InvalidPayload {
            event_type: event_type.to_string(),
            field,
        })
}

fn require_u64(
    payload: &Value,
    field: &'static str,
    event_type: &str,
) -> Result<u64, PlanProjectorError> {
    payload
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| PlanProjectorError::InvalidPayload {
            event_type: event_type.to_string(),
            field,
        })
}

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

fn insert_plan(
    conn: &Connection,
    session_id: &str,
    payload: &Value,
) -> Result<usize, PlanProjectorError> {
    let plan_id = require_str(payload, "plan_id", "plan_created")?;
    let title = require_str(payload, "title", "plan_created")?;
    let approval_required = payload
        .get("approval_required")
        .and_then(Value::as_bool)
        .ok_or(PlanProjectorError::InvalidPayload {
            event_type: "plan_created".to_string(),
            field: "approval_required",
        })?;
    let initial_status = if approval_required {
        "pending_approval"
    } else {
        "approved"
    };
    let inserted = conn.execute(
        "INSERT INTO plans (\
            id, session_id, title, status, approval_required, loop_policy, \
            hitl_checkpoints, risks, created_at\
         ) VALUES (?1, ?2, ?3, ?4, ?5, 'fresh_context_per_task', '[]', '[]', ?6) \
         ON CONFLICT(id) DO UPDATE SET title = excluded.title",
        params![
            plan_id,
            session_id,
            title,
            initial_status,
            i64::from(approval_required),
            now_ms(),
        ],
    )?;
    Ok(inserted)
}

fn update_plan_status(
    conn: &Connection,
    payload: &Value,
    status: &str,
    set_approved_at: bool,
) -> Result<usize, PlanProjectorError> {
    let plan_id = require_str(payload, "plan_id", "plan_*")?;
    let now = now_ms();
    let n = if set_approved_at {
        conn.execute(
            "UPDATE plans SET status = ?2, approved_at = ?3 WHERE id = ?1",
            params![plan_id, status, now],
        )?
    } else if status == "complete" {
        conn.execute(
            "UPDATE plans SET status = ?2, completed_at = ?3 WHERE id = ?1",
            params![plan_id, status, now],
        )?
    } else {
        conn.execute(
            "UPDATE plans SET status = ?2 WHERE id = ?1",
            params![plan_id, status],
        )?
    };
    Ok(n)
}

fn upsert_task_started(conn: &Connection, payload: &Value) -> Result<usize, PlanProjectorError> {
    let plan_id = require_str(payload, "plan_id", "task_started")?;
    let task_id = require_str(payload, "task_id", "task_started")?;
    let now = now_ms();
    // Don't downgrade terminal status on out-of-order arrival —
    // CASE in UPDATE pins the contract.
    let n = conn.execute(
        "INSERT INTO tasks (id, plan_id, title, status, started_at, created_at) \
         VALUES (?1, ?2, '', 'running', ?3, ?3) \
         ON CONFLICT(id) DO UPDATE SET \
            status = CASE \
                WHEN status IN ('done', 'skipped', 'escalated') THEN status \
                ELSE 'running' \
            END, \
            started_at = COALESCE(started_at, excluded.started_at)",
        params![task_id, plan_id, now],
    )?;
    Ok(n)
}

fn update_task_terminal(
    conn: &Connection,
    payload: &Value,
    status: &str,
) -> Result<usize, PlanProjectorError> {
    let plan_id = require_str(payload, "plan_id", "task_terminal")?;
    let task_id = require_str(payload, "task_id", "task_terminal")?;
    let now = now_ms();
    // UPSERT — replay path may surface terminal events before started.
    let n = conn.execute(
        "INSERT INTO tasks (id, plan_id, title, status, completed_at, created_at) \
         VALUES (?1, ?2, '', ?3, ?4, ?4) \
         ON CONFLICT(id) DO UPDATE SET \
            status = excluded.status, \
            completed_at = excluded.completed_at",
        params![task_id, plan_id, status, now],
    )?;
    Ok(n)
}

fn update_task_failed(conn: &Connection, payload: &Value) -> Result<usize, PlanProjectorError> {
    let plan_id = require_str(payload, "plan_id", "task_failed")?;
    let task_id = require_str(payload, "task_id", "task_failed")?;
    let failure_count = require_u64(payload, "failure_count", "task_failed")?;
    let now = now_ms();
    let n = conn.execute(
        "INSERT INTO tasks (id, plan_id, title, status, failure_count, created_at) \
         VALUES (?1, ?2, '', 'failed', ?3, ?4) \
         ON CONFLICT(id) DO UPDATE SET \
            status = 'failed', \
            failure_count = excluded.failure_count",
        params![
            task_id,
            plan_id,
            i64::try_from(failure_count).unwrap_or(i64::MAX),
            now,
        ],
    )?;
    Ok(n)
}

fn update_task_escalated(conn: &Connection, payload: &Value) -> Result<usize, PlanProjectorError> {
    let plan_id = require_str(payload, "plan_id", "task_escalated")?;
    let task_id = require_str(payload, "task_id", "task_escalated")?;
    let failure_count = require_u64(payload, "failure_count", "task_escalated")?;
    let max_failures = require_u64(payload, "max_failures", "task_escalated")?;
    let now = now_ms();
    let n = conn.execute(
        "INSERT INTO tasks (id, plan_id, title, status, failure_count, max_failures, completed_at, created_at) \
         VALUES (?1, ?2, '', 'escalated', ?3, ?4, ?5, ?5) \
         ON CONFLICT(id) DO UPDATE SET \
            status = 'escalated', \
            failure_count = excluded.failure_count, \
            max_failures = excluded.max_failures, \
            completed_at = excluded.completed_at",
        params![
            task_id,
            plan_id,
            i64::try_from(failure_count).unwrap_or(i64::MAX),
            i64::try_from(max_failures).unwrap_or(i64::MAX),
            now,
        ],
    )?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
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

    fn seed_signal(conn: &Connection, id: &str, payload: &Value) {
        conn.execute(
            "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
             VALUES (?1, 's1', 'agent', 'plan_or_task', '0', ?2, 'plan_create')",
            params![id, serde_json::to_string(payload).unwrap()],
        )
        .expect("insert signal");
    }

    fn select_plan_status(conn: &Connection, plan_id: &str) -> Option<String> {
        conn.query_row(
            "SELECT status FROM plans WHERE id = ?1",
            params![plan_id],
            |r| r.get::<_, String>(0),
        )
        .ok()
    }

    fn select_task_status(conn: &Connection, task_id: &str) -> Option<String> {
        conn.query_row(
            "SELECT status FROM tasks WHERE id = ?1",
            params![task_id],
            |r| r.get::<_, String>(0),
        )
        .ok()
    }

    #[test]
    fn project_signal_unknown_id_errors() {
        let (_dir, conn) = open();
        let err = project_signal(&conn, "nope").unwrap_err();
        assert!(matches!(err, PlanProjectorError::SignalNotFound(_)));
    }

    #[test]
    fn project_unknown_event_type_is_noop() {
        let (_dir, conn) = open();
        seed_signal(&conn, "sig-x", &json!({"type": "totally_unknown"}));
        let n = project_signal(&conn, "sig-x").expect("project");
        assert_eq!(n, 0);
    }

    #[test]
    fn project_empty_payload_is_noop() {
        let (_dir, conn) = open();
        // Insert a signal with empty payload_json directly.
        conn.execute(
            "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
             VALUES ('sig-empty', 's1', 'agent', 'plan', '0', '', 'plan_create')",
            [],
        )
        .expect("seed");
        let n = project_signal(&conn, "sig-empty").expect("project");
        assert_eq!(n, 0);
    }

    #[test]
    fn plan_created_with_approval_required_lands_pending_approval() {
        let (_dir, conn) = open();
        seed_signal(
            &conn,
            "sig-1",
            &json!({
                "type": "plan_created",
                "plan_id": "p1",
                "title": "Migrate auth",
                "task_count": 3,
                "approval_required": true
            }),
        );
        project_signal(&conn, "sig-1").expect("project");
        assert_eq!(
            select_plan_status(&conn, "p1").as_deref(),
            Some("pending_approval")
        );
    }

    #[test]
    fn plan_created_without_approval_required_lands_approved() {
        let (_dir, conn) = open();
        seed_signal(
            &conn,
            "sig-1",
            &json!({
                "type": "plan_created",
                "plan_id": "p1",
                "title": "Auto plan",
                "task_count": 1,
                "approval_required": false
            }),
        );
        project_signal(&conn, "sig-1").expect("project");
        assert_eq!(select_plan_status(&conn, "p1").as_deref(), Some("approved"));
    }

    #[test]
    fn plan_created_missing_required_field_errors() {
        let (_dir, conn) = open();
        // Missing approval_required
        seed_signal(
            &conn,
            "sig-1",
            &json!({"type": "plan_created", "plan_id": "p1", "title": "T", "task_count": 0}),
        );
        let err = project_signal(&conn, "sig-1").unwrap_err();
        assert!(matches!(
            err,
            PlanProjectorError::InvalidPayload {
                field: "approval_required",
                ..
            }
        ));
    }

    #[test]
    fn double_projection_is_idempotent() {
        let (_dir, conn) = open();
        let payload = json!({
            "type": "plan_created",
            "plan_id": "p1",
            "title": "T",
            "task_count": 1,
            "approval_required": true
        });
        seed_signal(&conn, "sig-1", &payload);
        project_signal(&conn, "sig-1").expect("first");
        project_signal(&conn, "sig-1").expect("second");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM plans WHERE id = 'p1'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(count, 1, "ON CONFLICT must keep row count at 1");
    }

    #[test]
    fn plan_approved_transitions_status_and_sets_approved_at() {
        let (_dir, conn) = open();
        // Seed plan first.
        seed_signal(
            &conn,
            "sig-1",
            &json!({
                "type": "plan_created",
                "plan_id": "p1",
                "title": "T",
                "task_count": 1,
                "approval_required": true
            }),
        );
        project_signal(&conn, "sig-1").expect("create");
        seed_signal(
            &conn,
            "sig-2",
            &json!({"type": "plan_approved", "plan_id": "p1", "approved_by": "user"}),
        );
        project_signal(&conn, "sig-2").expect("approve");
        let approved_at: Option<i64> = conn
            .query_row("SELECT approved_at FROM plans WHERE id = 'p1'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert!(approved_at.is_some());
        assert_eq!(select_plan_status(&conn, "p1").as_deref(), Some("approved"));
    }

    #[test]
    fn plan_complete_sets_completed_at() {
        let (_dir, conn) = open();
        seed_signal(
            &conn,
            "sig-1",
            &json!({
                "type": "plan_created",
                "plan_id": "p1",
                "title": "T",
                "task_count": 0,
                "approval_required": false
            }),
        );
        project_signal(&conn, "sig-1").expect("create");
        seed_signal(
            &conn,
            "sig-2",
            &json!({"type": "plan_complete", "plan_id": "p1", "duration_ms": 500}),
        );
        project_signal(&conn, "sig-2").expect("complete");
        let completed_at: Option<i64> = conn
            .query_row("SELECT completed_at FROM plans WHERE id = 'p1'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert!(completed_at.is_some());
        assert_eq!(select_plan_status(&conn, "p1").as_deref(), Some("complete"));
    }

    #[test]
    fn plan_aborted_transitions() {
        let (_dir, conn) = open();
        seed_signal(
            &conn,
            "sig-1",
            &json!({
                "type": "plan_created", "plan_id": "p1", "title": "T",
                "task_count": 0, "approval_required": false
            }),
        );
        project_signal(&conn, "sig-1").unwrap();
        seed_signal(
            &conn,
            "sig-2",
            &json!({"type": "plan_aborted", "plan_id": "p1", "reason": "user"}),
        );
        project_signal(&conn, "sig-2").unwrap();
        assert_eq!(select_plan_status(&conn, "p1").as_deref(), Some("aborted"));
    }

    #[test]
    fn plan_revised_returns_to_pending_approval() {
        let (_dir, conn) = open();
        seed_signal(
            &conn,
            "sig-1",
            &json!({
                "type": "plan_created", "plan_id": "p1", "title": "T",
                "task_count": 1, "approval_required": false
            }),
        );
        project_signal(&conn, "sig-1").unwrap();
        seed_signal(
            &conn,
            "sig-2",
            &json!({"type": "plan_revised", "plan_id": "p1", "revision_reason": "x"}),
        );
        project_signal(&conn, "sig-2").unwrap();
        assert_eq!(
            select_plan_status(&conn, "p1").as_deref(),
            Some("pending_approval")
        );
    }

    #[test]
    fn task_started_inserts_running_row() {
        let (_dir, conn) = open();
        // Plan must exist for FK constraint.
        seed_signal(
            &conn,
            "sig-p",
            &json!({
                "type": "plan_created", "plan_id": "p1", "title": "T",
                "task_count": 1, "approval_required": false
            }),
        );
        project_signal(&conn, "sig-p").unwrap();
        seed_signal(
            &conn,
            "sig-t",
            &json!({
                "type": "task_started", "plan_id": "p1",
                "task_id": "t1", "agent_id": "a1"
            }),
        );
        project_signal(&conn, "sig-t").unwrap();
        assert_eq!(select_task_status(&conn, "t1").as_deref(), Some("running"));
    }

    #[test]
    fn out_of_order_terminal_then_started_keeps_terminal() {
        let (_dir, conn) = open();
        // Seed plan.
        seed_signal(
            &conn,
            "sig-p",
            &json!({
                "type": "plan_created", "plan_id": "p1", "title": "T",
                "task_count": 1, "approval_required": false
            }),
        );
        project_signal(&conn, "sig-p").unwrap();
        // task_completed BEFORE task_started (replay-only path).
        seed_signal(
            &conn,
            "sig-tc",
            &json!({
                "type": "task_completed", "plan_id": "p1",
                "task_id": "t1", "duration_ms": 200
            }),
        );
        project_signal(&conn, "sig-tc").unwrap();
        seed_signal(
            &conn,
            "sig-ts",
            &json!({
                "type": "task_started", "plan_id": "p1",
                "task_id": "t1", "agent_id": "a1"
            }),
        );
        project_signal(&conn, "sig-ts").unwrap();
        // Terminal status preserved.
        assert_eq!(select_task_status(&conn, "t1").as_deref(), Some("done"));
    }

    #[test]
    fn task_failed_records_failure_count() {
        let (_dir, conn) = open();
        seed_signal(
            &conn,
            "sig-p",
            &json!({
                "type": "plan_created", "plan_id": "p1", "title": "T",
                "task_count": 1, "approval_required": false
            }),
        );
        project_signal(&conn, "sig-p").unwrap();
        seed_signal(
            &conn,
            "sig-tf",
            &json!({
                "type": "task_failed", "plan_id": "p1",
                "task_id": "t1", "error": "boom", "failure_count": 2
            }),
        );
        project_signal(&conn, "sig-tf").unwrap();
        let fc: i64 = conn
            .query_row("SELECT failure_count FROM tasks WHERE id = 't1'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(fc, 2);
    }

    #[test]
    fn task_escalated_records_failure_and_max() {
        let (_dir, conn) = open();
        seed_signal(
            &conn,
            "sig-p",
            &json!({
                "type": "plan_created", "plan_id": "p1", "title": "T",
                "task_count": 1, "approval_required": false
            }),
        );
        project_signal(&conn, "sig-p").unwrap();
        seed_signal(
            &conn,
            "sig-te",
            &json!({
                "type": "task_escalated", "plan_id": "p1",
                "task_id": "t1", "failure_count": 3, "max_failures": 3
            }),
        );
        project_signal(&conn, "sig-te").unwrap();
        let (status, fc, mx): (String, i64, i64) = conn
            .query_row(
                "SELECT status, failure_count, max_failures FROM tasks WHERE id = 't1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(status, "escalated");
        assert_eq!(fc, 3);
        assert_eq!(mx, 3);
    }

    #[test]
    fn task_skipped_lands_skipped() {
        let (_dir, conn) = open();
        seed_signal(
            &conn,
            "sig-p",
            &json!({
                "type": "plan_created", "plan_id": "p1", "title": "T",
                "task_count": 1, "approval_required": false
            }),
        );
        project_signal(&conn, "sig-p").unwrap();
        seed_signal(
            &conn,
            "sig-ts",
            &json!({
                "type": "task_skipped", "plan_id": "p1",
                "task_id": "t1", "reason": "HITL skip"
            }),
        );
        project_signal(&conn, "sig-ts").unwrap();
        assert_eq!(select_task_status(&conn, "t1").as_deref(), Some("skipped"));
    }

    #[test]
    fn task_rolled_back_lands_failed() {
        let (_dir, conn) = open();
        seed_signal(
            &conn,
            "sig-p",
            &json!({
                "type": "plan_created", "plan_id": "p1", "title": "T",
                "task_count": 1, "approval_required": false
            }),
        );
        project_signal(&conn, "sig-p").unwrap();
        seed_signal(
            &conn,
            "sig-tr",
            &json!({
                "type": "task_rolled_back", "plan_id": "p1",
                "task_id": "t1", "snapshot_id": "snap-1"
            }),
        );
        project_signal(&conn, "sig-tr").unwrap();
        assert_eq!(select_task_status(&conn, "t1").as_deref(), Some("failed"));
    }

    #[test]
    fn task_started_missing_task_id_errors() {
        let (_dir, conn) = open();
        seed_signal(
            &conn,
            "sig-p",
            &json!({
                "type": "plan_created", "plan_id": "p1", "title": "T",
                "task_count": 1, "approval_required": false
            }),
        );
        project_signal(&conn, "sig-p").unwrap();
        seed_signal(
            &conn,
            "sig-ts",
            &json!({"type": "task_started", "plan_id": "p1"}),
        );
        let err = project_signal(&conn, "sig-ts").unwrap_err();
        assert!(matches!(
            err,
            PlanProjectorError::InvalidPayload {
                field: "task_id",
                ..
            }
        ));
    }

    #[test]
    fn malformed_payload_json_errors() {
        let (_dir, conn) = open();
        conn.execute(
            "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
             VALUES ('sig-bad', 's1', 'agent', 'x', '0', 'not-json', 'plan_create')",
            [],
        )
        .unwrap();
        let err = project_signal(&conn, "sig-bad").unwrap_err();
        assert!(matches!(err, PlanProjectorError::Json(_)));
    }
}
