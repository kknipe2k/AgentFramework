//! IPC server — Unix domain socket / Windows named pipe + framed JSON.
//!
//! Mirrors `runtime_drone::ipc` in shape: bind the socket, accept a
//! connection from main, decode newline-delimited
//! [`crate::SandboxRequest`] messages, run the validator, write
//! [`crate::SandboxResponse`] lines back. Strict request-response — no
//! broadcast subscriber, no event stream.
//!
//! Malformed JSON does not kill the server: it emits a
//! [`crate::SandboxResponse::Alert`] (level `Warn`) and continues.

use std::path::{Path, PathBuf};

use futures::SinkExt;
use futures::StreamExt;
use runtime_core::MAX_IPC_FRAME_BYTES;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tracing::{info, warn};

use crate::error::IpcError;
use crate::protocol::{AlertLevel, SandboxRequest, SandboxResponse};
use crate::validator::{self, Artifact};

/// A bound, pre-fence IPC endpoint — the output of [`bind`], consumed
/// by [`serve_bound`].
///
/// Socket setup is PRE-FENCE work: on Unix the bind + `chmod 0600`
/// execute filesystem syscalls (`chmod` is not in the seccomp
/// allowlist), so they must run BEFORE `install_isolation` — CI run
/// #295 caught the chmod executing under the fence and killing the
/// subprocess. On Windows there is no fence and no pre-bind work; the
/// endpoint carries the pipe path for the accept loop's per-instance
/// pipe creation.
pub struct BoundEndpoint {
    #[cfg(unix)]
    listener: tokio::net::UnixListener,
    #[cfg(windows)]
    socket_path: PathBuf,
}

/// Bind the IPC endpoint — the pre-fence half of [`serve`].
///
/// Unix: removes a stale socket file, creates the parent directory,
/// binds, and tightens the socket to owner-only. Windows: carries the
/// pipe path unchanged (the named pipe is created per-instance inside
/// the accept loop).
///
/// Must be called inside a tokio runtime (the Unix listener registers
/// with the reactor).
///
/// # Errors
///
/// Returns [`IpcError::Io`] if the socket cannot be bound or its
/// permissions cannot be set.
pub fn bind(socket_path: &Path) -> Result<BoundEndpoint, IpcError> {
    info!(path = %socket_path.display(), "sandbox ipc endpoint binding");
    #[cfg(unix)]
    {
        if socket_path.exists() {
            std::fs::remove_file(socket_path)?;
        }
        if let Some(parent) = socket_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let listener = tokio::net::UnixListener::bind(socket_path)?;
        // Owner-only (TD-053): the bind itself is umask-dependent. Honest
        // race note: between bind and this chmod the socket briefly carries
        // umask-default permissions — acceptable for v0.1 single-user; the
        // v1.0 tightening is a restrictive-umask guard (or 0700 parent dir)
        // before bind.
        std::fs::set_permissions(
            socket_path,
            std::os::unix::fs::PermissionsExt::from_mode(0o600),
        )?;
        Ok(BoundEndpoint { listener })
    }
    #[cfg(windows)]
    {
        Ok(BoundEndpoint {
            socket_path: socket_path.to_path_buf(),
        })
    }
}

/// Run the accept loop on a pre-bound endpoint — the fence-safe half
/// of [`serve`].
///
/// Accepts connections from main and handles requests until a
/// `Shutdown` request arrives. Returns only on a fatal accept error or
/// `Shutdown`; abort the task externally to stop it.
///
/// # Errors
///
/// Returns [`IpcError::Io`] if accept (or, on Windows, per-instance
/// pipe creation) fails.
pub async fn serve_bound(endpoint: BoundEndpoint) -> Result<(), IpcError> {
    #[cfg(unix)]
    {
        accept_loop(endpoint.listener).await
    }
    #[cfg(windows)]
    {
        accept_loop(&endpoint.socket_path).await
    }
}

/// Run the IPC server: bind the socket / pipe at `socket_path`, accept a
/// connection from main, and handle requests until the client closes
/// the connection or a `Shutdown` request arrives.
///
/// Composes [`bind`] + [`serve_bound`] in one call — the form used by
/// unfenced callers (tests). The production sandbox (`run_inner`) calls
/// the two halves separately so the bind + chmod land BEFORE
/// `install_isolation` and the accept loop runs under the fence.
///
/// # Errors
///
/// Returns [`IpcError::Io`] if the socket cannot be bound or accept fails.
pub async fn serve(socket_path: PathBuf) -> Result<(), IpcError> {
    serve_bound(bind(&socket_path)?).await
}

#[cfg(unix)]
async fn accept_loop(listener: tokio::net::UnixListener) -> Result<(), IpcError> {
    loop {
        let (stream, _addr) = listener.accept().await?;
        let (rd, wr) = stream.into_split();
        if handle_connection(rd, wr).await {
            // Shutdown requested by client — exit the accept loop.
            return Ok(());
        }
    }
}

#[cfg(windows)]
async fn accept_loop(socket_path: &Path) -> Result<(), IpcError> {
    use tokio::net::windows::named_pipe::ServerOptions;
    let pipe_name = derive_pipe_name(socket_path);
    // Default (null) security descriptor + reject_remote_clients pinned —
    // the verified-default rationale and the read-eavesdropping residual
    // (M12-routed) are documented at `runtime_drone::ipc`'s module doc
    // ("Pipe security descriptor — the verified default", TD-053).
    let mut server = ServerOptions::new()
        .first_pipe_instance(true)
        .reject_remote_clients(true)
        .create(&pipe_name)?;
    loop {
        server.connect().await?;
        let connected = server;
        // Pre-create the next instance so a new client can connect.
        server = ServerOptions::new()
            .reject_remote_clients(true)
            .create(&pipe_name)?;
        let (rd, wr) = tokio::io::split(connected);
        if handle_connection(rd, wr).await {
            // Shutdown requested by client — exit the accept loop.
            return Ok(());
        }
    }
}

#[cfg(windows)]
fn derive_pipe_name(socket_path: &Path) -> String {
    let s = socket_path.to_string_lossy();
    if s.starts_with(r"\\.\pipe\") {
        s.into_owned()
    } else {
        let stem = socket_path.file_name().map_or_else(
            || "sandbox".to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        format!(r"\\.\pipe\{stem}")
    }
}

/// Handle a single client connection. Returns `true` iff the client
/// sent a `Shutdown` request (telling the caller to exit the accept
/// loop); returns `false` on normal disconnect or read error.
async fn handle_connection<R, W>(rd: R, wr: W) -> bool
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
{
    let mut reader = FramedRead::new(rd, LinesCodec::new_with_max_length(MAX_IPC_FRAME_BYTES));
    let mut writer = FramedWrite::new(wr, LinesCodec::new_with_max_length(MAX_IPC_FRAME_BYTES));

    while let Some(next) = reader.next().await {
        let line = match next {
            Ok(line) => line,
            Err(e) => {
                warn!(error = %e, "sandbox ipc framed read error");
                return false;
            }
        };
        match serde_json::from_str::<SandboxRequest>(&line) {
            Ok(SandboxRequest::Shutdown) => {
                info!("sandbox received shutdown");
                return true;
            }
            Ok(SandboxRequest::ValidateArtifact {
                artifact_code,
                declaration,
            }) => {
                let artifact = Artifact::new(artifact_code);
                let result = validator::validate(&artifact, &declaration);
                let resp = SandboxResponse::ValidationResult(result);
                if let Err(e) = send_response(&mut writer, &resp).await {
                    warn!(error = %e, "sandbox response write failed");
                    return false;
                }
            }
            Err(e) => {
                warn!(error = %e, line = %line, "malformed sandbox request");
                let alert = SandboxResponse::Alert {
                    level: AlertLevel::Warn,
                    message: format!("malformed sandbox request: {e}"),
                };
                if let Err(e) = send_response(&mut writer, &alert).await {
                    warn!(error = %e, "sandbox alert write failed");
                    return false;
                }
            }
        }
    }
    false
}

async fn send_response<W>(
    writer: &mut FramedWrite<W, LinesCodec>,
    resp: &SandboxResponse,
) -> Result<(), IpcError>
where
    W: AsyncWrite + Unpin + Send,
{
    let line = serde_json::to_string(resp)?;
    writer.send(line).await.map_err(map_codec_err)?;
    Ok(())
}

/// Map a `LinesCodecError` to [`IpcError`]. Extracted so the variant
/// mapping is unit-testable without constructing a real `FramedWrite`.
fn map_codec_err(e: tokio_util::codec::LinesCodecError) -> IpcError {
    match e {
        tokio_util::codec::LinesCodecError::Io(io) => IpcError::Io(io),
        tokio_util::codec::LinesCodecError::MaxLineLengthExceeded => IpcError::Io(
            std::io::Error::new(std::io::ErrorKind::InvalidData, "max line length exceeded"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator::ValidationResult;
    use runtime_core::generated::capability::{
        CapabilityDeclaration, CapabilityKind, CapabilityScope, GlobPattern, ResourceName,
        SideEffectClass,
    };
    use std::str::FromStr;
    use std::time::Duration;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::time::timeout;

    fn temp_socket_path() -> PathBuf {
        #[cfg(unix)]
        {
            let dir = tempfile::TempDir::new().expect("tempdir");
            let p = dir.path().join("sb.sock");
            std::mem::forget(dir);
            p
        }
        #[cfg(windows)]
        {
            let suffix = uuid::Uuid::new_v4();
            PathBuf::from(format!(r"\\.\pipe\sandbox-test-{suffix}"))
        }
    }

    #[cfg(unix)]
    async fn open_client(path: &Path) -> tokio::net::UnixStream {
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        loop {
            match tokio::net::UnixStream::connect(path).await {
                Ok(s) => return s,
                Err(_) if std::time::Instant::now() < deadline => {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
                Err(e) => panic!("connect: {e}"),
            }
        }
    }

    #[cfg(windows)]
    async fn open_client(path: &Path) -> tokio::net::windows::named_pipe::NamedPipeClient {
        use tokio::net::windows::named_pipe::ClientOptions;
        let mut attempts = 0u32;
        loop {
            match ClientOptions::new().open(path) {
                Ok(p) => return p,
                Err(_) if attempts < 100 => {
                    attempts += 1;
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
                Err(e) => panic!("client connect: {e}"),
            }
        }
    }

    fn sample_declaration() -> CapabilityDeclaration {
        CapabilityDeclaration {
            kind: CapabilityKind::Read,
            resource: ResourceName::from_str("*.md").expect("resource"),
            scope: CapabilityScope::Glob(GlobPattern::from_str("*.md").expect("glob")),
            side_effect_class: SideEffectClass::Pure,
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn server_replies_with_validation_result() {
        let socket = temp_socket_path();
        let socket_clone = socket.clone();
        let server = tokio::spawn(async move { serve(socket_clone).await });

        let mut client = open_client(&socket).await;
        let req = SandboxRequest::ValidateArtifact {
            artifact_code: "let x = 1;".to_string(),
            declaration: sample_declaration(),
        };
        let line = format!("{}\n", serde_json::to_string(&req).unwrap());
        client.write_all(line.as_bytes()).await.expect("write");
        client.flush().await.expect("flush");

        let (rd, _wr) = tokio::io::split(client);
        let mut reader = BufReader::new(rd);
        let mut line = String::new();
        timeout(Duration::from_secs(2), reader.read_line(&mut line))
            .await
            .expect("read timeout")
            .expect("read");
        let parsed: SandboxResponse = serde_json::from_str(line.trim()).expect("json");
        match parsed {
            SandboxResponse::ValidationResult(r) => assert_eq!(r, ValidationResult::Ok),
            SandboxResponse::Alert { .. } => panic!("expected ValidationResult"),
        }
        server.abort();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn malformed_json_emits_alert() {
        let socket = temp_socket_path();
        let socket_clone = socket.clone();
        let server = tokio::spawn(async move { serve(socket_clone).await });

        let mut client = open_client(&socket).await;
        client.write_all(b"{ not json }\n").await.expect("write");
        client.flush().await.expect("flush");

        let (rd, _wr) = tokio::io::split(client);
        let mut reader = BufReader::new(rd);
        let mut line = String::new();
        timeout(Duration::from_secs(2), reader.read_line(&mut line))
            .await
            .expect("read timeout")
            .expect("read");
        let parsed: SandboxResponse = serde_json::from_str(line.trim()).expect("json");
        assert!(matches!(parsed, SandboxResponse::Alert { .. }));
        server.abort();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shutdown_request_exits_server() {
        // A Shutdown request must terminate the accept loop, causing
        // `serve` to return Ok(()) rather than running forever. This
        // pins the wire-protocol contract that ipc.rs::handle_connection
        // returns true on Shutdown (and the accept loop honors that).
        let socket = temp_socket_path();
        let socket_clone = socket.clone();
        let server = tokio::spawn(async move { serve(socket_clone).await });

        let mut client = open_client(&socket).await;
        let req = SandboxRequest::Shutdown;
        let line = format!("{}\n", serde_json::to_string(&req).unwrap());
        client.write_all(line.as_bytes()).await.expect("write");
        client.flush().await.expect("flush");

        let result = timeout(Duration::from_secs(2), server)
            .await
            .expect("server did not exit on Shutdown")
            .expect("join");
        result.expect("serve returned an error");
    }

    #[cfg(windows)]
    #[test]
    fn derive_pipe_name_handles_explicit_pipe_paths() {
        let p = Path::new(r"\\.\pipe\sandbox-abc");
        assert_eq!(derive_pipe_name(p), r"\\.\pipe\sandbox-abc");
    }

    #[cfg(windows)]
    #[test]
    fn derive_pipe_name_handles_filesystem_paths() {
        let p = Path::new(r"C:\tmp\sandbox-abc.sock");
        assert_eq!(derive_pipe_name(p), r"\\.\pipe\sandbox-abc.sock");
    }

    #[cfg(windows)]
    #[test]
    fn derive_pipe_name_falls_back_to_sandbox_for_pathless_input() {
        // A path whose `file_name()` returns None (e.g. a root or trailing
        // separator) hits the default-name closure branch. Exercise it
        // explicitly so the coverage gate's accounting reflects reality.
        let p = Path::new(r"\\");
        let name = derive_pipe_name(p);
        assert!(name.starts_with(r"\\.\pipe\"), "got {name}");
    }

    /// A clean client disconnect (no Shutdown) returns from
    /// `handle_connection` with `false` so the accept loop keeps
    /// running. Drive the server through one client open + close to
    /// exercise the EOF path.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn handle_connection_returns_false_on_clean_eof() {
        // Build duplex halves directly and call handle_connection — this
        // bypasses the accept loop and isolates the EOF return-false
        // branch. Closing the peer halves immediately means the
        // FramedRead's first .next() returns None.
        let (client, peer) = tokio::io::duplex(64);
        drop(peer);
        let (rd, wr) = tokio::io::split(client);
        let result = handle_connection(rd, wr).await;
        assert!(!result, "clean EOF must return false (loop continues)");
    }

    /// Malformed UTF-8 bytes in the request stream surface as a framed-
    /// read error; `handle_connection` returns `false` so the accept
    /// loop keeps running (errors don't kill the server).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn handle_connection_returns_false_on_framed_read_error() {
        use tokio::io::AsyncWriteExt;
        let (client, peer) = tokio::io::duplex(64);
        let (peer_rd, mut peer_wr) = tokio::io::split(peer);
        // Write a clearly invalid UTF-8 sequence followed by a newline.
        // LinesCodec decodes bytes-to-str via from_utf8 and surfaces
        // Err(LinesCodecError::Io(invalid utf-8)) — the Err branch.
        peer_wr
            .write_all(&[0xff, 0xfe, 0xfd, b'\n'])
            .await
            .expect("write");
        peer_wr.flush().await.expect("flush");
        drop(peer_rd);
        drop(peer_wr);
        let (rd, wr) = tokio::io::split(client);
        let result = handle_connection(rd, wr).await;
        assert!(!result, "framed read error must return false");
    }

    #[test]
    fn map_codec_err_io_passes_through() {
        let io = std::io::Error::other("boom");
        let mapped = map_codec_err(tokio_util::codec::LinesCodecError::Io(io));
        assert!(matches!(mapped, IpcError::Io(_)));
    }

    #[test]
    fn map_codec_err_max_length_maps_to_invalid_data() {
        let mapped = map_codec_err(tokio_util::codec::LinesCodecError::MaxLineLengthExceeded);
        match mapped {
            IpcError::Io(io) => assert_eq!(io.kind(), std::io::ErrorKind::InvalidData),
            IpcError::Json(_) => panic!("expected Io(InvalidData)"),
        }
    }

    /// `send_response` write failure after parsing a valid
    /// `ValidateArtifact` request — the write-half is dropped, so the
    /// codec's `send` returns `Err(Io)`. `handle_connection` surfaces
    /// `false` and the accept loop continues.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn handle_connection_returns_false_when_response_write_fails() {
        use tokio::io::AsyncWriteExt;
        let (client, peer) = tokio::io::duplex(2048);
        let (peer_rd, mut peer_wr) = tokio::io::split(peer);

        // Feed a valid validate_artifact request, then drop the peer
        // read half so the server's write attempts fail.
        let req = SandboxRequest::ValidateArtifact {
            artifact_code: "let x = 1;".to_string(),
            declaration: sample_declaration(),
        };
        let line = format!("{}\n", serde_json::to_string(&req).expect("encode"));
        peer_wr.write_all(line.as_bytes()).await.expect("write");
        peer_wr.flush().await.expect("flush");
        drop(peer_rd);

        let (rd, wr) = tokio::io::split(client);
        let handle = tokio::spawn(async move { handle_connection(rd, wr).await });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        drop(peer_wr);
        let result = tokio::time::timeout(std::time::Duration::from_secs(2), handle)
            .await
            .expect("handle_connection did not return")
            .expect("join");
        // The branch we want covered is the "write failed → return
        // false". Either that branch fired (we get false) OR the read
        // EOFed cleanly first (also false). Either way the return is
        // false; the assertion is that we don't return true.
        assert!(!result, "write failure must not be treated as Shutdown");
    }

    /// Direct test of `send_response` failure surface. Drops the peer
    /// outright so the `FramedWrite`'s first flush fails. Exercises
    /// `send_response`'s `?` propagation through `map_codec_err`.
    /// Uses a payload larger than the duplex capacity so the flush
    /// must actually attempt an underlying write.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn send_response_propagates_io_error_on_dead_peer() {
        let (client, peer) = tokio::io::duplex(8);
        drop(peer);
        let (_rd, wr) = tokio::io::split(client);
        // Fixture mirrors the production cap (C.2: production sites
        // capped, the fixture follows) — zero bare LinesCodec::new()
        // anywhere in the crate.
        let mut writer = FramedWrite::new(wr, LinesCodec::new_with_max_length(MAX_IPC_FRAME_BYTES));
        let resp = SandboxResponse::Alert {
            level: AlertLevel::Warn,
            message: "x".repeat(256),
        };
        let result = send_response(&mut writer, &resp).await;
        assert!(result.is_err(), "send_response must surface BrokenPipe");
    }

    /// Mirrors `runtime_core::MAX_IPC_FRAME_BYTES` as a literal on
    /// purpose: the tests pin the agreed 4 MiB boundary VALUE
    /// (delimiter-exclusive per tokio-util 0.7.18 `LinesCodec`), so a
    /// silent change to the production constant fails here. TD-053.
    const CAP: usize = 4 * 1024 * 1024;

    /// TD-053 adversarial: a `CAP + 1` byte write with NO newline must
    /// error the framed read so `handle_connection` returns `false`
    /// (connection dropped, accept loop lives on).
    ///
    /// RED (pre-impl): the uncapped codec buffers forever — the handler
    /// never returns and the timeout below fails. The peer halves stay
    /// OPEN on purpose: at EOF `decode_eof` would deliver the blob as a
    /// line and today's failure would be a serde error, not the
    /// buffering bug.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn oversize_unterminated_frame_returns_false_and_does_not_hang() {
        use tokio::io::AsyncWriteExt;
        let (client, peer) = tokio::io::duplex(64 * 1024);
        let (_peer_rd, mut peer_wr) = tokio::io::split(peer);
        let (rd, wr) = tokio::io::split(client);
        let handle = tokio::spawn(async move { handle_connection(rd, wr).await });

        let blob = vec![b'x'; CAP + 1];
        peer_wr.write_all(&blob).await.expect("peer write");
        peer_wr.flush().await.expect("peer flush");

        let result = timeout(Duration::from_secs(5), handle)
            .await
            .expect(
                "handle_connection hung on an oversize unterminated frame — \
                 the unbounded LinesCodec buffered it (TD-053)",
            )
            .expect("join");
        assert!(!result, "an oversize frame must not read as Shutdown");
        drop(peer_wr);
    }

    /// PIN — green at red by design (rider 3): a request line of EXACTLY
    /// `CAP` content bytes (the codec's `\n` rides on top —
    /// delimiter-exclusive, rider 2) decodes, validates, and the
    /// response round-trips. Pins that the cap clips nothing legitimate.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn at_cap_frame_round_trips_a_validation() {
        use tokio::io::AsyncWriteExt;
        let (client, peer) = tokio::io::duplex(64 * 1024);
        let (peer_rd, mut peer_wr) = tokio::io::split(peer);
        let (rd, wr) = tokio::io::split(client);
        let _handle = tokio::spawn(async move { handle_connection(rd, wr).await });

        let base = serde_json::to_string(&SandboxRequest::ValidateArtifact {
            artifact_code: String::new(),
            declaration: sample_declaration(),
        })
        .expect("serialize base")
        .len();
        let pad = "x".repeat(CAP - base);
        let req = SandboxRequest::ValidateArtifact {
            artifact_code: pad,
            declaration: sample_declaration(),
        };
        let line = serde_json::to_string(&req).expect("serialize");
        assert_eq!(line.len(), CAP, "fixture bug: line must be exactly CAP");

        let writer = tokio::spawn(async move {
            peer_wr
                .write_all(format!("{line}\n").as_bytes())
                .await
                .expect("peer write");
            peer_wr.flush().await.expect("peer flush");
            peer_wr
        });
        let mut reader = BufReader::new(peer_rd);
        let mut resp_line = String::new();
        timeout(Duration::from_secs(10), reader.read_line(&mut resp_line))
            .await
            .expect("at-cap frame did not round-trip within 10s")
            .expect("read");
        let parsed: SandboxResponse = serde_json::from_str(resp_line.trim()).expect("json");
        assert!(
            matches!(parsed, SandboxResponse::ValidationResult(_)),
            "expected a ValidationResult for the at-cap request, got {parsed:?}"
        );
        let _ = writer.await;
    }

    /// TD-053: the Unix socket must be owner-only after bind. RED
    /// (pre-impl): the bind is umask-default (typically 0o755).
    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn unix_socket_mode_is_0600_after_bind() {
        use std::os::unix::fs::PermissionsExt;
        let socket = temp_socket_path();
        let socket_clone = socket.clone();
        let server = tokio::spawn(async move { serve(socket_clone).await });

        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while !socket.exists() && std::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let mode = std::fs::metadata(&socket)
            .expect("socket metadata")
            .permissions()
            .mode();
        assert_eq!(
            mode & 0o777,
            0o600,
            "sandbox socket must be 0600 (owner-only), got {:o}",
            mode & 0o777
        );
        server.abort();
    }
}
