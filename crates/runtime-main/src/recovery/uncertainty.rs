//! Tool-call uncertainty resolution — spec §1b.
//!
//! On resume, the drone projects the set of `tool_invoked` signals
//! lacking a matching `tool_result` (see
//! `runtime_drone::snapshot::recover_session_state`). The renderer
//! surfaces a prompt per invocation; the user picks one of four
//! actions. This module records the choice as a
//! `tool_call_uncertainty_resolved` decision signal so the VDR
//! projection carries the audit trail.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::drone_ipc::DroneIpcError;

/// The four spec §1b actions a user can pick for an uncertain tool
/// invocation. Each maps to a distinct `tool_call_uncertainty_resolved`
/// decision payload so VDR replay can attribute the choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallUncertaintyAction {
    /// `[r]etry the call` — re-invoke the tool from scratch. Caller is
    /// expected to enqueue a fresh `tool_invoked` on resume.
    Retry,
    /// `[s]kip` — treat the call as if it returned nothing; agent
    /// continues with that gap.
    Skip,
    /// `[m]ark complete` — assume the call completed successfully (no
    /// output recorded).
    MarkComplete,
    /// `[a]bort the session` — cancel the resume.
    Abort,
}

impl ToolCallUncertaintyAction {
    /// Stable string token for VDR signal payloads.
    #[must_use]
    pub const fn as_token(&self) -> &'static str {
        match self {
            Self::Retry => "retry",
            Self::Skip => "skip",
            Self::MarkComplete => "mark_complete",
            Self::Abort => "abort",
        }
    }

    /// Parse the renderer's token string. Returns `None` for unknown
    /// values so callers can surface a friendly error rather than panic.
    #[must_use]
    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "retry" => Some(Self::Retry),
            "skip" => Some(Self::Skip),
            "mark_complete" => Some(Self::MarkComplete),
            "abort" => Some(Self::Abort),
            _ => None,
        }
    }
}

/// Returned by [`respond_uncertainty_with`] on success — carries the
/// emitted signal id for the audit trail. `Serialize` + `Deserialize`
/// so the Tauri command surface can return it to the renderer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UncertaintyResolution {
    /// UUID of the emitted `tool_call_uncertainty_resolved` decision signal.
    pub signal_id: String,
    /// Action recorded.
    pub action: ToolCallUncertaintyAction,
    /// Invocation id the action applies to (the original `tool_invoked`
    /// signal id).
    pub invocation_id: String,
}

/// Errors raised by [`respond_uncertainty_with`].
#[derive(Debug, Error)]
pub enum UncertaintyError {
    /// Action token didn't match any of the 4 spec §1b actions.
    #[error("unknown uncertainty action token: {0}")]
    UnknownAction(String),
    /// Drone IPC failed during the `WriteSignal` round-trip.
    #[error(transparent)]
    Drone(#[from] DroneIpcError),
}

/// Record the user's resolution for one uncertain tool invocation.
///
/// The `emit` callback writes the signal to the drone (`WriteSignal`);
/// production callers wrap `DroneClient::write_signal`, tests inject a
/// deterministic future.
///
/// Returns an [`UncertaintyResolution`] describing what was recorded.
///
/// # Errors
///
/// - [`UncertaintyError::UnknownAction`] if `action_token` is not one of
///   `retry / skip / mark_complete / abort`.
/// - [`UncertaintyError::Drone`] if the signal write fails.
pub async fn respond_uncertainty_with<F, Fut>(
    session_id: String,
    invocation_id: String,
    action_token: String,
    agent_id: Option<String>,
    emit: F,
) -> Result<UncertaintyResolution, UncertaintyError>
where
    F: FnOnce(WriteSignalArgs) -> Fut,
    Fut: std::future::Future<Output = Result<(), DroneIpcError>>,
{
    let action = ToolCallUncertaintyAction::from_token(&action_token)
        .ok_or_else(|| UncertaintyError::UnknownAction(action_token.clone()))?;
    let signal_id = uuid::Uuid::new_v4().to_string();
    let payload = serde_json::json!({
        "type": "tool_call_uncertainty_resolved",
        "agent_id": agent_id.clone().unwrap_or_default(),
        "decision": format!("uncertainty_resolved:{}", action.as_token()),
        "rationale": format!("user picked {} for invocation {invocation_id}", action.as_token()),
        "tool_used": serde_json::Value::Null,
        "uncertainty_action": action.as_token(),
        "uncertain_invocation_id": invocation_id,
    });
    emit(WriteSignalArgs {
        signal_id: signal_id.clone(),
        session_id,
        kind: "decision".to_string(),
        event: "tool_call_uncertainty_resolved".to_string(),
        context_type: "tool_invoke".to_string(),
        payload,
    })
    .await?;
    Ok(UncertaintyResolution {
        signal_id,
        action,
        invocation_id,
    })
}

/// Argument bundle for the `emit` callback. Mirrors
/// `DroneClient::write_signal`'s parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteSignalArgs {
    /// Caller-generated signal UUID.
    pub signal_id: String,
    /// Session this signal belongs to.
    pub session_id: String,
    /// Signal kind (always `"decision"` for uncertainty resolution).
    pub kind: String,
    /// Event name (`"tool_call_uncertainty_resolved"`).
    pub event: String,
    /// Context type (`"tool_invoke"` per signal-schema §2b).
    pub context_type: String,
    /// Type-erased payload — see the JSON shape in
    /// [`respond_uncertainty_with`].
    pub payload: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn action_token_round_trip_for_all_four() {
        for a in [
            ToolCallUncertaintyAction::Retry,
            ToolCallUncertaintyAction::Skip,
            ToolCallUncertaintyAction::MarkComplete,
            ToolCallUncertaintyAction::Abort,
        ] {
            let token = a.as_token();
            assert_eq!(ToolCallUncertaintyAction::from_token(token), Some(a));
        }
    }

    #[test]
    fn from_token_rejects_unknown() {
        assert_eq!(ToolCallUncertaintyAction::from_token("nope"), None);
        assert_eq!(ToolCallUncertaintyAction::from_token(""), None);
    }

    #[tokio::test]
    async fn respond_uncertainty_records_signal_with_action_in_payload() {
        let captured: std::sync::Arc<Mutex<Option<WriteSignalArgs>>> =
            std::sync::Arc::new(Mutex::new(None));
        let captured_clone = std::sync::Arc::clone(&captured);
        let resolution = respond_uncertainty_with(
            "s1".to_string(),
            "sig-tool-1".to_string(),
            "skip".to_string(),
            Some("a1".to_string()),
            |args| async move {
                *captured_clone.lock().unwrap() = Some(args);
                Ok(())
            },
        )
        .await
        .expect("respond");
        assert_eq!(resolution.action, ToolCallUncertaintyAction::Skip);
        assert_eq!(resolution.invocation_id, "sig-tool-1");

        let written = captured.lock().unwrap().clone().expect("emit called");
        assert_eq!(written.session_id, "s1");
        assert_eq!(written.event, "tool_call_uncertainty_resolved");
        assert_eq!(written.kind, "decision");
        assert_eq!(written.context_type, "tool_invoke");
        assert_eq!(written.payload["uncertainty_action"], "skip");
        assert_eq!(written.payload["uncertain_invocation_id"], "sig-tool-1");
        assert_eq!(written.payload["agent_id"], "a1");
        assert!(written.payload["decision"]
            .as_str()
            .unwrap()
            .ends_with(":skip"));
    }

    #[tokio::test]
    async fn respond_uncertainty_emits_distinct_signal_for_each_action() {
        // Each of the 4 actions produces a distinct payload — the VDR
        // projection then carries the audit trail.
        for (token, expected) in [
            ("retry", ToolCallUncertaintyAction::Retry),
            ("skip", ToolCallUncertaintyAction::Skip),
            ("mark_complete", ToolCallUncertaintyAction::MarkComplete),
            ("abort", ToolCallUncertaintyAction::Abort),
        ] {
            let resolution = respond_uncertainty_with(
                "s1".to_string(),
                "sig-a".to_string(),
                token.to_string(),
                None,
                |_args| async move { Ok(()) },
            )
            .await
            .expect("respond");
            assert_eq!(resolution.action, expected);
        }
    }

    #[tokio::test]
    async fn respond_uncertainty_rejects_unknown_action() {
        let result = respond_uncertainty_with(
            "s1".to_string(),
            "sig-a".to_string(),
            "bogus".to_string(),
            None,
            |_args| async move { Ok(()) },
        )
        .await;
        match result {
            Err(UncertaintyError::UnknownAction(token)) => assert_eq!(token, "bogus"),
            other => panic!("expected UnknownAction, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn respond_uncertainty_propagates_drone_error() {
        let result = respond_uncertainty_with(
            "s1".to_string(),
            "sig-a".to_string(),
            "skip".to_string(),
            None,
            |_args| async move { Err(DroneIpcError::Codec("boom".to_string())) },
        )
        .await;
        assert!(matches!(result, Err(UncertaintyError::Drone(_))));
    }

    #[tokio::test]
    async fn respond_uncertainty_handles_missing_agent_id() {
        let captured: std::sync::Arc<Mutex<Option<WriteSignalArgs>>> =
            std::sync::Arc::new(Mutex::new(None));
        let captured_clone = std::sync::Arc::clone(&captured);
        let _ = respond_uncertainty_with(
            "s1".to_string(),
            "sig-x".to_string(),
            "mark_complete".to_string(),
            None,
            |args| async move {
                *captured_clone.lock().unwrap() = Some(args);
                Ok(())
            },
        )
        .await
        .expect("respond");
        let written = captured.lock().unwrap().clone().expect("emit called");
        assert_eq!(written.payload["agent_id"], "");
    }

    #[test]
    fn action_token_strings_are_stable() {
        // Lock the wire-format tokens — the renderer encodes these and
        // VDR replay attributes choices by them.
        assert_eq!(ToolCallUncertaintyAction::Retry.as_token(), "retry");
        assert_eq!(ToolCallUncertaintyAction::Skip.as_token(), "skip");
        assert_eq!(
            ToolCallUncertaintyAction::MarkComplete.as_token(),
            "mark_complete"
        );
        assert_eq!(ToolCallUncertaintyAction::Abort.as_token(), "abort");
    }
}
