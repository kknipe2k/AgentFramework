//! `SandboxClient` — main-side connection wrapper around the M05 sandbox.
//!
//! Cfg-platform: Unix domain socket on Linux/macOS, Windows named pipe
//! on Windows. Reconnects automatically on transport errors per the
//! policy in [`super::connection`].
//!
//! Stage C1 ships the strict request-response client: every `validate`
//! call sends one [`SandboxRequest::ValidateArtifact`] and reads one
//! [`SandboxResponse`]. Multi-call invariant is exercised from day 1 by
//! the `validate_succeeds_twice_in_sequence` test (gotcha #69).

use std::time::Duration;

use runtime_core::generated::capability::CapabilityDeclaration;
use runtime_sandbox::protocol::{SandboxRequest, SandboxResponse};
use runtime_sandbox::validator::ValidationResult;
use tokio::sync::Mutex;

use super::connection::{Connection, SandboxIpcError};

/// Maximum time to wait for a `validate` response before surfacing
/// `Io(TimedOut)`.
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

/// Main-side IPC client for the runtime-sandbox subprocess.
pub struct SandboxClient {
    inner: Mutex<Connection>,
}

impl SandboxClient {
    /// Connect to a running sandbox over its IPC socket / named pipe.
    ///
    /// `addr` is a filesystem path on Unix, or a named-pipe name on
    /// Windows (e.g. `\\.\pipe\runtime-sandbox-abc`).
    ///
    /// # Errors
    ///
    /// Returns [`SandboxIpcError::Io`] if the underlying open fails.
    pub async fn connect(addr: &str) -> Result<Self, SandboxIpcError> {
        let conn = Connection::connect(addr).await?;
        Ok(Self {
            inner: Mutex::new(conn),
        })
    }

    /// No-op constructor — `validate` returns
    /// [`ValidationResult::Ok`] immediately. Used by tests / paths that
    /// don't exercise a real sandbox.
    #[must_use]
    pub fn noop() -> Self {
        Self {
            inner: Mutex::new(Connection::noop()),
        }
    }

    /// Send a [`SandboxRequest::ValidateArtifact`] and await the
    /// matching [`SandboxResponse::ValidationResult`]. Alerts surface as
    /// [`SandboxIpcError::Codec`].
    ///
    /// On noop mode returns `ValidationResult::Ok` immediately.
    ///
    /// # Errors
    ///
    /// - [`SandboxIpcError::Disconnected`] on send retry exhaustion.
    /// - [`SandboxIpcError::Json`] on response parse failure.
    /// - [`SandboxIpcError::Codec`] on alert or framing error.
    /// - [`SandboxIpcError::Io`] with `TimedOut` if no response arrives
    ///   within `RESPONSE_TIMEOUT` (5 seconds).
    pub async fn validate(
        &self,
        artifact_code: String,
        declaration: CapabilityDeclaration,
    ) -> Result<ValidationResult, SandboxIpcError> {
        let mut guard = self.inner.lock().await;
        if guard.is_noop() {
            return Ok(ValidationResult::Ok);
        }
        guard
            .send_with_reconnect(SandboxRequest::ValidateArtifact {
                artifact_code,
                declaration,
            })
            .await?;
        let result = await_validation_result(&mut guard).await;
        drop(guard);
        result
    }

    /// Send a [`SandboxRequest::Shutdown`]. No response is awaited; the
    /// subprocess exits cleanly on receipt.
    ///
    /// On noop mode returns immediately.
    ///
    /// # Errors
    ///
    /// Surfaces [`SandboxIpcError::Disconnected`] if the retry budget
    /// is exhausted; [`SandboxIpcError::Json`] on serialization bugs.
    pub async fn shutdown(&self) -> Result<(), SandboxIpcError> {
        let mut guard = self.inner.lock().await;
        if guard.is_noop() {
            return Ok(());
        }
        guard.send_with_reconnect(SandboxRequest::Shutdown).await
    }
}

/// Pull responses from the connection until a `ValidationResult` arrives
/// or an `Alert` surfaces as a `Codec` error. Uses the borrow-not-move
/// [`Connection::next_response`] so subsequent `validate` calls can
/// continue reading from the same reader (gotcha #72).
async fn await_validation_result(
    conn: &mut Connection,
) -> Result<ValidationResult, SandboxIpcError> {
    let timed = tokio::time::timeout(RESPONSE_TIMEOUT, async {
        // Single-shot match: every sandbox `ValidateArtifact` request
        // gets exactly one response (validation result, alert, or
        // stream EOF). No skip-and-filter is needed because the wire
        // protocol has no heartbeat / out-of-band events on this
        // channel — unlike drone, where heartbeats interleave with
        // request-response traffic.
        match conn.next_response().await {
            Some(Ok(SandboxResponse::ValidationResult(r))) => Ok(r),
            Some(Ok(SandboxResponse::Alert { message, .. })) => {
                Err(SandboxIpcError::Codec(message))
            }
            Some(Err(e)) => Err(e),
            None => Err(SandboxIpcError::Codec(
                "response stream ended without ValidationResult".into(),
            )),
        }
    })
    .await;
    timed.unwrap_or_else(|_| {
        Err(SandboxIpcError::Io(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "sandbox response timeout",
        )))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::generated::capability::{
        CapabilityKind, CapabilityScope, GlobPattern, ResourceName, SideEffectClass,
    };
    use runtime_sandbox::protocol::AlertLevel;
    use std::pin::Pin;
    use std::str::FromStr;
    use tokio::io::{AsyncRead, AsyncWrite};

    type DynRead = Pin<Box<dyn AsyncRead + Send + Unpin>>;
    type DynWrite = Pin<Box<dyn AsyncWrite + Send + Unpin>>;

    fn pure_read_declaration() -> CapabilityDeclaration {
        CapabilityDeclaration {
            kind: CapabilityKind::Read,
            resource: ResourceName::from_str("*.md").expect("resource"),
            scope: CapabilityScope::Glob(GlobPattern::from_str("*.md").expect("glob")),
            side_effect_class: SideEffectClass::Pure,
        }
    }

    #[tokio::test]
    async fn noop_validate_returns_ok() {
        let c = SandboxClient::noop();
        let r = c
            .validate("code".to_string(), pure_read_declaration())
            .await
            .expect("noop validate");
        assert_eq!(r, ValidationResult::Ok);
    }

    #[tokio::test]
    async fn noop_shutdown_returns_ok() {
        let c = SandboxClient::noop();
        c.shutdown().await.expect("noop shutdown");
    }

    /// Single round-trip via duplex peer — feed a synthetic
    /// `ValidationResult` event and assert the client returns it.
    #[tokio::test]
    async fn validate_request_response_succeeds() {
        use tokio::io::AsyncWriteExt;
        let (a, b) = tokio::io::duplex(4096);
        let (a_rd, a_wr) = tokio::io::split(a);
        let (b_rd, mut b_wr) = tokio::io::split(b);
        let conn = Connection::from_streams(
            "/test",
            Box::pin(a_rd) as DynRead,
            Box::pin(a_wr) as DynWrite,
        );
        let resp = serde_json::to_string(&SandboxResponse::ValidationResult(ValidationResult::Ok))
            .unwrap();
        b_wr.write_all(format!("{resp}\n").as_bytes())
            .await
            .expect("peer write");
        b_wr.flush().await.expect("flush");
        drop(b_rd);

        let client = SandboxClient {
            inner: tokio::sync::Mutex::new(conn),
        };
        let r = client
            .validate("code".to_string(), pure_read_declaration())
            .await
            .expect("validate");
        assert_eq!(r, ValidationResult::Ok);
    }

    /// **Gotcha #69 first-class application.** Two consecutive `validate`
    /// calls over the same connection must both succeed. Sister test to
    /// `drone_ipc::client::tests::query_session_db_succeeds_twice_in_sequence`
    /// — the multi-call invariant is the load-bearing safety property
    /// that prevents the M04 IRL drone bug from recurring in sandbox.
    #[tokio::test]
    async fn validate_succeeds_twice_in_sequence() {
        use tokio::io::AsyncWriteExt;
        let (a, b) = tokio::io::duplex(4096);
        let (a_rd, a_wr) = tokio::io::split(a);
        let (b_rd, mut b_wr) = tokio::io::split(b);
        let conn = Connection::from_streams(
            "/test",
            Box::pin(a_rd) as DynRead,
            Box::pin(a_wr) as DynWrite,
        );
        let r1 = serde_json::to_string(&SandboxResponse::ValidationResult(ValidationResult::Ok))
            .unwrap();
        let r2 = serde_json::to_string(&SandboxResponse::ValidationResult(
            ValidationResult::reject(vec!["disallowed".to_string()]),
        ))
        .unwrap();
        b_wr.write_all(format!("{r1}\n{r2}\n").as_bytes())
            .await
            .expect("peer write");
        b_wr.flush().await.expect("flush");
        drop(b_rd);

        let client = SandboxClient {
            inner: tokio::sync::Mutex::new(conn),
        };

        let first = client
            .validate("code1".to_string(), pure_read_declaration())
            .await
            .expect("first validate");
        assert_eq!(first, ValidationResult::Ok);

        let second = client
            .validate("code2".to_string(), pure_read_declaration())
            .await
            .expect("second validate must also succeed (gotcha #69)");
        match second {
            ValidationResult::Reject { reasons } => {
                assert_eq!(reasons, vec!["disallowed".to_string()]);
            }
            ValidationResult::Ok => panic!("expected Reject"),
        }
    }

    #[tokio::test]
    async fn validate_alert_surfaces_as_codec_error() {
        use tokio::io::AsyncWriteExt;
        let (a, b) = tokio::io::duplex(4096);
        let (a_rd, a_wr) = tokio::io::split(a);
        let (b_rd, mut b_wr) = tokio::io::split(b);
        let conn = Connection::from_streams(
            "/test",
            Box::pin(a_rd) as DynRead,
            Box::pin(a_wr) as DynWrite,
        );
        let alert = serde_json::to_string(&SandboxResponse::Alert {
            level: AlertLevel::Critical,
            message: "malformed request".to_string(),
        })
        .unwrap();
        b_wr.write_all(format!("{alert}\n").as_bytes())
            .await
            .expect("peer write");
        b_wr.flush().await.expect("flush");
        drop(b_rd);

        let client = SandboxClient {
            inner: tokio::sync::Mutex::new(conn),
        };
        let result = client
            .validate("code".to_string(), pure_read_declaration())
            .await;
        assert!(
            matches!(result, Err(SandboxIpcError::Codec(_))),
            "got {result:?}"
        );
    }

    /// Stream that ends without a matching response surfaces as an
    /// error rather than hanging — pairs with `next_response` returning
    /// `None` on EOF.
    #[tokio::test(start_paused = true)]
    async fn validate_stream_close_surfaces_as_error_not_hang() {
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

        let client = SandboxClient {
            inner: tokio::sync::Mutex::new(conn),
        };
        let task = tokio::spawn(async move {
            client
                .validate("code".to_string(), pure_read_declaration())
                .await
        });
        for _ in 0..6 {
            tokio::time::advance(Duration::from_millis(700)).await;
            tokio::task::yield_now().await;
        }
        let result = task.await.expect("join");
        assert!(
            matches!(
                result,
                Err(SandboxIpcError::Codec(_) | SandboxIpcError::Disconnected { .. })
            ),
            "got: {result:?}"
        );
    }

    /// `await_validation_result` returns `Io(TimedOut)` when the peer
    /// keeps the stream open but never writes a matching response.
    /// Mirrors `drone_ipc::client::await_event_timeout_when_peer_silent`.
    #[tokio::test(start_paused = true)]
    async fn validate_timeout_when_peer_silent() {
        let (a, b) = tokio::io::duplex(4096);
        let (a_rd, a_wr) = tokio::io::split(a);
        let (b_rd, b_wr) = tokio::io::split(b);
        let conn = Connection::from_streams(
            "/test",
            Box::pin(a_rd) as DynRead,
            Box::pin(a_wr) as DynWrite,
        );
        let client = SandboxClient {
            inner: tokio::sync::Mutex::new(conn),
        };
        let task = tokio::spawn(async move {
            client
                .validate("code".to_string(), pure_read_declaration())
                .await
        });
        for _ in 0..7 {
            tokio::time::advance(Duration::from_secs(1)).await;
            tokio::task::yield_now().await;
        }
        let result = task.await.expect("join");
        assert!(
            matches!(
                &result,
                Err(SandboxIpcError::Io(e)) if e.kind() == std::io::ErrorKind::TimedOut
            ),
            "expected Io(TimedOut), got {result:?}"
        );
        drop(b_rd);
        drop(b_wr);
    }
}
