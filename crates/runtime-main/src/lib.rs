//! Runtime main process — owns the LLM provider, the agent SDK, and the
//! main-side drone IPC client.
//!
//! Top-level modules:
//! - [`providers`] — `LLMProvider` trait + `AnthropicProvider` impl (M02 Stage B/C).
//! - [`sdk`] — `AgentSdk<P>` agent loop + `EventPipeline` translator (M02 Stage D).
//! - [`drone_ipc`] — `DroneClient` main-side connection to the M01 drone (M02 Stage D).
//! - [`key_store`] — OS-keychain-backed Anthropic API key storage (M02 Stage E).

/// Audit log — spec §8.security L5 (M05 Stage E).
///
/// Append-only `skills.audit.jsonl` writer; one line per security
/// decision (framework load / gap / capability grant / capability
/// denial / tier transition). Best-effort observability — write
/// failures are logged via `tracing::error!` and never propagate
/// into dispatch.
pub mod audit;
/// Budget primitive — spec §2a (M04 Stage F).
///
/// 3-scope tightest-cap-wins enforcer + 4 threshold actions
/// (warn / downshift / hitl / hard-stop) + LRU `count_tokens` cache +
/// hardcoded opus → sonnet → haiku downshift ladder.
pub mod budget;
/// Capability enforcer — spec §8.security L1 + L2a (M05 Stage B).
///
/// In-process default-deny check fired before every tool dispatch +
/// sub-agent spawn. Owns per-agent capability grants; narrowing
/// evaluator enforces "child grants ⊆ parent grants" on Agent→Agent
/// edges. L3 sandbox is Stage C; L4 tier gates are Stage D; L5
/// provenance is Stage E.
pub mod capability;
pub mod drone_ipc;
/// Framework loader — spec §4b Layer 1 gap detection (M05 Stage A).
///
/// Parses framework JSON + walks declared primitives + emits gap events
/// for unresolved tool / skill / agent references via the in-process
/// [`framework_loader::Emitter`] trait. MCP gaps are Layer 2 only in
/// v0.1 (M06 adds Layer 1 MCP-server declaration).
pub mod framework_loader;
/// HITL primitive — spec §6a (M04 Stage E).
///
/// 9-trigger policy evaluator + `HitlSeam` (oneshot-channel gate) +
/// notifier plugin interface + 3 built-in notifiers (`terminal_bell`,
/// `desktop`, `sound`). Wires Stage B's failure-escalation flow to the
/// renderer's Panel / Modal / Toast surfaces.
pub mod hitl;
/// Verify & Rails primitive — spec §4a (M04 Stage D).
///
/// Hook executor + `JSONLogic`-evaluated rails + globset-backed
/// don't-touch matcher + cross-platform shell wrapper.
pub mod hooks;
pub mod key_store;
/// Plan + Task primitive — spec §3a (M04 Stage B).
pub mod plan;
pub mod providers;
/// Recovery primitive — spec §1b (M04 Stage F).
///
/// Resume coordinator + tool-call uncertainty handler.
///
/// Resume rebuilds SDK message history from the drone-projected
/// snapshot; tools are NOT re-invoked (gotcha #15). Uncertainty handler
/// records the user's 4-action choice as a
/// `tool_call_uncertainty_resolved` decision signal.
pub mod recovery;
/// Sandbox IPC client — spec §8.security L3 (M05 Stage C1).
///
/// Main-side framed-JSON client wrapping the `runtime-sandbox` subprocess.
/// Strict request-response (`validate(artifact, declaration) →
/// ValidationResult`). Borrow-not-move `next_response` from day 1 per
/// gotcha #72; multi-call invariant exercised by
/// `validate_succeeds_twice_in_sequence` per gotcha #69. Stage C2 adds
/// OS-level isolation inside the sandbox subprocess; this surface is
/// unchanged.
pub mod sandbox_ipc;
pub mod sdk;
/// Tier system — spec §8.security L4 (M05 Stage D).
///
/// Two-tier evaluator (Novice + Promoted per §0d) that sits BEFORE the
/// Stage B L1+L2a capability enforcer in the dispatch chain. Novice
/// caps the surface to a curated allowlist (Read + Domain-scoped
/// Network); Promoted is a pass-through at L4 (L1 still narrows).
/// Persisted in `<app_data_dir>/tier.json`; first-run defaults to
/// Novice.
pub mod tier;

/// Returns the string `"ok"`. Placeholder for Stage A; real exports come later.
///
/// # Examples
///
/// ```
/// assert_eq!(runtime_main::placeholder(), "ok");
/// ```
#[must_use]
pub const fn placeholder() -> &'static str {
    "ok"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_returns_ok() {
        assert_eq!(placeholder(), "ok");
    }
}
