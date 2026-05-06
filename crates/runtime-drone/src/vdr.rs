//! VDR projection — drone-internal continuous projector.
//!
//! Reads decision (4) + verify (5) signals from the `signals` table and
//! produces correlated rows in the `vdr` table. Per spec §2b: signals
//! are the source of truth, VDR is the read-optimized projection of
//! signal kinds 4 and 5.
//!
//! Idempotence: re-running [`project_signal`] on the same signal id
//! produces zero rows on the second call. Enforced by a `UNIQUE INDEX`
//! on `vdr.contributing_signal_id` that the schema migration in
//! [`crate::db`] creates. `INSERT OR IGNORE` is the SQL idiom.
//!
//! Triggering: this module exposes pure projection functions. Stage E
//! ships the projector + tests; signal-emission integration lands in
//! M04+ when the agent SDK starts persisting signals. The future
//! `WriteSignal` command-handler arm will call [`project_signal`] after
//! each successful insert.

use rusqlite::{params, Connection};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

/// Errors raised by the projector.
#[derive(Debug, Error)]
pub enum VdrError {
    /// Signal id not found in the signals table.
    #[error("signal not found: {0}")]
    SignalNotFound(String),
    /// Underlying rusqlite error.
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    /// JSON parse error reading `signals.payload_json`.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Project the signal identified by `signal_id` into the VDR table.
///
/// Returns the number of rows inserted: `1` for a new decision/verify
/// signal, `0` if the signal type is not projection-eligible (tool,
/// skill, agent, error, hitl, session) OR if the signal was already
/// projected (UNIQUE constraint).
///
/// # Errors
///
/// Returns [`VdrError::SignalNotFound`] if `signal_id` is not in the
/// signals table, [`VdrError::Sqlite`] on database errors, or
/// [`VdrError::Json`] if the signal's `payload_json` is malformed.
pub fn project_signal(conn: &Connection, signal_id: &str) -> Result<usize, VdrError> {
    let row = read_signal_row(conn, signal_id)?;
    let SignalRow {
        sig_type,
        session_id,
        payload_json,
        context_type,
    } = row;

    if !is_projection_eligible(&sig_type) {
        return Ok(0);
    }
    let payload: Value = if payload_json.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&payload_json)?
    };

    let agent_id = payload
        .get("agent_id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let (decision, rationale, tool_invoked, outcome) = match sig_type.as_str() {
        "decision" => {
            let d = payload
                .get("decision")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let r = payload
                .get("rationale")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let t = payload
                .get("tool_used")
                .and_then(Value::as_str)
                .map(String::from);
            (d, r, t, None::<String>)
        }
        "verify" => {
            let hook = payload
                .get("hook_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let passed = payload
                .get("passed")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let outcome = if passed { "pass" } else { "fail" }.to_string();
            (hook, outcome.clone(), None, Some(outcome))
        }
        _ => unreachable!("guarded by is_projection_eligible"),
    };

    let inserted = conn.execute(
        "INSERT OR IGNORE INTO vdr (\
            id, session_id, agent_id, timestamp, decision, rationale, tool_invoked, \
            tool_input_json, tool_output_json, token_cost_usd, outcome, snapshot_id, \
            signal_ids, context_type, contributing_signal_id\
         ) VALUES (?1, ?2, ?3, 0, ?4, ?5, ?6, NULL, NULL, NULL, ?7, NULL, ?8, ?9, ?10)",
        params![
            Uuid::new_v4().to_string(),
            session_id,
            agent_id,
            decision,
            rationale,
            tool_invoked,
            outcome,
            signal_id,
            context_type,
            signal_id,
        ],
    )?;
    Ok(inserted)
}

/// Project every projection-eligible signal in `session_id`. Returns
/// total rows inserted. Idempotent: replaying produces zero new rows
/// once the session has been fully projected.
///
/// # Errors
///
/// Same as [`project_signal`].
pub fn project_session(conn: &Connection, session_id: &str) -> Result<usize, VdrError> {
    let ids: Vec<String> = {
        let mut stmt =
            conn.prepare("SELECT id FROM signals WHERE session_id = ?1 ORDER BY timestamp, id")?;
        let rows = stmt
            .query_map(params![session_id], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };

    let mut total = 0;
    for id in ids {
        total += project_signal(conn, &id)?;
    }
    Ok(total)
}

/// Read all signals for a session as JSON objects, ordered by
/// `(timestamp, id)`. Used by the `ReadSignals` command path so main
/// can re-emit them as `AgentEvent`s for graph replay.
///
/// # Errors
///
/// Returns [`VdrError::Sqlite`] on database errors.
pub fn signals_for_session(conn: &Connection, session_id: &str) -> Result<Vec<Value>, VdrError> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, type, event, timestamp, duration_ms, payload_json, \
                pre_signal_id, parent_signal_id, retry_of, context_type \
         FROM signals WHERE session_id = ?1 ORDER BY timestamp, id",
    )?;
    let rows = stmt
        .query_map(params![session_id], |r| {
            let payload: String = r.get::<_, Option<String>>(6)?.unwrap_or_default();
            let payload_value: Value = if payload.is_empty() {
                Value::Null
            } else {
                serde_json::from_str(&payload).unwrap_or(Value::String(payload))
            };
            let mut obj = serde_json::Map::new();
            obj.insert("id".into(), Value::String(r.get::<_, String>(0)?));
            obj.insert(
                "session_id".into(),
                json_str_or_null(r.get::<_, Option<String>>(1)?),
            );
            obj.insert(
                "type".into(),
                json_str_or_null(r.get::<_, Option<String>>(2)?),
            );
            obj.insert(
                "event".into(),
                json_str_or_null(r.get::<_, Option<String>>(3)?),
            );
            obj.insert(
                "timestamp".into(),
                json_str_or_null(r.get::<_, Option<String>>(4)?),
            );
            obj.insert(
                "duration_ms".into(),
                r.get::<_, Option<i64>>(5)?
                    .map_or(Value::Null, |n| Value::Number(n.into())),
            );
            obj.insert("payload_json".into(), payload_value);
            obj.insert(
                "pre_signal_id".into(),
                json_str_or_null(r.get::<_, Option<String>>(7)?),
            );
            obj.insert(
                "parent_signal_id".into(),
                json_str_or_null(r.get::<_, Option<String>>(8)?),
            );
            obj.insert(
                "retry_of".into(),
                json_str_or_null(r.get::<_, Option<String>>(9)?),
            );
            obj.insert(
                "context_type".into(),
                json_str_or_null(r.get::<_, Option<String>>(10)?),
            );
            Ok(Value::Object(obj))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Execute a SELECT statement and return its rows as JSON objects.
///
/// Caller MUST validate `is_select_only` first; this function is the
/// lower-level SQL execution. Hand-rolled `Row → serde_json::Value`
/// per CLAUDE.md §6 (no third-party crate without `cargo deny check`
/// passing).
///
/// # Errors
///
/// Returns [`VdrError::Sqlite`] on prepare/query errors.
pub fn execute_select(conn: &Connection, sql: &str) -> Result<Vec<Value>, VdrError> {
    let mut stmt = conn.prepare(sql)?;
    let column_names: Vec<String> = stmt.column_names().into_iter().map(String::from).collect();
    let mut rows = stmt.query([])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        let mut obj = serde_json::Map::new();
        for (i, name) in column_names.iter().enumerate() {
            obj.insert(name.clone(), value_at(row, i)?);
        }
        out.push(Value::Object(obj));
    }
    Ok(out)
}

/// Validate that `sql` is a single SELECT statement.
///
/// Parser-style validation (not regex). Per Stage E E.1 Decision #3:
/// regex-matching `^SELECT` is trivially bypassed (`SELECT 1; DROP
/// TABLE foo` slips through). This check tokenizes:
///
/// 1. Rejects empty input outright.
/// 2. Rejects multiple statements via embedded `;` (one trailing
///    semicolon allowed; embedded ones reject — `SELECT 1; DROP TABLE
///    foo` is rejected at this layer).
/// 3. Rejects `pragma_*` shape via lowercase prefix.
/// 4. Rejects every leading keyword that isn't `SELECT` — DROP, DELETE,
///    INSERT, UPDATE, ALTER, CREATE, ATTACH, REPLACE, REINDEX, ANALYZE,
///    VACUUM, BEGIN, COMMIT, ROLLBACK, EXPLAIN-as-prefix, WITH, etc.
///    by allowlisting only `select` as the leading keyword.
///
/// We deliberately do NOT call `Connection::prepare()` for SQL-level
/// validation because prepare resolves table references against the
/// connection's schema; an in-memory probe without the schema would
/// reject every legitimate query. Execution-time errors (malformed
/// SELECT, missing column names) surface from `execute_select` via
/// `VdrError::Sqlite` as a Critical alert — the security boundary
/// here is "doesn't mutate state", which the lexical check enforces.
#[must_use]
pub fn is_select_only(sql: &str) -> bool {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return false;
    }
    let stripped = trimmed.trim_end_matches(';').trim_end();
    if stripped.contains(';') {
        return false;
    }
    let lower = stripped.to_lowercase();
    if lower.starts_with("pragma") {
        return false;
    }
    // Allowlist: only SELECT (followed by whitespace) is permitted.
    // `selectable` and `select_*` table names that happen to share the
    // prefix don't pass this check because they need a leading SELECT
    // keyword to be valid SQL anyway.
    let mut chars = lower.chars();
    let mut prefix = String::with_capacity(6);
    for _ in 0..6 {
        if let Some(c) = chars.next() {
            prefix.push(c);
        } else {
            return false;
        }
    }
    if prefix != "select" {
        return false;
    }
    // Char after `select` must be whitespace or end-of-input (for
    // `SELECT;` after stripping).
    // `Option::is_none_or` stabilized in Rust 1.82; project MSRV is 1.80.
    chars.next().map_or(true, char::is_whitespace)
}

fn is_projection_eligible(sig_type: &str) -> bool {
    matches!(sig_type, "decision" | "verify")
}

struct SignalRow {
    sig_type: String,
    session_id: String,
    payload_json: String,
    context_type: String,
}

fn read_signal_row(conn: &Connection, signal_id: &str) -> Result<SignalRow, VdrError> {
    conn.query_row(
        "SELECT type, session_id, payload_json, context_type FROM signals WHERE id = ?1",
        params![signal_id],
        |r| {
            Ok(SignalRow {
                sig_type: r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                session_id: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                payload_json: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                context_type: r.get::<_, Option<String>>(3)?.unwrap_or_default(),
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => VdrError::SignalNotFound(signal_id.to_string()),
        other => VdrError::from(other),
    })
}

fn json_str_or_null(s: Option<String>) -> Value {
    s.map_or(Value::Null, Value::String)
}

fn value_at(row: &rusqlite::Row<'_>, idx: usize) -> Result<Value, rusqlite::Error> {
    use rusqlite::types::ValueRef;
    match row.get_ref(idx)? {
        ValueRef::Null => Ok(Value::Null),
        ValueRef::Integer(i) => Ok(Value::Number(i.into())),
        ValueRef::Real(f) => Ok(serde_json::Number::from_f64(f).map_or(Value::Null, Value::Number)),
        ValueRef::Text(t) => Ok(Value::String(String::from_utf8_lossy(t).into_owned())),
        ValueRef::Blob(b) => Ok(Value::String(format!("<{} bytes blob>", b.len()))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
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

    fn seed_signal(conn: &Connection, id: &str, sig_type: &str, payload: &str) {
        conn.execute(
            "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
             VALUES (?1, 's1', ?2, ?2, '0', ?3, 'agent_loop')",
            params![id, sig_type, payload],
        )
        .expect("insert signal");
    }

    #[test]
    fn project_signal_for_unknown_id_errors() {
        let (_dir, conn) = open();
        let err = project_signal(&conn, "nope").unwrap_err();
        assert!(matches!(err, VdrError::SignalNotFound(_)));
    }

    #[test]
    fn execute_select_returns_rows_keyed_by_column_name() {
        let (_dir, conn) = open();
        seed_signal(&conn, "s-1", "tool", r#"{"x":1}"#);
        let rows = execute_select(&conn, "SELECT id, type FROM signals").expect("select");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("id").and_then(Value::as_str), Some("s-1"));
        assert_eq!(rows[0].get("type").and_then(Value::as_str), Some("tool"));
    }

    #[test]
    fn execute_select_handles_real_and_null_columns() {
        let (_dir, conn) = open();
        // token_usage table has a REAL column (cost_usd) and INTEGER columns.
        conn.execute(
            "INSERT INTO token_usage (id, session_id, agent_id, timestamp, model, input_tokens, output_tokens, cost_usd) \
             VALUES ('t1', 's1', 'a1', 0, NULL, 100, 50, 0.0125)",
            [],
        )
        .expect("seed");
        let rows = execute_select(
            &conn,
            "SELECT cost_usd, model, input_tokens FROM token_usage",
        )
        .expect("select");
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].get("cost_usd").and_then(Value::as_f64),
            Some(0.0125)
        );
        assert!(rows[0].get("model").is_some_and(Value::is_null));
        assert_eq!(
            rows[0].get("input_tokens").and_then(Value::as_i64),
            Some(100)
        );
    }

    #[test]
    fn signals_for_session_emits_json_objects_in_order() {
        let (_dir, conn) = open();
        // Insert in reverse-timestamp order; verify ORDER BY restores order.
        for (id, ts) in [("c", "3"), ("a", "1"), ("b", "2")] {
            conn.execute(
                "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
                 VALUES (?1, 's1', 'agent', 'spawned', ?2, '{}', 'agent_loop')",
                params![id, ts],
            )
            .expect("insert");
        }
        let rows = signals_for_session(&conn, "s1").expect("read");
        let ids: Vec<&str> = rows
            .iter()
            .filter_map(|r| r.get("id").and_then(Value::as_str))
            .collect();
        assert_eq!(ids, vec!["a", "b", "c"]);
    }

    #[test]
    fn signals_for_session_treats_invalid_payload_as_string_value() {
        let (_dir, conn) = open();
        conn.execute(
            "INSERT INTO signals (id, session_id, type, event, timestamp, payload_json, context_type) \
             VALUES ('bad', 's1', 'tool', 'invoked', '0', 'not json', 'agent_loop')",
            [],
        )
        .expect("insert");
        let rows = signals_for_session(&conn, "s1").expect("read");
        // Falls back to wrapping the raw text in a JSON string so callers
        // see the corruption rather than panicking.
        assert_eq!(
            rows[0].get("payload_json").and_then(Value::as_str),
            Some("not json")
        );
    }
}
