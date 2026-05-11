//! Don't-touch glob matcher. Spec §4a.
//!
//! Built-in pre-edit rail. Framework JSON declares glob patterns; any
//! agent attempting to write a matching path triggers a hard rail. The
//! `pre_file_edit` firing point is the integration site: every Write tool
//! invocation must route through [`DontTouchEvaluator::evaluate`] BEFORE
//! the OS write lands.
//!
//! ## Integration deferral
//!
//! No Write-tool dispatcher exists in `runtime-main` at v0.1 — the SDK
//! drives LLM streaming + structured-emitter parsing only. Stage D ships
//! the don't-touch evaluator as a callable primitive that the future
//! capability enforcer (M05) and plan loop (M07) will route through. The
//! evaluator is fully tested standalone; the wire-up ships when those
//! milestones land their dispatcher surfaces.

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use thiserror::Error;

/// Errors raised when constructing a [`DontTouchEvaluator`].
#[derive(Debug, Error)]
pub enum DontTouchError {
    /// One of the glob patterns failed to compile.
    #[error("invalid glob pattern at index {index}: {source}")]
    Invalid {
        /// Index of the invalid pattern in the input list.
        index: usize,
        /// Underlying globset compile error.
        #[source]
        source: globset::Error,
    },
}

/// Decision returned by [`DontTouchEvaluator::evaluate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DontTouchDecision {
    /// Path is not in any don't-touch glob; the edit may proceed.
    Allow,
    /// Path matches at least one don't-touch glob; the edit must be
    /// blocked. Carries the matched pattern (first match wins on
    /// multi-match — pattern recovery is index-stable per
    /// [`globset::GlobSet::matches`]).
    Block {
        /// The matched glob pattern (verbatim from the framework JSON).
        matched_pattern: String,
    },
}

/// Globset-backed don't-touch matcher.
///
/// Built once at framework load time; called per Write tool invocation.
/// Uses case-insensitive matching to align with Windows file-system
/// semantics — a framework that lists `package-lock.json` as don't-touch
/// must also block writes to `Package-Lock.json` on Windows. Linux is
/// case-sensitive at the FS layer but the runtime's enforcement layer
/// stays consistent across platforms (defense-in-depth: a cross-platform
/// framework shouldn't produce different behavior depending on host OS).
#[derive(Debug, Clone)]
pub struct DontTouchEvaluator {
    set: GlobSet,
    patterns: Vec<String>,
}

impl DontTouchEvaluator {
    /// Build an evaluator from a list of glob patterns. Empty list is
    /// allowed — every path will be allowed.
    ///
    /// # Errors
    ///
    /// Returns [`DontTouchError::Invalid`] if any pattern fails to compile
    /// (e.g., unbalanced brackets per gitignore syntax).
    pub fn new<I, S>(patterns: I) -> Result<Self, DontTouchError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut builder = GlobSetBuilder::new();
        let mut owned: Vec<String> = Vec::new();
        for (index, pattern) in patterns.into_iter().enumerate() {
            let p = pattern.as_ref();
            let glob = GlobBuilder::new(p)
                .case_insensitive(true)
                .literal_separator(false)
                .build()
                .map_err(|source| DontTouchError::Invalid { index, source })?;
            builder.add(glob);
            owned.push(p.to_string());
        }
        let set = builder
            .build()
            .map_err(|source| DontTouchError::Invalid { index: 0, source })?;
        Ok(Self {
            set,
            patterns: owned,
        })
    }

    /// Whether the evaluator has no patterns (always returns `Allow`).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Evaluate a write target against the don't-touch globs. Returns
    /// the first matching pattern (multi-match: first-by-index wins).
    #[must_use]
    pub fn evaluate(&self, path: &str) -> DontTouchDecision {
        if self.patterns.is_empty() {
            return DontTouchDecision::Allow;
        }
        let matched = self.set.matches(path);
        if let Some(&first) = matched.first() {
            DontTouchDecision::Block {
                matched_pattern: self.patterns[first].clone(),
            }
        } else {
            DontTouchDecision::Allow
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_pattern_list_allows_every_path() {
        let e = DontTouchEvaluator::new(Vec::<&str>::new()).expect("ok");
        assert!(e.is_empty());
        assert_eq!(e.evaluate("anything.rs"), DontTouchDecision::Allow);
        assert_eq!(e.evaluate(".env"), DontTouchDecision::Allow);
    }

    #[test]
    fn matched_glob_blocks_with_pattern_string() {
        let e = DontTouchEvaluator::new([".env*"]).expect("ok");
        assert_eq!(
            e.evaluate(".env"),
            DontTouchDecision::Block {
                matched_pattern: ".env*".to_string(),
            }
        );
        assert_eq!(
            e.evaluate(".env.local"),
            DontTouchDecision::Block {
                matched_pattern: ".env*".to_string(),
            }
        );
    }

    #[test]
    fn unmatched_glob_allows() {
        let e = DontTouchEvaluator::new([".env*"]).expect("ok");
        assert_eq!(e.evaluate("config.toml"), DontTouchDecision::Allow);
    }

    #[test]
    fn recursive_glob_matches_nested_paths() {
        let e = DontTouchEvaluator::new([".aria/state/**"]).expect("ok");
        assert_eq!(
            e.evaluate(".aria/state/snapshot.json"),
            DontTouchDecision::Block {
                matched_pattern: ".aria/state/**".to_string(),
            }
        );
        assert_eq!(
            e.evaluate(".aria/state/sub/dir/file.json"),
            DontTouchDecision::Block {
                matched_pattern: ".aria/state/**".to_string(),
            }
        );
        assert_eq!(e.evaluate(".aria/skills/x.md"), DontTouchDecision::Allow);
    }

    #[test]
    fn multi_glob_first_match_wins() {
        // Both patterns match `.env.production` — first-by-index returns.
        let e = DontTouchEvaluator::new([".env*", "**/*.production"]).expect("ok");
        match e.evaluate(".env.production") {
            DontTouchDecision::Block { matched_pattern } => {
                assert_eq!(matched_pattern, ".env*");
            }
            other @ DontTouchDecision::Allow => panic!("expected block, got {other:?}"),
        }
    }

    #[test]
    fn case_insensitive_matching_on_windows_style_paths() {
        let e = DontTouchEvaluator::new(["package-lock.json"]).expect("ok");
        assert!(matches!(
            e.evaluate("Package-Lock.json"),
            DontTouchDecision::Block { .. }
        ));
        assert!(matches!(
            e.evaluate("PACKAGE-LOCK.JSON"),
            DontTouchDecision::Block { .. }
        ));
    }

    #[test]
    fn invalid_glob_returns_invalid_error_with_index() {
        let err = DontTouchEvaluator::new(["ok.rs", "[unbalanced"]).expect_err("invalid");
        match err {
            DontTouchError::Invalid { index, .. } => assert_eq!(index, 1),
        }
    }

    #[test]
    fn typical_aria_dont_touch_set_blocks_expected_paths() {
        // Spec §4a example. Lock the behavior the framework JSON sample
        // depends on.
        let e =
            DontTouchEvaluator::new([".aria/state/**", "package-lock.json", ".env*"]).expect("ok");
        assert!(matches!(
            e.evaluate(".aria/state/foo"),
            DontTouchDecision::Block { .. }
        ));
        assert!(matches!(
            e.evaluate("package-lock.json"),
            DontTouchDecision::Block { .. }
        ));
        assert!(matches!(
            e.evaluate(".env.local"),
            DontTouchDecision::Block { .. }
        ));
        assert_eq!(e.evaluate("src/main.rs"), DontTouchDecision::Allow);
    }
}
