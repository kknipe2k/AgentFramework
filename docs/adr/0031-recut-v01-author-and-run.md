# ADR-0031: Re-cut v0.1 around author-and-run (workbench-first); defer generators to v1.0

**Status:** Proposed
**Date:** 2026-06-05
**Deciders:** @kknipe2k (maintainer / product owner)
**Tags:** scope, product

## Context

The original v0.1 (MVP-v0.1.md + §0d Release Scope Matrix) sequenced
foundation → … → **M8 Workbench** → **M9 Generators** → **M10 First-run** →
**M11 Ship**, with a **generator-centric** success criterion ("generate a tool +
a skill, wire them, run a Test").

Two facts, established this milestone, make that scope wrong:

- **The runtime paints most of what it draws** (`docs/execution-status.md`,
  rule 11, IRL-confirmed). Only single-agent streaming, built-in **Read/Write**,
  **MCP dispatch**, skills, gap-**suspend**, the budget engine, and tier
  enforcement execute. Sub-agents, plans, hooks, and gap-**resume** are drawable
  but not runnable.
- **The M08 Workbench is a *composer*, not an *authoring tool*** (grounded in
  `docs/workbench-delivery-plan.md` §3, file:line): a fresh project cannot create
  an agent on the canvas (`Palette.tsx:173-184` — the Agents tab lists only
  installed/loaded artifacts); an agent can't be granted `file_access` on the
  canvas (`NodeConfigPanel.tsx:105-146` + `builderAgent` omits the required
  `capabilities`, `builderStore.ts:145`); a real MCP server's data tools aren't
  attachable; and half the primitive vocabulary (Plan, MCP, rails, budget) has no
  palette.

The maintainer's product direction (this session): **the product *is* the
workbench** — build any agentic workflow (author from scratch or import JSON),
configure it for real (capabilities / file_access, MCP / API data), and run it
industrial-strength. Generators are not the v0.1 wedge; **author-and-run** is.
Left undecided, MVP-v0.1.md's generator-centric scope and success criterion
contradict the product direction, the delivery plan, and the execution-status
ledger.

## Decision

**We re-cut the post-M08 roadmap around author-and-run, per
`docs/workbench-delivery-plan.md`, and re-line the v0.1/v1.0 boundary.**

The feature spine becomes:

- **M09 — Vertical slice:** author one real agent from scratch + grant `file_access`
  + attach a real MCP tool; run it; it writes a real file at the enforced tier.
- **M10 — Author-anything:** the full palette (Plan / MCP / rails / budget) +
  config for every node kind + delete/rename + the palette-integrity fix.
- **M11 — Real data:** MCP servers as first-class canvas citizens + a data-source
  catalog (GitHub/Postgres/Slack/Drive/Notion) + credentials UX.
- **M12 — Execution breadth:** sub-agents run, plans drive tasks, hooks fire.
- **M13 — Hardening:** validated whole-workflow import/export + save-path + integrity.

**Scope re-line:** **v0.1 = M09 + M10 + M11 + the release milestone**
(first-run polish + ship) — *"a workbench that builds and runs single-agent,
MCP-data workflows from scratch."* **v1.0 = M12 + M13 + Generators** (the old M9)
+ the remainder of §0d's v1.0 column.

The **v0.1 success criterion** changes from generator-centric to author-and-run:
a from-scratch agent + `file_access` + a real MCP tool **runs and writes a real
file** at the enforced tier (the M09.D IRL is the seed; the full two-path
criterion is rewritten at the release milestone). Generators (old M9) move to
v1.0; first-run/polish (old M10) + ship-prep (old M11) become the v0.1 **release
milestone(s)** after the feature spine, with the success criterion updated.

This ADR **amends §0d Release Scope Matrix** (a pointer is added at §0d).
`docs/workbench-delivery-plan.md` is the authoritative detailed roadmap;
MVP-v0.1.md becomes the milestone **index** pointing to it.

## Consequences

### Positive
- v0.1 ships something **real and demoable** — build *and run* a real workflow —
  instead of a composer that paints.
- The vertical slice (M09) rides substrate that **already executes**, de-risking
  the whole plan and proving the loop before breadth is added.
- Scope is **honest** (paint→execute tracked in `execution-status.md`); it directly
  answers the M08.6 failure mode (shipped "done, 0🔴"; IRL found 7🔴).

### Negative
- v0.1 **no longer includes Generators** — a headline §0d feature — deferred to v1.0.
- **Full plan/hook/multi-agent execution** is deferred to v1.0 (M12).
- The original generator-centric **success criterion + demo script are
  superseded** and must be rewritten at the release milestone.
- MVP-v0.1.md milestone **numbering shifts** (M9–M11 repurposed); the old detail
  sections are marked superseded, not deleted.

### Neutral / future implications
- §0d is amended by this ADR (pointer added); the detailed roadmap lives in
  `docs/workbench-delivery-plan.md`.
- M08.8's in-flight **D/E/F** (budget-visible / gap-resume / save-polish) are
  superseded by the re-cut and fold into **M11 / M13** — reconciled at the M08.8
  closeout (M08.8 A/B/B.fix/C + the C.fix tier-display fix still land).

## Alternatives Considered

### Alternative A: Keep the generator-centric v0.1 (ship M9 Generators next)
**Rejected because:** the workbench can't author from scratch yet, so generators
would emit artifacts into a composer the user cannot build with; and the IRL
showed the runtime paints most primitives — stacking generators on paint compounds
the M08.6 failure (shipped "done", 7🔴).

### Alternative B: Jump straight to execution breadth (multi-agent/plans/hooks) next
**Rejected because:** wiring more primitives without a trustworthy, authorable,
observable app re-creates the paint-not-execute gap at larger scale; M12 needs the
authorable + observable workbench (M09–M11) to be IRL-verifiable in the first place.

### Alternative C: Treat this as a reordering, no ADR
**Rejected because:** it redefines *what v0.1 delivers* and the *success
criterion* — a scope change per MVP-v0.1.md line 498 ("Edits to milestone scope
require an ADR") and CLAUDE.md §11 — so an ADR is required.

## Related

- Spec sections: **§0d Release Scope Matrix** (amended by this ADR)
- Docs: `docs/workbench-delivery-plan.md` (the detailed roadmap), `docs/execution-status.md`,
  `docs/MVP-v0.1.md` (reconciled to this ADR), `docs/build-prompts/M09-workbench-vertical-slice.md`
- Prior ADRs: ADR-0019/0030 (tier), ADR-0020 (document-as-source-of-truth),
  ADR-0029 (gap-resume — now M12), ADR-0026 (plan_loop — now M12)
- Supersedes the generator-centric v0.1 framing in MVP-v0.1.md M9–M11.

## Notes

Maintainer-directed product re-cut (2026-06-05 session). The delivery plan and the
M09 phase doc were authored and committed first (`1e6dbb4`, `0e0e569`); this ADR
records the scope decision they imply and is the §11 gate for the MVP-v0.1.md
reconciliation that lands with it.
