//! Integration tests for the Builder backend — M08 Stage B.
//!
//! Covers all four `builder` surfaces against the real public API:
//! `validate_framework`, `framework_capability_summary`, `save_framework`
//! / `load_framework`, and `list_installed`. Filesystem-touching tests
//! use `tempfile`-backed paths (the path-agnostic archetype, CLAUDE.md
//! §9). Report structs carry no `PartialEq` (the embedded
//! `runtime_core` `CapabilityDeclaration` / `Source` derive only
//! `Clone` + `Debug`), so idempotency tests compare via
//! `serde_json::to_value`.

use std::path::Path;

use runtime_core::generated::capability::CapabilityKind;
use runtime_core::generated::framework::Framework;
use runtime_core::generated::skills_lock::{ArtifactKind, Source};
use runtime_main::builder::{
    framework_capability_summary, list_installed, load_framework, save_framework,
    validate_framework, BuilderError, Companion,
};
use serde_json::json;

// ── Fixtures ────────────────────────────────────────────────────────

/// Build a `capabilities` block JSON value.
fn caps(
    tools_called: &[&str],
    skills_loaded: &[&str],
    read: &[&str],
    write: &[&str],
    network: &[&str],
    shell: bool,
    spawn_agents: &[&str],
) -> serde_json::Value {
    json!({
        "tools_called": tools_called,
        "skills_loaded": skills_loaded,
        "file_access": { "read": read, "write": write },
        "network": network,
        "shell": shell,
        "spawn_agents": spawn_agents,
    })
}

/// An all-empty `capabilities` block.
fn empty_caps() -> serde_json::Value {
    caps(&[], &[], &[], &[], &[], false, &[])
}

/// Build an inline-agent JSON value.
fn agent_json(
    id: &str,
    capabilities: serde_json::Value,
    allowed_tools: &[&str],
    allowed_skills: &[&str],
    spawns: &[&str],
) -> serde_json::Value {
    let mut agent = json!({
        "id": id,
        "role": "worker",
        "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
        "allowed_tools": allowed_tools,
        "allowed_skills": allowed_skills,
        "spawns": spawns,
    });
    agent["capabilities"] = capabilities;
    agent
}

/// Build a framework JSON value from agents + declared tool/skill sets.
fn framework_json(
    agents: Vec<serde_json::Value>,
    tools: &[(&str, &str)],
    skills: &[(&str, &str)],
    root: &str,
) -> serde_json::Value {
    let tool_items: Vec<serde_json::Value> = tools
        .iter()
        .map(|(n, src)| json!({ "name": n, "source": src }))
        .collect();
    let skill_items: Vec<serde_json::Value> = skills
        .iter()
        .map(|(n, src)| json!({ "name": n, "source": src }))
        .collect();
    let mut framework = json!({
        "name": "test",
        "version": "1.0.0",
        "description": "test framework",
        "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
        "tools": tool_items,
        "skills": skill_items,
        "session_root_agent": root,
    });
    framework["agents"] = serde_json::Value::Array(agents);
    framework
}

/// Deserialize a framework JSON value into the typify-generated type.
fn framework(value: serde_json::Value) -> Framework {
    serde_json::from_value(value).expect("test framework deserializes")
}

/// A minimal valid framework: one agent with one resolved builtin tool.
fn valid_framework_value() -> serde_json::Value {
    framework_json(
        vec![agent_json("worker", empty_caps(), &["Read"], &[], &[])],
        &[("Read", "builtin")],
        &[],
        "worker",
    )
}

/// A valid framework carrying one Agent→Agent edge whose narrowing is
/// `Ok` (child grants ⊆ parent grants).
fn framework_with_ok_spawn_edge() -> serde_json::Value {
    let parent = agent_json(
        "parent",
        caps(&["Read"], &[], &[], &[], &[], false, &["child"]),
        &[],
        &[],
        &["child"],
    );
    let child = agent_json(
        "child",
        caps(&["Read"], &[], &[], &[], &[], false, &[]),
        &[],
        &[],
        &[],
    );
    framework_json(vec![parent, child], &[], &[], "parent")
}

/// Write a `skills.lock` fixture with the given `(key, kind, installed_at)`
/// entries. Every entry shares a valid SRI hash + file source.
fn write_lock(path: &Path, entries: &[(&str, &str, &str)]) {
    let mut installed = serde_json::Map::new();
    for (key, kind, installed_at) in entries {
        installed.insert(
            (*key).to_string(),
            json!({
                "content_hash": "sha256-AAAA",
                "installed_at": installed_at,
                "kind": kind,
                "source": { "type": "file", "path": "/imported/from/here.md" },
                "tier_at_install": "novice",
                "validation_report_id": "vr-1",
            }),
        );
    }
    let lock = json!({ "version": 1, "installed": serde_json::Value::Object(installed) });
    std::fs::write(
        path,
        serde_json::to_string_pretty(&lock).expect("lock serializes"),
    )
    .expect("lock fixture written");
}

// ── validate_framework (B.4.1) ──────────────────────────────────────

#[test]
fn validate_framework_valid_framework_reports_ok() {
    let report = validate_framework(&valid_framework_value());
    assert!(report.ok, "a valid framework validates clean");
    assert!(report.schema_errors.is_empty());
    assert!(report.capability_errors.is_empty());
    assert!(report.capability_summary.is_some());
}

#[test]
fn validate_framework_schema_invalid_reports_schema_error_keyed_to_json_path() {
    // A document that does not deserialize into `Framework`.
    let report = validate_framework(&json!({ "name": "incomplete" }));
    assert!(!report.ok);
    assert_eq!(report.schema_errors.len(), 1);
    assert_eq!(
        report.schema_errors[0].node_path, "(root)",
        "a shape failure is keyed to the document root",
    );
    assert!(
        !report.schema_errors[0].message.is_empty(),
        "the serde error message names the offending field",
    );
    assert!(report.capability_errors.is_empty());
}

#[test]
fn validate_framework_unresolved_tool_ref_reports_capability_error_keyed_to_agent() {
    let doc = framework_json(
        vec![agent_json(
            "worker",
            empty_caps(),
            &["MissingTool"],
            &[],
            &[],
        )],
        &[],
        &[],
        "worker",
    );
    let report = validate_framework(&doc);
    assert!(!report.ok);
    assert!(report.schema_errors.is_empty());
    assert_eq!(report.capability_errors.len(), 1);
    assert_eq!(report.capability_errors[0].node_path, "worker");
    assert!(report.capability_errors[0].message.contains("MissingTool"));
}

#[test]
fn validate_framework_unresolved_skill_ref_reports_capability_error() {
    let doc = framework_json(
        vec![agent_json(
            "worker",
            empty_caps(),
            &[],
            &["MissingSkill"],
            &[],
        )],
        &[],
        &[],
        "worker",
    );
    let report = validate_framework(&doc);
    assert!(!report.ok);
    assert_eq!(report.capability_errors.len(), 1);
    assert!(report.capability_errors[0].message.contains("MissingSkill"));
}

#[test]
fn validate_framework_unresolved_agent_ref_reports_capability_error() {
    let doc = framework_json(
        vec![agent_json("worker", empty_caps(), &[], &[], &["ghost"])],
        &[],
        &[],
        "worker",
    );
    let report = validate_framework(&doc);
    assert!(!report.ok);
    assert_eq!(report.capability_errors.len(), 1);
    assert_eq!(report.capability_errors[0].node_path, "worker");
    assert!(report.capability_errors[0].message.contains("ghost"));
}

#[test]
fn validate_framework_agent_to_agent_narrowing_violation_reports_capability_error_keyed_to_child() {
    // Parent declares no capabilities; child declares a tool grant the
    // parent does not hold — the B.3.2 step-4 fold makes the framework
    // invalid, keyed to the child agent.
    let parent = agent_json("worker", empty_caps(), &[], &[], &["child"]);
    let child = agent_json(
        "child",
        caps(&["Read"], &[], &[], &[], &[], false, &[]),
        &[],
        &[],
        &[],
    );
    let report = validate_framework(&framework_json(vec![parent, child], &[], &[], "worker"));
    assert!(!report.ok);
    assert!(report.schema_errors.is_empty());
    assert_eq!(report.capability_errors.len(), 1);
    assert_eq!(report.capability_errors[0].node_path, "child");
    assert!(report.capability_errors[0].message.contains("narrowing"));
}

#[test]
fn validate_framework_valid_agent_to_agent_edge_reports_ok_with_an_ok_narrowing_triple() {
    let report = validate_framework(&framework_with_ok_spawn_edge());
    assert!(report.ok);
    let summary = report
        .capability_summary
        .expect("summary rides on an ok report");
    assert_eq!(summary.spawn_edges.len(), 1);
    assert!(summary.spawn_edges[0].narrowed_caps.is_ok());
}

#[test]
fn validate_framework_report_serializes_to_json() {
    // The report crosses the Tauri IPC boundary — it must `Serialize`,
    // and `narrowed_caps` must serialize as a serde-tagged `Result`.
    let report = validate_framework(&framework_with_ok_spawn_edge());
    let v = serde_json::to_value(&report).expect("report serializes to JSON");
    assert_eq!(v["ok"], json!(true));
    assert!(v["schema_errors"].is_array());
    assert!(v["capability_errors"].is_array());
    assert!(v["capability_summary"].is_object());
    let narrowed = &v["capability_summary"]["spawn_edges"][0]["narrowed_caps"];
    assert!(
        narrowed.get("Ok").is_some(),
        "narrowed_caps serializes as a tagged Result: {narrowed}",
    );
}

#[test]
fn validate_framework_valid_framework_report_carries_capability_summary() {
    let doc = framework_json(
        vec![agent_json(
            "worker",
            caps(&[], &[], &["src/**"], &[], &[], false, &[]),
            &[],
            &[],
            &[],
        )],
        &[],
        &[],
        "worker",
    );
    let report = validate_framework(&doc);
    let summary = report
        .capability_summary
        .expect("the B.3.4 summary rides on the report");
    assert_eq!(summary.files_read, vec!["src/**".to_string()]);
}

#[test]
fn validate_framework_schema_invalid_report_has_no_capability_summary() {
    // The early-return path — no parsed Framework to summarize.
    let report = validate_framework(&json!({ "not": "a framework" }));
    assert!(!report.ok);
    assert!(!report.schema_errors.is_empty());
    assert!(
        report.capability_summary.is_none(),
        "no summary when schema validation fails",
    );
}

#[test]
fn validate_framework_called_twice_on_same_doc_returns_identical_report() {
    // Gotcha #69 — multi-call idempotency.
    let doc = framework_with_ok_spawn_edge();
    let first = validate_framework(&doc);
    let second = validate_framework(&doc);
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(&second).unwrap(),
        "validate_framework is deterministic across calls",
    );
}

// ── framework_capability_summary (B.4.2) ────────────────────────────

#[test]
fn summary_aggregates_file_read_globs_across_agents() {
    let a = agent_json(
        "agent-a",
        caps(&[], &[], &["src/**"], &[], &[], false, &[]),
        &[],
        &[],
        &[],
    );
    let b = agent_json(
        "agent-b",
        caps(&[], &[], &["docs/**", "src/**"], &[], &[], false, &[]),
        &[],
        &[],
        &[],
    );
    let fw = framework(framework_json(vec![a, b], &[], &[], "agent-a"));
    let summary = framework_capability_summary(&fw);
    assert_eq!(
        summary.files_read,
        vec!["docs/**".to_string(), "src/**".to_string()],
        "read globs are aggregated, de-duplicated, and sorted",
    );
}

#[test]
fn summary_aggregates_network_hosts_across_agents() {
    let a = agent_json(
        "agent-a",
        caps(&[], &[], &[], &[], &["api.example.com"], false, &[]),
        &[],
        &[],
        &[],
    );
    let b = agent_json(
        "agent-b",
        caps(&[], &[], &[], &[], &["cdn.example.com"], false, &[]),
        &[],
        &[],
        &[],
    );
    let fw = framework(framework_json(vec![a, b], &[], &[], "agent-a"));
    let summary = framework_capability_summary(&fw);
    assert_eq!(
        summary.network_hosts,
        vec!["api.example.com".to_string(), "cdn.example.com".to_string()],
    );
}

#[test]
fn summary_any_shell_true_when_any_agent_declares_shell() {
    let no_shell = agent_json("agent-a", empty_caps(), &[], &[], &[]);
    let with_shell = agent_json(
        "agent-b",
        caps(&[], &[], &[], &[], &[], true, &[]),
        &[],
        &[],
        &[],
    );
    let fw = framework(framework_json(
        vec![no_shell, with_shell],
        &[],
        &[],
        "agent-a",
    ));
    assert!(framework_capability_summary(&fw).any_shell);

    let none = framework(framework_json(
        vec![agent_json("solo", empty_caps(), &[], &[], &[])],
        &[],
        &[],
        "solo",
    ));
    assert!(
        !framework_capability_summary(&none).any_shell,
        "any_shell is false when no agent declares shell",
    );
}

#[test]
fn summary_spawn_edge_narrowing_ok_when_child_subset_of_parent() {
    let fw = framework(framework_with_ok_spawn_edge());
    let summary = framework_capability_summary(&fw);
    assert_eq!(summary.spawn_edges.len(), 1);
    let edge = &summary.spawn_edges[0];
    assert_eq!(edge.parent_id, "parent");
    assert_eq!(edge.child_id, "child");
    assert!(edge.narrowed_caps.is_ok());
}

#[test]
fn summary_spawn_edge_narrowing_err_when_child_exceeds_parent() {
    // Parent has no grants; child declares a tool grant — narrowing
    // fails (the red-badge case).
    let parent = agent_json("parent", empty_caps(), &[], &[], &["child"]);
    let child = agent_json(
        "child",
        caps(&["Read"], &[], &[], &[], &[], false, &[]),
        &[],
        &[],
        &[],
    );
    let fw = framework(framework_json(vec![parent, child], &[], &[], "parent"));
    let summary = framework_capability_summary(&fw);
    assert_eq!(summary.spawn_edges.len(), 1);
    assert!(
        summary.spawn_edges[0].narrowed_caps.is_err(),
        "an over-declaring child narrows to Err",
    );
}

#[test]
fn summary_spawn_edge_narrowing_triple_carries_parent_child_declared_and_narrowed() {
    // Gotcha #66 — the triple must carry ALL THREE of parent /
    // child_declared / narrowed, and `narrowed` must be the genuine
    // narrow() result, not merely a non-empty vec.
    let parent = agent_json(
        "parent",
        caps(&["Read", "Write"], &[], &[], &[], &[], false, &[]),
        &[],
        &[],
        &["child"],
    );
    let child = agent_json(
        "child",
        caps(&["Read"], &[], &[], &[], &[], false, &[]),
        &[],
        &[],
        &[],
    );
    let fw = framework(framework_json(vec![parent, child], &[], &[], "parent"));
    let summary = framework_capability_summary(&fw);
    assert_eq!(summary.spawn_edges.len(), 1);
    let edge = &summary.spawn_edges[0];
    assert_eq!(
        edge.parent_caps.len(),
        2,
        "the parent's two tool grants are carried verbatim",
    );
    assert_eq!(
        edge.child_declared_caps.len(),
        1,
        "the child's one declared grant is carried verbatim",
    );
    let narrowed = edge
        .narrowed_caps
        .as_ref()
        .expect("a subset child narrows OK");
    assert_eq!(
        narrowed.len(),
        1,
        "narrowed is the genuine narrow() result — all-or-nothing carries the child set",
    );
    assert_eq!(narrowed[0].kind, CapabilityKind::Exec);
    assert_eq!(narrowed[0].resource.as_str(), "Read");
}

#[test]
fn summary_no_spawn_edges_produces_empty_spawn_edges_list() {
    let fw = framework(framework_json(
        vec![agent_json("solo", empty_caps(), &[], &[], &[])],
        &[],
        &[],
        "solo",
    ));
    assert!(framework_capability_summary(&fw).spawn_edges.is_empty());
}

// ── save_framework / load_framework (B.4.3) ─────────────────────────

#[test]
fn save_framework_writes_framework_json_to_dir() {
    let dir = tempfile::tempdir().unwrap();
    let fw = framework(valid_framework_value());
    save_framework(dir.path(), &fw, &[]).expect("save succeeds");
    let written = dir.path().join("framework.json");
    assert!(written.exists(), "framework.json was written");
    let raw = std::fs::read_to_string(&written).unwrap();
    let _: Framework = serde_json::from_str(&raw).expect("written framework.json parses");
}

#[test]
fn save_framework_writes_one_companion_md_per_inline_artifact() {
    // ADR-0022 (M08.6 Stage C) — an inline agent re-splits to
    // `agents/<id>.md` (the archetype subdirectory layout), and the
    // `framework.json` agents[] entry is the {id,path} reference form.
    // Supersedes the M08-era flat `<name>.agent.md` companion
    // convention at the directory top level.
    let dir = tempfile::tempdir().unwrap();
    let fw = framework(valid_framework_value());
    save_framework(dir.path(), &fw, &[]).expect("save succeeds");
    // The inline `worker` agent re-splits to `agents/worker.md`.
    let worker_md = dir.path().join("agents/worker.md");
    assert!(
        worker_md.is_file(),
        "the inline agent re-splits to agents/<id>.md per ADR-0022",
    );
    let body = std::fs::read_to_string(&worker_md).expect("agents/worker.md readable");
    assert!(
        body.starts_with("---\n"),
        "the agent .md begins with the YAML frontmatter delimiter: {body}",
    );
    assert!(
        body.contains("id: worker"),
        "the YAML frontmatter carries the agent id: {body}",
    );
    // framework.json's agents[] entry is the {id,path} reference, not
    // inline — the re-split inverse of Stage B's variant flip.
    let raw = std::fs::read_to_string(dir.path().join("framework.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let agent = &json["agents"][0];
    assert_eq!(agent["id"], "worker");
    assert_eq!(agent["path"], "agents/worker.md");
    assert!(
        agent.get("role").is_none(),
        "the framework.json agents[] entry is the reference form: {agent}",
    );
}

#[test]
fn save_framework_to_a_path_that_is_a_file_returns_not_a_directory() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("i-am-a-file");
    std::fs::write(&file_path, "not a directory").unwrap();
    let fw = framework(valid_framework_value());
    let err = save_framework(&file_path, &fw, &[]).expect_err("a file target is rejected");
    assert!(matches!(err, BuilderError::NotADirectory(_)), "got {err:?}");
}

#[test]
fn load_framework_round_trips_a_saved_framework() {
    let dir = tempfile::tempdir().unwrap();
    let fw = framework(valid_framework_value());
    save_framework(dir.path(), &fw, &[]).unwrap();
    let loaded = load_framework(dir.path()).expect("load succeeds");
    assert_eq!(loaded.framework.name, fw.name);
    assert_eq!(loaded.framework.agents.len(), fw.agents.len());
}

#[test]
fn load_framework_recovers_companion_md_files() {
    // The M08-era flat-companion convention is preserved alongside the
    // ADR-0022 canonical-modular subdir layout (the loader's
    // `read_flat_companions` backward-compat path). The two flat
    // companions surface under their top-level file names; Stage C's
    // re-split additionally surfaces `agents/worker.md` for the inline
    // worker agent (the resolver writes it; the loader recovers it).
    let dir = tempfile::tempdir().unwrap();
    let fw = framework(valid_framework_value());
    let companions = vec![
        Companion {
            file_name: "alpha.skill.md".to_string(),
            body: "alpha body".to_string(),
        },
        Companion {
            file_name: "beta.agent.md".to_string(),
            body: "beta body".to_string(),
        },
    ];
    save_framework(dir.path(), &fw, &companions).unwrap();
    let loaded = load_framework(dir.path()).expect("load succeeds");
    // Flat top-level companions survive verbatim (backward compat).
    let alpha = loaded
        .companions
        .iter()
        .find(|c| c.file_name == "alpha.skill.md")
        .expect("alpha.skill.md surfaces as a flat companion");
    assert_eq!(alpha.body, "alpha body");
    let beta = loaded
        .companions
        .iter()
        .find(|c| c.file_name == "beta.agent.md")
        .expect("beta.agent.md surfaces as a flat companion");
    assert_eq!(beta.body, "beta body");
}

#[test]
fn save_load_save_cycle_is_byte_stable() {
    // MVP §M8 criterion 8 — a save→load→save cycle is byte-stable.
    let dir1 = tempfile::tempdir().unwrap();
    let fw = framework(valid_framework_value());
    let companions = vec![Companion {
        file_name: "x.tool.md".to_string(),
        body: "tool body\n".to_string(),
    }];
    save_framework(dir1.path(), &fw, &companions).unwrap();
    let json_bytes_1 = std::fs::read(dir1.path().join("framework.json")).unwrap();
    let companion_bytes_1 = std::fs::read(dir1.path().join("x.tool.md")).unwrap();

    let loaded = load_framework(dir1.path()).unwrap();

    let dir2 = tempfile::tempdir().unwrap();
    save_framework(dir2.path(), &loaded.framework, &loaded.companions).unwrap();
    let json_bytes_2 = std::fs::read(dir2.path().join("framework.json")).unwrap();
    let companion_bytes_2 = std::fs::read(dir2.path().join("x.tool.md")).unwrap();

    assert_eq!(
        json_bytes_1, json_bytes_2,
        "framework.json is byte-stable across a save→load→save cycle",
    );
    assert_eq!(
        companion_bytes_1, companion_bytes_2,
        "the companion file is byte-stable across the cycle",
    );
}

#[test]
fn load_framework_missing_framework_json_returns_io_error() {
    let dir = tempfile::tempdir().unwrap();
    let err = load_framework(dir.path()).expect_err("a missing framework.json errs");
    assert!(matches!(err, BuilderError::Io(_)), "got {err:?}");
}

#[test]
fn load_framework_corrupt_framework_json_returns_json_error() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("framework.json"), "{ not valid json").unwrap();
    let err = load_framework(dir.path()).expect_err("a corrupt framework.json errs");
    assert!(matches!(err, BuilderError::Json(_)), "got {err:?}");
}

// ── list_installed (B.4.4) ──────────────────────────────────────────

#[test]
fn list_installed_reads_entries_from_a_skills_lock_fixture() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("skills.lock");
    write_lock(
        &lock_path,
        &[("pdf-summarizer@1.0.0", "skill", "2026-05-21T12:00:00Z")],
    );
    let installed = list_installed(&lock_path).expect("read succeeds");
    assert_eq!(installed.len(), 1);
    assert_eq!(installed[0].key, "pdf-summarizer@1.0.0");
}

#[test]
fn list_installed_absent_lock_returns_empty_not_error() {
    // M07-IRL #6 contract — an absent lock is "nothing installed".
    let dir = tempfile::tempdir().unwrap();
    let installed =
        list_installed(&dir.path().join("skills.lock")).expect("an absent lock is not an error");
    assert!(installed.is_empty());
}

#[test]
fn list_installed_corrupt_lock_returns_lock_error() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("skills.lock");
    std::fs::write(&lock_path, "{ corrupt lock").unwrap();
    let err = list_installed(&lock_path).expect_err("a corrupt lock errs");
    assert!(matches!(err, BuilderError::Lock(_)), "got {err:?}");
}

#[test]
fn list_installed_returns_entries_sorted_by_key() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("skills.lock");
    write_lock(
        &lock_path,
        &[
            ("zeta@1.0.0", "skill", "2026-05-21T12:00:00Z"),
            ("alpha@1.0.0", "tool", "2026-05-21T12:00:00Z"),
            ("mu@1.0.0", "agent", "2026-05-21T12:00:00Z"),
        ],
    );
    let installed = list_installed(&lock_path).unwrap();
    let keys: Vec<&str> = installed.iter().map(|a| a.key.as_str()).collect();
    assert_eq!(
        keys,
        vec!["alpha@1.0.0", "mu@1.0.0", "zeta@1.0.0"],
        "the HashMap-backed lock is sorted for a stable Palette ordering",
    );
}

#[test]
fn list_installed_flattens_kind_source_and_installed_at_from_lock_entry() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("skills.lock");
    write_lock(
        &lock_path,
        &[("fetch-tool@2.0.0", "tool", "2026-05-21T12:00:00Z")],
    );
    let installed = list_installed(&lock_path).unwrap();
    assert_eq!(installed.len(), 1);
    let entry = &installed[0];
    assert_eq!(entry.key, "fetch-tool@2.0.0");
    assert_eq!(entry.kind, ArtifactKind::Tool);
    assert_eq!(
        entry.installed_at, "2026-05-21T12:00:00+00:00",
        "installed_at is the RFC-3339 rendering of the lock entry timestamp",
    );
    match &entry.source {
        Source::File { path } => assert_eq!(path.as_str(), "/imported/from/here.md"),
        Source::Url { .. } => panic!("expected a File source, got a Url source"),
    }
}

#[test]
fn list_installed_called_twice_returns_identical_list() {
    // Gotcha #69 — multi-call idempotency.
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("skills.lock");
    write_lock(
        &lock_path,
        &[
            ("a@1.0.0", "skill", "2026-05-21T12:00:00Z"),
            ("b@1.0.0", "tool", "2026-05-21T12:00:00Z"),
        ],
    );
    let first = list_installed(&lock_path).unwrap();
    let second = list_installed(&lock_path).unwrap();
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(&second).unwrap(),
        "list_installed is deterministic across calls",
    );
}
