//! L2a narrowing — spec §8.security L2a (M05 Stage B).
//!
//! When an agent spawns a sub-agent with capabilities, the child's
//! grants MUST be a subset of the parent's. [`narrow`] enforces this:
//! given `parent_grants` + `proposed_child_grants`, returns the
//! (possibly clamped) child grants when every proposed declaration is
//! subsumed by some parent grant; errors otherwise.
//!
//! The narrowing invariant is the load-bearing safety property:
//!
//! ```text
//! ∀ c ∈ child : ∃ p ∈ parent : p.subsumes(c)
//! ```
//!
//! Proptest verifies this asymmetric invariant — flipping the direction
//! (using `child.subsumes(parent)` rather than `parent.subsumes(child)`)
//! would let children widen parent grants (gotcha trap #2 from M05.B
//! stage prompt).

use runtime_core::generated::capability::CapabilityDeclaration;

use crate::capability::declaration::subsumes;
use crate::capability::error::NarrowingError;

/// Verify that `proposed` is a subset of `parent`'s capabilities.
///
/// Returns `Ok(proposed.to_vec())` when every proposed declaration is
/// subsumed by at least one parent grant. Returns
/// [`NarrowingError::CapabilityNotHeldByParent`] on the first proposed
/// declaration the parent's grants do not cover — short-circuits because
/// L2a narrowing is "all-or-nothing" (the spec doesn't define a
/// partial-clamp semantics for v0.1).
///
/// # Errors
///
/// - [`NarrowingError::CapabilityNotHeldByParent`] when any proposed
///   capability is not subsumed by any parent grant.
pub fn narrow(
    parent: &[CapabilityDeclaration],
    proposed: &[CapabilityDeclaration],
) -> Result<Vec<CapabilityDeclaration>, NarrowingError> {
    for prop in proposed {
        if !parent.iter().any(|p| subsumes(p, prop)) {
            return Err(NarrowingError::CapabilityNotHeldByParent {
                proposed: Box::new(prop.clone()),
            });
        }
    }
    Ok(proposed.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::collection::vec as prop_vec;
    use proptest::prelude::*;
    use runtime_core::generated::capability::{
        CapabilityDeclaration, CapabilityKind, CapabilityScope, GlobPattern, PathPattern,
        ResourceName, SideEffectClass,
    };
    use std::str::FromStr;

    fn read_src_glob() -> CapabilityDeclaration {
        CapabilityDeclaration {
            kind: CapabilityKind::Read,
            resource: ResourceName::from_str("src").unwrap(),
            scope: CapabilityScope::Glob(GlobPattern::from_str("src/**").unwrap()),
            side_effect_class: SideEffectClass::Pure,
        }
    }

    fn read_src_path(p: &str) -> CapabilityDeclaration {
        CapabilityDeclaration {
            kind: CapabilityKind::Read,
            resource: ResourceName::from_str("src").unwrap(),
            scope: CapabilityScope::Path(PathPattern::from_str(p).unwrap()),
            side_effect_class: SideEffectClass::Pure,
        }
    }

    fn write_src_glob() -> CapabilityDeclaration {
        CapabilityDeclaration {
            kind: CapabilityKind::Write,
            resource: ResourceName::from_str("src").unwrap(),
            scope: CapabilityScope::Glob(GlobPattern::from_str("src/**").unwrap()),
            side_effect_class: SideEffectClass::FilesystemMutate,
        }
    }

    #[test]
    fn child_identical_to_parent_ok() {
        let parent = vec![read_src_glob()];
        let proposed = vec![read_src_glob()];
        let narrowed = narrow(&parent, &proposed).expect("identical narrows OK");
        assert_eq!(narrowed.len(), 1);
    }

    #[test]
    fn child_subset_of_parent_ok() {
        // Parent: read `src/**` glob.
        // Child: read `src/lib/foo.rs` path — inside the glob.
        let parent = vec![read_src_glob()];
        let proposed = vec![read_src_path("src/lib/foo.rs")];
        let narrowed = narrow(&parent, &proposed).expect("subset narrows OK");
        assert_eq!(narrowed.len(), 1);
    }

    #[test]
    fn child_widening_scope_denied() {
        // Parent: read `src/lib/foo.rs` (narrow path).
        // Child: read `src/**` (wider glob). Denied.
        let parent = vec![read_src_path("src/lib/foo.rs")];
        let proposed = vec![read_src_glob()];
        let err = narrow(&parent, &proposed).expect_err("widening denied");
        match err {
            NarrowingError::CapabilityNotHeldByParent { .. } => {}
        }
    }

    #[test]
    fn child_capability_parent_lacks_denied() {
        // Parent: read.
        // Child: write — different kind. Parent doesn't have this kind
        // at all, so denied.
        let parent = vec![read_src_glob()];
        let proposed = vec![write_src_glob()];
        let err = narrow(&parent, &proposed).expect_err("missing kind denied");
        match err {
            NarrowingError::CapabilityNotHeldByParent { proposed: p } => {
                assert_eq!(p.kind, CapabilityKind::Write);
            }
        }
    }

    #[test]
    fn empty_proposed_always_ok() {
        // A child with no capabilities is trivially a subset.
        let parent = vec![read_src_glob()];
        let proposed: Vec<CapabilityDeclaration> = Vec::new();
        let narrowed = narrow(&parent, &proposed).expect("empty narrows OK");
        assert!(narrowed.is_empty());
    }

    #[test]
    fn proposed_with_multiple_caps_one_invalid_short_circuits_at_first() {
        // The error names the FIRST failing declaration, not the last.
        let parent = vec![read_src_glob()];
        let bad_first = write_src_glob();
        let ok_second = read_src_path("src/foo.rs");
        let proposed = vec![bad_first, ok_second];
        let err = narrow(&parent, &proposed).expect_err("first failure short-circuits");
        match err {
            NarrowingError::CapabilityNotHeldByParent { proposed: p } => {
                assert_eq!(p.kind, CapabilityKind::Write);
            }
        }
    }

    #[test]
    fn parent_with_multiple_grants_child_only_uses_subset() {
        // Parent has both read and write grants; child claims only read.
        // OK — the second parent grant matches.
        let parent = vec![write_src_glob(), read_src_glob()];
        let proposed = vec![read_src_glob()];
        let narrowed = narrow(&parent, &proposed).expect("subset of multi-grant parent");
        assert_eq!(narrowed.len(), 1);
    }

    // ── Property test (gotcha trap #2 — asymmetric direction) ─────────

    /// Strategy: generate a small set of "primitive" declarations the
    /// proptest can sample from to build parent + child vecs. Bounded
    /// universe keeps the property's claim falsifiable in practice
    /// (proptest needs enough overlap to test both Ok and Err paths).
    fn primitive_declarations() -> Vec<CapabilityDeclaration> {
        vec![
            // 1: read src/** (glob)
            read_src_glob(),
            // 2: read src/lib/foo.rs (path inside src/**)
            read_src_path("src/lib/foo.rs"),
            // 3: read src/main.rs (different path inside src/**)
            read_src_path("src/main.rs"),
            // 4: write src/** (different kind/class)
            write_src_glob(),
            // 5: read docs (outside src)
            CapabilityDeclaration {
                kind: CapabilityKind::Read,
                resource: ResourceName::from_str("docs").unwrap(),
                scope: CapabilityScope::Glob(GlobPattern::from_str("docs/**").unwrap()),
                side_effect_class: SideEffectClass::Pure,
            },
        ]
    }

    proptest! {
        /// THE narrowing invariant: when narrow returns Ok, every
        /// returned child declaration must be subsumed by some parent
        /// declaration. Asymmetric direction — flipping subsumes()
        /// argument order would falsify this property whenever child
        /// widens parent.
        #[test]
        fn property_narrowing_preserves_invariant(
            parent_idxs in prop_vec(0usize..5, 0..5),
            proposed_idxs in prop_vec(0usize..5, 0..5),
        ) {
            let universe = primitive_declarations();
            let parent: Vec<CapabilityDeclaration> =
                parent_idxs.iter().map(|&i| universe[i].clone()).collect();
            let proposed: Vec<CapabilityDeclaration> =
                proposed_idxs.iter().map(|&i| universe[i].clone()).collect();

            match narrow(&parent, &proposed) {
                Ok(narrowed) => {
                    // For every narrowed child, SOME parent must subsume.
                    for child in &narrowed {
                        let covered = parent.iter().any(|p| subsumes(p, child));
                        prop_assert!(
                            covered,
                            "narrowing returned Ok but child {child:?} not subsumed by any parent {parent:?}"
                        );
                    }
                    // Length is preserved (v0.1 narrow is all-or-nothing).
                    prop_assert_eq!(narrowed.len(), proposed.len());
                }
                Err(NarrowingError::CapabilityNotHeldByParent { proposed: bad }) => {
                    // The offending declaration must indeed NOT be
                    // subsumed by any parent.
                    let covered = parent.iter().any(|p| subsumes(p, &bad));
                    prop_assert!(
                        !covered,
                        "narrowing returned Err but {bad:?} IS subsumed by some parent {parent:?}"
                    );
                }
            }
        }

        /// Direction asymmetry: when narrower does NOT subsume wider,
        /// offering wider against parent=[narrower] MUST err. Mirror
        /// of the main invariant; the `subsumes` predicate is the
        /// authoritative comparator (CapabilityDeclaration has no
        /// PartialEq because typify's oneOf-wrapping CapabilityScope
        /// derives only Clone + Debug).
        #[test]
        fn property_widening_always_denied(
            wider_idx in 0usize..5,
            narrower_idx in 0usize..5,
        ) {
            let universe = primitive_declarations();
            let wider = &universe[wider_idx];
            let narrower = &universe[narrower_idx];

            // If narrower does NOT subsume wider, offering wider
            // against parent=[narrower] must err. The case
            // subsumes(narrower, wider) = true at distinct indices is
            // legitimate (two declarations may be equivalent under the
            // subsumes predicate) — the property below skips those.
            if !subsumes(narrower, wider) {
                let parent = vec![narrower.clone()];
                let proposed = vec![wider.clone()];
                let result = narrow(&parent, &proposed);
                prop_assert!(
                    result.is_err(),
                    "wider {wider:?} should be denied against narrower {narrower:?}"
                );
            }
        }
    }
}
