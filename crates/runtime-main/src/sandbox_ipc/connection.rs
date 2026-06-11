//! Connection state machine + reconnect policy for the main-side sandbox IPC.
//!
//! Parallel to [`crate::drone_ipc::connection`]: the [`open`] function is
//! a cfg-platform OS-call wrapper (`UnixStream` on Unix; `NamedPipeClient`
//! on Windows) and is excluded from the ≥95% coverage gate per
//! `CLAUDE.md` §5 — structurally infeasible to test cross-platform.
//! The testable seam [`Connection::from_streams`] accepts any pair of
//! `AsyncRead` + `AsyncWrite` halves; unit tests inject
//! `tokio::io::duplex` pairs.
//!
//! Per gotcha #72 the [`Connection::next_response`] method borrows the
//! reader rather than moving it, so request-response paths can issue
//! multiple validations across a single connection's lifetime. The
//! `validate_succeeds_twice_in_sequence` test in
//! [`super::client`] pins the multi-call invariant from day 1 (the
//! M04 IRL drone bug applied retroactively — gotcha #69).

use std::pin::Pin;
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use runtime_core::MAX_IPC_FRAME_BYTES;
use runtime_sandbox::protocol::{SandboxRequest, SandboxResponse};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::sleep;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec, LinesCodecError};

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
#[cfg(unix)]
use tokio::net::UnixStream;

/// Maximum number of send attempts before surfacing
/// [`SandboxIpcError::Disconnected`].
pub const MAX_RETRIES: u32 = 5;
/// Base backoff between attempts. Backoff doubles each retry.
pub const BASE_BACKOFF: Duration = Duration::from_millis(200);

/// Errors raised by the main-side sandbox IPC client.
#[derive(Debug, Error)]
pub enum SandboxIpcError {
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

impl From<LinesCodecError> for SandboxIpcError {
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
/// circuits all sends to `Ok(())` and yields no responses — used by
/// tests / paths that don't exercise a real sandbox.
pub enum Mode {
    /// Backed by real socket / pipe halves.
    Active,
    /// Test-affordance — `send` returns `Ok(())`; `next_response`
    /// returns `None`.
    Noop,
}

/// Internal connection state. `SandboxClient` wraps this in a `Mutex`.
pub struct Connection {
    addr: String,
    writer: Option<FramedWrite<DynWrite, LinesCodec>>,
    reader: Option<FramedRead<DynRead, LinesCodec>>,
    mode: Mode,
}

impl Connection {
    /// Open a real connection to the sandbox at `addr`.
    ///
    /// # Errors
    ///
    /// Returns [`SandboxIpcError::Io`] if the underlying open fails.
    pub async fn connect(addr: &str) -> Result<Self, SandboxIpcError> {
        let (rd, wr) = open(addr).await?;
        Ok(Self::from_streams(addr, rd, wr))
    }

    /// Test seam — construct from already-opened halves. Unit tests
    /// pass `tokio::io::duplex` pairs.
    pub fn from_streams(addr: &str, rd: DynRead, wr: DynWrite) -> Self {
        Self {
            addr: addr.to_string(),
            writer: Some(FramedWrite::new(
                wr,
                LinesCodec::new_with_max_length(MAX_IPC_FRAME_BYTES),
            )),
            reader: Some(FramedRead::new(
                rd,
                LinesCodec::new_with_max_length(MAX_IPC_FRAME_BYTES),
            )),
            mode: Mode::Active,
        }
    }

    /// No-op constructor.
    #[must_use]
    pub fn noop() -> Self {
        Self {
            addr: String::new(),
            writer: None,
            reader: None,
            mode: Mode::Noop,
        }
    }

    /// Whether this connection is in noop mode.
    #[must_use]
    pub const fn is_noop(&self) -> bool {
        matches!(self.mode, Mode::Noop)
    }

    /// Read one response from the reader half, borrowing rather than
    /// moving. Reader stays installed across calls — callers can invoke
    /// `next_response` repeatedly across the connection's lifetime.
    /// Returns `None` when the underlying stream is exhausted; on
    /// exhaustion the reader is dropped so subsequent calls are fast
    /// no-ops. Returns `None` on a noop connection.
    ///
    /// Per gotcha #72 — this is the borrow-not-move counterpart to
    /// `drone_ipc`'s `Connection::next_event`. Applied from day 1 so the
    /// `validate_succeeds_twice_in_sequence` multi-call invariant
    /// (gotcha #69) holds without retrofit.
    pub async fn next_response(&mut self) -> Option<Result<SandboxResponse, SandboxIpcError>> {
        let reader = self.reader.as_mut()?;
        match reader.next().await {
            Some(Ok(line)) => {
                Some(serde_json::from_str::<SandboxResponse>(&line).map_err(SandboxIpcError::Json))
            }
            Some(Err(e)) => Some(Err(SandboxIpcError::from(e))),
            None => {
                self.reader = None;
                None
            }
        }
    }

    /// Send a [`SandboxRequest`] with exponential-backoff retry on
    /// transport errors. Surfaces [`SandboxIpcError::Disconnected`]
    /// after [`MAX_RETRIES`] failed attempts.
    ///
    /// # Errors
    ///
    /// - [`SandboxIpcError::Json`] if `req` cannot serialize.
    /// - [`SandboxIpcError::Disconnected`] after exhausting retries.
    pub async fn send_with_reconnect(
        &mut self,
        req: SandboxRequest,
    ) -> Result<(), SandboxIpcError> {
        if matches!(self.mode, Mode::Noop) {
            return Ok(());
        }
        let line = serde_json::to_string(&req)?;
        for attempt in 0..MAX_RETRIES {
            match self.send_line(&line).await {
                Ok(()) => return Ok(()),
                Err(SandboxIpcError::Json(e)) => return Err(SandboxIpcError::Json(e)),
                Err(_) => {
                    if attempt == MAX_RETRIES - 1 {
                        break;
                    }
                    sleep(BASE_BACKOFF * 2u32.pow(attempt)).await;
                    let _ = self.reconnect().await;
                }
            }
        }
        Err(SandboxIpcError::Disconnected {
            retries: MAX_RETRIES,
        })
    }

    async fn send_line(&mut self, line: &str) -> Result<(), SandboxIpcError> {
        let writer = self.writer.as_mut().ok_or_else(|| {
            SandboxIpcError::Io(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "no writer",
            ))
        })?;
        writer.send(line.to_string()).await?;
        Ok(())
    }

    async fn reconnect(&mut self) -> Result<(), SandboxIpcError> {
        let (rd, wr) = open(&self.addr).await?;
        self.writer = Some(FramedWrite::new(
            wr,
            LinesCodec::new_with_max_length(MAX_IPC_FRAME_BYTES),
        ));
        self.reader = Some(FramedRead::new(
            rd,
            LinesCodec::new_with_max_length(MAX_IPC_FRAME_BYTES),
        ));
        Ok(())
    }
}

// ── Cfg-platform OS-call wrapper. Excluded from coverage gate per ──
// ── CLAUDE.md §5 — sandbox_ipc/connection.rs is the runtime-main ──
// ── equivalent of drone_ipc/connection.rs's open() holdout. ─────────

#[cfg(unix)]
async fn open(addr: &str) -> Result<(DynRead, DynWrite), SandboxIpcError> {
    let stream = UnixStream::connect(addr).await?;
    let (rd, wr) = stream.into_split();
    Ok((Box::pin(rd) as DynRead, Box::pin(wr) as DynWrite))
}

#[cfg(windows)]
#[allow(
    clippy::unused_async,
    reason = "cfg(unix) sibling awaits UnixStream::connect; this Windows variant does not but the call site uniformly `.await`s for cross-platform shape parity"
)]
async fn open(addr: &str) -> Result<(DynRead, DynWrite), SandboxIpcError> {
    let pipe: NamedPipeClient = ClientOptions::new().open(addr)?;
    let (rd, wr) = tokio::io::split(pipe);
    Ok((Box::pin(rd) as DynRead, Box::pin(wr) as DynWrite))
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::generated::capability::{
        CapabilityDeclaration, CapabilityKind, CapabilityScope, GlobPattern, ResourceName,
        SideEffectClass,
    };
    use runtime_sandbox::validator::ValidationResult;
    use std::str::FromStr;
    use tokio::io::AsyncReadExt;

    fn declaration() -> CapabilityDeclaration {
        CapabilityDeclaration {
            kind: CapabilityKind::Read,
            resource: ResourceName::from_str("*.md").expect("resource"),
            scope: CapabilityScope::Glob(GlobPattern::from_str("*.md").expect("glob")),
            side_effect_class: SideEffectClass::Pure,
        }
    }

    fn validate_request() -> SandboxRequest {
        SandboxRequest::ValidateArtifact {
            artifact_code: "let x = 1;".to_string(),
            declaration: declaration(),
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
        conn.send_with_reconnect(validate_request())
            .await
            .expect("send");
        let mut buf = vec![0u8; 256];
        let n = peer_rd.read(&mut buf).await.expect("peer read");
        let s = std::str::from_utf8(&buf[..n]).unwrap();
        assert!(s.contains("validate_artifact"), "got {s}");
    }

    #[tokio::test]
    async fn noop_send_returns_ok_without_io() {
        let mut conn = Connection::noop();
        conn.send_with_reconnect(validate_request())
            .await
            .expect("noop ok");
    }

    #[tokio::test]
    async fn noop_next_response_returns_none() {
        let mut conn = Connection::noop();
        assert!(conn.next_response().await.is_none());
        // Multiple calls are safe — still None.
        assert!(conn.next_response().await.is_none());
    }

    #[tokio::test(start_paused = true)]
    async fn exhausts_retries_then_disconnected() {
        let ((client_rd, client_wr), peer) = dyn_pair(8);
        drop(peer);
        let mut conn = Connection::from_streams("/nonexistent-path-xyz", client_rd, client_wr);
        let send_fut = conn.send_with_reconnect(validate_request());
        tokio::pin!(send_fut);
        for _ in 0..6 {
            tokio::time::advance(Duration::from_millis(700)).await;
            tokio::task::yield_now().await;
        }
        let result = (&mut send_fut).await;
        assert!(
            matches!(
                result,
                Err(SandboxIpcError::Disconnected {
                    retries: MAX_RETRIES
                })
            ),
            "got: {result:?}"
        );
    }

    /// Gotcha #72 invariant test applied from day 1 (not retrofitted).
    /// The reader must survive across consecutive `next_response` calls
    /// so that a request-response client can issue multiple validations
    /// over the same connection without the second call seeing an empty
    /// reader. Sister test to `drone_ipc::connection::tests::
    /// next_event_returns_consecutive_events_without_consuming_reader`.
    #[tokio::test]
    async fn next_response_returns_consecutive_responses_without_consuming_reader() {
        use tokio::io::AsyncWriteExt;
        let ((client_rd, client_wr), (_peer_rd, mut peer_wr)) = dyn_pair(512);
        let mut conn = Connection::from_streams("/test", client_rd, client_wr);

        let r1 = serde_json::to_string(&SandboxResponse::ValidationResult(ValidationResult::Ok))
            .expect("ser1");
        let r2 = serde_json::to_string(&SandboxResponse::ValidationResult(
            ValidationResult::reject(vec!["x".to_string()]),
        ))
        .expect("ser2");
        peer_wr
            .write_all(format!("{r1}\n{r2}\n").as_bytes())
            .await
            .expect("peer write");

        let first = conn.next_response().await.expect("first").expect("ok1");
        let second = conn.next_response().await.expect("second").expect("ok2");

        assert!(matches!(
            first,
            SandboxResponse::ValidationResult(ValidationResult::Ok)
        ));
        match second {
            SandboxResponse::ValidationResult(ValidationResult::Reject { reasons }) => {
                assert_eq!(reasons, vec!["x".to_string()]);
            }
            other => panic!("expected Reject, got {other:?}"),
        }
        // Reader stays installed across consecutive reads — the
        // invariant the M04 IRL drone bug violated, applied here from
        // day 1 per CLAUDE.md §19 Decisions for the next stage.
        assert!(
            conn.reader.is_some(),
            "reader must persist across next_response calls",
        );
    }

    #[tokio::test]
    async fn next_response_returns_none_after_eof() {
        // Drop the peer halves so the duplex EOFs. next_response should
        // surface None and drop the reader so subsequent calls are
        // no-ops.
        let ((client_rd, client_wr), peer) = dyn_pair(64);
        drop(peer);
        let mut conn = Connection::from_streams("/test", client_rd, client_wr);
        let first = conn.next_response().await;
        assert!(first.is_none(), "got {first:?}");
        // Reader should be cleared.
        assert!(conn.reader.is_none());
        // Subsequent calls are immediate-None.
        assert!(conn.next_response().await.is_none());
    }

    #[tokio::test]
    async fn next_response_surfaces_json_error_for_malformed_line() {
        use tokio::io::AsyncWriteExt;
        let ((client_rd, client_wr), (_peer_rd, mut peer_wr)) = dyn_pair(64);
        let mut conn = Connection::from_streams("/test", client_rd, client_wr);
        peer_wr
            .write_all(b"{ not json }\n")
            .await
            .expect("peer write");
        peer_wr.flush().await.expect("flush");
        let result = conn.next_response().await.expect("some");
        assert!(
            matches!(result, Err(SandboxIpcError::Json(_))),
            "got {result:?}"
        );
    }

    #[test]
    fn sandbox_ipc_error_from_lines_codec_max_length() {
        let e: SandboxIpcError = LinesCodecError::MaxLineLengthExceeded.into();
        assert!(matches!(e, SandboxIpcError::Codec(_)));
    }

    #[test]
    fn sandbox_ipc_error_from_lines_codec_io() {
        let e: SandboxIpcError = LinesCodecError::Io(std::io::Error::other("x")).into();
        assert!(matches!(e, SandboxIpcError::Io(_)));
    }

    #[test]
    fn is_noop_distinguishes_modes() {
        assert!(Connection::noop().is_noop());
        let ((rd, wr), _peer) = dyn_pair(8);
        let conn = Connection::from_streams("/x", rd, wr);
        assert!(!conn.is_noop());
    }

    /// Mirrors `runtime_core::MAX_IPC_FRAME_BYTES` as a literal on
    /// purpose: the tests pin the agreed 4 MiB boundary VALUE
    /// (delimiter-exclusive per tokio-util 0.7.18 `LinesCodec`), so a
    /// silent change to the production constant fails here. TD-053.
    const CAP: usize = 4 * 1024 * 1024;

    /// Kills the `reconnect -> Ok(())` mutant (M09.5.C mutation gate) —
    /// sister to `drone_ipc::connection::tests::
    /// send_with_reconnect_reestablishes_transport_via_real_endpoint`;
    /// see that test's doc for the rationale.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn send_with_reconnect_reestablishes_transport_via_real_endpoint() {
        use tokio::io::AsyncBufReadExt;
        let (line_tx, line_rx) = tokio::sync::oneshot::channel::<String>();

        #[cfg(unix)]
        let addr = {
            let dir = tempfile::TempDir::new().expect("tempdir");
            let path = dir.path().join("reconnect.sock");
            std::mem::forget(dir);
            let listener = tokio::net::UnixListener::bind(&path).expect("bind");
            tokio::spawn(async move {
                let (stream, _) = listener.accept().await.expect("accept");
                let mut reader = tokio::io::BufReader::new(stream);
                let mut line = String::new();
                reader.read_line(&mut line).await.expect("read");
                let _ = line_tx.send(line);
            });
            path.to_string_lossy().into_owned()
        };
        #[cfg(windows)]
        let addr = {
            let addr = format!(r"\\.\pipe\sandbox-ipc-reconnect-{}", uuid::Uuid::new_v4());
            let server = tokio::net::windows::named_pipe::ServerOptions::new()
                .create(&addr)
                .expect("create pipe");
            tokio::spawn(async move {
                server.connect().await.expect("pipe connect");
                let mut reader = tokio::io::BufReader::new(server);
                let mut line = String::new();
                reader.read_line(&mut line).await.expect("read");
                let _ = line_tx.send(line);
            });
            addr
        };

        let ((client_rd, client_wr), peer) = dyn_pair(8);
        drop(peer);
        let mut conn = Connection::from_streams(&addr, client_rd, client_wr);
        conn.send_with_reconnect(validate_request())
            .await
            .expect("send must succeed once reconnect re-opens the real endpoint");
        let line = tokio::time::timeout(Duration::from_secs(5), line_rx)
            .await
            .expect("server did not receive the request in time")
            .expect("server task dropped");
        assert!(line.contains("validate_artifact"), "got: {line}");
    }

    /// TD-053 adversarial: a hostile/corrupted sandbox writing `CAP + 1`
    /// bytes with NO newline must surface the typed
    /// `SandboxIpcError::Codec` (the dead `MaxLineLengthExceeded` arm at
    /// the `From` impl goes live) instead of buffering unbounded.
    ///
    /// RED (pre-impl): `next_response` never returns (the uncapped codec
    /// waits for a newline forever) and the timeout below fails. The
    /// peer write half stays OPEN on purpose: at EOF `decode_eof` would
    /// deliver the blob as a line and today's failure would be a Json
    /// error, not the buffering bug.
    #[tokio::test]
    async fn next_response_surfaces_codec_error_on_oversize_unterminated_line() {
        use tokio::io::AsyncWriteExt;
        let ((client_rd, client_wr), (_peer_rd, mut peer_wr)) = dyn_pair(64 * 1024);
        let mut conn = Connection::from_streams("/test", client_rd, client_wr);
        let writer = tokio::spawn(async move {
            let blob = vec![b'x'; CAP + 1];
            peer_wr.write_all(&blob).await.expect("peer write");
            peer_wr.flush().await.expect("peer flush");
            peer_wr
        });
        let result = tokio::time::timeout(Duration::from_secs(5), conn.next_response())
            .await
            .expect(
                "next_response hung on an oversize unterminated line — \
                 the unbounded LinesCodec buffered it (TD-053)",
            );
        let err = result
            .expect("expected Some(Err(..))")
            .expect_err("an oversize line must error, not decode");
        assert!(
            matches!(&err, SandboxIpcError::Codec(m) if m.contains("max line length")),
            "expected the typed max-length Codec error, got {err:?}"
        );
        let _ = writer.await;
    }

    /// PIN — green at red by design (rider 3): a response line of
    /// EXACTLY `CAP` content bytes + `\n` decodes (delimiter-exclusive,
    /// rider 2). Pins that the cap clips nothing legitimate.
    #[tokio::test]
    async fn next_response_decodes_a_line_at_exactly_the_cap() {
        use tokio::io::AsyncWriteExt;
        let ((client_rd, client_wr), (_peer_rd, mut peer_wr)) = dyn_pair(64 * 1024);
        let mut conn = Connection::from_streams("/test", client_rd, client_wr);

        let base = serde_json::to_string(&SandboxResponse::Alert {
            level: runtime_sandbox::protocol::AlertLevel::Warn,
            message: String::new(),
        })
        .expect("serialize base")
        .len();
        let pad = "x".repeat(CAP - base);
        let resp = SandboxResponse::Alert {
            level: runtime_sandbox::protocol::AlertLevel::Warn,
            message: pad,
        };
        let line = serde_json::to_string(&resp).expect("serialize");
        assert_eq!(line.len(), CAP, "fixture bug: line must be exactly CAP");

        let writer = tokio::spawn(async move {
            peer_wr
                .write_all(format!("{line}\n").as_bytes())
                .await
                .expect("peer write");
            peer_wr.flush().await.expect("peer flush");
            peer_wr
        });
        let received = tokio::time::timeout(Duration::from_secs(10), conn.next_response())
            .await
            .expect("at-cap line did not decode within 10s")
            .expect("some")
            .expect("ok");
        match received {
            SandboxResponse::Alert { message, .. } => assert_eq!(message.len(), CAP - base),
            other @ SandboxResponse::ValidationResult(_) => {
                panic!("expected the at-cap Alert back, got {other:?}")
            }
        }
        let _ = writer.await;
    }
}
