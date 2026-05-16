//! §5a Tool Namespace Resolution — M06.D behavior tests.
//!
//! Each of the five locked §5a rules (canonical, short, alias,
//! re-resolution, server-name `__` constraint / first-split) is pinned
//! by a named test. Multi-call invariant per gotcha #69.

use std::collections::BTreeMap;

use runtime_mcp::namespace::{AliasError, Aliases};
use runtime_mcp::{NamespaceError, NamespaceResolver, ResolvedTool};

fn connected(pairs: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
    pairs
        .iter()
        .map(|(s, tools)| {
            (
                (*s).to_string(),
                tools.iter().map(|t| (*t).to_string()).collect(),
            )
        })
        .collect()
}

const fn no_aliases() -> BTreeMap<String, String> {
    BTreeMap::new()
}

// ── §5a rule 1: canonical `<server>__<tool>` ──────────────────────────

#[test]
fn resolve_canonical_succeeds_when_server_and_tool_match() {
    let r = NamespaceResolver::new(connected(&[("pdf-mcp", &["extract_text"])]));
    let got = r
        .resolve("pdf-mcp__extract_text", &no_aliases())
        .expect("canonical name for a connected server+tool resolves");
    assert_eq!(
        got,
        ResolvedTool {
            server: "pdf-mcp".to_string(),
            tool: "extract_text".to_string(),
        }
    );
}

#[test]
fn resolve_canonical_fails_when_server_not_connected() {
    let r = NamespaceResolver::new(connected(&[("pdf-mcp", &["extract_text"])]));
    let err = r
        .resolve("image-mcp__extract_text", &no_aliases())
        .expect_err("canonical name for an UNconnected server must not resolve");
    assert_eq!(
        err,
        NamespaceError::NotFound("image-mcp__extract_text".to_string())
    );
}

// ── §5a rule 4: server names cannot contain `__`; split on FIRST `__`
//    so tool names MAY contain `__` ───────────────────────────────────

#[test]
fn resolve_canonical_splits_on_first_double_underscore_so_tool_may_contain_it() {
    // Tool name itself contains `__`. The parser splits on the FIRST
    // `__` from the left: server = "git-mcp", tool = "log__oneline".
    let r = NamespaceResolver::new(connected(&[("git-mcp", &["log__oneline"])]));
    let got = r
        .resolve("git-mcp__log__oneline", &no_aliases())
        .expect("split on first __ keeps the rest as the tool name");
    assert_eq!(
        got,
        ResolvedTool {
            server: "git-mcp".to_string(),
            tool: "log__oneline".to_string(),
        }
    );
}

// ── §5a rule 2: short name iff unambiguous ────────────────────────────

#[test]
fn resolve_short_name_succeeds_when_unambiguous() {
    let r = NamespaceResolver::new(connected(&[
        ("pdf-mcp", &["extract_text"]),
        ("git-mcp", &["log"]),
    ]));
    let got = r
        .resolve("extract_text", &no_aliases())
        .expect("unambiguous short name resolves");
    assert_eq!(got.server, "pdf-mcp");
    assert_eq!(got.tool, "extract_text");
}

#[test]
fn resolve_short_name_fails_when_ambiguous_returns_candidate_list() {
    // Spec §5a example: pdf-mcp + image-mcp both expose extract_text.
    let r = NamespaceResolver::new(connected(&[
        ("pdf-mcp", &["extract_text"]),
        ("image-mcp", &["extract_text"]),
    ]));
    let err = r
        .resolve("extract_text", &no_aliases())
        .expect_err("ambiguous short name must fail");
    match err {
        NamespaceError::Ambiguous {
            name,
            mut candidates,
        } => {
            assert_eq!(name, "extract_text");
            candidates.sort();
            assert_eq!(
                candidates,
                vec![
                    "image-mcp__extract_text".to_string(),
                    "pdf-mcp__extract_text".to_string(),
                ]
            );
        }
        other => panic!("expected Ambiguous, got {other:?}"),
    }
}

#[test]
fn resolve_short_name_fails_when_not_found() {
    let r = NamespaceResolver::new(connected(&[("pdf-mcp", &["extract_text"])]));
    let err = r
        .resolve("nonexistent", &no_aliases())
        .expect_err("unknown short name must fail");
    assert_eq!(err, NamespaceError::NotFound("nonexistent".to_string()));
}

// ── §5a rule 3: explicit `mcp_aliases` override ───────────────────────

#[test]
fn resolve_alias_succeeds_when_alias_maps_to_valid_canonical() {
    // Two servers expose extract_text (ambiguous); the framework pins
    // the short name to pdf-mcp via mcp_aliases. The alias overrides
    // the ambiguity error.
    let r = NamespaceResolver::new(connected(&[
        ("pdf-mcp", &["extract_text"]),
        ("image-mcp", &["extract_text"]),
    ]));
    let mut aliases = BTreeMap::new();
    aliases.insert(
        "extract_text".to_string(),
        "pdf-mcp__extract_text".to_string(),
    );
    let got = r
        .resolve("extract_text", &aliases)
        .expect("alias overrides short-name ambiguity");
    assert_eq!(got.server, "pdf-mcp");
    assert_eq!(got.tool, "extract_text");
}

#[test]
fn resolve_alias_fails_when_alias_maps_to_unknown_canonical() {
    let r = NamespaceResolver::new(connected(&[("pdf-mcp", &["extract_text"])]));
    let mut aliases = BTreeMap::new();
    aliases.insert(
        "extract_text".to_string(),
        "ghost-mcp__extract_text".to_string(),
    );
    let err = r
        .resolve("extract_text", &aliases)
        .expect_err("alias pointing at an unconnected canonical must fail");
    assert_eq!(
        err,
        NamespaceError::UnknownAlias(
            "extract_text".to_string(),
            "ghost-mcp__extract_text".to_string(),
        )
    );
}

// ── §5a rule 5: re-resolution on connect/disconnect ───────────────────

#[test]
fn re_evaluate_short_names_emits_new_ambiguity_when_server_connects_with_overlapping_tool() {
    // pdf-mcp connected first (extract_text unambiguous). image-mcp
    // connects, also exposing extract_text → the short name BECOMES
    // ambiguous → connect_server returns a NewAmbiguity for it.
    let mut r = NamespaceResolver::new(connected(&[("pdf-mcp", &["extract_text"])]));
    let new_ambiguities = r.connect_server("image-mcp", vec!["extract_text".to_string()]);
    assert_eq!(
        new_ambiguities.len(),
        1,
        "exactly one short name became ambiguous"
    );
    let amb = &new_ambiguities[0];
    assert_eq!(amb.short_name, "extract_text");
    let mut cands = amb.candidates.clone();
    cands.sort();
    assert_eq!(
        cands,
        vec![
            "image-mcp__extract_text".to_string(),
            "pdf-mcp__extract_text".to_string(),
        ]
    );
    // And the resolver state reflects the connect: the short name now
    // fails as ambiguous.
    assert!(matches!(
        r.resolve("extract_text", &no_aliases()),
        Err(NamespaceError::Ambiguous { .. })
    ));
}

#[test]
fn re_evaluate_short_names_returns_empty_when_no_new_ambiguity() {
    // A server connecting with a tool no other server exposes creates
    // no new ambiguity.
    let mut r = NamespaceResolver::new(connected(&[("pdf-mcp", &["extract_text"])]));
    let new_ambiguities = r.connect_server("git-mcp", vec!["log".to_string()]);
    assert!(
        new_ambiguities.is_empty(),
        "non-overlapping connect creates no new ambiguity, got {new_ambiguities:?}"
    );
    // Both short names still resolve unambiguously.
    assert_eq!(
        r.resolve("extract_text", &no_aliases()).unwrap().server,
        "pdf-mcp"
    );
    assert_eq!(r.resolve("log", &no_aliases()).unwrap().server, "git-mcp");
}

// ── Multi-call invariant (gotcha #69) ─────────────────────────────────

#[test]
fn resolve_twice_in_sequence_both_succeed() {
    let r = NamespaceResolver::new(connected(&[("pdf-mcp", &["extract_text"])]));
    let first = r
        .resolve("pdf-mcp__extract_text", &no_aliases())
        .expect("first resolve");
    let second = r
        .resolve("pdf-mcp__extract_text", &no_aliases())
        .expect("second resolve must not be perturbed by the first");
    assert_eq!(first, second);
}

// ── Aliases wrapper validation (module ≥95% coverage) ─────────────────

#[test]
fn aliases_validate_rejects_non_canonical_value() {
    let mut m = BTreeMap::new();
    m.insert("extract_text".to_string(), "not-canonical".to_string());
    let err = Aliases::new(m)
        .validate()
        .expect_err("a value without `__` is not canonical");
    assert!(matches!(err, AliasError::NotCanonical { .. }));
}

#[test]
fn aliases_validate_rejects_collision_on_same_canonical() {
    let mut m = BTreeMap::new();
    m.insert("a".to_string(), "pdf-mcp__extract_text".to_string());
    m.insert("b".to_string(), "pdf-mcp__extract_text".to_string());
    let err = Aliases::new(m)
        .validate()
        .expect_err("two short names mapping to the same canonical collide");
    assert!(matches!(err, AliasError::Collision { .. }));
}

#[test]
fn aliases_validate_accepts_well_formed_distinct_map() {
    let mut m = BTreeMap::new();
    m.insert(
        "extract_text".to_string(),
        "pdf-mcp__extract_text".to_string(),
    );
    m.insert("render".to_string(), "image-mcp__render".to_string());
    let aliases = Aliases::new(m.clone());
    aliases
        .validate()
        .expect("well-formed distinct map validates");
    assert_eq!(aliases.as_map(), &m);
}
