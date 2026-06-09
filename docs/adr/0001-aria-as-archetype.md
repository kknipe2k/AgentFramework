# ADR-0001: ARIA as Archetype, Not Built-In Default

**Status:** Accepted — positioning **amended by ADR-0033** (2026-06-09: ARIA demoted from raison-d'être/headline MVP success criterion to a conformance suite; the technical decision below — generic platform, ARIA reconstructed-not-bundled, zero built-in frameworks — is unchanged).
**Date:** 2026-04-18
**Deciders:** @kknipe2k (with Claude review and recommendation)
**Tags:** scope, positioning, schema

## Context

The repository already contained the **shell-based ARIA framework** (`.aria/`, ~13K LOC of shell + Python: engine, skills, Ralph loop, dashboard, offline RL, hooks, signal schema v2, model selector). The new `agent-runtime-spec.md` proposed a Tauri-based desktop runtime for agentic AI workflows.

The original spec said *"Aria ships as the built-in default framework"*, implying the runtime would bundle ARIA's behavior as a built-in. This conflicted with the intent that the runtime be a **generic agent-building, maintenance, and runtime-management platform**.

Three positioning options were on the table:
- **Replace:** delete `.aria/`, port skills/rails/verify/etc. to TypeScript or Rust. Highest risk; loses ~9 months of shell tooling.
- **Wrapper:** runtime shells out to existing `.aria/verify.sh`, `.aria/ralph/ralph.sh`, etc. Pragmatic but ties product to shell-script semantics indefinitely.
- **Archetype:** runtime is a generic platform; ARIA is the canonical example reconstructed inside it. Existing `.aria/` stays as reference material.

A decision was needed before downstream design choices (framework JSON schema, what primitives to ship, what's in MVP scope, what the example artifacts demonstrate) could land coherently.

## Decision

We adopt the **Archetype** model.

The runtime is a **generic platform**. It ships **primitives** (drone, event pipeline, live graph, plan/task model, hooks, rails, mode field, HITL, registry, builder canvas, generators) — not opinionated agent workflows. The runtime ships zero built-in frameworks.

ARIA is the **reference framework**. It is recreated inside the runtime as `examples/aria/` — a framework JSON plus bundled skills/agents/tools that demonstrate every row of the §0a Capability Matrix is reconstructible using only the runtime's primitives. The existing `.aria/` shell codebase remains untouched as reference material; users who prefer the shell experience continue to use it.

**MVP success criterion** (locked in §0d): a user can reconstruct every row of the §0a Capability Matrix inside the runtime using only framework JSON and primitives, **without modifying runtime source.** `examples/aria/` is the executable proof of this criterion.

## Consequences

### Positive
- The runtime is a real platform, not "ARIA-the-app." Users with different agentic patterns (Ralph-style continuous loop, research workflows, evaluation harnesses) can express their patterns by composing the same primitives differently. `examples/ralph/` demonstrates this with a sibling framework.
- Existing `.aria/` users are unaffected. No migration pressure.
- The capability matrix becomes a contract: if a row can't be reconstructed, the runtime is missing a primitive, not just an opinionated default.
- Schema validation, generators, and the builder canvas all operate against the same primitives — there's no "built-in special case" to maintain.

### Negative
- More upfront design work to identify and ship the right primitives. Each primitive must be general enough to express ARIA's behavior and any other framework's plausible behavior.
- Users who want "load ARIA and run" don't get a one-click experience — they must explicitly choose `examples/aria/` as their starting framework. (Mitigated in §14 First-Run UX where ARIA is presented as the recommended starting template.)
- Primitives that turn out to be ARIA-specific need refactoring. Risk of leaking ARIA-isms into the primitive layer if not careful.

### Neutral / future implications
- The shell `.aria/` codebase will eventually move to `archive/aria-shell/` once v0.1 of the runtime ships. Until then, both coexist at the repo root.
- Adding a new framework (e.g., `examples/research/` for research workflows) is a v1.1+ task and does not gate v0.1.
- The `.aria/docs/` reference material stays as the source of truth for what an agentic system should do; the runtime spec stays as the source of truth for how to build one.

## Alternatives Considered

### Alternative A: Replace
Port all of `.aria/` to Rust/TypeScript. Eliminate the shell dependency.

**Rejected because:** ~6–9 months of work just to reach the existing shell-ARIA capability. Doesn't add value to a runtime that should be a platform anyway. Delays MVP indefinitely.

### Alternative B: Wrapper
Runtime spawns shell scripts under the hood. Framework JSON references shell entry-points. Electron is the face; ARIA-the-shell is the engine.

**Rejected because:** ties the runtime forever to shell-script semantics. Loses portability (Windows runs shell only via Git Bash / WSL). Doesn't expose primitives in a way other frameworks can use. Wrong abstraction — framework authors shouldn't need to know what `verify.sh` is.

### Alternative C: Multi-product README
Treat the repo as a multi-product: shell ARIA at `.aria/`, runtime at `crates/`, shared docs in the middle.

**Rejected because:** confusing to users (which is the product?), and the runtime is intended to subsume the shell experience for users who want it. The Archetype model lets both coexist without forcing a choice.

## Related

- Spec section: §0 Project Positioning & Relationship to ARIA
- Spec section: §0a ARIA Archetype Capability Matrix (the contract)
- Spec section: §0d Release Scope Matrix (gates v0.1 ship on §0a row reconstruction)
- Reference frameworks: `examples/aria/` and `examples/ralph/`
- Substantive analysis: `.aria/docs/AGENT-RUNTIME-SPEC-REVIEW.md` (the review that surfaced the positioning ambiguity) and `.aria/docs/AGENT-RUNTIME-SPEC-REMEDIATION.md` (the work-item plan that drove this decision; WI-00 and WI-01)

## Notes

This ADR documents a decision made on 2026-04-18 during the review of the initial spec ideation commit (`2e36f5b Create agent-runtime-spec.md`). The remediation plan (`.aria/docs/AGENT-RUNTIME-SPEC-REMEDIATION.md`) tracks 32 work items that fell out of this and subsequent decisions; this ADR is the durable artifact for the relationship-to-existing-ARIA decision specifically.
