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

/// Failure types raised by [`load_and_validate`].
pub mod error;
/// Pure-function walker + Gap/GapKind types. Spec §4b Layer 1.
pub mod walker;

pub use error::FrameworkLoadError;
pub use walker::{walk, Gap, GapKind};

use runtime_core::event::AgentEvent;
use runtime_core::event::GapSourceRef;
use std::path::Path;

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

/// Load a framework JSON from `path` + walk it for Layer-1 gaps.
///
/// Returns `Ok(Framework)` only when every declared reference resolves.
/// Otherwise emits one gap event (`requested_via: loader`) per unresolved
/// reference via `emitter` and returns `Err(FrameworkLoadError::GapsFound)`.
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
    let raw = tokio::fs::read_to_string(path).await?;
    load_and_validate_str(&raw, emitter).await
}

/// Validate a framework JSON string (no disk read).
///
/// Variant of [`load_and_validate`] for tests and callers that already
/// have the bytes (e.g., framework-import flows in M07).
///
/// # Errors
///
/// Same as [`load_and_validate`], minus the `Io` variant (no disk read).
pub async fn load_and_validate_str(
    raw: &str,
    emitter: &impl Emitter,
) -> Result<runtime_core::generated::framework::Framework, FrameworkLoadError> {
    let framework: runtime_core::generated::framework::Framework = serde_json::from_str(raw)?;
    let gaps = walker::walk(&framework);
    if gaps.is_empty() {
        return Ok(framework);
    }
    for gap in &gaps {
        emitter.emit(gap.to_event(GapSourceRef::Loader)).await;
    }
    Err(FrameworkLoadError::GapsFound { count: gaps.len() })
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
