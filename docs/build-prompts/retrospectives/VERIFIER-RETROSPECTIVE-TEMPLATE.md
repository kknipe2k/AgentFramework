# Verifier Stage Retrospective Template

> Per-V retrospective shape. Filled in by the verifier agent at the end of the V stage. Distinct from per-stage retrospectives (`RETROSPECTIVE-TEMPLATE.md`) — verification axes, not build axes. Briefer because V doesn't ship code; it surfaces findings. Companion to `STAGE-PROMPT-PROTOCOL.md` §14 and ADR-0008.

---

## File location + naming

`docs/build-prompts/retrospectives/M[NN].V-retrospective.md` — one per milestone V run. If D.fix → V re-run produces additional findings, append a `M[NN].V.iter2-retrospective.md` for that iteration (don't overwrite — preserves audit trail).

## Required sections

### Front matter

```markdown
# M[NN].V — Verifier Retrospective

**Date:** YYYY-MM-DD
**Verifier session ID:** session_<id> (from CLI prompt context — for audit-trail provenance)
**Milestone phase doc:** docs/build-prompts/M[NN]-<short-title>.md
**Iteration:** 1 (or 2 if a re-run after D.fix)
**Prior V iteration retros:** N/A (or list of prior M[NN].V.iter*-retrospective.md files)
**Cross-machine state at start:**
  git log --oneline main..HEAD: <pasted output>
  ls docs/build-prompts/retrospectives/M[NN].*-retrospective.md: <pasted output>
```

### Per-pass summary

Single table; one row per pass.

```markdown
| Pass | Time | Findings | Notable |
|---|---|---|---|
| Inventory | <X> min | <N>🔴 / <N>🟡 / <N>🟢 | <one-line summary of the most surprising finding, OR "no findings — all files present + shape-matching"> |
| Wire | <X> min | <N>🔴 / <N>🟡 / <N>🟢 | <one-line; common pattern: "trace broke at step 4 for <spec claim>"> |
| Behavior | <X> min | <N>🔴 / <N>🟡 / <N>🟢 | <one-line; common pattern: "computed-style assertion failed for <component>"> |
| Multi-call invariants | <X> min | <N>🔴 / <N>🟡 / <N>🟢 | <one-line; common pattern: "second call to <API> returned <error|empty>"> |
```

### Findings

One subsection per finding. Numbered globally across passes (i.e., #1, #2, #3 — not per-pass).

```markdown
#### 🔴 #1 — <pass>: <primitive>

**Spec claim:** <quote or paraphrase the spec line being verified>
**Observed:** <what V actually saw — concrete: file path, line number, computed value, etc.>
**Expected (per spec):** <what V was looking for>
**Trace / harness:** <which harness exercised this; for Wire pass, the 5-step trace>
**Action:** Open D.fix iteration <N>; address by <recommended approach>; cite finding #1 in D.fix's `<deliverable>`.
```

Repeat for 🟡 (carries forward to next milestone's Stage A) and 🟢 (logged to `docs/tech-debt.md`).

### Verification axes scoring (three axes — borrowed from `PROCESS-VALIDATION.md` shape, adapted)

```markdown
| Axis | Score (1–5) | Rationale |
|---|---|---|
| **Coverage adequacy** | <N> | Did the four passes exercise the milestone's full deliverable surface? Score 5 = every primitive in V.2 scope was checked; 3 = some primitives skipped due to harness gap (note which); 1 = significant blind spots. |
| **Finding signal-to-noise** | <N> | Did findings represent real bugs (not false positives or noise)? Score 5 = every 🔴/🟡 traceable to a spec line; 3 = some findings ambiguous; 1 = pattern of false-positives. |
| **Fresh-context discipline** | <N> | Did the verifier successfully ignore prior retros / summary / gap-analysis? Score 5 = clear-and-paste pattern held + `<read_first>` honored; 3 = agent self-noted reading something on the forbidden list; 1 = bias guard failed. |
```

Sum of three axes < 9 (avg < 3) is the **soft signal**: the V protocol needs iteration; document specifics in the next section.

### Standing-rule compliance (v1.8 — M06.V Decisions 6 + 7)

Recorded, not scored (does not affect the three-axis sum / soft-signal threshold). Confirms the codified standing rules from `STAGE-V-VERIFIER-PROMPT-TEMPLATE.md` were applied this V run:

- **Decision 6 — delivered+tested / driver-absent / root = accepted-ADR carry-forward → 🟡 + mandatory enumeration:** [Applied to finding(s) #N — named ADR + quoted carry-forward clause + owning next milestone/stage | "Not triggered this V — no delivered-but-undriven primitive with an accepted-ADR root"]. Cross-checked `<wire_trace_vs_adr_reconcile>` / `<scope_change>` blocks: [yes / n/a — milestone has none].
- **Decision 7 — `--features integration` reference-MCP-server smoke (binding from M07.V):** integration smoke executed: [N/M, not 0/0 | "0/0 — informational only, this V predates M07.V" | "🟡 #N — unrunnable, blocker: <what>"].

### Outcome (one-line marker)

Pick one:

- **Sound** — all four passes produced clean signal; merge recommendation is "Proceed to E (closeout)". Findings (if any) are 🟡 or 🟢 only, not 🔴.
- **Sound but rough** — findings surfaced are addressable via D.fix iter 1; merge recommendation is "Open D.fix"; cited findings are scope-bounded.
- **Friction-heavy** — D.fix scope exceeds 2-iteration budget OR a finding requires architectural reframing not patching; merge recommendation is "Re-tier"; maintainer adjudicates.
- **Not ready** — V's own discipline failed (fresh-context broke, harness coverage was incomplete, pass produced false positives); revisit V protocol before re-running.

### `[END] Decisions for D.fix or next milestone`

Concrete, file-line specific. Same shape as work-stage retro's `[END] Decisions` section:

```markdown
- Decision 1: <specific change to apply at file path:line>; addresses finding #<N>; estimated <X> min in D.fix iter 1.
- Decision 2: 🟡 finding #<M> → next milestone's Stage A `<read_prior_milestones>` should reference this entry; absorb into Stage A.3 detailed changes.
- Decision 3: 🟢 finding #<P> → log to `docs/tech-debt.md` under "<category>" with one-line rationale.
- Decision 4 (if applicable): refine V protocol — <which pass> missed <which bug class>; update `STAGE-PROMPT-PROTOCOL.md` §14 OR `STAGE-V-VERIFIER-PROMPT-TEMPLATE.md` template before next milestone's V.
```

### Cross-machine state at end

```markdown
**Cross-machine state at end:**
  git log --oneline main..HEAD: <pasted output, including V's own pending commit>
  ls docs/build-prompts/retrospectives/M[NN].*-retrospective.md: <pasted output, including this file>
```

Mirrors the start-of-stage state; preserves audit trail for any downstream session.

---

## Authoring rules

1. **Brevity over completeness.** V is a verification stage, not a work stage. Findings are detailed; verification-axis rationale is one sentence each.
2. **Specifics over summaries.** Every finding cites a file:line, a spec section, an observed value, an expected value. "Generally looks ok" is not a finding shape.
3. **No work-axes scoring.** V doesn't ship code, so the per-stage retrospective axes (process quality, deliverable quality, complexity) don't apply. Verification axes (coverage, signal-to-noise, fresh-context discipline) replace them.
4. **Honest score 3s.** A 5 means perfect; if any pass surfaced surprises that weren't anticipated, score is lower than 5. The protocol benefits from the trailing signal.
5. **Outcome is one of four.** No mixed outcomes. If 🔴 findings are present, outcome is at minimum "Sound but rough" — never "Sound". If the verifier itself struggled, outcome is "Not ready" regardless of finding count.

## What goes elsewhere (not in this retro)

- **Per-stage retros** (`M[NN].A-retrospective.md`, `M[NN].B-retrospective.md`, etc.) — work stages' shape. V doesn't reference these for the bias-guard reason.
- **Milestone summary** (`M[NN]-summary.md`) — written by the closeout (Stage E) agent. V's findings feed into closeout's summary as a "Verifier results" section, not into this retro.
- **Gap analysis entry** (`docs/gap-analysis.md`) — written by closeout. V's 🟡 findings carry forward into closeout's Carry-forward section.
- **Tech-debt ledger** (`docs/tech-debt.md`) — V's 🟢 findings are logged there. Append-only.
- **ADR-class waivers** (`docs/adr/NNNN-waiver-M[NN]-finding-N.md`) — written by the build agent (NOT V) when a 🔴 finding is disputed on interpretation grounds. The waiver is its own artifact; this retro just records that the waiver was filed.
