//! Tier transition primitive — spec §8.security L4 + L5 (M05 Stage E).
//!
//! Records a `tier_transition` audit line when the user's tier flips.
//! Same-tier transitions are idempotent — the audit log skips them so
//! grep + a human reading the file see only real flips.
//!
//! Split from [`crate::tier::evaluator`] because the evaluator is the
//! stateless predicate; transitions are stateful events that need a
//! seam where audit emission can happen.

use std::sync::Arc;

use crate::audit::{self, AuditWriter};
use crate::tier::Tier;

/// Outcome of a [`transition`] call. Carried back to the caller so the
/// renderer-side event emission + persistence can chain off the same
/// `previous`/`current` pair the audit line recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TierTransitionRecord {
    /// Tier the user was on before the call. Equal to `current` for
    /// same-tier (no-op) transitions.
    pub previous: Tier,
    /// Tier the user is on after the call.
    pub current: Tier,
    /// `true` when the call changed tier (audit line was emitted);
    /// `false` for same-tier no-ops.
    pub changed: bool,
}

/// Record a tier transition.
///
/// Writes a `tier_transition` audit line to `writer` when
/// `previous != current` (best-effort — write failures `tracing::error!`
/// and continue). Same-tier transitions are no-ops (no audit line,
/// `changed: false`).
///
/// The caller is responsible for the per-side effects:
/// - Persistence: caller invokes [`crate::tier::save_tier`] to land the
///   new tier on disk.
/// - Renderer state: caller emits the
///   [`runtime_core::event::AgentEvent::TierTransition`] event so the
///   graph store updates.
/// - Capability enforcer: caller invokes
///   [`crate::capability::CapabilityEnforcer::set_tier`] so subsequent
///   `check()` calls see the new tier.
///
/// This function only writes the audit line + returns the record; the
/// chain stays explicit at the call site.
pub async fn transition(
    writer: Option<&Arc<AuditWriter>>,
    session_id: &str,
    previous: Tier,
    current: Tier,
    reason: &str,
) -> TierTransitionRecord {
    let changed = previous != current;
    if changed {
        if let Some(w) = writer {
            let entry = audit::tier_transition(session_id, previous, current, reason);
            if let Err(e) = w.log(&entry).await {
                tracing::error!(error = %e, "audit log write failed");
            }
        }
    }
    TierTransitionRecord {
        previous,
        current,
        changed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn open_writer(dir: &std::path::Path) -> Arc<AuditWriter> {
        let path = dir.join("skills.audit.jsonl");
        Arc::new(AuditWriter::open(&path).await.expect("open"))
    }

    #[tokio::test]
    async fn novice_to_promoted_writes_audit_line() {
        let dir = tempdir().unwrap();
        let writer = open_writer(dir.path()).await;
        let record = transition(
            Some(&writer),
            "sess-1",
            Tier::Novice,
            Tier::Promoted,
            "user accepted prompt",
        )
        .await;
        assert!(record.changed);
        assert_eq!(record.previous, Tier::Novice);
        assert_eq!(record.current, Tier::Promoted);
        let raw = tokio::fs::read_to_string(dir.path().join("skills.audit.jsonl"))
            .await
            .unwrap();
        let lines: Vec<_> = raw.lines().collect();
        assert_eq!(lines.len(), 1);
        let parsed: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed["kind"], "tier_transition");
        assert_eq!(parsed["details"]["previous"], "novice");
        assert_eq!(parsed["details"]["current"], "promoted");
        assert_eq!(parsed["details"]["reason"], "user accepted prompt");
    }

    #[tokio::test]
    async fn promoted_to_novice_writes_audit_line() {
        let dir = tempdir().unwrap();
        let writer = open_writer(dir.path()).await;
        let record = transition(
            Some(&writer),
            "sess-1",
            Tier::Promoted,
            Tier::Novice,
            "user demoted",
        )
        .await;
        assert!(record.changed);
        let raw = tokio::fs::read_to_string(dir.path().join("skills.audit.jsonl"))
            .await
            .unwrap();
        assert!(raw.contains("\"previous\":\"promoted\""));
        assert!(raw.contains("\"current\":\"novice\""));
    }

    #[tokio::test]
    async fn same_tier_transition_is_no_op_no_audit_line() {
        // Per the doc: same-tier transitions are idempotent and skip
        // the audit log so a human reading the file sees only real flips.
        let dir = tempdir().unwrap();
        let writer = open_writer(dir.path()).await;
        let record = transition(
            Some(&writer),
            "sess-1",
            Tier::Novice,
            Tier::Novice,
            "idempotent set",
        )
        .await;
        assert!(!record.changed);
        let raw = tokio::fs::read_to_string(dir.path().join("skills.audit.jsonl"))
            .await
            .unwrap();
        assert!(
            raw.is_empty(),
            "same-tier transition must not emit audit line"
        );
    }

    #[tokio::test]
    async fn transition_without_writer_is_silent_no_op() {
        // Per phase doc E.3.4: audit availability is not a dispatch
        // gate. When no writer is wired, the transition still returns
        // the record (caller can still drive persistence + event
        // emission); only the audit emission is skipped.
        let record = transition(
            None,
            "sess-1",
            Tier::Novice,
            Tier::Promoted,
            "no audit wired",
        )
        .await;
        assert!(record.changed);
        assert_eq!(record.current, Tier::Promoted);
    }

    #[tokio::test]
    async fn three_sequential_transitions_write_three_lines() {
        // Multi-call invariant — sequential flips each land their own
        // line in order. Gotcha #69.
        let dir = tempdir().unwrap();
        let writer = open_writer(dir.path()).await;
        transition(Some(&writer), "s", Tier::Novice, Tier::Promoted, "1").await;
        transition(Some(&writer), "s", Tier::Promoted, Tier::Novice, "2").await;
        transition(Some(&writer), "s", Tier::Novice, Tier::Promoted, "3").await;
        let raw = tokio::fs::read_to_string(dir.path().join("skills.audit.jsonl"))
            .await
            .unwrap();
        let lines: Vec<_> = raw.lines().collect();
        assert_eq!(lines.len(), 3);
        let r1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let r2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        let r3: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
        assert_eq!(r1["details"]["reason"], "1");
        assert_eq!(r2["details"]["reason"], "2");
        assert_eq!(r3["details"]["reason"], "3");
    }
}
