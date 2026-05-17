//! Integration smoke tests for the M05 Stage E audit log.
//!
//! Exercises the three wiring seams end-to-end:
//! - Framework loader (Stage A) — `framework_loaded` + `gap_detected`
//! - Capability enforcer (Stage B) — `capability_granted` + `capability_denied`
//! - Tier transition (Stage D + Stage E) — `tier_transition`
//!
//! Each test wires a real [`runtime_main::audit::AuditWriter`] backed
//! by a `tempfile::TempDir` directory, exercises the seam, then reads
//! the audit file and asserts the per-kind line is present + parses.

use std::str::FromStr;
use std::sync::Arc;

use runtime_core::event::AgentEvent;
use runtime_core::generated::capability::{
    CapabilityDeclaration, CapabilityKind, CapabilityScope, DomainPattern, GlobPattern,
    ResourceName, SideEffectClass,
};
use runtime_main::audit::AuditWriter;
use runtime_main::capability::CapabilityEnforcer;
use runtime_main::framework_loader::{load_and_validate_str_with_audit, AuditContext, Emitter};
use runtime_main::tier::{transition, Tier};
use tempfile::TempDir;
use tokio::sync::Mutex;

fn read_glob(resource: &str, glob: &str) -> CapabilityDeclaration {
    CapabilityDeclaration {
        kind: CapabilityKind::Read,
        resource: ResourceName::from_str(resource).unwrap(),
        scope: CapabilityScope::Glob(GlobPattern::from_str(glob).unwrap()),
        side_effect_class: SideEffectClass::Pure,
    }
}

fn network_domain(resource: &str, domain: &str) -> CapabilityDeclaration {
    CapabilityDeclaration {
        kind: CapabilityKind::Network,
        resource: ResourceName::from_str(resource).unwrap(),
        scope: CapabilityScope::Domain(DomainPattern::from_str(domain).unwrap()),
        side_effect_class: SideEffectClass::NetworkEgress,
    }
}

async fn open_audit(dir: &TempDir) -> Arc<AuditWriter> {
    let path = dir.path().join("skills.audit.jsonl");
    Arc::new(AuditWriter::open(&path).await.expect("open audit"))
}

async fn read_lines(dir: &TempDir) -> Vec<serde_json::Value> {
    let raw = tokio::fs::read_to_string(dir.path().join("skills.audit.jsonl"))
        .await
        .expect("read audit");
    raw.lines()
        .map(|l| serde_json::from_str(l).expect("each line parses"))
        .collect()
}

#[derive(Default)]
struct CollectingEmitter {
    events: Mutex<Vec<AgentEvent>>,
}

#[async_trait::async_trait]
impl Emitter for CollectingEmitter {
    async fn emit(&self, event: AgentEvent) {
        self.events.lock().await.push(event);
    }
}

#[tokio::test]
async fn capability_grant_writes_audit_line() {
    let dir = tempfile::tempdir().unwrap();
    let writer = open_audit(&dir).await;
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_audit_writer(Arc::clone(&writer), "sess-grant");
    let cap = read_glob("src", "src/**");
    enforcer.grant("worker", cap.clone());
    enforcer.audit_grant("worker", &cap).await;
    let lines = read_lines(&dir).await;
    assert_eq!(lines.len(), 1, "exactly one audit line expected");
    assert_eq!(lines[0]["kind"], "capability_granted");
    assert_eq!(lines[0]["session_id"], "sess-grant");
    assert_eq!(lines[0]["details"]["agent_id"], "worker");
    assert_eq!(lines[0]["details"]["capability_kind"], "read");
    assert_eq!(lines[0]["details"]["resource"], "src");
}

#[tokio::test]
async fn capability_denial_writes_audit_line() {
    let dir = tempfile::tempdir().unwrap();
    let writer = open_audit(&dir).await;
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_audit_writer(Arc::clone(&writer), "sess-deny");
    // No grants; Read passes L4 (Novice default) but L1 default-denies.
    let req = read_glob("src", "src/**");
    let result = enforcer.check("worker", &req);
    enforcer.audit_check_result("worker", &req, &result).await;
    let lines = read_lines(&dir).await;
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["kind"], "capability_denied");
    assert_eq!(lines[0]["details"]["agent_id"], "worker");
    assert_eq!(lines[0]["details"]["capability_kind"], "read");
    assert_eq!(lines[0]["details"]["reason"], "no_declarations");
}

#[tokio::test]
async fn capability_check_ok_writes_no_audit_line() {
    // Hot path: successful checks must NOT emit audit lines per phase
    // doc E.1 (audit emits on the cold rejection paths only).
    let dir = tempfile::tempdir().unwrap();
    let writer = open_audit(&dir).await;
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_audit_writer(Arc::clone(&writer), "sess-ok");
    let cap = read_glob("src", "src/**");
    enforcer.grant("worker", cap.clone());
    let result = enforcer.check("worker", &cap);
    assert!(result.is_ok());
    enforcer.audit_check_result("worker", &cap, &result).await;
    let raw = tokio::fs::read_to_string(dir.path().join("skills.audit.jsonl"))
        .await
        .unwrap();
    assert!(raw.is_empty(), "Ok check must not emit audit line");
}

#[tokio::test]
async fn tier_transition_writes_audit_line() {
    let dir = tempfile::tempdir().unwrap();
    let writer = open_audit(&dir).await;
    let record = transition(
        Some(&writer),
        "sess-tier",
        Tier::Novice,
        Tier::Promoted,
        "user accepted Settings prompt",
    )
    .await;
    assert!(record.changed);
    let lines = read_lines(&dir).await;
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["kind"], "tier_transition");
    assert_eq!(lines[0]["session_id"], "sess-tier");
    assert_eq!(lines[0]["details"]["previous"], "novice");
    assert_eq!(lines[0]["details"]["current"], "promoted");
}

#[tokio::test]
async fn framework_load_success_writes_framework_loaded_audit_line() {
    let dir = tempfile::tempdir().unwrap();
    let writer = open_audit(&dir).await;
    let audit_ctx = AuditContext {
        writer: Some(Arc::clone(&writer)),
        session_id: "sess-fw".to_string(),
    };
    let json = serde_json::to_string(&serde_json::json!({
        "name": "test_audit",
        "version": "1.0.0",
        "description": "framework loaded successfully — audit smoke",
        "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
        "agents": [{
            "id": "worker",
            "role": "worker",
            "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
            "capabilities": {
                "file_access": { "read": [], "write": [] },
                "network": [], "shell": false,
                "skills_loaded": [], "spawn_agents": [], "tools_called": []
            },
            "allowed_tools": ["Read"],
            "allowed_skills": [],
            "spawns": []
        }],
        "tools": [{ "name": "Read", "source": "builtin" }],
        "skills": [],
        "session_root_agent": "worker"
    }))
    .unwrap();
    let emitter = CollectingEmitter::default();
    let result = load_and_validate_str_with_audit(&json, &emitter, &audit_ctx).await;
    assert!(result.is_ok());
    let lines = read_lines(&dir).await;
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["kind"], "framework_loaded");
    assert_eq!(lines[0]["details"]["framework_name"], "test_audit");
    assert_eq!(lines[0]["details"]["agent_count"], 1);
}

#[tokio::test]
async fn framework_load_with_gaps_writes_gap_detected_audit_lines() {
    let dir = tempfile::tempdir().unwrap();
    let writer = open_audit(&dir).await;
    let audit_ctx = AuditContext {
        writer: Some(Arc::clone(&writer)),
        session_id: "sess-gap".to_string(),
    };
    // Framework with two unresolved tool refs — walker emits two gaps;
    // audit writes two `gap_detected` lines + NO framework_loaded line.
    let json = serde_json::to_string(&serde_json::json!({
        "name": "gappy",
        "version": "1.0.0",
        "description": "framework with two missing tool refs",
        "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
        "agents": [{
            "id": "worker",
            "role": "worker",
            "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
            "capabilities": {
                "file_access": { "read": [], "write": [] },
                "network": [], "shell": false,
                "skills_loaded": [], "spawn_agents": [], "tools_called": []
            },
            "allowed_tools": ["MissingA", "MissingB"],
            "allowed_skills": [],
            "spawns": []
        }],
        "tools": [],
        "skills": [],
        "session_root_agent": "worker"
    }))
    .unwrap();
    let emitter = CollectingEmitter::default();
    let result = load_and_validate_str_with_audit(&json, &emitter, &audit_ctx).await;
    assert!(result.is_err());
    let lines = read_lines(&dir).await;
    assert_eq!(lines.len(), 2, "two gaps must produce two audit lines");
    for line in &lines {
        assert_eq!(line["kind"], "gap_detected");
        assert_eq!(line["details"]["gap_kind"], "tool_missing");
        assert_eq!(line["details"]["agent_id"], "worker");
        assert_eq!(line["details"]["requested_via"], "loader");
    }
    let names: Vec<&str> = lines
        .iter()
        .map(|l| l["details"]["missing_name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"MissingA"));
    assert!(names.contains(&"MissingB"));
}

#[tokio::test]
async fn all_seams_together_produce_complete_audit_trail() {
    // End-to-end: framework load + tier transition + grant + denial all
    // share one audit log. Verifies the writer multiplexing across
    // call sites preserves order + each entry parses.
    let dir = tempfile::tempdir().unwrap();
    let writer = open_audit(&dir).await;
    let audit_ctx = AuditContext {
        writer: Some(Arc::clone(&writer)),
        session_id: "sess-all".to_string(),
    };
    // 1) Framework load
    let json = serde_json::to_string(&serde_json::json!({
        "name": "test_all",
        "version": "1.0.0",
        "description": "end-to-end smoke",
        "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
        "agents": [{
            "id": "worker",
            "role": "worker",
            "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
            "capabilities": {
                "file_access": { "read": [], "write": [] },
                "network": [], "shell": false,
                "skills_loaded": [], "spawn_agents": [], "tools_called": []
            },
            "allowed_tools": ["Read"],
            "allowed_skills": [],
            "spawns": []
        }],
        "tools": [{ "name": "Read", "source": "builtin" }],
        "skills": [],
        "session_root_agent": "worker"
    }))
    .unwrap();
    let emitter = CollectingEmitter::default();
    load_and_validate_str_with_audit(&json, &emitter, &audit_ctx)
        .await
        .unwrap();
    // 2) Tier promotion
    transition(
        Some(&writer),
        "sess-all",
        Tier::Novice,
        Tier::Promoted,
        "promoted",
    )
    .await;
    // 3) Capability grant + audit
    let mut enforcer = CapabilityEnforcer::new();
    enforcer.set_audit_writer(Arc::clone(&writer), "sess-all");
    let cap = network_domain("api", "api.example.com");
    enforcer.grant("worker", cap.clone());
    enforcer.audit_grant("worker", &cap).await;
    // 4) Capability denial
    let req = read_glob("docs", "docs/**");
    let result = enforcer.check("intruder", &req);
    enforcer.audit_check_result("intruder", &req, &result).await;
    let lines = read_lines(&dir).await;
    assert_eq!(lines.len(), 4, "four events expected in this scenario");
    let kinds: Vec<&str> = lines.iter().map(|l| l["kind"].as_str().unwrap()).collect();
    assert_eq!(
        kinds,
        vec![
            "framework_loaded",
            "tier_transition",
            "capability_granted",
            "capability_denied",
        ]
    );
}

// ── M06.C — mcp_installed / mcp_uninstalled / mcp_auth_granted entry tests ──

#[tokio::test]
async fn mcp_installed_entry_writes_correlated_audit_line() {
    let dir = tempfile::tempdir().unwrap();
    let writer = open_audit(&dir).await;
    let entry = runtime_main::audit::mcp_installed("sess-mcp", "github", "stdio", true);
    writer.log(&entry).await.expect("log");
    let lines = read_lines(&dir).await;
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["kind"], "mcp_installed");
    assert_eq!(lines[0]["session_id"], "sess-mcp");
    assert_eq!(lines[0]["details"]["name"], "github");
    assert_eq!(lines[0]["details"]["transport_kind"], "stdio");
    assert_eq!(lines[0]["details"]["has_auth"], true);
}

#[tokio::test]
async fn mcp_uninstalled_entry_writes_audit_line_with_name_only() {
    let dir = tempfile::tempdir().unwrap();
    let writer = open_audit(&dir).await;
    let entry = runtime_main::audit::mcp_uninstalled("sess-mcp", "vault");
    writer.log(&entry).await.expect("log");
    let lines = read_lines(&dir).await;
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["kind"], "mcp_uninstalled");
    assert_eq!(lines[0]["details"]["name"], "vault");
    // Per spec §13.5 zero-secret-logging: uninstall details carry name only.
    assert!(
        lines[0]["details"].get("auth_secret_ref").is_none(),
        "auth_secret_ref must NOT appear in the uninstall details"
    );
}

#[tokio::test]
async fn mcp_auth_granted_entry_does_not_log_secret_value() {
    // Per spec §13.5: the secret value is never logged. The audit line
    // carries server name only — no secret material in `details`.
    let dir = tempfile::tempdir().unwrap();
    let writer = open_audit(&dir).await;
    let entry = runtime_main::audit::mcp_auth_granted("sess-mcp", "kafka");
    writer.log(&entry).await.expect("log");
    let lines = read_lines(&dir).await;
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["kind"], "mcp_auth_granted");
    assert_eq!(lines[0]["details"]["name"], "kafka");
    let raw = serde_json::to_string(&lines[0]).unwrap();
    assert!(
        !raw.contains("secret"),
        "secret-value substrings must not appear anywhere in audit line: {raw}"
    );
}

#[tokio::test]
async fn enforcer_without_audit_writer_silently_skips_emission() {
    // Audit availability is not a dispatch gate — when no writer is
    // wired, grants + check_result emissions are silent no-ops. The
    // file is never created.
    let dir = tempfile::tempdir().unwrap();
    let mut enforcer = CapabilityEnforcer::new();
    let cap = read_glob("src", "src/**");
    enforcer.grant("worker", cap.clone());
    enforcer.audit_grant("worker", &cap).await;
    let result = enforcer.check("worker", &cap);
    enforcer.audit_check_result("worker", &cap, &result).await;
    let audit_path = dir.path().join("skills.audit.jsonl");
    assert!(
        !audit_path.exists(),
        "no writer wired → no audit file created"
    );
}
