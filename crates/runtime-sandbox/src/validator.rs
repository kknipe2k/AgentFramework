//! L3 sandbox validator — pure-function check that an artifact's
//! observable behavior is bounded by a [`CapabilityDeclaration`].
//!
//! Stage C1 ships a token-scanning implementation: the artifact's source
//! text is scanned for a fixed set of syscall-name tokens (e.g. `write`,
//! `connect`, `spawn`), each token maps to a [`CapabilityKind`], and any
//! detected syscall whose kind exceeds the declaration's kind is reported
//! in the [`ValidationResult::Reject`] reasons.
//!
//! The validator is cross-platform and pure — no IO, no allocation
//! beyond the rejection-reasons vector, no platform branches. Stage C2
//! layers seccomp / landlock / Job Objects on top: the OS-level fence
//! installs BEFORE this function runs, so a maliciously-crafted artifact
//! that tries to escape the token scan is still bounded by the kernel.
//!
//! M09 (generators) is the first production caller; v0.1 has no caller
//! and the boundary stays callable-but-unwired (per the M05 phase doc
//! `<execution_warnings>`).

use runtime_core::generated::capability::{CapabilityDeclaration, CapabilityKind};
use serde::{Deserialize, Serialize};

/// An artifact submitted for sandbox validation. Stage C1 covers only
/// the `code` field — a flat source string. Stage C2 + M09 may extend
/// with language hint, AST cache, etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Artifact {
    /// Source text of the artifact (tool implementation or skill recipe).
    pub code: String,
}

impl Artifact {
    /// Construct an `Artifact` from a source string.
    #[must_use]
    pub fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }

    /// Scan the artifact's `code` for syscall tokens and return every
    /// match paired with its [`CapabilityKind`]. Token list is a coarse
    /// allow-list keyed off the five v0.1 kinds.
    #[must_use]
    pub fn scan_syscalls(&self) -> Vec<DetectedSyscall> {
        // Token table — `(token, kind)` pairs. Substring match against
        // the artifact's code. The list is deliberately small and
        // conservative for C1; M09 will extend per language.
        const TABLE: &[(&str, CapabilityKind)] = &[
            // Filesystem writes — `kind: write`.
            ("write_file", CapabilityKind::Write),
            ("create_file", CapabilityKind::Write),
            ("remove_file", CapabilityKind::Write),
            ("truncate", CapabilityKind::Write),
            // Filesystem reads — `kind: read`.
            ("read_file", CapabilityKind::Read),
            ("open_file", CapabilityKind::Read),
            // Network egress — `kind: network`.
            ("http_get", CapabilityKind::Network),
            ("http_post", CapabilityKind::Network),
            ("tcp_connect", CapabilityKind::Network),
            ("socket_connect", CapabilityKind::Network),
            // Child-process spawn — `kind: process_spawn`.
            ("spawn_process", CapabilityKind::ProcessSpawn),
            ("execve", CapabilityKind::ProcessSpawn),
            // Generic exec / tool invocation — `kind: exec`.
            ("invoke_tool", CapabilityKind::Exec),
        ];

        TABLE
            .iter()
            .filter(|(token, _)| self.code.contains(token))
            .map(|&(name, kind)| DetectedSyscall { name, kind })
            .collect()
    }
}

/// A syscall detected by [`Artifact::scan_syscalls`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct DetectedSyscall {
    /// Token that matched in the artifact's code.
    pub name: &'static str,
    /// Capability kind the token implies.
    pub kind: CapabilityKind,
}

/// Result of a `validate` call. `Ok` means the artifact's detected
/// syscalls are bounded by the declaration; `Reject` carries one reason
/// string per disallowed syscall.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum ValidationResult {
    /// Artifact's syscalls all match the declaration's kind.
    Ok,
    /// One or more detected syscalls exceeded the declaration. Reasons
    /// are human-readable; order matches the scan order.
    Reject {
        /// Per-syscall reason strings.
        reasons: Vec<String>,
    },
}

impl ValidationResult {
    /// Constructor for the cleared case.
    #[must_use]
    pub const fn ok() -> Self {
        Self::Ok
    }

    /// Constructor for the rejection case.
    #[must_use]
    pub const fn reject(reasons: Vec<String>) -> Self {
        Self::Reject { reasons }
    }

    /// Whether this result is `Ok`.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }
}

/// Pure-function validator. Returns [`ValidationResult::Ok`] iff every
/// detected syscall's `kind` matches the declaration's `kind`.
///
/// Stage C1 enforces `kind`-only matching; `scope` containment is the
/// L2a enforcer's job (see `runtime_main::capability::declaration`).
/// The L3 sandbox sits below L2a and only needs to verify the syscall
/// surface is bounded by the kind already cleared at L2a — anything
/// else would duplicate the enforcer's per-variant scope logic in two
/// places.
#[must_use]
pub fn validate(artifact: &Artifact, declaration: &CapabilityDeclaration) -> ValidationResult {
    let detected = artifact.scan_syscalls();
    let exceeded: Vec<_> = detected
        .into_iter()
        .filter(|sc| sc.kind != declaration.kind)
        .collect();
    if exceeded.is_empty() {
        ValidationResult::ok()
    } else {
        let reasons = exceeded
            .into_iter()
            .map(|sc| format!("disallowed syscall: {} ({})", sc.name, sc.kind))
            .collect();
        ValidationResult::reject(reasons)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::generated::capability::{
        CapabilityKind, CapabilityScope, GlobPattern, ResourceName, SideEffectClass,
    };
    use std::str::FromStr;

    fn pure_read_declaration() -> CapabilityDeclaration {
        CapabilityDeclaration {
            kind: CapabilityKind::Read,
            resource: ResourceName::from_str("*.md").expect("resource"),
            scope: CapabilityScope::Glob(GlobPattern::from_str("*.md").expect("glob")),
            side_effect_class: SideEffectClass::Pure,
        }
    }

    fn network_declaration() -> CapabilityDeclaration {
        use runtime_core::generated::capability::DomainPattern;
        CapabilityDeclaration {
            kind: CapabilityKind::Network,
            resource: ResourceName::from_str("api.example.com").expect("resource"),
            scope: CapabilityScope::Domain(
                DomainPattern::from_str("api.example.com").expect("domain"),
            ),
            side_effect_class: SideEffectClass::NetworkEgress,
        }
    }

    #[test]
    fn pure_artifact_passes() {
        // An empty / non-syscall artifact passes any declaration — the
        // scan returns nothing, so there's nothing to exceed.
        let artifact = Artifact::new("let x = 1 + 2;");
        let result = validate(&artifact, &pure_read_declaration());
        assert_eq!(result, ValidationResult::Ok);
        assert!(result.is_ok());
    }

    #[test]
    fn filesystem_syscall_exceeds_pure_declaration_rejects() {
        // Code containing a `write_file` token against a `kind: read`
        // declaration must reject. The reason string surfaces the
        // detected syscall name and kind so the caller can route to
        // the on_capability_violation HITL trigger meaningfully.
        let artifact = Artifact::new("write_file(path, data)");
        let result = validate(&artifact, &pure_read_declaration());
        match result {
            ValidationResult::Reject { reasons } => {
                assert_eq!(reasons.len(), 1, "exactly one detected syscall");
                assert!(
                    reasons[0].contains("write_file"),
                    "reason must name the syscall: got {}",
                    reasons[0]
                );
                assert!(
                    reasons[0].contains("write"),
                    "reason must name the kind: got {}",
                    reasons[0]
                );
            }
            ValidationResult::Ok => panic!("expected reject, got Ok"),
        }
    }

    #[test]
    fn network_syscall_in_network_declaration_passes() {
        // Code containing a network syscall token against a network-kind
        // declaration is the cleared case — kinds match, scope check
        // is L2a's job (already cleared by the time we get here).
        let artifact = Artifact::new("http_get(\"https://api.example.com\")");
        let result = validate(&artifact, &network_declaration());
        assert!(result.is_ok(), "got {result:?}");
    }

    #[test]
    fn multiple_exceedances_each_reported() {
        // A single artifact mixing two disallowed kinds produces one
        // reason per detected syscall — caller can present all of them.
        let artifact = Artifact::new("write_file(p); http_post(u)");
        let result = validate(&artifact, &pure_read_declaration());
        match result {
            ValidationResult::Reject { reasons } => {
                assert_eq!(reasons.len(), 2);
                assert!(reasons.iter().any(|r| r.contains("write_file")));
                assert!(reasons.iter().any(|r| r.contains("http_post")));
            }
            ValidationResult::Ok => panic!("expected reject"),
        }
    }

    #[test]
    fn scan_returns_empty_for_pure_code() {
        let artifact = Artifact::new("fn main() { let _ = 42; }");
        assert!(artifact.scan_syscalls().is_empty());
    }

    #[test]
    fn validation_result_serde_round_trip_ok() {
        let v = ValidationResult::Ok;
        let s = serde_json::to_string(&v).expect("ser");
        let back: ValidationResult = serde_json::from_str(&s).expect("de");
        assert_eq!(v, back);
    }

    #[test]
    fn validation_result_serde_round_trip_reject() {
        let v = ValidationResult::reject(vec!["one".to_string(), "two".to_string()]);
        let s = serde_json::to_string(&v).expect("ser");
        let back: ValidationResult = serde_json::from_str(&s).expect("de");
        assert_eq!(v, back);
    }
}
