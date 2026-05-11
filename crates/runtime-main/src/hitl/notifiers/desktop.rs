//! `desktop` notifier — surfaces a desktop notification via the Tauri
//! notification plugin (Tauri 2.x).
//!
//! Cross-stack glue point — gotcha #32 verbatim-quote-or-verify discipline
//! applies.
//!
//! The real Tauri-plugin call lives in `src-tauri/src/main.rs` (where the
//! plugin is registered) and is injected into `Desktop` at construction
//! via a boxed async callable. Production wiring builds the boxed callable
//! around `tauri_plugin_notification::NotificationExt::notification()`;
//! unit tests pass an in-memory stub. The split is the same M02 / A2 /
//! M04.D OS-call holdout pattern — testable seam
//! (`Desktop::with_dispatcher`) covered to 95%+, OS-call wrapper
//! exercised only by the production wiring.
//!
//! Per Tauri 2.x notification plugin docs (verified against
//! <https://v2.tauri.app/plugin/notification/> at 2026-05-10):
//!
//! ```rust,ignore
//! // Production wiring (lives in src-tauri/src/main.rs Stage E):
//! use tauri_plugin_notification::NotificationExt;
//! let handle = app_handle.clone();
//! let dispatcher = runtime_main::hitl::notifiers::desktop::dispatcher_from_tauri(handle);
//! ```
//!
//! Notifier failures are NON-FATAL — permission-denied / plugin-error
//! returns [`NotifierError::Dispatch`] and the seam still resolves on
//! user response or timeout regardless.

use async_trait::async_trait;
use std::pin::Pin;
use std::sync::Arc;

use super::{HitlNotifier, HitlNotifyEvent, NotifierError};

/// Boxed async dispatcher type. Production wires this to the Tauri
/// notification plugin's `NotificationExt::notification()` builder; unit
/// tests pass an in-memory stub.
///
/// The closure receives the title + body the notifier composes from the
/// [`HitlNotifyEvent`]; returning `Err` produces a `notifier_failed`
/// event downstream.
pub type DesktopDispatcher = Arc<
    dyn Fn(
            String,
            String,
        ) -> Pin<Box<dyn std::future::Future<Output = Result<(), NotifierError>> + Send>>
        + Send
        + Sync,
>;

/// Desktop notification notifier. Carries a [`DesktopDispatcher`] injected
/// at construction; the dispatcher abstracts whether the call hits the
/// real Tauri plugin or an in-memory stub.
pub struct Desktop {
    dispatcher: DesktopDispatcher,
}

impl Desktop {
    /// Construct a notifier with the default (no-op) dispatcher. Useful
    /// for tests that exercise the registry-build path without a real
    /// Tauri runtime. Production constructs via
    /// [`Desktop::with_dispatcher`] passing a Tauri-aware closure.
    #[must_use]
    pub fn new() -> Self {
        Self::with_dispatcher(default_dispatcher())
    }

    /// Construct with a caller-supplied dispatcher. Production passes a
    /// closure that calls into the Tauri notification plugin; tests pass
    /// an in-memory stub that records the dispatch.
    #[must_use]
    pub fn with_dispatcher(dispatcher: DesktopDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl Default for Desktop {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HitlNotifier for Desktop {
    fn notifier_type(&self) -> &'static str {
        "desktop"
    }
    async fn notify(&self, event: &HitlNotifyEvent) -> Result<(), NotifierError> {
        let (title, body) = compose_title_body(event);
        (self.dispatcher)(title, body).await
    }
}

/// Compose the title + body strings the desktop notification displays.
/// Title is short ("Agent Runtime — HITL") so the OS toast renders;
/// body carries the question (truncated to keep the toast readable).
///
/// Pure logic; covered by unit test without any Tauri runtime.
#[must_use]
pub fn compose_title_body(event: &HitlNotifyEvent) -> (String, String) {
    let title = "Agent Runtime — HITL".to_string();
    // Truncate question at 240 chars so the body stays within typical
    // OS notification body limits (Windows 10/11 toasts: 200-ish; macOS
    // / Linux: similar order of magnitude). 240 is a soft cap; the
    // renderer's Panel / Modal / Toast shows the full text.
    let body = truncate(&event.question, 240);
    (title, body)
}

fn truncate(s: &str, max_chars: usize) -> String {
    let mut acc = String::with_capacity(s.len().min(max_chars + 1));
    for (i, c) in s.chars().enumerate() {
        if i >= max_chars {
            acc.push('…');
            return acc;
        }
        acc.push(c);
    }
    acc
}

/// Default no-op dispatcher used by [`Desktop::new`].
///
/// Returns `Ok(())` immediately — useful for renderer-less tests that
/// exercise the registry-build path. Production wiring overrides via
/// [`Desktop::with_dispatcher`].
#[must_use]
pub fn default_dispatcher() -> DesktopDispatcher {
    Arc::new(|_title: String, _body: String| Box::pin(async { Ok(()) }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::event::HitlTriggerRef;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    fn sample_event() -> HitlNotifyEvent {
        HitlNotifyEvent {
            trigger: HitlTriggerRef::OnFailureThreshold,
            session_id: "s1".into(),
            prompt_id: "u-1".into(),
            question: "Task t-1 exceeded failure budget. Retry, skip, or abort?".into(),
            options: vec!["retry".into(), "skip".into(), "abort".into()],
            timeout_at_unix_ms: 1_000_000_000,
        }
    }

    #[test]
    fn compose_title_body_includes_question_in_body() {
        let (title, body) = compose_title_body(&sample_event());
        assert!(title.contains("Agent Runtime"));
        assert!(body.contains("exceeded failure budget"));
    }

    #[test]
    fn compose_title_body_truncates_long_questions() {
        let mut event = sample_event();
        event.question = "x".repeat(500);
        let (_title, body) = compose_title_body(&event);
        // 240 chars + ellipsis = 241 chars.
        assert_eq!(body.chars().count(), 241);
        assert!(body.ends_with('…'));
    }

    #[test]
    fn compose_title_body_keeps_short_questions_intact() {
        let mut event = sample_event();
        event.question = "short".into();
        let (_title, body) = compose_title_body(&event);
        assert_eq!(body, "short");
    }

    #[test]
    fn compose_title_body_handles_utf8_codepoints_correctly() {
        let mut event = sample_event();
        // 250 emoji codepoints; each is multi-byte. truncate must count
        // chars not bytes so the cap is uniform.
        event.question = "😀".repeat(250);
        let (_title, body) = compose_title_body(&event);
        assert_eq!(body.chars().count(), 241);
    }

    #[tokio::test]
    async fn notify_invokes_dispatcher_with_composed_title_body() {
        let captured: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(Vec::new()));
        let cap = Arc::clone(&captured);
        let dispatcher: DesktopDispatcher = Arc::new(move |title, body| {
            let cap = Arc::clone(&cap);
            Box::pin(async move {
                cap.lock().unwrap().push((title, body));
                Ok(())
            })
        });
        let notifier = Desktop::with_dispatcher(dispatcher);
        notifier.notify(&sample_event()).await.expect("notify");
        let captured_snapshot = captured.lock().unwrap().clone();
        assert_eq!(captured_snapshot.len(), 1);
        assert!(captured_snapshot[0].0.contains("Agent Runtime"));
        assert!(captured_snapshot[0].1.contains("exceeded failure budget"));
    }

    #[tokio::test]
    async fn notify_propagates_dispatcher_error() {
        let dispatcher: DesktopDispatcher = Arc::new(|_title, _body| {
            Box::pin(async { Err(NotifierError::Dispatch("permission denied".into())) })
        });
        let notifier = Desktop::with_dispatcher(dispatcher);
        let err = notifier.notify(&sample_event()).await.unwrap_err();
        assert!(matches!(err, NotifierError::Dispatch(s) if s.contains("permission denied")));
    }

    #[tokio::test]
    async fn notify_dispatcher_called_exactly_once_per_notify() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let dispatcher: DesktopDispatcher = Arc::new(move |_t, _b| {
            let c = Arc::clone(&c);
            Box::pin(async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        });
        let notifier = Desktop::with_dispatcher(dispatcher);
        notifier.notify(&sample_event()).await.expect("first");
        notifier.notify(&sample_event()).await.expect("second");
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn default_dispatcher_returns_ok_and_can_be_invoked() {
        let d = default_dispatcher();
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(d("t".to_string(), "b".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn notifier_type_is_desktop() {
        assert_eq!(Desktop::new().notifier_type(), "desktop");
        assert_eq!(Desktop::default().notifier_type(), "desktop");
    }
}
