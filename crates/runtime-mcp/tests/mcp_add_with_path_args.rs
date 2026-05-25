//! M08.5.5 Stage B.fix — assembled regression for the MCP Add flow with
//! Windows path arguments. Reproduces the IRL-failing command (`npx` +
//! `@modelcontextprotocol/server-filesystem` + Windows path arg) and
//! asserts the spawn does NOT produce the pre-fix Windows OS-level
//! "filename, directory name, or volume label syntax is incorrect"
//! command-line-parse error.
//!
//! Pre-fix `main` (Windows): fails the assertion — Rust 1.77.2+
//! `BatBadBut`-safe escaping (`CVE-2024-24576` fix) produces a
//! `cmd.exe`-unparseable command line for path args containing `:` + `\`.
//! Post-fix (`ADR-0023`): the `cmd.exe /C` wrapper bypasses `BatBadBut`;
//! the spawn either succeeds or fails for a downstream reason (`npx`
//! not installed, package not cached, etc.) that does NOT match the
//! pre-fix OS-level signature.

#[cfg(target_os = "windows")]
mod windows {
    use std::time::Duration;

    use runtime_mcp::transport::{StdioTransport, Transport};
    use tempfile::TempDir;
    use tokio::time::timeout;

    #[tokio::test]
    async fn mcp_npx_with_windows_path_arg_spawns_without_os_error() {
        // The exact arg combination from M08.5 IRL re-verify (2026-05-23,
        // build machine, real Tauri app): npx -y server-filesystem with a
        // scratch directory path arg (drive-letter + backslashes).
        let scratch = TempDir::new().expect("create scratch dir");
        let scratch_path = scratch.path().to_string_lossy().to_string();
        let transport = StdioTransport::new("npx").with_args(vec![
            "-y".into(),
            "@modelcontextprotocol/server-filesystem".into(),
            scratch_path,
        ]);

        // We do not expect the npx invocation to fully succeed in CI —
        // server-filesystem may not be available depending on the runner's
        // npm cache. What we DO assert: the spawn does NOT fail with the
        // Windows OS-level "filename, directory name, or volume label
        // syntax is incorrect" command-line-parse error. A ConnectFailed
        // for any downstream reason (server failed to start, handshake
        // timeout, package not cached) is acceptable — those prove the
        // spawn at least reached cmd.exe / npx.
        let result = timeout(Duration::from_secs(30), transport.connect()).await;
        match result {
            Ok(Ok(_conn)) => {
                // Spawn + initialize handshake succeeded entirely. Nothing
                // more to assert — the wrapper let the real command line
                // through cleanly.
            }
            Ok(Err(err)) => {
                let msg = err.to_string();
                assert!(
                    !msg.contains("filename, directory name, or volume label syntax is incorrect"),
                    "M08.5 IRL 🔴 #6 regression: cmd.exe wrapper did not bypass the BatBadBut-\
                     escape path; got OS-level command-line parse error: {msg}"
                );
            }
            Err(_elapsed) => {
                // Timeout — the spawn took longer than the budget but did
                // NOT return the pre-fix OS-level error. Acceptable for
                // this regression; the BatBadBut failure is synchronous.
            }
        }
    }
}
