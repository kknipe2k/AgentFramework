//! End-to-end hook lifecycle test — spec §4a (M04 Stage D).
//!
//! Pure-Rust integration test: no real subprocess, no real IPC. Wires
//! the hook executor against a fake [`ShellSpawner`] that returns a
//! canned outcome, then asserts the executor's [`HookOutcome`]
//! correctly carries the spec §4a fields the SDK projects into
//! `verify_*` events. The drone-side `RevertToSnapshot { reason:
//! HookRollback { hook_id } }` dispatch is exercised by inspecting the
//! `on_failure` field on a Failed outcome — the actual IPC roundtrip
//! is covered by `runtime-drone`'s `command_handler` tests.

use runtime_core::generated::common::{
    Hook, HookCategory, HookId, HookOnFailure, HookRef, HookRefCommand,
};
use runtime_main::hooks::shell::{ShellError, ShellOutput, ShellSpawner, SpawnArgs};
use runtime_main::hooks::{execute_hook, HookContext, HookDeps, HookOutcome};
use std::sync::Mutex;

/// A spawner that returns a configurable canned outcome.
struct CannedSpawner {
    outcome: Mutex<Result<ShellOutput, ShellError>>,
}

impl CannedSpawner {
    fn ok(stdout: &str) -> Self {
        Self {
            outcome: Mutex::new(Ok(ShellOutput {
                stdout: stdout.to_string(),
                stderr: String::new(),
                status: 0,
            })),
        }
    }
    fn fail(stderr: &str) -> Self {
        Self {
            outcome: Mutex::new(Err(ShellError::NonZeroExit {
                status: 1,
                stderr: stderr.to_string(),
            })),
        }
    }
}

#[async_trait::async_trait]
impl ShellSpawner for CannedSpawner {
    async fn spawn(&self, _args: SpawnArgs) -> Result<ShellOutput, ShellError> {
        let guard = self.outcome.lock().unwrap();
        match &*guard {
            Ok(o) => Ok(o.clone()),
            Err(ShellError::NonZeroExit { status, stderr }) => Err(ShellError::NonZeroExit {
                status: *status,
                stderr: stderr.clone(),
            }),
            Err(ShellError::Timeout(t)) => Err(ShellError::Timeout(*t)),
            Err(ShellError::Spawn(e)) => Err(ShellError::Spawn(e.clone())),
        }
    }
}

fn verify_hook(id: &str, command: &str, on_failure: HookOnFailure) -> Hook {
    Hook {
        id: HookId::try_from(id.to_string()).unwrap(),
        category: HookCategory::Verify,
        level: None,
        ref_: HookRef::Shell {
            command: HookRefCommand::try_from(command.to_string()).unwrap(),
            cwd: None,
            timeout_ms: Some(300_000),
        },
        on_failure,
    }
}

fn ctx(firing_point: &str) -> HookContext {
    HookContext {
        firing_point: firing_point.to_string(),
        session_id: "session-1".to_string(),
        agent_id: Some("agent-1".to_string()),
    }
}

#[tokio::test]
async fn post_task_hook_passes_emits_passed_outcome() {
    // ARIA verify.sh exits 0 → hook passes; SDK emits `verify_passed`.
    let spawner = CannedSpawner::ok("All checks passed");
    let deps = HookDeps {
        shell_spawner: &spawner,
    };
    let hook = verify_hook("verify", "bash .aria/verify.sh", HookOnFailure::Rollback);

    let outcome = execute_hook(&hook, &ctx("post_task"), &deps)
        .await
        .expect("executor ok");

    match outcome {
        HookOutcome::Passed {
            hook_id,
            output_preview,
            ..
        } => {
            assert_eq!(hook_id, "verify");
            assert_eq!(output_preview.as_deref(), Some("All checks passed"));
        }
        other @ HookOutcome::Failed { .. } => panic!("expected Passed, got {other:?}"),
    }
}

#[tokio::test]
async fn post_task_hook_fails_with_rollback_flags_outcome_for_drone_revert() {
    // Spec §4a + MVP §M4 acceptance: verify.sh fails + on_failure=rollback
    // → executor returns Failed { on_failure: Rollback }; the SDK then
    // dispatches DroneCommand::RevertToSnapshot { reason: HookRollback }.
    let spawner = CannedSpawner::fail("Test failure: 1 of 47");
    let deps = HookDeps {
        shell_spawner: &spawner,
    };
    let hook = verify_hook("verify", "bash .aria/verify.sh", HookOnFailure::Rollback);

    let outcome = execute_hook(&hook, &ctx("post_task"), &deps)
        .await
        .expect("executor ok");

    match outcome {
        HookOutcome::Failed {
            hook_id,
            on_failure,
            error,
            ..
        } => {
            assert_eq!(hook_id, "verify");
            assert!(matches!(on_failure, HookOnFailure::Rollback));
            assert!(error.contains("Test failure"));
            // Locks the contract: the SDK uses `hook_id` to construct
            // RevertReason::HookRollback { hook_id: "verify" } when
            // dispatching to the drone.
        }
        other @ HookOutcome::Passed { .. } => panic!("expected Failed Rollback, got {other:?}"),
    }
}

#[tokio::test]
async fn post_file_edit_lint_hook_with_warn_does_not_drive_rollback() {
    // Lint hook fails + on_failure=warn → SDK emits verify_failed but
    // does NOT dispatch a drone rollback.
    let spawner = CannedSpawner::fail("123 lint warnings");
    let deps = HookDeps {
        shell_spawner: &spawner,
    };
    let hook = Hook {
        id: HookId::try_from("lint".to_string()).unwrap(),
        category: HookCategory::Lint,
        level: None,
        ref_: HookRef::Shell {
            command: HookRefCommand::try_from("npm run lint".to_string()).unwrap(),
            cwd: None,
            timeout_ms: None,
        },
        on_failure: HookOnFailure::Warn,
    };

    let outcome = execute_hook(&hook, &ctx("post_file_edit"), &deps)
        .await
        .expect("executor ok");

    match outcome {
        HookOutcome::Failed { on_failure, .. } => {
            assert!(matches!(on_failure, HookOnFailure::Warn));
        }
        other @ HookOutcome::Passed { .. } => panic!("expected Failed Warn, got {other:?}"),
    }
}

#[tokio::test]
async fn revert_reason_hook_rollback_carries_hook_id() {
    // Locks the drone IPC contract: RevertReason::HookRollback now
    // carries hook_id (M04.D struct-variant change). Round-trip via
    // serde to confirm wire format.
    let reason = runtime_core::RevertReason::HookRollback {
        hook_id: "verify".to_string(),
    };
    let json = serde_json::to_value(&reason).expect("serialize");
    assert_eq!(json["kind"], "hook_rollback");
    assert_eq!(json["hook_id"], "verify");
    let parsed: runtime_core::RevertReason = serde_json::from_value(json).expect("deserialize");
    assert_eq!(reason, parsed);
}

#[tokio::test]
async fn revert_reason_user_rollback_stays_unit_shape() {
    let reason = runtime_core::RevertReason::UserRollback;
    let json = serde_json::to_value(&reason).expect("serialize");
    assert_eq!(json["kind"], "user_rollback");
    let parsed: runtime_core::RevertReason = serde_json::from_value(json).expect("deserialize");
    assert_eq!(reason, parsed);
}

#[tokio::test]
async fn revert_reason_gap_recovery_stays_unit_shape() {
    let reason = runtime_core::RevertReason::GapRecovery;
    let json = serde_json::to_value(&reason).expect("serialize");
    assert_eq!(json["kind"], "gap_recovery");
}
