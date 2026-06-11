//! IPC server — Unix domain socket / Windows named pipe + framed JSON.
//!
//! Per `agent-runtime-spec.md` §1d (IPC Channels — Layer 2: Main ↔ Drone)
//! this module accepts a connection from main, decodes
//! newline-delimited JSON `DroneCommand` messages from the read half, and
//! encodes `DroneEvent` messages from a broadcast subscriber on the write
//! half. Malformed JSON does not kill the server — it emits a
//! `DroneEvent::Alert` and continues.
//!
//! `framed` JSON here means each message is one line ending in `\n`,
//! decoded by `tokio_util::codec::LinesCodec`.
//!
//! ## Windows named pipe configuration (implementer note)
//!
//! Windows uses `tokio::net::windows::named_pipe::ServerOptions` /
//! `NamedPipeServer`. The path format is `\\.\pipe\<name>`. The drone
//! accepts an `--ipc-socket` argument that may be:
//! - a Unix-style path (e.g. `/tmp/sessions/abc.sock`); the file name
//!   component is used as the pipe name (`\\.\pipe\abc.sock`).
//! - an absolute pipe path (e.g. `\\.\pipe\agent-runtime-abc`); used
//!   verbatim.
//!
//! `ServerOptions` defaults are used (`PIPE_ACCESS_DUPLEX`, BYTE type,
//! WAIT mode, instance count 255). The security descriptor defaults to
//! "owner only" — the same SID as the creating process — which is
//! sufficient for v0.1 single-user. Multi-session UX in M02+ passes a
//! session-id-scoped pipe name so multiple drones can run concurrently
//! without clashing. Hardened DACLs (deny Everyone, audit) land with M05
//! sandboxing. **This note is the source for the post-M01 `docs(spec):`
//! PR that folds Windows IPC details into `agent-runtime-spec.md` §1d.**

use futures::StreamExt;
use runtime_core::{AlertLevel, DroneCommand, DroneEvent};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{broadcast, mpsc};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tracing::{error, info, warn};

/// Errors raised by the IPC server.
#[derive(Debug, Error)]
pub enum IpcError {
    /// I/O error binding or accepting on the socket.
    #[error("ipc io: {0}")]
    Io(#[from] std::io::Error),
}

/// Run the IPC server, accepting connections from main and dispatching
/// commands / events between the in-process channels.
///
/// The function loops accepting new connections and spawning a per-
/// connection handler. It returns only on a fatal accept error; abort the
/// task to stop it cleanly.
///
/// # Errors
///
/// Returns `IpcError::Io` if the socket cannot be bound or accept fails.
pub async fn serve(
    socket_path: PathBuf,
    cmd_tx: mpsc::Sender<DroneCommand>,
    event_tx: broadcast::Sender<DroneEvent>,
) -> Result<(), IpcError> {
    info!(path = %socket_path.display(), "ipc server starting");
    accept_loop(&socket_path, cmd_tx, event_tx).await
}

#[cfg(unix)]
async fn accept_loop(
    socket_path: &Path,
    cmd_tx: mpsc::Sender<DroneCommand>,
    event_tx: broadcast::Sender<DroneEvent>,
) -> Result<(), IpcError> {
    if socket_path.exists() {
        std::fs::remove_file(socket_path)?;
    }
    if let Some(parent) = socket_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let listener = tokio::net::UnixListener::bind(socket_path)?;
    loop {
        let (stream, _addr) = listener.accept().await?;
        let cmd_tx = cmd_tx.clone();
        let event_rx = event_tx.subscribe();
        let event_tx_for_alerts = event_tx.clone();
        tokio::spawn(async move {
            let (rd, wr) = stream.into_split();
            handle_connection(rd, wr, cmd_tx, event_rx, event_tx_for_alerts).await;
        });
    }
}

#[cfg(windows)]
async fn accept_loop(
    socket_path: &Path,
    cmd_tx: mpsc::Sender<DroneCommand>,
    event_tx: broadcast::Sender<DroneEvent>,
) -> Result<(), IpcError> {
    use tokio::net::windows::named_pipe::ServerOptions;
    let pipe_name = derive_pipe_name(socket_path);
    let mut server = ServerOptions::new()
        .first_pipe_instance(true)
        .create(&pipe_name)?;
    loop {
        server.connect().await?;
        let connected = server;
        // Pre-create the next instance so a new client can connect.
        server = ServerOptions::new().create(&pipe_name)?;
        let cmd_tx = cmd_tx.clone();
        let event_rx = event_tx.subscribe();
        let event_tx_for_alerts = event_tx.clone();
        tokio::spawn(async move {
            let (rd, wr) = tokio::io::split(connected);
            handle_connection(rd, wr, cmd_tx, event_rx, event_tx_for_alerts).await;
        });
    }
}

#[cfg(windows)]
fn derive_pipe_name(socket_path: &Path) -> String {
    let s = socket_path.to_string_lossy();
    if s.starts_with(r"\\.\pipe\") {
        s.into_owned()
    } else {
        let stem = socket_path
            .file_name()
            .map_or_else(|| "drone".to_string(), |n| n.to_string_lossy().into_owned());
        format!(r"\\.\pipe\{stem}")
    }
}

async fn handle_connection<R, W>(
    rd: R,
    wr: W,
    cmd_tx: mpsc::Sender<DroneCommand>,
    mut event_rx: broadcast::Receiver<DroneEvent>,
    event_tx_for_alerts: broadcast::Sender<DroneEvent>,
) where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
{
    let mut reader = FramedRead::new(rd, LinesCodec::new());
    let read_task = tokio::spawn(async move {
        while let Some(next) = reader.next().await {
            match next {
                Ok(line) => match serde_json::from_str::<DroneCommand>(&line) {
                    Ok(cmd) => {
                        if cmd_tx.send(cmd).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        warn!(error = %e, line = %line, "malformed drone command");
                        let _ = event_tx_for_alerts.send(DroneEvent::Alert {
                            level: AlertLevel::Warn,
                            message: format!("malformed drone command: {e}"),
                        });
                    }
                },
                Err(e) => {
                    warn!(error = %e, "ipc framed read error");
                    let _ = event_tx_for_alerts.send(DroneEvent::Alert {
                        level: AlertLevel::Warn,
                        message: format!("malformed drone command: {e}"),
                    });
                    break;
                }
            }
        }
    });

    let write_task = tokio::spawn(async move {
        let mut writer = FramedWrite::new(wr, LinesCodec::new());
        loop {
            match event_rx.recv().await {
                Ok(event) => match serde_json::to_string(&event) {
                    Ok(line) => {
                        use futures::SinkExt;
                        if writer.send(line).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => error!(error = %e, "failed to encode drone event"),
                },
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!(skipped, "broadcast lagged; continuing");
                }
            }
        }
    });

    let _ = read_task.await;
    write_task.abort();
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::{DroneCommand, DroneEvent, HeartbeatStatus};
    use tokio::sync::{broadcast, mpsc};
    use tokio::time::{timeout, Duration};

    fn temp_socket_path() -> std::path::PathBuf {
        #[cfg(unix)]
        {
            let dir = tempfile::TempDir::new().expect("tempdir");
            let p = dir.path().join("d.sock");
            std::mem::forget(dir);
            p
        }
        #[cfg(windows)]
        {
            let suffix = uuid::Uuid::new_v4();
            std::path::PathBuf::from(format!(r"\\.\pipe\drone-test-{suffix}"))
        }
    }

    #[cfg(unix)]
    async fn open_client(path: &std::path::Path) -> tokio::net::UnixStream {
        // Wait briefly for the server's bind to land.
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
    async fn open_client(
        path: &std::path::Path,
    ) -> tokio::net::windows::named_pipe::NamedPipeClient {
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

    #[tokio::test]
    async fn server_decodes_command() {
        use tokio::io::AsyncWriteExt;
        let socket = temp_socket_path();
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, _event_rx) = broadcast::channel::<DroneEvent>(8);

        let socket_clone = socket.clone();
        let server = tokio::spawn(async move { serve(socket_clone, cmd_tx, event_tx).await });

        let mut client = open_client(&socket).await;
        let cmd = DroneCommand::SnapshotNow {
            reason: "manual".to_string(),
            state_json: serde_json::json!({"a": 1}),
        };
        let line = format!("{}\n", serde_json::to_string(&cmd).unwrap());
        client.write_all(line.as_bytes()).await.expect("write");
        client.flush().await.expect("flush");

        let received = timeout(Duration::from_secs(2), cmd_rx.recv())
            .await
            .expect("recv timeout")
            .expect("channel closed");
        assert_eq!(received, cmd);
        server.abort();
    }

    #[tokio::test]
    async fn server_encodes_event() {
        use tokio::io::AsyncBufReadExt;
        let socket = temp_socket_path();
        let (cmd_tx, _cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, _event_rx) = broadcast::channel::<DroneEvent>(8);

        let event_tx_clone = event_tx.clone();
        let socket_clone = socket.clone();
        let server = tokio::spawn(async move { serve(socket_clone, cmd_tx, event_tx_clone).await });

        let client = open_client(&socket).await;
        let (rd, _wr) = tokio::io::split(client);
        let mut reader = tokio::io::BufReader::new(rd);

        // Wait briefly for the server to register the subscriber.
        tokio::time::sleep(Duration::from_millis(100)).await;
        event_tx
            .send(DroneEvent::Heartbeat {
                status: HeartbeatStatus::Ok,
                timestamp: 1,
            })
            .expect("broadcast");

        let mut line = String::new();
        timeout(Duration::from_secs(2), reader.read_line(&mut line))
            .await
            .expect("read timeout")
            .expect("read");
        let parsed: DroneEvent = serde_json::from_str(line.trim()).expect("json");
        assert!(matches!(parsed, DroneEvent::Heartbeat { .. }));
        server.abort();
    }

    #[tokio::test]
    async fn malformed_json_emits_alert() {
        use tokio::io::AsyncWriteExt;
        let socket = temp_socket_path();
        let (cmd_tx, _cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, mut event_rx) = broadcast::channel::<DroneEvent>(8);

        let socket_clone = socket.clone();
        let server = tokio::spawn(async move { serve(socket_clone, cmd_tx, event_tx).await });

        let mut client = open_client(&socket).await;
        client.write_all(b"{ not json }\n").await.expect("write");
        client.flush().await.expect("flush");

        let mut got_alert = false;
        for _ in 0..10 {
            if let Ok(Ok(DroneEvent::Alert { .. })) =
                timeout(Duration::from_millis(500), event_rx.recv()).await
            {
                got_alert = true;
                break;
            }
        }
        assert!(got_alert, "malformed JSON should produce an Alert event");
        server.abort();
    }

    /// Mirrors `runtime_core::MAX_IPC_FRAME_BYTES` as a literal on
    /// purpose: the tests pin the agreed 4 MiB boundary VALUE
    /// (delimiter-exclusive per tokio-util 0.7.18 `LinesCodec`), so a
    /// silent change to the production constant fails here. TD-053.
    const CAP: usize = 4 * 1024 * 1024;

    /// TD-053 adversarial: a `CAP + 1` byte write with NO newline must
    /// surface the max-length error as an `Alert` and drop the
    /// connection (handler returns; daemon's accept loop lives on).
    ///
    /// RED (pre-impl): the uncapped codec buffers forever — no Alert
    /// fires and the wait below times out. The peer halves stay OPEN on
    /// purpose: at EOF `decode_eof` would deliver the blob as a line and
    /// today's failure would be a serde error, not the buffering bug.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn oversize_unterminated_frame_emits_alert_and_drops_connection() {
        use tokio::io::AsyncWriteExt;
        let (client, peer) = tokio::io::duplex(64 * 1024);
        let (_peer_rd, mut peer_wr) = tokio::io::split(peer);
        let (rd, wr) = tokio::io::split(client);
        let (cmd_tx, _cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, mut event_rx) = broadcast::channel::<DroneEvent>(8);
        let handler = tokio::spawn(handle_connection(
            rd,
            wr,
            cmd_tx,
            event_tx.subscribe(),
            event_tx.clone(),
        ));

        let blob = vec![b'x'; CAP + 1];
        peer_wr.write_all(&blob).await.expect("peer write");
        peer_wr.flush().await.expect("peer flush");

        let message = timeout(Duration::from_secs(5), async {
            loop {
                match event_rx.recv().await {
                    Ok(DroneEvent::Alert { message, .. }) => break message,
                    Err(broadcast::error::RecvError::Closed) => {
                        panic!("event channel closed without an Alert")
                    }
                    Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => {}
                }
            }
        })
        .await
        .expect("no Alert within 5s — the oversize unterminated frame was accepted (TD-053)");
        assert!(
            message.contains("max line length"),
            "Alert must carry the length signal, got: {message}"
        );

        timeout(Duration::from_secs(2), handler)
            .await
            .expect("handle_connection did not return after the oversize frame")
            .expect("join");
        drop(peer_wr);
    }

    /// PIN — green at red by design (rider 3): a command line of EXACTLY
    /// `CAP` content bytes (the codec's `\n` rides on top —
    /// delimiter-exclusive, rider 2) decodes and reaches the command
    /// channel. Pins that the cap clips nothing legitimate.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn at_cap_frame_decodes_to_the_command_channel() {
        use tokio::io::AsyncWriteExt;
        let (client, peer) = tokio::io::duplex(64 * 1024);
        let (_peer_rd, mut peer_wr) = tokio::io::split(peer);
        let (rd, wr) = tokio::io::split(client);
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, _event_rx) = broadcast::channel::<DroneEvent>(8);
        let _handler = tokio::spawn(handle_connection(
            rd,
            wr,
            cmd_tx,
            event_tx.subscribe(),
            event_tx.clone(),
        ));

        let base = serde_json::to_string(&DroneCommand::SnapshotNow {
            reason: "at-cap".to_string(),
            state_json: serde_json::json!({"pad": ""}),
        })
        .expect("serialize base")
        .len();
        let pad = "x".repeat(CAP - base);
        let cmd = DroneCommand::SnapshotNow {
            reason: "at-cap".to_string(),
            state_json: serde_json::json!({ "pad": pad }),
        };
        let line = serde_json::to_string(&cmd).expect("serialize");
        assert_eq!(line.len(), CAP, "fixture bug: line must be exactly CAP");

        let writer = tokio::spawn(async move {
            peer_wr
                .write_all(format!("{line}\n").as_bytes())
                .await
                .expect("peer write");
            peer_wr.flush().await.expect("peer flush");
            peer_wr
        });
        let received = timeout(Duration::from_secs(10), cmd_rx.recv())
            .await
            .expect("at-cap frame did not decode within 10s")
            .expect("channel closed");
        assert_eq!(received, cmd);
        let _ = writer.await;
    }

    /// TD-053: the Unix socket must be owner-only after bind. RED
    /// (pre-impl): the bind is umask-default (typically 0o755).
    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn unix_socket_mode_is_0600_after_bind() {
        use std::os::unix::fs::PermissionsExt;
        let socket = temp_socket_path();
        let (cmd_tx, _cmd_rx) = mpsc::channel::<DroneCommand>(8);
        let (event_tx, _event_rx) = broadcast::channel::<DroneEvent>(8);
        let socket_clone = socket.clone();
        let server = tokio::spawn(async move { serve(socket_clone, cmd_tx, event_tx).await });

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
            "drone socket must be 0600 (owner-only), got {:o}",
            mode & 0o777
        );
        server.abort();
    }
}
