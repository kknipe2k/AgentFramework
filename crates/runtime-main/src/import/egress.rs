//! Import-fetch egress security — SSRF defense for the import URL.
//!
//! The user-supplied import URL is fetched server-side (M07.5 /
//! ADR-0018). `classify_ip` + `check_url` are PURE — fully unit-tested.
//! DNS resolution is behind the injected `Resolver` seam (the `Fetcher`
//! / `Sandbox` / `Clock` injected-seam archetype) so tests inject "this
//! host resolves to 127.0.0.1" and assert rejection without touching
//! the network. Only the real syscalls (the `reqwest` GET, the DNS
//! lookup) live in `import/fetch.rs`, the runtime-main OS-call holdout.

use std::net::{IpAddr, SocketAddr};

use reqwest::Url;

use super::ImportError;

/// Largest import artifact body accepted (anti-DoS / decompression cap).
pub const MAX_BODY_BYTES: usize = 8 * 1024 * 1024;

/// Largest redirect chain followed before the fetch is abandoned.
pub const MAX_REDIRECTS: usize = 5;

/// Why an egress request was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EgressReject {
    /// The URL did not parse.
    Parse(String),
    /// The scheme is not `https`.
    Scheme(String),
    /// The URL has no host component.
    NoHost,
    /// A resolved address is not publicly routable (the SSRF defense).
    PrivateAddress(IpAddr),
    /// The host resolved to no addresses.
    NoAddress(String),
}

/// Classify a resolved IP — `Ok` only for a publicly-routable address.
///
/// PURE. `IPv4`-mapped `IPv6` (`::ffff:a.b.c.d`) is unwrapped to its v4
/// form FIRST — a mapped private address is a classic SSRF bypass. The
/// CGNAT (`100.64/10`), `IPv6` ULA (`fc00::/7`), and `IPv6` link-local
/// (`fe80::/10`) ranges are hand-coded because `std`'s predicates for
/// them are unstable on the project MSRV (ADR-0018 §Notes).
///
/// # Errors
///
/// [`EgressReject::PrivateAddress`] for any non-public address.
pub fn classify_ip(ip: IpAddr) -> Result<(), EgressReject> {
    // Unwrap an IPv4-mapped IPv6 address so a mapped private address
    // classifies as the v4 address it really is.
    let normalized = match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map_or(ip, IpAddr::V4),
        IpAddr::V4(_) => ip,
    };
    let blocked = match normalized {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_multicast()
                || v4.is_documentation()
                // CGNAT shared address space 100.64.0.0/10.
                || (v4.octets()[0] == 100 && (v4.octets()[1] & 0xc0) == 0x40)
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                // Unique local fc00::/7.
                || (v6.segments()[0] & 0xfe00) == 0xfc00
                // Link-local unicast fe80::/10.
                || (v6.segments()[0] & 0xffc0) == 0xfe80
        }
    };
    if blocked {
        Err(EgressReject::PrivateAddress(normalized))
    } else {
        Ok(())
    }
}

/// Parse the raw URL and enforce the `https`-only scheme allowlist.
///
/// PURE. `reqwest::Url` is the WHATWG parser — it canonicalizes `IPv4`
/// encoding tricks (`0x7f000001`, `2130706433`) to a normal host, so a
/// later [`classify_ip`] sees the real address.
///
/// # Errors
///
/// [`EgressReject::Parse`] when the URL is malformed;
/// [`EgressReject::Scheme`] when the scheme is not `https`.
pub fn check_url(raw: &str) -> Result<Url, EgressReject> {
    let url = Url::parse(raw).map_err(|e| EgressReject::Parse(e.to_string()))?;
    if url.scheme() == "https" {
        Ok(url)
    } else {
        Err(EgressReject::Scheme(url.scheme().to_string()))
    }
}

/// DNS-resolution seam for egress validation.
///
/// The real impl is [`super::fetch::SystemResolver`]
/// (`tokio::net::lookup_host`); tests inject a fake mapping host →
/// addresses. Async because the real path is a syscall.
#[async_trait::async_trait]
pub trait Resolver: Send + Sync {
    /// Resolve `host` (a domain or an IP literal, no brackets) to its
    /// addresses.
    ///
    /// # Errors
    ///
    /// The resolver failure, stringified.
    async fn resolve(&self, host: &str) -> Result<Vec<IpAddr>, String>;
}

/// A URL that passed egress validation, with its pinned address.
///
/// The HTTP connection is pinned to `addr` (the DNS-rebinding defense —
/// the client must not re-resolve).
#[derive(Debug, Clone)]
pub struct ValidatedTarget {
    /// The parsed, scheme-checked URL.
    pub url: Url,
    /// The host string (for the `Host:` header / DNS-pin key).
    pub host: String,
    /// The validated address the connection pins to.
    pub addr: SocketAddr,
}

/// One hop of a fetch: a final body, or a redirect to re-validate.
#[derive(Debug)]
pub enum FetchHop {
    /// The terminal response body.
    Body(Vec<u8>),
    /// A 3xx `Location` — `fetch_with` re-validates it before following.
    Redirect(String),
}

/// Validate a URL for egress — parse, `https`, resolve, classify, pin.
///
/// Every resolved address is classified; a single non-public address
/// fails the whole request. This is the one egress decision point
/// `fetch_with` calls — for the initial URL and every redirect target.
///
/// # Errors
///
/// [`ImportError::Fetch`] carrying the [`EgressReject`] reason, or the
/// resolver's own failure.
pub async fn validate_egress(
    raw: &str,
    resolver: &dyn Resolver,
) -> Result<ValidatedTarget, ImportError> {
    let url = check_url(raw).map_err(|r| reject_to_import(&r))?;
    // `check_url` accepts only `https`, which always carries a host;
    // the `None` arm is unreachable but handled without a panic.
    let Some(raw_host) = url.host_str() else {
        return Err(reject_to_import(&EgressReject::NoHost));
    };
    let host = raw_host
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_string();
    let port = url.port_or_known_default().unwrap_or(443);

    let ips = resolver
        .resolve(&host)
        .await
        .map_err(|e| ImportError::Fetch(format!("DNS resolution failed: {e}")))?;
    let Some(&first) = ips.first() else {
        return Err(reject_to_import(&EgressReject::NoAddress(host)));
    };
    // A host resolving to ANY non-public address is treated as hostile
    // (a host that resolves to both public and private fails too).
    for &ip in &ips {
        classify_ip(ip).map_err(|r| reject_to_import(&r))?;
    }
    Ok(ValidatedTarget {
        url,
        host,
        addr: SocketAddr::new(first, port),
    })
}

/// Map an [`EgressReject`] onto the pipeline's [`ImportError::Fetch`]
/// — consistent with how the old `NetworkGate` denial surfaced (no new
/// `ImportError` variant).
fn reject_to_import(reason: &EgressReject) -> ImportError {
    ImportError::Fetch(format!("egress blocked: {reason:?}"))
}
