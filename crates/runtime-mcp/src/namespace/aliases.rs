//! Framework `mcp_aliases` wrapper + validation — M06.D, spec §5a step 3.
//!
//! `framework.mcp_aliases` is a `{ short → canonical }` map. This
//! wrapper validates that every value is a well-formed canonical
//! `<server>__<tool>` (contains `__`, non-empty server + tool) and
//! detects collisions where two short names map to the same canonical
//! (last-write-wins is silent data loss otherwise). The resolver
//! consults the raw map; this type is the load-time guard the framework
//! loader (M07) + Settings UI (Stage E) use to reject a malformed
//! `mcp_aliases` block before it reaches dispatch.

use std::collections::BTreeMap;

/// Why [`Aliases::validate`] rejected an `mcp_aliases` block.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AliasError {
    /// An alias value is not a canonical `<server>__<tool>` form.
    #[error("alias '{alias}' value '{value}' is not a canonical '<server>__<tool>' name")]
    NotCanonical {
        /// The offending short name (map key).
        alias: String,
        /// The malformed value (map value).
        value: String,
    },
    /// Two distinct short names map to the same canonical target.
    #[error("aliases '{a}' and '{b}' both map to canonical '{canonical}'")]
    Collision {
        /// First short name.
        a: String,
        /// Second short name.
        b: String,
        /// The shared canonical target.
        canonical: String,
    },
}

/// Validated wrapper over a framework `mcp_aliases` map.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Aliases(BTreeMap<String, String>);

impl Aliases {
    /// Wrap a raw `mcp_aliases` map WITHOUT validation (the resolver's
    /// hot path; `validate` is the load-time guard).
    #[must_use]
    pub fn new(map: BTreeMap<String, String>) -> Self {
        // Red-phase stub (M06.D strict TDD): green phase implements.
        let _ = map;
        unimplemented!("M06.D green phase: Aliases::new")
    }

    /// Borrow the underlying map for [`super::NamespaceResolver::resolve`].
    #[must_use]
    pub fn as_map(&self) -> &BTreeMap<String, String> {
        // Red-phase stub (M06.D strict TDD): green phase implements.
        unimplemented!("M06.D green phase: Aliases::as_map")
    }

    /// Validate every value is a canonical `<server>__<tool>` form and
    /// no two short names collide on the same canonical.
    ///
    /// # Errors
    ///
    /// - [`AliasError::NotCanonical`] — a value lacks `__` / has an
    ///   empty server or tool segment.
    /// - [`AliasError::Collision`] — two short names share a canonical.
    pub fn validate(&self) -> Result<(), AliasError> {
        // Red-phase stub (M06.D strict TDD): green phase implements.
        unimplemented!("M06.D green phase: Aliases::validate")
    }
}
