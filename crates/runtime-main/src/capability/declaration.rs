//! Pure-function predicates over [`CapabilityDeclaration`] — spec §8.security
//! L1.
//!
//! Two predicates form the L1+L2a contract:
//!
//! - [`subsumes`] — does `parent` cover everything `requested` asks for?
//!   The enforcer's `check` walks the agent's grants looking for any
//!   `subsumes(grant, requested)` match; the narrowing evaluator's
//!   contract is `proposed_child ⊆ parent_grants` for every proposed
//!   declaration. Asymmetric — flipping argument order would let
//!   children widen parent grants (gotcha trap #2).
//!
//! - [`scope_contains`] — does `outer` scope cover `inner`? Per-variant
//!   logic: globs match via `globset::Glob`; domains via host or
//!   suffix; paths via prefix-with-separator. Cross-variant containment
//!   is asymmetric — a glob may subsume a path if the path matches the
//!   glob; a path never subsumes a glob (gotcha trap #3).
//!
//! Free functions rather than inherent methods because typify-generated
//! types cannot be extended outside the generating crate.

use runtime_core::generated::capability::{CapabilityDeclaration, CapabilityScope};

/// Does `parent`'s grant cover the entire surface that `requested`
/// declares?
///
/// Returns `true` when ALL of:
/// - `parent.kind == requested.kind` (same coarse category)
/// - `parent.side_effect_class == requested.side_effect_class`
/// - `parent.resource == requested.resource` (same target identifier)
/// - `scope_contains(&parent.scope, &requested.scope)` (parent's scope
///   subsumes child's)
///
/// The kind + class + resource equality check is conservative — v0.1
/// enforces "exact same coarse triple"; future tier-aware code may
/// loosen (e.g. a `read` grant subsuming a `pure` reflection request).
/// For v0.1 keep it strict: violations should be obvious.
#[must_use]
pub fn subsumes(parent: &CapabilityDeclaration, requested: &CapabilityDeclaration) -> bool {
    parent.kind == requested.kind
        && parent.side_effect_class == requested.side_effect_class
        && *parent.resource == *requested.resource
        && scope_contains(&parent.scope, &requested.scope)
}

/// Does `outer` scope cover everything `inner` scope describes?
///
/// Per-variant containment logic:
///
/// - `GlobScope(g)` contains `PathScope(p)` iff `g` matches `p`.
/// - `GlobScope(g)` contains `GlobScope(g')` iff `g == g'` (no glob
///   subset analysis in v0.1 — equality is the only safe shortcut).
/// - `DomainScope(d)` contains `DomainScope(d')` iff `d == d'` OR `d`
///   is `.example.com`-form and `d'` ends with `.example.com` or is
///   exactly `example.com`.
/// - `PathScope(p)` contains `PathScope(p')` iff `p == p'` OR `p'`
///   starts with `p + '/'` (prefix-with-separator; `src` covers
///   `src/foo` but NOT `src-other`).
/// - All other combinations return `false`.
#[must_use]
pub fn scope_contains(outer: &CapabilityScope, inner: &CapabilityScope) -> bool {
    match (outer, inner) {
        (CapabilityScope::Glob(g_outer), CapabilityScope::Glob(g_inner)) => {
            // v0.1 — equality only. A real glob-subset analysis would
            // require AST comparison of the pattern languages.
            **g_outer == **g_inner
        }
        (CapabilityScope::Glob(g), CapabilityScope::Path(p)) => glob_matches_path(g, p),
        (CapabilityScope::Domain(d_outer), CapabilityScope::Domain(d_inner)) => {
            domain_contains(d_outer, d_inner)
        }
        (CapabilityScope::Path(p_outer), CapabilityScope::Path(p_inner)) => {
            path_contains(p_outer, p_inner)
        }
        // All other combinations are intentionally non-containing.
        // Cross-kind: a network domain doesn't subsume a filesystem
        // path; a path doesn't subsume a glob (the inverse may hold).
        _ => false,
    }
}

fn glob_matches_path(glob_pattern: &str, candidate_path: &str) -> bool {
    // globset::Glob handles ** + per-segment patterns. Build-failure is
    // false (caller-supplied glob has minLength 1 from the schema; an
    // invalid pattern is a caller bug — surface as no-match rather than
    // panic).
    globset::Glob::new(glob_pattern).is_ok_and(|g| g.compile_matcher().is_match(candidate_path))
}

fn domain_contains(outer: &str, inner: &str) -> bool {
    if outer == inner {
        return true;
    }
    // `.example.com` covers `example.com` AND `*.example.com`.
    if let Some(suffix) = outer.strip_prefix('.') {
        return inner == suffix || inner.ends_with(outer);
    }
    false
}

fn path_contains(outer: &str, inner: &str) -> bool {
    if outer == inner {
        return true;
    }
    // Prefix-with-separator: `src` covers `src/foo` but NOT `src-other`.
    // Strip a single trailing separator from `outer` if present to
    // normalize.
    let outer = outer.trim_end_matches('/');
    inner.starts_with(outer)
        && inner
            .as_bytes()
            .get(outer.len())
            .is_some_and(|&b| b == b'/')
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_core::generated::capability::{
        CapabilityDeclaration, CapabilityKind, CapabilityScope, DomainPattern, GlobPattern,
        PathPattern, ResourceName, SideEffectClass,
    };
    use std::str::FromStr;

    fn decl(
        kind: CapabilityKind,
        resource: &str,
        scope: CapabilityScope,
        side: SideEffectClass,
    ) -> CapabilityDeclaration {
        CapabilityDeclaration {
            kind,
            resource: ResourceName::from_str(resource).expect("non-empty resource"),
            scope,
            side_effect_class: side,
        }
    }

    fn glob_scope(s: &str) -> CapabilityScope {
        CapabilityScope::Glob(GlobPattern::from_str(s).expect("non-empty glob"))
    }
    fn domain_scope(s: &str) -> CapabilityScope {
        CapabilityScope::Domain(DomainPattern::from_str(s).expect("non-empty domain"))
    }
    fn path_scope(s: &str) -> CapabilityScope {
        CapabilityScope::Path(PathPattern::from_str(s).expect("non-empty path"))
    }

    #[test]
    fn subsumes_identical_declarations_returns_true() {
        let parent = decl(
            CapabilityKind::Read,
            "src",
            glob_scope("src/**"),
            SideEffectClass::Pure,
        );
        let child = decl(
            CapabilityKind::Read,
            "src",
            glob_scope("src/**"),
            SideEffectClass::Pure,
        );
        assert!(subsumes(&parent, &child));
    }

    #[test]
    fn subsumes_different_kind_returns_false() {
        let parent = decl(
            CapabilityKind::Read,
            "src",
            glob_scope("src/**"),
            SideEffectClass::Pure,
        );
        let child = decl(
            CapabilityKind::Write,
            "src",
            glob_scope("src/**"),
            SideEffectClass::FilesystemMutate,
        );
        assert!(!subsumes(&parent, &child));
    }

    #[test]
    fn subsumes_different_side_effect_class_returns_false() {
        let parent = decl(
            CapabilityKind::Read,
            "src",
            glob_scope("src/**"),
            SideEffectClass::Pure,
        );
        let child = decl(
            CapabilityKind::Read,
            "src",
            glob_scope("src/**"),
            SideEffectClass::FilesystemMutate,
        );
        assert!(!subsumes(&parent, &child));
    }

    #[test]
    fn subsumes_different_resource_returns_false() {
        let parent = decl(
            CapabilityKind::Read,
            "src",
            glob_scope("src/**"),
            SideEffectClass::Pure,
        );
        let child = decl(
            CapabilityKind::Read,
            "docs",
            glob_scope("src/**"),
            SideEffectClass::Pure,
        );
        assert!(!subsumes(&parent, &child));
    }

    #[test]
    fn scope_contains_glob_matches_path_inside_glob() {
        let outer = glob_scope("src/**");
        let inner = path_scope("src/lib/foo.rs");
        assert!(scope_contains(&outer, &inner));
    }

    #[test]
    fn scope_contains_glob_rejects_path_outside_glob() {
        let outer = glob_scope("src/**");
        let inner = path_scope("docs/foo.md");
        assert!(!scope_contains(&outer, &inner));
    }

    #[test]
    fn scope_contains_path_subsumes_only_prefix_with_separator() {
        let outer = path_scope("src");
        let inside = path_scope("src/foo.rs");
        let sibling = path_scope("src-other/foo.rs");
        let equal = path_scope("src");
        assert!(scope_contains(&outer, &inside));
        assert!(!scope_contains(&outer, &sibling));
        assert!(scope_contains(&outer, &equal));
    }

    #[test]
    fn scope_contains_path_with_trailing_slash_normalized() {
        let outer = path_scope("src/");
        let inside = path_scope("src/foo.rs");
        assert!(scope_contains(&outer, &inside));
    }

    #[test]
    fn scope_contains_domain_exact_match() {
        let outer = domain_scope("api.example.com");
        let inner = domain_scope("api.example.com");
        assert!(scope_contains(&outer, &inner));
    }

    #[test]
    fn scope_contains_domain_subdomain_match_via_leading_dot() {
        // `.example.com` covers `example.com` AND `api.example.com`.
        let outer = domain_scope(".example.com");
        let bare = domain_scope("example.com");
        let sub = domain_scope("api.example.com");
        let unrelated = domain_scope("evil.com");
        assert!(scope_contains(&outer, &bare));
        assert!(scope_contains(&outer, &sub));
        assert!(!scope_contains(&outer, &unrelated));
    }

    #[test]
    fn scope_contains_domain_no_match_without_leading_dot() {
        // Bare host `example.com` does NOT cover `api.example.com`.
        let outer = domain_scope("example.com");
        let sub = domain_scope("api.example.com");
        assert!(!scope_contains(&outer, &sub));
    }

    #[test]
    fn scope_contains_cross_variant_path_does_not_subsume_glob() {
        // Reverse direction of glob-subsumes-path: a path scope must NOT
        // subsume a glob scope. Confirms asymmetric containment.
        let outer = path_scope("src/lib/foo.rs");
        let inner = glob_scope("src/**");
        assert!(!scope_contains(&outer, &inner));
    }

    #[test]
    fn scope_contains_glob_equals_glob() {
        let outer = glob_scope("src/**");
        let inner = glob_scope("src/**");
        let other = glob_scope("docs/**");
        assert!(scope_contains(&outer, &inner));
        assert!(!scope_contains(&outer, &other));
    }

    #[test]
    fn scope_contains_invalid_glob_returns_false_not_panic() {
        // globset returns Err for unbalanced brackets; we surface as
        // no-match rather than panicking.
        let outer = glob_scope("src/[unclosed");
        let inner = path_scope("src/foo");
        assert!(!scope_contains(&outer, &inner));
    }

    #[test]
    fn scope_contains_domain_vs_path_returns_false() {
        // Network domain doesn't subsume filesystem path.
        let outer = domain_scope("example.com");
        let inner = path_scope("src");
        assert!(!scope_contains(&outer, &inner));
    }
}
