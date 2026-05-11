//! `HitlSeam` — channel-backed gate the SDK awaits on for a HITL response.
//!
//! Mirrors [`crate::sdk::ApprovalSeam`] (Stage B archetype). The SDK calls
//! [`HitlSeam::await_response`] with a `prompt_id`; the renderer's
//! `respond_hitl(prompt_id, choice)` Tauri command calls [`HitlSeam::resolve`]
//! to deliver the user's choice. `await_response` also accepts a timeout;
//! when it elapses, the future resolves to [`HitlError::TimedOut`] and the
//! pending registration is removed.
//!
//! Safety primitive: ≥95% coverage gate per CLAUDE.md §5.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;
use tokio::sync::{oneshot, Mutex};
use tokio::time::timeout;

/// User's choice in response to a HITL prompt.
///
/// `Choice(token)` carries one of the originating [`HitlPrompt::options`]
/// (or free-text when `options` was empty). The renderer chooses; this
/// type does not validate the token against the options list — the
/// originating SDK consumer routes per `token`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitlChoice {
    /// Free-text token chosen by the user. Empty string allowed.
    pub token: String,
}

impl HitlChoice {
    /// Build a new [`HitlChoice`] from any string-like input.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

/// Outstanding HITL request as held by the seam. Carries the originating
/// `prompt_id` + the metadata downstream consumers (renderer + notifiers)
/// need to surface the prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitlPrompt {
    /// Correlation id (UUID) round-tripped via `respond_hitl`.
    pub prompt_id: String,
    /// Human-readable question.
    pub question: String,
    /// Expected choice tokens. Empty means free-text.
    pub options: Vec<String>,
}

/// Errors raised by [`HitlSeam`] operations.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum HitlError {
    /// `resolve` called for a `prompt_id` with no pending `await_response`.
    /// Either the SDK never registered, the resolution already fired, or
    /// the registration was cancelled / timed out.
    #[error("no pending hitl prompt for prompt_id: {0}")]
    NotFound(String),
    /// SDK awaited but the `oneshot` channel was dropped before delivery
    /// (e.g. the seam was dropped or `cancel` was called).
    #[error("hitl channel cancelled before resolution")]
    Cancelled,
    /// `resolve` could not deliver because the SDK had already dropped its
    /// receiver (e.g. timeout fired and the receiver was abandoned).
    #[error("hitl receiver dropped before resolve completed")]
    ReceiverDropped,
    /// `await_response` reached its timeout deadline. The pending
    /// registration has been removed; subsequent `resolve` calls for the
    /// same `prompt_id` return [`HitlError::NotFound`].
    #[error("hitl prompt timed out after {0:?}")]
    TimedOut(Duration),
}

/// Channel-backed HITL gate. Cheap to clone (internally an `Arc`).
#[derive(Clone, Default)]
pub struct HitlSeam {
    pending: Arc<Mutex<HashMap<String, oneshot::Sender<HitlChoice>>>>,
}

impl HitlSeam {
    /// New empty seam.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// SDK calls this to suspend on a pending HITL request. Future resolves
    /// when the renderer (or test harness) calls [`Self::resolve`] for the
    /// same `prompt_id`, OR when `wait` elapses (returns
    /// [`HitlError::TimedOut`]).
    ///
    /// # Errors
    ///
    /// - [`HitlError::Cancelled`] if the `oneshot` sender is dropped before
    ///   delivering a choice (e.g. [`Self::cancel`] called).
    /// - [`HitlError::TimedOut`] if `wait` elapses; the pending registration
    ///   is removed before this returns.
    pub async fn await_response(
        &self,
        prompt_id: &str,
        wait: Duration,
    ) -> Result<HitlChoice, HitlError> {
        let (tx, rx) = oneshot::channel();
        {
            let mut guard = self.pending.lock().await;
            guard.insert(prompt_id.to_string(), tx);
        }
        match timeout(wait, rx).await {
            Ok(Ok(choice)) => Ok(choice),
            Ok(Err(_)) => Err(HitlError::Cancelled),
            Err(_) => {
                // Clean up the pending registration before returning so a
                // late `resolve` returns NotFound instead of ReceiverDropped.
                self.pending.lock().await.remove(prompt_id);
                Err(HitlError::TimedOut(wait))
            }
        }
    }

    /// Renderer (or test harness) calls this to deliver the user's choice.
    /// Removes the registration from the pending map; second call for the
    /// same `prompt_id` returns [`HitlError::NotFound`].
    ///
    /// # Errors
    ///
    /// - [`HitlError::NotFound`] when no pending await is registered for
    ///   `prompt_id`.
    /// - [`HitlError::ReceiverDropped`] when the SDK's receiver was already
    ///   dropped (e.g. timeout fired between the registration cleanup and
    ///   this call).
    pub async fn resolve(&self, prompt_id: &str, choice: HitlChoice) -> Result<(), HitlError> {
        let tx = {
            let mut guard = self.pending.lock().await;
            guard
                .remove(prompt_id)
                .ok_or_else(|| HitlError::NotFound(prompt_id.to_string()))?
        };
        tx.send(choice).map_err(|_| HitlError::ReceiverDropped)
    }

    /// Cancel a pending HITL request without delivering a choice. The
    /// awaiting future will resolve to [`HitlError::Cancelled`]. No-op if
    /// no pending await is registered for `prompt_id`.
    pub async fn cancel(&self, prompt_id: &str) {
        let mut guard = self.pending.lock().await;
        guard.remove(prompt_id);
        // Sender dropped → receiver wakes with Cancelled.
    }

    /// Number of pending awaits. Useful for diagnostics + test assertions.
    #[must_use]
    pub async fn pending_len(&self) -> usize {
        self.pending.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout as tokio_timeout;

    const FAST: Duration = Duration::from_secs(10);

    async fn wait_for_pending(seam: &HitlSeam, n: usize) {
        for _ in 0..100 {
            if seam.pending_len().await == n {
                return;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        panic!("pending_len did not reach {n}");
    }

    #[tokio::test]
    async fn await_then_resolve_carries_choice_token() {
        let seam = HitlSeam::new();
        let s2 = seam.clone();
        let task = tokio::spawn(async move { s2.await_response("p1", FAST).await });

        wait_for_pending(&seam, 1).await;
        seam.resolve("p1", HitlChoice::new("skip")).await.unwrap();

        let res = tokio_timeout(FAST, task).await.unwrap().unwrap().unwrap();
        assert_eq!(res.token, "skip");
        assert_eq!(seam.pending_len().await, 0);
    }

    #[tokio::test]
    async fn await_times_out_after_wait() {
        let seam = HitlSeam::new();
        let res = seam
            .await_response("p1", Duration::from_millis(20))
            .await
            .unwrap_err();
        assert!(matches!(res, HitlError::TimedOut(_)));
        // Timeout path must clean up the pending registration so a later
        // resolve returns NotFound, not ReceiverDropped.
        assert_eq!(seam.pending_len().await, 0);
    }

    #[tokio::test]
    async fn resolve_after_timeout_returns_not_found() {
        let seam = HitlSeam::new();
        let _ = seam
            .await_response("p1", Duration::from_millis(20))
            .await
            .unwrap_err();
        let err = seam
            .resolve("p1", HitlChoice::new("retry"))
            .await
            .unwrap_err();
        assert!(matches!(err, HitlError::NotFound(p) if p == "p1"));
    }

    #[tokio::test]
    async fn resolve_before_await_returns_not_found() {
        let seam = HitlSeam::new();
        let err = seam
            .resolve("ghost", HitlChoice::new("skip"))
            .await
            .unwrap_err();
        assert!(matches!(err, HitlError::NotFound(p) if p == "ghost"));
    }

    #[tokio::test]
    async fn double_resolve_returns_not_found_on_second() {
        let seam = HitlSeam::new();
        let s2 = seam.clone();
        let task = tokio::spawn(async move { s2.await_response("p1", FAST).await });
        wait_for_pending(&seam, 1).await;
        seam.resolve("p1", HitlChoice::new("retry")).await.unwrap();
        let _ = task.await.unwrap();
        let err = seam
            .resolve("p1", HitlChoice::new("retry"))
            .await
            .unwrap_err();
        assert!(matches!(err, HitlError::NotFound(_)));
    }

    #[tokio::test]
    async fn cancel_drops_sender_and_awaiting_returns_cancelled() {
        let seam = HitlSeam::new();
        let s2 = seam.clone();
        let task = tokio::spawn(async move { s2.await_response("p1", FAST).await });
        wait_for_pending(&seam, 1).await;
        seam.cancel("p1").await;
        let res = tokio_timeout(FAST, task).await.unwrap().unwrap();
        assert!(matches!(res, Err(HitlError::Cancelled)));
    }

    #[tokio::test]
    async fn cancel_for_unknown_prompt_is_noop() {
        let seam = HitlSeam::new();
        seam.cancel("unknown").await;
        assert_eq!(seam.pending_len().await, 0);
    }

    #[tokio::test]
    async fn concurrent_awaits_on_different_prompt_ids() {
        let seam = HitlSeam::new();
        let sa = seam.clone();
        let sb = seam.clone();
        let task_a = tokio::spawn(async move { sa.await_response("a", FAST).await });
        let task_b = tokio::spawn(async move { sb.await_response("b", FAST).await });
        wait_for_pending(&seam, 2).await;
        seam.resolve("b", HitlChoice::new("retry")).await.unwrap();
        seam.resolve("a", HitlChoice::new("skip")).await.unwrap();
        let ra = task_a.await.unwrap().unwrap();
        let rb = task_b.await.unwrap().unwrap();
        assert_eq!(ra.token, "skip");
        assert_eq!(rb.token, "retry");
    }

    #[tokio::test]
    async fn resolve_after_receiver_dropped_returns_receiver_dropped() {
        let seam = HitlSeam::new();
        let (sender, receiver) = oneshot::channel::<HitlChoice>();
        drop(receiver);
        seam.pending.lock().await.insert("p1".into(), sender);
        let err = seam
            .resolve("p1", HitlChoice::new("skip"))
            .await
            .unwrap_err();
        assert!(matches!(err, HitlError::ReceiverDropped));
    }

    #[test]
    fn errors_format_with_useful_text() {
        assert!(HitlError::NotFound("p1".into()).to_string().contains("p1"));
        assert!(HitlError::Cancelled.to_string().contains("cancel"));
        assert!(HitlError::ReceiverDropped.to_string().contains("receiver"));
        assert!(HitlError::TimedOut(Duration::from_secs(1))
            .to_string()
            .contains("timed out"));
    }

    #[test]
    fn hitl_choice_new_carries_token() {
        let c = HitlChoice::new("retry");
        assert_eq!(c.token, "retry");
        let c2: HitlChoice = HitlChoice::new(String::from("skip"));
        assert_eq!(c2.token, "skip");
    }

    #[test]
    fn hitl_prompt_carries_metadata() {
        let p = HitlPrompt {
            prompt_id: "u-1".into(),
            question: "Continue?".into(),
            options: vec!["retry".into(), "skip".into()],
        };
        assert_eq!(p.prompt_id, "u-1");
        assert_eq!(p.options.len(), 2);
    }
}
