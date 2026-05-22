# ADR-0020: Builder canvas ↔ framework.json state model

**Status:** Accepted
**Date:** 2026-05-21
**Deciders:** @kknipe2k
**Tags:** architecture, state, renderer

## Context

M08 delivers the Workbench / Builder Canvas (spec Phase 9): a three-panel
build-time tool where a user composes runtime primitives (Tools / Skills
/ Agents) visually, edits the same framework as raw JSON, validates it
continuously against `schemas/framework.v1.json`, and saves it to disk.
The Builder spans four work stages after the shell — D1 (node editor),
D2 (edges + capability narrowing + validation), E (Inspector + two-way
canvas↔JSON binding) — and M09's three Generators write generated
artifacts into the same in-progress framework.

The Builder needs a renderer state model. The renderer already has one
Zustand store, `graphStore`, which holds **live-execution** graph state
(an `applyEvent` reducer over ~34 `AgentEvent` variants, plus tier / MCP
/ import slots). Two questions must be settled before D1 builds the
interactive canvas, because every later stage and M09 build on the
answer:

1. **Which store?** Reuse `graphStore`, or a new store?
2. **What is authoritative — the canvas (React-Flow nodes/edges) or the
   `framework.json` document?** A visual editor can treat either as the
   source of truth: edit the node graph and serialize to JSON on save,
   or edit the JSON and re-derive the node graph on every change.

Spec Phase 9 states the intent directly: *"It generates valid
`framework.json` against `schemas/framework.v1.json`; output is the
source of truth, the canvas is the editor."* The MVP §M8 acceptance
criteria require a Canvas | JSON two-way binding where *"canvas edits
update the JSON; JSON edits re-render the canvas"* (criterion 6) and a
byte-stable save→load→save cycle (criterion 8). If we don't settle this
now, D1 and D2 will each pick a model implicitly and E's two-way binding
will have to reconcile two divergent representations.

## Decision

**We adopt `framework.json` as the single source of truth for the
Builder, held in a new, separate Zustand store `builderStore`. The
React-Flow canvas (nodes + edges) is a *projection* derived from the
`framework` document — never an independent state.**

Concretely:

- `builderStore` (`src/lib/builderStore.ts`) is a **new** `create()`
  store, **separate** from `graphStore`. Its core slot is
  `framework: Framework` — the generated type from
  `schemas/framework.v1.json` (CLAUDE.md §14; never hand-written). It
  also holds `diskFramework` (the last saved/loaded snapshot, for E's
  disk-diff), `selectedNodeId`, and `validation` (the Stage B
  `FrameworkValidationReport`).
- **Every edit mutates `framework`.** A canvas drop, an inline-config
  edit, an edge connection, a node deletion, a JSON-tab edit, and (M09)
  a Generator result all flow through store actions that mutate the
  `framework` document. There is no separate "canvas nodes" state that
  can drift from the document.
- **The canvas is a pure projection.** D1/D2 add derived selectors
  (`canvasNodes` / `canvasEdges`) that compute React-Flow nodes and
  edges from `framework` (plus a `nodePositions` slot for user-placed
  manual layout, which is editor-local view state, not framework data).
  Rendering the canvas is `framework → projection`; it is never the
  reverse.
- **Two-way binding falls out for free.** The Canvas | JSON toggle (E)
  needs no reconciliation engine: a canvas edit mutates `framework` and
  the JSON view re-serializes it; a JSON-tab edit calls
  `replaceFramework` and the canvas re-derives its projection. Both
  directions are the same one-way data flow `framework → view`.

M08 Stage C ships `builderStore` with this shape final — the
`replaceFramework` / `setDiskFramework` / `selectNode` / `setValidation`
actions implemented and the canvas-mutation actions (`addNode` /
`updateNode` / `connectEdge` / `removeNode`) as typed no-op stubs D1/D2
fill. Shipping the full interface at C means no later stage re-shapes a
`useBuilderStore` selector.

## Consequences

### Positive

- **The two-way binding is structurally trivial.** Because there is only
  one source of truth and the canvas is a derived view, MVP §M8
  criterion 6 needs no diff/merge engine — both edit directions reduce
  to "mutate `framework`, re-render".
- **Save/load is byte-stable by construction.** `framework` *is* the
  document `save_framework` writes (Stage B); a save→load→save cycle
  round-trips the same object (MVP §M8 criterion 8).
- **Validation has one input.** Stage B's `validate_framework` takes a
  framework document; D2's continuous validation feeds it `framework`
  directly — no need to first serialize a canvas graph.
- **M09's Generators have one write target.** "Generate Tool / Skill /
  Agent" appends to `framework`, the same as a manual canvas edit; the
  Generators do not need canvas-graph knowledge.
- **Build-time and run-time state stay isolated.** A separate store
  means `graphStore`'s live-execution reducer and `builderStore`'s
  build-time document never share a code path; neither can corrupt the
  other.

### Negative

- **Every canvas interaction round-trips through the document.** Dragging
  a node, connecting an edge, or typing in an inline field mutates
  `framework` and re-derives the projection. For v0.1 framework sizes
  (tens of nodes) this is negligible; a very large framework could make
  re-derivation a perf concern — deferred until it is measured, not
  pre-optimized.
- **Manual node positions are not framework data.** `framework.json` has
  no field for canvas coordinates, so user-placed positions live in an
  editor-local `nodePositions` slot (D1) that is *not* part of the
  source of truth and is not persisted by `save_framework`. This is
  acceptable — layout is a view concern — but it means re-opening a
  saved framework re-lays-out rather than restoring exact positions
  (an auto-layout affordance, D1, covers this).
- **Derived array/object selectors need `useShallow`** (gotcha #75) —
  `canvasNodes` / `canvasEdges` are computed arrays; D1/D2 must wrap
  them. This is a known, mechanical requirement, not a design flaw.

### Neutral / future implications

- The `Framework` type the store holds is schema-generated; a
  `framework.v1.json` schema change (a `v2`) would require regenerating
  the type and is itself an ADR-triggering event (CLAUDE.md §11/§14) —
  this ADR does not change that.
- v1.0's multi-framework comparison view would hold several `framework`
  documents; the single-source-of-truth model extends naturally to a
  map of documents without revisiting this decision.

## Alternatives Considered

### Alternative A: Reuse `graphStore` for Builder state

**Rejected because:** `graphStore` is the live-execution store — its
state is an event-sourced projection of a *running* session. Build-time
framework composition has a disjoint lifecycle (no session, no events,
no drone) and disjoint data. Overloading one store with both is the
dual-purpose-store anti-pattern: the `applyEvent` reducer and the
framework-edit actions would share a state object with nothing in
common, and a bug in one surface could corrupt the other. A separate
store keeps each contract clean.

### Alternative B: The canvas (React-Flow nodes/edges) is the source of truth

**Rejected because:** it directly contradicts spec Phase 9 ("output is
the source of truth, the canvas is the editor"). It also makes the
Canvas | JSON two-way binding a genuine reconciliation problem — a JSON
edit would have to be parsed and merged into an authoritative node
graph, and a node graph would have to be serialized to JSON on every
change, with the two representations free to drift. It would also force
M09's Generators to understand React-Flow's node model rather than just
appending to a document.

### Alternative C: Dual source of truth with an explicit sync layer

**Rejected because:** a sync layer between an authoritative canvas and an
authoritative document is exactly the complexity Alternative B incurs,
made explicit. There is no requirement that benefits from two
authoritative representations; one source of truth with a derived view
is strictly simpler and satisfies every MVP §M8 criterion.

## Related

- Spec sections: §9 (Phase 9 Visual Canvas and Tester), §6 (Phase 6
  Framework JSON Loader), §0b (runtime primitives)
- MVP: `docs/MVP-v0.1.md` §M8 (criteria 6 + 8)
- Schemas: `schemas/framework.v1.json` (the source-of-truth document)
- Prior ADRs: ADR-0002 (Tauri + Rust + React/Zustand stack); ADR-0014
  (skills-lock integrity — `save_framework`/`load_framework` round-trip)
- Build prompt: `docs/build-prompts/M08-workbench.md` Stage C (C.3.3),
  Stages D1 / D2 / E
- Gotchas: `docs/gotchas.md` #75 (`useShallow` for derived selectors)

## Notes

This ADR is filed at M08 Stage C (the stage that creates `builderStore`)
and is load-bearing for Stages D1, D2, E, and M09's Generators — each
builds the canvas, the edges, the two-way binding, and the Generator
write paths on the `framework`-as-source-of-truth model decided here.
Status flips `Proposed → Accepted` in the M08 PR before merge.
