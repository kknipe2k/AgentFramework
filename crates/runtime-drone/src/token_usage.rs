//! `token_usage` projection — drone-internal continuous projector
//! (M07.D2, ADR-0011 d; closes the M06.5 `token_usage = 0` finding).
//!
//! The third drone projector, parallel to [`crate::vdr`] +
//! [`crate::plan_projector`]: it runs in the SAME `handle_write_signal`
//! transaction (NOT a new `DroneCommand` — no IPC-protocol change, so
//! no §11 ADR). Per spec §2c.3 + §937 a `token_usage` signal carries
//! `{ input, output, model, cost_usd }`; the multi-turn
//! agent-with-tools loop is the first production emitter of it
//! (`AgentEvent::TokenUsage`), so this projector is what finally
//! populates the `token_usage` table in a real session.
//!
//! Idempotence: the projected row's primary key IS the contributing
//! signal id, so `INSERT OR IGNORE` makes re-projecting the same signal
//! a zero-row no-op (mirrors the `vdr` `contributing_signal_id` UNIQUE
//! idiom without needing a new column / migration).

use rusqlite::{params, Connection};
use serde_json::Value;
use thiserror::Error;

/// Errors raised by the projector.
#[derive(Debug, Error)]
pub enum TokenUsageProjectorError {
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

struct SignalRow {
    event: String,
    session_id: String,
    timestamp: String,
    payload_json: String,
}

fn read_signal_row(
    conn: &Connection,
    signal_id: &str,
) -> Result<SignalRow, TokenUsageProjectorError> {
    conn.query_row(
        "SELECT event, session_id, timestamp, payload_json FROM signals WHERE id = ?1",
        params![signal_id],
        |r| {
            Ok(SignalRow {
                event: r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                session_id: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                timestamp: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                payload_json: r.get::<_, Option<String>>(3)?.unwrap_or_default(),
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            TokenUsageProjectorError::SignalNotFound(signal_id.to_string())
        }
        other => TokenUsageProjectorError::from(other),
    })
}

/// Project the signal identified by `signal_id` into the `token_usage`
/// table.
///
/// Returns the number of rows inserted: `1` for a new `token_usage`
/// signal, `0` if the signal is not a `token_usage` event OR if it was
/// already projected (the `INSERT OR IGNORE` on the signal-id primary
/// key).
///
/// # Errors
///
/// [`TokenUsageProjectorError::SignalNotFound`] if `signal_id` is
/// absent, [`TokenUsageProjectorError::Sqlite`] on database errors,
/// [`TokenUsageProjectorError::Json`] if `payload_json` is malformed.
pub fn project_signal(
    conn: &Connection,
    signal_id: &str,
) -> Result<usize, TokenUsageProjectorError> {
    let SignalRow {
        event,
        session_id,
        timestamp,
        payload_json,
    } = read_signal_row(conn, signal_id)?;

    // Keyed on the AgentEvent serde tag (`AgentEvent::TokenUsage`
    // serializes `type = "token_usage"`, carried in `signals.event`);
    // the coarse `signals.type` is "agent" for this kind, so `event`
    // is the correct discriminator (parallel to vdr keying on `type`).
    if event != "token_usage" {
        return Ok(0);
    }
    let payload: Value = if payload_json.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&payload_json)?
    };

    let input_tokens = payload.get("input").and_then(Value::as_i64).unwrap_or(0);
    let output_tokens = payload.get("output").and_then(Value::as_i64).unwrap_or(0);
    let model = payload
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let cost_usd = payload
        .get("cost_usd")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let agent_id = payload
        .get("agent_id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let ts = timestamp.parse::<i64>().unwrap_or(0);

    let inserted = conn.execute(
        "INSERT OR IGNORE INTO token_usage (\
            id, session_id, agent_id, timestamp, model, input_tokens, output_tokens, cost_usd\
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            signal_id,
            session_id,
            agent_id,
            ts,
            model,
            input_tokens,
            output_tokens,
            cost_usd,
        ],
    )?;
    Ok(inserted)
}
