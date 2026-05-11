//! Runtime main process — owns the LLM provider, the agent SDK, and the
//! main-side drone IPC client.
//!
//! Top-level modules:
//! - [`providers`] — `LLMProvider` trait + `AnthropicProvider` impl (M02 Stage B/C).
//! - [`sdk`] — `AgentSdk<P>` agent loop + `EventPipeline` translator (M02 Stage D).
//! - [`drone_ipc`] — `DroneClient` main-side connection to the M01 drone (M02 Stage D).
//! - [`key_store`] — OS-keychain-backed Anthropic API key storage (M02 Stage E).

pub mod drone_ipc;
/// Verify & Rails primitive — spec §4a (M04 Stage D).
///
/// Hook executor + `JSONLogic`-evaluated rails + globset-backed
/// don't-touch matcher + cross-platform shell wrapper.
pub mod hooks;
pub mod key_store;
/// Plan + Task primitive — spec §3a (M04 Stage B).
pub mod plan;
pub mod providers;
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
