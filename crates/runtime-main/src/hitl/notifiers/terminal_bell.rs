//! `terminal_bell` notifier — writes ASCII BEL (\x07) to stderr.
//!
//! Cross-platform without deps; works in any terminal that honors BEL.
//! Stderr (not stdout) so the bell does not interleave with the renderer's
//! event stream on stdout.
//!
//! Testable via a `*_with` seam (`emit_bell_with`) and the unit tests. The
//! production `TerminalBell::notify` uses `eprintln!`, which is structurally
//! testable in process even though capturing stderr cross-thread is
//! unreliable; the seam injects the writer to keep coverage tight.

use async_trait::async_trait;

use super::{HitlNotifier, HitlNotifyEvent, NotifierError};

/// ASCII BEL byte. Writing this to a terminal emits the bell.
const BEL: &[u8] = b"\x07";

/// Cross-platform terminal-bell notifier.
pub struct TerminalBell;

impl TerminalBell {
    /// Construct a new notifier. Cheap; carries no state.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for TerminalBell {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HitlNotifier for TerminalBell {
    fn notifier_type(&self) -> &'static str {
        "terminal_bell"
    }
    async fn notify(&self, _event: &HitlNotifyEvent) -> Result<(), NotifierError> {
        let mut stderr = std::io::stderr();
        emit_bell_with(&mut stderr)
    }
}

/// Test seam — write `BEL` to a caller-supplied writer. Production
/// [`TerminalBell::notify`] passes `std::io::stderr()`; unit tests pass an
/// in-memory `Vec<u8>` and assert the byte landed.
///
/// # Errors
///
/// Returns [`NotifierError::Dispatch`] if the write fails (e.g. stderr
/// closed).
pub fn emit_bell_with<W: std::io::Write>(w: &mut W) -> Result<(), NotifierError> {
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
    fn emit_bell_with_writes_bel_byte() {
        let mut buf: Vec<u8> = Vec::new();
        emit_bell_with(&mut buf).expect("emit");
        assert_eq!(buf, vec![0x07]);
    }

    #[test]
    fn emit_bell_with_propagates_writer_error() {
        // Custom writer that always fails on write_all.
        struct FailingWriter;
        impl std::io::Write for FailingWriter {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("disk full"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let err = emit_bell_with(&mut FailingWriter).unwrap_err();
        assert!(matches!(err, NotifierError::Dispatch(s) if s.contains("disk full")));
    }

    #[test]
    fn notifier_type_is_terminal_bell() {
        assert_eq!(TerminalBell::new().notifier_type(), "terminal_bell");
        assert_eq!(
            <TerminalBell as Default>::default().notifier_type(),
            "terminal_bell"
        );
    }

    #[tokio::test]
    async fn notify_writes_to_stderr_and_returns_ok() {
        // The production path writes to std::io::stderr() — we can't easily
        // capture that here, but the call must return Ok and not panic.
        let n = TerminalBell::new();
        n.notify(&sample_event()).await.expect("notify");
    }
}
