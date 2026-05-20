//! M07 Stage C — import-pipeline backend (spec Phase 7 §2152–2211;
//! MVP §M7; ADR-0005 `share_provenance`; §15c/§15d metadata).
//!
//! Behavioral contract tests for the path-agnostic `import` pipeline:
//! capability-gated URL / local-file fetch → schema validation (the
//! generated typify type is the schema's enforced shape, CLAUDE.md §14)
//! → L3 sandbox (reuse `runtime-sandbox` via the injected `Sandbox`
//! seam) → tier-gate (reuse the M05 L4 `Tier`) → install + `skills.lock`
//! write (reuse the M07.B `skills_lock` module) → `Installed`.
//! `compatible_os` mismatch is a BLOCKING `OsMismatch` checked BEFORE
//! the expensive L3 (spec §15c). `share_provenance` round-trips
//! export→import, runtime-to-runtime only (`rebake_changes` always `[]`,
//! ADR-0005). MCP-server-config import routes into the injected
//! `McpRegistry` seam (the M06 MCP Manager — dependency-inverted to
//! avoid the `runtime-mcp → runtime-main` Cargo cycle, the
//! `sdk::mcp_dispatch` archetype).
//!
//! Every stage takes an injected fake so the pipeline is fully
//! unit-testable in `runtime-main`; the real reqwest `Fetcher` lives in
//! `import::fetch` (the new runtime-main OS-call-holdout coverage
//! exclusion `src.import.fetch.rs`) and is exercised here against a
//! local `wiremock` server — no live network in the gate (CLAUDE.md
//! capability-adherence rule; Hard Rule 4).
//!
//! Strict-TDD (CLAUDE.md §6, v1.8 two-commit): every test here lands in
//! the red commit; the impl commit touches zero `**/tests/**` files
//! (`git diff <red>..<impl> -- '**/tests/**'` EMPTY).

use std::path::PathBuf;
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use runtime_core::generated::skills_lock::SkillsLock;
use runtime_main::import::{
    self, ArtifactKind, Clock, Fetcher, ImportError, ImportSource, L3Report, McpRegistry,
    McpServerImport, NetworkGate, Sandbox,
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

struct FakeFetcher {
    body: Vec<u8>,
}
#[async_trait]
impl Fetcher for FakeFetcher {
    async fn get(&self, _url: &str) -> Result<Vec<u8>, String> {
        Ok(self.body.clone())
    }
}

struct AllowGate;
impl NetworkGate for AllowGate {
    fn check(&self, _host: &str) -> Result<(), String> {
        Ok(())
    }
}

struct DenyGate;
impl NetworkGate for DenyGate {
    fn check(&self, host: &str) -> Result<(), String> {
        Err(format!("capability denied for {host}"))
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

// ── fetch_with: file + URL happy paths + capability gate ────────────

#[tokio::test]
async fn fetch_with_file_source_reads_local_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("artifact.json");
    std::fs::write(&p, b"local-artifact-bytes").unwrap();
    let bytes = import::fetch_with(
        &ImportSource::File(p),
        &AllowGate,
        &FakeFetcher { body: vec![] },
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
        &AllowGate,
        &FakeFetcher { body: vec![] },
    )
    .await
    .expect_err("missing file must error, not pass");
    assert!(matches!(err, ImportError::Fetch(_)), "got {err:?}");
}

#[tokio::test]
async fn fetch_with_url_source_returns_body_when_capability_allowed() {
    let bytes = import::fetch_with(
        &ImportSource::Url("https://raw.githubusercontent.com/o/r/main/skill.json".into()),
        &AllowGate,
        &FakeFetcher {
            body: b"remote-bytes".to_vec(),
        },
    )
    .await
    .expect("allowed URL fetch returns body");
    assert_eq!(bytes, b"remote-bytes");
}

#[tokio::test]
async fn fetch_with_url_is_blocked_when_network_capability_denied() {
    // Hard Rule 4 — egress is capability-gated through the M05 L1
    // enforcer; a denied capability blocks the fetch BEFORE any GET.
    let err = import::fetch_with(
        &ImportSource::Url("https://evil.example.com/x.json".into()),
        &DenyGate,
        &FakeFetcher {
            body: b"should-never-be-returned".to_vec(),
        },
    )
    .await
    .expect_err("denied network capability must block the fetch");
    match err {
        ImportError::Fetch(m) => assert!(
            m.contains("capability denied"),
            "fetch error must name the capability denial: {m}"
        ),
        other => panic!("expected Fetch(capability denied), got {other:?}"),
    }
}

#[tokio::test]
async fn real_http_fetcher_fetches_from_wiremock_no_live_network() {
    // The real reqwest `Fetcher` (import::fetch — the new
    // src.import.fetch.rs coverage holdout) hits ONLY the user-supplied
    // URL. Exercised against a local wiremock server: no live network
    // in the gate (Hard Rule 4 / CLAUDE.md capability-adherence).
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"wiremock-body".to_vec()))
        .mount(&server)
        .await;
    let fetcher = import::fetch::HttpFetcher::new();
    let bytes = import::fetch_with(
        &ImportSource::Url(format!("{}/skill.json", server.uri())),
        &AllowGate,
        &fetcher,
    )
    .await
    .expect("real fetcher returns the wiremock body");
    assert_eq!(bytes, b"wiremock-body");
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
        &AllowGate,
        &FakeFetcher { body: vec![] },
        &PanicSandbox,
        &PanicRegistry,
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
        &AllowGate,
        &FakeFetcher { body: vec![] },
        &OkSandbox,
        &PanicRegistry,
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
        &AllowGate,
        &FakeFetcher {
            body: valid_skill_bytes(),
        },
        &RejectSandbox,
        &PanicRegistry,
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
        }
        other => panic!("expected TierReviewRequired, got {other:?}"),
    }

    import::tier_gate(&a, Tier::Promoted, &report)
        .expect("Promoted within bounds is an L4 pass-through (auto-accept)");
}

// ── full pipeline happy path — install + skills.lock (B reuse) ──────

#[tokio::test]
async fn promoted_url_import_installs_and_writes_spec_faithful_lock_entry() {
    let dir = tempfile::tempdir().unwrap();
    let lp = lock_path(&dir);
    let body = valid_skill_bytes();
    let installed = import::import_artifact_with(
        ImportSource::Url("https://raw.githubusercontent.com/o/r/main/pdf.json".into()),
        ArtifactKind::Skill,
        Tier::Promoted,
        "windows",
        &lp,
        &AllowGate,
        &FakeFetcher { body: body.clone() },
        &OkSandbox,
        &PanicRegistry,
        &FixedClock,
    )
    .await
    .expect("happy-path import succeeds");

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
    let dir = tempfile::tempdir().unwrap();
    let lp = lock_path(&dir);
    let art = dir.path().join("local-skill.json");
    std::fs::write(&art, valid_skill_bytes()).unwrap();
    import::import_artifact_with(
        ImportSource::File(art.clone()),
        ArtifactKind::Skill,
        Tier::Novice,
        "windows",
        &lp,
        &AllowGate,
        &FakeFetcher { body: vec![] },
        &OkSandbox,
        &PanicRegistry,
        &FixedClock,
    )
    .await
    .expect("Novice file import installs (review is a renderer concern; backend installs)");

    let v = serde_json::to_value(skills_lock::read(&lp).unwrap()).unwrap();
    let src = &v["installed"]["pdf-summarizer@1.0.0"]["source"];
    assert_eq!(src["type"], json!("file"));
    assert_eq!(src["path"], json!(art.to_string_lossy().as_ref()));
    assert_eq!(
        v["installed"]["pdf-summarizer@1.0.0"]["tier_at_install"],
        json!("novice"),
        "tier_at_install records the tier at install time (spec §2201)"
    );
}

// ── MCP-server-config import → the M06 registry (reuse, inverted) ───

#[tokio::test]
async fn mcp_server_config_import_lands_in_the_m06_registry() {
    let dir = tempfile::tempdir().unwrap();
    let lp = lock_path(&dir);
    let reg = RecordingRegistry::default();
    let installed = import::import_artifact_with(
        ImportSource::File({
            let p = dir.path().join("mcp.json");
            std::fs::write(&p, valid_mcp_bytes()).unwrap();
            p
        }),
        ArtifactKind::McpServer,
        Tier::Promoted,
        "windows",
        &lp,
        &AllowGate,
        &FakeFetcher { body: vec![] },
        &OkSandbox,
        &reg,
        &FixedClock,
    )
    .await
    .expect("mcp-server-config import succeeds");

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
    let installed = import::import_artifact_with(
        ImportSource::Url("https://raw.githubusercontent.com/o/r/main/fs.json".into()),
        ArtifactKind::Skill,
        Tier::Novice,
        "windows",
        &lp,
        &AllowGate,
        &FakeFetcher {
            body: skill_with_caps_bytes(),
        },
        &OkSandbox,
        &PanicRegistry,
        &FixedClock,
    )
    .await
    .expect("happy-path import succeeds");

    assert_eq!(installed.lock_key, "fs-test@2.0.0");
    // Real extraction from the artifact's declared capabilities — order
    // follows capability_summary's key walk (tools_called, network,
    // spawn_agents, then shell).
    assert_eq!(
        installed.capabilities,
        vec![
            "tools_called: Read".to_string(),
            "tools_called: Write".to_string(),
            "network: api.example.com".to_string(),
            "spawn_agents: sub-agent".to_string(),
            "shell: true".to_string(),
        ],
        "the enriched return must carry the artifact's REAL declared \
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

    let installed = import::import_artifact_with(
        ImportSource::File(p),
        ArtifactKind::Agent,
        Tier::Novice,
        "windows",
        &lp,
        &AllowGate,
        &FakeFetcher { body: vec![] },
        &OkSandbox,
        &PanicRegistry,
        &FixedClock,
    )
    .await
    .expect("provenance-carrying framework imports");

    assert!(installed.report.passed, "L3 cleared → report.passed");
    assert!(
        installed.report.reasons.is_empty(),
        "a passing L3 report carries no reasons"
    );
    let prov = installed
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
    let installed = import::import_artifact_with(
        ImportSource::Url("https://raw.githubusercontent.com/o/r/main/s.json".into()),
        ArtifactKind::Skill,
        Tier::Promoted,
        "windows",
        &lp,
        &AllowGate,
        &FakeFetcher {
            body: skill_with_caps_bytes(),
        },
        &OkSandbox,
        &PanicRegistry,
        &FixedClock,
    )
    .await
    .expect("unexported import succeeds");

    assert!(
        installed.share_provenance.is_none(),
        "an unexported artifact has no share_provenance — None, not {{}}"
    );
}
