//! `DroneClient` — main-side connection wrapper around the M01 drone.
//!
//! Cfg-platform under the hood: Unix domain socket on Linux/macOS, Windows
//! named pipe on Windows. Reconnects automatically on transport errors per
//! the policy in [`super::connection`].
//!
//! M02 only sends [`runtime_core::DroneCommand::SnapshotNow`] (one per
//! task lifecycle). M03+ adds the rest of the variants as new subsystems
//! land.

use std::pin::Pin;

use futures::stream::Stream;
use runtime_core::drone::{DroneCommand, DroneEvent};
use tokio::sync::Mutex;

use super::connection::{Connection, DroneIpcError};

/// Main-side IPC client for the runtime-drone subprocess.
pub struct DroneClient {
    inner: Mutex<Connection>,
}

impl DroneClient {
    /// Connect to a running drone over its IPC socket / named pipe.
    ///
    /// `addr` is a filesystem path on Unix, or a named-pipe name on
    /// Windows (e.g. `\\.\pipe\drone-abc`).
    ///
    /// # Errors
    ///
    /// Returns [`DroneIpcError::Io`] if the underlying open fails.
    pub async fn connect(addr: &str) -> Result<Self, DroneIpcError> {
        let conn = Connection::connect(addr).await?;
        Ok(Self {
            inner: Mutex::new(conn),
        })
    }

    /// Test affordance — construct a no-op client. `send` returns
    /// `Ok(())` immediately and `events` yields nothing. Used by SDK
    /// tests that don't exercise the drone path.
    #[must_use]
    pub fn noop() -> Self {
        Self {
            inner: Mutex::new(Connection::noop()),
        }
    }

    /// Send a `DroneCommand`, retrying transport errors per the
    /// connection's reconnect policy.
    ///
    /// # Errors
    ///
    /// Surfaces [`DroneIpcError::Disconnected`] if the retry budget is
    /// exhausted; [`DroneIpcError::Json`] on serialization bugs.
    pub async fn send(&self, cmd: DroneCommand) -> Result<(), DroneIpcError> {
        let mut guard = self.inner.lock().await;
        guard.send_with_reconnect(cmd).await
    }

    /// Take the inbound `DroneEvent` stream. Single-consumer per
    /// connection; subsequent calls return an empty stream.
    ///
    /// # Errors
    ///
    /// The returned stream yields `Err(DroneIpcError::Codec)` on framing
    /// errors and `Err(DroneIpcError::Json)` on payload parse errors;
    /// neither terminates the stream.
    pub async fn events(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<DroneEvent, DroneIpcError>> + Send>>, DroneIpcError>
    {
        let mut guard = self.inner.lock().await;
        Ok(guard.take_event_stream())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn noop_send_succeeds() {
        let c = DroneClient::noop();
        c.send(DroneCommand::SnapshotNow {
            reason: "x".into(),
            state_json: serde_json::json!({}),
        })
        .await
        .expect("noop send");
    }

    #[tokio::test]
    async fn noop_events_yields_nothing() {
        let c = DroneClient::noop();
        let mut s = c.events().await.expect("events");
        assert!(s.next().await.is_none());
    }
}
