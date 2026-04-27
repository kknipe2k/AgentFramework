---
name: Feature proposal
about: Propose a new capability or significant change. Read §0d release scope first.
title: '[proposal] '
labels: enhancement, needs-triage
assignees: ''
---

<!--
Before filing: please read the spec (§0d Release Scope Matrix in particular).
Many "missing" features are deliberately deferred to v1.0 or v2.0; filing them
won't accelerate them.

Significant features require an ADR before implementation. This template helps
us decide whether the feature is worth an ADR.
-->

## Problem statement

<!-- One paragraph: what real-world problem does the absence of this feature
     cause? Be specific about who it affects and how often. -->

## Proposed solution

<!-- Specific enough to argue against. "Some kind of plugin system" is too vague.
     "A NotifierPlugin trait that takes HitlNotifyEvent and returns Result<()>"
     is specific. -->

## Why now? Why not v1.0+?

<!-- Per §0d, scope-creep kills OSS projects. Make the case for prioritizing
     this over what's already in v0.1 scope. If it should land in v1.0+, say
     so explicitly. -->

- [ ] Must land in v0.1 because:
- [ ] Should land in v1.0 — happy to wait
- [ ] v2.0+ is fine — filing now to track the idea

## Scope impact

<!-- What sections of the spec would change? Which §0a Capability Matrix rows? -->

- [ ] §0a Capability Matrix — adds row(s):
- [ ] §0d Release Scope Matrix — moves to:
- [ ] No spec change required (purely additive in user code)
- [ ] Schema change required (will need an ADR per §12)
- [ ] Capability enforcement change required (will need security review per §8)

## Alternatives considered

<!-- What else did you think about? Why is this proposal better? -->

1. **Alternative A:** ... (rejected because: ...)
2. **Alternative B:** ... (rejected because: ...)

## Implementation sketch (optional)

<!-- If you have a concrete implementation in mind, sketch it here. Helps a
     maintainer decide whether to commission an ADR. Not required at proposal stage. -->

## Compatibility

- [ ] Backwards-compatible — no existing usage breaks
- [ ] Breaking change — requires major version bump
- [ ] New surface only (no impact on existing code)

## Threat model implications

<!-- If this feature touches capability enforcement, generators, registry, or
     anything user-facing that could affect security, describe the implications.
     Per §8.security threat model: do we still defend against malicious model
     output / compromised registry / user error after this lands? -->

## What success looks like

<!-- One sentence: how will we know the feature is done and worthwhile? -->

## Willing to contribute?

- [ ] Yes, I can write the ADR.
- [ ] Yes, I can implement after ADR is approved.
- [ ] I can help review and test.
- [ ] Filing for visibility; not committing to work.
