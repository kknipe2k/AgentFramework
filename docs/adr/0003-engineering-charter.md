# ADR-0003: Engineering Charter Adoption (Process > Language)

**Status:** Accepted
**Date:** 2026-04-18
**Deciders:** @kknipe2k (with Claude analysis and recommendation)
**Tags:** quality, oss, ci, process

## Context

Open-sourcing a project changes the inputs to "code quality":

- Strangers will contribute. Their tooling, conventions, and experience levels vary widely.
- The codebase will be scrutinized harder than internal code (publicly visible bugs are reputational, not just functional).
- Sustainability matters more — a one-person passion project has different load than a multi-contributor project.
- "Don't ship junk code" was raised by the user as a project-level concern: how do we keep quality high regardless of contributor skill?

Honest answer: language choice (Rust + TypeScript per ADR-0002) catches a class of bugs at compile time, but **process gates** are what prevent the bulk of junk code. Static analysis stops what static analysis can stop; tests, reviews, doc requirements, dependency audits, ADRs, and release rigor stop the rest.

A specification-level engineering policy was needed before the first line of code, so the contracts are clear from day one and not negotiated PR-by-PR.

## Decision

We adopt the **Engineering Charter** as documented in `agent-runtime-spec.md` §12. Summarized:

1. **Test rigor:** ≥80% line coverage on all new code; **100% on safety primitives** (drone, capability enforcer, plan state machine, snapshot/recovery). Property tests for all public state machines + serde round-trips. Fuzz harnesses for parsers (framework JSON, capability declarations, signal codec, IPC frame codec).
2. **Type strictness:** `deny(warnings)` workspace-wide. `clippy::pedantic` + `clippy::nursery`. `forbid(unsafe_code)` everywhere except `runtime-sandbox` (with mandatory `// SAFETY:` comments). TS `strict: true`; no `any`; ts-ignore requires linked issue.
3. **Lint and format:** `cargo fmt`, `cargo clippy -- -D warnings`, `cargo deny check`, `prettier`, `eslint` all blocking PRs. Pre-commit hook runs them locally; CI mirror prevents `--no-verify` bypass.
4. **Code review:** branch protection on `main`. 2-maintainer approval for core; CODEOWNERS auto-assigns security-track reviewers for sensitive paths. Squash-merge only. No force-push to `main`. PR-must-link-ADR for §0a primitive changes.
5. **Dependency hygiene:** `cargo audit` + `npm audit` blocking high/critical. `renovate` for weekly upgrades. `cargo deny` policy denies GPL/AGPL deps, duplicate major versions, unmaintained crates. SBOM (CycloneDX) per release.
6. **Documentation:** `cargo doc --no-deps -- -D rustdoc::missing_docs`; doc tests must compile (`cargo test --doc`); every `pub` API has at least one example. ADRs immutable; superseded ones link to successors.
7. **Versioning and release:** strict SemVer. Schemas in `schemas/` are the source of truth — Rust and TS types are generated, hand-written types are forbidden. Conventional Commits enforced via `commitlint`. `release-please` automates changelog. Sigstore-signed reproducible releases with SLSA Level 3 provenance from v1.0.
8. **Security disclosure:** SECURITY.md with private channel + 90-day embargo default. Threat model in `docs/SECURITY.md`. CVEs via GitHub Security Advisories. Severity-tagged in changelog.
9. **License and CLA:** Apache 2.0 (patent grant for AI tooling). DCO sign-off (lower friction than CLA, adequate IP hygiene).
10. **Contributor experience:** CONTRIBUTING.md walks clone → setup → first PR. `.devcontainer/` with all toolchains. Issue + PR templates. Maintainer onboarding doc.
11. **ADRs required for:** §0a Capability Matrix changes, schema changes, new `LLMProvider` impls, capability enforcement changes, IPC protocol changes, new core dependencies. Smaller decisions (refactors, internal abstractions) don't require an ADR.
12. **CI matrix:** Linux/macOS/Windows × stable/MSRV. Each cell runs fmt-check, clippy, test, doc-test, build, e2e (smoke). Nightly extends with extended fuzz, audit, dependency review.
13. **Observability of quality:** `docs/QUALITY.md` auto-generated weekly with coverage, advisories, test count, fuzz hours, CVE-fix-time SLO compliance. Public read of project health.

## Consequences

### Positive
- Quality bar is encoded in CI, not tribal knowledge. New contributors don't need to read 100 PRs to learn "what's good here" — the gates tell them.
- Coverage thresholds + 100% on safety primitives mean a regression in the drone, capability enforcer, or plan state machine fails CI before review.
- Schemas-as-source-of-truth + generated types prevents the "drift between TS and Rust types" failure mode that bit other multi-language projects.
- Apache 2.0 + DCO is the lowest-friction OSS posture that still gets the patent grant. CLAs deter contributors; we don't need one.
- Provenance + SBOM + signed releases address supply-chain attack vectors that have hit other agentic projects (chains of compromised npm deps, etc.).
- ADR requirement for primitive changes prevents scope creep AND captures rationale durably. Years from now, "why did we choose tier-3 when tier-2 would work?" has a written answer.

### Negative
- More upfront ceremony than a typical hobby project. First few PRs take longer to land.
- Coverage gates can encourage shallow tests if not paired with property/fuzz testing. Mitigated by explicit coverage-quality requirements (property tests for state machines).
- Renovate noise — weekly upgrade PRs create review load. Mitigated by grouping semver-minor in one PR.
- 2-maintainer approval requirement is hard to satisfy with one maintainer (the current state). Until a second maintainer is named, "2-maintainer approval" pragmatically becomes "1-maintainer approval + 24-hour cooling-off period for non-trivial changes."
- AI-assistance disclosure requirement may surface the fact that early contributions are largely Claude-written. This is an honest trade-off — we'd rather be transparent about AI usage than pretend.

### Neutral / future implications
- The "1-maintainer pragmatic compromise" gets revisited when a second maintainer joins. Until then, the spirit of "extra eyes" is maintained via the cooling-off period.
- `docs/QUALITY.md` is a v1.0+ artifact; v0.1 ships with manual quality reporting in release notes.
- Some gates (Sigstore signing, SLSA L3 provenance) are themselves v1.0+. v0.1 has a code-signing certificate but not the full Sigstore + SLSA pipeline.

## Alternatives Considered

### Alternative A: Minimal gates, trust contributors
Apache 2.0 + a basic CI with `cargo test`. Let reviewers catch issues.

**Rejected because:** doesn't scale past one or two contributors. "Trust contributors" works in tight teams; in OSS it means inconsistent quality, drive-by bug reports asking why the project doesn't work, and slow erosion of contributor confidence.

### Alternative B: Full corporate-process (CLA, DCO, multi-tier review, RFC process)
Highest formality. Best for huge multi-stakeholder projects.

**Rejected because:** kills contribution at this scale. CLAs deter contributors disproportionately to the IP protection they offer for an Apache 2.0 project. RFC processes are heavy; ADRs are lighter and accomplish the same intent.

### Alternative C: GPL / AGPL license with stronger copyleft
Forces downstream to keep the runtime and modifications open source.

**Rejected because:** AGPL is incompatible with many corporate contributors and downstream uses. Apache 2.0 with a patent grant balances openness with adoption. AGPL is a values choice that's defensible but deters the contributor pool we want.

### Alternative D: No coverage threshold; rely on review
Skip the 80%/100% coverage requirement. Reviewers eyeball.

**Rejected because:** coverage gates catch the "PR adds 200 lines of new logic with zero new tests" pattern automatically. Reviewers shouldn't have to manually compute coverage delta.

## Related

- Spec section: §12 Engineering Charter (the canonical text)
- Spec section: §13 Privacy & Telemetry (related no-telemetry policy enforced by `cargo deny`)
- Configuration files: `.github/workflows/ci.yml` (CI gates), `.github/workflows/release.yml` (release pipeline), `.github/CODEOWNERS`, `CONTRIBUTING.md` (contributor-facing process), `SECURITY.md` (disclosure flow), `LICENSE` (Apache 2.0), `NOTICE`, `CHANGELOG.md` (Keep-a-Changelog format)
- ADR-0001: ARIA as Archetype (positioning that made OSS the path)
- ADR-0002: Tauri + Rust over Electron (stack choice consistent with this charter's quality posture)

## Notes

This ADR is the durable artifact for the engineering-process posture committed to in §12. The charter itself is written in the spec because contributors will encounter it there first; this ADR documents the decision and rationale separately.

When a §12 rule is loosened or tightened in the future, file a successor ADR rather than editing this one or §12 silently. Examples of changes that would warrant a successor:
- Lowering coverage thresholds (e.g., from 80% to 70%)
- Adopting a CLA in addition to or instead of DCO
- Switching license (e.g., to AGPL — would also require 100% contributor consent)
- Removing the ADR-required-for-primitive-changes rule

The 1-maintainer pragmatic compromise (24-hour cooling-off for non-trivial changes when only one maintainer is available) is documented here as a transitional state, not a permanent rule. It expires when a second maintainer is named.
