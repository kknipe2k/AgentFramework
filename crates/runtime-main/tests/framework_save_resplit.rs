//! M08.6 Stage C — `save_framework` re-splits to the modular form
//! (ADR-0022, the write half).
//!
//! Discriminator (gotcha #66): the round-trip regressions drive the
//! REAL `examples/aria/` archetype through `save_framework` + Stage B's
//! `load_framework`. The byte-stability and round-trip assertions
//! exercise the actual modular archetype, not a hand-built inline
//! fixture (the M08-Builder blind spot Stage B closed for the read
//! half; Stage C closes it for the write half).

use std::path::PathBuf;

use runtime_main::builder::{load_framework, save_framework};

/// `crates/runtime-main/` → workspace root.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR has a parent (crates/)")
        .parent()
        .expect("crates/ has a parent (workspace root)")
        .to_path_buf()
}

fn aria_dir() -> PathBuf {
    workspace_root().join("examples/aria")
}

// ── C.4 — the four save-re-split regressions ────────────────────────

#[test]
fn save_then_load_round_trips_a_modular_framework() {
    // Load examples/aria/ (Stage B resolves it), save to a tempdir,
    // load back; assert the re-loaded resolved framework equals the
    // original resolved framework via JSON-value comparison. Fails on
    // `main` today — save writes the flat inline format and the
    // re-load cannot recover the modular structure.
    let original = load_framework(&aria_dir()).expect("examples/aria/ loads");
    let dir = tempfile::tempdir().expect("temp dir");
    save_framework(dir.path(), &original.framework, &original.companions)
        .expect("save succeeds on a resolved modular framework");
    let reloaded = load_framework(dir.path()).expect("re-load of the saved framework succeeds");
    assert_eq!(
        serde_json::to_value(&original.framework).expect("original framework serializes"),
        serde_json::to_value(&reloaded.framework).expect("reloaded framework serializes"),
        "the re-loaded resolved framework equals the original (via JSON-value comparison)",
    );
}

#[test]
fn save_writes_the_modular_subdirectory_layout() {
    // ADR-0022 canonical on-disk layout: agents/<id>.md +
    // tools/<name>.md + skills/<name>.md subdirectories, plus a
    // framework.json whose agents[] are {id,path} references (NOT
    // inline). Fails on `main` today — save writes the flat M08-era
    // companion convention and inline agents in framework.json.
    let original = load_framework(&aria_dir()).expect("examples/aria/ loads");
    let dir = tempfile::tempdir().expect("temp dir");
    save_framework(dir.path(), &original.framework, &original.companions).expect("save succeeds");
    // The three subdirectories carry the resolved `.md` files.
    assert!(
        dir.path().join("agents/orchestrator.md").is_file(),
        "agents/orchestrator.md written under the agents/ subdirectory",
    );
    assert!(
        dir.path().join("tools/git_checkpoint.md").is_file(),
        "tools/git_checkpoint.md written under the tools/ subdirectory",
    );
    assert!(
        dir.path().join("skills/planning.md").is_file(),
        "skills/planning.md written under the skills/ subdirectory",
    );
    // framework.json's agents[] entries are {id,path} references.
    let raw =
        std::fs::read_to_string(dir.path().join("framework.json")).expect("framework.json written");
    let json: serde_json::Value =
        serde_json::from_str(&raw).expect("written framework.json parses");
    let agents = json["agents"].as_array().expect("agents[] is an array");
    assert_eq!(agents.len(), 8, "all eight ARIA agents are emitted");
    for (i, agent) in agents.iter().enumerate() {
        assert!(
            agent
                .get("path")
                .and_then(serde_json::Value::as_str)
                .is_some(),
            "agents[{i}] is a {{id,path}} reference: {agent}",
        );
        assert!(
            agent.get("role").is_none(),
            "agents[{i}] carries no inline `role` — it is the reference form: {agent}",
        );
    }
}

#[test]
fn save_load_save_is_byte_stable_on_a_modular_framework() {
    // MVP §M8 criterion 8 — save→load→save is byte-stable on a real
    // modular framework. The existing line-590 byte-stability test in
    // builder.rs exercises only the inline-only path (no path-ref
    // round-trip); this one drives the archetype. Fails on `main`
    // today.
    let original = load_framework(&aria_dir()).expect("examples/aria/ loads");
    let dir1 = tempfile::tempdir().expect("temp dir 1");
    save_framework(dir1.path(), &original.framework, &original.companions)
        .expect("save 1 succeeds");
    let json_bytes_1 =
        std::fs::read(dir1.path().join("framework.json")).expect("framework.json from save 1");
    let orchestrator_bytes_1 = std::fs::read(dir1.path().join("agents/orchestrator.md"))
        .expect("agents/orchestrator.md from save 1");
    let planning_bytes_1 = std::fs::read(dir1.path().join("skills/planning.md"))
        .expect("skills/planning.md from save 1");

    let reloaded = load_framework(dir1.path()).expect("re-load succeeds");
    let dir2 = tempfile::tempdir().expect("temp dir 2");
    save_framework(dir2.path(), &reloaded.framework, &reloaded.companions)
        .expect("save 2 succeeds");
    let json_bytes_2 =
        std::fs::read(dir2.path().join("framework.json")).expect("framework.json from save 2");
    let orchestrator_bytes_2 = std::fs::read(dir2.path().join("agents/orchestrator.md"))
        .expect("agents/orchestrator.md from save 2");
    let planning_bytes_2 = std::fs::read(dir2.path().join("skills/planning.md"))
        .expect("skills/planning.md from save 2");

    assert_eq!(
        json_bytes_1, json_bytes_2,
        "framework.json is byte-stable across save→load→save on the archetype",
    );
    assert_eq!(
        orchestrator_bytes_1, orchestrator_bytes_2,
        "agents/orchestrator.md is byte-stable across save→load→save",
    );
    assert_eq!(
        planning_bytes_1, planning_bytes_2,
        "skills/planning.md is byte-stable across save→load→save",
    );
}

#[test]
fn save_round_trips_the_agent_md_body() {
    // The agent `.md` body carries the system-prompt content per
    // `agent.v1.json` (Stage B captures, M09 applies). A save→load
    // round-trip MUST preserve the body verbatim — orchestrator.md's
    // narrative ("# Orchestrator", "You are the root agent ...") is
    // load-bearing for M09's prompt-application path. Fails on `main`
    // today — the flat save discards the modular layout entirely, so a
    // round-trip on a path-ref framework has no path to carry the
    // body content.
    let original = load_framework(&aria_dir()).expect("examples/aria/ loads");
    let orchestrator_before = original
        .companions
        .iter()
        .find(|c| c.file_name == "agents/orchestrator.md")
        .expect("agents/orchestrator.md surfaces in the original load");
    assert!(
        orchestrator_before.body.contains("# Orchestrator"),
        "the source archetype's orchestrator.md body contains its narrative header",
    );

    let dir = tempfile::tempdir().expect("temp dir");
    save_framework(dir.path(), &original.framework, &original.companions).expect("save succeeds");
    let reloaded = load_framework(dir.path()).expect("re-load succeeds");
    let orchestrator_after = reloaded
        .companions
        .iter()
        .find(|c| c.file_name == "agents/orchestrator.md")
        .expect("agents/orchestrator.md surfaces in the reloaded companions");
    assert!(
        orchestrator_after.body.contains("# Orchestrator"),
        "the orchestrator.md narrative header survives the round-trip",
    );
    assert!(
        orchestrator_after.body.contains("You are the root agent"),
        "the orchestrator.md system-prompt body survives the round-trip",
    );
    assert_eq!(
        orchestrator_before.body, orchestrator_after.body,
        "the agent .md body is byte-identical pre/post save→load",
    );
}
