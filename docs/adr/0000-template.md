# ADR-NNNN: Title

**Status:** Proposed | Accepted | Superseded by ADR-XXXX | Deprecated
**Date:** YYYY-MM-DD
**Deciders:** @maintainer1, @maintainer2
**Tags:** schema, security, capability, ipc, provider, scope, ...

## Context

What's the situation that calls for a decision? Be specific. Include:
- The technical or product problem we're trying to solve
- Constraints (time, dependencies, scope per §0d, regulatory, etc.)
- What's already been tried, if anything
- What would happen if we don't decide

Avoid editorializing here. Stick to facts. The decision and rationale come below.

## Decision

What we decided. One sentence summary, then a paragraph or two of detail.

State the decision in active voice: "We adopt X." Not "X was adopted" or "It was decided that X."

## Consequences

What this decision causes — both intended and unintended.

### Positive
- ...
- ...

### Negative
- ...
- ...

### Neutral / future implications
- ...

## Alternatives Considered

What else was on the table and why each was rejected.

### Alternative A: ...
**Rejected because:** ...

### Alternative B: ...
**Rejected because:** ...

## Related

- Spec sections: §X, §Y
- Issues: #N
- Prior ADRs: ADR-XXXX (if this builds on or supersedes another)
- External references: links to relevant articles, RFCs, prior art

## Notes

Any additional context that doesn't fit elsewhere. Conversation excerpts, vote tallies if applicable, links to related discussion threads.

---

## How to use this template

1. Copy this file to `docs/adr/NNNN-short-title.md`. Use the next available number; never reuse.
2. Fill in every section. Empty sections are not OK — explicitly write "None" if there are none.
3. Status starts as `Proposed`. Becomes `Accepted` when a maintainer approves the PR. Becomes `Superseded` (with link to successor) only when a later ADR replaces it; the file itself is never deleted or rewritten.
4. ADRs are immutable once accepted. If something changes, file a new ADR that supersedes the old one.
5. The PR that introduces the ADR should also reference it from any spec sections, code, or other docs the decision affects.

## When an ADR is required (per §12)

- Adding/changing/removing any §0a Capability Matrix primitive
- Changing any `schemas/*.v*.json` file
- Adding a new `LLMProvider` impl
- Changing capability enforcement behavior (any §8.security L1–L5)
- Changing the IPC protocol between main, drone, or sandbox
- Adopting a new core dependency (runtime, not dev-only)
- Significant scope changes to v0.1 / v1.0 / v2.0 per §0d

For smaller decisions (refactors, minor optimizations, internal abstractions that don't cross the §0a primitive boundary), a clear PR description is enough — no ADR required.
