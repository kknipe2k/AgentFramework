//! Signal Schema v2 — forensic event log types (spec §2b).
//!
//! Signals are write-heavy operational forensics. The VDR projection layer
//! consumes them and produces decision-focused rows (`vdr` table). This
//! module defines the type surface; emission integration lands in M04+
//! (verify, HITL, plan).

use serde::{Deserialize, Serialize};

/// Reference to a prior signal in a causal chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreSignalId(pub String);

/// Reference to a parent signal (e.g., the agent that triggered this).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParentSignalId(pub String);

/// Reference to a signal this is a retry of.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryOfSignalId(pub String);

/// What kind of context this signal was produced in. Spec §2b artifact-source
/// taxonomy — reconciled with spec at M05.A (M02/M03/M04 carry-forward).
///
/// Variants name the artifact source of the signal (what the signal was
/// forensically about) rather than the runtime call context (where in the
/// SDK the signal was emitted). The latter is recoverable from
/// `signal.kind`, `parent_signal_id`, and `payload_json`; the former is
/// the cross-cutting classifier that drives VDR projection and dashboard
/// filtering.
///
/// M05.A reconciliation: spec §2b L1134 + L1178 lists the canonical 7-value
/// set `{skill, framework, code, search, verify, commit, subagent}`. The M02
/// scaffold defined operational-context variants (`AgentLoop / SkillLoad /
/// ToolInvoke / HookExecute / PlanCreate / HitlPrompt / SessionLifecycle`)
/// with no consumers; M02 + M03 + M04 retrospectives deferred the
/// reconciliation pending signal-emission integration evidence (none
/// materialized — `Signal` writers at `WriteSignal` + decision-record +
/// uncertainty paths take `context_type` as a parameter from the caller).
/// M05.A adopts the spec set per CLAUDE.md §10 (spec wins absent maintainer
/// override) + gotcha trap "`ContextType` reconcile: spec §2b is source of
/// truth".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextType {
    /// Signal about a skill operation (load / discovery / execute).
    Skill,
    /// Signal about framework-level operation (session lifecycle, plan,
    /// HITL, mode change).
    Framework,
    /// Signal about code modification (Write / Edit tools, file ops).
    Code,
    /// Signal about search / discovery (Glob / Grep / `WebFetch` tools).
    Search,
    /// Signal about a verification operation (hook execute, rail check).
    Verify,
    /// Signal about a git commit operation.
    Commit,
    /// Signal about a sub-agent operation (spawn / complete / error).
    Subagent,
}

/// Forensic event — 8 kinds per spec §2b.
//
// `Eq` is intentionally omitted: variants embed `serde_json::Value`, which
// contains `f64` numeric values and so cannot impl `Eq`. Equality on
// signals is `PartialEq`-only by design.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Signal {
    /// A tool was invoked.
    Tool {
        /// Unique signal identifier.
        signal_id: String,
        /// Agent that emitted this signal.
        agent_id: String,
        /// Name of the tool invoked.
        tool_name: String,
        /// Tool-specific payload.
        payload_json: serde_json::Value,
        /// Optional pre-signal correlation.
        pre_signal_id: Option<PreSignalId>,
        /// Optional parent signal correlation.
        parent_signal_id: Option<ParentSignalId>,
        /// Optional retry correlation.
        retry_of: Option<RetryOfSignalId>,
        /// Context in which this signal was emitted.
        context_type: ContextType,
    },
    /// A skill was loaded or executed.
    Skill {
        /// Unique signal identifier.
        signal_id: String,
        /// Agent that emitted this signal.
        agent_id: String,
        /// Name of the skill.
        skill_name: String,
        /// Version of the skill.
        skill_version: String,
        /// Skill-specific payload.
        payload_json: serde_json::Value,
        /// Optional parent signal correlation.
        parent_signal_id: Option<ParentSignalId>,
        /// Context in which this signal was emitted.
        context_type: ContextType,
    },
    /// An agent lifecycle event (spawned / complete / error).
    Agent {
        /// Unique signal identifier.
        signal_id: String,
        /// Agent that emitted this signal.
        agent_id: String,
        /// Lifecycle event name.
        event: String,
        /// Event-specific payload.
        payload_json: serde_json::Value,
        /// Optional parent signal correlation.
        parent_signal_id: Option<ParentSignalId>,
        /// Context in which this signal was emitted.
        context_type: ContextType,
    },
    /// An agent decision with rationale.
    Decision {
        /// Unique signal identifier.
        signal_id: String,
        /// Agent that emitted this signal.
        agent_id: String,
        /// Decision summary.
        decision: String,
        /// Rationale for the decision.
        rationale: String,
        /// Optional tool used to inform the decision.
        tool_used: Option<String>,
        /// Decision-specific payload.
        payload_json: serde_json::Value,
        /// Optional parent signal correlation.
        parent_signal_id: Option<ParentSignalId>,
        /// Context in which this signal was emitted.
        context_type: ContextType,
    },
    /// A verification hook fired.
    Verify {
        /// Unique signal identifier.
        signal_id: String,
        /// Agent that emitted this signal.
        agent_id: String,
        /// Identifier of the verification hook.
        hook_id: String,
        /// Whether the hook passed.
        passed: bool,
        /// Hook-specific payload.
        payload_json: serde_json::Value,
        /// Optional parent signal correlation.
        parent_signal_id: Option<ParentSignalId>,
        /// Context in which this signal was emitted.
        context_type: ContextType,
    },
    /// An error occurred.
    Error {
        /// Unique signal identifier.
        signal_id: String,
        /// Agent that emitted this signal, if any.
        agent_id: Option<String>,
        /// Kind of error (e.g. `timeout`, `tool_failure`).
        error_kind: String,
        /// Human-readable error message.
        message: String,
        /// Error-specific payload.
        payload_json: serde_json::Value,
        /// Optional parent signal correlation.
        parent_signal_id: Option<ParentSignalId>,
        /// Optional retry correlation.
        retry_of: Option<RetryOfSignalId>,
        /// Context in which this signal was emitted.
        context_type: ContextType,
    },
    /// A human-in-the-loop prompt cycle.
    Hitl {
        /// Unique signal identifier.
        signal_id: String,
        /// Agent that emitted this signal.
        agent_id: String,
        /// Prompt shown to the user.
        prompt: String,
        /// User's response, if any.
        response: Option<String>,
        /// HITL-specific payload.
        payload_json: serde_json::Value,
        /// Optional parent signal correlation.
        parent_signal_id: Option<ParentSignalId>,
        /// Context in which this signal was emitted.
        context_type: ContextType,
    },
    /// Session lifecycle event (start / suspend / resume / end).
    Session {
        /// Unique signal identifier.
        signal_id: String,
        /// Lifecycle event name.
        event: String,
        /// Event-specific payload.
        payload_json: serde_json::Value,
        /// Context in which this signal was emitted.
        context_type: ContextType,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_round_trip(s: &Signal) {
        let json = serde_json::to_string(s).unwrap();
        let back: Signal = serde_json::from_str(&json).unwrap();
        assert_eq!(*s, back);
    }

    fn payload() -> serde_json::Value {
        serde_json::json!({"k": "v", "n": 42})
    }

    #[test]
    fn round_trip_tool() {
        check_round_trip(&Signal::Tool {
            signal_id: "sig-1".into(),
            agent_id: "agent-1".into(),
            tool_name: "search".into(),
            payload_json: payload(),
            pre_signal_id: Some(PreSignalId("sig-prev".into())),
            parent_signal_id: Some(ParentSignalId("sig-parent".into())),
            retry_of: None,
            context_type: ContextType::Code,
        });
    }

    #[test]
    fn round_trip_skill() {
        check_round_trip(&Signal::Skill {
            signal_id: "sig-2".into(),
            agent_id: "agent-1".into(),
            skill_name: "skim-skill".into(),
            skill_version: "1.0.0".into(),
            payload_json: payload(),
            parent_signal_id: None,
            context_type: ContextType::Skill,
        });
    }

    #[test]
    fn round_trip_agent() {
        check_round_trip(&Signal::Agent {
            signal_id: "sig-3".into(),
            agent_id: "agent-1".into(),
            event: "spawned".into(),
            payload_json: payload(),
            parent_signal_id: None,
            context_type: ContextType::Subagent,
        });
    }

    #[test]
    fn round_trip_decision() {
        check_round_trip(&Signal::Decision {
            signal_id: "sig-4".into(),
            agent_id: "agent-1".into(),
            decision: "pick haiku".into(),
            rationale: "cost-sensitive".into(),
            tool_used: Some("estimate_cost".into()),
            payload_json: payload(),
            parent_signal_id: None,
            context_type: ContextType::Subagent,
        });
    }

    #[test]
    fn round_trip_verify() {
        check_round_trip(&Signal::Verify {
            signal_id: "sig-5".into(),
            agent_id: "agent-1".into(),
            hook_id: "test-suite".into(),
            passed: true,
            payload_json: payload(),
            parent_signal_id: None,
            context_type: ContextType::Verify,
        });
    }

    #[test]
    fn round_trip_error() {
        check_round_trip(&Signal::Error {
            signal_id: "sig-6".into(),
            agent_id: Some("agent-1".into()),
            error_kind: "timeout".into(),
            message: "tool exceeded 60s".into(),
            payload_json: payload(),
            parent_signal_id: None,
            retry_of: Some(RetryOfSignalId("sig-orig".into())),
            context_type: ContextType::Code,
        });
    }

    #[test]
    fn round_trip_hitl() {
        check_round_trip(&Signal::Hitl {
            signal_id: "sig-7".into(),
            agent_id: "agent-1".into(),
            prompt: "approve plan?".into(),
            response: Some("yes".into()),
            payload_json: payload(),
            parent_signal_id: None,
            context_type: ContextType::Framework,
        });
    }

    #[test]
    fn round_trip_session() {
        check_round_trip(&Signal::Session {
            signal_id: "sig-8".into(),
            event: "start".into(),
            payload_json: payload(),
            context_type: ContextType::Framework,
        });
    }

    #[test]
    fn tag_serialization_is_snake_case() {
        let s = Signal::Tool {
            signal_id: "x".into(),
            agent_id: "x".into(),
            tool_name: "x".into(),
            payload_json: payload(),
            pre_signal_id: None,
            parent_signal_id: None,
            retry_of: None,
            context_type: ContextType::Code,
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"kind\":\"tool\""), "got: {json}");
    }

    #[test]
    fn signal_round_trip_preserves_all_fields() {
        let s = Signal::Decision {
            signal_id: "sig-roundtrip".into(),
            agent_id: "agent-x".into(),
            decision: "skip retry".into(),
            rationale: "previous attempt timed out".into(),
            tool_used: Some("estimate_cost".into()),
            payload_json: serde_json::json!({"attempts": 3}),
            parent_signal_id: Some(ParentSignalId("sig-parent".into())),
            context_type: ContextType::Subagent,
        };
        let json = serde_json::to_string(&s).expect("encode");
        let back: Signal = serde_json::from_str(&json).expect("decode");
        assert_eq!(s, back, "decoded signal must equal original");
    }
}
