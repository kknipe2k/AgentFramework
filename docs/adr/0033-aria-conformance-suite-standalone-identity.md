# ADR-0033: ARIA demoted from archetype-as-raison-d'être to conformance suite; standalone product identity stated

**Status:** Accepted (2026-06-10 — flipped in the spec-reconciliation pass this ADR's Related section named as outstanding; the introducing PR #131 merged without applying the in-PR flip)
**Date:** 2026-06-09
**Deciders:** @kknipe2k (maintainer / product owner; Claude review + external-review recommendation)
**Tags:** scope, positioning, product, market

## Context

ADR-0001 ("ARIA as Archetype, Not Built-In Default") established that the runtime is a generic platform and ARIA is its canonical reconstructed example, with the **v0.1 MVP success criterion** defined (in spec §0d) as *"a user can reconstruct every row of the §0a Capability Matrix inside the runtime."* At that time, ARIA-reconstructibility **was** the organizing deliverable — the thing whose completion meant "v0.1 is done."

ADR-0031 then re-cut v0.1 around **author-and-run**, and ADR-0032 made the **lighthouse deliverable** the maintainer's own software-development loop (research → PRD → plan → implement → `bash verify.sh`), pulling execution breadth and shell-exec into v0.1 as vertical slices (M09–M13). Those two ADRs **already** moved the product's center of gravity off ARIA-reconstruction and onto "author any agentic workflow on the canvas and run it for real, under enforced capability contracts, watched live."

But the framing in the spec (§0/§0a "ARIA Archetype Capability Matrix"), in CLAUDE.md §1, and in ADR-0001 itself still reads as though ARIA-reconstructibility is the reason the product exists. A 2026-06-09 maintainer-commissioned external review (`docs/code-design-review-2026-06-09.md`) asked the question directly — *"ignoring ARIA, does the runtime make sense?"* — and concluded **yes**, on its own merits (the gap-suspend-clean model, capability *enforcement* rather than capability *prompting*, live observability, local-first/no-telemetry), with the caveat that the docs still over-anchor on the archetype. This ADR closes that framing gap: it demotes ARIA explicitly and states the standalone identity, **without disturbing ADR-0001's technical decision**.

This ADR also records the **target market**, which had been stated only negatively ("not a low-code tool for non-technical users in v1") and needed a positive frame.

## Decision

**1. ARIA is a conformance suite, not the raison d'être.** ARIA's role is reframed — by analogy, it is the project's *TodoMVC*: a fixed, representative reference workload that proves the primitives compose into a real framework and guards against capability-matrix regressions. It is **no longer** the definition of "why the product exists" or the headline success criterion. The lighthouse deliverable (ADR-0032: the dev-loop, author-and-run, `bash verify.sh` as the objective verify gate) is the organizing deliverable. `examples/aria/` and `examples/ralph/` remain in-tree as the conformance proofs and continue to gate v0.1 ship as a *regression contract* (a matrix row that stops being reconstructible is still a regression) — but they are the test suite, not the product thesis.

**2. The standalone product identity (the one-line frame):**

> *A local desktop runtime where agentic workflows are authored, executed under enforced capability contracts, watched live as a graph, and suspended cleanly when they hit a capability they don't have — with first-class cost control and no telemetry.*

Every primitive the runtime ships earns its place against *that* sentence, not against ARIA-reconstructibility. ARIA is one workload that exercises the sentence; it is not the sentence.

**3. Target market (positive frame, superseding the "not for non-technical users" negative-only statement):** the runtime **is** a low-code / node-based authoring tool — but **not for truly non-technical users**, who will be lost by the underlying concepts (capabilities, tiers, MCP, gap/resume). The market is two adjacent audiences who build *the same way* on the canvas:
   - **Low-code / less-experienced-but-technical builders** who want structure and guardrails rather than a blank terminal; and
   - **Well-versed / experienced users** who want **visibility** (the live graph + drill-able trace), **structure** (enforced capability contracts, tiers, plans), and **better cost control** (the budget enforcer + per-run spend) than an unstructured agent harness gives them.

   "Novices and experienced users build agentic processes the same way" (CLAUDE.md §1) is retained and sharpened: *novice* here means low-code-capable and technically literate, not non-technical.

**4. This AMENDS, it does not SUPERSEDE, ADR-0001.** ADR-0001's technical decision stands in full: the runtime is a generic platform, ARIA is reconstructed inside it (not bundled, not wrapped, not ported), the runtime ships zero built-in frameworks, and `.aria/` shell stays as untouched reference material. Only the **positioning/emphasis** — ARIA's status as the product's reason-for-being and headline success criterion — is amended. ADR-0001's Status line gains an "Amended by ADR-0033" pointer (the §11-prescribed mechanism, applied for an amendment rather than a full supersession).

## Consequences

### Positive
- A fresh session (or a prospective contributor/user) reads a product identity that matches what M09–M13 actually build, instead of an archetype-reconstruction frame that ADR-0031/0032 already moved past.
- The market is stated positively, so product decisions (which affordances, how much hand-holding, where the cost surfaces live) have a target to serve rather than only a group to exclude.
- ARIA-as-conformance-suite is a healthier relationship: it keeps its regression-contract value (matrix rows must stay reconstructible) without distorting roadmap priority.

### Negative
- Spec §0/§0a and the §0d success criterion are now framed in terms ADR-0033 demotes; they need a reconciliation pass (owned by the orchestrator — see Related). Until that lands, the spec and this ADR are in tension, which CLAUDE.md §2's "surface the contradiction" rule requires be visible — hence this ADR is explicit that the spec pass is outstanding.
- "Conformance suite, gate v0.1 as a regression contract" must not quietly become "ARIA is optional / can rot." The matrix-row regression gate is retained precisely to prevent that.

### Neutral / future implications
- No code changes. No schema, capability-matrix, drone, sandbox, or provider changes. Docs + positioning only.
- Adding further conformance workloads (e.g. the ML/data-science vertical blueprint already in `docs/proposals/`) is consistent with this framing — they become additional conformance suites, not additional reasons-for-being.

## Alternatives Considered

### Alternative A: Leave the framing as-is (ARIA remains the stated raison d'être)
**Rejected because:** it contradicts ADR-0031/0032's already-accepted re-cut and the lived roadmap; fresh sessions and readers get a product story the build no longer matches. CLAUDE.md §2 treats spec/doc/reality drift as a bug to surface, not tolerate.

### Alternative B: Fully supersede ADR-0001
**Rejected because:** ADR-0001's *technical* decision (generic platform, ARIA reconstructed-not-bundled, zero built-ins) is correct and unchanged. Superseding it would wrongly imply the architecture is being revisited. This is an amendment of positioning only.

### Alternative C: Delete ARIA from the project entirely
**Rejected because:** ARIA is a genuinely useful conformance workload — it exercises every capability-matrix row and is the closest thing the project has to an end-to-end acceptance test of the primitive set. Its value as a regression contract is real; only its status as the *product thesis* was wrong.

## Related
- **Amends (positioning only; technical decision intact):** ADR-0001 (ARIA as Archetype). ADR-0001 Status line updated with the "Amended by ADR-0033" pointer in this PR.
- **Builds on:** ADR-0031 (author-and-run re-cut), ADR-0032 (vertical re-cut; lighthouse = the dev-loop, `bash verify.sh` verify gate).
- **Triggered by:** `docs/code-design-review-2026-06-09.md` §"ignoring ARIA, does the runtime make sense" + the standalone-identity recommendation.
- **Spec reconciliation OUTSTANDING (orchestrator-owned):** spec §0 (Project Positioning & Relationship to ARIA), §0a (Capability Matrix framing — keep the matrix as the conformance contract; demote the "reason for being" language), §0d (restate the v0.1 success criterion around the ADR-0032 lighthouse, with ARIA-reconstructibility retained as a regression gate). CLAUDE.md §1 identity updated in this PR; the deeper spec pass is a follow-up the orchestrator picks up.
- CLAUDE.md §1 (project identity — updated in this PR), §2 (surface-the-contradiction rule).

## Notes
Maintainer-directed positioning amendment (2026-06-09 session), recorded as an ADR to match the project's convention that positioning/scope decisions are ADRs (ADR-0001, ADR-0031, ADR-0032 precedent) and to give the orchestrator a durable, propagatable artifact. No code, schema, or capability-matrix change. The market framing is the maintainer's product-owner call, captured verbatim in intent: low-code/no-code authoring for technically-literate builders and experienced users who want structure, visibility, and cost control — explicitly **not** truly non-technical users.
