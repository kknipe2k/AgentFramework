//! Per-event idle timeout for provider event streams (TD-054, review C5).
//!
//! The run loop awaits `stream.next()` with no deadline (`agent_sdk.rs` —
//! deliberately untouched by this stage); a stalled connection therefore
//! parked the session forever. This adapter wraps the provider's stream so
//! every `next()` races a fresh idle timer: the timer RESETS on each
//! yielded event (per-event idle, not a total-stream deadline — long
//! streams stay legitimate), and on expiry the EXISTING
//! [`ProviderError::Timeout`] is constructed and surfaced through the one
//! mid-stream channel the locked trait signature permits —
//! [`ProviderEvent::Error`] — then the stream is FUSED (the inner stream
//! is dropped and never polled again). The loop's existing
//! `ProviderEvent::Error` → `AgentError` → break path is the clean-suspend
//! consumer.
//!
//! This module is the COVERED home for the timeout logic (§6 gate);
//! `providers/anthropic.rs` — the excluded thin shell — only wraps here.

use std::time::Duration;

use futures::stream::Stream;
use futures::StreamExt;

use super::{ProviderError, ProviderEvent};

/// Per-event idle window (TD-054). Generous: model thinking gaps and the
/// API's `ping` events keep healthy streams well under it.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(90);

/// Machine tag carried in the synthetic timeout event's `code`; the
/// plain-language phrase lives in the `message` (the trace renders
/// "{code}: {message}").
pub const IDLE_TIMEOUT_CODE: &str = "provider_idle_timeout";

/// Wrap `stream` so each `next()` is raced against `idle`.
///
/// On expiry the adapter yields one synthetic [`ProviderEvent::Error`]
/// carrying the [`ProviderError::Timeout`] identity (code
/// [`IDLE_TIMEOUT_CODE`], message with the plain-language phrase + the
/// timeout's duration figure), then ends: the inner stream is dropped —
/// fused, never polled again.
pub fn with_idle_timeout<S>(stream: S, idle: Duration) -> impl Stream<Item = ProviderEvent>
where
    S: Stream<Item = ProviderEvent> + Unpin,
{
    // The outer .fuse() makes polling past the end safe (Unfold panics on
    // poll-after-None; consumers like the fused-after-timeout regression
    // legitimately poll again).
    futures::stream::unfold(Some(stream), move |state| async move {
        let mut stream = state?;
        match tokio::time::timeout(idle, stream.next()).await {
            Ok(Some(event)) => Some((event, Some(stream))),
            Ok(None) => None,
            Err(_elapsed) => {
                let timeout = ProviderError::Timeout(idle);
                Some((
                    ProviderEvent::Error {
                        code: IDLE_TIMEOUT_CODE.to_string(),
                        message: format!("provider idle timeout — {timeout}"),
                    },
                    // Fuse at the state level too: the inner stream is
                    // dropped here and never polled again (rider 1b).
                    None,
                ))
            }
        }
    })
    .fuse()
}
