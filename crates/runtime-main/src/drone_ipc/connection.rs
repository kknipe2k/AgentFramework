//! Connection state machine + reconnect policy for the main-side drone IPC.
//!
//! The [`open`] function is a cfg-platform OS-call wrapper (`UnixStream` on
//! Linux/macOS; `NamedPipeClient` on Windows) and is excluded from the
//! ≥95% coverage gate per `CLAUDE.md` §5 — it is structurally infeasible
//! to test cross-platform. The testable seam
//! [`Connection::from_streams`] accepts any pair of `AsyncRead`+
//! `AsyncWrite` halves; unit tests inject `tokio::io::duplex` pairs.

use std::pin::Pin;
use std::time::Duration;

use futures::stream::Stream;
use futures::{SinkExt, StreamExt};
use runtime_core::drone::{DroneCommand, DroneEvent};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::sleep;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec, LinesCodecError};

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
#[cfg(unix)]
use tokio::net::UnixStream;

/// Maximum number of send attempts before surfacing
/// [`DroneIpcError::Disconnected`].
pub const MAX_RETRIES: u32 = 5;
/// Base backoff between attempts. Backoff doubles each retry up to
/// `BASE_BACKOFF * 2^(MAX_RETRIES - 2)` (no sleep after the final attempt).
pub const BASE_BACKOFF: Duration = Duration::from_millis(200);

/// Errors raised by the main-side drone IPC client.
#[derive(Debug, Error)]
pub enum DroneIpcError {
    /// Underlying I/O error (socket / named pipe).
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// Framing-level codec error (e.g. line length exceeded).
    #[error("codec: {0}")]
    Codec(String),
    /// Send failed after the configured retry budget.
    #[error("disconnected after {retries} retries")]
    Disconnected {
        /// Number of attempts that failed.
        retries: u32,
    },
    /// JSON (de)serialization error.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<LinesCodecError> for DroneIpcError {
    fn from(err: LinesCodecError) -> Self {
        match err {
            LinesCodecError::Io(io) => Self::Io(io),
            LinesCodecError::MaxLineLengthExceeded => {
                Self::Codec("max line length exceeded".into())
            }
        }
    }
}

type DynRead = Pin<Box<dyn AsyncRead + Send + Unpin>>;
type DynWrite = Pin<Box<dyn AsyncWrite + Send + Unpin>>;

/// Connection mode. `Active` carries real socket halves; `Noop` short-
/// circuits all sends to `Ok(())` and yields an empty event stream — used
/// by tests that exercise the SDK loop without wanting a real drone.
pub enum Mode {
    Active,
    Noop,
}

/// Internal connection state. `DroneClient` wraps this in a `Mutex`.
pub struct Connection {
    addr: String,
    writer: Option<FramedWrite<DynWrite, LinesCodec>>,
    reader: Option<FramedRead<DynRead, LinesCodec>>,
    mode: Mode,
}

impl Connection {
    /// Open a real connection to the drone at `addr`.
    ///
    /// `addr` is a filesystem path on Unix or a named-pipe name on
    /// Windows (e.g. `\\.\pipe\drone-abc`).
    ///
    /// # Errors
    ///
    /// Returns [`DroneIpcError::Io`] if the underlying open fails.
    pub async fn connect(addr: &str) -> Result<Self, DroneIpcError> {
        let (rd, wr) = open(addr).await?;
        Ok(Self::from_streams(addr, rd, wr))
    }

    /// Test seam — construct from already-opened halves. Unit tests pass
    /// `tokio::io::duplex` pairs.
    pub fn from_streams(addr: &str, rd: DynRead, wr: DynWrite) -> Self {
        Self {
            addr: addr.to_string(),
            writer: Some(FramedWrite::new(wr, LinesCodec::new())),
            reader: Some(FramedRead::new(rd, LinesCodec::new())),
            mode: Mode::Active,
        }
    }

    /// No-op constructor. `send` returns `Ok(())` immediately; `events`
    /// yields nothing. Test affordance for paths that don't exercise the
    /// drone (e.g. `tests/sdk_cancellation.rs`).
    pub fn noop() -> Self {
        Self {
            addr: String::new(),
            writer: None,
            reader: None,
            mode: Mode::Noop,
        }
    }

    /// Take the inbound `DroneEvent` stream. Single-consumer; subsequent
    /// calls return an empty stream.
    pub fn take_event_stream(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<DroneEvent, DroneIpcError>> + Send>> {
        if let Some(reader) = self.reader.take() {
            Box::pin(futures::stream::unfold(reader, |mut r| async move {
                match r.next().await {
                    Some(Ok(line)) => {
                        let parsed =
                            serde_json::from_str::<DroneEvent>(&line).map_err(DroneIpcError::Json);
                        Some((parsed, r))
                    }
                    Some(Err(e)) => Some((Err(DroneIpcError::from(e)), r)),
                    None => None,
                }
            }))
        } else {
            Box::pin(futures::stream::empty())
        }
    }

    /// Send a `DroneCommand` with exponential-backoff retry on transport
    /// errors. Surfaces [`DroneIpcError::Disconnected`] after
    /// [`MAX_RETRIES`] failed attempts.
    ///
    /// Backoff: `BASE_BACKOFF * 2^attempt` between attempts. There is no
    /// trailing sleep after the final attempt.
    ///
    /// # Errors
    ///
    /// - [`DroneIpcError::Json`] if `cmd` cannot serialize (programmer
    ///   bug; not retried).
    /// - [`DroneIpcError::Disconnected`] after exhausting retries.
    pub async fn send_with_reconnect(&mut self, cmd: DroneCommand) -> Result<(), DroneIpcError> {
        if matches!(self.mode, Mode::Noop) {
            return Ok(());
        }
        let line = serde_json::to_string(&cmd)?;
        for attempt in 0..MAX_RETRIES {
            match self.send_line(&line).await {
                Ok(()) => return Ok(()),
                Err(DroneIpcError::Json(e)) => return Err(DroneIpcError::Json(e)),
                Err(_) => {
                    if attempt == MAX_RETRIES - 1 {
                        break;
                    }
                    sleep(BASE_BACKOFF * 2u32.pow(attempt)).await;
                    let _ = self.reconnect().await;
                }
            }
        }
        Err(DroneIpcError::Disconnected {
            retries: MAX_RETRIES,
        })
    }

    async fn send_line(&mut self, line: &str) -> Result<(), DroneIpcError> {
        let writer = self.writer.as_mut().ok_or_else(|| {
            DroneIpcError::Io(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "no writer",
            ))
        })?;
        writer.send(line.to_string()).await?;
        Ok(())
    }

    async fn reconnect(&mut self) -> Result<(), DroneIpcError> {
        let (rd, wr) = open(&self.addr).await?;
        self.writer = Some(FramedWrite::new(wr, LinesCodec::new()));
        self.reader = Some(FramedRead::new(rd, LinesCodec::new()));
        Ok(())
    }
}

// ── Cfg-platform OS-call wrapper. Excluded from coverage gate per ──
// ── CLAUDE.md §5 (drone_ipc connection.rs is the runtime-main equivalent ─
// ── of the M01.C drone shutdown.rs holdout). ────────────────────────────

#[cfg(unix)]
async fn open(addr: &str) -> Result<(DynRead, DynWrite), DroneIpcError> {
    let stream = UnixStream::connect(addr).await?;
    let (rd, wr) = stream.into_split();
    Ok((Box::pin(rd) as DynRead, Box::pin(wr) as DynWrite))
}

#[cfg(windows)]
#[allow(
    clippy::unused_async,
    reason = "cfg(unix) sibling awaits UnixStream::connect; this Windows variant does not but the call site uniformly `.await`s for cross-platform shape parity"
)]
async fn open(addr: &str) -> Result<(DynRead, DynWrite), DroneIpcError> {
    let pipe: NamedPipeClient = ClientOptions::new().open(addr)?;
    let (rd, wr) = tokio::io::split(pipe);
    Ok((Box::pin(rd) as DynRead, Box::pin(wr) as DynWrite))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    fn cmd() -> DroneCommand {
        DroneCommand::SnapshotNow {
            reason: "test".into(),
            state_json: serde_json::json!({}),
        }
    }

    fn dyn_pair(buf: usize) -> ((DynRead, DynWrite), (DynRead, DynWrite)) {
        let (a, b) = tokio::io::duplex(buf);
        let (a_rd, a_wr) = tokio::io::split(a);
        let (b_rd, b_wr) = tokio::io::split(b);
        (
            (Box::pin(a_rd) as DynRead, Box::pin(a_wr) as DynWrite),
            (Box::pin(b_rd) as DynRead, Box::pin(b_wr) as DynWrite),
        )
    }

    #[tokio::test]
    async fn successful_send_reaches_peer() {
        let ((client_rd, client_wr), (mut peer_rd, _peer_wr)) = dyn_pair(1024);
        let mut conn = Connection::from_streams("/test", client_rd, client_wr);
        conn.send_with_reconnect(cmd()).await.expect("send");
        let mut buf = vec![0u8; 256];
        let n = peer_rd.read(&mut buf).await.expect("peer read");
        let s = std::str::from_utf8(&buf[..n]).unwrap();
        assert!(s.contains("snapshot_now"));
    }

    #[tokio::test]
    async fn noop_send_returns_ok_without_io() {
        let mut conn = Connection::noop();
        conn.send_with_reconnect(cmd()).await.expect("noop ok");
    }

    #[tokio::test]
    async fn noop_event_stream_is_empty() {
        let mut conn = Connection::noop();
        let mut s = conn.take_event_stream();
        assert!(s.next().await.is_none());
    }

    #[tokio::test(start_paused = true)]
    async fn exhausts_retries_then_disconnected() {
        // Drop the peer so writes immediately broken-pipe.
        let ((client_rd, client_wr), peer) = dyn_pair(8);
        drop(peer);
        let mut conn = Connection::from_streams("/nonexistent-path-xyz", client_rd, client_wr);
        // Drive the send while we advance time past each backoff.
        let send_fut = conn.send_with_reconnect(cmd());
        tokio::pin!(send_fut);
        // Advance well past the cumulative backoff (200+400+800+1600 = 3000ms).
        for _ in 0..6 {
            tokio::time::advance(Duration::from_millis(700)).await;
            tokio::task::yield_now().await;
        }
        let result = (&mut send_fut).await;
        assert!(
            matches!(
                result,
                Err(DroneIpcError::Disconnected {
                    retries: MAX_RETRIES
                })
            ),
            "got: {result:?}"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn backoff_grows_exponentially_between_attempts() {
        let ((client_rd, client_wr), peer) = dyn_pair(8);
        drop(peer);
        let mut conn = Connection::from_streams("/nonexistent-path-xyz", client_rd, client_wr);
        let start = tokio::time::Instant::now();
        // Drive to completion. Total sleep = 200+400+800+1600 = 3000ms (4 sleeps for 5 attempts).
        let task = tokio::spawn(async move { conn.send_with_reconnect(cmd()).await });
        // Advance 3001ms past the cumulative backoff so the task can complete.
        for _ in 0..5 {
            tokio::time::advance(Duration::from_millis(700)).await;
            tokio::task::yield_now().await;
        }
        let result = task.await.expect("join");
        let elapsed = tokio::time::Instant::now() - start;
        assert!(
            elapsed >= Duration::from_millis(2900),
            "expected ≥2900ms cumulative backoff, got {elapsed:?}"
        );
        assert!(matches!(result, Err(DroneIpcError::Disconnected { .. })));
    }

    #[tokio::test]
    async fn take_event_stream_returns_empty_after_first_take() {
        let ((client_rd, client_wr), peer) = dyn_pair(64);
        let mut conn = Connection::from_streams("/test", client_rd, client_wr);
        let _first = conn.take_event_stream();
        let mut second = conn.take_event_stream();
        assert!(second.next().await.is_none());
        drop(peer);
    }
}
