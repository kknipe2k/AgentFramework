//! M07 Stage C / M07.5 Stage B.fix — import-pipeline backend (spec
//! Phase 7 §2152–2211; MVP §M7; ADR-0005 `share_provenance`; ADR-0017
//! validate/commit split; ADR-0018 SSRF egress hardening).
//!
//! Behavioral contract tests for the path-agnostic `import` pipeline:
//! SSRF-hardened URL / local-file fetch → schema validation (the
//! generated typify type is the schema's enforced shape, CLAUDE.md §14)
//! → L3 sandbox (reuse `runtime-sandbox` via the injected `Sandbox`
//! seam) → tier-gate (reuse the M05 L4 `Tier`) → install + `skills.lock`
//! write → `ImportOutcome`.
//!
//! M07.5 Stage B.fix replaces the tautological `NetworkGate` egress
//! check with the real `import::egress` gate (ADR-0018): `classify_ip`
//! rejects every non-public address range, `check_url` is `https`-only,
//! `validate_egress` resolves + classifies + pins, and `fetch_with`
//! re-validates every redirect hop. The egress decision logic is
//! exercised here through the injected `Resolver` seam; the real
//! `reqwest` `HttpFetcher` + `SystemResolver` are exercised against a
//! local `wiremock` server — no live network in the gate (Hard Rule 4).
//!
//! Strict-TDD (CLAUDE.md §6, v1.8 two-commit): every test here lands in
//! the red commit; the impl commit touches zero `**/tests/**` files
//! (`git diff <red>..<impl> -- '**/tests/**'` EMPTY).

use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use reqwest::Url;
use runtime_core::generated::skills_lock::SkillsLock;
use runtime_main::import::egress::{self, EgressReject, FetchHop, Resolver, ValidatedTarget};
use runtime_main::import::{
    self, ArtifactKind, Clock, Fetcher, ImportError, ImportOutcome, ImportSource, L3Report,
    McpRegistry, McpServerImport, Sandbox,
};
use runtime_main::skills_lock;
use runtime_main::tier::Tier;
use serde_json::json;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

// ── fixtures ────────────────────────────────────────────────────────

/// Schema-valid skill (the `schemas/skill.v1.json` `examples[0]` shape,
/// trimmed). The generated `skill::Skill` type IS the schema's enforced
/// surface (CLAUDE.md §14) — this must deserialize.
fn valid_skill_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "name": "pdf-summarizer",
        "version": "1.0.0",
        "description": "Summarize PDFs.",
        "capabilities": {
            "tools_called": [],
            "skills_loaded": [],
            "file_access": { "read": [], "write": [] },
            "network": [],
            "shell": false,
            "spawn_agents": []
        }
    }))
    .unwrap()
}

/// Schema-valid skill with a NON-trivial declared `capabilities` block
/// (M07.E / ADR-0015 — the §M7 review screen's disclosure source). The
/// pipeline's `capability_summary` reads `tools_called` / `network` /
/// `spawn_agents` (str arrays) + `shell` (bool); this fixture populates
/// each so the enriched return carries a real, non-empty disclosure
/// extracted from the artifact (NOT a mocked review payload — the
/// condition-2 anti-false-green anchor). `requires_secrets` is a
/// framework-schema field (§15d), NOT a skill-schema field; the skill
/// disclosure exercise here intentionally omits it (the secrets-notice
/// path is exercised by the framework-shaped `requires_secrets` test
/// `validate_extracts_15d_metadata_with_schema_defaults` above).
fn skill_with_caps_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "name": "fs-test",
        "version": "2.0.0",
        "description": "A skill that touches tools, network, and spawns.",
        "capabilities": {
            "tools_called": ["Read", "Write"],
            "skills_loaded": [],
            "file_access": { "read": [], "write": [] },
            "network": ["api.example.com"],
            "shell": true,
            "spawn_agents": ["sub-agent"]
        }
    }))
    .unwrap()
}

/// Schema-INVALID skill — `capabilities` is required by
/// `schemas/skill.v1.json`. The generated type must reject it.
fn invalid_skill_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "name": "broken",
        "version": "1.0.0",
        "description": "missing capabilities"
    }))
    .unwrap()
}

/// Schema-valid MCP server config (`schemas/mcp.v1.json`: required
/// `name` + `transport`; stdio variant requires `type` + `command`).
fn valid_mcp_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "name": "pdf-mcp",
        "transport": { "type": "stdio", "command": "node", "args": ["server.js"] }
    }))
    .unwrap()
}

/// A framework value carrying an explicit `compatible_os` (spec §15c)
/// and `requires_secrets` (spec §15d) for the metadata-extraction +
/// OS-gate contracts. `compatible_os` is a framework-schema field; the
/// pipeline extracts it generically off the imported JSON so the same
/// gate applies regardless of artifact kind (absent → schema default
/// `["windows","macos","linux"]` → never blocks).
fn framework_value(compatible_os: serde_json::Value) -> serde_json::Value {
    let mut v = json!({
        "name": "demo",
        "version": "1.0.0",
        "description": "demo framework",
        "model": { "provider": "anthropic", "id": "claude-opus-4-7" },
        "tools": [],
        "skills": [],
        "agents": [],
        "session_root_agent": "root",
        "requires_secrets": ["GITHUB_TOKEN"]
    });
    v["compatible_os"] = compatible_os;
    v
}

// ── injected seam fakes ─────────────────────────────────────────────

/// A `Fetcher` that returns the same body in a single hop.
struct FakeFetcher {
    body: Vec<u8>,
}
#[async_trait]
impl Fetcher for FakeFetcher {
    async fn fetch_hop(&self, _target: &ValidatedTarget) -> Result<FetchHop, String> {
        Ok(FetchHop::Body(self.body.clone()))
    }
}

/// A `Fetcher` that always responds with a redirect to a fixed URL —
/// drives the `fetch_with` redirect-revalidation loop.
struct RedirectingFetcher {
    to: String,
}
#[async_trait]
impl Fetcher for RedirectingFetcher {
    async fn fetch_hop(&self, _target: &ValidatedTarget) -> Result<FetchHop, String> {
        Ok(FetchHop::Redirect(self.to.clone()))
    }
}

/// A `Fetcher` replaying a scripted sequence of hops — one entry
/// consumed per `fetch_hop` call.
struct ScriptedFetcher {
    hops: Mutex<VecDeque<FetchHop>>,
}
impl ScriptedFetcher {
    fn new(hops: Vec<FetchHop>) -> Self {
        Self {
            hops: Mutex::new(hops.into()),
        }
    }
}
#[async_trait]
impl Fetcher for ScriptedFetcher {
    async fn fetch_hop(&self, _target: &ValidatedTarget) -> Result<FetchHop, String> {
        self.hops
            .lock()
            .unwrap()
            .pop_front()
            .ok_or_else(|| "ScriptedFetcher: no scripted hop remaining".to_string())
    }
}

/// A `Resolver` mapping every host to a public address (an IP-literal
/// host resolves to itself, mirroring the real resolver). Used by the
/// import-pipeline happy-path tests, where egress must always pass.
struct PublicResolver;
#[async_trait]
impl Resolver for PublicResolver {
    async fn resolve(&self, host: &str) -> Result<Vec<IpAddr>, String> {
        if let Ok(ip) = host.parse::<IpAddr>() {
            return Ok(vec![ip]);
        }
        Ok(vec![IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34))])
    }
}

/// A `Resolver` backed by an explicit host → addresses map. An
/// IP-literal host resolves to itself; an unmapped named host is an
/// error — each SSRF / redirect test configures exactly the mappings
/// it needs.
struct MapResolver {
    map: HashMap<String, Vec<IpAddr>>,
}
impl MapResolver {
    fn new(entries: &[(&str, &[&str])]) -> Self {
        let mut map = HashMap::new();
        for (host, addrs) in entries {
            let parsed = addrs
                .iter()
                .map(|a| a.parse::<IpAddr>().expect("a test IP literal parses"))
                .collect();
            map.insert((*host).to_string(), parsed);
        }
        Self { map }
    }
}
#[async_trait]
impl Resolver for MapResolver {
    async fn resolve(&self, host: &str) -> Result<Vec<IpAddr>, String> {
        if let Ok(ip) = host.parse::<IpAddr>() {
            return Ok(vec![ip]);
        }
        self.map
            .get(host)
            .cloned()
            .ok_or_else(|| format!("MapResolver: no mapping for {host}"))
    }
}

struct OkSandbox;
#[async_trait]
impl Sandbox for OkSandbox {
    async fn validate(&self, _code: &str) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
}

struct RejectSandbox;
#[async_trait]
impl Sandbox for RejectSandbox {
    async fn validate(&self, _code: &str) -> Result<Vec<String>, String> {
        Ok(vec![
            "disallowed syscall: spawn_process (process_spawn)".into()
        ])
    }
}

/// Panics if L3 is reached — used to prove the §15c `compatible_os`
/// gate short-circuits BEFORE the expensive sandbox run (C.3.3).
struct PanicSandbox;
#[async_trait]
impl Sandbox for PanicSandbox {
    async fn validate(&self, _code: &str) -> Result<Vec<String>, String> {
        panic!("L3 must not run when compatible_os mismatches (spec §15c — block before L3)");
    }
}

#[derive(Default)]
struct RecordingRegistry {
    upserts: Mutex<Vec<McpServerImport>>,
}
impl McpRegistry for RecordingRegistry {
    fn upsert(&self, cfg: &McpServerImport) -> Result<(), String> {
        self.upserts.lock().unwrap().push(cfg.clone());
        Ok(())
    }
}

/// Panics if MCP upsert is reached — proves non-MCP imports never
/// touch the M06 registry.
struct PanicRegistry;
impl McpRegistry for PanicRegistry {
    fn upsert(&self, _cfg: &McpServerImport) -> Result<(), String> {
        panic!("non-MCP import must not call the MCP registry");
    }
}

struct FixedClock;
impl Clock for FixedClock {
    fn now(&self) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 18, 14, 23, 0).unwrap()
    }
}

fn lock_path(dir: &tempfile::TempDir) -> PathBuf {
    dir.path().join("skills.lock")
}

/// A `ValidatedTarget` pointing at a local `wiremock` server — lets the
/// `HttpFetcher` holdout tests exercise `fetch_hop` directly (wiremock
/// serves HTTP; the `https`-only `validate_egress` gate is exercised
/// separately through the `Resolver` seam).
fn wiremock_target(server: &MockServer, path: &str) -> ValidatedTarget {
    ValidatedTarget {
        url: Url::parse(&format!("{}/{path}", server.uri())).unwrap(),
        host: "127.0.0.1".to_string(),
        addr: *server.address(),
    }
}

// ── fetch_with: file + URL happy paths ──────────────────────────────

#[tokio::test]
async fn fetch_with_file_source_reads_local_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("artifact.json");
    std::fs::write(&p, b"local-artifact-bytes").unwrap();
    let bytes = import::fetch_with(
        &ImportSource::File(p),
        &FakeFetcher { body: vec![] },
        &PublicResolver,
    )
    .await
    .expect("file fetch reads bytes");
    assert_eq!(bytes, b"local-artifact-bytes");
}

#[tokio::test]
async fn fetch_with_missing_file_is_fetch_error() {
    let dir = tempfile::tempdir().unwrap();
    let err = import::fetch_with(
        &ImportSource::File(dir.path().join("nope.json")),
        &FakeFetcher { body: vec![] },
        &PublicResolver,
    )
    .await
    .expect_err("missing file must error, not pass");
    assert!(matches!(err, ImportError::Fetch(_)), "got {err:?}");
}

#[tokio::test]
async fn fetch_with_url_source_returns_body_when_egress_allowed() {
    let bytes = import::fetch_with(
        &ImportSource::Url("https://raw.githubusercontent.com/o/r/main/skill.json".into()),
        &FakeFetcher {
            body: b"remote-bytes".to_vec(),
        },
        &PublicResolver,
    )
    .await
    .expect("an egress-allowed URL fetch returns the body");
    assert_eq!(bytes, b"remote-bytes");
}

// ── M07.5 / ADR-0018 — the SSRF egress gate ─────────────────────────
//
// `classify_ip` + `check_url` are PURE; `validate_egress` + the
// `fetch_with` redirect loop are exercised through the injected
// `Resolver` seam — no live network. The exhaustive classifier matrix
// IS the security assurance (ADR-0018 — the hand-rolled classifier's
// correctness comes from this matrix, not a crate's reputation).

#[test]
fn classify_ip_rejects_every_internal_and_special_range() {
    // Every non-public range, both families. 169.254.169.254 is the
    // cloud-metadata address; 100.64.0.1 is CGNAT shared space; the
    // ::ffff: pair is the IPv4-mapped-IPv6 SSRF bypass.
    let blocked = [
        "127.0.0.1",
        "10.0.0.1",
        "172.16.0.1",
        "192.168.1.1",
        "169.254.169.254",
        "100.64.0.1",
        "0.0.0.0",
        "255.255.255.255",
        "::1",
        "fe80::1",
        "fc00::1",
        "fd00::1",
        "ff02::1",
        "::",
        "::ffff:127.0.0.1",
        "::ffff:10.0.0.1",
    ];
    for raw in blocked {
        let ip: IpAddr = raw.parse().unwrap();
        assert!(
            egress::classify_ip(ip).is_err(),
            "{raw} must be rejected as a non-public address"
        );
    }
}

#[test]
fn classify_ip_accepts_public_addresses() {
    for raw in ["8.8.8.8", "1.1.1.1", "2606:4700::1111"] {
        let ip: IpAddr = raw.parse().unwrap();
        assert!(
            egress::classify_ip(ip).is_ok(),
            "{raw} is a public address and must be accepted"
        );
    }
}

#[test]
fn classify_ip_unwraps_ipv4_mapped_ipv6_before_classifying() {
    // ::ffff:a.b.c.d smuggles a v4 address inside a v6 — classify_ip
    // MUST unwrap it via to_ipv4_mapped(), or a mapped private address
    // passes as an opaque (public-looking) v6.
    let mapped: IpAddr = "::ffff:127.0.0.1".parse().unwrap();
    assert!(matches!(mapped, IpAddr::V6(_)), "parsed as a v6 address");
    assert_eq!(
        egress::classify_ip(mapped),
        Err(EgressReject::PrivateAddress("127.0.0.1".parse().unwrap())),
        "the mapped loopback is rejected as the v4 address it really is"
    );
}

#[test]
fn check_url_rejects_non_https_schemes() {
    for raw in [
        "http://example.com/x.json",
        "file:///etc/passwd",
        "ftp://ftp.example.com/x",
    ] {
        match egress::check_url(raw) {
            Err(EgressReject::Scheme(_)) => {}
            other => panic!("{raw} must be rejected as a bad scheme, got {other:?}"),
        }
    }
}

#[test]
fn check_url_rejects_an_unparseable_url() {
    match egress::check_url("not a url at all") {
        Err(EgressReject::Parse(_)) => {}
        other => panic!("a non-URL must be a Parse rejection, got {other:?}"),
    }
}

#[test]
fn check_url_accepts_an_https_url() {
    let url = egress::check_url("https://raw.githubusercontent.com/o/r/main/x.json")
        .expect("a well-formed https URL passes");
    assert_eq!(url.scheme(), "https");
}

#[tokio::test]
async fn validate_egress_accepts_a_public_host_and_pins_the_address() {
    let resolver = MapResolver::new(&[("example.com", &["93.184.216.34"])]);
    let target = egress::validate_egress("https://example.com/skill.json", &resolver)
        .await
        .expect("a public host passes egress validation");
    assert_eq!(target.host, "example.com");
    assert_eq!(
        target.addr,
        "93.184.216.34:443".parse::<SocketAddr>().unwrap(),
        "the connection is pinned to the validated address (DNS-rebinding defense)"
    );
}

#[tokio::test]
async fn validate_egress_rejects_a_host_that_resolves_to_a_private_address() {
    // The DNS-rebinding class: a "looks-public" host name that resolves
    // to an internal address. The Resolver seam lets the test prove the
    // ADDRESS — not the name — is what is classified.
    let resolver = MapResolver::new(&[("looks-fine.example", &["127.0.0.1"])]);
    let err = egress::validate_egress("https://looks-fine.example/x.json", &resolver)
        .await
        .expect_err("a host resolving to loopback must be rejected");
    let ImportError::Fetch(m) = &err else {
        panic!("expected ImportError::Fetch, got {err:?}");
    };
    assert!(m.contains("egress blocked"), "names the egress block: {m}");
}

#[tokio::test]
async fn validate_egress_rejects_when_any_resolved_address_is_private() {
    // A host resolving to both a public and a private address is
    // treated as hostile — one bad address fails the whole fetch.
    let resolver = MapResolver::new(&[("mixed.example", &["8.8.8.8", "10.0.0.1"])]);
    let err = egress::validate_egress("https://mixed.example/x.json", &resolver)
        .await
        .expect_err("one private address among the resolved set fails the fetch");
    assert!(matches!(err, ImportError::Fetch(_)), "got {err:?}");
}

#[tokio::test]
async fn validate_egress_rejects_the_ipv4_encoding_bypass() {
    // reqwest::Url canonicalizes 0x7f000001 / 2130706433 to the host
    // 127.0.0.1; the resolver (IP literals resolve to themselves) then
    // yields loopback, which classify_ip rejects.
    let resolver = MapResolver::new(&[]);
    for raw in ["https://0x7f000001/x.json", "https://2130706433/x.json"] {
        let err = egress::validate_egress(raw, &resolver)
            .await
            .expect_err("an IP-encoded loopback URL must be rejected");
        assert!(matches!(err, ImportError::Fetch(_)), "{raw} → {err:?}");
    }
}

#[tokio::test]
async fn validate_egress_rejects_a_non_https_url() {
    let resolver = MapResolver::new(&[("example.com", &["93.184.216.34"])]);
    let err = egress::validate_egress("http://example.com/x.json", &resolver)
        .await
        .expect_err("a non-https URL must be rejected before any resolution");
    assert!(matches!(err, ImportError::Fetch(_)), "got {err:?}");
}

#[tokio::test]
async fn validate_egress_rejects_a_host_with_no_addresses() {
    let resolver = MapResolver::new(&[("void.example", &[])]);
    let err = egress::validate_egress("https://void.example/x.json", &resolver)
        .await
        .expect_err("a host that resolves to nothing must be rejected");
    assert!(matches!(err, ImportError::Fetch(_)), "got {err:?}");
}

#[tokio::test]
async fn validate_egress_surfaces_a_resolver_failure() {
    let resolver = MapResolver::new(&[]);
    let err = egress::validate_egress("https://unmapped.example/x.json", &resolver)
        .await
        .expect_err("a resolver failure surfaces as a fetch error");
    let ImportError::Fetch(m) = &err else {
        panic!("expected ImportError::Fetch, got {err:?}");
    };
    assert!(
        m.contains("DNS resolution failed"),
        "the error names the resolution failure: {m}"
    );
}

// ── fetch_with redirect loop — every hop independently re-validated ──

#[tokio::test]
async fn redirect_to_private_address_is_blocked() {
    // The headline assembled regression (ADR-0016 / CQ-M07-1). The
    // fetcher 302s to a host that resolves PRIVATE; fetch_with must
    // re-validate the REDIRECT target — an original-host-only check
    // would let this through. Drives the real fetch_with redirect loop.
    let resolver = MapResolver::new(&[
        ("raw.githubusercontent.com", &["93.184.216.34"]),
        ("internal.example", &["10.0.0.1"]),
    ]);
    let fetcher = RedirectingFetcher {
        to: "https://internal.example/skill.json".to_string(),
    };
    let err = import::fetch_with(
        &ImportSource::Url("https://raw.githubusercontent.com/o/r/main/skill.json".into()),
        &fetcher,
        &resolver,
    )
    .await
    .expect_err("a redirect to a private address must be blocked on re-validation");
    let ImportError::Fetch(m) = &err else {
        panic!("expected ImportError::Fetch, got {err:?}");
    };
    assert!(
        m.contains("egress blocked"),
        "the redirect target must be blocked AT egress re-validation, not \
         merely exhausted as a redirect loop: {m}"
    );
}

#[tokio::test]
async fn redirect_chain_past_the_cap_is_abandoned() {
    let resolver = MapResolver::new(&[
        ("raw.githubusercontent.com", &["93.184.216.34"]),
        ("hop.example", &["93.184.216.34"]),
    ]);
    let fetcher = RedirectingFetcher {
        to: "https://hop.example/next.json".to_string(),
    };
    let err = import::fetch_with(
        &ImportSource::Url("https://raw.githubusercontent.com/o/r/main/skill.json".into()),
        &fetcher,
        &resolver,
    )
    .await
    .expect_err("an unbounded redirect chain must be abandoned");
    let ImportError::Fetch(m) = &err else {
        panic!("expected ImportError::Fetch, got {err:?}");
    };
    assert!(
        m.contains("too many redirects"),
        "the cap rejection names the limit: {m}"
    );
}

#[tokio::test]
async fn redirect_to_public_then_body_succeeds() {
    let resolver = MapResolver::new(&[
        ("raw.githubusercontent.com", &["93.184.216.34"]),
        ("cdn.example", &["93.184.216.34"]),
    ]);
    let fetcher = ScriptedFetcher::new(vec![
        FetchHop::Redirect("https://cdn.example/skill.json".to_string()),
        FetchHop::Body(b"redirected-body".to_vec()),
    ]);
    let bytes = import::fetch_with(
        &ImportSource::Url("https://raw.githubusercontent.com/o/r/main/skill.json".into()),
        &fetcher,
        &resolver,
    )
    .await
    .expect("a public redirect followed by a body succeeds");
    assert_eq!(bytes, b"redirected-body");
}

// ── HttpFetcher / SystemResolver — the import/fetch.rs OS holdout ────
//
// The real reqwest Fetcher + system resolver hit ONLY the supplied
// target. Exercised against a local wiremock server (HTTP) by calling
// fetch_hop directly with a constructed ValidatedTarget — no live
// network in the gate (Hard Rule 4 / CLAUDE.md capability-adherence).

#[tokio::test]
async fn http_fetcher_returns_body_for_a_2xx_response() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"wiremock-body".to_vec()))
        .mount(&server)
        .await;
    let hop = import::fetch::HttpFetcher::new()
        .fetch_hop(&wiremock_target(&server, "skill.json"))
        .await
        .expect("a 2xx response yields a body hop");
    match hop {
        FetchHop::Body(b) => assert_eq!(b, b"wiremock-body"),
        FetchHop::Redirect(loc) => panic!("expected a body, got a redirect to {loc}"),
    }
}

#[tokio::test]
async fn http_fetcher_maps_a_3xx_to_a_redirect_hop() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(302).insert_header("location", "/moved.json"))
        .mount(&server)
        .await;
    let hop = import::fetch::HttpFetcher::new()
        .fetch_hop(&wiremock_target(&server, "skill.json"))
        .await
        .expect("a 3xx response yields a redirect hop");
    match hop {
        FetchHop::Redirect(loc) => assert!(
            loc.ends_with("/moved.json"),
            "the redirect hop carries the resolved Location: {loc}"
        ),
        FetchHop::Body(_) => panic!("expected a redirect hop, got a body"),
    }
}

#[tokio::test]
async fn http_fetcher_rejects_an_oversize_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200).set_body_bytes(vec![0_u8; egress::MAX_BODY_BYTES + 1]),
        )
        .mount(&server)
        .await;
    let err = import::fetch::HttpFetcher::new()
        .fetch_hop(&wiremock_target(&server, "big.json"))
        .await
        .expect_err("a body past MAX_BODY_BYTES must be rejected");
    assert!(
        err.contains("exceeds"),
        "the body-cap rejection names the limit: {err}"
    );
}

#[tokio::test]
async fn http_fetcher_errors_on_an_http_failure_status() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    let err = import::fetch::HttpFetcher::new()
        .fetch_hop(&wiremock_target(&server, "missing.json"))
        .await
        .expect_err("a 4xx must be a fetch error");
    assert!(err.contains("404"), "the error names the status: {err}");
}

#[tokio::test]
async fn http_fetcher_errors_on_a_redirect_with_no_location() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(302))
        .mount(&server)
        .await;
    let err = import::fetch::HttpFetcher::new()
        .fetch_hop(&wiremock_target(&server, "x.json"))
        .await
        .expect_err("a 3xx with no Location header must error");
    assert!(
        err.contains("Location"),
        "the error names the missing header: {err}"
    );
}

#[tokio::test]
async fn system_resolver_resolves_an_ip_literal_to_itself() {
    // Resolving an IP literal is a no-op lookup — deterministic, no
    // network. (A named-host lookup would need live DNS.)
    let ips = import::fetch::SystemResolver
        .resolve("93.184.216.34")
        .await
        .expect("an IP literal resolves");
    assert_eq!(ips, vec!["93.184.216.34".parse::<IpAddr>().unwrap()]);
}

// ── validate: schema is the source of truth (CLAUDE.md §14) ─────────

#[test]
fn validate_accepts_schema_valid_skill_and_extracts_name_version() {
    let a = import::validate(ArtifactKind::Skill, &valid_skill_bytes())
        .expect("schema-valid skill validates");
    assert_eq!(a.kind, ArtifactKind::Skill);
    assert_eq!(a.name, "pdf-summarizer");
    assert_eq!(a.version, "1.0.0");
    assert_eq!(a.name_at_version(), "pdf-summarizer@1.0.0");
}

#[test]
fn validate_rejects_schema_invalid_artifact_with_report() {
    let err = import::validate(ArtifactKind::Skill, &invalid_skill_bytes())
        .expect_err("skill missing required `capabilities` must be rejected");
    match err {
        ImportError::SchemaInvalid(msg) => assert!(
            !msg.is_empty(),
            "SchemaInvalid must carry the validation report"
        ),
        other => panic!("expected SchemaInvalid, got {other:?}"),
    }
}

#[test]
fn validate_extracts_15d_metadata_with_schema_defaults() {
    // compatible_os absent → schema default (all three OSes); explicit
    // requires_secrets surfaces for the E review screen (spec §15d).
    let bytes = serde_json::to_vec(&framework_value(json!(["linux"]))).unwrap();
    let a = import::validate(ArtifactKind::Agent, &bytes).expect("framework-shaped value parses");
    assert_eq!(a.meta.compatible_os, vec!["linux".to_string()]);
    assert_eq!(a.meta.requires_secrets, vec!["GITHUB_TOKEN".to_string()]);

    let bare = import::validate(ArtifactKind::Skill, &valid_skill_bytes()).expect("skill");
    assert_eq!(
        bare.meta.compatible_os,
        vec![
            "windows".to_string(),
            "macos".to_string(),
            "linux".to_string()
        ],
        "absent compatible_os defaults to all three (spec §15c default)"
    );
}

// ── §15c compatible_os — BLOCKING, checked before L3 ────────────────

#[tokio::test]
async fn compatible_os_mismatch_blocks_before_l3() {
    // spec §15c: a host-OS mismatch is a BLOCKING error, NOT a warning,
    // and it halts BEFORE the expensive L3 sandbox run (C.3.3). The
    // PanicSandbox proves L3 is never reached.
    let dir = tempfile::tempdir().unwrap();
    let bytes = serde_json::to_vec(&framework_value(json!(["linux"]))).unwrap();
    let err = import::import_artifact_with(
        ImportSource::File({
            let p = dir.path().join("a.json");
            std::fs::write(&p, &bytes).unwrap();
            p
        }),
        ArtifactKind::Agent,
        Tier::Promoted,
        "windows",
        &lock_path(&dir),
        &FakeFetcher { body: vec![] },
        &PanicSandbox,
        &PanicRegistry,
        &PublicResolver,
        &FixedClock,
    )
    .await
    .expect_err("linux-only artifact on a windows host must block");
    match err {
        ImportError::OsMismatch { artifact, host } => {
            assert_eq!(artifact, vec!["linux".to_string()]);
            assert_eq!(host, "windows");
        }
        other => panic!("expected OsMismatch, got {other:?}"),
    }
}

#[tokio::test]
async fn compatible_os_match_passes_the_gate() {
    let dir = tempfile::tempdir().unwrap();
    let bytes = serde_json::to_vec(&framework_value(json!(["windows", "linux"]))).unwrap();
    let p = dir.path().join("a.json");
    std::fs::write(&p, &bytes).unwrap();
    let res = import::import_artifact_with(
        ImportSource::File(p),
        ArtifactKind::Agent,
        Tier::Promoted,
        "windows",
        &lock_path(&dir),
        &FakeFetcher { body: vec![] },
        &OkSandbox,
        &PanicRegistry,
        &PublicResolver,
        &FixedClock,
    )
    .await;
    assert!(
        res.is_ok(),
        "windows host in compatible_os must pass: {res:?}"
    );
}

// ── L3 reuse (runtime-sandbox) — reject blocks the install ──────────

#[tokio::test]
async fn l3_rejection_blocks_install_and_writes_no_lock() {
    let dir = tempfile::tempdir().unwrap();
    let lp = lock_path(&dir);
    let err = import::import_artifact_with(
        ImportSource::Url("https://example.com/s.json".into()),
        ArtifactKind::Skill,
        Tier::Promoted,
        "windows",
        &lp,
        &FakeFetcher {
            body: valid_skill_bytes(),
        },
        &RejectSandbox,
        &PanicRegistry,
        &PublicResolver,
        &FixedClock,
    )
    .await
    .expect_err("an L3-rejected artifact must not install");
    match err {
        ImportError::L3(reasons) => assert!(
            reasons.iter().any(|r| r.contains("spawn_process")),
            "L3 error must carry the sandbox rejection reasons: {reasons:?}"
        ),
        other => panic!("expected L3, got {other:?}"),
    }
    assert!(!lp.exists(), "a blocked import must not write skills.lock");
}

// ── L4 tier-gate reuse — Novice → review required, Promoted → pass ──

#[test]
fn tier_gate_novice_requires_review_promoted_passes() {
    let a = import::validate(ArtifactKind::Skill, &valid_skill_bytes()).unwrap();
    let report = L3Report {
        report_id: "vr-1".into(),
        passed: true,
        reasons: vec![],
    };

    let novice = import::tier_gate(&a, Tier::Novice, &report)
        .expect_err("Novice always sees the capability-disclosure review");
    match novice {
        ImportError::TierReviewRequired(review) => {
            assert_eq!(review.l3_report, report, "review carries the L3 report");
            assert!(
                review.share_provenance.is_none(),
                "an unexported artifact carries no share_provenance trust line"
            );
        }
        other => panic!("expected TierReviewRequired, got {other:?}"),
    }

    import::tier_gate(&a, Tier::Promoted, &report)
        .expect("Promoted within bounds is an L4 pass-through (auto-accept)");
}

// ── M07.5 / ADR-0017 — the validate/commit lifecycle split ──────────
//
// These drive the ASSEMBLED `import_artifact_with` composition (not
// `tier_gate` in isolation — that is the M07.V Stage-V blind spot,
// gotcha #82). The phase-doc root cause — "the pipeline installs +
// hash-locks every artifact before a Novice Reject can refuse it" — is
// the falsifiable hypothesis `reject_rolls_back_lock_and_registry`
// disproves: it FAILS on `main` today, where the pipeline installs
// unconditionally (CLAUDE.md §6 assembled-app-regression mandate).

#[tokio::test]
async fn novice_import_returns_pending_and_writes_no_lock() {
    // The assembled-pipeline assertion Stage V never made: a Novice
    // import driven through import_artifact_with is HELD (Pending), and
    // NOTHING is written to skills.lock.
    let dir = tempfile::tempdir().unwrap();
    let lp = lock_path(&dir);
    let outcome = import::import_artifact_with(
        ImportSource::Url("https://raw.githubusercontent.com/o/r/main/pdf.json".into()),
        ArtifactKind::Skill,
        Tier::Novice,
        "windows",
        &lp,
        &FakeFetcher {
            body: valid_skill_bytes(),
        },
        &OkSandbox,
        &PanicRegistry,
        &PublicResolver,
        &FixedClock,
    )
    .await
    .expect("a Novice import is held, not failed");

    let ImportOutcome::Pending { pending, .. } = outcome else {
        panic!("a Novice import must be Pending, got {outcome:?}");
    };
    assert_eq!(pending.lock_key(), "pdf-summarizer@1.0.0");
    assert!(
        !lp.exists(),
        "a held Novice import must write NO skills.lock entry"
    );
}

#[tokio::test]
async fn reject_rolls_back_lock_and_registry() {
    // ADR-0016's named regression — closes M07.V 🔴 #1 (backend). A
    // Novice import of an mcp_server is HELD: no skills.lock entry AND
    // no MCP registry upsert. Dropping the PendingImport (the renderer's
    // "Reject") leaves both stores empty — there is nothing to roll
    // back because the install half never ran. FAILS on `main` today,
    // where import_artifact_with installs + upserts unconditionally.
    let dir = tempfile::tempdir().unwrap();
    let lp = lock_path(&dir);
    let reg = RecordingRegistry::default();
    let outcome = import::import_artifact_with(
        ImportSource::File({
            let p = dir.path().join("mcp.json");
            std::fs::write(&p, valid_mcp_bytes()).unwrap();
            p
        }),
        ArtifactKind::McpServer,
        Tier::Novice,
        "windows",
        &lp,
        &FakeFetcher { body: vec![] },
        &OkSandbox,
        &reg,
        &PublicResolver,
        &FixedClock,
    )
    .await
    .expect("a Novice mcp import is held, not failed");

    let ImportOutcome::Pending { pending, .. } = outcome else {
        panic!("a Novice import must be Pending, got {outcome:?}");
    };
    assert!(!lp.exists(), "a held Novice import writes no skills.lock");
    assert!(
        reg.upserts.lock().unwrap().is_empty(),
        "a held Novice import performs no MCP registry upsert"
    );

    // The renderer's "Reject": drop the held import, call nothing.
    drop(pending);
    assert!(!lp.exists(), "after Reject, still no skills.lock entry");
    assert!(
        reg.upserts.lock().unwrap().is_empty(),
        "after Reject, still no MCP registry row"
    );
}

#[tokio::test]
async fn complete_import_with_installs_after_a_held_review() {
    // The renderer's "Install" confirm: complete_import_with runs the
    // held install half — a skills.lock entry (tier_at_install novice)
    // AND exactly one MCP registry upsert.
    let dir = tempfile::tempdir().unwrap();
    let lp = lock_path(&dir);
    let reg = RecordingRegistry::default();
    let outcome = import::import_artifact_with(
        ImportSource::File({
            let p = dir.path().join("mcp.json");
            std::fs::write(&p, valid_mcp_bytes()).unwrap();
            p
        }),
        ArtifactKind::McpServer,
        Tier::Novice,
        "windows",
        &lp,
        &FakeFetcher { body: vec![] },
        &OkSandbox,
        &reg,
        &PublicResolver,
        &FixedClock,
    )
    .await
    .expect("a Novice mcp import is held");

    let ImportOutcome::Pending { pending, .. } = outcome else {
        panic!("a Novice import must be Pending, got {outcome:?}");
    };
    assert!(!lp.exists(), "precondition: the held import wrote nothing");

    let installed = import::complete_import_with(&pending, &reg, &lp, &FixedClock)
        .expect("completing a held review installs");
    assert_eq!(installed.lock_key, "pdf-mcp@0.0.0");

    let v = serde_json::to_value(skills_lock::read(&lp).unwrap()).unwrap();
    assert_eq!(v["installed"]["pdf-mcp@0.0.0"]["kind"], json!("mcp_server"));
    assert_eq!(
        v["installed"]["pdf-mcp@0.0.0"]["tier_at_install"],
        json!("novice"),
        "tier_at_install records the Novice tier at the deferred install"
    );
    let upserts = reg.upserts.lock().unwrap().clone();
    assert_eq!(
        upserts.len(),
        1,
        "completing a held mcp import upserts once"
    );
    assert_eq!(upserts[0].name, "pdf-mcp");
}

#[tokio::test]
async fn promoted_import_installs_inline() {
    // The Promoted L4 auto-accept path is unchanged by the split — the
    // artifact installs + locks inline, no review held.
    let dir = tempfile::tempdir().unwrap();
    let lp = lock_path(&dir);
    let outcome = import::import_artifact_with(
        ImportSource::Url("https://raw.githubusercontent.com/o/r/main/pdf.json".into()),
        ArtifactKind::Skill,
        Tier::Promoted,
        "windows",
        &lp,
        &FakeFetcher {
            body: valid_skill_bytes(),
        },
        &OkSandbox,
        &PanicRegistry,
        &PublicResolver,
        &FixedClock,
    )
    .await
    .expect("a Promoted import installs");

    assert!(
        matches!(outcome, ImportOutcome::Installed(_)),
        "a Promoted import installs inline, got {outcome:?}"
    );
    assert!(
        lp.exists(),
        "a Promoted import writes the skills.lock entry"
    );
}

// ── full pipeline happy path — install + skills.lock (B reuse) ──────

#[tokio::test]
async fn promoted_url_import_installs_and_writes_spec_faithful_lock_entry() {
    let dir = tempfile::tempdir().unwrap();
    let lp = lock_path(&dir);
    let body = valid_skill_bytes();
    let outcome = import::import_artifact_with(
        ImportSource::Url("https://raw.githubusercontent.com/o/r/main/pdf.json".into()),
        ArtifactKind::Skill,
        Tier::Promoted,
        "windows",
        &lp,
        &FakeFetcher { body: body.clone() },
        &OkSandbox,
        &PanicRegistry,
        &PublicResolver,
        &FixedClock,
    )
    .await
    .expect("happy-path import succeeds");

    let ImportOutcome::Installed(installed) = outcome else {
        panic!("a Promoted import installs inline, got {outcome:?}");
    };
    assert_eq!(installed.lock_key, "pdf-summarizer@1.0.0");

    // The lock entry is spec-faithful (B's shape) and the content hash
    // is B's SRI hash over the fetched bytes — so a later
    // skills_lock::verify of the same bytes passes.
    let lock: SkillsLock = skills_lock::read(&lp).expect("lock written");
    let v = serde_json::to_value(&lock).unwrap();
    let entry = &v["installed"]["pdf-summarizer@1.0.0"];
    assert_eq!(entry["kind"], json!("skill"));
    assert_eq!(
        entry["source"],
        json!({ "type": "url", "url": "https://raw.githubusercontent.com/o/r/main/pdf.json" }),
        "ImportSource::Url must serialize to B's `Source` url shape"
    );
    assert_eq!(
        entry["content_hash"],
        json!(skills_lock::content_hash(&body))
    );
    assert_eq!(entry["installed_at"], json!("2026-05-18T14:23:00Z"));
    assert_eq!(entry["tier_at_install"], json!("promoted"));

    skills_lock::verify(&lp, "pdf-summarizer@1.0.0", &body)
        .expect("the locked hash verifies the originally-fetched bytes");
}

#[tokio::test]
async fn file_import_locks_source_as_file_shape() {
    // B carry-forward: ImportSource::File must serialize to exactly B's
    // `Source` `{ "type": "file", "path": ... }` discriminated shape.
    // Promoted so the install commits inline (a Novice import is held —
    // the deferred-install file-shape path is covered by
    // `complete_import_with_installs_after_a_held_review`).
    let dir = tempfile::tempdir().unwrap();
    let lp = lock_path(&dir);
    let art = dir.path().join("local-skill.json");
    std::fs::write(&art, valid_skill_bytes()).unwrap();
    let outcome = import::import_artifact_with(
        ImportSource::File(art.clone()),
        ArtifactKind::Skill,
        Tier::Promoted,
        "windows",
        &lp,
        &FakeFetcher { body: vec![] },
        &OkSandbox,
        &PanicRegistry,
        &PublicResolver,
        &FixedClock,
    )
    .await
    .expect("Promoted file import installs inline");
    assert!(
        matches!(outcome, ImportOutcome::Installed(_)),
        "a Promoted import installs inline, got {outcome:?}"
    );

    let v = serde_json::to_value(skills_lock::read(&lp).unwrap()).unwrap();
    let src = &v["installed"]["pdf-summarizer@1.0.0"]["source"];
    assert_eq!(src["type"], json!("file"));
    assert_eq!(src["path"], json!(art.to_string_lossy().as_ref()));
    assert_eq!(
        v["installed"]["pdf-summarizer@1.0.0"]["tier_at_install"],
        json!("promoted"),
        "tier_at_install records the tier at install time (spec §2201)"
    );
}

// ── MCP-server-config import → the M06 registry (reuse, inverted) ───

#[tokio::test]
async fn mcp_server_config_import_lands_in_the_m06_registry() {
    let dir = tempfile::tempdir().unwrap();
    let lp = lock_path(&dir);
    let reg = RecordingRegistry::default();
    let outcome = import::import_artifact_with(
        ImportSource::File({
            let p = dir.path().join("mcp.json");
            std::fs::write(&p, valid_mcp_bytes()).unwrap();
            p
        }),
        ArtifactKind::McpServer,
        Tier::Promoted,
        "windows",
        &lp,
        &FakeFetcher { body: vec![] },
        &OkSandbox,
        &reg,
        &PublicResolver,
        &FixedClock,
    )
    .await
    .expect("mcp-server-config import succeeds");

    let ImportOutcome::Installed(installed) = outcome else {
        panic!("a Promoted import installs inline, got {outcome:?}");
    };
    let upserts = reg.upserts.lock().unwrap().clone();
    assert_eq!(upserts.len(), 1, "exactly one registry upsert");
    assert_eq!(upserts[0].name, "pdf-mcp");
    assert_eq!(upserts[0].transport, "stdio");
    assert_eq!(upserts[0].command.as_deref(), Some("node"));

    // And it is still locked like any other artifact (kind mcp_server).
    assert_eq!(installed.lock_key, "pdf-mcp@0.0.0");
    let v = serde_json::to_value(skills_lock::read(&lp).unwrap()).unwrap();
    assert_eq!(v["installed"]["pdf-mcp@0.0.0"]["kind"], json!("mcp_server"));
}

// ── share_provenance (ADR-0005) — runtime-to-runtime round-trip ─────

#[test]
fn share_provenance_round_trips_export_then_import_no_rebake() {
    // ADR-0005 / MVP §M7 line 215: export populates share_provenance;
    // import surfaces it. v0.1 is runtime-to-runtime ONLY —
    // rebake_changes is ALWAYS [] (no Share It module, no rebake).
    let mut fw = framework_value(json!(["windows", "macos", "linux"]));
    assert!(
        fw.get("share_provenance").is_none(),
        "precondition: unexported framework has no provenance"
    );

    let now = Utc.with_ymd_and_hms(2026, 5, 18, 9, 0, 0).unwrap();
    import::export_with_provenance(&mut fw, now);

    let prov = import::read_share_provenance(&fw)
        .expect("import surfaces the exported share_provenance block");
    assert_eq!(prov["exported_at"], json!("2026-05-18T09:00:00+00:00"));
    assert_eq!(prov["exported_by"], json!(import::SHARE_IT_ID));
    assert_eq!(
        prov["rebake_changes"],
        json!([]),
        "v0.1 export is runtime-to-runtime — NEVER any rebake (ADR-0005)"
    );
    assert_eq!(
        prov["for_os"],
        json!(["windows", "macos", "linux"]),
        "for_os mirrors the framework's compatible_os at export time"
    );

    // The exported framework still validates (share_provenance is a
    // known framework-schema field, not free-form drift).
    let bytes = serde_json::to_vec(&fw).unwrap();
    let a = import::validate(ArtifactKind::Agent, &bytes)
        .expect("exported framework still schema-valid");
    assert!(
        a.meta.share_provenance.is_some(),
        "validate() surfaces share_provenance into ArtifactMeta for the E trust signal"
    );
}

#[test]
fn read_share_provenance_is_none_when_absent() {
    let fw = framework_value(json!(["windows"]));
    assert!(
        import::read_share_provenance(&fw).is_none(),
        "no provenance block → None (not a synthesized empty block)"
    );
}

// ── M07.E / ADR-0015 — enriched return for the §M7 review screen ─────
//
// The shipped Stage C `Installed` discarded the capability disclosure +
// L3 report + share_provenance the spec'd review screen requires. These
// tests drive the REAL `import_artifact_with` pipeline (real fetch seam
// → real validate → real Sandbox seam → real install) and assert the
// ENRICHED return carries them — extracted from the real artifact, NOT
// a mocked review payload (the condition-2 anti-false-green anchor;
// ADR-0015 Decision/Consequences).

#[tokio::test]
async fn enriched_install_carries_real_capability_disclosure() {
    // capability_summary runs over the artifact's REAL `capabilities`
    // block (skill_with_caps_bytes) — the disclosure the renderer shows
    // is the artifact's own declaration, surfaced verbatim.
    let dir = tempfile::tempdir().unwrap();
    let lp = lock_path(&dir);
    let outcome = import::import_artifact_with(
        ImportSource::Url("https://raw.githubusercontent.com/o/r/main/fs.json".into()),
        ArtifactKind::Skill,
        Tier::Novice,
        "windows",
        &lp,
        &FakeFetcher {
            body: skill_with_caps_bytes(),
        },
        &OkSandbox,
        &PanicRegistry,
        &PublicResolver,
        &FixedClock,
    )
    .await
    .expect("a Novice import is held for review");

    let ImportOutcome::Pending { review, pending } = outcome else {
        panic!("a Novice import is held as Pending, got {outcome:?}");
    };
    assert_eq!(pending.lock_key(), "fs-test@2.0.0");
    // Real extraction from the artifact's declared capabilities — order
    // follows capability_summary's key walk (tools_called, network,
    // spawn_agents, then shell).
    assert_eq!(
        review.capabilities,
        vec![
            "tools_called: Read".to_string(),
            "tools_called: Write".to_string(),
            "network: api.example.com".to_string(),
            "spawn_agents: sub-agent".to_string(),
            "shell: true".to_string(),
        ],
        "the held review must carry the artifact's REAL declared \
         capability disclosure (ADR-0015)"
    );
}

#[tokio::test]
async fn enriched_install_carries_l3_report_and_present_provenance() {
    // L3Report is already built by the pipeline; the enriched return
    // exposes it. share_provenance is surfaced when the imported
    // artifact carries an exported block (ADR-0005 round-trip).
    let dir = tempfile::tempdir().unwrap();
    let lp = lock_path(&dir);
    let mut fw = framework_value(json!(["windows", "macos", "linux"]));
    import::export_with_provenance(&mut fw, Utc.with_ymd_and_hms(2026, 5, 18, 9, 0, 0).unwrap());
    let p = dir.path().join("fw.json");
    std::fs::write(&p, serde_json::to_vec(&fw).unwrap()).unwrap();

    let outcome = import::import_artifact_with(
        ImportSource::File(p),
        ArtifactKind::Agent,
        Tier::Novice,
        "windows",
        &lp,
        &FakeFetcher { body: vec![] },
        &OkSandbox,
        &PanicRegistry,
        &PublicResolver,
        &FixedClock,
    )
    .await
    .expect("provenance-carrying framework is held for review");

    let ImportOutcome::Pending { review, .. } = outcome else {
        panic!("a Novice import is held as Pending, got {outcome:?}");
    };
    assert!(review.l3_report.passed, "L3 cleared → report.passed");
    assert!(
        review.l3_report.reasons.is_empty(),
        "a passing L3 report carries no reasons"
    );
    let prov = review
        .share_provenance
        .as_ref()
        .expect("an exported artifact surfaces share_provenance (ADR-0005)");
    assert_eq!(
        prov["rebake_changes"],
        json!([]),
        "v0.1 is runtime-to-runtime — rebake_changes is ALWAYS [] (ADR-0005)"
    );
    assert_eq!(prov["exported_by"], json!(import::SHARE_IT_ID));
}

#[tokio::test]
async fn enriched_install_provenance_is_none_when_artifact_unexported() {
    // No export block on the artifact → share_provenance is None (not a
    // synthesized empty block); the renderer renders the "no provenance"
    // state, never fabricated data.
    let dir = tempfile::tempdir().unwrap();
    let lp = lock_path(&dir);
    let outcome = import::import_artifact_with(
        ImportSource::Url("https://raw.githubusercontent.com/o/r/main/s.json".into()),
        ArtifactKind::Skill,
        Tier::Promoted,
        "windows",
        &lp,
        &FakeFetcher {
            body: skill_with_caps_bytes(),
        },
        &OkSandbox,
        &PanicRegistry,
        &PublicResolver,
        &FixedClock,
    )
    .await
    .expect("unexported import succeeds");

    let ImportOutcome::Installed(installed) = outcome else {
        panic!("a Promoted import installs inline, got {outcome:?}");
    };
    assert!(
        installed.share_provenance.is_none(),
        "an unexported artifact has no share_provenance — None, not {{}}"
    );
}
