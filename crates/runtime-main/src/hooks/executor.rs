//! Hook executor — runs a [`Hook`] and surfaces a [`HookOutcome`].
//!
//! Spec §4a.
//!
//! ## Entry point
//!
//! The executor is the single entry point for hook execution. Callers
//! pass a [`Hook`] (from framework JSON), a [`HookContext`] (firing
//! point and agent/session ids), and an injected [`HookDeps`] bag of
//! execution dependencies.
//!
//! The executor translates [`HookRef`] variants to the appropriate
//! dependency (shell → spawner; tool/agent → deferred per integration
//! scope), times the run with [`std::time::Instant`], and returns a
//! [`HookOutcome::Passed`] or [`HookOutcome::Failed`] with the spec
//! §4a fields the SDK projects into `verify_*` events.
//!
//! ## Tool / Agent variants — deferred
//!
//! v0.1's runtime-main has no tool dispatcher (M05+ capability enforcer
//! lands the central dispatch surface) and no child-agent spawn
//! mechanism (M07 plan loop). The executor surfaces both as
//! [`HookError::DeferredVariant`]; the caller decides whether to surface
//! the deferral to the user or treat it as `verify_failed`.

use runtime_core::generated::common::{Hook, HookOnFailure, HookRef};
use std::time::Instant;
use thiserror::Error;

use super::shell::{self, ShellError, ShellOutput, ShellSpawner};

/// Captured-at-fire context for a hook execution.
#[derive(Debug, Clone)]
pub struct HookContext {
    /// Lifecycle moment that fired this hook (one of the 7 firing
    /// points in `framework.v1.json` `hooks`).
    pub firing_point: String,
    /// Owning session id.
    pub session_id: String,
    /// Optional agent id when the firing point is agent-scoped (e.g.,
    /// `pre_file_edit` carries the agent that requested the write).
    pub agent_id: Option<String>,
}

/// Outcome of a single hook execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookOutcome {
    /// Hook completed successfully. Surfaces as `verify_passed` event.
    Passed {
        /// Hook id (echoed for emit convenience).
        hook_id: String,
        /// Wall-clock duration of the execution in milliseconds.
        duration_ms: u64,
        /// Optional preview of the hook's stdout (truncated by
        /// [`OUTPUT_PREVIEW_MAX_BYTES`]).
        output_preview: Option<String>,
    },
    /// Hook failed. Surfaces as `verify_failed` event. Carries the
    /// `on_failure` policy so the caller knows whether to block, warn,
    /// or rollback.
    Failed {
        /// Hook id.
        hook_id: String,
        /// Wall-clock duration in milliseconds.
        duration_ms: u64,
        /// User-facing error string (from spawner's stderr / non-zero
        /// exit / timeout / spawn-failure).
        error: String,
        /// `on_failure` policy from framework JSON: `block | warn |
        /// rollback`. Drives the caller's downstream action.
        on_failure: HookOnFailure,
    },
}

/// Truncation cap for `output_preview` in `verify_passed` events. Keeps
/// the event payload bounded; the full output remains in the framework's
/// audit log (when capability enforcer M05 lands).
pub const OUTPUT_PREVIEW_MAX_BYTES: usize = 512;

/// Dependencies the executor needs to run. Stage D ships shell only;
/// tool / agent are wired by M05 / M07.
pub struct HookDeps<'a> {
    /// Cross-platform shell spawner. Pass [`shell::TokioShellSpawner`]
    /// in production; a mock for tests.
    pub shell_spawner: &'a dyn ShellSpawner,
}

/// Errors raised by the executor that aren't user-facing hook failures.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum HookError {
    /// Hook references a variant the runtime can't yet execute.
    #[error("hook ref variant deferred: {0}")]
    DeferredVariant(String),
}

/// Execute a single hook against the given context + deps.
///
/// Always returns a [`HookOutcome`] for shell hooks (success or failure
/// is surfaced via [`HookOutcome::Failed`], not via `Result::Err`).
/// Returns [`HookError::DeferredVariant`] only for tool / agent
/// variants until M05 / M07 wires those dispatch paths.
///
/// # Errors
///
/// See [`HookError`].
pub async fn execute_hook(
    hook: &Hook,
    ctx: &HookContext,
    deps: &HookDeps<'_>,
) -> Result<HookOutcome, HookError> {
    let _ = ctx; // ctx is currently informational; M05 wires it into facts.
    let hook_id: String = hook.id.clone().into();
    let started = Instant::now();
    match &hook.ref_ {
        HookRef::Shell {
            command,
            cwd,
            timeout_ms,
        } => {
            let cwd_owned: Option<std::path::PathBuf> = cwd.as_ref().map(std::path::PathBuf::from);
            let timeout_u: Option<u64> =
                (*timeout_ms).map(|t| u64::try_from(t).unwrap_or(u64::MAX));
            let cmd_str: String = command.clone().into();
            let result = shell::execute_shell_with(
                deps.shell_spawner,
                &cmd_str,
                timeout_u,
                cwd_owned.as_deref(),
            )
            .await;
            let elapsed = duration_ms(started);
            Ok(map_shell_result(hook_id, hook.on_failure, elapsed, result))
        }
        HookRef::Tool { tool_name, .. } => {
            // M05 capability enforcer wires the tool dispatcher; until
            // then the executor surfaces the deferral to the caller.
            let name: String = tool_name.clone().into();
            Err(HookError::DeferredVariant(format!("tool:{name}")))
        }
        HookRef::Agent { agent_id, .. } => {
            // M07 plan loop wires child-agent spawn.
            let id: String = agent_id.clone().into();
            Err(HookError::DeferredVariant(format!("agent:{id}")))
        }
    }
}

fn map_shell_result(
    hook_id: String,
    on_failure: HookOnFailure,
    duration_ms: u64,
    result: Result<ShellOutput, ShellError>,
) -> HookOutcome {
    match result {
        Ok(out) => HookOutcome::Passed {
            hook_id,
            duration_ms,
            output_preview: preview(&out.stdout),
        },
        Err(e) => HookOutcome::Failed {
            hook_id,
            duration_ms,
            error: e.to_string(),
            on_failure,
        },
    }
}

fn duration_ms(start: Instant) -> u64 {
    let elapsed = start.elapsed();
    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
}

fn preview(stdout: &str) -> Option<String> {
    if stdout.is_empty() {
        return None;
    }
    let trimmed: String = if stdout.len() <= OUTPUT_PREVIEW_MAX_BYTES {
        stdout.to_string()
    } else {
        // Walk to a UTF-8 boundary at-or-below the cap.
        let mut end = OUTPUT_PREVIEW_MAX_BYTES;
        while !stdout.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        format!("{}…", &stdout[..end])
    };
    Some(trimmed)
}

#[cfg(test)]
mod tests {
    use super::super::shell::{ShellError, ShellOutput, ShellSpawner, SpawnArgs};
    use super::*;
    use runtime_core::generated::common::{HookCategory, HookId, HookRefCommand};
    use std::sync::Mutex;

    struct FakeSpawner {
        outcome: Mutex<Result<ShellOutput, ShellError>>,
    }

    impl FakeSpawner {
        fn ok(stdout: &str) -> Self {
            Self {
                outcome: Mutex::new(Ok(ShellOutput {
                    stdout: stdout.to_string(),
                    stderr: String::new(),
                    status: 0,
                })),
            }
        }
        fn fail(status: i32, stderr: &str) -> Self {
            Self {
                outcome: Mutex::new(Err(ShellError::NonZeroExit {
                    status,
                    stderr: stderr.to_string(),
                })),
            }
        }
    }

    #[async_trait::async_trait]
    impl ShellSpawner for FakeSpawner {
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

    fn shell_hook(id: &str, command: &str, on_failure: HookOnFailure) -> Hook {
        Hook {
            id: HookId::try_from(id.to_string()).expect("ok"),
            category: HookCategory::Verify,
            level: None,
            ref_: HookRef::Shell {
                command: HookRefCommand::try_from(command.to_string()).expect("ok"),
                cwd: None,
                timeout_ms: None,
            },
            on_failure,
        }
    }

    fn ctx() -> HookContext {
        HookContext {
            firing_point: "post_task".to_string(),
            session_id: "s1".to_string(),
            agent_id: None,
        }
    }

    #[tokio::test]
    async fn shell_hook_pass_returns_passed_outcome() {
        let spawner = FakeSpawner::ok("hello\n");
        let deps = HookDeps {
            shell_spawner: &spawner,
        };
        let hook = shell_hook("verify", "echo hi", HookOnFailure::Block);
        let outcome = execute_hook(&hook, &ctx(), &deps).await.expect("ok");
        match outcome {
            HookOutcome::Passed {
                hook_id,
                output_preview,
                ..
            } => {
                assert_eq!(hook_id, "verify");
                assert_eq!(output_preview.as_deref(), Some("hello\n"));
            }
            other @ HookOutcome::Failed { .. } => panic!("expected Passed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn shell_hook_fail_with_block_returns_failed_block() {
        let spawner = FakeSpawner::fail(1, "boom");
        let deps = HookDeps {
            shell_spawner: &spawner,
        };
        let hook = shell_hook("verify", "false", HookOnFailure::Block);
        let outcome = execute_hook(&hook, &ctx(), &deps).await.expect("ok");
        match outcome {
            HookOutcome::Failed {
                on_failure, error, ..
            } => {
                assert!(matches!(on_failure, HookOnFailure::Block));
                assert!(error.contains("boom"));
            }
            other @ HookOutcome::Passed { .. } => panic!("expected Failed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn shell_hook_fail_with_rollback_marks_outcome_for_drone_revert() {
        let spawner = FakeSpawner::fail(1, "verify failed");
        let deps = HookDeps {
            shell_spawner: &spawner,
        };
        let hook = shell_hook("verify", "false", HookOnFailure::Rollback);
        let outcome = execute_hook(&hook, &ctx(), &deps).await.expect("ok");
        match outcome {
            HookOutcome::Failed { on_failure, .. } => {
                assert!(matches!(on_failure, HookOnFailure::Rollback));
            }
            other @ HookOutcome::Passed { .. } => panic!("expected Failed Rollback, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn shell_hook_fail_with_warn_marks_outcome_for_warn_only() {
        let spawner = FakeSpawner::fail(1, "lint warning");
        let deps = HookDeps {
            shell_spawner: &spawner,
        };
        let hook = shell_hook("lint", "false", HookOnFailure::Warn);
        let outcome = execute_hook(&hook, &ctx(), &deps).await.expect("ok");
        match outcome {
            HookOutcome::Failed { on_failure, .. } => {
                assert!(matches!(on_failure, HookOnFailure::Warn));
            }
            other @ HookOutcome::Passed { .. } => panic!("expected Failed Warn, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn tool_variant_returns_deferred_until_m05() {
        let spawner = FakeSpawner::ok("");
        let deps = HookDeps {
            shell_spawner: &spawner,
        };
        let hook = Hook {
            id: HookId::try_from("lint".to_string()).unwrap(),
            category: HookCategory::Lint,
            level: None,
            ref_: HookRef::Tool {
                tool_name: runtime_core::generated::common::HookRefToolName::try_from(
                    "lint_changed_files".to_string(),
                )
                .unwrap(),
                input: serde_json::Map::new(),
            },
            on_failure: HookOnFailure::Warn,
        };
        let err = execute_hook(&hook, &ctx(), &deps)
            .await
            .expect_err("deferred");
        match err {
            HookError::DeferredVariant(s) => assert!(s.contains("tool:lint_changed_files")),
        }
    }

    #[tokio::test]
    async fn agent_variant_returns_deferred_until_m07() {
        let spawner = FakeSpawner::ok("");
        let deps = HookDeps {
            shell_spawner: &spawner,
        };
        let hook = Hook {
            id: HookId::try_from("review".to_string()).unwrap(),
            category: HookCategory::Custom,
            level: None,
            ref_: HookRef::Agent {
                agent_id: runtime_core::generated::common::HookRefAgentId::try_from(
                    "reviewer".to_string(),
                )
                .unwrap(),
                prompt: None,
            },
            on_failure: HookOnFailure::Block,
        };
        let err = execute_hook(&hook, &ctx(), &deps)
            .await
            .expect_err("deferred");
        match err {
            HookError::DeferredVariant(s) => assert!(s.contains("agent:reviewer")),
        }
    }

    #[tokio::test]
    async fn empty_stdout_yields_no_output_preview() {
        let spawner = FakeSpawner::ok("");
        let deps = HookDeps {
            shell_spawner: &spawner,
        };
        let hook = shell_hook("verify", "true", HookOnFailure::Block);
        let outcome = execute_hook(&hook, &ctx(), &deps).await.expect("ok");
        match outcome {
            HookOutcome::Passed { output_preview, .. } => assert!(output_preview.is_none()),
            other @ HookOutcome::Failed { .. } => panic!("expected Passed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn long_stdout_is_truncated_at_preview_cap() {
        let huge = "a".repeat(OUTPUT_PREVIEW_MAX_BYTES + 200);
        let spawner = FakeSpawner::ok(&huge);
        let deps = HookDeps {
            shell_spawner: &spawner,
        };
        let hook = shell_hook("verify", "echo", HookOnFailure::Block);
        let outcome = execute_hook(&hook, &ctx(), &deps).await.expect("ok");
        match outcome {
            HookOutcome::Passed { output_preview, .. } => {
                let preview = output_preview.expect("some");
                // ASCII-only: truncated to exactly cap bytes + ellipsis.
                assert!(preview.ends_with('…'));
                assert!(preview.len() < huge.len());
            }
            other @ HookOutcome::Failed { .. } => panic!("expected Passed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn multibyte_stdout_truncates_on_char_boundary() {
        // 3-byte chars ('é' is 2 bytes; pick 3-byte for clearer boundary).
        let multibyte: String = "中".repeat(OUTPUT_PREVIEW_MAX_BYTES); // bytes far exceed cap
        let spawner = FakeSpawner::ok(&multibyte);
        let deps = HookDeps {
            shell_spawner: &spawner,
        };
        let hook = shell_hook("verify", "echo", HookOnFailure::Block);
        let outcome = execute_hook(&hook, &ctx(), &deps).await.expect("ok");
        match outcome {
            HookOutcome::Passed { output_preview, .. } => {
                let preview = output_preview.expect("some");
                // Must be a valid &str slice — the boundary walk rejected
                // mid-codepoint cuts.
                assert!(preview.ends_with('…'));
                assert!(preview.len() <= OUTPUT_PREVIEW_MAX_BYTES + '…'.len_utf8());
            }
            other @ HookOutcome::Failed { .. } => panic!("expected Passed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn timeout_error_surfaces_as_failed_outcome() {
        let spawner = FakeSpawner {
            outcome: Mutex::new(Err(ShellError::Timeout(500))),
        };
        let deps = HookDeps {
            shell_spawner: &spawner,
        };
        let hook = shell_hook("verify", "sleep 9", HookOnFailure::Rollback);
        let outcome = execute_hook(&hook, &ctx(), &deps).await.expect("ok");
        match outcome {
            HookOutcome::Failed { error, .. } => assert!(error.contains("timed out")),
            other @ HookOutcome::Passed { .. } => panic!("expected Failed timeout, got {other:?}"),
        }
    }
}
