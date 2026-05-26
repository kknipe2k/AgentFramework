//! Framework save/load (M08 Stage B; M08.6 Stage B per ADR-0022).
//!
//! Path-agnostic persistence per CLAUDE.md §9 (the `audit::file_path` /
//! `skills_lock` archetype): the functions take `dir: &Path`; the Tauri
//! shell resolves the directory (from the save dialog / file picker)
//! and passes it in. No workspace dependency on `dirs`. Tested with
//! `tempfile`-backed paths.
//!
//! [`save_framework`] writes `framework.json` (pretty-printed, stable
//! field order, trailing newline) plus one companion `.md` per inline
//! artifact; [`load_framework`] parses `framework.json` AND resolves
//! every `{id,path}` agents[] reference (plus path-referenced tools /
//! skills) into the same typify-generated `Framework` type the rest of
//! the runtime uses (spec §9 — no second loader). Reference resolution
//! lives **only** here per ADR-0022 — the canvas projection, the
//! Tester, and the runtime's `spawn_framework_subagents` consume the
//! resolved inline form.

use std::path::Path;

use runtime_core::generated::framework::{Agent, Framework, FrameworkAgentsItem};

use crate::builder::error::BuilderError;

/// One inline-defined artifact's companion markdown file.
///
/// Derives `serde` both ways: it crosses the Tauri IPC boundary as a
/// `save_framework` argument and a `load_framework` return value.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Companion {
    /// File name relative to the framework directory — e.g.
    /// `summarize.skill.md` (M08-era flat layout) or
    /// `agents/orchestrator.md` (canonical-modular layout per
    /// ADR-0022, incl. cross-framework `../aria/tools/aria_verify.md`).
    pub file_name: String,
    /// Full markdown body (frontmatter + content), written verbatim.
    pub body: String,
}

/// A framework reloaded from disk — the canvas reconstructs from this.
///
/// Derives `serde::Serialize` (it crosses the Tauri IPC boundary as the
/// `load_framework` return value). The round-trip is byte-stable so the
/// canvas reconstructs identical to save state (the ADR-0020
/// canvas-projection contract).
#[derive(Debug, Clone, serde::Serialize)]
pub struct LoadedFramework {
    /// The parsed `framework.json` with every `{id,path}` agents[]
    /// reference resolved to its inline `Agent` variant per ADR-0022.
    pub framework: Framework,
    /// The companion markdown files found alongside it, sorted by file
    /// name for a deterministic (byte-stable) round-trip. Includes
    /// both the resolved subdirectory `.md` files (agents/tools/skills
    /// referenced from `framework.json`) and any M08-era flat
    /// `*.{agent,skill,tool}.md` companions at the top level.
    pub companions: Vec<Companion>,
}

/// Write `framework.json` + a companion `.md` for every inline artifact.
///
/// Writes `dir/framework.json` (pretty-printed, stable field order, one
/// trailing newline) plus one file per `companions` entry. `dir` is
/// created if absent.
///
/// # Errors
///
/// - [`BuilderError::NotADirectory`] if `dir` exists and is a file.
/// - [`BuilderError::Json`] if the framework cannot serialize.
/// - [`BuilderError::Io`] on any write failure.
pub fn save_framework(
    dir: &Path,
    fw: &Framework,
    companions: &[Companion],
) -> Result<(), BuilderError> {
    if dir.exists() && !dir.is_dir() {
        return Err(BuilderError::NotADirectory(dir.display().to_string()));
    }
    std::fs::create_dir_all(dir)?;
    // Pretty-printed + a single trailing newline gives a deterministic
    // serialization — a save→load→save cycle is byte-stable.
    let json = serde_json::to_string_pretty(fw)?;
    std::fs::write(dir.join("framework.json"), format!("{json}\n"))?;
    for companion in companions {
        std::fs::write(dir.join(&companion.file_name), &companion.body)?;
    }
    Ok(())
}

/// Read `framework.json`, resolve every `{id,path}` reference, and
/// return the resolved framework + its companion `.md` bodies.
///
/// Per ADR-0022 this is the single resolution boundary. For every
/// `FrameworkAgentsItem::Object { id, path }` entry the function reads
/// `dir.join(path)`, splits its YAML frontmatter, parses it into an
/// inline `Agent`, and replaces the entry with the
/// `FrameworkAgentsItem::Agent(_)` variant. For every path-referenced
/// `tools[]` / `skills[]` entry it reads the referenced `.md` and
/// surfaces the body in `companions` (those item types are flat structs
/// in the schema, not a `oneOf` — there is no inline variant to flip
/// into, so the resolved body lives in `companions` for the canvas +
/// Stage C re-split).
///
/// Gap-tolerance is preserved for *partially-built inline* frameworks
/// (a partial inline framework whose fields are still unfilled reloads
/// fine; the Builder surfaces gaps via `validate_framework`). A
/// **broken reference** — a referenced `.md` that is missing,
/// unreadable, or whose YAML frontmatter does not parse — is the
/// distinct [`BuilderError::ReferenceResolution`] error per ADR-0022
/// (a `{id,path}` is a structural promise; a missing target is an
/// error, not a gap).
///
/// Relative paths resolve against `dir`, including `../` cross-framework
/// references (Ralph's `../aria/tools/...`). The loader reads only the
/// files `framework.json` explicitly names — no glob, no symlink walk;
/// the single referenced file is the deliberate read.
///
/// # Errors
///
/// - [`BuilderError::Io`] if `dir/framework.json` is missing/unreadable.
/// - [`BuilderError::Json`] if `framework.json` does not parse.
/// - [`BuilderError::ReferenceResolution`] if any referenced `.md` is
///   missing, unreadable, or has unparseable YAML frontmatter.
pub fn load_framework(dir: &Path) -> Result<LoadedFramework, BuilderError> {
    let raw = std::fs::read_to_string(dir.join("framework.json"))?;
    // Parse into the same typify-generated `Framework` type the rest of
    // the runtime uses — no second loader (spec §9). Gap-tolerant on
    // partially-built inline frameworks; broken references are caught
    // in `resolve_references` below.
    let mut framework: Framework = serde_json::from_str(&raw)?;
    let mut companions = resolve_references(dir, &mut framework)?;
    // Backward compat: pick up M08-era top-level flat `.skill.md` /
    // `.tool.md` / `.agent.md` files. The canonical-modular layout uses
    // subdirectory paths from `framework.json` (already handled above);
    // this scan keeps inline-only / flat-companion frameworks loading.
    // Dedupe so a file referenced both ways is included once.
    for c in read_flat_companions(dir)? {
        if !companions.iter().any(|r| r.file_name == c.file_name) {
            companions.push(c);
        }
    }
    companions.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(LoadedFramework {
        framework,
        companions,
    })
}

/// Resolve every `{id,path}` agents[] entry and every path-referenced
/// `tools[]` / `skills[]` entry, mutating `framework.agents[]` in place
/// (the variant flip) and accumulating the read `.md` bodies as
/// companions.
///
/// Order: agents first (so the variant flip happens before downstream
/// code sees `framework.agents[]`), then tools, then skills. The final
/// `companions` list is sorted by file name in [`load_framework`], so
/// internal order here does not affect the wire shape.
fn resolve_references(
    dir: &Path,
    framework: &mut Framework,
) -> Result<Vec<Companion>, BuilderError> {
    let mut companions: Vec<Companion> = Vec::new();
    for item in &mut framework.agents {
        let path_str = match item {
            FrameworkAgentsItem::Object { path, .. } => path.clone(),
            FrameworkAgentsItem::Agent(_) => continue,
        };
        let body = read_referenced_md(dir, &path_str)?;
        let normalized = body.replace("\r\n", "\n");
        let (frontmatter, _md_body) =
            split_frontmatter(&normalized).ok_or_else(|| BuilderError::ReferenceResolution {
                reference: path_str.clone(),
                cause: format!(
                    "no YAML frontmatter found (expected a leading `---` block) in {path_str}"
                ),
            })?;
        let agent: Agent =
            serde_yaml::from_str(frontmatter).map_err(|err| BuilderError::ReferenceResolution {
                reference: path_str.clone(),
                cause: format!("YAML frontmatter parse: {err}"),
            })?;
        companions.push(Companion {
            file_name: path_str,
            body,
        });
        *item = FrameworkAgentsItem::Agent(agent);
    }
    for tool in &framework.tools {
        if let Some(path) = &tool.path {
            let body = read_referenced_md(dir, path)?;
            companions.push(Companion {
                file_name: path.clone(),
                body,
            });
        }
    }
    for skill in &framework.skills {
        if let Some(path) = &skill.path {
            let body = read_referenced_md(dir, path)?;
            companions.push(Companion {
                file_name: path.clone(),
                body,
            });
        }
    }
    Ok(companions)
}

/// Read the single `.md` file `dir/path` (relative join, `../`
/// permitted for cross-framework references per ADR-0022). A missing /
/// unreadable target is [`BuilderError::ReferenceResolution`], NOT a
/// silent drop and NOT a panic.
fn read_referenced_md(dir: &Path, path: &str) -> Result<String, BuilderError> {
    let resolved = dir.join(path);
    std::fs::read_to_string(&resolved).map_err(|err| BuilderError::ReferenceResolution {
        reference: path.to_string(),
        cause: format!("read {} failed: {err}", resolved.display()),
    })
}

/// Scan `dir` for top-level flat companion markdown files (the M08-era
/// `*.{agent,skill,tool}.md` convention). The canonical-modular layout
/// per ADR-0022 stores `.md` files in `agents/`, `tools/`, `skills/`
/// subdirectories — those are resolved from `framework.json`
/// references in [`resolve_references`]; this flat scan only picks up
/// inline / hand-saved companions at the top level.
fn read_flat_companions(dir: &Path) -> Result<Vec<Companion>, BuilderError> {
    let mut companions: Vec<Companion> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if is_companion_md(file_name) {
            companions.push(Companion {
                file_name: file_name.to_string(),
                body: std::fs::read_to_string(&path)?,
            });
        }
    }
    Ok(companions)
}

/// Whether `name` is a top-level flat companion markdown file (the
/// M08-era convention preserved for inline frameworks).
fn is_companion_md(name: &str) -> bool {
    name.ends_with(".skill.md") || name.ends_with(".tool.md") || name.ends_with(".agent.md")
}

/// Split a `.md` artifact's YAML frontmatter from its markdown body.
///
/// Returns `Some((frontmatter, body))` when `text` is a well-formed
/// `---\n<yaml>\n---\n<body>` document — both delimiters are line-leading
/// `---\n`. Returns `None` otherwise.
///
/// Caller-normalize contract: `text` must use LF line endings; the
/// caller normalizes CRLF first (mirrors the
/// `runtime-core/tests/round_trip.rs:139` precedent). Borrowed slices
/// into `text`, no allocation.
#[must_use]
pub fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = text.splitn(3, "---\n").collect();
    if parts.len() == 3 && parts[0].is_empty() {
        Some((parts[1], parts[2]))
    } else {
        None
    }
}
