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
}
