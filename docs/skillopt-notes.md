# SkillOpt — M9 phase-doc input notes

**Source**: Microsoft Research, *SkillOpt: Executive Strategy for Self-Evolving Agent Skills*, arxiv 2605.23904 (May 2026). Repo: `github.com/microsoft/SkillOpt`. Project page: `microsoft.github.io/SkillOpt`. **Notes drawn from**: public README, `configs/_base_/default.yaml`, `configs/searchqa/default.yaml`, `docs/guide/skill-document.md`, `docs/guide/training-loop.md`, `docs/guide/dl-analogy.md`, and `skillopt/envs/alfworld/skills/initial.md`. The arxiv PDF was blocked by 403 from this environment; the upstream docs cover the structural surface we need for M9 phase-doc authoring.

**Why this file exists**: per the M08.5 IRL re-verify handoff (`docs/m08.5-irl-re-verify-handoff.md` lines 333–414), the M9 phase doc has a reconciliation gate that absorbs starter-kit methodology + DESIGN.md/Stage D inputs before authoring. SkillOpt's structural insight (skill.md as the trainable external state of a frozen agent) is a third input to that gate — not a v0.1 deliverable, but a design input to the `skill.md` shape M9 Phase 8b will lock in. The verification claims in the marketing material (52-cell sweep, +23.5 / +24.8 / +19.1 deltas on GPT-5.5) are not load-bearing for M9 design — the **structural recommendations** are.

---

## The load-bearing claim

A skill doc is the agent's "prompt weights." A separate optimizer model performs bounded **add / modify / delete** edits per training step against the skill markdown, gated by a held-out validation split. The agent itself is frozen. The edit unit is a **patch**, not a rewrite. The "textual learning rate" is the **cap on edits per step**, not a scalar — `learning_rate: 4` means at most 4 edits per step. Cosine decay over epochs is recommended.

If our M9 Skill Writer generates a `skill.md` shape that's hostile to surgical patches, we burn the v1.x optimization roadmap before we get there. If it generates a shape that's friendly to surgical patches, we get the option for free.

---

## What the skill.md shape looks like in SkillOpt

The ALFWorld seed (`skillopt/envs/alfworld/skills/initial.md`) is representative — plain markdown, no YAML frontmatter, organized as:

```
# <Domain> Skill

## Overview
<one-paragraph framing + output format directive>

## Task Types
<table mapping task type → goal → key steps>

## General Principles
1. **<rule name>**: <directive>
2. **<rule name>**: <directive>
…

## Common Mistakes to Avoid
- **<failure mode>**: <how to avoid>
- **<failure mode>**: <how to avoid>
```

The SearchQA seed (`skillopt/envs/searchqa/skills/initial.md`) is a one-liner stub: `# Question Answering Skill\n\n(No learned rules yet. Rules will be added through the reflection process.)` — i.e., the optimizer is expected to grow structure from empty too. Both empty-seed and rich-seed are first-class inputs.

The `docs/guide/skill-document.md` example structure:

```markdown
# Task Strategy

## General Approach
- Break complex problems into sub-steps
- Always verify intermediate results

## Common Patterns
- When you see X, try approach Y
- Avoid Z because it leads to errors

## Edge Cases
- If the input contains A, handle it specially by...
- Watch out for B — it requires C

## Output Format
- Always include reasoning before the answer
- Format numbers with proper units
```

**No YAML frontmatter** in SkillOpt's skill docs. This is the structural difference vs the Anthropic / OpenAI SKILL.md spec (which mandates `name` + `description` frontmatter for skill activation by an agent). Our M9 generated skills will need the frontmatter for runtime-side capability + trigger declarations (M9 Phase 8b deliverable line: "generate `skill.md` instruction-set markdown with frontmatter (capabilities, mode_variants, triggers)"). So our shape is **frontmatter + body**, where the body is the SkillOpt-style surgical-patch surface.

---

## How the optimizer edits — the six-stage loop

Per `docs/guide/training-loop.md`:

1. **Rollout** — frozen target model runs tasks with current skill doc; trajectories + scores collected.
2. **Reflect** — optimizer analyzes **failed** trajectories, produces edit patches. Shallow mode = independent; deep mode = cross-referenced systemic analysis.
3. **Aggregate** — semantically-similar patches merged to drop redundancy.
4. **Select** — edits ranked + filtered, capped at `learning_rate` count.
5. **Update** — selected patches applied → new skill version.
6. **Gate** — validation-split rollout; accept the new version only if it strictly improves the score, else reject + roll back.

Between epochs:
- **Slow update** = rollout the prior epoch's skill alongside the current one, identify improvement patterns, inject guidance back into the doc (momentum analog; prevents catastrophic forgetting).
- **Meta skill** = cross-epoch strategy memory that accumulates as context for future Reflect calls.

The DL-analogy table in `dl-analogy.md` is exact: edit patches ↔ gradients, edit selection ↔ gradient clipping, gate ↔ validation-based early stopping.

---

## The textual learning rate (concretely)

From `configs/_base_/default.yaml`:

| Knob | Default | Meaning |
|---|---|---|
| `learning_rate` | 4 | Max edits applied per step. |
| `min_learning_rate` | 2 | Floor under cosine decay. |
| `lr_scheduler` | `cosine` | Decay schedule: `cosine` / `linear` / `constant`. |
| `skill_update_mode` | `patch` | Incremental patches, not rewrites. |
| `use_slow_update` | `true` | Epoch-boundary momentum. |
| `use_meta_skill` | `true` | Cross-epoch optimizer strategy memory. |
| `batch_size` | 40 | Tasks sampled per rollout. |
| `num_epochs` | 4 | "2–4 usually enough" per docs. |
| `split_ratio` | `2:1:7` | train : selection (val) : test. **Only 20% is train.** |

Reported transfer rules from the paper / docs: cosine > constant; moderate LR (4–16) > extremes; slow update + meta skill both lift scores. Bigger batch ≠ better (API cost diminishing returns); more epochs ≠ better.

---

## Structural recommendations to fold into M9 Phase 8b

These are the M9 phase-doc input items, derived from the SkillOpt structural pattern. Each is cheap **if locked in at M9** and expensive to retrofit.

1. **Body sections must be optimizer-targetable units.** Generate body content under named `##` headings that map to discrete behavioral categories. The SkillOpt-validated pattern: `Overview`, `Task Types` (table), `General Principles` (numbered list of named rules), `Common Mistakes to Avoid` (bulleted list of named failure modes), `Output Format`. A future optimizer's add/modify/delete patch acts on a **rule** or **bullet** inside one of these sections — not on a free-form paragraph.

2. **Rule items get a bold name + colon + directive.** `1. **<rule name>**: <directive text>` is the SkillOpt convention. The bold name is the rule's stable identifier across edits. Delete = remove the line; modify = rewrite the directive while keeping the name; add = append a new named rule. Free-form prose paragraphs are not surgically editable and should be avoided in generated body content.

3. **Keep frontmatter and body separate concerns.** Frontmatter carries runtime contract (capabilities, mode_variants, triggers, content_hash, provenance — M9 deliverable line 270). Body carries instructional content (the optimizer-editable surface). A future optimizer must NOT edit frontmatter — schema validation would reject malformed edits, and capability declarations are a security boundary (§8.security L4). Generated `skill.md` must declare this split as a contract.

4. **Provenance per section, not per file.** The M9 deliverable already includes `provenance: {generator, model, prompt_hash, generated_at, validated_at, content_hash}`. For optimizer-friendliness, generated body sections should carry inline section-level provenance markers (e.g., HTML comments at section heads) so a future optimization pass can diff which sections it touched and recompute `content_hash` over the body excluding the markers. This is the cheap version; the expensive version is a separate per-section content addressed store, which is out of scope for v0.1.

5. **Empty-seed parity.** The SearchQA stub proves SkillOpt accepts an empty skill that grows from nothing. M9 should support generating a minimal stub `skill.md` (frontmatter + `# <Name>\n\n## Overview\n\n(<one sentence>)\n`) as a first-class generation mode, not just a rich generation. This lines up with v1.x "skill bootstrap" flows where the user generates the shape, then iterates.

6. **The body length budget matters.** Anthropic's SKILL.md guidance is "keep body under 500 lines" (context bloat degrades performance). SkillOpt's `learning_rate: 4` cap means a body grows by ~4 rules per epoch under default settings, so the budget is naturally bounded — but M9 should enforce a structural max in the schema (e.g., body ≤ N lines) and the Skill Writer should refuse to generate over-budget initial skills.

7. **Tables for enumerable categories.** The ALFWorld `## Task Types` table is a `| Type | Goal | Key Steps |` shape. Tables are surgical-edit-friendly (add a row, delete a row, modify a cell) and dense. M9 Skill Writer should prefer table-form when the content is enumerable (task types, mode-variant mappings, capability-narrowed sub-cases).

8. **Output Format directive belongs in the Overview.** SkillOpt's ALFWorld example puts the `<think>...</think> / <action>...</action>` directive in the Overview as a bolded "Output format" callout. M9 generations targeting structured-output agents should follow the same shape — the output contract is part of the skill identity, not a leaf rule subject to deletion.

---

## What this is NOT input for

- **v0.1 doesn't get an optimizer loop.** No rollout / reflect / gate cycle in M9. M9 generates once + validates once + the user installs. Iterative training is v1.x.
- **No "textual learning rate" knob in the v0.1 UI.** The Builder Canvas generates; it doesn't train.
- **The validation gating mechanism doesn't replace L3 sandbox validation.** SkillOpt's gate is "did the score on the val split improve" — a benchmark-driven check. L3 is "does the artifact pass schema + capability + sandboxed example execution" — a contract-driven check. They are orthogonal. v1.x optimization-loop additions would compose, not replace.
- **The 52-cell benchmark claim isn't load-bearing for our design.** Even if the numbers don't transfer to our domain, the structural recommendations above stand because they are about edit-unit granularity, not about the optimizer's effectiveness.

---

## Forward-looking — when this gets revisited

- **M9 phase-doc authoring** (after the reconciliation gate per `docs/m08.5-irl-re-verify-handoff.md` line 369). Fold items 1–8 above into the Phase 8b Skill Writer's generation spec + the `schemas/skill.v1.json` body-section enumeration.
- **v1.x optimization roadmap** (post-v0.1). If we add an optimization loop, this file is the spec-side input. ADR required (new feature, new dependency surface, new capability — likely a new "skill-optimizer" capability narrowing).
- **DESIGN.md / Stage D interaction**. Skill body shape is a UX surface in the Builder review screen (the user reads the generated skill before clicking Install per M9 acceptance criteria line 276). The section convention above gives the review screen a stable layout to render.

---

## Open questions for verification before locking the M9 phase doc

1. Does the SkillOpt paper's arxiv PDF section on edit-patch grammar specify a stricter schema than the `add / modify / delete` × `rule | bullet | section` shape inferred here? (PDF was 403-blocked from this environment; user could fetch.)
2. Does the paper claim transferability of skill docs across **different agent harnesses** (direct chat ↔ Codex ↔ Claude Code) at the body level, or only at the seed level? The marketing claim says "learned skills transfer across models and harnesses" — if body-level, our generated skills become more portable than v0.1 currently assumes.
3. Does SkillOpt's meta-skill mechanism mutate the skill doc directly or maintain a sidecar file? If sidecar, our generated `skill.md` should reserve a frontmatter slot or a sibling file path for v1.x extensibility.

Verification of these three would tighten items 4, 5, and the post-v0.1 scope above. None block M9 Phase 8b authoring at the structural level — the eight recommendations stand on the public docs alone.
