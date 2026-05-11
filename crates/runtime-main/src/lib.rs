//! Runtime main process — owns the LLM provider, the agent SDK, and the
//! main-side drone IPC client.
//!
//! Top-level modules:
//! - [`providers`] — `LLMProvider` trait + `AnthropicProvider` impl (M02 Stage B/C).
//! - [`sdk`] — `AgentSdk<P>` agent loop + `EventPipeline` translator (M02 Stage D).
//! - [`drone_ipc`] — `DroneClient` main-side connection to the M01 drone (M02 Stage D).
//! - [`key_store`] — OS-keychain-backed Anthropic API key storage (M02 Stage E).

/// Budget primitive — spec §2a (M04 Stage F).
///
/// 3-scope tightest-cap-wins enforcer + 4 threshold actions
/// (warn / downshift / hitl / hard-stop) + LRU `count_tokens` cache +
/// hardcoded opus → sonnet → haiku downshift ladder.
pub mod budget;
pub mod drone_ipc;
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
pub mod sdk;

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
