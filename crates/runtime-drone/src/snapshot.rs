//! Append-only snapshot writer with SHA-256 `state_hash`.
//!
//! Per `agent-runtime-spec.md` §1 (Session Snapshots).
//!
//! Snapshots are immutable: a write is always an `INSERT`, never an
//! `UPDATE`. The `state_hash` is `sha256(state_json_canonical)` where
//! the canonical form is `serde_json::to_string` of the value.
//!
//! ## M04 Stage B — projection-aware recovery (spec §1b)
//!
//! `write` continues to accept arbitrary `state: Value`. The SDK-side
//! caller (M04+) extends the state to `{ events, plans, tasks }` so
//! recovery can rebuild the projection without re-running the projector.
//!
//! `recover_session_state` reads the latest snapshot + the projected
//! `plans` + `tasks` tables, normalizes currently-running tasks to
//! `pending` (per spec §1b — the agent process that was running is
//! dead), and surfaces tool-call uncertainty for the renderer's UI
//! prompt (`tool_invoked` without matching `tool_result`).

use rusqlite::{params, Connection};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use uuid::Uuid;

/// Errors raised by `write`.
#[derive(Debug, Error)]
pub enum SnapshotError {
    /// Underlying rusqlite error.
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    /// JSON serialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Append a snapshot row for `session_id` with the given `reason` and
/// `state`. Returns the new snapshot UUID.
///
/// `event_type` in the table holds the human-readable `reason` string (e.g.
/// `"manual"`, `"sigterm"`, `"task_completed"`). The wire-level event
/// taxonomy in `runtime-core::event` carries the structured event names.
///
/// # Errors
///
/// Returns `SnapshotError::Json` if `state` cannot be serialized, or
/// `SnapshotError::Sqlite` if the row cannot be inserted.
pub fn write(
    conn: &Connection,
    session_id: &str,
    reason: &str,
    state: &Value,
) -> Result<String, SnapshotError> {
    let id = Uuid::new_v4().to_string();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX));
    let state_json = serde_json::to_string(state)?;
    let state_hash = sha256_hex(&state_json);

    conn.execute(
        "INSERT INTO snapshots (id, session_id, timestamp, event_type, state_json, state_hash) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, session_id, timestamp, reason, state_json, state_hash],
    )?;
    Ok(id)
}

fn sha256_hex(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Recovered session state ready for the SDK to resume from per spec §1b.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveredSession {
    /// The snapshot id this state was loaded from. `None` if the session
    /// has no snapshots yet.
    pub snapshot_id: Option<String>,
    /// Decoded `state_json` from the snapshot (or `Value::Null` when no
    /// snapshot). The SDK-side shape is `{ events, plans, tasks }` once
    /// M04+ callers extend the state.
    pub state: Value,
    /// Plan rows projected from signals. Recovery surfaces them in
    /// status order so the SDK can resume.
    pub plans: Vec<Value>,
    /// Task rows projected from signals. **Currently-running tasks are
    /// downgraded to `pending`** per spec §1b: the agent process that
    /// was running them is dead and must be re-scheduled.
    pub tasks: Vec<Value>,
    /// Set of `tool_invoked` signal IDs whose matching `tool_result`
    /// was not observed in the signal log. The renderer prompts the
    /// user with retry / skip / mark-complete / abort options for each
    /// (Stage F UI).
    pub uncertain_tool_invocations: Vec<String>,
}

/// Read the latest snapshot for `session_id` along with the projected
/// `plans` + `tasks` rows. Implements spec §1b recovery semantics:
///
/// - Currently-running tasks (`status = 'running'`) are returned as
///   `pending` — the agent process that was running them is gone.
/// - Tool-call uncertainty: any `tool_invoked` signal lacking a matching
///   `tool_result` is surfaced as uncertain. The renderer prompts the
///   user (Stage F).
///
/// # Errors
///
/// Returns `SnapshotError::Sqlite` for database errors.
pub fn recover_session_state(
    conn: &Connection,
    session_id: &str,
) -> Result<RecoveredSession, SnapshotError> {
    let (snapshot_id, state) = read_latest_snapshot(conn, session_id);
    let plans = read_plans(conn, session_id)?;
    let tasks = read_tasks_normalized(conn, session_id)?;
    let uncertain_tool_invocations = uncertain_tool_invocations(conn, session_id)?;
    Ok(RecoveredSession {
        snapshot_id,
        state,
        plans,
        tasks,
        uncertain_tool_invocations,
    })
}

fn read_latest_snapshot(conn: &Connection, session_id: &str) -> (Option<String>, Value) {
    let row = conn
        .query_row(
            "SELECT id, state_json FROM snapshots \
             WHERE session_id = ?1 \
             ORDER BY timestamp DESC, id DESC LIMIT 1",
            params![session_id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .ok();
    row.map_or((None, Value::Null), |(id, state_json)| {
        let value = serde_json::from_str::<Value>(&state_json).unwrap_or(Value::Null);
        (Some(id), value)
    })
}

fn read_plans(conn: &Connection, session_id: &str) -> Result<Vec<Value>, SnapshotError> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, title, status, approval_required, loop_policy, \
                hitl_checkpoints, risks, created_by, created_at, approved_at, completed_at \
         FROM plans WHERE session_id = ?1 ORDER BY created_at",
    )?;
    let rows = stmt
        .query_map(params![session_id], |r| {
            Ok(serde_json::json!({
                "id": r.get::<_, String>(0)?,
                "session_id": r.get::<_, String>(1)?,
                "title": r.get::<_, String>(2)?,
                "status": r.get::<_, String>(3)?,
                "approval_required": r.get::<_, i64>(4)? != 0,
                "loop_policy": r.get::<_, String>(5)?,
                "hitl_checkpoints": r.get::<_, String>(6)?,
                "risks": r.get::<_, String>(7)?,
                "created_by": r.get::<_, Option<String>>(8)?,
                "created_at": r.get::<_, i64>(9)?,
                "approved_at": r.get::<_, Option<i64>>(10)?,
                "completed_at": r.get::<_, Option<i64>>(11)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn read_tasks_normalized(conn: &Connection, session_id: &str) -> Result<Vec<Value>, SnapshotError> {
    // Join via plans.session_id since tasks doesn't carry session_id directly.
    let mut stmt = conn.prepare(
        "SELECT t.id, t.plan_id, t.title, t.status, t.failure_count, t.max_failures, \
                t.hitl, t.created_at, t.started_at, t.completed_at \
         FROM tasks t JOIN plans p ON t.plan_id = p.id \
         WHERE p.session_id = ?1 ORDER BY t.created_at",
    )?;
    let rows = stmt
        .query_map(params![session_id], |r| {
            let raw_status: String = r.get(3)?;
            // Spec §1b: currently-running tasks → pending on recovery.
            let normalized_status = if raw_status == "running" {
                "pending"
            } else {
                raw_status.as_str()
            }
            .to_string();
            Ok(serde_json::json!({
                "id": r.get::<_, String>(0)?,
                "plan_id": r.get::<_, String>(1)?,
                "title": r.get::<_, String>(2)?,
                "status": normalized_status,
                "failure_count": r.get::<_, i64>(4)?,
                "max_failures": r.get::<_, i64>(5)?,
                "hitl": r.get::<_, i64>(6)? != 0,
                "created_at": r.get::<_, i64>(7)?,
                "started_at": r.get::<_, Option<i64>>(8)?,
                "completed_at": r.get::<_, Option<i64>>(9)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn uncertain_tool_invocations(
    conn: &Connection,
    session_id: &str,
) -> Result<Vec<String>, SnapshotError> {
    // Spec §1b tool-call uncertainty. A `tool_invoked` payload contains
    // a tool_name + agent_id; the matching `tool_result` references the
    // same. We use signal id pairing: any tool-kind signal whose payload
    // type is `tool_invoked` and which has no later `tool_result` from
    // the same agent_id+tool_name is uncertain. Simple heuristic — Stage
    // F refines as needed.
    let mut stmt = conn.prepare(
        "SELECT s.id, s.payload_json FROM signals s \
         WHERE s.session_id = ?1 AND s.type = 'tool' \
         ORDER BY s.timestamp",
    )?;
    let mut invokes: Vec<(String, String, String)> = Vec::new(); // (signal_id, agent_id, tool_name)
    let mut results: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    for row in stmt.query_map(params![session_id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?.unwrap_or_default(),
        ))
    })? {
        let (sig_id, payload_json) = row?;
        if payload_json.is_empty() {
            continue;
        }
        let payload: Value = match serde_json::from_str(&payload_json) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let event_type = payload
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let agent_id = payload
            .get("agent_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let tool_name = payload
            .get("tool_name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        match event_type.as_str() {
            "tool_invoked" => invokes.push((sig_id, agent_id, tool_name)),
            "tool_result" => {
                results.insert((agent_id, tool_name));
            }
            _ => {}
        }
    }
    let uncertain: Vec<String> = invokes
        .into_iter()
        .filter(|(_, agent, tool)| !results.contains(&(agent.clone(), tool.clone())))
        .map(|(sig, _, _)| sig)
        .collect();
    Ok(uncertain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::TempDir;

    fn open() -> (TempDir, rusqlite::Connection) {
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

    #[test]
    fn snapshot_writes_correct_row() {
        let (_dir, conn) = open();
        let state = serde_json::json!({"k": "v"});
        let id = write(&conn, "s1", "test", &state).expect("write");

        let (got_session, got_reason, got_state, got_hash): (String, String, String, String) = conn
            .query_row(
                "SELECT session_id, event_type, state_json, state_hash FROM snapshots WHERE id = ?1",
                [&id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .expect("row");

        assert_eq!(got_session, "s1");
        assert_eq!(got_reason, "test");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&got_state).unwrap(),
            state
        );
        assert_eq!(got_hash.len(), 64, "sha256 hex is 64 chars");
    }

    #[test]
    fn snapshot_state_hash_is_sha256() {
        let (_dir, conn) = open();
        let state = serde_json::json!({"k": "v"});
        let id = write(&conn, "s1", "h", &state).expect("write");
        let got_hash: String = conn
            .query_row(
                "SELECT state_hash FROM snapshots WHERE id = ?1",
                [&id],
                |r| r.get(0),
            )
            .expect("row");
        let canonical = serde_json::to_string(&state).unwrap();
        let expected = sha256_hex(&canonical);
        assert_eq!(got_hash, expected);
    }

    #[test]
    fn snapshot_appends_does_not_update() {
        let (_dir, conn) = open();
        let state = serde_json::json!({"k": "v"});
        let _id1 = write(&conn, "s1", "a", &state).expect("write 1");
        let _id2 = write(&conn, "s1", "a", &state).expect("write 2");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get(0))
            .expect("count");
        assert_eq!(count, 2, "two writes must produce two rows (append-only)");
    }

    // ── M04 Stage B: recover_session_state (spec §1b) ─────────────────

    fn seed_plan(conn: &rusqlite::Connection, plan_id: &str, status: &str) {
        conn.execute(
            "INSERT INTO plans (id, session_id, title, status, approval_required, loop_policy, created_at) \
             VALUES (?1, 's1', 'T', ?2, 0, 'fresh_context_per_task', 0)",
            rusqlite::params![plan_id, status],
        )
        .expect("seed plan");
    }

    fn seed_task(conn: &rusqlite::Connection, task_id: &str, plan_id: &str, status: &str) {
        conn.execute(
            "INSERT INTO tasks (id, plan_id, title, status, created_at) \
             VALUES (?1, ?2, 'T', ?3, 0)",
            rusqlite::params![task_id, plan_id, status],
        )
        .expect("seed task");
    }

    fn seed_signal(conn: &rusqlite::Connection, id: &str, payload: &serde_json::Value) {
        conn.execute(
            "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
             VALUES (?1, 's1', 'tool', 'x', '0', ?2, 'tool_invoke')",
            rusqlite::params![id, serde_json::to_string(payload).unwrap()],
        )
        .expect("seed signal");
    }

    #[test]
    fn recover_with_no_snapshot_returns_null_state() {
        let (_dir, conn) = open();
        let r = recover_session_state(&conn, "s1").expect("recover");
        assert!(r.snapshot_id.is_none());
        assert!(r.state.is_null());
        assert!(r.plans.is_empty());
        assert!(r.tasks.is_empty());
        assert!(r.uncertain_tool_invocations.is_empty());
    }

    #[test]
    fn recover_returns_latest_snapshot_id_and_decoded_state() {
        let (_dir, conn) = open();
        // Seed with explicit timestamps so the LIMIT 1 latest-by-timestamp
        // ordering is deterministic; the production write() uses second-
        // granularity timestamps which can collide in a fast test.
        conn.execute(
            "INSERT INTO snapshots (id, session_id, timestamp, event_type, state_json, state_hash) \
             VALUES ('s-old', 's1', 100, 'first', '{\"v\":1}', 'h1')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO snapshots (id, session_id, timestamp, event_type, state_json, state_hash) \
             VALUES ('s-new', 's1', 200, 'second', '{\"v\":2}', 'h2')",
            [],
        )
        .unwrap();
        let r = recover_session_state(&conn, "s1").expect("recover");
        assert_eq!(r.snapshot_id.as_deref(), Some("s-new"));
        assert_eq!(r.state, serde_json::json!({"v": 2}));
    }

    #[test]
    fn recover_running_task_is_normalized_to_pending() {
        let (_dir, conn) = open();
        seed_plan(&conn, "p1", "approved");
        seed_task(&conn, "t1", "p1", "running");
        let r = recover_session_state(&conn, "s1").expect("recover");
        assert_eq!(r.plans.len(), 1);
        assert_eq!(r.tasks.len(), 1);
        assert_eq!(
            r.tasks[0].get("status").and_then(|v| v.as_str()),
            Some("pending"),
            "spec §1b: running → pending on recovery"
        );
    }

    #[test]
    fn recover_preserves_terminal_task_status() {
        let (_dir, conn) = open();
        seed_plan(&conn, "p1", "complete");
        seed_task(&conn, "t1", "p1", "done");
        seed_task(&conn, "t2", "p1", "skipped");
        let r = recover_session_state(&conn, "s1").expect("recover");
        let statuses: Vec<&str> = r
            .tasks
            .iter()
            .filter_map(|t| t.get("status").and_then(|v| v.as_str()))
            .collect();
        assert!(statuses.contains(&"done"));
        assert!(statuses.contains(&"skipped"));
    }

    #[test]
    fn recover_uncertain_tool_invocation_when_no_matching_result() {
        let (_dir, conn) = open();
        seed_signal(
            &conn,
            "sig-invoke",
            &serde_json::json!({
                "type": "tool_invoked",
                "agent_id": "a1",
                "tool_name": "Read"
            }),
        );
        let r = recover_session_state(&conn, "s1").expect("recover");
        assert_eq!(r.uncertain_tool_invocations, vec!["sig-invoke".to_string()]);
    }

    #[test]
    fn recover_no_uncertain_when_matching_result_present() {
        let (_dir, conn) = open();
        seed_signal(
            &conn,
            "sig-invoke",
            &serde_json::json!({
                "type": "tool_invoked",
                "agent_id": "a1",
                "tool_name": "Bash"
            }),
        );
        seed_signal(
            &conn,
            "sig-result",
            &serde_json::json!({
                "type": "tool_result",
                "agent_id": "a1",
                "tool_name": "Bash"
            }),
        );
        let r = recover_session_state(&conn, "s1").expect("recover");
        assert!(r.uncertain_tool_invocations.is_empty());
    }

    #[test]
    fn recover_skips_signals_with_invalid_payload() {
        let (_dir, conn) = open();
        // Insert a tool signal with non-JSON payload.
        conn.execute(
            "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
             VALUES ('bad', 's1', 'tool', 'x', '0', 'not json', 'tool_invoke')",
            [],
        )
        .unwrap();
        let r = recover_session_state(&conn, "s1").expect("recover");
        assert!(r.uncertain_tool_invocations.is_empty());
    }

    #[test]
    fn recover_skips_empty_payloads() {
        let (_dir, conn) = open();
        conn.execute(
            "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
             VALUES ('empty', 's1', 'tool', 'x', '0', '', 'tool_invoke')",
            [],
        )
        .unwrap();
        let r = recover_session_state(&conn, "s1").expect("recover");
        assert!(r.uncertain_tool_invocations.is_empty());
    }

    #[test]
    fn recover_returns_plans_in_created_order() {
        let (_dir, conn) = open();
        // Insert with different created_at to force ORDER BY exercise.
        conn.execute(
            "INSERT INTO plans (id, session_id, title, status, approval_required, loop_policy, created_at) \
             VALUES ('p2', 's1', 'B', 'approved', 0, 'fresh_context_per_task', 200)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO plans (id, session_id, title, status, approval_required, loop_policy, created_at) \
             VALUES ('p1', 's1', 'A', 'approved', 0, 'fresh_context_per_task', 100)",
            [],
        )
        .unwrap();
        let r = recover_session_state(&conn, "s1").expect("recover");
        let ids: Vec<&str> = r
            .plans
            .iter()
            .filter_map(|p| p.get("id").and_then(|v| v.as_str()))
            .collect();
        assert_eq!(ids, vec!["p1", "p2"]);
    }

    #[test]
    fn recovered_session_round_trips_via_serde() {
        let r = RecoveredSession {
            snapshot_id: Some("snap-1".into()),
            state: serde_json::json!({"x": 1}),
            plans: vec![],
            tasks: vec![],
            uncertain_tool_invocations: vec!["sig-1".into()],
        };
        let cloned = r.clone();
        assert_eq!(r, cloned);
    }
}
