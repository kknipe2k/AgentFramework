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

use super::egress::{FetchHop, Resolver, ValidatedTarget};
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
    async fn fetch_hop(&self, _target: &ValidatedTarget) -> Result<FetchHop, String> {
        todo!("M07.5 Stage B.fix green phase — HttpFetcher::fetch_hop")
    }
}

/// Production `Resolver` — the OS DNS resolver via `tokio`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemResolver;

#[async_trait::async_trait]
impl Resolver for SystemResolver {
    async fn resolve(&self, _host: &str) -> Result<Vec<IpAddr>, String> {
        todo!("M07.5 Stage B.fix green phase — SystemResolver::resolve")
    }
}
