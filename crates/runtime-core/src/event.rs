//! `AgentEvent` — canonical event union emitted by the runtime.
//!
//! Variants span the spec sections they belong to; cross-references in
//! variant doc comments link back to the originating spec section.
//!
//! Variants that aren't yet emitted at v0.1 are still defined here so later
//! milestones extend the union additively (semver-minor, no breaking change).

use serde::{Deserialize, Serialize};

/// The canonical event union emitted by the runtime across all phases.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    // ── Session lifecycle (spec §2) ──
    /// A new session has started.
    SessionStart {
        /// Unique session identifier.
        session_id: String,
        /// Framework name/path.
        framework: String,
        /// LLM model identifier.
        model: String,
    },
    /// A session has ended.
    SessionEnd {
        /// The session that ended.
        session_id: String,
        /// Total session duration in milliseconds.
        duration_ms: u64,
        /// Why the session ended.
        end_reason: String,
    },

    // ── Agent lifecycle (spec §2) ──
    /// An agent was spawned.
    AgentSpawned {
        /// Unique agent identifier.
        agent_id: String,
        /// Agent name from the framework.
        agent_name: String,
        /// Parent agent id, if this is a child agent.
        parent_id: Option<String>,
    },
    /// An agent completed successfully.
    AgentComplete {
        /// The agent that completed.
        agent_id: String,
        /// Result summary.
        result: String,
    },
    /// An agent encountered an error.
    AgentError {
        /// The agent that errored.
        agent_id: String,
        /// Error description.
        error: String,
    },

    // ── Tool / Skill (spec §0b + §2) ──
    /// A tool was invoked by an agent.
    ToolInvoked {
        /// The invoking agent.
        agent_id: String,
        /// Tool name.
        tool_name: String,
        /// Tool input payload.
        input: serde_json::Value,
    },
    /// A tool returned a result.
    ToolResult {
        /// The invoking agent.
        agent_id: String,
        /// Tool name.
        tool_name: String,
        /// Tool output payload.
        output: serde_json::Value,
        /// Execution time in milliseconds.
        duration_ms: u64,
    },
    /// A tool call errored.
    ToolError {
        /// The invoking agent.
        agent_id: String,
        /// Tool name.
        tool_name: String,
        /// Error description.
        error: String,
    },
    /// A skill was loaded into an agent's context.
    SkillLoaded {
        /// The agent that loaded the skill.
        agent_id: String,
        /// Skill name.
        skill_name: String,
        /// Mode variant used for filtering, if any.
        mode: Option<String>,
    },

    // ── Plan / Task lifecycle (spec §3a) ──
    /// A plan was created.
    PlanCreated {
        /// Unique plan identifier.
        plan_id: String,
        /// Number of tasks in the plan.
        task_count: u32,
    },
    /// A plan was approved.
    PlanApproved {
        /// The approved plan.
        plan_id: String,
    },
    /// A plan was rejected.
    PlanRejected {
        /// The rejected plan.
        plan_id: String,
        /// Rejection reason.
        reason: String,
    },
    /// A task within a plan started executing.
    TaskStarted {
        /// The parent plan.
        plan_id: String,
        /// The task that started.
        task_id: String,
        /// The agent executing the task.
        agent_id: String,
    },
    /// A task within a plan completed.
    TaskCompleted {
        /// The parent plan.
        plan_id: String,
        /// The completed task.
        task_id: String,
        /// Task execution time in milliseconds.
        duration_ms: u64,
    },
    /// A task within a plan failed.
    TaskFailed {
        /// The parent plan.
        plan_id: String,
        /// The failed task.
        task_id: String,
        /// Error description.
        error: String,
        /// How many times this task has failed.
        failure_count: u32,
    },
    /// A task was rolled back to a prior snapshot.
    TaskRolledBack {
        /// The parent plan.
        plan_id: String,
        /// The rolled-back task.
        task_id: String,
        /// Snapshot used for rollback.
        snapshot_id: String,
    },
    /// A task was escalated (e.g., to a human).
    TaskEscalated {
        /// The parent plan.
        plan_id: String,
        /// The escalated task.
        task_id: String,
        /// Escalation reason.
        reason: String,
    },

    // ── Mode (spec §3b) ──
    /// The active mode changed.
    ModeChanged {
        /// Previous mode.
        from: String,
        /// New mode.
        to: String,
        /// Reason for the change.
        reason: String,
    },

    // ── Verify + Rails (spec §4a) ──
    /// A verification hook started.
    VerifyStarted {
        /// Hook identifier.
        hook_id: String,
        /// Verification level.
        level: String,
    },
    /// A verification hook passed.
    VerifyPassed {
        /// Hook identifier.
        hook_id: String,
        /// Verification duration in milliseconds.
        duration_ms: u64,
    },
    /// A verification hook failed.
    VerifyFailed {
        /// Hook identifier.
        hook_id: String,
        /// Error description.
        error: String,
    },
    /// A rail was triggered.
    RailTriggered {
        /// Rail identifier.
        rail_id: String,
        /// Severity level.
        severity: String,
        /// Human-readable message.
        message: String,
    },

    // ── Gap detection (spec §4b) ──
    /// A required skill is missing.
    SkillMissing {
        /// The agent that needs the skill.
        agent_id: String,
        /// Name of the missing skill.
        skill_name: String,
        /// Severity of the gap.
        severity: String,
    },
    /// A required tool is missing.
    ToolMissing {
        /// The agent that needs the tool.
        agent_id: String,
        /// Name of the missing tool.
        tool_name: String,
        /// Severity of the gap.
        severity: String,
    },
    /// A previously detected gap was resolved.
    GapResolved {
        /// The agent whose gap was resolved.
        agent_id: String,
        /// The capability that was provided.
        capability: String,
        /// Kind of capability (tool or skill).
        kind: String,
    },

    // ── HITL (spec §6a) ──
    /// A human-in-the-loop interaction was requested.
    HitlRequested {
        /// The requesting agent.
        agent_id: String,
        /// The prompt shown to the human.
        prompt: String,
        /// Kind of HITL interaction.
        hitl_kind: String,
    },
    /// A human-in-the-loop interaction was resolved.
    HitlResolved {
        /// The requesting agent.
        agent_id: String,
        /// The human's response.
        response: String,
        /// Time the human took to respond in milliseconds.
        duration_ms: u64,
    },

    // ── Capability enforcement (spec §8.security) ──
    /// An agent attempted an action outside its declared capabilities.
    CapabilityViolation {
        /// The violating agent.
        agent_id: String,
        /// What was declared.
        declared: String,
        /// What was attempted.
        attempted: String,
    },
    /// A capability was granted to an agent.
    CapabilityGrant {
        /// The agent receiving the grant.
        agent_id: String,
        /// The granted capability.
        capability: String,
        /// Scope of the grant.
        scope: String,
    },

    // ── Budget (spec §2a) ──
    /// Budget warning threshold reached.
    BudgetWarn {
        /// Amount spent so far in USD.
        spent_usd: f64,
        /// Budget cap in USD.
        cap_usd: f64,
        /// Percentage of budget used.
        percent: u32,
    },
    /// Budget triggered a model downshift.
    BudgetDownshift {
        /// Model being downshifted from.
        from_model: String,
        /// Model being downshifted to.
        to_model: String,
        /// Reason for the downshift.
        reason: String,
    },
    /// Session suspended due to budget.
    BudgetSuspended {
        /// Amount spent so far in USD.
        spent_usd: f64,
        /// Budget cap in USD.
        cap_usd: f64,
    },
    /// Budget exceeded.
    BudgetExceeded {
        /// Amount spent so far in USD.
        spent_usd: f64,
        /// Budget cap in USD.
        cap_usd: f64,
    },

    // ── Stream + decision trace (spec §2 + §2b) ──
    /// Streaming text from an agent.
    StreamText {
        /// The streaming agent.
        agent_id: String,
        /// The text chunk.
        text: String,
    },
    /// A decision was recorded in the VDR.
    DecisionRecord {
        /// The agent that made the decision.
        agent_id: String,
        /// What was decided.
        decision: String,
        /// Why it was decided.
        rationale: String,
        /// Tool used, if any.
        tool_used: Option<String>,
    },
    /// Token usage report.
    TokenUsage {
        /// Input tokens consumed.
        input: u64,
        /// Output tokens produced.
        output: u64,
        /// Model used.
        model: String,
        /// Estimated cost in USD.
        cost_usd: f64,
    },
}

#[cfg(test)]
mod proptest_round_trip {
    use super::*;
    use proptest::prelude::*;

    fn arb_session_start() -> impl Strategy<Value = AgentEvent> {
        (any::<String>(), any::<String>(), any::<String>()).prop_map(
            |(session_id, framework, model)| AgentEvent::SessionStart {
                session_id,
                framework,
                model,
            },
        )
    }

    fn arb_tool_invoked() -> impl Strategy<Value = AgentEvent> {
        (any::<String>(), any::<String>()).prop_map(|(agent_id, tool_name)| {
            AgentEvent::ToolInvoked {
                agent_id,
                tool_name,
                input: serde_json::json!({"key": "value"}),
            }
        })
    }

    proptest! {
        #[test]
        fn session_start_round_trips(event in arb_session_start()) {
            let json: serde_json::Value = serde_json::to_value(&event).unwrap();
            let back: AgentEvent = serde_json::from_value(json).unwrap();
            prop_assert_eq!(event, back);
        }

        #[test]
        fn tool_invoked_round_trips(event in arb_tool_invoked()) {
            let json: serde_json::Value = serde_json::to_value(&event).unwrap();
            let back: AgentEvent = serde_json::from_value(json).unwrap();
            prop_assert_eq!(event, back);
        }
    }
}
