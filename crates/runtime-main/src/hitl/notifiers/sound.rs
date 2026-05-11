//! `sound` notifier — short audio bell. v0.1 wraps the same ASCII BEL as
//! `terminal_bell` but reports its notifier type as `sound`.
//!
//! Cross-platform sound playback (rodio / cpal / OS-specific WAV) is a
//! v1.0 / M11 ship-prep deliverable; the no-new-deps preference from the
//! phase doc and §0d release scope dictate the v0.1 stub. The notifier
//! type still exists in the schema + matrix so frameworks can opt in;
//! when M11 wires the real audio path, the notifier_type stays stable.
//!
//! Same `*_with` testable-seam shape as `terminal_bell`.

use async_trait::async_trait;

use super::{HitlNotifier, HitlNotifyEvent, NotifierError};

const BEL: &[u8] = b"\x07";

/// Cross-platform sound notifier (v0.1 BEL stub).
pub struct Sound;

impl Sound {
    /// Construct a new notifier. Cheap; carries no state.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for Sound {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HitlNotifier for Sound {
    fn notifier_type(&self) -> &'static str {
        "sound"
    }
    async fn notify(&self, _event: &HitlNotifyEvent) -> Result<(), NotifierError> {
        let mut stderr = std::io::stderr();
        emit_sound_with(&mut stderr)
    }
}

/// Test seam — write the audible bell to a caller-supplied writer.
///
/// # Errors
///
/// Returns [`NotifierError::Dispatch`] if the write fails.
pub fn emit_sound_with<W: std::io::Write>(w: &mut W) -> Result<(), NotifierError> {
    w.write_all(BEL)
        .and_then(|()| w.flush())
        .map_err(|e| NotifierError::Dispatch(format!("stderr write failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::event::HitlTriggerRef;

    fn sample_event() -> HitlNotifyEvent {
        HitlNotifyEvent {
            trigger: HitlTriggerRef::OnFailureThreshold,
            session_id: "s".into(),
            prompt_id: "u".into(),
            question: "?".into(),
            options: Vec::new(),
            timeout_at_unix_ms: 0,
        }
    }

    #[test]
    fn emit_sound_with_writes_bel_byte() {
        let mut buf: Vec<u8> = Vec::new();
        emit_sound_with(&mut buf).expect("emit");
        assert_eq!(buf, vec![0x07]);
    }

    #[test]
    fn emit_sound_with_propagates_writer_error() {
        struct FailingWriter;
        impl std::io::Write for FailingWriter {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("device unavailable"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let err = emit_sound_with(&mut FailingWriter).unwrap_err();
        assert!(matches!(err, NotifierError::Dispatch(s) if s.contains("device unavailable")));
    }

    #[test]
    fn notifier_type_is_sound() {
        assert_eq!(Sound::new().notifier_type(), "sound");
        assert_eq!(<Sound as Default>::default().notifier_type(), "sound");
    }

    #[tokio::test]
    async fn notify_writes_to_stderr_and_returns_ok() {
        let n = Sound::new();
        n.notify(&sample_event()).await.expect("notify");
    }
}
