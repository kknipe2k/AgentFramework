//! Per-kind constructors for `AuditEntry` — spec §8.security L5
//! (M05 Stage E).
//!
//! The on-disk shape is the schema-generated
//! `runtime_core::generated::audit::AuditEntry`. This module wraps it
//! with per-kind builder fns that pin the `details` shape at the call
//! site so renderers + future maintainers grep `skills.audit.jsonl` by
//! `kind` and find a consistent payload for each.
//!
//! v0.1 emits six kinds: `framework_loaded`, `gap_detected`,
//! `gap_resolved`, `capability_granted`, `capability_denied`,
//! `tier_transition`. Each constructor takes the call-site primitives
//! and returns an `AuditEntry` ready to hand to
//! `crate::audit::AuditWriter::log`.

use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use runtime_core::generated::audit::{AuditEntry, AuditEntryKind, AuditSessionId};
use serde_json::{json, Map, Value};

use crate::capability::DenyReason;
use crate::tier::Tier;

/// Wall-clock unix milliseconds.
///
/// Per phase doc E.3.2 the entry carries `timestamp_unix_ms`; we
/// capture it at entry-construction so the recorded time matches when
/// the security decision was made, not when the writer's mutex
/// acquired the file handle. Pre-1970 system clocks fall back to 0 —
/// same posture as `tier::persistence::now_unix_ms`.
#[must_use]
pub fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

fn session_id(raw: &str) -> AuditSessionId {
    // The `AuditSessionId` newtype validates non-empty. Empty session ids
    // would be a programmer error at the call site; we substitute a
    // sentinel to keep audit availability best-effort (failing here
    // would defeat the §13.5 best-effort posture).
    AuditSessionId::from_str(raw)
        .unwrap_or_else(|_| AuditSessionId::from_str("unknown-session").expect("non-empty literal"))
}

fn entry(session: &str, kind: AuditEntryKind, details: Map<String, Value>) -> AuditEntry {
    AuditEntry {
        timestamp_unix_ms: now_unix_ms(),
        session_id: session_id(session),
        kind,
        details,
    }
}

fn details(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(m) => m,
        _ => Map::new(),
    }
}

/// `framework_loaded` — emitted by Stage A's `framework_loader`.
///
/// Fires on every successful `load_and_validate`. Records the path + the
/// agent count (a coarse shape signal that's safe to log; framework
/// names go into `framework_name`).
#[must_use]
pub fn framework_loaded(session: &str, framework_name: &str, agent_count: usize) -> AuditEntry {
    entry(
        session,
        AuditEntryKind::FrameworkLoaded,
        details(json!({
            "framework_name": framework_name,
            "agent_count": agent_count,
        })),
    )
}

/// `gap_detected` — emitted by Stage A's `framework_loader` walker.
///
/// Fires when an unresolved reference is found. Records the agent + the
/// gap kind + the missing name so the renderer's `GapPanel` + a human
/// maintainer share the same trace.
#[must_use]
pub fn gap_detected(
    session: &str,
    agent_id: &str,
    gap_kind: &str,
    missing_name: &str,
    requested_via: &str,
) -> AuditEntry {
    entry(
        session,
        AuditEntryKind::GapDetected,
        details(json!({
            "agent_id": agent_id,
            "gap_kind": gap_kind,
            "missing_name": missing_name,
            "requested_via": requested_via,
        })),
    )
}

/// `gap_resolved` — emitted when a previously detected gap is no longer
/// unresolved (e.g., the user installed the missing tool).
#[must_use]
pub fn gap_resolved(session: &str, agent_id: &str, gap_kind: &str, capability: &str) -> AuditEntry {
    entry(
        session,
        AuditEntryKind::GapResolved,
        details(json!({
            "agent_id": agent_id,
            "gap_kind": gap_kind,
            "capability": capability,
        })),
    )
}

/// `capability_granted` — emitted by Stage B's capability enforcer.
///
/// Fires when a `grant()` call adds a capability to an agent's set, or
/// when a successful `check()` confirms a pre-existing grant. Records
/// the agent + the kind + the resource so the audit log captures the
/// authorization chain.
#[must_use]
pub fn capability_granted(
    session: &str,
    agent_id: &str,
    capability_kind: &str,
    resource: &str,
) -> AuditEntry {
    entry(
        session,
        AuditEntryKind::CapabilityGranted,
        details(json!({
            "agent_id": agent_id,
            "capability_kind": capability_kind,
            "resource": resource,
        })),
    )
}

/// `capability_denied` — emitted on rejected `check()` calls.
///
/// Records the agent + the kind + the deny reason so the audit log
/// distinguishes "no declarations" from "no matching grant" (same
/// `DenyReason` discriminator the renderer routes on).
#[must_use]
pub fn capability_denied(
    session: &str,
    agent_id: &str,
    capability_kind: &str,
    reason: DenyReason,
) -> AuditEntry {
    let reason_str = match reason {
        DenyReason::NoDeclarations => "no_declarations",
        DenyReason::NoMatchingGrant => "no_matching_grant",
    };
    entry(
        session,
        AuditEntryKind::CapabilityDenied,
        details(json!({
            "agent_id": agent_id,
            "capability_kind": capability_kind,
            "reason": reason_str,
        })),
    )
}

/// `mcp_installed` — emitted by M06.C `McpClient::add_server` on success.
///
/// Records server name + transport discriminant + presence-of-auth so the
/// audit consumer can show installation history without exposing secret
/// values. Per gotcha #66 correlation: `add_server` with auth emits BOTH
/// `mcp_installed` AND `mcp_auth_granted` in order.
#[must_use]
pub fn mcp_installed(
    _session: &str,
    _name: &str,
    _transport_kind: &str,
    _has_auth: bool,
) -> AuditEntry {
    todo!("M06.C green phase")
}

/// `mcp_uninstalled` — emitted by M06.C `McpClient::remove_server` on success.
///
/// Records server name only — transport / auth-state are stripped on
/// removal so the audit trail reflects the post-state.
#[must_use]
pub fn mcp_uninstalled(_session: &str, _name: &str) -> AuditEntry {
    todo!("M06.C green phase")
}

/// `mcp_auth_granted` — emitted by M06.C when a per-server secret is
/// stored. The secret value is NEVER logged; only the server name lands
/// in the audit details. Spec §13.5 zero-secret-logging discipline.
#[must_use]
pub fn mcp_auth_granted(_session: &str, _name: &str) -> AuditEntry {
    todo!("M06.C green phase")
}

/// `tier_transition` — emitted when the user's tier changes.
///
/// Records both sides of the flip + the human-readable reason so the
/// audit log captures the full transition chain.
#[must_use]
pub fn tier_transition(session: &str, previous: Tier, current: Tier, reason: &str) -> AuditEntry {
    entry(
        session,
        AuditEntryKind::TierTransition,
        details(json!({
            "previous": tier_str(previous),
            "current": tier_str(current),
            "reason": reason,
        })),
    )
}

const fn tier_str(t: Tier) -> &'static str {
    match t {
        Tier::Novice => "novice",
        Tier::Promoted => "promoted",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_unix_ms_returns_post_epoch() {
        let now = now_unix_ms();
        assert!(
            now > 1_700_000_000_000,
            "clock returned pre-2024 value: {now}"
        );
    }

    #[test]
    fn framework_loaded_entry_has_expected_shape() {
        let e = framework_loaded("sess-1", "aria", 3);
        assert!(matches!(e.kind, AuditEntryKind::FrameworkLoaded));
        assert_eq!(e.session_id.as_str(), "sess-1");
        assert_eq!(e.details.get("framework_name").unwrap(), "aria");
        assert_eq!(e.details.get("agent_count").unwrap(), 3);
    }

    #[test]
    fn gap_detected_entry_carries_gap_kind() {
        let e = gap_detected("sess-1", "worker", "tool_missing", "fetch_prs", "loader");
        assert!(matches!(e.kind, AuditEntryKind::GapDetected));
        assert_eq!(e.details.get("agent_id").unwrap(), "worker");
        assert_eq!(e.details.get("gap_kind").unwrap(), "tool_missing");
        assert_eq!(e.details.get("missing_name").unwrap(), "fetch_prs");
        assert_eq!(e.details.get("requested_via").unwrap(), "loader");
    }

    #[test]
    fn gap_resolved_entry_carries_capability() {
        let e = gap_resolved("sess-1", "worker", "tool_missing", "fetch_prs");
        assert!(matches!(e.kind, AuditEntryKind::GapResolved));
        assert_eq!(e.details.get("capability").unwrap(), "fetch_prs");
    }

    #[test]
    fn capability_granted_entry_carries_resource() {
        let e = capability_granted("sess-1", "worker", "read", "src/**");
        assert!(matches!(e.kind, AuditEntryKind::CapabilityGranted));
        assert_eq!(e.details.get("resource").unwrap(), "src/**");
    }

    #[test]
    fn capability_denied_serializes_no_declarations_reason() {
        let e = capability_denied("sess-1", "worker", "read", DenyReason::NoDeclarations);
        assert!(matches!(e.kind, AuditEntryKind::CapabilityDenied));
        assert_eq!(e.details.get("reason").unwrap(), "no_declarations");
    }

    #[test]
    fn capability_denied_serializes_no_matching_grant_reason() {
        let e = capability_denied("sess-1", "worker", "write", DenyReason::NoMatchingGrant);
        assert_eq!(e.details.get("reason").unwrap(), "no_matching_grant");
    }

    #[test]
    fn tier_transition_entry_records_both_sides() {
        let e = tier_transition(
            "sess-1",
            Tier::Novice,
            Tier::Promoted,
            "user accepted prompt",
        );
        assert!(matches!(e.kind, AuditEntryKind::TierTransition));
        assert_eq!(e.details.get("previous").unwrap(), "novice");
        assert_eq!(e.details.get("current").unwrap(), "promoted");
        assert_eq!(e.details.get("reason").unwrap(), "user accepted prompt");
    }

    #[test]
    fn entry_serializes_to_compact_jsonl_line() {
        // Per phase doc E.3.2: one entry per line. The serialized form must
        // be compact JSON (no embedded newlines from pretty-printing) so
        // the writer's `write_all(line) + write_all(b"\n")` produces a
        // valid JSONL line.
        let e = framework_loaded("sess-1", "aria", 1);
        let s = serde_json::to_string(&e).expect("serialize");
        assert!(
            !s.contains('\n'),
            "compact JSON must have no embedded newlines: {s}"
        );
        // Round-trip through JSON parser confirms shape.
        let v: serde_json::Value = serde_json::from_str(&s).expect("parse");
        assert_eq!(v["kind"], "framework_loaded");
        assert_eq!(v["session_id"], "sess-1");
    }

    #[test]
    fn empty_session_id_falls_back_to_unknown_session() {
        // Audit availability is best-effort — empty session ids would be
        // a programmer error at the call site, but the writer must not
        // panic. The sentinel keeps the audit line landing even when the
        // session id was lost.
        let e = framework_loaded("", "aria", 1);
        assert_eq!(e.session_id.as_str(), "unknown-session");
    }
}
