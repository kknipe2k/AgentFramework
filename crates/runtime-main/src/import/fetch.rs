//! The real `reqwest` `Fetcher`.
//!
//! The new runtime-main OS-call-holdout coverage exclusion
//! `src.import.fetch.rs` (CLAUDE.md §5/§6 seam-vs-wrapper category,
//! parallel to `providers/anthropic.rs`).
//!
//! This is the ONLY place an outbound HTTP request is made for an
//! import: it GETs exactly the user-supplied URL and nothing else (Hard
//! Rule 4 — no phone-home). The capability gate runs in
//! `super::fetch_with` BEFORE this is called. The logic-bearing
//! pipeline is unit-tested through the injected `Fetcher` seam; this
//! thin wrapper is excluded from the runtime-main ≥95 gate and instead
//! exercised behaviourally against a local `wiremock` server (no live
//! network in the gate; the `--features integration` live smoke is the
//! optional real-endpoint check).

use super::Fetcher;

/// Production `Fetcher` backed by `reqwest`.
#[derive(Debug, Clone, Default)]
pub struct HttpFetcher;

impl HttpFetcher {
    /// Construct an `HttpFetcher`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Fetcher for HttpFetcher {
    async fn get(&self, url: &str) -> Result<Vec<u8>, String> {
        let resp = reqwest::Client::new()
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
        Ok(bytes.to_vec())
    }
}
