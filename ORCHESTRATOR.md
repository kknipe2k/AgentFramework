# ORCHESTRATOR.md — Orchestration operating manual

> **Scope — orchestrator role only.** If you are a build-stage or fix-stage
> CLI session, this file is not yours: follow your §X.5 stage prompt and
> CLAUDE.md, and ignore this document. It is never listed in a stage prompt's
> `<read_first>`.

> Read this first, every orchestration session — then CLAUDE.md, then the
> current milestone's live docs (phase doc, retrospectives, gap-analysis,
> `git log`). This is the **decision index** for the orchestrator role: it
> names the authoritative doc for each decision rather than duplicating it.
> Keep it small — if it grows into an essay it has failed.

## Why this exists

The orchestrator drifts across long sessions; the build machine does not,
because each build stage is a fresh session scoped by a §X.5 prompt. The
information to act correctly was never missing — it is spread across ADR-0008,
CLAUDE.md §8/§11/§12/§19/§20, STAGE-PROMPT-PROTOCOL, and the phase docs. This
doc collapses the synthesis surface so a fresh orchestration session acts
correctly without re-deriving the pattern. Most entries below trace to a real
failure in M07 and the rule that prevents the repeat.

## 0. Roles

- **Orchestrator** (this role) — authors spec / phase docs / ADRs / protocol
  docs; adjudicates build surfaces; routes V findings; runs GitHub PR/merge;
  sequences milestones.
- **Build CLI** — executes one stage per fresh session from a §X.5 prompt.
  Does NOT decide sequencing, routing, or milestone structure.
- **User** — HITL: approves outcomes, conduits prompts/surfaces, owns
  scope/priority calls.
- The orchestrator decides the steps; the user approves the outcomes; the
  build executes.

## 1. The loop (start → finish)

Per milestone M[NN]:

1. Author the phase doc (+ any ADRs).
2. Stages A…E — per stage: build surfaces **red** → adjudicate → build
   surfaces **final** → adjudicate. Nothing commits without approval.
3. Stage V — verifier (fresh-context, four passes).
4. Route V findings (§3).
5. Stage G — closeout (summary + gap-analysis + simplify_pass + coverage
   reconciliation + PR draft).
6. PR → CI green → flip ADRs Proposed→Accepted (the last commit) → merge.
7. If V deferred a 🔴: the X.5 fix-cycle runs before the next milestone.
8. Next milestone.

## 2. Authoring

| Artifact | Standard | Authoritative doc |
|---|---|---|
| Phase doc | M06-density: numbered N.3 subsections, verbatim code + why-prose, v1.8 audit slots. Split a stage (D1/D2) if large — never thin it. | latest merged M[NN] phase doc |
| §X.5 stage prompt | XML per the schema; the validator must pass. Stable artifact — delegates live state to `<read_first>`/`<cumulative_reads>`. | STAGE-PROMPT-PROTOCOL.md |
| ADR | File for the §11 triggers; Proposed → Accepted at merge; immutable after; supersede via a new ADR. | CLAUDE.md §11 |
| Waiver ADR | `docs/adr/NNNN-waiver-M[NN]-finding-N.md`; honest about defect-vs-dispute. | ADR-0008; precedents 0009, 0016 |

- Before editing a phase doc (>50 lines, or any X.5): cross-machine pre-flight
  — get `git log --oneline main..HEAD` from the build PC first (CLAUDE.md §8).
- Never improvise a pattern. Read the authoritative doc and the latest merged
  equivalent.
- Never rewrite or overlay a §X.5 prompt because it "looks stale" — drop it
  verbatim; it reads live orientation docs, so when it was authored is
  irrelevant.

## 3. Decision procedures (if / then)

| Situation | Action | Doc |
|---|---|---|
| V finds 🔴, fixing in-milestone | scoped D.fix (real §X.5 prompt + gates, max 2 iter) | ADR-0008 |
| V finds 🔴, deferring the fix | waiver ADR → X.5 fix-cycle before the next milestone | ADR-0008 / 0009 / 0016 |
| V finds 🟡 | fix-in-cluster, OR open a new cluster THIS phase, OR explicit ADR scope-out — **NEVER "carry to next milestone"** (zero-propagation) | cluster-pattern.md §2 |
| V finds 🟢 | docs/tech-debt.md (the explicit scope-out ledger — a legal disposition) | ADR-0008 / cluster-pattern.md §2 |
| A cluster/stage claims "done" | NOT closed until the assembled thing was **run and observed** (IRL for user-observable surfaces); "tests green" is not close | CLAUDE.md §4 rule 11 / cluster-pattern.md §1 |
| Tempted to route a finding forward | banned — three legal dispositions only: fix-now / new cluster / explicit ADR scope-out | cluster-pattern.md §2 |
| Post-V regression blocking CI | fix commit in the same milestone (its own bug) | — |
| Phase-doc code ≠ shipped reality | grandfather: record the defect in the retro; do NOT edit the phase doc mid-flight | CLAUDE.md §8 |
| A stage is too large | split into N1/N2 | D1/D2 precedent |
| Build work is needed | a structured §X.5 stage prompt with red/approval gates — NEVER a freeform relay | STAGE-PROMPT-PROTOCOL |
| Spec / ADR / CLAUDE.md / phase doc disagree | surface the contradiction; do not pick | CLAUDE.md §2 |
| Decision is scope / product-surface / irreversible architecture | escalate to the user, with a recommendation | CLAUDE.md §12 |
| Decision is technical best-practice | decide, document the rationale, proceed | CLAUDE.md §12 |

Always, before acting:

- **Precedent-check first.** Has this happened before? Which ADR / milestone?
  (M07 missed that D.fix was never-used and ADR-0009 was the waiver precedent.)
- **Verify before you assert or escalate.** Fetch, read the doc, search the
  web — never raise an alarm on an assumption. (M07: declared `main` broken
  with no fetch.)
- **If the user states something that contradicts a documented pattern**,
  verify it; if it does contradict, surface the contradiction with the
  evidence and let them decide knowingly — do not just agree. (M07: agreed
  "D.fix is a mistake"; it was the documented pattern.)

## 4. Communication

### To the user (HITL)

- Adjudication: **brief narrative (2–4 sentences) + the option call-out + one
  concise CLI prompt.** Nothing else.
- Design discussion (the user is reasoning through architecture): substance is
  welcome — organized, still no word-salad.
- Own an error in one line. No grovelling, no defensiveness. Pivot to the fix.
- Don't over-escalate. Don't ask what you can investigate yourself. Give one
  pasteable command, never a flow.
- The user picks a CLI option OR types one response — there is no "alongside."
  If you have something to add, it is a single typed response.
- Never dump options without a recommendation.

### To the build CLI

- Brief and exact. Include **only what it does not already know** — filter
  every line through "does it know this?"
- **The paste block is DO-ONLY.** It contains the directive — what the build
  must act on — and nothing else: no reasoning, no "because", no adjudication
  rationale, no restating what the build already surfaced or already plans to
  do. If a line is not an instruction the build must act on, it does not go in
  the block. ALL editorial goes in the message to the user, OUTSIDE the block.
- A fresh stage session needs the context it lacks; a continuing session must
  not be told what it already surfaced.
- The build executes; it does not orchestrate.

## 5. Process hygiene

- **One instruction in flight.** Before issuing a new instruction, reconcile
  what the build is currently executing. (M07: a reset prompt was sent after
  the user had already sent finish-it.)
- **Trust but verify.** A build surface is a claim of intent — verify against
  `git` / the diff before reporting done.
- **Append-only / grandfather.** gap-analysis is append-only; accepted ADRs
  are immutable; committed phase docs are not edited mid-flight.
- **Error recovery by rule** — salvage branch → reset. Never improvise it.
- **Git truth = origin.** The build PC's local `main` can be stale; verify
  with `git fetch` before reasoning about merge state.

## 6. Standing rules

- Web-research before any medium/significant decision or authoring — pricing,
  API shapes, library / security / UX best practice, third-party schemas.
  Research → decide → document the rationale. (CLAUDE.md §12.)
- Never commit or push without explicit user approval; never push to `main`;
  never open a PR unless explicitly asked.
- CLAUDE.md §4 hard rules apply in full.

## 7. Session model

- Orchestration runs as **fresh, scoped sessions per task** — adjudicate one
  surface, author one doc, run one closeout. Free-flow reasoning lives inside
  each session; the session boundary kills cross-turn drift.
- State lives in artifacts — retros, gap-analysis, git, the §X.5 prompts, and
  §9 below — not in session memory.
- Tell the user to clear the build session at natural boundaries (before G;
  when near the context limit).

## 8. Good / bad — M07 source cases

| Bad | Good |
|---|---|
| D.fix authorized via a relay paragraph — red+impl in one pass, no phase doc, no gate | Build work = a §X.5 stage prompt with gates |
| Started rewriting a §G.5 prompt because it "looked stale" | §X.5 is stable; drop it verbatim; it reads orientation docs |
| Declared `main` broken without a fetch | Verify before escalate |
| Agreed "D.fix is a mistake" — it was the documented pattern | Verify a user claim against the docs; surface the contradiction |
| Reset prompt issued after the user had sent finish-it | One instruction in flight; reconcile first |
| Relays restating what the build already surfaced | Relay only what the build doesn't know |
| Walls of reasoning sent to the user | Brief narrative + option + prompt |
| Reasoning / rationale / restated build-plan inside the CLI paste block | The paste block is DO-only; editorial goes to the user, outside it |

## 9. Current state (live — rewrite at every handoff)

- **Milestone:** M08.6 merged (PR #105 — framework-representation; ADR-0022 Accepted at Stage B). Post-M08.6 docs merged (PR #106 — `STAGE-PROMPT-PROTOCOL.md` §6 `<approval_surface>` codification + M9 governance/mentor-scope handoff notes + runtime capabilities roadmap + the CI matrix-skip cascade-layer fix).
- **Last completed:** post-M08.6 IRL walk (`docs/M08.6-irl-findings.md` — 32 findings, 7🔴; load-bearing finding: the runtime *paints* most of what it claims — only provider-streaming + MCP execute; built-in tools / sub-agents / skills / gaps / plans / hooks do not). **Process reset:** CLAUDE.md §4 rule 11 (grounded-claims / no-gaslighting) + `docs/cluster-pattern.md` (cluster-gate close discipline) + zero-propagation in §3 above + `DESIGN.md` rules seed. PRs #106/#107/#108/#109 merged.
- **Next action:** **M08.7.A rung 1** (build-side — the built-in tool executor; CLI prompt in the phase doc §1.5). **Phase 0a is CLOSED:** cluster-gate codified (`docs/cluster-pattern.md`) + zero-propagation wired (§3) + v1.9 protocol fold-in (`<close_gate>` + cluster-cycle step names + the Stage V 5th `assembled_execution` pass + V template) + M08.6 retro graduations applied (gotchas #88–91 + #75 clarification + the Stage-0 routing template slot) + Stryker spec'd (`cluster-pattern.md` §5; build-side install pending — NOT needed for the Rust M08.7a). **Pre-M9 plan (confirmed):** Phase 0 (0a done; 0b DESIGN.md visual via Claude Design — user-driven, before the Builder UI track) → **M08.7 Execution Engine** (the eval-first ladder; make the runtime RUN; ends at *ARIA runs* the v0.1 subset) → **M08.6.7 Builder correctness** (the 32 findings as clusters) → **M9 mentor**.
- **Open threads:**
  - Claude Design / DESIGN.md / Stage D protocol scaffolding (master plan steps 10–12 — `STAGE-DESIGN-REVIEW-PROMPT-TEMPLATE.md`, `DESIGN-REVIEW-RETROSPECTIVE-TEMPLATE.md`, `docs/design-quality.md` append-only ledger, CLAUDE.md §19 retro extension, STAGE-PROMPT-PROTOCOL `<design_review_stage_prompt>` schema; first Stage D run is retroactive on M08.6).
  - Starter-kit reconciliation gate (`docs/methodology-delta.md` ↔ `RUNTIME-DELTA.md`) — both delta logs maintained five-minute-weekly; reconciliation reads happen before authoring the M9 phase doc.
  - The post-M08.6 findings are **clusters in M08.7 / M08.6.7, NOT M9 intake** (zero-propagation): execution gaps (built-in tools / sub-agents / skills / gaps / plans / hooks — incl. `plan_loop` rung 7, `TestOutcome.vdr` rung 6) → M08.7; Builder 🔴/🟡 (#12 recovery, #17 template, #19 tier-desync, #22 budget-persist, #23 MCP-spawn, #32 save-companions, validation/UX) → M08.6.7. #31 narrowing = strict-narrowing-fix-ARIA at M08.7 rung N (grant/use split = v1.0 ADR).
  - M08.6 retro graduations — **APPLIED** (the retro→protocol loop is closed): the 3-angle schema/contract reality check → gotchas #90 (HashMap byte-stability) + #88 (wdio dual-generic); angle-1 (schema asymmetry) is covered by the existing #41 grep-verify + the v1.8 shape audits. Stage-0 routing slot → `RETROSPECTIVE-TEMPLATE.md`. e2e-tauri CI-only-locally → gotcha #89. additive-source-dedup-tautology → gotcha #91. useShallow-for-derived-only → #75 clarification. **Deferred-with-rationale:** the phase-doc E.3 "OR vs AND" nuance — 1-milestone-local wording, not a recurred pattern (schema-follows-use; revisit if it recurs).
