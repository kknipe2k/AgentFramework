//! Cross-platform shell hook execution. Spec §4a Verify & Rails.
//!
//! ## Wrapper choice
//!
//! On Windows the runtime invokes `powershell.exe -NoProfile -Command "..."`
//! (Windows PowerShell 5.1 — pwsh.exe was unavailable on the M04.D build
//! machine; documented in the M04.D retrospective). On Linux/macOS the
//! runtime invokes `bash -c "..."`. The flag spelling + semantics are
//! verified verbatim against Microsoft's `about_PowerShell_exe` reference
//! (M04.D WEBCHECK).
//!
//! ## Testable seam (`execute_shell_with`)
//!
//! Per CLAUDE.md §5 + the M02/A2 wrapper-vs-seam pattern: the real OS-spawn
//! wrapper [`execute_shell`] is excluded from coverage gates (constructs a
//! [`tokio::process::Command`] against the live OS); the testable seam
//! [`execute_shell_with`] takes a [`ShellSpawner`] trait and runs the same
//! decision logic against a mock spawner. Tests cover the wrapper-arg
//! decision (Windows vs Unix flags + cwd + timeout) without spawning a
//! real process.

use std::path::Path;
use std::time::Duration;
use thiserror::Error;
use tokio::time::timeout;

/// Errors raised by shell hook execution.
#[derive(Debug, Error)]
pub enum ShellError {
    /// Spawn failed (binary not found, permission denied, etc.).
    #[error("shell spawn failed: {0}")]
    Spawn(String),
    /// Process exited with a non-zero status.
    #[error("shell exited {status}: {stderr}")]
    NonZeroExit {
        /// Exit code (or `-1` for signals).
        status: i32,
        /// Captured stderr (truncated to a preview by the caller).
        stderr: String,
    },
    /// Process exceeded `timeout_ms`.
    #[error("shell timed out after {0} ms")]
    Timeout(u64),
}

/// Captured output of a successful shell invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellOutput {
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr (often empty on success).
    pub stderr: String,
    /// Exit code (always 0 for `Ok` results).
    pub status: i32,
}

/// Args the runtime asked the OS spawner to run.
///
/// Captured by `execute_shell_with` for testable assertions on what a
/// production spawner would have invoked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnArgs {
    /// Program name (e.g., `"powershell.exe"` or `"bash"`).
    pub program: String,
    /// Arg list (e.g., `["-NoProfile", "-Command", "..."]`).
    pub args: Vec<String>,
    /// Working directory, if requested.
    pub cwd: Option<String>,
    /// Timeout in milliseconds, if requested.
    pub timeout_ms: Option<u64>,
}

/// Trait abstracting subprocess spawn. Production impl is
/// [`TokioShellSpawner`]; tests inject mock implementations.
#[async_trait::async_trait]
pub trait ShellSpawner: Send + Sync {
    /// Run the program with the given args. Implementations capture
    /// stdout/stderr/status and return them.
    async fn spawn(&self, args: SpawnArgs) -> Result<ShellOutput, ShellError>;
}

/// Production spawner — runs the real OS subprocess via
/// [`tokio::process::Command`]. Excluded from coverage gates per the
/// runtime-main wrapper-vs-seam pattern (see crate-level docs).
#[derive(Debug, Default, Clone, Copy)]
pub struct TokioShellSpawner;

#[async_trait::async_trait]
impl ShellSpawner for TokioShellSpawner {
    async fn spawn(&self, args: SpawnArgs) -> Result<ShellOutput, ShellError> {
        let mut cmd = tokio::process::Command::new(&args.program);
        cmd.args(&args.args);
        if let Some(cwd) = &args.cwd {
            cmd.current_dir(cwd);
        }
        cmd.kill_on_drop(true);
        let fut = cmd.output();
        let output = if let Some(t) = args.timeout_ms {
            match timeout(Duration::from_millis(t), fut).await {
                Ok(r) => r.map_err(|e| ShellError::Spawn(e.to_string()))?,
                Err(_) => return Err(ShellError::Timeout(t)),
            }
        } else {
            fut.await.map_err(|e| ShellError::Spawn(e.to_string()))?
        };
        let status = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        if output.status.success() {
            Ok(ShellOutput {
                stdout,
                stderr,
                status,
            })
        } else {
            Err(ShellError::NonZeroExit { status, stderr })
        }
    }
}

/// Build the [`SpawnArgs`] the OS spawner will receive.
///
/// Centralizes the platform-conditional wrapper choice (PowerShell on
/// Windows; bash on Linux/macOS) so tests can assert the wrapper
/// decision without spawning a process.
#[must_use]
pub fn build_spawn_args(command: &str, timeout_ms: Option<u64>, cwd: Option<&Path>) -> SpawnArgs {
    let (program, args) = if cfg!(target_os = "windows") {
        (
            "powershell.exe".to_string(),
            vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                command.to_string(),
            ],
        )
    } else {
        (
            "bash".to_string(),
            vec!["-c".to_string(), command.to_string()],
        )
    };
    SpawnArgs {
        program,
        args,
        cwd: cwd.map(|p| p.display().to_string()),
        timeout_ms,
    }
}

/// Production wrapper — spawns the real shell against the OS. Excluded
/// from coverage gates per the runtime-main wrapper-vs-seam pattern.
///
/// # Errors
///
/// See [`ShellError`].
pub async fn execute_shell(
    command: &str,
    timeout_ms: Option<u64>,
    cwd: Option<&Path>,
) -> Result<ShellOutput, ShellError> {
    execute_shell_with(&TokioShellSpawner, command, timeout_ms, cwd).await
}

/// Testable seam — runs the shell-execution decision logic (build args,
/// dispatch to spawner) against an injected [`ShellSpawner`].
///
/// # Errors
///
/// See [`ShellError`].
pub async fn execute_shell_with(
    spawner: &dyn ShellSpawner,
    command: &str,
    timeout_ms: Option<u64>,
    cwd: Option<&Path>,
) -> Result<ShellOutput, ShellError> {
    let args = build_spawn_args(command, timeout_ms, cwd);
    spawner.spawn(args).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Mutex;

    /// Captures the `SpawnArgs` it was given and returns a canned outcome.
    struct MockSpawner {
        captured: Mutex<Option<SpawnArgs>>,
        outcome: Result<ShellOutput, ShellError>,
    }

    impl MockSpawner {
        fn ok(stdout: &str) -> Self {
            Self {
                captured: Mutex::new(None),
                outcome: Ok(ShellOutput {
                    stdout: stdout.to_string(),
                    stderr: String::new(),
                    status: 0,
                }),
            }
        }
        fn fail(status: i32, stderr: &str) -> Self {
            Self {
                captured: Mutex::new(None),
                outcome: Err(ShellError::NonZeroExit {
                    status,
                    stderr: stderr.to_string(),
                }),
            }
        }
        fn timeout_err(ms: u64) -> Self {
            Self {
                captured: Mutex::new(None),
                outcome: Err(ShellError::Timeout(ms)),
            }
        }
        fn captured(&self) -> SpawnArgs {
            self.captured.lock().unwrap().clone().expect("not captured")
        }
    }

    #[async_trait::async_trait]
    impl ShellSpawner for MockSpawner {
        async fn spawn(&self, args: SpawnArgs) -> Result<ShellOutput, ShellError> {
            *self.captured.lock().unwrap() = Some(args);
            // Clone the canned outcome each call.
            match &self.outcome {
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

    #[test]
    fn build_spawn_args_uses_powershell_on_windows() {
        let args = build_spawn_args("echo hi", Some(5000), None);
        if cfg!(target_os = "windows") {
            assert_eq!(args.program, "powershell.exe");
            assert_eq!(args.args, vec!["-NoProfile", "-Command", "echo hi"]);
        } else {
            assert_eq!(args.program, "bash");
            assert_eq!(args.args, vec!["-c", "echo hi"]);
        }
        assert_eq!(args.timeout_ms, Some(5000));
        assert!(args.cwd.is_none());
    }

    #[test]
    fn build_spawn_args_carries_cwd_when_supplied() {
        let cwd = PathBuf::from("/tmp/work");
        let args = build_spawn_args("ls", None, Some(&cwd));
        assert_eq!(
            args.cwd.as_deref(),
            Some(cwd.display().to_string().as_str())
        );
    }

    #[tokio::test]
    async fn execute_shell_with_dispatches_to_spawner() {
        let spawner = MockSpawner::ok("hello\n");
        let out = execute_shell_with(&spawner, "echo hello", Some(2000), None)
            .await
            .expect("ok");
        assert_eq!(out.stdout, "hello\n");
        assert_eq!(out.status, 0);
        let captured = spawner.captured();
        assert_eq!(captured.timeout_ms, Some(2000));
    }

    #[tokio::test]
    async fn execute_shell_with_propagates_non_zero_exit() {
        let spawner = MockSpawner::fail(2, "boom\n");
        let err = execute_shell_with(&spawner, "false", None, None)
            .await
            .expect_err("expected non-zero");
        match err {
            ShellError::NonZeroExit { status, stderr } => {
                assert_eq!(status, 2);
                assert_eq!(stderr, "boom\n");
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[tokio::test]
    async fn execute_shell_with_propagates_timeout() {
        let spawner = MockSpawner::timeout_err(100);
        let err = execute_shell_with(&spawner, "sleep 9999", Some(100), None)
            .await
            .expect_err("expected timeout");
        match err {
            ShellError::Timeout(ms) => assert_eq!(ms, 100),
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[tokio::test]
    async fn execute_shell_with_carries_cwd_to_spawner() {
        let spawner = MockSpawner::ok("");
        let cwd = PathBuf::from("/work/dir");
        let _ = execute_shell_with(&spawner, "pwd", None, Some(&cwd))
            .await
            .expect("ok");
        let captured = spawner.captured();
        assert!(captured.cwd.unwrap().contains("/work/dir"));
    }

    #[test]
    fn shell_error_display_covers_each_variant() {
        // Locks the user-facing error string; the executor surfaces these
        // verbatim in `verify_failed.error`.
        assert!(ShellError::Spawn("boom".into())
            .to_string()
            .contains("boom"));
        assert!(ShellError::NonZeroExit {
            status: 7,
            stderr: "msg".into()
        }
        .to_string()
        .contains('7'));
        assert!(ShellError::Timeout(500).to_string().contains("500"));
    }
}
