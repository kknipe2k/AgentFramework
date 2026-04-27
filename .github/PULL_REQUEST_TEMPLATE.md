<!--
Thanks for the contribution. Please complete this template; PRs that
skip required sections will be asked to update before review.
-->

## What this PR does

<!-- One paragraph: what changed and why. -->

## Linked issue / ADR

<!--
Required for substantive changes. Reference an existing issue or ADR by number.
For changes touching §0a Capability Matrix primitives, schemas/, capability
enforcement, IPC protocol, or new dependencies, an ADR is required (per §12).
-->

Refs: #
ADR: docs/adr/XXXX-...

## Type of change

- [ ] Bug fix (non-breaking)
- [ ] New feature (non-breaking)
- [ ] Breaking change (semver-major bump if 1.x; clearly marked otherwise)
- [ ] Documentation
- [ ] ADR
- [ ] Schema change (requires ADR)
- [ ] Build / CI / tooling

## Scope check

<!--
Per §0d Release Scope Matrix, additions to v0.1 require equivalent removals.
Confirm where in the release this change lands.
-->

- [ ] In §0d v0.1.0 Windows Preview scope
- [ ] In §0d v1.0 scope (will not block v0.1 ship)
- [ ] In §0d v2.0+ scope (queued; do not merge yet unless explicitly approved)

## Tests added / updated

<!--
Per §12 Engineering Charter: 80% line coverage, 100% on safety primitives.
Doc tests for any new public API. Property tests for new state machines.
Fuzz harnesses for new parsers.
-->

- [ ] Unit tests added/updated and passing
- [ ] Integration tests (where applicable) added/updated and passing
- [ ] Property tests for state machines (if relevant)
- [ ] Doc tests for new public APIs
- [ ] Coverage delta does not regress

## Quality gates passed locally

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo audit` clean
- [ ] `npm run lint`
- [ ] `npm run test`
- [ ] `npm audit` clean (high/critical)

## Capability / security review

<!--
For changes touching §8.security layers, capability declarations, IPC,
or skill/tool/agent generation, fill these in.
-->

- [ ] No new `unsafe` blocks (or each has a `// SAFETY:` comment)
- [ ] No new third-party dependencies (or `cargo deny check` passes)
- [ ] Capability set declared in any new artifact
- [ ] Threat model in `docs/SECURITY.md` updated if scope changed

## DCO sign-off

- [ ] All commits include `Signed-off-by:` (`git commit -s`)

## AI assistance disclosure

<!--
Per CONTRIBUTING.md: AI-assisted contributions are welcome but must be disclosed.
-->

- [ ] No AI tools used
- [ ] AI tools used; described in commit message or here:
      <!-- Tool name + how it was used (e.g., "Claude Code wrote the bulk; I
           reviewed and edited"). -->

## Documentation

- [ ] README updated (if user-facing change)
- [ ] CHANGELOG.md updated under [Unreleased]
- [ ] ADR filed if required (see Linked issue / ADR above)
- [ ] Spec updated if any §0–§14 section changed

## Breaking changes

<!-- If this PR introduces a breaking change, describe what breaks and the migration path. -->

## Screenshots / GIFs

<!-- For UI changes only. Optional otherwise. -->
