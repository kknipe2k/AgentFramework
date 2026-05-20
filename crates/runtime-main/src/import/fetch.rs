//! The real `reqwest` `Fetcher` + the system DNS `Resolver`.
//!
//! The runtime-main OS-call-holdout coverage exclusion
//! `src.import.fetch.rs` (CLAUDE.md §5/§6 seam-vs-wrapper category,
//! parallel to `providers/anthropic.rs`).
//!
//! This is the ONLY place an outbound HTTP request or a DNS lookup is
//! made for an import. The SSRF egress decision runs in the pure
//! `super::egress` module (`validate_egress`) BEFORE `fetch_hop` is
//! ever called; `fetch_hop` DNS-pins the connection to the validated
//! address and disables redirect-following (`super::fetch_with`
//! re-validates each hop — ADR-0018). The logic-bearing egress checks
//! are unit-tested through the `egress` pure functions + the injected
//! `Resolver` seam; this thin wrapper is excluded from the runtime-main
//! ≥95 gate and instead exercised behaviourally against a local
//! `wiremock` server (no live network in the gate).

use std::net::IpAddr;
use std::time::Duration;

use super::egress::{FetchHop, Resolver, ValidatedTarget, MAX_BODY_BYTES};
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
    async fn fetch_hop(&self, target: &ValidatedTarget) -> Result<FetchHop, String> {
        let client = reqwest::Client::builder()
            // No auto-follow — `fetch_with` re-validates each hop.
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            // DNS-pin: connect to the validated address, not a fresh
            // resolution (the DNS-rebinding defense).
            .resolve(&target.host, target.addr)
            .build()
            .map_err(|e| e.to_string())?;

        let mut resp = client
            .get(target.url.as_str())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();

        if status.is_redirection() {
            let location = resp
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| format!("HTTP {status}: redirect with no Location header"))?;
            // Resolve a relative Location against the current URL.
            let next = target.url.join(location).map_err(|e| e.to_string())?;
            return Ok(FetchHop::Redirect(next.to_string()));
        }
        if !status.is_success() {
            return Err(format!("HTTP {status}"));
        }

        // Streamed, capped body read — robust against a lying
        // Content-Length.
        let mut body = Vec::new();
        while let Some(chunk) = resp.chunk().await.map_err(|e| e.to_string())? {
            body.extend_from_slice(&chunk);
            if body.len() > MAX_BODY_BYTES {
                return Err(format!(
                    "response body exceeds the {MAX_BODY_BYTES}-byte import cap"
                ));
            }
        }
        Ok(FetchHop::Body(body))
    }
}

/// Production `Resolver` — the OS DNS resolver via `tokio`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemResolver;

#[async_trait::async_trait]
impl Resolver for SystemResolver {
    async fn resolve(&self, host: &str) -> Result<Vec<IpAddr>, String> {
        // The port is irrelevant to classification — 443 is a
        // placeholder so `lookup_host` accepts the (host, port) form.
        let addrs = tokio::net::lookup_host((host, 443_u16))
            .await
            .map_err(|e| e.to_string())?;
        Ok(addrs.map(|sa| sa.ip()).collect())
    }
}
