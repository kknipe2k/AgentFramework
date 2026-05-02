//! Drone IPC types — `DroneEvent` / `DroneCommand` and supporting enums.
//!
//! Specified in `agent-runtime-spec.md` §1d. These types are the wire format
//! for main↔drone IPC over Unix domain socket / Windows named pipe.

use serde::{Deserialize, Serialize};

/// Events emitted by the drone process to main.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DroneEvent {
    /// Periodic heartbeat from the drone.
    Heartbeat {
        /// Current drone status.
        status: String,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RevertReason {
    /// A verification hook triggered a rollback.
    HookRollback,
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

    fn arb_heartbeat() -> impl Strategy<Value = DroneEvent> {
        (any::<String>(), any::<i64>())
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
}
