//! §5a Tool Namespace Resolution — M06.D.
//!
//! Resolves a tool name an agent invoked to a concrete
//! `(server, tool)` pair across the set of currently-connected MCP
//! servers, per spec §5a (locked 2026-04-18, WI-11):
//!
//! 1. **Alias override** — a framework `mcp_aliases` entry maps a
//!    short name to a canonical `<server>__<tool>`.
//! 2. **Canonical name** — `<server>__<tool>`, split on the FIRST `__`
//!    (server names cannot contain `__`; tool names may — spec §5a
//!    step 4).
//! 3. **Short name** — resolves iff unambiguous across all connected
//!    servers; ambiguous fails with the candidate list so the framework
//!    can pin via `mcp_aliases`.
//! 4. **Re-resolution on connect/disconnect** — [`NamespaceResolver::
//!    connect_server`] / [`NamespaceResolver::disconnect_server`]
//!    re-evaluate short-name uniqueness; names that BECAME ambiguous on
//!    a connect surface as [`NewAmbiguity`] records the caller
//!    translates to `tool_alias_ambiguous` events.
//!
//! Pure: no I/O, no async. Resolution is total over the connected-
//! server snapshot held at construction / last connect-disconnect.

pub mod aliases;

pub use aliases::{AliasError, Aliases};

use std::collections::BTreeMap;

/// A tool name resolved to a concrete connected server + tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTool {
    /// Connected MCP server that exposes `tool`.
    pub server: String,
    /// Tool name as the server exposes it (may contain `__`).
    pub tool: String,
}

/// Why [`NamespaceResolver::resolve`] could not resolve a name.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NamespaceError {
    /// No connected server exposes a tool matching `name` (neither a
    /// canonical `<server>__<tool>` for a connected server nor a short
    /// name present on any connected server).
    #[error("tool '{0}' not found in any connected MCP server")]
    NotFound(String),
    /// A short name matched >1 connected server. `candidates` lists the
    /// canonical `<server>__<tool>` forms so the framework can pin one
    /// via `mcp_aliases`.
    #[error("tool '{name}' is ambiguous; candidates: {candidates:?}")]
    Ambiguous {
        /// The ambiguous short name.
        name: String,
        /// The ≥2 canonical candidates.
        candidates: Vec<String>,
    },
    /// An `mcp_aliases` entry maps `0` to a canonical name `1` that no
    /// connected server exposes.
    #[error("alias '{0}' points at unknown canonical '{1}'")]
    UnknownAlias(String, String),
}

/// A short name that BECAME ambiguous on a connect (spec §5a step 5).
/// The caller (`McpClient` lifecycle) translates each to a
/// `tool_alias_ambiguous` warning event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAmbiguity {
    /// The short tool name that is now ambiguous.
    pub short_name: String,
    /// The ≥2 canonical `<server>__<tool>` candidates.
    pub candidates: Vec<String>,
}

/// §5a resolver over the currently-connected MCP server set.
///
/// Holds a snapshot of `server → [tool, ...]`. Connect/disconnect
/// mutate the snapshot and return any newly-ambiguous short names.
#[derive(Debug, Clone, Default)]
pub struct NamespaceResolver {
    connected: BTreeMap<String, Vec<String>>,
}

impl NamespaceResolver {
    /// Construct from the initial connected-server snapshot.
    #[must_use]
    pub const fn new(connected: BTreeMap<String, Vec<String>>) -> Self {
        Self { connected }
    }

    /// Resolve a tool name per §5a (alias → canonical → short).
    ///
    /// # Errors
    ///
    /// - [`NamespaceError::NotFound`] — no connected server exposes it.
    /// - [`NamespaceError::Ambiguous`] — short name on >1 server.
    /// - [`NamespaceError::UnknownAlias`] — alias points at an
    ///   unconnected canonical.
    pub fn resolve(
        &self,
        name: &str,
        aliases: &BTreeMap<String, String>,
    ) -> Result<ResolvedTool, NamespaceError> {
        // §5a step 1 — alias override. An alias resolves to its
        // canonical; failing that is an UnknownAlias (a framework
        // config error), distinct from a NotFound short name.
        if let Some(canonical) = aliases.get(name) {
            return self
                .resolve_canonical(canonical)
                .ok_or_else(|| NamespaceError::UnknownAlias(name.to_string(), canonical.clone()));
        }
        // §5a step 2 — canonical `<server>__<tool>`, split on the FIRST
        // `__` (server names cannot contain `__`; tool names may, so
        // the remainder after the first `__` is the whole tool name).
        if name.contains("__") {
            return self
                .resolve_canonical(name)
                .ok_or_else(|| NamespaceError::NotFound(name.to_string()));
        }
        // §5a step 3 — short name; resolves iff unambiguous. Match the
        // servers directly (no canonical string round-trip) so there's
        // no infallible-`expect` to document.
        let servers: Vec<&String> = self
            .connected
            .iter()
            .filter(|(_, tools)| tools.iter().any(|t| t == name))
            .map(|(server, _)| server)
            .collect();
        match servers.as_slice() {
            [] => Err(NamespaceError::NotFound(name.to_string())),
            [server] => Ok(ResolvedTool {
                server: (*server).clone(),
                tool: name.to_string(),
            }),
            _ => Err(NamespaceError::Ambiguous {
                name: name.to_string(),
                candidates: self.candidates_for(name),
            }),
        }
    }

    /// Record a server connect + its tool list. Returns the short names
    /// that BECAME ambiguous as a result (spec §5a step 5). Idempotent:
    /// re-connecting the same server with the same tools yields no new
    /// ambiguity.
    pub fn connect_server(
        &mut self,
        server: impl Into<String>,
        tools: Vec<String>,
    ) -> Vec<NewAmbiguity> {
        let before = self.ambiguous_short_names();
        self.connected.insert(server.into(), tools);
        self.newly_ambiguous_since(&before)
    }

    /// Record a server disconnect. Disconnect can only REMOVE ambiguity,
    /// never create it, so this returns an empty vec; the re-evaluation
    /// still updates the snapshot so subsequent `resolve` calls reflect
    /// the smaller connected set.
    pub fn disconnect_server(&mut self, server: &str) -> Vec<NewAmbiguity> {
        let before = self.ambiguous_short_names();
        self.connected.remove(server);
        // Removing a server cannot introduce a new collision; the
        // diff against `before` is provably empty. Kept symmetric with
        // `connect_server` so the re-evaluation site is one shape.
        self.newly_ambiguous_since(&before)
    }

    /// Snapshot accessor — the canonical `<server>__<tool>` names a
    /// short name currently matches, sorted for deterministic output.
    #[must_use]
    pub fn candidates_for(&self, short_name: &str) -> Vec<String> {
        let mut out: Vec<String> = self
            .connected
            .iter()
            .filter(|(_, tools)| tools.iter().any(|t| t == short_name))
            .map(|(server, _)| format!("{server}__{short_name}"))
            .collect();
        out.sort();
        out
    }

    /// Resolve a canonical `<server>__<tool>` to a `ResolvedTool` iff
    /// that server is connected AND exposes that tool. `None` otherwise
    /// (caller maps to `NotFound` / `UnknownAlias` per context).
    fn resolve_canonical(&self, canonical: &str) -> Option<ResolvedTool> {
        let resolved = Self::split_canonical(canonical)?;
        let exposed = self
            .connected
            .get(&resolved.server)
            .is_some_and(|tools| tools.contains(&resolved.tool));
        exposed.then_some(resolved)
    }

    /// Split a canonical name on the FIRST `__`. `None` if there is no
    /// `__` or either segment is empty (spec §5a step 4 — server names
    /// cannot contain `__`; the remainder is the tool, which MAY).
    fn split_canonical(canonical: &str) -> Option<ResolvedTool> {
        let (server, tool) = canonical.split_once("__")?;
        if server.is_empty() || tool.is_empty() {
            return None;
        }
        Some(ResolvedTool {
            server: server.to_string(),
            tool: tool.to_string(),
        })
    }

    /// The set of short names currently ambiguous (exposed by ≥2
    /// connected servers), mapped to their sorted canonical candidates.
    fn ambiguous_short_names(&self) -> BTreeMap<String, Vec<String>> {
        let mut counts: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (server, tools) in &self.connected {
            for tool in tools {
                counts
                    .entry(tool.clone())
                    .or_default()
                    .push(format!("{server}__{tool}"));
            }
        }
        counts.retain(|_, c| c.len() >= 2);
        for c in counts.values_mut() {
            c.sort();
        }
        counts
    }

    /// Short names ambiguous now but not in `before` — the spec §5a
    /// step 5 "newly ambiguous on connect" delta.
    fn newly_ambiguous_since(&self, before: &BTreeMap<String, Vec<String>>) -> Vec<NewAmbiguity> {
        self.ambiguous_short_names()
            .into_iter()
            .filter(|(name, _)| !before.contains_key(name))
            .map(|(short_name, candidates)| NewAmbiguity {
                short_name,
                candidates,
            })
            .collect()
    }
}
