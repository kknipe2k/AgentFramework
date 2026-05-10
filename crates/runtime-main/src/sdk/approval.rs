//! Approval-gate seam — spec §3a (Plan approval primitive).
//!
//! `tokio::sync::oneshot` channel pattern. The SDK awaits on
//! [`ApprovalSeam::await_approval`] when a plan needs HITL approval; the
//! renderer's HITL flow (Stage E wiring) calls [`ApprovalSeam::resolve`]
//! to deliver the user's decision.
//!
//! Stage B authors the seam; Stage E wires the UI. Two-phase landing keeps
//! the SDK's plan-loop tractable in Stage B without blocking on the HITL
//! flow design.
//!
//! Safety primitive: ≥95% coverage gate per CLAUDE.md §5.

use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{oneshot, Mutex};

/// User decision on a pending plan-approval request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalDecision {
    /// Plan approved as-is.
    Approved,
    /// User asked for revisions; SDK regenerates the plan and awaits
    /// approval again. The contained string is free-text the user
    /// supplied describing the requested changes.
    Revised(String),
    /// User cancelled the plan. The contained string is free-text
    /// describing the reason.
    Aborted(String),
}

/// Errors raised by [`ApprovalSeam`] operations.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ApprovalError {
    /// `resolve` called for a `plan_id` with no pending `await_approval`.
    /// Either the SDK never registered, the resolution already fired, or
    /// the registration was cancelled.
    #[error("no pending approval for plan_id: {0}")]
    NotFound(String),
    /// SDK awaited but the channel was dropped before a decision arrived
    /// (e.g., the renderer disappeared mid-flight).
    #[error("approval channel cancelled before resolution")]
    Cancelled,
    /// `resolve` could not deliver because the SDK had already dropped
    /// its receiver (likely due to timeout / cancellation).
    #[error("approval receiver dropped before resolve completed")]
    ReceiverDropped,
}

/// Channel-backed approval gate. Cheap to clone (internally an `Arc`).
#[derive(Clone, Default)]
pub struct ApprovalSeam {
    pending: Arc<Mutex<HashMap<String, oneshot::Sender<ApprovalDecision>>>>,
}

impl ApprovalSeam {
    /// New empty seam.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// SDK calls this to suspend on a pending approval. Future resolves
    /// when the renderer (or a test harness) calls [`Self::resolve`] for
    /// the same `plan_id`.
    ///
    /// # Errors
    ///
    /// Returns [`ApprovalError::Cancelled`] if the `oneshot` sender is
    /// dropped before delivering a decision.
    pub async fn await_approval(&self, plan_id: &str) -> Result<ApprovalDecision, ApprovalError> {
        let (tx, rx) = oneshot::channel();
        {
            let mut guard = self.pending.lock().await;
            guard.insert(plan_id.to_string(), tx);
        }
        rx.await.map_err(|_| ApprovalError::Cancelled)
    }

    /// Renderer (or test harness) calls this to deliver the user's
    /// decision. Removes the registration from the pending map; second
    /// call for the same `plan_id` returns [`ApprovalError::NotFound`].
    ///
    /// # Errors
    ///
    /// - [`ApprovalError::NotFound`] when no pending await is registered
    ///   for `plan_id`.
    /// - [`ApprovalError::ReceiverDropped`] when the SDK's receiver was
    ///   already dropped (e.g., the awaiting task was cancelled).
    pub async fn resolve(
        &self,
        plan_id: &str,
        decision: ApprovalDecision,
    ) -> Result<(), ApprovalError> {
        let tx = {
            let mut guard = self.pending.lock().await;
            guard
                .remove(plan_id)
                .ok_or_else(|| ApprovalError::NotFound(plan_id.to_string()))?
        };
        tx.send(decision)
            .map_err(|_| ApprovalError::ReceiverDropped)
    }

    /// Cancel a pending approval without delivering a decision. The
    /// awaiting future will resolve to [`ApprovalError::Cancelled`].
    /// No-op if no pending await is registered for `plan_id`.
    pub async fn cancel(&self, plan_id: &str) {
        let mut guard = self.pending.lock().await;
        guard.remove(plan_id);
        // Sender dropped → receiver wakes with `Cancelled`.
    }

    /// Number of pending awaits. Useful for diagnostics + test
    /// assertions; not part of the public flow contract.
    #[must_use]
    pub async fn pending_len(&self) -> usize {
        self.pending.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn await_then_resolve_approved() {
        let seam = ApprovalSeam::new();
        let s2 = seam.clone();
        let task = tokio::spawn(async move { s2.await_approval("p1").await });

        // Wait for the awaiting side to register before resolving.
        for _ in 0..50 {
            if seam.pending_len().await == 1 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        seam.resolve("p1", ApprovalDecision::Approved)
            .await
            .unwrap();

        let result = timeout(Duration::from_secs(1), task)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.unwrap(), ApprovalDecision::Approved);
        assert_eq!(seam.pending_len().await, 0);
    }

    #[tokio::test]
    async fn resolve_revised_carries_reason() {
        let seam = ApprovalSeam::new();
        let s2 = seam.clone();
        let task = tokio::spawn(async move { s2.await_approval("p1").await });

        for _ in 0..50 {
            if seam.pending_len().await == 1 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        seam.resolve("p1", ApprovalDecision::Revised("more risk callouts".into()))
            .await
            .unwrap();

        let result = task.await.unwrap().unwrap();
        match result {
            ApprovalDecision::Revised(reason) => assert_eq!(reason, "more risk callouts"),
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[tokio::test]
    async fn resolve_aborted_carries_reason() {
        let seam = ApprovalSeam::new();
        let s2 = seam.clone();
        let task = tokio::spawn(async move { s2.await_approval("p1").await });

        for _ in 0..50 {
            if seam.pending_len().await == 1 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        seam.resolve("p1", ApprovalDecision::Aborted("user cancelled".into()))
            .await
            .unwrap();

        match task.await.unwrap().unwrap() {
            ApprovalDecision::Aborted(reason) => assert_eq!(reason, "user cancelled"),
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[tokio::test]
    async fn resolve_before_await_returns_not_found() {
        let seam = ApprovalSeam::new();
        let err = seam
            .resolve("p1", ApprovalDecision::Approved)
            .await
            .unwrap_err();
        assert!(matches!(err, ApprovalError::NotFound(p) if p == "p1"));
    }

    #[tokio::test]
    async fn double_resolve_returns_not_found_on_second() {
        let seam = ApprovalSeam::new();
        let s2 = seam.clone();
        let task = tokio::spawn(async move { s2.await_approval("p1").await });

        for _ in 0..50 {
            if seam.pending_len().await == 1 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        seam.resolve("p1", ApprovalDecision::Approved)
            .await
            .unwrap();
        let _ = task.await.unwrap();

        let err = seam
            .resolve("p1", ApprovalDecision::Approved)
            .await
            .unwrap_err();
        assert!(matches!(err, ApprovalError::NotFound(_)));
    }

    #[tokio::test]
    async fn concurrent_awaits_on_different_plan_ids() {
        let seam = ApprovalSeam::new();
        let s_a = seam.clone();
        let s_b = seam.clone();
        let task_a = tokio::spawn(async move { s_a.await_approval("a").await });
        let task_b = tokio::spawn(async move { s_b.await_approval("b").await });

        for _ in 0..50 {
            if seam.pending_len().await == 2 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        seam.resolve("b", ApprovalDecision::Approved).await.unwrap();
        seam.resolve("a", ApprovalDecision::Aborted("nope".into()))
            .await
            .unwrap();

        let res_a = task_a.await.unwrap().unwrap();
        let res_b = task_b.await.unwrap().unwrap();
        assert!(matches!(res_a, ApprovalDecision::Aborted(_)));
        assert_eq!(res_b, ApprovalDecision::Approved);
    }

    #[tokio::test]
    async fn cancel_drops_sender_and_awaiting_returns_cancelled() {
        let seam = ApprovalSeam::new();
        let s2 = seam.clone();
        let task = tokio::spawn(async move { s2.await_approval("p1").await });

        for _ in 0..50 {
            if seam.pending_len().await == 1 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        seam.cancel("p1").await;

        let result = timeout(Duration::from_secs(1), task)
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(result, Err(ApprovalError::Cancelled)));
    }

    #[tokio::test]
    async fn cancel_for_unknown_plan_id_is_noop() {
        let seam = ApprovalSeam::new();
        seam.cancel("unknown").await;
        assert_eq!(seam.pending_len().await, 0);
    }

    #[tokio::test]
    async fn resolve_after_receiver_dropped_returns_receiver_dropped() {
        let seam = ApprovalSeam::new();
        // Manually inject a sender whose receiver is already dropped.
        let (sender, receiver) = oneshot::channel::<ApprovalDecision>();
        drop(receiver);
        seam.pending.lock().await.insert("p1".into(), sender);
        let err = seam
            .resolve("p1", ApprovalDecision::Approved)
            .await
            .unwrap_err();
        assert!(matches!(err, ApprovalError::ReceiverDropped));
    }

    #[test]
    fn errors_format_with_useful_text() {
        let e1 = ApprovalError::NotFound("p1".into());
        assert!(e1.to_string().contains("p1"));
        let e2 = ApprovalError::Cancelled;
        assert!(e2.to_string().contains("cancel"));
        let e3 = ApprovalError::ReceiverDropped;
        assert!(e3.to_string().contains("receiver"));
    }
}
