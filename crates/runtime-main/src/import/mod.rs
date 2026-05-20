//! Import-pipeline backend — spec Phase 7 §2152-2211; MVP §M7; M07
//! Stage C.
//!
//! Composes the artifact import flow over already-shipped primitives —
//! it builds NOTHING that M05/M06/M07.B already provide:
//!
//! - **fetch** — local-file read or SSRF-hardened URL GET. URL egress
//!   is gated through the pure `import::egress` module (ADR-0018):
//!   `https`-only, every resolved IP classified against the non-public
//!   ranges, the connection DNS-pinned, and every redirect hop
//!   re-validated. Only the user-supplied URL (and any public redirect
//!   target) is hit, no phone-home (Hard Rule 4). The real `reqwest`
//!   impl is `fetch::HttpFetcher` + `fetch::SystemResolver` — the
//!   runtime-main OS-call-holdout coverage exclusion
//!   `src.import.fetch.rs`; the egress decision logic is the pure,
//!   fully-tested `egress` module, only the socket + DNS syscalls are
//!   the holdout.
//! - **validate** — the schema is the source of truth (CLAUDE.md §14):
//!   skill / tool / mcp_server are gated by deserializing into their
//!   generated typify type. Identity (`name`/`version`) + the §15c/§15d
//!   metadata + `share_provenance` are extracted generically off the
//!   imported JSON so the same `compatible_os` gate and the E
//!   trust-signal apply uniformly regardless of kind. Agent imports are
//!   identity+metadata-validated here; the full agent-graph schema is
//!   enforced by `framework_loader` at framework load, not at import
//!   (the two layers are deliberately distinct — import gates supply
//!   chain + integrity, load gates the agent graph).
//! - **L3** — reuse `runtime-sandbox` via the injected `Sandbox` seam
//!   (the real adapter wraps `sandbox_ipc::SandboxClient`).
//! - **L4 tier-gate** — reuse the M05 `Tier`: Novice always returns
//!   `ImportError::TierReviewRequired` (the renderer shows the
//!   capability-disclosure review screen, Stage E); Promoted is the L4
//!   pass-through (auto-accept) per `tier` module semantics.
//! - **install + lock** — reuse the M07.B `skills_lock` module:
//!   `skills_lock::content_hash` over the fetched bytes +
//!   `skills_lock::write_entry`. `ImportSource` serializes to
//!   exactly B's discriminated `Source` shape so a later
//!   `skills_lock::verify` of the same bytes passes.
//! - **MCP-server-config import** — routes into the M06 MCP Manager via
//!   the injected `McpRegistry` seam. `runtime-mcp` depends on
//!   `runtime-main`, so a direct dependency would close a Cargo cycle;
//!   the concrete registry adapter is constructed in the Tauri shell
//!   (the `sdk::mcp_dispatch` / ADR-0010 dependency-inversion archetype).
//!
//! `compatible_os` mismatch is a BLOCKING `ImportError::OsMismatch`
//! checked BEFORE the expensive L3 run (spec §15c — fail loudly, do not
//! silently misbehave). `share_provenance` round-trips export→import,
//! runtime-to-runtime ONLY: `rebake_changes` is ALWAYS `[]` — there is
//! no Share It module and no rebake in v0.1 (ADR-0005; the Sigstore /
//! SLSA / TUF provenance layer attaches at this same seam in v1.0).
//!
//! Every stage is an injected-seam function so the pipeline is fully
//! unit-testable in `runtime-main` (CLAUDE.md §5 `*_with` archetype);
//! the Tauri command is the thin §5 shell holdout.

/// The real `reqwest` `Fetcher` — the `src.import.fetch.rs` OS-call
/// holdout (seam-tested; behaviourally smoke-tested via `wiremock`).
pub mod fetch;

/// Import-fetch egress security — the SSRF-hardened egress gate.
///
/// `classify_ip` / `check_url` / `validate_egress` (ADR-0018) — pure
/// decision logic, fully tested to the runtime-main ≥95 gate.
pub mod egress;

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde_json::{json, Value};

use crate::skills_lock;
use crate::tier::Tier;

/// `share-it` exporter identity stamped into `share_provenance`
/// (`schemas/framework.v1.json` `^share-it@\d+\.\d+\.\d+$`). v0.1 only
/// populates provenance for runtime-to-runtime transfer — no rebake.
pub const SHARE_IT_ID: &str = "share-it@0.1.0";

/// Default OS support when an artifact does not narrow `compatible_os`
/// (spec §15c default — assume portable across all three v0.1 targets).
const ALL_OS: [&str; 3] = ["windows", "macos", "linux"];

/// Where an artifact is imported from. Serializes to exactly the M07.B
/// `skills_lock` `Source` discriminated shape (`{ "type": "url"|"file",
/// .. }`) so a lock entry written here round-trips B's schema.
#[derive(Debug, Clone)]
pub enum ImportSource {
    /// GitHub-raw / HTTPS URL (capability-gated fetch).
    Url(String),
    /// Local filesystem path (file picker).
    File(PathBuf),
}

/// The artifact primitive being imported. Mirrors the M07.B
/// `skills_lock` `ArtifactKind` wire values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    /// A context-loaded skill (`schemas/skill.v1.json`).
    Skill,
    /// A callable tool (`schemas/tool.v1.json`).
    Tool,
    /// An agent definition (`schemas/agent.v1.json`).
    Agent,
    /// An MCP server config (`schemas/mcp.v1.json`) — installs into the
    /// M06 MCP Manager registry.
    McpServer,
}

impl ArtifactKind {
    /// The `skills_lock` schema wire string for this kind.
    #[must_use]
    pub const fn wire(self) -> &'static str {
        match self {
            Self::Skill => "skill",
            Self::Tool => "tool",
            Self::Agent => "agent",
            Self::McpServer => "mcp_server",
        }
    }
}

/// §15d metadata read off the imported JSON (framework-schema fields;
/// extracted generically so the gate is kind-uniform). Absent fields
/// take the schema default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactMeta {
    /// Named secrets the recipient must provision before first run
    /// (spec §15d — surfaced on the Stage E review screen).
    pub requires_secrets: Vec<String>,
    /// `desktop_runtime` (default) or `headless_compatible` (spec §15d).
    pub runtime_dependency_class: String,
    /// OSes the artifact supports (spec §15c). Absent → all three.
    pub compatible_os: Vec<String>,
    /// The `share_provenance` trust block (ADR-0005), if exported. Read
    /// only — surfaced to the renderer, never a v0.1 gate.
    pub share_provenance: Option<Value>,
}

/// A schema-validated artifact plus its extracted identity + metadata.
#[derive(Debug, Clone)]
pub struct ValidatedArtifact {
    /// The imported primitive kind.
    pub kind: ArtifactKind,
    /// Artifact `name`.
    pub name: String,
    /// Artifact `version` (`0.0.0` when the kind carries none, e.g. MCP).
    pub version: String,
    /// The exact fetched bytes — hashed into the lock so a later
    /// `skills_lock::verify` of the same bytes passes.
    pub bytes: Vec<u8>,
    /// §15c/§15d metadata + `share_provenance`.
    pub meta: ArtifactMeta,
    raw: Value,
}

impl ValidatedArtifact {
    /// The `name@version` lock key (spec §2200).
    #[must_use]
    pub fn name_at_version(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

/// L3 sandbox outcome for the imported artifact.
///
/// `Serialize` is derived so the §M7 review screen's L3 sub-object
/// rides verbatim inside the enriched `ImportOutcome` Tauri-bridge
/// shape (ADR-0015). The crate-internal `PartialEq`/`Eq` semantics for
/// in-process comparisons are unaffected.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct L3Report {
    /// Opaque report id recorded in the lock (`validation_report_id`).
    pub report_id: String,
    /// Whether the artifact cleared L3.
    pub passed: bool,
    /// Per-syscall rejection reasons (empty when `passed`).
    pub reasons: Vec<String>,
}

/// The capability disclosure the Stage E review screen renders for a
/// Novice import (spec §M7 — "Novice sees the disclosure + L3 report").
///
/// `Eq` is not derived: `share_provenance` is a `serde_json::Value`,
/// which is `PartialEq` but not `Eq`. `PartialEq` (used by `assert_eq!`)
/// is preserved, mirroring the `Installed` struct's derive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TierReview {
    /// Human-readable declared-capability summary for the disclosure.
    pub capabilities: Vec<String>,
    /// The L3 report shown alongside the disclosure.
    pub l3_report: L3Report,
    /// Secrets to provision before first run (spec §15d).
    pub requires_secrets: Vec<String>,
    /// The ADR-0005 `share_provenance` trust block (when exported) — the
    /// review modal renders it as a trust line. `None` when the artifact
    /// was never exported through the trust chain.
    pub share_provenance: Option<Value>,
}

/// A completed install.
///
/// Enriched at M07.E per ADR-0015 with the §M7 review fields the
/// pipeline already computes: `capabilities` (`capability_summary` over
/// the artifact's declared `capabilities` block) and `share_provenance`
/// (the ADR-0005 trust block, when the artifact was exported). The
/// `report` is the same `L3Report` `install_with` recorded. `Eq` is
/// dropped because `serde_json::Value` is `PartialEq` but not `Eq`
/// (its `Number` variant can hold a non-totally-ordered float);
/// `PartialEq` (used by `assert_eq!`) is preserved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installed {
    /// The `name@version` written into `skills.lock`.
    pub lock_key: String,
    /// The L3 report recorded for this install.
    pub report: L3Report,
    /// Secrets the artifact needs before first run (for the E notice).
    pub requires_secrets: Vec<String>,
    /// Plain-English declared-capability summary extracted by
    /// `capability_summary` for the §M7 disclosure (ADR-0015).
    pub capabilities: Vec<String>,
    /// The `share_provenance` trust block (ADR-0005) when the imported
    /// artifact carries one; `None` when unexported (the renderer
    /// renders "No provenance" rather than synthesizing an empty block).
    pub share_provenance: Option<Value>,
}

/// A Novice import held at the tier-gate review (M07.5 / ADR-0017 — the
/// install-after-confirm split).
///
/// The pipeline has already run fetch / validate / §15c / L3; the only
/// work left is the install half — MCP-registry upsert (`mcp_server`
/// only) + install + lock — which [`complete_import_with`] runs on the
/// renderer's confirm.
#[derive(Debug, Clone)]
pub struct PendingImport {
    art: ValidatedArtifact,
    src: ImportSource,
    report: L3Report,
    tier: Tier,
}

impl PendingImport {
    /// The `name@version` this import will lock under once completed.
    /// The renderer keys its review record by this — it is stable across
    /// the pending → installed transition.
    #[must_use]
    pub fn lock_key(&self) -> String {
        self.art.name_at_version()
    }
}

/// The outcome of [`import_artifact_with`] (M07.5 / ADR-0017 — the
/// validate/commit split that closes M07.V 🔴 #1).
///
/// - `Installed` — the tier-gate passed (Promoted, L4 auto-accept); the
///   artifact is installed + hash-locked inline.
/// - `Pending` — the tier-gate requires a Novice capability-disclosure
///   review (spec §8.security L4); NOTHING is installed or locked. The
///   renderer shows the review screen; [`complete_import_with`] finishes
///   the install on confirm, or the pending state is dropped on reject
///   (nothing to roll back — the install half never ran).
#[derive(Debug)]
pub enum ImportOutcome {
    /// Installed + hash-locked (Promoted / L4 auto-accept).
    Installed(Installed),
    /// Held for a Novice review — no install, no lock (the 🔴 #1 fix).
    Pending {
        /// The capability disclosure the review screen renders. Boxed
        /// to keep the enum's variant sizes balanced.
        review: Box<TierReview>,
        /// The in-flight import [`complete_import_with`] finishes.
        pending: PendingImport,
    },
}

/// A normalized MCP-server-config import handed to the [`McpRegistry`]
/// seam. Flat (no typify coupling) — mirrors the M06
/// `registry::McpServerRecord` wire columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpServerImport {
    /// MCP server name (`mcp.v1.json` `McpServerName`).
    pub name: String,
    /// `stdio` or `http`.
    pub transport: String,
    /// Stdio executable, when `transport == "stdio"`.
    pub command: Option<String>,
    /// Stdio args as a JSON-array string, when present.
    pub args_json: Option<String>,
    /// Stdio env as a JSON-object string, when present.
    pub env_json: Option<String>,
    /// Stdio working directory, when present.
    pub cwd: Option<String>,
    /// HTTP url, when `transport == "http"`.
    pub url: Option<String>,
    /// Per-server keychain key reference, when declared.
    pub auth_secret_ref: Option<String>,
}

/// Errors surfaced by the import pipeline. Each maps to a distinct
/// renderer phase (Stage E); `TierReviewRequired` is a review *outcome*,
/// not a failure-stop.
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    /// Fetch failed — bad URL, IO, network capability denied (Hard Rule
    /// 4), or HTTP error status.
    #[error("fetch failed: {0}")]
    Fetch(String),
    /// The bytes do not validate against the kind's schema (CLAUDE.md
    /// §14). Carries the validation report.
    #[error("schema invalid: {0}")]
    SchemaInvalid(String),
    /// L3 sandbox rejected the artifact — carries the per-syscall
    /// reasons.
    #[error("L3 sandbox rejected: {0:?}")]
    L3(Vec<String>),
    /// Novice tier — the renderer must show the capability-disclosure
    /// review screen before the install is accepted (spec §M7). The
    /// `TierReview` is boxed so this outcome variant does not bloat
    /// every `Result<_, ImportError>` in the module.
    #[error("tier-gated: review required before install")]
    TierReviewRequired(Box<TierReview>),
    /// §15c — the artifact does not support the host OS. BLOCKING (fail
    /// loudly, checked before L3).
    #[error("compatible_os mismatch: artifact {artifact:?} vs host {host}")]
    OsMismatch {
        /// The artifact's declared `compatible_os`.
        artifact: Vec<String>,
        /// The host OS that is not in it.
        host: String,
    },
    /// `skills.lock` write failed (M07.B `skills_lock`).
    #[error("skills.lock write failed: {0}")]
    Lock(String),
    /// MCP-server-config registry upsert failed (M06 MCP Manager).
    #[error("MCP registry import failed: {0}")]
    Registry(String),
}

/// Fetch transport seam — the real impl is [`fetch::HttpFetcher`];
/// tests inject a fake. One hop: a body, or a redirect for
/// [`fetch_with`] to re-validate (M07.5 / ADR-0018 — the SSRF-hardened
/// fetch path).
#[async_trait::async_trait]
pub trait Fetcher: Send + Sync {
    /// Issue one GET to the validated, DNS-pinned `target`.
    ///
    /// # Errors
    ///
    /// Any transport / HTTP-status / body-cap failure, stringified.
    async fn fetch_hop(&self, target: &egress::ValidatedTarget)
        -> Result<egress::FetchHop, String>;
}

/// L3 sandbox seam — the real impl wraps `sandbox_ipc::SandboxClient`
/// (reuse `runtime-sandbox`, M05); tests inject fakes.
#[async_trait::async_trait]
pub trait Sandbox: Send + Sync {
    /// Validate the artifact source. `Ok(reasons)` — an empty vec means
    /// the artifact cleared L3; a non-empty vec is a rejection.
    ///
    /// # Errors
    ///
    /// Transport / sandbox-IPC failure, stringified.
    async fn validate(&self, code: &str) -> Result<Vec<String>, String>;
}

/// MCP Manager registry seam (ADR-0010 dependency inversion — the
/// concrete adapter wrapping `runtime_mcp::Registry` is constructed in
/// the Tauri shell to avoid the `runtime-mcp → runtime-main` cycle).
pub trait McpRegistry: Send + Sync {
    /// Insert-or-replace the imported MCP server config.
    ///
    /// # Errors
    ///
    /// The registry failure, stringified.
    fn upsert(&self, cfg: &McpServerImport) -> Result<(), String>;
}

/// Time seam — the install timestamp. Real impl is [`SystemClock`];
/// tests inject a fixed clock for deterministic `installed_at`.
pub trait Clock: Send + Sync {
    /// The current UTC time.
    fn now(&self) -> DateTime<Utc>;
}

/// Production [`Clock`] — wall-clock UTC.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Fetch the artifact bytes.
///
/// File reads are local. URL fetches are SSRF-hardened (ADR-0018): each
/// hop — the initial URL and every redirect target — is re-validated
/// through [`egress::validate_egress`] (`https`-only, public-address-
/// only, DNS-pinned) before the GET. The redirect chain is bounded by
/// [`egress::MAX_REDIRECTS`].
///
/// # Errors
///
/// [`ImportError::Fetch`] for a missing file, an egress rejection, a
/// transport failure, or a redirect chain past the cap.
pub async fn fetch_with(
    src: &ImportSource,
    get: &dyn Fetcher,
    resolver: &dyn egress::Resolver,
) -> Result<Vec<u8>, ImportError> {
    match src {
        ImportSource::File(p) => std::fs::read(p).map_err(|e| ImportError::Fetch(e.to_string())),
        ImportSource::Url(u) => {
            let mut current = u.clone();
            for _ in 0..=egress::MAX_REDIRECTS {
                let target = egress::validate_egress(&current, resolver).await?;
                match get.fetch_hop(&target).await.map_err(ImportError::Fetch)? {
                    egress::FetchHop::Body(bytes) => return Ok(bytes),
                    egress::FetchHop::Redirect(next) => current = next,
                }
            }
            Err(ImportError::Fetch(format!(
                "too many redirects (max {})",
                egress::MAX_REDIRECTS
            )))
        }
    }
}

/// Read a `["a","b"]`-shaped value into a `Vec<String>`.
fn str_array(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Validate raw bytes by `kind` and extract identity + metadata.
///
/// The generated typify type IS the schema's enforced shape
/// (CLAUDE.md §14) for skill / tool / `mcp_server`; agent imports are
/// identity+metadata-gated here (the agent-graph schema is
/// `framework_loader`'s concern at load).
///
/// # Errors
///
/// [`ImportError::SchemaInvalid`] when the bytes are not JSON, fail the
/// kind's schema, or lack a `name`.
pub fn validate(kind: ArtifactKind, bytes: &[u8]) -> Result<ValidatedArtifact, ImportError> {
    let v: Value = serde_json::from_slice(bytes)
        .map_err(|e| ImportError::SchemaInvalid(format!("not valid JSON: {e}")))?;

    match kind {
        ArtifactKind::Skill => {
            serde_json::from_value::<runtime_core::generated::skill::Skill>(v.clone())
                .map_err(|e| ImportError::SchemaInvalid(format!("skill schema: {e}")))?;
        }
        ArtifactKind::Tool => {
            serde_json::from_value::<runtime_core::generated::tool::Tool>(v.clone())
                .map_err(|e| ImportError::SchemaInvalid(format!("tool schema: {e}")))?;
        }
        ArtifactKind::McpServer => {
            serde_json::from_value::<runtime_core::generated::mcp::McpServerConfig>(v.clone())
                .map_err(|e| ImportError::SchemaInvalid(format!("mcp schema: {e}")))?;
        }
        ArtifactKind::Agent => { /* identity+metadata gate below; agent
             graph validated at framework load */
        }
    }

    let name = v
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ImportError::SchemaInvalid("missing required `name`".to_string()))?
        .to_string();
    let version = v
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or("0.0.0")
        .to_string();

    let compatible_os = {
        let declared = str_array(v.get("compatible_os"));
        if declared.is_empty() {
            ALL_OS.iter().map(ToString::to_string).collect()
        } else {
            declared
        }
    };
    let meta = ArtifactMeta {
        requires_secrets: str_array(v.get("requires_secrets")),
        runtime_dependency_class: v
            .get("runtime_dependency_class")
            .and_then(Value::as_str)
            .unwrap_or("desktop_runtime")
            .to_string(),
        compatible_os,
        share_provenance: v.get("share_provenance").filter(|x| !x.is_null()).cloned(),
    };

    Ok(ValidatedArtifact {
        kind,
        name,
        version,
        bytes: bytes.to_vec(),
        meta,
        raw: v,
    })
}

/// A short capability-disclosure summary from the artifact's declared
/// `capabilities` (best-effort — the disclosure is human-facing).
fn capability_summary(raw: &Value) -> Vec<String> {
    let caps = raw.get("capabilities");
    let mut out = Vec::new();
    for key in ["tools_called", "network", "spawn_agents"] {
        for item in str_array(caps.and_then(|c| c.get(key))) {
            out.push(format!("{key}: {item}"));
        }
    }
    if caps
        .and_then(|c| c.get("shell"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        out.push("shell: true".to_string());
    }
    out
}

/// L4 tier gate (reuse the M05 [`Tier`]). Novice → the renderer must
/// show the capability-disclosure review (spec §M7); Promoted is the L4
/// pass-through (auto-accept) per the `tier` module semantics.
///
/// # Errors
///
/// [`ImportError::TierReviewRequired`] for the Novice tier.
pub fn tier_gate(a: &ValidatedArtifact, tier: Tier, report: &L3Report) -> Result<(), ImportError> {
    match tier {
        Tier::Promoted => Ok(()),
        Tier::Novice => Err(ImportError::TierReviewRequired(Box::new(TierReview {
            capabilities: capability_summary(&a.raw),
            l3_report: report.clone(),
            requires_secrets: a.meta.requires_secrets.clone(),
            share_provenance: a.meta.share_provenance.clone(),
        }))),
    }
}

/// The `skills_lock` wire string for a tier (`tier_at_install` enum).
const fn tier_wire(tier: Tier) -> &'static str {
    match tier {
        Tier::Novice => "novice",
        Tier::Promoted => "promoted",
    }
}

/// Build the M06 registry import from a validated MCP-server config
/// (extracted off the raw JSON — no typify `McpTransport` coupling,
/// mirroring the M07.B "pin the wire shape, not typify naming" rule).
fn mcp_import_of(a: &ValidatedArtifact) -> McpServerImport {
    let t = a.raw.get("transport");
    let transport = t
        .and_then(|x| x.get("type"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let s = |key: &str| {
        t.and_then(|x| x.get(key))
            .and_then(Value::as_str)
            .map(ToString::to_string)
    };
    let as_json = |key: &str| {
        t.and_then(|x| x.get(key))
            .filter(|x| !x.is_null())
            .map(ToString::to_string)
    };
    McpServerImport {
        name: a.name.clone(),
        transport,
        command: s("command"),
        args_json: as_json("args"),
        env_json: as_json("env"),
        cwd: s("cwd"),
        url: s("url"),
        auth_secret_ref: a
            .raw
            .get("auth_secret_ref")
            .and_then(Value::as_str)
            .map(ToString::to_string),
    }
}

/// Install a validated, L3-cleared artifact.
///
/// Writes the M07.B `skills.lock` entry (path-agnostic — the shell
/// resolves the framework-root path). The lock `source` is exactly B's
/// discriminated shape and `content_hash` is B's SRI hash over the
/// fetched bytes.
///
/// # Errors
///
/// [`ImportError::Lock`] when the lock entry cannot be built or written.
pub fn install_with(
    a: &ValidatedArtifact,
    src: &ImportSource,
    report: &L3Report,
    tier: Tier,
    lock: &Path,
    clock: &dyn Clock,
) -> Result<(), ImportError> {
    let source = match src {
        ImportSource::Url(u) => json!({ "type": "url", "url": u }),
        ImportSource::File(p) => json!({ "type": "file", "path": p.to_string_lossy() }),
    };
    let entry_json = json!({
        "kind": a.kind.wire(),
        "source": source,
        "content_hash": skills_lock::content_hash(&a.bytes),
        "installed_at": clock.now(),
        "tier_at_install": tier_wire(tier),
        "validation_report_id": report.report_id,
    });
    let entry = serde_json::from_value(entry_json)
        .map_err(|e| ImportError::Lock(format!("lock entry shape: {e}")))?;
    skills_lock::write_entry(lock, &a.name_at_version(), entry)
        .map_err(|e| ImportError::Lock(e.to_string()))
}

/// The install half of the import pipeline — install + `skills.lock`
/// write, then MCP-registry upsert (`mcp_server` only). Runs ONLY past a
/// passed tier-gate: inline for Promoted, or via [`complete_import_with`]
/// on a Novice confirm. It NEVER runs for an unconfirmed Novice review —
/// that is the M07.V 🔴 #1 fix.
///
/// The MCP-registry upsert is ordered AFTER the `skills.lock` write: the
/// lock is the ADR-0014 integrity source of truth, and the two stores
/// are not transactionally linked — if the upsert fails after the lock
/// is written, the artifact is locked-but-not-registered (a re-import
/// reconciles it), the more recoverable partial state than a registry
/// row with no lock entry.
fn commit_import(
    art: &ValidatedArtifact,
    src: &ImportSource,
    report: &L3Report,
    tier: Tier,
    reg: &dyn McpRegistry,
    lock: &Path,
    clock: &dyn Clock,
) -> Result<Installed, ImportError> {
    install_with(art, src, report, tier, lock, clock)?;
    if matches!(art.kind, ArtifactKind::McpServer) {
        reg.upsert(&mcp_import_of(art))
            .map_err(ImportError::Registry)?;
    }
    Ok(Installed {
        lock_key: art.name_at_version(),
        report: report.clone(),
        requires_secrets: art.meta.requires_secrets.clone(),
        capabilities: capability_summary(&art.raw),
        share_provenance: art.meta.share_provenance.clone(),
    })
}

/// The full import pipeline (the unit-tested seam the Tauri command
/// wraps): fetch → validate → §15c gate (BEFORE L3) → L3 → L4 tier-gate.
///
/// A passed tier-gate (Promoted, L4 auto-accept) installs + hash-locks
/// inline; a `TierReviewRequired` (Novice) holds the import as a
/// [`PendingImport`] and installs NOTHING — the renderer's confirm runs
/// the held install half via [`complete_import_with`] (M07.5 / ADR-0017,
/// closes M07.V 🔴 #1).
///
/// # Errors
///
/// Each stage's distinct [`ImportError`]. `TierReviewRequired` is NOT
/// surfaced as an error — it is folded into [`ImportOutcome::Pending`].
#[allow(clippy::too_many_arguments)]
pub async fn import_artifact_with(
    src: ImportSource,
    kind: ArtifactKind,
    tier: Tier,
    host_os: &str,
    lock: &Path,
    get: &dyn Fetcher,
    sb: &dyn Sandbox,
    reg: &dyn McpRegistry,
    resolver: &dyn egress::Resolver,
    clock: &dyn Clock,
) -> Result<ImportOutcome, ImportError> {
    let bytes = fetch_with(&src, get, resolver).await?;
    let art = validate(kind, &bytes)?;

    // §15c — BLOCKING, and BEFORE the expensive L3 (cheap reject).
    if !art.meta.compatible_os.iter().any(|o| o == host_os) {
        return Err(ImportError::OsMismatch {
            artifact: art.meta.compatible_os.clone(),
            host: host_os.to_string(),
        });
    }

    let code = String::from_utf8_lossy(&art.bytes);
    let reasons = sb
        .validate(&code)
        .await
        .map_err(|e| ImportError::L3(vec![e]))?;
    if !reasons.is_empty() {
        return Err(ImportError::L3(reasons));
    }
    let report = L3Report {
        report_id: uuid::Uuid::new_v4().to_string(),
        passed: true,
        reasons: Vec::new(),
    };

    // L4 tier-gate — the M07.5 install-after-confirm gate (closes
    // 🔴 #1). Promoted → Ok → install inline. Novice →
    // TierReviewRequired → HOLD: nothing is upserted or locked until
    // `complete_import_with` runs on the renderer's confirm.
    match tier_gate(&art, tier, &report) {
        Ok(()) => {
            let installed = commit_import(&art, &src, &report, tier, reg, lock, clock)?;
            Ok(ImportOutcome::Installed(installed))
        }
        Err(ImportError::TierReviewRequired(review)) => Ok(ImportOutcome::Pending {
            review,
            pending: PendingImport {
                art,
                src,
                report,
                tier,
            },
        }),
        // `tier_gate` yields only `Ok` or `TierReviewRequired`; this arm
        // keeps the match total without a panic.
        Err(other) => Err(other),
    }
}

/// Finish a Novice import the renderer confirmed at the tier-gate
/// review (the `complete_import_artifact` Tauri command).
///
/// Runs the install half — install + `skills.lock` write + MCP-registry
/// upsert — that [`import_artifact_with`] deliberately held back for the
/// review.
///
/// # Errors
///
/// [`ImportError::Lock`] / [`ImportError::Registry`] when the lock write
/// or the registry upsert fails.
pub fn complete_import_with(
    pending: &PendingImport,
    reg: &dyn McpRegistry,
    lock: &Path,
    clock: &dyn Clock,
) -> Result<Installed, ImportError> {
    commit_import(
        &pending.art,
        &pending.src,
        &pending.report,
        pending.tier,
        reg,
        lock,
        clock,
    )
}

/// Populate `share_provenance` on a framework value at export time
/// (ADR-0005).
///
/// v0.1 is runtime-to-runtime ONLY: `rebake_changes` is ALWAYS `[]`
/// (no Share It module, no rebake — the v1.0 layer attaches here).
/// `for_os` mirrors the framework's `compatible_os`.
pub fn export_with_provenance(framework: &mut Value, now: DateTime<Utc>) {
    let for_os = framework
        .get("compatible_os")
        .cloned()
        .unwrap_or_else(|| json!(ALL_OS));
    let for_runtime_class = framework
        .get("runtime_dependency_class")
        .and_then(Value::as_str)
        .unwrap_or("desktop_runtime")
        .to_string();
    framework["share_provenance"] = json!({
        "exported_at": now.to_rfc3339(),
        "exported_by": SHARE_IT_ID,
        "for_runtime_class": for_runtime_class,
        "for_os": for_os,
        "rebake_changes": [],
    });
}

/// Surface the `share_provenance` trust block on import (ADR-0005) —
/// `None` when the artifact was never exported (not a synthesized
/// empty block).
#[must_use]
pub fn read_share_provenance(artifact_json: &Value) -> Option<Value> {
    artifact_json
        .get("share_provenance")
        .filter(|x| !x.is_null())
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_clock_is_post_2024() {
        assert!(SystemClock.now() > Utc::now() - chrono::Duration::days(1));
    }

    #[test]
    fn artifact_kind_wire_matches_skills_lock_enum() {
        assert_eq!(ArtifactKind::Skill.wire(), "skill");
        assert_eq!(ArtifactKind::McpServer.wire(), "mcp_server");
        assert_eq!(tier_wire(Tier::Novice), "novice");
        assert_eq!(tier_wire(Tier::Promoted), "promoted");
    }
}
