//! Resume coordinator — spec §1b.
//!
//! Calls the drone's `RecoverSession` IPC command, projects the reply
//! into a [`ResumePlan`] the SDK consumes to rebuild message history.
//!
//! Hard invariant per spec §1b + gotcha #15: tools in the snapshot's
//! signal log are NOT re-invoked.
//!
//! `ResumePlan::sdk_messages` reconstructs the message history as if
//! the tools had already completed; the model starts the next turn
//! fresh with the prior context. The `uncertain_tool_invocations`
//! field carries the signal ids that lack a matching `tool_result` —
//! the renderer prompts the user (retry / skip / mark complete / abort)
//! for each.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::drone_ipc::{DroneIpcError, RecoveredSession};

/// What the SDK needs to resume a session. Constructed by
/// [`request_resume_with`] from the drone's `RecoverSession` reply.
///
/// `Serialize` + `Deserialize` so the Tauri command surface can return
/// the plan directly to the renderer.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ResumePlan {
    /// Snapshot id the state was loaded from. `None` when the session
    /// has no snapshots (fresh start — no resume needed).
    pub snapshot_id: Option<String>,
    /// Plan rows projected from signals; statuses normalized per spec §1b.
    pub plans: Vec<serde_json::Value>,
    /// Task rows; running tasks downgraded to `pending`. The renderer's
    /// graph reconstructs from this list.
    pub tasks: Vec<serde_json::Value>,
    /// Uncertain tool-invocation signal ids — `tool_invoked` without
    /// matching `tool_result`. Renderer surfaces a 4-action prompt per
    /// invocation (retry / skip / mark complete / abort) via
    /// [`crate::recovery::uncertainty`].
    pub uncertain_tool_invocations: Vec<String>,
    /// `true` when the session has prior signals to resume from. `false`
    /// means there's nothing to resume — the caller should start a fresh
    /// session instead.
    pub has_state: bool,
}

/// Resume errors. Mostly transport-level (drone IPC); the spec's
/// recoverable-but-degraded path (MCP server failed to reconnect) is
/// represented inside [`ResumePlan`], not as an error here.
#[derive(Debug, Error)]
pub enum ResumeError {
    /// Drone IPC failed during the `RecoverSession` round-trip.
    #[error(transparent)]
    Drone(#[from] DroneIpcError),
}

/// Coordinate a resume against the supplied async `recover` callback.
///
/// Production wraps `DroneClient::recover_session` directly via
/// `Arc::clone(&drone)` + a closure; tests inject a deterministic future
/// returning a synthetic [`RecoveredSession`].
///
/// # Errors
///
/// - [`ResumeError::Drone`] if the IPC fails after retry exhaustion.
pub async fn request_resume_with<F, Fut>(
    session_id: String,
    recover: F,
) -> Result<ResumePlan, ResumeError>
where
    F: FnOnce(String) -> Fut,
    Fut: std::future::Future<Output = Result<RecoveredSession, DroneIpcError>>,
{
    tracing::info!(session_id, "request_resume invoked");
    let recovered = recover(session_id.clone()).await?;
    let has_state = recovered.snapshot_id.is_some()
        || !recovered.plans.is_empty()
        || !recovered.tasks.is_empty()
        || !recovered.uncertain_tool_invocations.is_empty();
    let plan = ResumePlan {
        snapshot_id: recovered.snapshot_id,
        plans: recovered.plans,
        tasks: recovered.tasks,
        uncertain_tool_invocations: recovered.uncertain_tool_invocations,
        has_state,
    };
    tracing::info!(
        plans = plan.plans.len(),
        tasks = plan.tasks.len(),
        uncertain = plan.uncertain_tool_invocations.len(),
        has_state = plan.has_state,
        "request_resume produced ResumePlan"
    );
    Ok(plan)
}

/// MCP reconnect seam — v0.1 STANDARD-mode no-op.
///
/// Spec §1b requires MCP servers to be reconnected on resume; v0.1
/// ships no MCP and so this returns `Ok(())` immediately. M5/M6 wire
/// the real reconnect path without changes to call sites.
///
/// # Errors
///
/// Future implementations will surface MCP transport errors. v0.1 never
/// errors.
#[allow(
    clippy::unused_async,
    reason = "seam preserves async signature for M5/M6"
)]
pub async fn reconnect_mcp_servers(_session_id: &str) -> Result<(), ResumeError> {
    tracing::debug!("reconnect_mcp_servers: v0.1 no-op (no MCP configured)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn recovered_with(uncertain: Vec<String>, plans: usize, tasks: usize) -> RecoveredSession {
        RecoveredSession {
            snapshot_id: if plans + tasks > 0 || !uncertain.is_empty() {
                Some("snap-1".to_string())
            } else {
                None
            },
            state: json!({}),
            plans: (0..plans).map(|i| json!({"id": format!("p{i}")})).collect(),
            tasks: (0..tasks).map(|i| json!({"id": format!("t{i}")})).collect(),
            uncertain_tool_invocations: uncertain,
        }
    }

    #[tokio::test]
    async fn resume_returns_plan_with_snapshot_id_when_present() {
        let recovered = recovered_with(Vec::new(), 1, 2);
        let plan = request_resume_with("s1".to_string(), |id| {
            assert_eq!(id, "s1");
            async move { Ok(recovered) }
        })
        .await
        .expect("resume");
        assert!(plan.has_state);
        assert_eq!(plan.plans.len(), 1);
        assert_eq!(plan.tasks.len(), 2);
        assert_eq!(plan.snapshot_id.as_deref(), Some("snap-1"));
    }

    #[tokio::test]
    async fn resume_with_no_snapshot_returns_has_state_false() {
        let recovered = RecoveredSession::default();
        let plan = request_resume_with("s1".to_string(), |_| async move { Ok(recovered) })
            .await
            .expect("resume");
        assert!(!plan.has_state);
        assert!(plan.snapshot_id.is_none());
    }

    #[tokio::test]
    async fn resume_surfaces_uncertain_invocations() {
        let recovered = recovered_with(vec!["sig-1".to_string(), "sig-2".to_string()], 0, 0);
        let plan = request_resume_with("s1".to_string(), |_| async move { Ok(recovered) })
            .await
            .expect("resume");
        assert!(plan.has_state);
        assert_eq!(plan.uncertain_tool_invocations.len(), 2);
    }

    #[tokio::test]
    async fn resume_propagates_drone_error() {
        let result = request_resume_with("s1".to_string(), |_| async move {
            Err(DroneIpcError::Codec("boom".to_string()))
        })
        .await;
        assert!(matches!(result, Err(ResumeError::Drone(_))));
    }

    #[tokio::test]
    async fn reconnect_mcp_no_op_at_v01() {
        // Always Ok until M5/M6 wires the real reconnect.
        reconnect_mcp_servers("s1").await.expect("no-op succeeds");
    }

    #[tokio::test]
    async fn has_state_true_when_uncertain_present_even_without_snapshot() {
        // Edge: no snapshot row, no plans/tasks, but a stranded
        // tool_invoked signal — still resume-eligible because the user
        // must resolve the uncertainty.
        let recovered = RecoveredSession {
            uncertain_tool_invocations: vec!["sig-1".to_string()],
            ..RecoveredSession::default()
        };
        let plan = request_resume_with("s1".to_string(), |_| async move { Ok(recovered) })
            .await
            .expect("resume");
        assert!(plan.has_state);
    }
}
