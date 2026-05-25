//! M08.5.5 Stage B2.fix — assembled regression for the `cmd.exe /C`
//! outer-quoting bug surfaced by the IRL re-verify on 2026-05-24
//! against the post-B.fix build (commit `94c2bc7`).
//!
//! ## The bug (current HEAD)
//!
//! `build_command` constructs the inner full command line with each
//! segment individually quoted: `"npx.cmd" -y "@..." "C:\path"` (when
//! the path arg triggers quoting). The B.fix wrapper passes this to
//! `cmd.exe` as `/C <full command line>` via `raw_arg`. The resulting
//! command line that cmd.exe parses is:
//!
//! ```text
//! /C "npx.cmd" -y "@..." "C:\path"
//! ```
//!
//! cmd.exe's `/?`-documented quote-handling has two rules. Rule 1
//! preserves outer quotes when ALL of the following hold: no `/S` switch,
//! EXACTLY two quote characters total, no special characters between
//! them, whitespace between them, and the inside-quotes string names an
//! executable. A multi-arg invocation with multiple quoted segments
//! fails the "exactly two quote characters" condition immediately, so
//! rule 1 doesn't fire. Rule 2 then takes over: "if the first character
//! is a quote character, strip the leading character and remove the
//! last quote character on the command line, preserving any text after
//! the last quote character."
//!
//! Applied to our command line, rule 2 strips the leading `"` (before
//! `npx.cmd`) and the trailing `"` (after `C:\path`), leaving:
//!
//! ```text
//! npx.cmd" -y "@..." "C:\path
//! ```
//!
//! cmd.exe then parses this as a command: the first whitespace-delimited
//! token is `npx.cmd"` — the resolved program name with a stray literal
//! `"` from the second quoted segment's stripped opener. The literal
//! `"` is not a valid filename character on Windows, so cmd.exe exits
//! with status 1 after writing:
//!
//! ```text
//! The filename, directory name, or volume label syntax is incorrect.
//! ```
//!
//! to its own stderr. This is the SAME error-message class as the
//! pre-B.fix ``BatBadBut`` failure (cmd.exe's generic
//! invalid-filename-syntax error), so the user-visible symptom is
//! identical to pre-B.fix: the UI sees an infinite loop (cmd.exe
//! itself launched fine — Rust's spawn succeeded — but the child
//! immediately exits with status 1 and no JSON-RPC output; rmcp's
//! `serve()` handshake hangs waiting for an initialize response that
//! never comes). B.fix's wrapper bypassed `BatBadBut` at the spawn layer
//! but introduced an equivalent failure at the cmd.exe parsing layer.
//!
//! ## The fix (this stage's impl)
//!
//! Wrap the inner full command line in an OUTER pair of quotes:
//!
//! ```text
//! /C ""npx.cmd" -y "@..." "C:\path""
//! ```
//!
//! cmd.exe's rule 2 strips the first `"` and the last `"`, leaving:
//!
//! ```text
//! "npx.cmd" -y "@..." "C:\path"
//! ```
//!
//! which cmd.exe parses correctly: `"npx.cmd"` is the program (the
//! quotes are then stripped during program lookup, finding `npx.cmd`
//! on PATH), and each subsequent quoted segment is one arg.
//!
//! ## Test mechanics
//!
//! The test spawns the command directly via `tokio::process::Command::
//! output()` (not via `transport.connect()`) so we capture cmd.exe's
//! stderr. `connect()`'s rmcp child-process handshake inherits the
//! child's stderr to the parent and surfaces a generic
//! "child exited before handshake" error — useless for asserting the
//! SPECIFIC cmd.exe-quote-strip signature.
//!
//! ## Environment requirement
//!
//! Requires `npx` (i.e., `npx.cmd`) on PATH. The build machine + the
//! GitHub `windows-latest` CI runners satisfy this via Node.js. In an
//! env without npx, the same "is not recognized" string would fire
//! post-fix for a different reason (cmd.exe couldn't find `npx.cmd` at
//! all) — the test would fail with a misleading-but-detectable message;
//! install Node + retry.
//!
//! ## References
//!
//! - Microsoft `cmd` docs (quote-handling rules):
//!   <https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd>
//! - The pre-fix B.fix ADR-0023 (Accepted at `230e1c3`) is amended by
//!   this stage's impl commit; the amendment adds the outer-quote
//!   paragraph to the Decision section.

#[cfg(target_os = "windows")]
mod windows {
    use std::time::Duration;

    use runtime_mcp::transport::StdioTransport;
    use tempfile::TempDir;
    use tokio::time::timeout;

    #[tokio::test]
    async fn npx_cmd_with_path_arg_does_not_fire_cmd_exe_quote_strip_error() {
        // The exact arg combination from M08.5 + M08.5.5 IRL re-verify
        // (real Tauri app, 2026-05-23 + 2026-05-24): npx + -y +
        // server-filesystem + a path arg with drive-letter + backslashes.
        // The path arg is the trigger for the third quoted segment that
        // pushes the inner command line over cmd.exe rule 1's "exactly
        // two quote characters" condition.
        let scratch = TempDir::new().expect("create scratch dir");
        let scratch_path = scratch.path().to_string_lossy().to_string();
        let transport = StdioTransport::new("npx").with_args(vec![
            "-y".into(),
            "@modelcontextprotocol/server-filesystem".into(),
            scratch_path,
        ]);

        let mut cmd = transport.build_command();
        // Close stdin so any downstream MCP server reading from stdin
        // (post-fix path) sees EOF + exits, instead of blocking the
        // 60s budget on a stdin-read.
        cmd.stdin(std::process::Stdio::null());

        let output = timeout(Duration::from_secs(60), cmd.output())
            .await
            .expect("cmd spawned + completed within 60s budget")
            .expect("spawn returned an Err");

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            !stderr.contains("The filename, directory name, or volume label syntax is incorrect"),
            "M08.5.5 IRL 🔴 B2: cmd.exe stripped the first + last quotes of the inner \
             /C command line (cmd /? rule 2 — multi-quote-pair case), leaving a mangled \
             first token with a literal `\"` character that Windows rejects as an invalid \
             filename. Fix: outer-quote the inner full command line so rule 2 strips only \
             that outer pair, preserving the inner program-+-args quoting. Exit status: \
             {status:?}; stderr:\n{stderr}\nstdout:\n{stdout}",
            status = output.status,
        );
    }
}
