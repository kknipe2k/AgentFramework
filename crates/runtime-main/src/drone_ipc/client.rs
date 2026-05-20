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
use std::time::Duration;

use futures::stream::Stream;
use runtime_core::drone::{DroneCommand, DroneEvent};
use serde_json::Value;
use tokio::sync::Mutex;

use super::connection::{Connection, DroneIpcError};

/// Maximum time to wait for a request/response event before giving up.
/// Used by [`DroneClient::query_session_db`] and
/// [`DroneClient::read_signals`].
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

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

    /// Send a `QuerySessionDb` command and await the matching
    /// `QueryResult` response from the inbound event stream. Heartbeats
    /// and unrelated events are skipped. On noop mode returns an empty
    /// result vector immediately.
    ///
    /// # Errors
    ///
    /// - [`DroneIpcError::Disconnected`] on send retry exhaustion.
    /// - [`DroneIpcError::Json`] if the response cannot be parsed.
    /// - [`DroneIpcError::Codec`] (wire format) on framing errors.
    /// - [`DroneIpcError::Io`] with `TimedOut` if no response arrives
    ///   within `RESPONSE_TIMEOUT` (5 seconds).
    pub async fn query_session_db(&self, sql: String) -> Result<Vec<Value>, DroneIpcError> {
        let mut guard = self.inner.lock().await;
        if guard.is_noop() {
            return Ok(Vec::new());
        }
        guard
            .send_with_reconnect(DroneCommand::QuerySessionDb { sql })
            .await?;
        let result = await_event(&mut guard, |e| match e {
            DroneEvent::QueryResult { rows } => Some(Ok(rows)),
            DroneEvent::Alert { message, .. } if message.starts_with("query_session_db") => {
                Some(Err(DroneIpcError::Codec(message)))
            }
            _ => None,
        })
        .await;
        drop(guard);
        result
    }

    /// Write a signal to the drone's `signals` table. Drone-side handler
    /// runs the projectors (`vdr` for decision/verify; `plan_projector`
    /// for plan/task) inside the same transaction. Fire-and-forget — no
    /// response awaited; failures surface as Drone Alerts on the event
    /// stream. Spec §2b. M04 Stage B (closes M03 🟡 carry-forward).
    ///
    /// # Errors
    ///
    /// Surfaces [`DroneIpcError::Disconnected`] if the retry budget is
    /// exhausted; [`DroneIpcError::Json`] on serialization bugs.
    #[allow(clippy::too_many_arguments)]
    pub async fn write_signal(
        &self,
        signal_id: String,
        session_id: String,
        kind: String,
        event: String,
        context_type: String,
        payload: Value,
    ) -> Result<(), DroneIpcError> {
        let mut guard = self.inner.lock().await;
        if guard.is_noop() {
            return Ok(());
        }
        guard
            .send_with_reconnect(DroneCommand::WriteSignal {
                signal_id,
                session_id,
                kind,
                event,
                context_type,
                payload,
            })
            .await
    }

    /// Send a `ReadSignals` command and await the matching `SignalLog`
    /// response. Same skip-and-filter behavior as
    /// [`Self::query_session_db`].
    ///
    /// # Errors
    ///
    /// Same as [`Self::query_session_db`].
    pub async fn read_signals(&self, session_id: String) -> Result<Vec<Value>, DroneIpcError> {
        let mut guard = self.inner.lock().await;
        if guard.is_noop() {
            return Ok(Vec::new());
        }
        guard
            .send_with_reconnect(DroneCommand::ReadSignals { session_id })
            .await?;
        let result = await_event(&mut guard, |e| match e {
            DroneEvent::SignalLog { signals } => Some(Ok(signals)),
            DroneEvent::Alert { message, .. } if message.starts_with("read_signals") => {
                Some(Err(DroneIpcError::Codec(message)))
            }
            _ => None,
        })
        .await;
        drop(guard);
        result
    }

    /// Send a `RecoverSession` command and await the matching
    /// `SessionRecovered` response. Per spec §1b — rebuilds history from
    /// the latest snapshot + projected plan/task rows + uncertain
    /// tool-invocation ids. Tools are NOT re-invoked on resume
    /// (gotcha #15); caller is expected to load the returned state into
    /// SDK message history and prompt the user for each
    /// `uncertain_tool_invocation`. On noop mode returns a default
    /// [`RecoveredSession`] immediately. M04 Stage F.
    ///
    /// # Errors
    ///
    /// Same as [`Self::query_session_db`].
    pub async fn recover_session(
        &self,
        session_id: String,
    ) -> Result<RecoveredSession, DroneIpcError> {
        let mut guard = self.inner.lock().await;
        if guard.is_noop() {
            return Ok(RecoveredSession::default());
        }
        guard
            .send_with_reconnect(DroneCommand::RecoverSession { session_id })
            .await?;
        let result = await_recovery(&mut guard).await;
        drop(guard);
        result
    }
}

/// Recovered session state surfaced to main by the drone.
///
/// See [`DroneEvent::SessionRecovered`] for the wire shape. Mirrors
/// `runtime_drone::snapshot::RecoveredSession` but lives here so
/// consumers of `runtime_main` don't transitively pull `runtime_drone`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecoveredSession {
    /// Snapshot id the state was loaded from.
    pub snapshot_id: Option<String>,
    /// Decoded snapshot state.
    pub state: Value,
    /// Plan rows in creation order.
    pub plans: Vec<Value>,
    /// Task rows with running-tasks normalized to pending per spec §1b.
    pub tasks: Vec<Value>,
    /// Signal ids of tool invocations lacking a matching result.
    pub uncertain_tool_invocations: Vec<String>,
}

async fn await_recovery(conn: &mut Connection) -> Result<RecoveredSession, DroneIpcError> {
    let timed = tokio::time::timeout(RESPONSE_TIMEOUT, async {
        loop {
            match conn.next_event().await {
                Some(Ok(DroneEvent::SessionRecovered {
                    snapshot_id,
                    state,
                    plans,
                    tasks,
                    uncertain_tool_invocations,
                })) => {
                    return Ok(RecoveredSession {
                        snapshot_id,
                        state,
                        plans,
                        tasks,
                        uncertain_tool_invocations,
                    });
                }
                Some(Ok(DroneEvent::Alert { message, .. }))
                    if message.starts_with("recover_session") =>
                {
                    return Err(DroneIpcError::Codec(message));
                }
                Some(Ok(_)) => {}
                Some(Err(e)) => return Err(e),
                None => {
                    return Err(DroneIpcError::Codec(
                        "event stream ended without recover_session response".into(),
                    ));
                }
            }
        }
    })
    .await;
    timed.unwrap_or_else(|_| {
        Err(DroneIpcError::Io(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "drone response timeout",
        )))
    })
}

/// Pull events from the connection's reader half (via borrow-not-move
/// [`Connection::next_event`]) until `filter` returns `Some`. Used by
/// `query_session_db` and `read_signals` to ignore Heartbeats / unrelated
/// events while the matching response arrives.
///
/// The borrow-not-move pattern is load-bearing: each request-response
/// call leaves the reader installed on the connection so the *next*
/// request-response call can read more events. The prior `take_event_stream`
/// pattern moved the reader into a per-call stream that was dropped at
/// return; the second call would see an empty reader and immediately
/// surface `"event stream ended without response"`. Caught by M04 IRL test
/// (SQL inspector + replay both broken post-smoke).
async fn await_event(
    conn: &mut Connection,
    mut filter: impl FnMut(DroneEvent) -> Option<Result<Vec<Value>, DroneIpcError>>,
) -> Result<Vec<Value>, DroneIpcError> {
    let timed = tokio::time::timeout(RESPONSE_TIMEOUT, async {
        loop {
            match conn.next_event().await {
                Some(Ok(event)) => {
                    if let Some(result) = filter(event) {
                        return result;
                    }
                }
                Some(Err(e)) => return Err(e),
                None => {
                    return Err(DroneIpcError::Codec(
                        "event stream ended without response".into(),
                    ));
                }
            }
        }
    })
    .await;
    timed.unwrap_or_else(|_| {
        Err(DroneIpcError::Io(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "drone response timeout",
        )))
    })
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

    #[tokio::test]
    async fn noop_query_session_db_returns_empty_rows() {
        let c = DroneClient::noop();
        let rows = c
            .query_session_db("SELECT 1".to_string())
            .await
            .expect("noop query");
        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn noop_read_signals_returns_empty_signals() {
        let c = DroneClient::noop();
        let signals = c.read_signals("s1".to_string()).await.expect("noop read");
        assert!(signals.is_empty());
    }

    #[tokio::test]
    async fn noop_recover_session_returns_default_recovered() {
        let c = DroneClient::noop();
        let r = c
            .recover_session("s1".to_string())
            .await
            .expect("noop recover");
        assert!(r.snapshot_id.is_none());
        assert!(r.plans.is_empty());
        assert!(r.tasks.is_empty());
        assert!(r.uncertain_tool_invocations.is_empty());
    }

    /// Round-trip `recover_session` via a duplex peer — feed a synthetic
    /// `SessionRecovered` event and assert the client decodes it.
    #[tokio::test]
    async fn recover_session_filters_response_from_event_stream() {
        use std::pin::Pin;
        use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

        type DynRead = Pin<Box<dyn AsyncRead + Send + Unpin>>;
        type DynWrite = Pin<Box<dyn AsyncWrite + Send + Unpin>>;
        let (a, b) = tokio::io::duplex(4096);
        let (a_rd, a_wr) = tokio::io::split(a);
        let (b_rd, mut b_wr) = tokio::io::split(b);
        let conn = Connection::from_streams(
            "/test",
            Box::pin(a_rd) as DynRead,
            Box::pin(a_wr) as DynWrite,
        );
        // Pre-write a Heartbeat (should be skipped) + the SessionRecovered.
        let hb = serde_json::to_string(&DroneEvent::Heartbeat {
            status: runtime_core::HeartbeatStatus::Ok,
            timestamp: 0,
        })
        .unwrap();
        let sr = serde_json::to_string(&DroneEvent::SessionRecovered {
            snapshot_id: Some("snap-1".to_string()),
            state: serde_json::json!({"foo": 1}),
            plans: vec![serde_json::json!({"id": "p1"})],
            tasks: vec![serde_json::json!({"id": "t1", "status": "pending"})],
            uncertain_tool_invocations: vec!["sig-tool-1".to_string()],
        })
        .unwrap();
        b_wr.write_all(format!("{hb}\n{sr}\n").as_bytes())
            .await
            .expect("write peer");
        b_wr.flush().await.expect("flush");
        drop(b_rd);

        let client = DroneClient {
            inner: tokio::sync::Mutex::new(conn),
        };
        let r = client
            .recover_session("s1".to_string())
            .await
            .expect("recover");
        assert_eq!(r.snapshot_id.as_deref(), Some("snap-1"));
        assert_eq!(r.plans.len(), 1);
        assert_eq!(r.tasks.len(), 1);
        assert_eq!(r.uncertain_tool_invocations, vec!["sig-tool-1".to_string()]);
    }

    /// Drone-side rejection alert surfaces as a Codec error to the client.
    #[tokio::test]
    async fn recover_session_alert_surfaces_as_codec_error() {
        use std::pin::Pin;
        use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

        type DynRead = Pin<Box<dyn AsyncRead + Send + Unpin>>;
        type DynWrite = Pin<Box<dyn AsyncWrite + Send + Unpin>>;
        let (a, b) = tokio::io::duplex(4096);
        let (a_rd, a_wr) = tokio::io::split(a);
        let (b_rd, mut b_wr) = tokio::io::split(b);
        let conn = Connection::from_streams(
            "/test",
            Box::pin(a_rd) as DynRead,
            Box::pin(a_wr) as DynWrite,
        );
        let alert = serde_json::to_string(&DroneEvent::Alert {
            level: runtime_core::AlertLevel::Critical,
            message: "recover_session failed: bad session".to_string(),
        })
        .unwrap();
        b_wr.write_all(format!("{alert}\n").as_bytes())
            .await
            .expect("write");
        b_wr.flush().await.expect("flush");
        drop(b_rd);

        let client = DroneClient {
            inner: tokio::sync::Mutex::new(conn),
        };
        let result = client.recover_session("s1".to_string()).await;
        assert!(matches!(result, Err(DroneIpcError::Codec(_))));
    }

    /// Stream that ends without a matching response surfaces as an error.
    #[tokio::test(start_paused = true)]
    async fn recover_session_stream_close_surfaces_as_error_not_hang() {
        use std::pin::Pin;
        use tokio::io::{AsyncRead, AsyncWrite};

        type DynRead = Pin<Box<dyn AsyncRead + Send + Unpin>>;
        type DynWrite = Pin<Box<dyn AsyncWrite + Send + Unpin>>;
        let (a, b) = tokio::io::duplex(64);
        let (a_rd, a_wr) = tokio::io::split(a);
        let (b_rd, b_wr) = tokio::io::split(b);
        let conn = Connection::from_streams(
            "/test",
            Box::pin(a_rd) as DynRead,
            Box::pin(a_wr) as DynWrite,
        );
        drop(b_rd);
        drop(b_wr);

        let client = DroneClient {
            inner: tokio::sync::Mutex::new(conn),
        };
        let task = tokio::spawn(async move { client.recover_session("s1".to_string()).await });
        for _ in 0..6 {
            tokio::time::advance(std::time::Duration::from_millis(700)).await;
            tokio::task::yield_now().await;
        }
        let result = task.await.expect("join");
        assert!(
            matches!(
                result,
                Err(DroneIpcError::Codec(_) | DroneIpcError::Disconnected { .. })
            ),
            "got: {result:?}"
        );
    }

    /// Round-trip via duplex pair — feed a synthetic `QueryResult` event
    /// from the peer side and assert the client returns its rows.
    #[tokio::test]
    async fn query_session_db_filters_response_from_event_stream() {
        use std::pin::Pin;
        use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

        type DynRead = Pin<Box<dyn AsyncRead + Send + Unpin>>;
        type DynWrite = Pin<Box<dyn AsyncWrite + Send + Unpin>>;
        let (a, b) = tokio::io::duplex(4096);
        let (a_rd, a_wr) = tokio::io::split(a);
        let (b_rd, mut b_wr) = tokio::io::split(b);
        let conn = Connection::from_streams(
            "/test",
            Box::pin(a_rd) as DynRead,
            Box::pin(a_wr) as DynWrite,
        );
        // Pre-write a Heartbeat (should be skipped) and then the QueryResult.
        let hb = serde_json::to_string(&DroneEvent::Heartbeat {
            status: runtime_core::HeartbeatStatus::Ok,
            timestamp: 0,
        })
        .unwrap();
        let qr = serde_json::to_string(&DroneEvent::QueryResult {
            rows: vec![serde_json::json!({"id": "x"})],
        })
        .unwrap();
        b_wr.write_all(format!("{hb}\n{qr}\n").as_bytes())
            .await
            .expect("write peer");
        b_wr.flush().await.expect("flush");
        // Drop the peer reader to keep the duplex alive but ignore
        // commands the client sends.
        drop(b_rd);

        let client = DroneClient {
            inner: tokio::sync::Mutex::new(conn),
        };
        let rows = client
            .query_session_db("SELECT id FROM signals".to_string())
            .await
            .expect("query");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("id").and_then(|v| v.as_str()), Some("x"));
    }

    /// Regression test for the M04 IRL drone-IPC bug: two consecutive
    /// `query_session_db` calls must both succeed. Prior to the borrow-not-
    /// move refactor in `Connection::next_event`, the first call moved the
    /// reader into a per-call stream that was dropped at return; the second
    /// call would see an empty reader and immediately surface
    /// `"event stream ended without response"`. Caught by the M04 IRL test
    /// (SQL inspector returned no rows after the smoke session). Pairs with
    /// `connection::tests::next_event_returns_consecutive_events_without_consuming_reader`.
    #[tokio::test]
    async fn query_session_db_succeeds_twice_in_sequence() {
        use std::pin::Pin;
        use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

        type DynRead = Pin<Box<dyn AsyncRead + Send + Unpin>>;
        type DynWrite = Pin<Box<dyn AsyncWrite + Send + Unpin>>;
        let (a, b) = tokio::io::duplex(4096);
        let (a_rd, a_wr) = tokio::io::split(a);
        let (b_rd, mut b_wr) = tokio::io::split(b);
        let conn = Connection::from_streams(
            "/test",
            Box::pin(a_rd) as DynRead,
            Box::pin(a_wr) as DynWrite,
        );
        // Pre-write two QueryResult events — one for each call.
        let qr1 = serde_json::to_string(&DroneEvent::QueryResult {
            rows: vec![serde_json::json!({"call": "first"})],
        })
        .unwrap();
        let qr2 = serde_json::to_string(&DroneEvent::QueryResult {
            rows: vec![serde_json::json!({"call": "second"})],
        })
        .unwrap();
        b_wr.write_all(format!("{qr1}\n{qr2}\n").as_bytes())
            .await
            .expect("write peer");
        b_wr.flush().await.expect("flush");
        drop(b_rd);

        let client = DroneClient {
            inner: tokio::sync::Mutex::new(conn),
        };

        let first = client
            .query_session_db("SELECT 1".to_string())
            .await
            .expect("first query");
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].get("call").and_then(|v| v.as_str()), Some("first"));

        let second = client
            .query_session_db("SELECT 2".to_string())
            .await
            .expect("second query must also succeed (M04 IRL regression)");
        assert_eq!(second.len(), 1);
        assert_eq!(
            second[0].get("call").and_then(|v| v.as_str()),
            Some("second")
        );
    }

    /// Round-trip for `read_signals` with a duplex peer feeding a
    /// `SignalLog` event.
    #[tokio::test]
    async fn read_signals_filters_signal_log_from_event_stream() {
        use std::pin::Pin;
        use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

        type DynRead = Pin<Box<dyn AsyncRead + Send + Unpin>>;
        type DynWrite = Pin<Box<dyn AsyncWrite + Send + Unpin>>;
        let (a, b) = tokio::io::duplex(4096);
        let (a_rd, a_wr) = tokio::io::split(a);
        let (b_rd, mut b_wr) = tokio::io::split(b);
        let conn = Connection::from_streams(
            "/test",
            Box::pin(a_rd) as DynRead,
            Box::pin(a_wr) as DynWrite,
        );
        let sl = serde_json::to_string(&DroneEvent::SignalLog {
            signals: vec![serde_json::json!({"id": "sig-1"})],
        })
        .unwrap();
        b_wr.write_all(format!("{sl}\n").as_bytes())
            .await
            .expect("write peer");
        b_wr.flush().await.expect("flush");
        drop(b_rd);

        let client = DroneClient {
            inner: tokio::sync::Mutex::new(conn),
        };
        let signals = client.read_signals("s1".to_string()).await.expect("read");
        assert_eq!(signals.len(), 1);
    }

    /// Drone-side rejection alert surfaces as a `Codec` error to the
    /// client (filter intentionally re-tags the alert message).
    #[tokio::test]
    async fn query_session_db_rejection_alert_surfaces_as_codec_error() {
        use std::pin::Pin;
        use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

        type DynRead = Pin<Box<dyn AsyncRead + Send + Unpin>>;
        type DynWrite = Pin<Box<dyn AsyncWrite + Send + Unpin>>;
        let (a, b) = tokio::io::duplex(4096);
        let (a_rd, a_wr) = tokio::io::split(a);
        let (b_rd, mut b_wr) = tokio::io::split(b);
        let conn = Connection::from_streams(
            "/test",
            Box::pin(a_rd) as DynRead,
            Box::pin(a_wr) as DynWrite,
        );
        let alert = serde_json::to_string(&DroneEvent::Alert {
            level: runtime_core::AlertLevel::Critical,
            message: "query_session_db rejected: only SELECT".to_string(),
        })
        .unwrap();
        b_wr.write_all(format!("{alert}\n").as_bytes())
            .await
            .expect("write");
        b_wr.flush().await.expect("flush");
        drop(b_rd);

        let client = DroneClient {
            inner: tokio::sync::Mutex::new(conn),
        };
        let result = client.query_session_db("DROP TABLE x".to_string()).await;
        assert!(matches!(result, Err(DroneIpcError::Codec(_))));
    }

    /// `await_event` returns `DroneIpcError::Io(TimedOut)` when the peer
    /// keeps the stream open but never writes a matching response.
    /// Closes the M03.E coverage regression on the 5s timeout branch
    /// in `await_event` — the existing `*_stream_close_*` test exercises
    /// EOF, not timeout. Pattern: `connection.rs::backoff_grows_*`.
    #[tokio::test(start_paused = true)]
    async fn await_event_timeout_when_peer_silent() {
        use std::pin::Pin;
        use tokio::io::{AsyncRead, AsyncWrite};

        type DynRead = Pin<Box<dyn AsyncRead + Send + Unpin>>;
        type DynWrite = Pin<Box<dyn AsyncWrite + Send + Unpin>>;
        let (a, b) = tokio::io::duplex(4096);
        let (a_rd, a_wr) = tokio::io::split(a);
        // Hold both peer halves alive (do NOT drop) — this distinguishes
        // the timeout branch from the EOF branch covered by the
        // `*_stream_close_*` test below.
        let (b_rd, b_wr) = tokio::io::split(b);
        let conn = Connection::from_streams(
            "/test",
            Box::pin(a_rd) as DynRead,
            Box::pin(a_wr) as DynWrite,
        );
        let client = DroneClient {
            inner: tokio::sync::Mutex::new(conn),
        };
        let task =
            tokio::spawn(async move { client.query_session_db("SELECT 1".to_string()).await });
        // Advance well past the 5s `RESPONSE_TIMEOUT` so the `tokio::time::
        // timeout` future inside `await_event` resolves to its
        // elapsed-Err branch.
        for _ in 0..7 {
            tokio::time::advance(std::time::Duration::from_secs(1)).await;
            tokio::task::yield_now().await;
        }
        let result = task.await.expect("join");
        assert!(
            matches!(
                &result,
                Err(DroneIpcError::Io(e)) if e.kind() == std::io::ErrorKind::TimedOut
            ),
            "expected Io(TimedOut), got {result:?}"
        );
        // Explicit drop ordering: peer halves outlive the spawned client
        // task so the duplex stream stays open during the timeout window.
        drop(b_rd);
        drop(b_wr);
    }

    /// Stream that ends without a matching response surfaces as an
    /// error (Codec or Disconnected depending on whether the send
    /// succeeded before the read EOFs) rather than hanging the call.
    #[tokio::test(start_paused = true)]
    async fn read_signals_stream_close_surfaces_as_error_not_hang() {
        use std::pin::Pin;
        use tokio::io::{AsyncRead, AsyncWrite};

        type DynRead = Pin<Box<dyn AsyncRead + Send + Unpin>>;
        type DynWrite = Pin<Box<dyn AsyncWrite + Send + Unpin>>;
        let (a, b) = tokio::io::duplex(64);
        let (a_rd, a_wr) = tokio::io::split(a);
        let (b_rd, b_wr) = tokio::io::split(b);
        let conn = Connection::from_streams(
            "/test",
            Box::pin(a_rd) as DynRead,
            Box::pin(a_wr) as DynWrite,
        );
        // Drop the peer halves so the stream EOFs immediately.
        drop(b_rd);
        drop(b_wr);

        let client = DroneClient {
            inner: tokio::sync::Mutex::new(conn),
        };
        let task = tokio::spawn(async move { client.read_signals("s1".to_string()).await });
        // Advance well past the cumulative reconnect backoff so the send
        // exhausts retries without us having to wait wall-clock time.
        for _ in 0..6 {
            tokio::time::advance(std::time::Duration::from_millis(700)).await;
            tokio::task::yield_now().await;
        }
        let result = task.await.expect("join");
        assert!(
            matches!(
                result,
                Err(DroneIpcError::Codec(_) | DroneIpcError::Disconnected { .. })
            ),
            "got: {result:?}"
        );
    }

    /// TD-002 — `read_signals` must succeed twice in sequence on the
    /// same client. Mirrors `query_session_db_succeeds_twice_in_sequence`
    /// for the second of the three drone-IPC read methods that compose
    /// `send_with_reconnect` + `await_event` + `next_event`. Pins the
    /// borrow-not-move multi-call invariant per-method (defense in depth
    /// over `connection::next_event_returns_consecutive_events_without_consuming_reader`).
    #[tokio::test]
    async fn read_signals_succeeds_twice_in_sequence() {
        use std::pin::Pin;
        use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

        type DynRead = Pin<Box<dyn AsyncRead + Send + Unpin>>;
        type DynWrite = Pin<Box<dyn AsyncWrite + Send + Unpin>>;
        let (a, b) = tokio::io::duplex(4096);
        let (a_rd, a_wr) = tokio::io::split(a);
        let (b_rd, mut b_wr) = tokio::io::split(b);
        let conn = Connection::from_streams(
            "/test",
            Box::pin(a_rd) as DynRead,
            Box::pin(a_wr) as DynWrite,
        );
        let sl1 = serde_json::to_string(&DroneEvent::SignalLog {
            signals: vec![serde_json::json!({"call": "first"})],
        })
        .unwrap();
        let sl2 = serde_json::to_string(&DroneEvent::SignalLog {
            signals: vec![serde_json::json!({"call": "second"})],
        })
        .unwrap();
        b_wr.write_all(format!("{sl1}\n{sl2}\n").as_bytes())
            .await
            .expect("write peer");
        b_wr.flush().await.expect("flush");
        drop(b_rd);

        let client = DroneClient {
            inner: tokio::sync::Mutex::new(conn),
        };

        let first = client
            .read_signals("s1".to_string())
            .await
            .expect("first read_signals");
        assert_eq!(first[0].get("call").and_then(|v| v.as_str()), Some("first"));
        let second = client
            .read_signals("s1".to_string())
            .await
            .expect("second read_signals must also succeed (TD-002)");
        assert_eq!(
            second[0].get("call").and_then(|v| v.as_str()),
            Some("second")
        );
    }

    /// TD-002 — `recover_session` must succeed twice in sequence on the
    /// same client. The third drone-IPC read method composing the
    /// borrow-not-move `next_event` path; closes the TD-002 per-method
    /// coverage gap alongside `read_signals_succeeds_twice_in_sequence`.
    #[tokio::test]
    async fn recover_session_succeeds_twice_in_sequence() {
        use std::pin::Pin;
        use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

        type DynRead = Pin<Box<dyn AsyncRead + Send + Unpin>>;
        type DynWrite = Pin<Box<dyn AsyncWrite + Send + Unpin>>;
        let (a, b) = tokio::io::duplex(4096);
        let (a_rd, a_wr) = tokio::io::split(a);
        let (b_rd, mut b_wr) = tokio::io::split(b);
        let conn = Connection::from_streams(
            "/test",
            Box::pin(a_rd) as DynRead,
            Box::pin(a_wr) as DynWrite,
        );
        let sr1 = serde_json::to_string(&DroneEvent::SessionRecovered {
            snapshot_id: Some("snap-first".to_string()),
            state: serde_json::json!({}),
            plans: vec![],
            tasks: vec![],
            uncertain_tool_invocations: vec![],
        })
        .unwrap();
        let sr2 = serde_json::to_string(&DroneEvent::SessionRecovered {
            snapshot_id: Some("snap-second".to_string()),
            state: serde_json::json!({}),
            plans: vec![],
            tasks: vec![],
            uncertain_tool_invocations: vec![],
        })
        .unwrap();
        b_wr.write_all(format!("{sr1}\n{sr2}\n").as_bytes())
            .await
            .expect("write peer");
        b_wr.flush().await.expect("flush");
        drop(b_rd);

        let client = DroneClient {
            inner: tokio::sync::Mutex::new(conn),
        };

        let first = client
            .recover_session("s1".to_string())
            .await
            .expect("first recover_session");
        assert_eq!(first.snapshot_id.as_deref(), Some("snap-first"));
        let second = client
            .recover_session("s1".to_string())
            .await
            .expect("second recover_session must also succeed (TD-002)");
        assert_eq!(second.snapshot_id.as_deref(), Some("snap-second"));
    }

    /// M04 🟡 per-module coverage close — the `read_signals` Codec-on-
    /// rejection-alert error branch. Mirrors
    /// `query_session_db_rejection_alert_surfaces_as_codec_error` for the
    /// `read_signals` filter arm (`Alert` message starting with
    /// `"read_signals"` re-tagged as `DroneIpcError::Codec`), the
    /// uncovered error path lifting `drone_ipc/client.rs` within the
    /// runtime-main ≥95 gate.
    #[tokio::test]
    async fn read_signals_rejection_alert_surfaces_as_codec_error() {
        use std::pin::Pin;
        use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

        type DynRead = Pin<Box<dyn AsyncRead + Send + Unpin>>;
        type DynWrite = Pin<Box<dyn AsyncWrite + Send + Unpin>>;
        let (a, b) = tokio::io::duplex(4096);
        let (a_rd, a_wr) = tokio::io::split(a);
        let (b_rd, mut b_wr) = tokio::io::split(b);
        let conn = Connection::from_streams(
            "/test",
            Box::pin(a_rd) as DynRead,
            Box::pin(a_wr) as DynWrite,
        );
        let alert = serde_json::to_string(&DroneEvent::Alert {
            level: runtime_core::AlertLevel::Critical,
            message: "read_signals rejected: unknown session".to_string(),
        })
        .unwrap();
        b_wr.write_all(format!("{alert}\n").as_bytes())
            .await
            .expect("write");
        b_wr.flush().await.expect("flush");
        drop(b_rd);

        let client = DroneClient {
            inner: tokio::sync::Mutex::new(conn),
        };
        let result = client.read_signals("s1".to_string()).await;
        assert!(matches!(result, Err(DroneIpcError::Codec(_))));
    }
}
