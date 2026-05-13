//! Sandbox IPC wire format — framed JSON, request/response.
//!
//! Two request variants (`validate_artifact`, `shutdown`) + two response
//! variants (`validation_result`, `alert`). Both enums are
//! `#[serde(tag = "type", rename_all = "snake_case")]` so the wire form
//! is `{"type":"validate_artifact","artifact_code":"…", …}` — same
//! shape as `runtime_core::DroneCommand` / `DroneEvent`.
//!
//! The shared [`CapabilityDeclaration`] field is schema-derived
//! (`schemas/capability.v1.json`) and travels by-value across the wire.
//! Adding a new variant or field requires bumping consumers in lockstep
//! — IPC is internal, not a stable public API.

use runtime_core::generated::capability::CapabilityDeclaration;
use serde::{Deserialize, Serialize};

use crate::validator::ValidationResult;

/// Requests sent from main to the sandbox subprocess.
///
/// `PartialEq` / `Eq` are intentionally NOT derived — the `declaration`
/// field is a `CapabilityDeclaration` (typify-generated) whose `oneOf`
/// scope variants wrap non-`Copy` newtype payloads (`GlobPattern` /
/// `DomainPattern` / `PathPattern`) and so do not derive equality. Per
/// the M05.B retrospective surprise event (graduated to a working
/// pattern this stage), compare via serde round-trip in tests instead.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SandboxRequest {
    /// Validate an artifact against a capability declaration. Reply is
    /// [`SandboxResponse::ValidationResult`].
    ValidateArtifact {
        /// The artifact's source code (free-form text per language).
        artifact_code: String,
        /// The capability declaration the artifact must be bounded by.
        declaration: CapabilityDeclaration,
    },
    /// Tell the subprocess to exit. No response; subprocess exits cleanly.
    Shutdown,
}

/// Severity of a [`SandboxResponse::Alert`] line. Mirrors
/// `runtime_core::AlertLevel` but stays local to keep the sandbox crate
/// dep-light.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlertLevel {
    /// Informational; client may log and continue.
    Info,
    /// Warning; client should log.
    Warn,
    /// Fatal; client should treat as a transport-level codec error.
    Critical,
}

/// Responses sent from the sandbox subprocess back to main.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SandboxResponse {
    /// Result of a [`SandboxRequest::ValidateArtifact`].
    ValidationResult(ValidationResult),
    /// Out-of-band alert (malformed request line, internal error). The
    /// client surfaces this as a `Codec` error per drone-ipc convention.
    Alert {
        /// Severity.
        level: AlertLevel,
        /// Human-readable message.
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::generated::capability::{
        CapabilityKind, CapabilityScope, GlobPattern, ResourceName, SideEffectClass,
    };
    use std::str::FromStr;

    fn sample_declaration() -> CapabilityDeclaration {
        CapabilityDeclaration {
            kind: CapabilityKind::Read,
            resource: ResourceName::from_str("*.md").expect("resource"),
            scope: CapabilityScope::Glob(GlobPattern::from_str("*.md").expect("glob")),
            side_effect_class: SideEffectClass::Pure,
        }
    }

    #[test]
    fn validate_artifact_request_round_trip() {
        let req = SandboxRequest::ValidateArtifact {
            artifact_code: "let x = 1;".to_string(),
            declaration: sample_declaration(),
        };
        let line = serde_json::to_string(&req).expect("ser");
        // Wire shape sanity — externally-tagged variant under snake_case.
        assert!(
            line.contains("\"type\":\"validate_artifact\""),
            "got {line}"
        );
        // SandboxRequest doesn't derive Eq (CapabilityDeclaration's
        // oneOf scope variants wrap non-Copy newtypes per M05.B retro
        // surprise event). Compare via serde re-serialization equality.
        let back: SandboxRequest = serde_json::from_str(&line).expect("de");
        let back_line = serde_json::to_string(&back).expect("re-ser");
        assert_eq!(line, back_line);
    }

    #[test]
    fn shutdown_request_round_trip() {
        let req = SandboxRequest::Shutdown;
        let line = serde_json::to_string(&req).expect("ser");
        assert!(line.contains("\"type\":\"shutdown\""), "got {line}");
        let back: SandboxRequest = serde_json::from_str(&line).expect("de");
        let back_line = serde_json::to_string(&back).expect("re-ser");
        assert_eq!(line, back_line);
    }

    #[test]
    fn validation_result_response_round_trip() {
        let resp = SandboxResponse::ValidationResult(ValidationResult::Ok);
        let line = serde_json::to_string(&resp).expect("ser");
        assert!(
            line.contains("\"type\":\"validation_result\""),
            "got {line}"
        );
        let back: SandboxResponse = serde_json::from_str(&line).expect("de");
        assert_eq!(resp, back);
    }

    #[test]
    fn alert_response_round_trip() {
        let resp = SandboxResponse::Alert {
            level: AlertLevel::Warn,
            message: "malformed request".to_string(),
        };
        let line = serde_json::to_string(&resp).expect("ser");
        assert!(line.contains("\"type\":\"alert\""));
        let back: SandboxResponse = serde_json::from_str(&line).expect("de");
        assert_eq!(resp, back);
    }
}
