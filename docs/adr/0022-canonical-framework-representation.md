# ADR-0022: Canonical framework representation — modular multi-file, resolved at the loader boundary

**Status:** Proposed
**Date:** 2026-05-22
**Deciders:** @kknipe2k
**Tags:** architecture, builder, persistence, framework, loader

## Context

M08 shipped the Workbench / Builder Canvas. The post-M08 IRL pass
(`docs/M08-irl-findings.md`, plus the follow-on review) established that
**the Builder cannot load the archetype frameworks** — `examples/aria/`
or `examples/ralph/` — at all. Loading ARIA produces a pile of
unconnected nodes with no workflow.

The root cause is two un-reconciled framework-on-disk representations
that no milestone bridged:

- **The archetype / `examples/` model.** A `framework.json` whose
  `agents` / `tools` / `skills` are `{id, path}` references into
  subdirectories — `agents/orchestrator.md`, `tools/git_checkpoint.md`,
  `skills/planning.md`. Each `.md` is YAML frontmatter (the
  `agent.v1.json` / tool / skill shape) plus a markdown body. Both v0.1
  reference frameworks use this; Ralph even references ARIA's tools
  cross-framework (`../aria/tools/…`).
- **The Builder model.** `crates/runtime-main/src/builder/persist.rs` +
  `src/lib/builderStore.ts` assume a `framework.json` with **inline**
  agents plus flat `<name>.agent.md` companion files at the directory
  top level.

`load_framework` (`persist.rs`) parses `framework.json` and scans only
the top directory for `*.{agent,skill,tool}.md`; against `examples/aria/`
it finds zero companions and never resolves the `{id, path}` references.
`projectCanvasEdges` (`builderStore.ts`) then skips every non-inline
agent (`if (!isInlineAgent(entry)) continue`), so no edges are
projected. The runtime's `spawn_framework_subagents` likewise walks
`inline_agents` and skips path-refs. Path-ref handling is shallow
end-to-end.

`schemas/framework.v1.json` already permits **both** forms — `agents[]`
is a `oneOf` of `{id, path}` or an inline `agent.v1.json`. The schema
never said which is canonical, or where a `{id, path}` reference
resolves. That decision was deferred by omission; M08.6 must make it,
because M09 (the LLM generators write into the Builder's `framework`)
and MVP §M8 ("the canvas is the editor"; load/save round-trip) both
depend on a Builder that can load a real, modular framework.

A best-practice review across three domains informed this decision:
the JSON-Schema / OpenAPI `$ref` ecosystem (the formalized
external-reference handling domain); the agent-definition file-format
idiom (Claude Code, SKILL.md, AGENTS.md, ForgeCode, Microsoft's Agent
Framework); and reference-resolution architecture (the MSBuild / IDE
project-model pattern).

## Decision

**The modular, multi-file, markdown-with-frontmatter representation is
the canonical on-disk form of a framework. References resolve at a
single loader boundary into a fully reference-resolved in-memory
`Framework`. Save re-splits back to the modular form.**

Concretely:

- **Canonical on disk = modular.** A framework on disk is
  `framework.json` plus `agents/*.md`, `tools/*.md`, `skills/*.md`, with
  agents / tools / skills referenced by `{id, path}`. Each `.md` is YAML
  frontmatter (the `agent.v1.json` / tool / skill shape) plus a markdown
  body. `examples/aria/` and `examples/ralph/` are **not rebuilt** —
  they already are this form, and it is the form the whole stack must
  support. This is the universal agent-definition idiom (every current
  agent system stores agents / skills as markdown-frontmatter files in
  directories) and the form external-reference best practice prescribes
  ("maintain reusable parts as separate files"; "total dereferencing is
  not advised").

- **One resolution boundary — the loader.**
  `runtime_main::builder::load_framework` walks the framework directory,
  reads each referenced `.md`, parses its frontmatter into an inline
  `Agent` / tool / skill, and returns a `LoadedFramework` whose
  `framework` is **fully reference-resolved** — every `agents[]` entry
  inline. Relative paths, including `../` cross-framework references
  (Ralph), resolve against the framework directory; the loader resolves
  paths deliberately (no glob, no symlink-escape) and surfaces a
  malformed / missing referenced file as a load error.

- **Downstream consumes the resolved model.** The canvas projection
  (`projectCanvasNodes` / `projectCanvasEdges`), the Tester, and the
  runtime's `spawn_framework_subagents` already speak the inline form;
  once the loader resolves, none of them needs path-ref logic.
  Reference resolution lives in exactly one place — the MSBuild / IDE
  single-resolution-boundary pattern.

- **Save re-splits.** `save_framework` writes the modular form back —
  `framework.json` with `{id, path}` references plus the companion
  `.md` files — so a load → edit → save round-trip preserves the
  modular structure. The Builder never collapses a framework to a
  monolithic inline document on disk. (This is "bundle, then persist
  modular," not "dereference and flatten" — the OpenAPI distinction.)

- **The agent `.md` body is the agent's system prompt.** The loader
  captures it; per `agent.v1.json` ("if `system_prompt_template` is
  absent, the runtime uses the agent.md body as the prompt template").
  M08.6 *captures* the body in the resolved model; *applying* it at run
  time is the M09 carry-forward (the prompt-application gap deferred
  from M08.5 decision 1).

## Consequences

### Positive

- The Builder can load the archetype frameworks. ARIA and Ralph render
  as real, wired workflows; MVP §M8's "the canvas is the editor" and the
  load → reload round-trip hold for genuine frameworks, not only the
  Builder's own inline fixtures.
- Artifacts stay genuinely shareable and reusable — separate `.md`
  files, the industry-standard agent-definition idiom. A loaded
  framework's `.md` artifacts can populate the Palette by type, which
  structurally addresses the IRL "defined agents are not shareable"
  observation.
- Reference resolution is one testable seam, not logic duplicated across
  the loader, the canvas, the Tester, and the runtime spawn walk.
- M09's generators write into the resolved in-memory `Framework`
  (ADR-0020's write target); save re-splits, so a generated artifact
  becomes a new companion `.md` automatically.

### Negative

- The loader grows real work: a directory walk, `.md` frontmatter
  parsing for three artifact types, and relative-path (incl. `../`)
  resolution. The existing gap-tolerant `load_framework` posture
  (a partially-built framework still loads; gaps surface as validation
  errors) is preserved, but a malformed referenced file is now a load
  error.
- `save_framework` must re-split — derive companion files and their
  frontmatter from the inline model — which is more than today's flat
  write.
- Cross-framework references resolve outside the framework directory
  (Ralph's `../aria/tools/…`). Reading user-picked local files is far
  lower-risk than M07.5's remote-URL SSRF surface, but the loader still
  resolves `..` paths deliberately.

### Neutral / future implications

- `schemas/framework.v1.json` is **unchanged** — the
  `oneOf(inline | {id,path})` already permits both forms. This ADR
  decides which is canonical and where references resolve; it does not
  change validation behavior, so there is no schema version bump and no
  §14 schema ADR.
- A v1.0 registry / "Share It" story builds naturally on modular,
  individually-addressable artifact files.

## Alternatives Considered

### Alternative A — teach the whole stack to handle `{id, path}` refs

**Rejected:** it spreads reference resolution across the loader, the
canvas projection, the Tester, and the runtime spawn walk. Reference
resolution belongs at a single boundary (the MSBuild / IDE pattern);
multiple resolution sites drift out of sync.

### Alternative B — rebuild `examples/aria/` (and Ralph) to the flat inline representation

**Rejected:** inlining the archetype fights the universal
markdown-frontmatter agent-definition idiom, makes `framework.json` a
monolith, and destroys artifact shareability — the explicit IRL goal.
It "fixes" the two examples while the Builder still cannot load any
other modular framework. External-reference best practice is explicit:
maintain reusable parts as separate files; total dereferencing is not
advised. ADR-0001 makes ARIA the archetype — the tool must adapt to the
archetype, not the reverse.

### Alternative C — single-file framework (the n8n model)

**Rejected:** n8n stores one self-contained JSON per workflow because
its nodes are built-in *types*, not user-authored reusable artifacts.
This project's agents / tools / skills are exactly reusable, shareable,
cross-framework artifacts (Ralph already reuses ARIA's tools). A
single-file format cannot express cross-framework artifact reuse.

## Related

- IRL findings: `docs/M08-irl-findings.md` (the Builder-cannot-load-ARIA
  observation and the post-findings review that surfaced it)
- Build prompts: `docs/build-prompts/M08.6-*.md` (the milestone this ADR
  founds); `docs/build-prompts/M08.5-irl-fix.md` (the sibling 🔴 + harness
  cycle)
- Prior ADRs: ADR-0020 (`framework.json` is the Builder's single source
  of truth — this ADR settles what that document's *references* mean and
  where they resolve); ADR-0001 (ARIA as archetype — why ARIA must be
  loadable); ADR-0021 (the real-app `tauri-driver` gate that
  regression-guards the load path)
- Schemas: `schemas/framework.v1.json` (`agents[]` `oneOf`),
  `schemas/agent.v1.json` (the `.md` frontmatter shape;
  `system_prompt_template` and the agent-body-as-prompt rule)
- Research domains (best-practice basis): JSON-Schema / OpenAPI `$ref`
  external-reference handling (bundle vs. dereference; "keep reusable
  parts as separate files"); the markdown-frontmatter agent-definition
  idiom (Claude Code, SKILL.md, AGENTS.md, ForgeCode, Microsoft Agent
  Framework); MSBuild / IDE single-boundary reference resolution
- Spec: §6 (Phase 6 Framework JSON Loader), §9 (Phase 9 the Canvas)

## Notes

Filed `Proposed` alongside the M08.6 phase doc. Flips
`Proposed → Accepted` in the M08.6 stage that implements the loader
resolution (the M06.5.A.fix / ADR-0012 precedent — the stage that
implements an ADR flips it).
