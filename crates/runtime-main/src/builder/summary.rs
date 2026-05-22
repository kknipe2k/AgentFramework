//! Whole-framework capability summary (M08 Stage B).
//!
//! [`framework_capability_summary`] aggregates the agents' coarse
//! `Capabilities` blocks into whole-framework totals and carries, per
//! Agent→Agent spawn edge, the narrowing triple
//! `{ parent_caps, child_declared_caps, narrowed_caps }` computed by the
//! **reused** [`crate::capability::narrowing::narrow`] (M05.B L2a). Spec
//! §9 forbids a second copy of the narrowing intersection in TS — the
//! renderer renders the triple, it never computes an intersection.
//!
//! [`validate_framework`] embeds the result as the report's
//! `capability_summary` field — there is no separate
//! `framework_capability_summary` Tauri command; the Inspector (E) and
//! the canvas (D2) read one report.
//!
//! [`validate_framework`]: crate::builder::validate::validate_framework

use runtime_core::generated::capability::CapabilityDeclaration;
use runtime_core::generated::framework::Framework;

use crate::capability::narrowing::narrow;
use crate::framework_loader::capability_map;

/// The narrowing decision for one Agent→Agent (`spawns`) edge.
///
/// Carries no `PartialEq` — `CapabilityDeclaration` derives only
/// `Clone` and `Debug` (its `CapabilityScope` is a typify `oneOf`
/// wrapper). Tests compare via `serde_json::to_value` or per-field.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SpawnEdgeNarrowing {
    /// Parent agent id.
    pub parent_id: String,
    /// Child (spawned) agent id.
    pub child_id: String,
    /// The parent's grant set.
    pub parent_caps: Vec<CapabilityDeclaration>,
    /// The child's declared grant set (pre-narrowing).
    pub child_declared_caps: Vec<CapabilityDeclaration>,
    /// The narrowed result — `narrow(parent_caps, child_declared_caps)`.
    /// `Ok` carries the child's declared set verbatim (L2a is
    /// all-or-nothing — there is no partial clamp in v0.1); `Err`
    /// stringifies the [`crate::capability::error::NarrowingError`]
    /// because the report crosses the IPC boundary and `NarrowingError`
    /// is not `Serialize`. `validate_framework` folds every `Err` into
    /// `capability_errors` keyed to the child agent.
    pub narrowed_caps: Result<Vec<CapabilityDeclaration>, String>,
}

/// Whole-framework capability picture (spec Phase 9 Inspector).
///
/// Carries no `PartialEq` — see [`SpawnEdgeNarrowing`].
#[derive(Debug, Clone, serde::Serialize)]
pub struct FrameworkCapabilitySummary {
    /// Distinct file-read globs across every inline agent (sorted).
    pub files_read: Vec<String>,
    /// Distinct file-write globs across every inline agent (sorted).
    pub files_written: Vec<String>,
    /// Distinct network hosts across every inline agent (sorted).
    pub network_hosts: Vec<String>,
    /// Whether any inline agent declares `shell: true`.
    pub any_shell: bool,
    /// The narrowing decision for every Agent→Agent spawn edge, in
    /// framework declaration order.
    pub spawn_edges: Vec<SpawnEdgeNarrowing>,
}

/// Compute the whole-framework capability summary.
///
/// Reuses [`crate::framework_loader::capability_map`] for the per-agent
/// grant translation and [`crate::capability::narrowing::narrow`] for
/// the per-spawn-edge intersection — neither is reimplemented (spec §9).
#[must_use]
pub fn framework_capability_summary(fw: &Framework) -> FrameworkCapabilitySummary {
    let agents = capability_map::inline_agents(fw);

    // 1. Aggregate the coarse file / network / shell totals across
    //    every inline agent's `Capabilities` block.
    let mut files_read: Vec<String> = Vec::new();
    let mut files_written: Vec<String> = Vec::new();
    let mut network_hosts: Vec<String> = Vec::new();
    let mut any_shell = false;
    for &agent in &agents {
        let caps = &agent.capabilities;
        for glob in caps.file_access.read.iter() {
            files_read.push((**glob).clone());
        }
        for glob in caps.file_access.write.iter() {
            files_written.push((**glob).clone());
        }
        for host in &caps.network {
            network_hosts.push(host.clone());
        }
        if caps.shell {
            any_shell = true;
        }
    }
    dedup_sorted(&mut files_read);
    dedup_sorted(&mut files_written);
    dedup_sorted(&mut network_hosts);

    // 2. Build one SpawnEdgeNarrowing per Agent→Agent (`spawns`) edge,
    //    in framework declaration order. The narrowing is the reused
    //    L2a `narrow()` (M05.B) — never reimplemented (spec §9); its
    //    `NarrowingError` is stringified because the report crosses the
    //    IPC boundary. An agent whose grants are not walkable (an
    //    unresolved or registry-form id) contributes an empty set.
    let mut spawn_edges: Vec<SpawnEdgeNarrowing> = Vec::new();
    for &agent in &agents {
        let parent_id = agent.id.as_str().to_string();
        let parent_caps =
            capability_map::parent_grants_for_agent(fw, &parent_id).unwrap_or_default();
        for child_id in &agent.spawns {
            let child_declared_caps =
                capability_map::parent_grants_for_agent(fw, child_id).unwrap_or_default();
            let narrowed_caps =
                narrow(&parent_caps, &child_declared_caps).map_err(|error| error.to_string());
            spawn_edges.push(SpawnEdgeNarrowing {
                parent_id: parent_id.clone(),
                child_id: child_id.clone(),
                parent_caps: parent_caps.clone(),
                child_declared_caps,
                narrowed_caps,
            });
        }
    }

    FrameworkCapabilitySummary {
        files_read,
        files_written,
        network_hosts,
        any_shell,
        spawn_edges,
    }
}

/// Sort + de-duplicate a glob/host list so the summary is stable and
/// carries each distinct entry once.
fn dedup_sorted(items: &mut Vec<String>) {
    items.sort();
    items.dedup();
}
