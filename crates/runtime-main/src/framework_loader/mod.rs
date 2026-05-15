//! Framework loader — spec §4b Layer 1 gap detection.
//!
//! Parses a framework JSON from disk, walks declared primitives, and emits
//! one gap event per unresolved reference (`tool_missing` / `skill_missing`
//! / `agent_missing`). MCP-server gaps are NOT detected at Layer 1 in v0.1
//! (the framework schema has no MCP-server declaration yet — that lands in
//! M06); the `mcp_missing` event is reserved for Layer 2 (`request_capability`)
//! emission this milestone.
//!
//! Architecture:
//! - `walker::walk` is a pure function over a parsed
//!   `runtime_core::generated::framework::Framework`.
//! - `Emitter` is the SDK-side trait the loader fans gap events out
//!   through. The loader does NOT route to the HITL seam directly — the
//!   SDK consumer chains `on_gap` per spec §6a. This keeps the loader
//!   testable without IO + matches the M04 budget enforcer + HITL seam
//!   in-process pattern (ADR-0007).
//! - `load_and_validate` is the thin async wrapper: read JSON →
//!   `serde_json::from_str` → `walker::walk` → emit per gap → return
//!   `Ok(Framework)` if zero gaps, else `Err(FrameworkLoadError::GapsFound)`.

/// M06.A wire-up support — `capabilities_for_tool` + `parent_grants_for_agent`
/// + `Capabilities → Vec<CapabilityDeclaration>` translation.
pub mod capability_map;
/// Failure types raised by [`load_and_validate`].
pub mod error;
/// Pure-function walker + Gap/GapKind types. Spec §4b Layer 1.
pub mod walker;

pub use capability_map::{
    capabilities_for_tool, capabilities_to_declarations, declaration_to_narrowed_from_str,
    inline_agents, parent_grants_for_agent, CapabilityLookupError, FrameworkRef,
};
pub use error::FrameworkLoadError;
pub use walker::{walk, Gap, GapKind};

use runtime_core::event::AgentEvent;
use runtime_core::event::GapSourceRef;
use std::path::Path;
use std::sync::Arc;

use crate::audit::{self, AuditWriter};

/// In-process event-emission seam.
///
/// Loader (Layer 1) + `request_capability` meta-tool (Layer 2) both fan
/// gap events out through this trait so the SDK consumer (`AgentSdk`)
/// routes per spec §6a `on_gap` trigger.
///
/// Cheap to mock — tests inject a `Vec<AgentEvent>`-collecting impl.
/// Mirrors the M04 budget + HITL in-process emitter pattern; the live
/// SDK impl forwards to the renderer via the Tauri event bus.
#[async_trait::async_trait]
pub trait Emitter: Send + Sync {
    /// Emit one `AgentEvent`. Errors are logged by the implementation —
    /// emission failure does NOT propagate up because per spec §13.5
    /// dev-logging the renderer's view is non-load-bearing for the
    /// loader's correctness contract.
    async fn emit(&self, event: AgentEvent);
}

/// Audit emission seam — M05 Stage E.
///
/// Carries the `Arc<AuditWriter>` + the session id through to
/// [`load_and_validate`] so successful loads emit `framework_loaded`
/// audit lines and each detected gap emits a `gap_detected` audit
/// line. `None` skips audit emission entirely (tests + headless
/// invocations); `Some` wires the production audit trail.
#[derive(Clone, Default)]
pub struct AuditContext {
    /// Audit writer reference. `None` skips audit emission.
    pub writer: Option<Arc<AuditWriter>>,
    /// Session id carried per-entry per phase doc E.3.1.
    pub session_id: String,
}

async fn audit_log(audit_ctx: &AuditContext, entry: runtime_core::generated::audit::AuditEntry) {
    if let Some(writer) = &audit_ctx.writer {
        if let Err(e) = writer.log(&entry).await {
            tracing::error!(error = %e, "audit log write failed");
        }
    }
}

/// Load a framework JSON from `path` + walk it for Layer-1 gaps.
///
/// Convenience wrapper around [`load_and_validate_with_audit`] that
/// skips audit emission. Tests + headless invocations use this; the
/// Tauri shell uses the `_with_audit` variant.
///
/// # Errors
///
/// - [`FrameworkLoadError::Io`] — disk read failed.
/// - [`FrameworkLoadError::Json`] — JSON parse failed.
/// - [`FrameworkLoadError::GapsFound`] — framework parsed but references
///   did not resolve; events were emitted before this returned.
pub async fn load_and_validate(
    path: &Path,
    emitter: &impl Emitter,
) -> Result<runtime_core::generated::framework::Framework, FrameworkLoadError> {
    load_and_validate_with_audit(path, emitter, &AuditContext::default()).await
}

/// Like [`load_and_validate`] but wires audit emission.
///
/// On successful load, emits one `framework_loaded` audit line. On
/// gaps, emits one `gap_detected` audit line per gap before returning
/// the error. Audit failures `tracing::error!` and continue — never
/// propagated.
///
/// # Errors
///
/// Same as [`load_and_validate`].
pub async fn load_and_validate_with_audit(
    path: &Path,
    emitter: &impl Emitter,
    audit_ctx: &AuditContext,
) -> Result<runtime_core::generated::framework::Framework, FrameworkLoadError> {
    let raw = tokio::fs::read_to_string(path).await?;
    load_and_validate_str_with_audit(&raw, emitter, audit_ctx).await
}

/// Validate a framework JSON string (no disk read).
///
/// Convenience wrapper around [`load_and_validate_str_with_audit`] that
/// skips audit emission.
///
/// # Errors
///
/// Same as [`load_and_validate`], minus the `Io` variant.
pub async fn load_and_validate_str(
    raw: &str,
    emitter: &impl Emitter,
) -> Result<runtime_core::generated::framework::Framework, FrameworkLoadError> {
    load_and_validate_str_with_audit(raw, emitter, &AuditContext::default()).await
}

/// Like [`load_and_validate_str`] but wires audit emission.
///
/// # Errors
///
/// Same as [`load_and_validate_str`].
pub async fn load_and_validate_str_with_audit(
    raw: &str,
    emitter: &impl Emitter,
    audit_ctx: &AuditContext,
) -> Result<runtime_core::generated::framework::Framework, FrameworkLoadError> {
    let framework: runtime_core::generated::framework::Framework = serde_json::from_str(raw)?;
    let gaps = walker::walk(&framework);
    if gaps.is_empty() {
        audit_log(
            audit_ctx,
            audit::framework_loaded(
                &audit_ctx.session_id,
                &framework.name,
                framework.agents.len(),
            ),
        )
        .await;
        return Ok(framework);
    }
    for gap in &gaps {
        audit_log(
            audit_ctx,
            audit::gap_detected(
                &audit_ctx.session_id,
                &gap.agent_id,
                gap_kind_str(gap.kind),
                &gap.missing_name,
                "loader",
            ),
        )
        .await;
        emitter.emit(gap.to_event(GapSourceRef::Loader)).await;
    }
    Err(FrameworkLoadError::GapsFound { count: gaps.len() })
}

/// Snake-case audit-log kind string for the gap. Mirrors the event
/// variant naming (`tool_missing`, `skill_missing`, `mcp_missing`,
/// `agent_missing`) so the audit log + the event stream + the
/// renderer all share the same discriminator vocabulary.
const fn gap_kind_str(kind: GapKind) -> &'static str {
    match kind {
        GapKind::Tool => "tool_missing",
        GapKind::Skill => "skill_missing",
        GapKind::Mcp => "mcp_missing",
        GapKind::Agent => "agent_missing",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Test emitter — collects emitted events for assertion.
    #[derive(Default)]
    struct CollectingEmitter {
        events: Mutex<Vec<AgentEvent>>,
    }

    #[async_trait::async_trait]
    impl Emitter for CollectingEmitter {
        async fn emit(&self, event: AgentEvent) {
            self.events.lock().expect("no poisoning").push(event);
        }
    }

    fn fw_with_two_unresolved_tools_json() -> String {
        serde_json::to_string(&serde_json::json!({
            "name": "test",
            "version": "1.0.0",
            "description": "test framework",
            "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
            "agents": [{
                "id": "worker",
                "role": "worker",
                "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
                "capabilities": {
                    "file_access": { "read": [], "write": [] },
                    "network": [], "shell": false,
                    "skills_loaded": [], "spawn_agents": [], "tools_called": []
                },
                "allowed_tools": ["MissingA", "MissingB"],
                "allowed_skills": [],
                "spawns": []
            }],
            "tools": [],
            "skills": [],
            "session_root_agent": "worker"
        }))
        .unwrap()
    }

    #[tokio::test]
    async fn valid_framework_returns_ok_with_zero_emissions() {
        let json = serde_json::to_string(&serde_json::json!({
            "name": "test",
            "version": "1.0.0",
            "description": "test framework",
            "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
            "agents": [{
                "id": "worker",
                "role": "worker",
                "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
                "capabilities": {
                    "file_access": { "read": [], "write": [] },
                    "network": [], "shell": false,
                    "skills_loaded": [], "spawn_agents": [], "tools_called": []
                },
                "allowed_tools": ["Read"],
                "allowed_skills": [],
                "spawns": []
            }],
            "tools": [{ "name": "Read", "source": "builtin" }],
            "skills": [],
            "session_root_agent": "worker"
        }))
        .unwrap();
        let emitter = CollectingEmitter::default();
        let result = load_and_validate_str(&json, &emitter).await;
        assert!(result.is_ok());
        assert!(emitter.events.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn unresolved_refs_emit_events_then_err() {
        let emitter = CollectingEmitter::default();
        let err = load_and_validate_str(&fw_with_two_unresolved_tools_json(), &emitter)
            .await
            .expect_err("two unresolved tools must surface");

        assert!(matches!(err, FrameworkLoadError::GapsFound { count: 2 }));
        let events = emitter.events.lock().unwrap();
        assert_eq!(events.len(), 2);
        // Both events carry requested_via = Loader (Layer 1).
        for evt in events.iter() {
            match evt {
                AgentEvent::ToolMissing { requested_via, .. } => {
                    assert_eq!(*requested_via, GapSourceRef::Loader);
                }
                _ => panic!("expected ToolMissing, got {evt:?}"),
            }
        }
    }

    #[tokio::test]
    async fn malformed_json_returns_json_err() {
        let emitter = CollectingEmitter::default();
        let err = load_and_validate_str("{not json", &emitter)
            .await
            .expect_err("malformed JSON must err");
        assert!(matches!(err, FrameworkLoadError::Json(_)));
    }

    #[tokio::test]
    async fn io_err_when_path_missing() {
        let emitter = CollectingEmitter::default();
        let err = load_and_validate(Path::new("/nonexistent/framework.json"), &emitter)
            .await
            .expect_err("missing file must err");
        assert!(matches!(err, FrameworkLoadError::Io(_)));
    }
}
