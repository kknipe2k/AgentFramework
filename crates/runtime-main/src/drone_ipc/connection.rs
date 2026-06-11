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

    /// Whether this connection is in noop mode. Callers that want to
    /// short-circuit a read/write before allocating a request can check
    /// this to skip the IPC round-trip.
    #[must_use]
    pub const fn is_noop(&self) -> bool {
        matches!(self.mode, Mode::Noop)
    }

    /// Take the inbound `DroneEvent` stream. Single-consumer; subsequent
    /// calls return an empty stream.
    ///
    /// Use [`Self::next_event`] for request-response paths (e.g.
    /// `query_session_db`, `read_signals`, `recover_session`): those need
    /// to call repeatedly across the connection's lifetime, and taking the
    /// stream would dispose of the reader after one call. Keep this method
    /// for long-lived subscriptions where the stream's lifetime equals the
    /// connection's (the renderer's `events()` subscription pattern).
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

    /// Read one event from the reader half, borrowing rather than moving.
    /// Unlike [`Self::take_event_stream`], the reader stays in the
    /// connection across calls — callers can invoke `next_event` repeatedly
    /// across the connection's lifetime. Returns `None` when the underlying
    /// stream is exhausted (e.g. drone disconnected); on exhaustion, the
    /// reader is dropped so subsequent calls are fast no-ops.
    ///
    /// Returns `None` on a noop connection (no reader installed).
    ///
    /// Used by request-response paths: `query_session_db`, `read_signals`,
    /// `recover_session`. Each request sends a command and then loops on
    /// `next_event` filtering for the matching response. Without this
    /// borrow-not-move pattern, the second call to any request-response
    /// method would see an empty reader and immediately surface
    /// `"event stream ended without response"` — the M04 IRL-test bug.
    pub async fn next_event(&mut self) -> Option<Result<DroneEvent, DroneIpcError>> {
        let reader = self.reader.as_mut()?;
        match reader.next().await {
            Some(Ok(line)) => {
                Some(serde_json::from_str::<DroneEvent>(&line).map_err(DroneIpcError::Json))
            }
            Some(Err(e)) => Some(Err(DroneIpcError::from(e))),
            None => {
                // Reader exhausted; drop it so future calls return None
                // immediately without re-polling the closed stream.
                self.reader = None;
                None
            }
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

    #[tokio::test]
    async fn noop_next_event_returns_none() {
        let mut conn = Connection::noop();
        assert!(conn.next_event().await.is_none());
        // Multiple calls are safe — still None.
        assert!(conn.next_event().await.is_none());
    }

    #[tokio::test]
    async fn next_event_returns_consecutive_events_without_consuming_reader() {
        use tokio::io::AsyncWriteExt;
        // Pair: peer writer feeds two heartbeat lines; client reads via
        // next_event. The reader must survive across calls — the M04 IRL
        // bug was every read disposing of the reader.
        //
        // Asserts only the multi-call invariant (two reads succeed in
        // sequence). EOF-on-drop behavior is covered by the
        // `*_stream_close_*` tests in client.rs which run with paused
        // tokio time; reproducing it here would require either explicit
        // timeout wrapping or paused-time machinery to avoid a hang on
        // duplex EOF-propagation semantics with the unused peer-rd half.
        let ((client_rd, client_wr), (_peer_rd, mut peer_wr)) = dyn_pair(256);
        let mut conn = Connection::from_streams("/test", client_rd, client_wr);

        let hb1 = serde_json::to_string(&DroneEvent::Heartbeat {
            status: runtime_core::HeartbeatStatus::Ok,
            timestamp: 1,
        })
        .expect("ser1");
        let hb2 = serde_json::to_string(&DroneEvent::Heartbeat {
            status: runtime_core::HeartbeatStatus::Ok,
            timestamp: 2,
        })
        .expect("ser2");
        peer_wr
            .write_all(format!("{hb1}\n{hb2}\n").as_bytes())
            .await
            .expect("peer write");

        let first = conn.next_event().await.expect("first").expect("ok1");
        let second = conn.next_event().await.expect("second").expect("ok2");

        assert!(matches!(first, DroneEvent::Heartbeat { timestamp: 1, .. }));
        assert!(matches!(second, DroneEvent::Heartbeat { timestamp: 2, .. }));
        // Reader stays installed across consecutive reads — that's the
        // invariant the M04 IRL bug violated. (Old `take_event_stream`
        // moved the reader into a per-call stream that was dropped at
        // return; the second call would see an empty reader.)
        assert!(
            conn.reader.is_some(),
            "reader must persist across next_event calls",
        );
    }

    /// Mirrors `runtime_core::MAX_IPC_FRAME_BYTES` as a literal on
    /// purpose: the tests pin the agreed 4 MiB boundary VALUE
    /// (delimiter-exclusive per tokio-util 0.7.18 `LinesCodec`), so a
    /// silent change to the production constant fails here. TD-053.
    const CAP: usize = 4 * 1024 * 1024;

    /// TD-053 adversarial: a hostile/corrupted drone writing `CAP + 1`
    /// bytes with NO newline must surface the typed
    /// `DroneIpcError::Codec` (the dead `MaxLineLengthExceeded` arm at
    /// the `From` impl goes live) instead of buffering unbounded.
    ///
    /// RED (pre-impl): `next_event` never returns (the uncapped codec
    /// waits for a newline forever) and the timeout below fails. The
    /// peer write half stays OPEN on purpose: at EOF `decode_eof` would
    /// deliver the blob as a line and today's failure would be a Json
    /// error, not the buffering bug.
    #[tokio::test]
    async fn next_event_surfaces_codec_error_on_oversize_unterminated_line() {
        use tokio::io::AsyncWriteExt;
        let ((client_rd, client_wr), (_peer_rd, mut peer_wr)) = dyn_pair(64 * 1024);
        let mut conn = Connection::from_streams("/test", client_rd, client_wr);
        let writer = tokio::spawn(async move {
            let blob = vec![b'x'; CAP + 1];
            peer_wr.write_all(&blob).await.expect("peer write");
            peer_wr.flush().await.expect("peer flush");
            peer_wr
        });
        let result = tokio::time::timeout(Duration::from_secs(5), conn.next_event())
            .await
            .expect(
                "next_event hung on an oversize unterminated line — \
                 the unbounded LinesCodec buffered it (TD-053)",
            );
        let err = result
            .expect("expected Some(Err(..))")
            .expect_err("an oversize line must error, not decode");
        assert!(
            matches!(&err, DroneIpcError::Codec(m) if m.contains("max line length")),
            "expected the typed max-length Codec error, got {err:?}"
        );
        let _ = writer.await;
    }

    /// PIN — green at red by design (rider 3): an event line of EXACTLY
    /// `CAP` content bytes + `\n` decodes (delimiter-exclusive, rider
    /// 2). Pins that the cap clips nothing legitimate.
    #[tokio::test]
    async fn next_event_decodes_a_line_at_exactly_the_cap() {
        use tokio::io::AsyncWriteExt;
        let ((client_rd, client_wr), (_peer_rd, mut peer_wr)) = dyn_pair(64 * 1024);
        let mut conn = Connection::from_streams("/test", client_rd, client_wr);

        let base = serde_json::to_string(&DroneEvent::Alert {
            level: runtime_core::AlertLevel::Warn,
            message: String::new(),
        })
        .expect("serialize base")
        .len();
        let pad = "x".repeat(CAP - base);
        let event = DroneEvent::Alert {
            level: runtime_core::AlertLevel::Warn,
            message: pad,
        };
        let line = serde_json::to_string(&event).expect("serialize");
        assert_eq!(line.len(), CAP, "fixture bug: line must be exactly CAP");

        let writer = tokio::spawn(async move {
            peer_wr
                .write_all(format!("{line}\n").as_bytes())
                .await
                .expect("peer write");
            peer_wr.flush().await.expect("peer flush");
            peer_wr
        });
        let received = tokio::time::timeout(Duration::from_secs(10), conn.next_event())
            .await
            .expect("at-cap line did not decode within 10s")
            .expect("some")
            .expect("ok");
        assert_eq!(received, event);
        let _ = writer.await;
    }
}
