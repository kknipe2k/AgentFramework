//! xtask: project-wide build/maintenance tasks.
//!
//! Subcommands:
//!   regenerate-types         — run typify against schemas/*.v1.json
//!                              (Rust types in crates/runtime-core/src/generated/)
//!                              AND `npx json-schema-to-typescript` against the
//!                              TS-target schemas (TypeScript types in src/types/).
//!   regenerate-types --check — regenerate both Rust + TS to compare against
//!                              committed; non-zero exit on drift in either.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "xtask", about = "project build tasks")]
struct Args {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Regenerate Rust types from JSON schemas via typify.
    RegenerateTypes {
        /// Verify committed types match regenerated; exit non-zero on drift.
        #[arg(long)]
        check: bool,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Cmd::RegenerateTypes { check } => regenerate_types(check),
    }
}

fn regenerate_types(check: bool) -> Result<()> {
    use std::fs;
    let workspace_root = workspace_root()?;
    let schemas_dir = workspace_root.join("schemas");
    let target_dir = workspace_root.join("crates/runtime-core/src/generated");

    let schemas = [
        "common",
        "framework",
        "skill",
        "tool",
        "agent",
        "event",
        "error",
        "plan",
        "task",
        "hitl",
    ];
    let mut all_drift = Vec::new();

    for name in schemas {
        let schema_path = schemas_dir.join(format!("{name}.v1.json"));
        let generated = generate_one(&schema_path, name, &schemas_dir)?;

        let target_path = target_dir.join(format!("{name}.rs"));

        if check {
            let committed = fs::read_to_string(&target_path)
                .with_context(|| format!("read committed: {}", target_path.display()))?;
            if committed != generated {
                all_drift.push(name.to_string());
            }
        } else {
            fs::write(&target_path, &generated)
                .with_context(|| format!("write {}", target_path.display()))?;
        }
    }

    // TypeScript codegen — runs alongside the Rust pipeline so a single
    // `--check` command catches drift in either output. Per CLAUDE.md §14
    // schemas-as-source-of-truth: `src/types/agent_event.ts` is owned by
    // this step (was hand-mirrored at M02; M02-summary Decisions §"M03
    // prompt should add event.v1.json schema + cargo xtask regenerate-types
    // for TS types").
    let ts_output_dir = workspace_root.join("src/types");
    let ts_targets: Vec<(&str, std::path::PathBuf)> = vec![
        ("agent_event", schemas_dir.join("event.v1.json")),
        ("error", schemas_dir.join("error.v1.json")),
        ("plan", schemas_dir.join("plan.v1.json")),
        ("task", schemas_dir.join("task.v1.json")),
        ("hitl", schemas_dir.join("hitl.v1.json")),
    ];
    let ts_targets_refs: Vec<(&str, &std::path::Path)> = ts_targets
        .iter()
        .map(|(name, p)| (*name, p.as_path()))
        .collect();
    let ts_drift = regenerate_typescript_types_with(
        &ts_targets_refs,
        &ts_output_dir,
        run_npx_json_schema_to_typescript,
        check,
    )?;
    for name in ts_drift {
        all_drift.push(format!("{name}.ts"));
    }

    if check && !all_drift.is_empty() {
        anyhow::bail!(
            "type generation drift in: {}\nrun `cargo xtask regenerate-types` and commit the result",
            all_drift.join(", ")
        );
    }
    Ok(())
}

/// Test-seam for the TypeScript codegen path.
///
/// Iterates `schemas`, calls `runner(schema_path)` to produce the raw TS,
/// prepends a deterministic auto-generated header banner, and writes (or
/// drift-checks) `output_dir/<name>.ts` for each entry. Production wires
/// `runner = run_npx_json_schema_to_typescript`; tests inject a stub runner
/// that returns deterministic strings without crossing the npx subprocess
/// boundary.
///
/// The header banner is byte-stable across re-runs (no timestamps, no
/// incrementing counters) so `cargo xtask regenerate-types --check` produces
/// zero diff on PR-merged state.
///
/// # Errors
///
/// - Returns an `Err` if `runner` errors for any schema.
/// - Returns an `Err` if reading the committed file in `--check` mode fails.
/// - Returns an `Err` if writing the regenerated file fails.
///
/// In `--check` mode, drift is reported via the returned `Vec<String>`
/// (caller decides whether to bail), not via `Err`.
fn regenerate_typescript_types_with<R>(
    schemas: &[(&str, &std::path::Path)],
    output_dir: &std::path::Path,
    runner: R,
    check: bool,
) -> Result<Vec<String>>
where
    R: Fn(&std::path::Path) -> Result<String>,
{
    use std::fs;
    let mut drift = Vec::new();
    for (name, schema_path) in schemas {
        let body = runner(schema_path)
            .with_context(|| format!("ts codegen for {}", schema_path.display()))?;
        let schema_basename = schema_path.file_name().map_or_else(
            || format!("{name}.v1.json"),
            |s| s.to_string_lossy().into_owned(),
        );
        let header = ts_header(&schema_basename);
        let generated = format!("{header}{body}");
        let target_path = output_dir.join(format!("{name}.ts"));
        if check {
            let committed = fs::read_to_string(&target_path)
                .with_context(|| format!("read committed ts: {}", target_path.display()))?;
            if committed != generated {
                drift.push((*name).to_string());
            }
        } else {
            fs::write(&target_path, &generated)
                .with_context(|| format!("write ts: {}", target_path.display()))?;
        }
    }
    Ok(drift)
}

fn ts_header(schema_basename: &str) -> String {
    format!(
        "// AUTO-GENERATED FILE — DO NOT EDIT\n\
         //\n\
         // Regenerate with: `cargo xtask regenerate-types`\n\
         // Source schema:   schemas/{schema_basename}\n\
         // Generated by:    json-schema-to-typescript\n\
         //\n\
         // Drift detection runs in CI via `cargo xtask regenerate-types --check`.\n\
         \n",
    )
}

/// Production runner — shells out to `npx --yes json-schema-to-typescript`
/// against the supplied schema and returns the generated TS source.
fn run_npx_json_schema_to_typescript(schema_path: &std::path::Path) -> Result<String> {
    use std::process::Command;
    let npx_bin = if cfg!(windows) { "npx.cmd" } else { "npx" };
    let output = Command::new(npx_bin)
        .args([
            "--yes",
            "json-schema-to-typescript",
            "--unreachableDefinitions",
        ])
        .arg(schema_path)
        .output()
        .with_context(|| format!("spawn {npx_bin} json-schema-to-typescript"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "json-schema-to-typescript failed for {}: {stderr}",
            schema_path.display()
        );
    }
    String::from_utf8(output.stdout).context("json-schema-to-typescript output is not utf-8")
}

/// Resolve external `$ref` entries by inlining `$defs` from referenced schema files.
///
/// Typify does not support external references (e.g., `common.v1.json#/$defs/HookRef`).
/// This function:
/// 1. Finds all external schema files referenced via `$ref`
/// 2. Imports ALL `$defs` from each referenced file (to satisfy transitive internal refs)
/// 3. Rewrites external `$ref` to internal `#/$defs/<name>` format
fn resolve_external_refs(
    schema: &mut serde_json::Value,
    schemas_dir: &std::path::Path,
) -> Result<()> {
    // Collect all external file names referenced.
    let mut referenced_files: std::collections::HashSet<String> = std::collections::HashSet::new();
    collect_referenced_files(schema, &mut referenced_files);

    // For each referenced file, import ALL $defs into this schema.
    let mut external_defs: std::collections::HashMap<String, serde_json::Value> =
        std::collections::HashMap::new();

    for file in &referenced_files {
        let ext_path = schemas_dir.join(file);
        let ext_text = std::fs::read_to_string(&ext_path)
            .with_context(|| format!("read external schema: {}", ext_path.display()))?;
        let ext_schema: serde_json::Value = serde_json::from_str(&ext_text)?;

        // Import all $defs from the external schema.
        if let Some(ext_defs) = ext_schema.get("$defs").and_then(|d| d.as_object()) {
            for (name, def) in ext_defs {
                external_defs
                    .entry(name.clone())
                    .or_insert_with(|| def.clone());
            }
        }

        // For bare file refs (no fragment), also add the whole schema as a def.
        // Check if any ref is just the filename without a #.
        if has_bare_file_ref(schema, file) {
            let def_name = derive_def_name(file);
            external_defs.entry(def_name).or_insert_with(|| {
                let mut def = ext_schema;
                if let Some(obj) = def.as_object_mut() {
                    obj.remove("$schema");
                    obj.remove("$id");
                }
                def
            });
        }
    }

    // Merge external defs into this schema's $defs.
    if !external_defs.is_empty() {
        let defs = schema
            .as_object_mut()
            .context("schema must be an object")?
            .entry("$defs")
            .or_insert_with(|| serde_json::json!({}));
        let defs_obj = defs.as_object_mut().context("$defs must be an object")?;
        for (name, def) in external_defs {
            defs_obj.entry(&name).or_insert(def);
        }
    }

    // Rewrite external $ref strings to internal format.
    rewrite_refs(schema);
    Ok(())
}

/// Check if the schema has a bare file ref (no fragment) to the given file.
fn has_bare_file_ref(value: &serde_json::Value, file: &str) -> bool {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::String(ref_str)) = map.get("$ref") {
                if ref_str == file {
                    return true;
                }
            }
            map.values().any(|v| has_bare_file_ref(v, file))
        }
        serde_json::Value::Array(arr) => arr.iter().any(|v| has_bare_file_ref(v, file)),
        _ => false,
    }
}

/// Collect all external file names referenced via `$ref`.
#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn collect_referenced_files(
    value: &serde_json::Value,
    files: &mut std::collections::HashSet<String>,
) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::String(ref_str)) = map.get("$ref") {
                if let Some((file, _)) = ref_str.split_once('#') {
                    if !file.is_empty() {
                        files.insert(file.to_string());
                    }
                } else if ref_str.ends_with(".json") {
                    files.insert(ref_str.clone());
                }
            }
            for v in map.values() {
                collect_referenced_files(v, files);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                collect_referenced_files(v, files);
            }
        }
        _ => {}
    }
}

/// Derive a definition name from a bare file reference.
/// E.g., "agent.v1.json" → "Agent", "common.v1.json" → "Common".
fn derive_def_name(filename: &str) -> String {
    let stem = filename
        .strip_suffix(".json")
        .unwrap_or(filename)
        .split('.')
        .next()
        .unwrap_or(filename);
    let mut chars = stem.chars();
    chars.next().map_or_else(String::new, |c| {
        c.to_uppercase().to_string() + chars.as_str()
    })
}

#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn rewrite_refs(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::String(ref_str)) = map.get("$ref").cloned() {
                if let Some((file, path)) = ref_str.split_once('#') {
                    if !file.is_empty() {
                        // External ref with fragment → internal ref.
                        map.insert("$ref".to_string(), serde_json::json!(format!("#{path}")));
                    }
                } else if ref_str.ends_with(".json") {
                    // Bare file ref → internal ref to derived def name.
                    let def_name = derive_def_name(&ref_str);
                    map.insert(
                        "$ref".to_string(),
                        serde_json::json!(format!("#/$defs/{def_name}")),
                    );
                }
            }
            for v in map.values_mut() {
                rewrite_refs(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                rewrite_refs(v);
            }
        }
        _ => {}
    }
}

fn generate_one(
    schema_path: &std::path::Path,
    name: &str,
    schemas_dir: &std::path::Path,
) -> Result<String> {
    let schema_text = std::fs::read_to_string(schema_path)
        .with_context(|| format!("read schema: {}", schema_path.display()))?;
    let mut schema_value: serde_json::Value = serde_json::from_str(&schema_text)?;

    // Resolve external $ref entries before passing to typify.
    resolve_external_refs(&mut schema_value, schemas_dir)?;

    let schema: schemars::schema::RootSchema = serde_json::from_value(schema_value)?;

    let mut type_space =
        typify::TypeSpace::new(typify::TypeSpaceSettings::default().with_struct_builder(true));
    type_space
        .add_root_schema(schema)
        .context("typify add_root_schema")?;

    let header = format!(
        "// AUTO-GENERATED FILE — DO NOT EDIT\n\
         //\n\
         // Regenerate with: `cargo xtask regenerate-types`\n\
         // Source schema:   schemas/{name}.v1.json\n\
         // Generated by:    typify\n\
         //\n\
         // Drift detection runs in CI via `cargo xtask regenerate-types --check`.\n\
         \n\
         #![allow(clippy::pedantic, clippy::nursery, clippy::all, missing_docs, unused_imports, rustdoc::invalid_html_tags)]\n\
         \n",
    );

    let body = type_space.to_stream().to_string();
    let unformatted = format!("{header}{body}\n");

    // Format the generated code via rustfmt so it passes `cargo fmt --check`.
    format_rust(&unformatted)
}

/// Format Rust source code via `rustfmt`.
fn format_rust(source: &str) -> Result<String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("rustfmt")
        .arg("--edition=2021")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn rustfmt")?;

    child
        .stdin
        .as_mut()
        .context("rustfmt stdin")?
        .write_all(source.as_bytes())?;

    let output = child.wait_with_output().context("rustfmt wait")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("rustfmt failed: {stderr}");
    }
    String::from_utf8(output.stdout).context("rustfmt output is not utf-8")
}

fn workspace_root() -> Result<PathBuf> {
    let metadata = std::process::Command::new("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .arg("--no-deps")
        .output()
        .context("cargo metadata")?;
    let json: serde_json::Value = serde_json::from_slice(&metadata.stdout)?;
    let workspace = json["workspace_root"]
        .as_str()
        .context("workspace_root in cargo metadata output")?;
    Ok(PathBuf::from(workspace))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn regenerate_typescript_types_with_writes_header_and_body() {
        // Inject a stub runner that returns a deterministic TS body. The seam
        // must (a) prepend the auto-gen header banner naming the source schema
        // + generator, (b) write to <output_dir>/<name>.ts. No npx / no real
        // codegen — the test exercises the file-write + header-prepend logic.
        let dir = TempDir::new().expect("tempdir");
        let schemas_dir = dir.path().join("schemas");
        let output_dir = dir.path().join("types");
        std::fs::create_dir_all(&schemas_dir).expect("create schemas dir");
        std::fs::create_dir_all(&output_dir).expect("create output dir");
        let schema_path = schemas_dir.join("event.v1.json");
        std::fs::write(&schema_path, "{}").expect("write stub schema");

        let drift = regenerate_typescript_types_with(
            &[("event", schema_path.as_path())],
            &output_dir,
            |path| {
                assert_eq!(path, schema_path.as_path());
                Ok("export type Stub = 'ok';\n".to_string())
            },
            false,
        )
        .expect("regenerate_typescript_types_with");
        assert!(drift.is_empty(), "no drift expected on fresh write");

        let written = std::fs::read_to_string(output_dir.join("event.ts")).expect("read written");
        assert!(
            written.contains("AUTO-GENERATED FILE"),
            "header banner missing in {written}"
        );
        assert!(
            written.contains("schemas/event.v1.json"),
            "source-schema reference missing in {written}"
        );
        assert!(
            written.contains("json-schema-to-typescript"),
            "generator reference missing in {written}"
        );
        assert!(
            written.contains("export type Stub = 'ok';"),
            "body missing in {written}"
        );
    }

    #[test]
    fn regenerate_typescript_types_with_check_detects_drift() {
        // Pre-write a TS file with stale content; --check mode must flag drift
        // when the runner's fresh output differs from committed.
        let dir = TempDir::new().expect("tempdir");
        let schemas_dir = dir.path().join("schemas");
        let output_dir = dir.path().join("types");
        std::fs::create_dir_all(&schemas_dir).expect("create schemas dir");
        std::fs::create_dir_all(&output_dir).expect("create output dir");
        let schema_path = schemas_dir.join("event.v1.json");
        std::fs::write(&schema_path, "{}").expect("write stub schema");
        // Write stale content (no header, mismatching body).
        std::fs::write(output_dir.join("event.ts"), "stale content\n").expect("write stale");

        let drift = regenerate_typescript_types_with(
            &[("event", schema_path.as_path())],
            &output_dir,
            |_path| Ok("export type Fresh = 'new';\n".to_string()),
            true,
        )
        .expect("regenerate_typescript_types_with check");
        assert!(
            drift.contains(&"event".to_string()),
            "expected 'event' in drift list, got {drift:?}"
        );
    }

    #[test]
    fn regenerate_typescript_types_with_runner_error_propagates() {
        // Runner failure (e.g., npx subprocess returns non-zero) must surface
        // as an Err rather than silently writing a partial file.
        let dir = TempDir::new().expect("tempdir");
        let schemas_dir = dir.path().join("schemas");
        let output_dir = dir.path().join("types");
        std::fs::create_dir_all(&schemas_dir).expect("create schemas dir");
        std::fs::create_dir_all(&output_dir).expect("create output dir");
        let schema_path = schemas_dir.join("event.v1.json");
        std::fs::write(&schema_path, "{}").expect("write stub schema");

        let result = regenerate_typescript_types_with(
            &[("event", schema_path.as_path())],
            &output_dir,
            |_path| Err(anyhow::anyhow!("runner exploded")),
            false,
        );
        assert!(result.is_err(), "expected Err, got {result:?}");
        assert!(
            !output_dir.join("event.ts").exists(),
            "no file should be written on runner error"
        );
    }

    #[test]
    fn regenerate_typescript_types_with_byte_stable_on_repeat_runs() {
        // Determinism: running the seam twice with the same runner output
        // produces byte-identical files. This is the contract the
        // `cargo xtask regenerate-types --check` drift detector relies on.
        let dir = TempDir::new().expect("tempdir");
        let schemas_dir = dir.path().join("schemas");
        let output_dir = dir.path().join("types");
        std::fs::create_dir_all(&schemas_dir).expect("create schemas dir");
        std::fs::create_dir_all(&output_dir).expect("create output dir");
        let schema_path = schemas_dir.join("event.v1.json");
        std::fs::write(&schema_path, "{}").expect("write stub schema");

        let runner = |_path: &std::path::Path| Ok("export type X = 'x';\n".to_string());

        regenerate_typescript_types_with(
            &[("event", schema_path.as_path())],
            &output_dir,
            runner,
            false,
        )
        .expect("first run");
        let first = std::fs::read_to_string(output_dir.join("event.ts")).expect("read first");

        regenerate_typescript_types_with(
            &[("event", schema_path.as_path())],
            &output_dir,
            runner,
            false,
        )
        .expect("second run");
        let second = std::fs::read_to_string(output_dir.join("event.ts")).expect("read second");

        assert_eq!(first, second, "TS codegen output must be byte-stable");
    }
}
