//! Connection lifecycle + health-ping loop (M06 Stage C).
//!
//! Owns the connect/disconnect/health-ping mechanics for the `McpClient`.
//! Health-ping cadence is 30s by default per spec §5; sustained ping
//! failure routes through the existing M05.A `mcp_missing` event variant
//! plus the existing M04.E `on_gap` HITL trigger. **No new event variant
//! and no new HITL trigger** for the offline case — the user-visible
//! semantics of "MCP server went offline" match "MCP server reference
//! was missing at framework load."
//!
//! Per ADR-0007 the seam state lives in the main process; the drone is
//! audit + projection, not orchestrator. The health-ping loop runs as a
//! detached `tokio::task::JoinHandle<()>` **returned to the caller** (the
//! Tauri shell), not stored on the `McpClient`. The spawned task holds an
//! `Arc<McpClient>`, so the client stays alive for the loop's lifetime;
//! the loop only stops when the caller explicitly `.abort()`s the handle
//! (e.g., at app shutdown). There is no abort-on-`McpClient`-drop — the
//! `Arc` the loop holds prevents that drop from ever firing while the
//! loop runs.

use std::sync::Arc;
use std::time::Duration;

use tokio::task::JoinHandle;

use crate::client::McpClient;

/// Default cadence for health-ping loop.
///
/// Per spec §5; configurable via [`spawn_health_pinger_with_interval`]
/// for tests.
pub const DEFAULT_HEALTH_PING_INTERVAL: Duration = Duration::from_secs(30);

/// Spawn the health-ping loop using [`DEFAULT_HEALTH_PING_INTERVAL`].
#[must_use]
pub fn spawn_health_pinger(client: Arc<McpClient>) -> JoinHandle<()> {
    spawn_health_pinger_with_interval(client, DEFAULT_HEALTH_PING_INTERVAL)
}

/// Spawn the health-ping loop with a caller-supplied interval.
///
/// Per the CLAUDE.md §9 `*_with` archetype: tests inject short intervals
/// (e.g., 50ms) so the loop's behavior is observable within a single
/// test run. Failed pings route through the supplied `emit_missing`
/// callback bound at the call site to the existing `mcp_missing` event
/// variant. The loop runs until the caller `.abort()`s the returned
/// [`JoinHandle`]; it does not stop on `McpClient` drop (the spawned
/// task holds an `Arc<McpClient>`, keeping the client alive until the
/// handle is aborted).
#[must_use]
pub fn spawn_health_pinger_with_interval(
    client: Arc<McpClient>,
    interval: Duration,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(interval);
        // Skip the immediate first tick; wait one interval before the
        // first health pass so a freshly-spawned client doesn't
        // simultaneously connect + ping.
        tick.tick().await;
        loop {
            tick.tick().await;
            client
                .run_health_pass(|name| {
                    tracing::warn!(name = %name, "MCP server health-ping failed; routing through mcp_missing");
                })
                .await;
        }
    })
}
