//! Drone IPC types — `DroneEvent` / `DroneCommand` and supporting enums.
//!
//! Specified in `agent-runtime-spec.md` §1d. These types are the wire format
//! for main↔drone IPC over Unix domain socket / Windows named pipe.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Status reported in [`DroneEvent::Heartbeat`].
///
/// Matches the strings written by `runtime-drone::heartbeat::run` to the
/// `heartbeats.status` `SQLite` column. Defined per spec §1d (post-M01
/// `docs(spec):` PR #36 closeout — see M01 gap-analysis "Important
/// `HeartbeatStatus` typed enum").
///
/// # Examples
///
/// ```
/// use runtime_core::HeartbeatStatus;
///
/// assert_eq!(HeartbeatStatus::Ok.to_string(), "ok");
/// let parsed: HeartbeatStatus = "degraded".parse().unwrap();
/// assert_eq!(parsed, HeartbeatStatus::Degraded);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeartbeatStatus {
    /// Drone is processing normally.
    Ok,
    /// Drone is making progress but degraded.
    Degraded,
    /// Drone has not made forward progress.
    Stalled,
}

impl fmt::Display for HeartbeatStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Ok => "ok",
            Self::Degraded => "degraded",
            Self::Stalled => "stalled",
        };
        f.write_str(s)
    }
}

/// Error returned when [`HeartbeatStatus::from_str`] sees an unknown value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseHeartbeatStatusError(String);

impl fmt::Display for ParseHeartbeatStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown heartbeat status: {}", self.0)
    }
}

impl std::error::Error for ParseHeartbeatStatusError {}

impl FromStr for HeartbeatStatus {
    type Err = ParseHeartbeatStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ok" => Ok(Self::Ok),
            "degraded" => Ok(Self::Degraded),
            "stalled" => Ok(Self::Stalled),
            other => Err(ParseHeartbeatStatusError(other.to_string())),
        }
    }
}

/// Events emitted by the drone process to main.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DroneEvent {
    /// Periodic heartbeat from the drone.
    Heartbeat {
        /// Current drone status.
        status: HeartbeatStatus,
        /// Unix timestamp (seconds since epoch).
        timestamp: i64,
    },
    /// A snapshot was written to the database.
    SnapshotWritten {
        /// Unique snapshot identifier.
        snapshot_id: String,
        /// The session this snapshot belongs to.
        session_id: String,
        /// Why the snapshot was taken.
        reason: String,
        /// Unix timestamp (seconds since epoch).
        timestamp: i64,
    },
    /// The drone's activity state changed.
    ActivityStateChange {
        /// Previous state.
        from: ActivityState,
        /// New state.
        to: ActivityState,
    },
    /// A child process was spawned.
    ProcessSpawned {
        /// OS process ID.
        pid: u32,
        /// What kind of process.
        process_type: ProcessType,
    },
    /// A child process stopped.
    ProcessStopped {
        /// OS process ID.
        pid: u32,
        /// Why it stopped.
        reason: StopReason,
    },
    /// A recoverable session snapshot is available.
    RecoveryAvailable {
        /// The session that can be recovered.
        session_id: String,
        /// The snapshot to recover from.
        snapshot_id: String,
    },
    /// An alert from the drone.
    Alert {
        /// Alert severity.
        level: AlertLevel,
        /// Human-readable alert message.
        message: String,
    },
    /// Result of a `QuerySessionDb` command — read-only SELECT against
    /// the session database. Each row is a JSON object keyed by column
    /// name. Spec §2b VDR projection / Stage E SQL inspector.
    QueryResult {
        /// Row data, in execution order. Empty for queries that match
        /// no rows.
        rows: Vec<serde_json::Value>,
    },
    /// Result of a `ReadSignals` command — full signal log for a
    /// session, ordered by timestamp. Spec §2b + Stage E replay.
    SignalLog {
        /// Each signal as a JSON object mirroring the `signals` table
        /// columns. Empty for sessions with no signals.
        signals: Vec<serde_json::Value>,
    },
}

/// Commands sent from main to the drone.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DroneCommand {
    /// Take a snapshot immediately.
    SnapshotNow {
        /// Why the snapshot is being requested.
        reason: String,
        /// Current state to snapshot.
        state_json: serde_json::Value,
    },
    /// Shut down gracefully within a timeout.
    GracefulShutdown {
        /// Maximum time to wait in milliseconds.
        timeout_ms: u64,
    },
    /// Spawn a child process.
    SpawnProcess {
        /// What kind of process to spawn.
        process_type: ProcessType,
        /// Process configuration.
        config: ProcessConfig,
    },
    /// Stop a running child process.
    StopProcess {
        /// OS process ID to stop.
        pid: u32,
        /// Whether to force-kill.
        force: bool,
    },
    /// Set the activity timeout.
    SetActivityTimeout {
        /// Timeout in milliseconds.
        ms: u64,
    },
    /// Revert to a prior snapshot.
    RevertToSnapshot {
        /// Snapshot to revert to.
        snapshot_id: String,
        /// Why the revert is happening.
        reason: RevertReason,
    },
    /// Read-only SQL query against the session database. Drone validates
    /// SELECT-only via parser-based check (per Stage E E.1 Decision #3 —
    /// regex-based ^SELECT is trivially bypassable). Reply is
    /// [`DroneEvent::QueryResult`].
    QuerySessionDb {
        /// SQL string. Must be a single SELECT statement; compound
        /// statements via semicolons + DDL/DML/PRAGMA are rejected by the
        /// drone with `DroneEvent::Alert(Critical)`.
        sql: String,
    },
    /// Read all signals for a session, ordered by timestamp. Spec §2b
    /// + Stage E replay path. Reply is [`DroneEvent::SignalLog`].
    ReadSignals {
        /// Session whose signals to return.
        session_id: String,
    },
    /// Write a signal to the `signals` table. Drone-side handler also
    /// runs the projectors: `vdr::project_signal` for decision/verify
    /// signals; `plan_projector::project_signal` for plan/task signals.
    /// Spec §2b. M04 Stage B (closes M03 🟡 carry-forward "vdr projector
    /// wired at signal-write call-site").
    WriteSignal {
        /// Caller-generated signal UUID.
        signal_id: String,
        /// Session this signal belongs to.
        session_id: String,
        /// Signal type tag (matches `Signal` discriminator: `tool` /
        /// `skill` / `agent` / `decision` / `verify` / `error` / `hitl`
        /// / `session`). Plan/task events are carried under the `agent`
        /// kind with the `AgentEvent` type embedded in `payload_json.type`.
        kind: String,
        /// Free-text event name (e.g., `plan_created`, `task_started`).
        /// Mirrors the schema's `AgentEvent` variant tag.
        event: String,
        /// Optional context tag (matches `ContextType` discriminator).
        context_type: String,
        /// Type-erased event payload — the JSON-encoded `AgentEvent`.
        payload: serde_json::Value,
    },
}

/// Activity states the drone tracks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActivityState {
    /// Actively processing.
    Active,
    /// Idle — no work in progress.
    Idle,
    /// Stalled — no progress detected.
    Stalled,
    /// Activity timeout exceeded.
    TimedOut,
    /// User aborted the session.
    UserAborted,
    /// Recovering from a crash or revert.
    Recovering,
}

/// Reasons a process stopped.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Process exited cleanly.
    Graceful,
    /// Process crashed.
    Crash,
    /// Process timed out.
    Timeout,
    /// User aborted.
    UserAbort,
    /// Force-killed by the drone.
    ForceKill,
}

/// Types of child processes the drone manages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessType {
    /// An agent process.
    Agent,
    /// An MCP server process.
    Mcp,
    /// A sandboxed skill execution process.
    SkillSandbox,
}

/// Alert severity levels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlertLevel {
    /// Warning — something is off but not critical.
    Warn,
    /// Critical — immediate attention required.
    Critical,
}

/// Reasons for reverting to a snapshot.
///
/// `HookRollback` carries the firing hook's id so the SDK / drone can emit
/// `task_failed { error: "rolled_back_after_hook_<hook_id>" }` per spec §4a.
/// `UserRollback` and `GapRecovery` stay unit-shape — neither needs a payload
/// at v0.1 (user-driven revert ships in M07; gap recovery in M05).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RevertReason {
    /// A verification hook triggered a rollback.
    HookRollback {
        /// Firing hook's `id` from the framework JSON. Drives the
        /// downstream `task_failed.error` string.
        hook_id: String,
    },
    /// The user requested a rollback.
    UserRollback,
    /// Gap recovery required reverting state.
    GapRecovery,
}

/// Configuration for spawning a child process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessConfig {
    /// Command to execute.
    pub command: String,
    /// Command-line arguments.
    pub args: Vec<String>,
    /// Environment variables.
    pub env: std::collections::HashMap<String, String>,
}

#[cfg(test)]
mod proptest_round_trip {
    use super::*;
    use proptest::prelude::*;

    fn arb_heartbeat_status() -> impl Strategy<Value = HeartbeatStatus> {
        prop_oneof![
            Just(HeartbeatStatus::Ok),
            Just(HeartbeatStatus::Degraded),
            Just(HeartbeatStatus::Stalled),
        ]
    }

    fn arb_heartbeat() -> impl Strategy<Value = DroneEvent> {
        (arb_heartbeat_status(), any::<i64>())
            .prop_map(|(status, timestamp)| DroneEvent::Heartbeat { status, timestamp })
    }

    fn arb_snapshot_now() -> impl Strategy<Value = DroneCommand> {
        any::<String>().prop_map(|reason| DroneCommand::SnapshotNow {
            reason,
            state_json: serde_json::json!({}),
        })
    }

    fn arb_graceful_shutdown() -> impl Strategy<Value = DroneCommand> {
        any::<u64>().prop_map(|timeout_ms| DroneCommand::GracefulShutdown { timeout_ms })
    }

    proptest! {
        #[test]
        fn heartbeat_round_trips(event in arb_heartbeat()) {
            let json: serde_json::Value = serde_json::to_value(&event).unwrap();
            let back: DroneEvent = serde_json::from_value(json).unwrap();
            prop_assert_eq!(event, back);
        }

        #[test]
        fn snapshot_now_round_trips(cmd in arb_snapshot_now()) {
            let json: serde_json::Value = serde_json::to_value(&cmd).unwrap();
            let back: DroneCommand = serde_json::from_value(json).unwrap();
            prop_assert_eq!(cmd, back);
        }

        #[test]
        fn graceful_shutdown_round_trips(cmd in arb_graceful_shutdown()) {
            let json: serde_json::Value = serde_json::to_value(&cmd).unwrap();
            let back: DroneCommand = serde_json::from_value(json).unwrap();
            prop_assert_eq!(cmd, back);
        }

        // Newline-delimited JSON codec round trip — the wire format used by
        // the main↔drone IPC channel (`tokio_util::codec::LinesCodec`).
        #[test]
        fn drone_event_codec_round_trip_proptest(event in arb_heartbeat()) {
            let line = serde_json::to_string(&event).unwrap();
            prop_assert!(!line.contains('\n'), "encoded line must not contain a newline");
            let back: DroneEvent = serde_json::from_str(&line).unwrap();
            prop_assert_eq!(event, back);
        }

        #[test]
        fn drone_command_codec_round_trip_proptest(cmd in arb_graceful_shutdown()) {
            let line = serde_json::to_string(&cmd).unwrap();
            prop_assert!(!line.contains('\n'));
            let back: DroneCommand = serde_json::from_str(&line).unwrap();
            prop_assert_eq!(cmd, back);
        }
    }

    #[test]
    fn heartbeat_status_display_round_trip() {
        for s in [
            HeartbeatStatus::Ok,
            HeartbeatStatus::Degraded,
            HeartbeatStatus::Stalled,
        ] {
            let displayed = s.to_string();
            let parsed: HeartbeatStatus = displayed.parse().expect("parse");
            assert_eq!(parsed, s);
        }
    }

    #[test]
    fn heartbeat_status_serializes_as_snake_case() {
        let json = serde_json::to_string(&HeartbeatStatus::Degraded).unwrap();
        assert_eq!(json, "\"degraded\"");
    }

    #[test]
    fn heartbeat_status_parse_rejects_unknown() {
        let err = "boom".parse::<HeartbeatStatus>().unwrap_err();
        assert!(err.to_string().contains("boom"));
    }
}
