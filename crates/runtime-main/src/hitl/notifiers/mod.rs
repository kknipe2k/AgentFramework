//! HITL notifier plugin interface + 3 built-in notifiers.
//!
//! Per spec §6a: notifiers are plugins called when any enabled HITL trigger
//! fires.
//!
//! Built-in v1: `terminal_bell`, `desktop`, `sound`. Plugin notifiers
//! (Slack, email, custom) load from `notifiers/` dir under §8.security; M9
//! generators wire the plugin loader. v0.1 ships only the 3 built-ins;
//! [`NotifierRegistry::build`] returns [`NotifierError::PluginNotSupported`]
//! for `plugin` type.
//!
//! Notifier failures are NON-FATAL (spec §6a): the HITL seam still
//! resolves on user response or timeout regardless of which notifiers fired.
//! [`NotifierRegistry::dispatch_all`] dispatches in parallel and returns
//! per-notifier outcomes; callers translate to `notifier_dispatched` /
//! `notifier_failed` events.

/// `desktop` notifier — Tauri 2.x notification plugin wrapper.
pub mod desktop;
/// `sound` notifier — short tone via the OS audio bell (v0.1 BEL stub).
///
/// Same audible-bell stub as `terminal_bell` but with explicit `Sound`
/// notifier type. Cross-platform sound playback (rodio / cpal) is deferred
/// to v1.0 / M11; the notifier_type stays stable when M11 wires real audio.
pub mod sound;
/// `terminal_bell` notifier — writes ASCII BEL (\x07) to stderr.
pub mod terminal_bell;

use async_trait::async_trait;
use runtime_core::event::HitlTriggerRef;
use runtime_core::generated::hitl::{HitlNotifier as HitlNotifierConfig, HitlNotifierType};
use thiserror::Error;

/// Notifier dispatch payload — spec §6a `HitlNotifyEvent`. Carries
/// everything a notifier needs to surface the HITL request out-of-band
/// (terminal, desktop, sound, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitlNotifyEvent {
    /// Which trigger fired.
    pub trigger: HitlTriggerRef,
    /// Session id (carried for diagnostics + plugin routing).
    pub session_id: String,
    /// Correlation id for the originating `hitl_requested`.
    pub prompt_id: String,
    /// User-facing question.
    pub question: String,
    /// Expected choice tokens. Empty means free-text.
    pub options: Vec<String>,
    /// Wall-clock deadline (unix milliseconds).
    pub timeout_at_unix_ms: u64,
}

/// Errors a notifier may raise. Non-fatal at the seam level (the seam
/// resolves on user response / timeout regardless) — surfaced via
/// `notifier_failed` events for renderer diagnostics.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum NotifierError {
    /// Dispatch raised an error (e.g. desktop notification permission denied,
    /// terminal stderr closed, sound device unavailable).
    #[error("notifier dispatch failed: {0}")]
    Dispatch(String),
    /// `plugin` notifier type encountered. v0.1 plugin loader returns
    /// `NotImplemented` per M9 deferral; M9 generators wire the loader.
    #[error("plugin notifier not supported in v0.1: {0}")]
    PluginNotSupported(String),
}

/// Per-notifier dispatch outcome. Carries the notifier type for the
/// downstream `notifier_dispatched` / `notifier_failed` event mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifierOutcome {
    /// Notifier type as the string discriminator (e.g. `terminal_bell`).
    pub notifier_type: String,
    /// `Ok(())` on success; `Err(NotifierError)` on failure.
    pub result: Result<(), NotifierError>,
}

/// Notifier plugin trait. Built-in notifiers + plugin notifiers (M9)
/// implement this; [`NotifierRegistry`] dispatches in parallel.
///
/// `Send + Sync` so the registry can dispatch from a tokio runtime.
#[async_trait]
pub trait HitlNotifier: Send + Sync {
    /// Notifier type as the string discriminator (matches
    /// `HitlNotifierType` serde representation).
    fn notifier_type(&self) -> &'static str;
    /// Fire the notifier for `event`. Errors are non-fatal at the seam;
    /// caller maps to `notifier_failed` event.
    async fn notify(&self, event: &HitlNotifyEvent) -> Result<(), NotifierError>;
}

/// Registry of configured notifiers. Constructed from the framework JSON's
/// `hitl_policy.notifiers` list; only `enabled = true` entries are dispatched.
pub struct NotifierRegistry {
    notifiers: Vec<Box<dyn HitlNotifier>>,
}

impl std::fmt::Debug for NotifierRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NotifierRegistry")
            .field(
                "notifier_types",
                &self
                    .notifiers
                    .iter()
                    .map(|n| n.notifier_type())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl NotifierRegistry {
    /// Empty registry. Useful for tests + frameworks with no notifiers.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            notifiers: Vec::new(),
        }
    }

    /// Build the registry from a framework JSON `notifiers` list. Skips
    /// `enabled = false` entries. Returns [`NotifierError::PluginNotSupported`]
    /// for `plugin` type per the v0.1 / M9 deferral.
    ///
    /// # Errors
    ///
    /// - [`NotifierError::PluginNotSupported`] if any entry has type
    ///   `plugin`. v0.1 ships only the 3 built-ins.
    pub fn build(configs: &[HitlNotifierConfig]) -> Result<Self, NotifierError> {
        let mut notifiers: Vec<Box<dyn HitlNotifier>> = Vec::new();
        for cfg in configs {
            if !cfg.enabled {
                continue;
            }
            let notifier: Box<dyn HitlNotifier> = match cfg.type_ {
                HitlNotifierType::TerminalBell => Box::new(terminal_bell::TerminalBell::new()),
                HitlNotifierType::Sound => Box::new(sound::Sound::new()),
                HitlNotifierType::Desktop => Box::new(desktop::Desktop::new()),
                HitlNotifierType::Plugin => {
                    return Err(NotifierError::PluginNotSupported(
                        cfg.name.clone().unwrap_or_else(|| "<unnamed>".to_string()),
                    ));
                }
            };
            notifiers.push(notifier);
        }
        Ok(Self { notifiers })
    }

    /// Dispatch the event to all configured notifiers in parallel. Returns
    /// per-notifier outcomes in registration order. Errors are non-fatal:
    /// every notifier is dispatched regardless of which ones fail. Callers
    /// translate outcomes to `notifier_dispatched` / `notifier_failed`
    /// events.
    pub async fn dispatch_all(&self, event: &HitlNotifyEvent) -> Vec<NotifierOutcome> {
        use futures::future::join_all;
        let futures = self.notifiers.iter().map(|n| async move {
            let result = n.notify(event).await;
            NotifierOutcome {
                notifier_type: n.notifier_type().to_string(),
                result,
            }
        });
        join_all(futures).await
    }

    /// Number of configured (enabled) notifiers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.notifiers.len()
    }

    /// `true` if no notifiers are configured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.notifiers.is_empty()
    }

    /// Test-only helper: push a notifier without going through
    /// [`Self::build`]. Used by integration tests that want to observe
    /// dispatch with a stub notifier alongside the built-in registry.
    #[doc(hidden)]
    pub fn push_notifier_for_test(&mut self, notifier: Box<dyn HitlNotifier>) {
        self.notifiers.push(notifier);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn sample_event() -> HitlNotifyEvent {
        HitlNotifyEvent {
            trigger: HitlTriggerRef::OnFailureThreshold,
            session_id: "s1".into(),
            prompt_id: "u-1".into(),
            question: "Continue?".into(),
            options: vec!["retry".into(), "skip".into()],
            timeout_at_unix_ms: 1_000_000_000,
        }
    }

    /// In-memory notifier that counts dispatches + optionally fails.
    struct CountingNotifier {
        name: &'static str,
        counter: Arc<AtomicUsize>,
        fail: bool,
    }

    #[async_trait]
    impl HitlNotifier for CountingNotifier {
        fn notifier_type(&self) -> &'static str {
            self.name
        }
        async fn notify(&self, _event: &HitlNotifyEvent) -> Result<(), NotifierError> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                Err(NotifierError::Dispatch("boom".into()))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn build_empty_when_no_configs() {
        let r = NotifierRegistry::build(&[]).unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn build_skips_disabled_entries() {
        let configs = vec![HitlNotifierConfig {
            type_: HitlNotifierType::TerminalBell,
            enabled: false,
            name: None,
            config: ::serde_json::Map::new(),
        }];
        let r = NotifierRegistry::build(&configs).unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn build_includes_each_built_in_type() {
        let configs = vec![
            HitlNotifierConfig {
                type_: HitlNotifierType::TerminalBell,
                enabled: true,
                name: None,
                config: ::serde_json::Map::new(),
            },
            HitlNotifierConfig {
                type_: HitlNotifierType::Desktop,
                enabled: true,
                name: None,
                config: ::serde_json::Map::new(),
            },
            HitlNotifierConfig {
                type_: HitlNotifierType::Sound,
                enabled: true,
                name: None,
                config: ::serde_json::Map::new(),
            },
        ];
        let r = NotifierRegistry::build(&configs).unwrap();
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn build_rejects_plugin_type_with_plugin_not_supported() {
        let configs = vec![HitlNotifierConfig {
            type_: HitlNotifierType::Plugin,
            enabled: true,
            name: Some("slack-webhook".into()),
            config: ::serde_json::Map::new(),
        }];
        let err = NotifierRegistry::build(&configs).expect_err("plugin must reject");
        match err {
            NotifierError::PluginNotSupported(name) => assert_eq!(name, "slack-webhook"),
            NotifierError::Dispatch(_) => panic!("expected PluginNotSupported, got Dispatch"),
        }
    }

    #[test]
    fn build_rejects_unnamed_plugin_with_placeholder() {
        let configs = vec![HitlNotifierConfig {
            type_: HitlNotifierType::Plugin,
            enabled: true,
            name: None,
            config: ::serde_json::Map::new(),
        }];
        let err = NotifierRegistry::build(&configs).expect_err("plugin must reject");
        assert!(err.to_string().contains("<unnamed>"));
    }

    #[tokio::test]
    async fn dispatch_all_fires_every_notifier_in_parallel() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut registry = NotifierRegistry::empty();
        registry.notifiers.push(Box::new(CountingNotifier {
            name: "stub_a",
            counter: Arc::clone(&counter),
            fail: false,
        }));
        registry.notifiers.push(Box::new(CountingNotifier {
            name: "stub_b",
            counter: Arc::clone(&counter),
            fail: false,
        }));
        let outcomes = registry.dispatch_all(&sample_event()).await;
        assert_eq!(outcomes.len(), 2);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
        assert!(outcomes.iter().all(|o| o.result.is_ok()));
        assert_eq!(outcomes[0].notifier_type, "stub_a");
        assert_eq!(outcomes[1].notifier_type, "stub_b");
    }

    #[tokio::test]
    async fn dispatch_all_continues_when_one_notifier_fails() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut registry = NotifierRegistry::empty();
        registry.notifiers.push(Box::new(CountingNotifier {
            name: "stub_a",
            counter: Arc::clone(&counter),
            fail: true,
        }));
        registry.notifiers.push(Box::new(CountingNotifier {
            name: "stub_b",
            counter: Arc::clone(&counter),
            fail: false,
        }));
        let outcomes = registry.dispatch_all(&sample_event()).await;
        assert_eq!(outcomes.len(), 2);
        // Both notifiers ran; the first failed, the second succeeded.
        assert_eq!(counter.load(Ordering::SeqCst), 2);
        assert!(outcomes[0].result.is_err());
        assert!(outcomes[1].result.is_ok());
    }

    #[test]
    fn errors_have_useful_display() {
        assert!(NotifierError::Dispatch("io".into())
            .to_string()
            .contains("io"));
        assert!(NotifierError::PluginNotSupported("slack".into())
            .to_string()
            .contains("slack"));
    }
}
