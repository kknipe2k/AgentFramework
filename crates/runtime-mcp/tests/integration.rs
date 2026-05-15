//! Feature-gated integration smoke against a real MCP server.
//!
//! Run with: `cargo test -p runtime-mcp --features integration`
//!
//! Skipped from default CI (the unit tests via [`MockTransport`] are the
//! canonical CI surface). This file is the M02.C `anthropic_smoke.rs`
//! analogue — manual gate against real protocol.
//!
//! Requires `npx` on PATH and network access for `npx -y
//! @modelcontextprotocol/server-everything` to download + run the
//! reference MCP server. If `npx` is unavailable the test logs a SKIP
//! message and exits cleanly.

#![cfg(feature = "integration")]

use runtime_mcp::transport::{StdioTransport, Transport};

fn has_command(name: &str) -> bool {
    let probe = if cfg!(windows) { "where" } else { "which" };
    std::process::Command::new(probe)
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[tokio::test]
async fn stdio_against_reference_server_everything() {
    let npx = if cfg!(windows) { "npx.cmd" } else { "npx" };
    if !has_command(npx) {
        eprintln!("SKIP: {npx} not on PATH");
        return;
    }

    let transport = StdioTransport::new(npx).with_args(vec![
        "-y".into(),
        "@modelcontextprotocol/server-everything".into(),
    ]);
    let conn = transport.connect().await.expect("connect");
    let tools = conn.list_tools().await.expect("list_tools");
    assert!(
        !tools.is_empty(),
        "server-everything must expose at least one tool"
    );
    assert!(
        tools.iter().any(|t| t.name == "echo"),
        "server-everything must expose `echo` (got {tools:?})"
    );
    conn.ping().await.expect("ping");
}
