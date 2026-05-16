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

pub use aliases::{Aliases, AliasError};

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
    // M06.D red-phase: the green-phase resolve/connect/disconnect impl
    // reads this. `expect` (not `allow`) is self-deactivating — it
    // warns "unfulfilled" once green phase reads the field, so the
    // green-phase clippy-fix step removes it with no manual cleanup
    // (M06.C decision; gotcha #79 candidate).
    #[expect(dead_code, reason = "M06.D green phase reads the connected-server snapshot")]
    connected: BTreeMap<String, Vec<String>>,
}

impl NamespaceResolver {
    /// Construct from the initial connected-server snapshot.
    #[must_use]
    pub fn new(connected: BTreeMap<String, Vec<String>>) -> Self {
        // Red-phase stub (M06.D strict TDD): green phase implements.
        let _ = connected;
        unimplemented!("M06.D green phase: NamespaceResolver::new")
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
        // Red-phase stub (M06.D strict TDD): green phase implements.
        let _ = (name, aliases);
        unimplemented!("M06.D green phase: NamespaceResolver::resolve")
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
        // Red-phase stub (M06.D strict TDD): green phase implements.
        let _ = (server.into(), tools);
        unimplemented!("M06.D green phase: NamespaceResolver::connect_server")
    }

    /// Record a server disconnect. Disconnect can only REMOVE ambiguity,
    /// never create it, so this returns an empty vec; the re-evaluation
    /// still updates the snapshot so subsequent `resolve` calls reflect
    /// the smaller connected set.
    pub fn disconnect_server(&mut self, server: &str) -> Vec<NewAmbiguity> {
        // Red-phase stub (M06.D strict TDD): green phase implements.
        let _ = server;
        unimplemented!("M06.D green phase: NamespaceResolver::disconnect_server")
    }

    /// Snapshot accessor — the canonical `<server>__<tool>` names a
    /// short name currently matches (used by dispatch + tests).
    #[must_use]
    pub fn candidates_for(&self, short_name: &str) -> Vec<String> {
        // Red-phase stub (M06.D strict TDD): green phase implements.
        let _ = short_name;
        unimplemented!("M06.D green phase: NamespaceResolver::candidates_for")
    }
}
