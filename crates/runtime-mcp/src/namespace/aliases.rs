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
    pub const fn new(map: BTreeMap<String, String>) -> Self {
        Self(map)
    }

    /// Borrow the underlying map for [`super::NamespaceResolver::resolve`].
    #[must_use]
    pub const fn as_map(&self) -> &BTreeMap<String, String> {
        &self.0
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
        // BTreeMap iteration is key-sorted → deterministic collision
        // pair (a < b lexically on the short names).
        let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
        for (alias, value) in &self.0 {
            match value.split_once("__") {
                Some((server, tool)) if !server.is_empty() && !tool.is_empty() => {}
                _ => {
                    return Err(AliasError::NotCanonical {
                        alias: alias.clone(),
                        value: value.clone(),
                    });
                }
            }
            if let Some(first) = seen.insert(value.as_str(), alias.as_str()) {
                return Err(AliasError::Collision {
                    a: first.to_string(),
                    b: alias.clone(),
                    canonical: value.clone(),
                });
            }
        }
        Ok(())
    }
}
