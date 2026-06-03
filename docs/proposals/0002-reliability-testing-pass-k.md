# Proposal 0002 — Reliability Testing (`pass^k`) for Built and Generated Artifacts

> **Status:** Proposed (back-pocket).
> **Trigger:** Product question (2026-06-03) — "would it make sense to use a Monte Carlo test with τ-bench to test skills and agents before deploying them?" The intuition maps directly onto a metric (`pass^k`) and a methodology (simulated-user + state-oracle) the agent-eval field has already converged on.
> **Scope:** Post-v0.1 only (v1.0+). Nothing here lands before M11 ship. Per [§0d Release Scope Matrix](../../agent-runtime-spec.md) — v0.1 is single-session, STANDARD-mode, Anthropic-only; multi-trial reliability testing brushes all three locks and does not qualify as a v0.1 addition without equivalent removals.
> **Author:** kknipe2k (back-pocket exploration; Claude-drafted with web-research backing).

---

## 1. Context

Skills and agents are **stochastic** systems: the same input produces different
outputs across runs (model sampling, tool-call ordering, retries, intermittent
tool failures). The runtime's existing pre-deploy check — the Phase 9 **Tester**
([`agent-runtime-spec.md` §9, line ~2529](../../agent-runtime-spec.md)) — runs a
framework against a single test task and reports one pass/fail. That answers
*"did it work once?"*, which the agent-eval literature now shows is close to
uninformative for predicting deployment behavior.

Two external findings frame the gap:

- **Single-run estimates are noise-dominated.** *On Randomness in Agentic Evals*
  (arXiv 2602.07150) found single-run `pass@1` swings **2.2–6.0 percentage
  points** depending on which run is sampled, with std-dev > 1.5pp **even at
  temperature 0**. A one-shot pre-deploy gate can pass a regression or fail a
  good build by luck alone.
- **`pass^k` is the reliability metric, and it comes from τ-bench.**
  [τ-bench](https://arxiv.org/abs/2406.12045) (Sierra Research, 2024) introduced
  `pass^k` — run the same task `k` times, require **all `k`** to pass — to
  measure reliability over multiple trials. Its headline finding (GPT-4o-class
  agents at **`pass^8` < 25%** in retail) is the argument for why single-shot
  testing misleads. [τ²-bench](https://github.com/sierra-research/tau2-bench)
  extends it (telecom domain, dual-control, code fixes).

τ-bench contributes **three transferable mechanisms** (the *methodology*
transfers; the *datasets* — retail/airline/telecom customer-service — do not fit
the runtime's workflow/coding agents):

1. **Simulated user** — a second model role-plays the user across a multi-turn
   conversation, testing dynamic interaction rather than a frozen prompt.
2. **State-oracle scoring** — success is judged by comparing the **final
   database state to an annotated goal state**, *not* by an LLM judging the
   transcript. This is a deterministic success predicate, which sidesteps the
   "judge adds its own variance" failure mode.
3. **`pass^k`** — the multi-trial reliability metric above.

---

## 2. The seam already exists

This is not a new subsystem — it is a generalization of two already-specced
mechanisms from one-shot to multi-trial.

### 2.1 Phase 9 Tester (build-time, ships v0.1)

The Tester already provides every property a multi-trial harness needs **except
the "many trials" part** ([`spec` §9, ~2529](../../agent-runtime-spec.md)):

- Runs each trial in an **isolated session with a separate SQLite database**
  (drone-managed sandbox, §1c) — independent, side-effect-free, repeatable
  trials are the hard part of Monte Carlo evaluation, and the spec already
  mandates the isolation.
- Test runs **do not write to user data**; results discarded on close.
- Surfaces pass/fail with full trace, **token spend + timing vs benchmarks**,
  and capability violations as test failures (defaults applied — never blocks on
  HITL, so trials never stall).

Because each trial already runs against its own SQLite DB, the runtime can use
**DB/VDR end-state as the oracle** exactly as τ-bench does — no LLM judge
required.

### 2.2 Phase 8 Generators → L3 Sandboxed Validation (the generate-then-test loop)

The LLM-generates-from-scratch path is where multi-trial testing fits **best**,
and it is already half-wired. The **Phase 8 Generators** (Tool Writer / Skill
Writer / Agent Composer, [`spec` §8, ~2255](../../agent-runtime-spec.md)) feed
into **L3 Sandboxed Validation** ([`spec` §8.security L3, ~2334](../../agent-runtime-spec.md)),
which today runs declared examples + adversarial inputs in a drone-spawned
sandbox — **once** — before install. That is already a generate-then-test loop;
it just runs `k = 1`.

---

## 3. The proposal

A **v1.0 "Reliability Tester"** — generalize the existing one-shot machinery to
`pass^k`, with three concrete changes:

### 3.1 Tester → multi-trial (build-time)

| Today (v0.1 Tester) | Reliability Tester (v1.0) |
|---|---|
| 1 test task → 1 pass/fail | `N` trials → `pass^k` (all-pass) + `pass@k` (any-pass) |
| Token spend (single number) | Token/latency **distribution** + p95 cost |
| Capability violation = test fail | Violation **frequency** across trials (a 1-in-30 rail trip is invisible to single-shot) |
| Single frozen NL task, defaults applied | Optional **τ-bench-style simulated user** for multi-turn skills; optional **injected tool-failure** perturbation (wired through the §4a Verify/Rails hooks) |

`N` is **explicit and user-set**, with **projected spend shown before running**
(consistent with §13's no-surprise-cost posture and the Tester's existing
"check token spend" line).

### 3.2 `pass^k`-gated L3/L4 (the generated-artifact gate)

- **L3** stays always-on; make its trial count configurable. Keep the
  always-on path at `k = 1` or low-`k` to bound generation cost; reserve high-`k`
  for the promotion gate (3.3). Store `pass^k` in the existing
  `validation_report`.
- **L4 auto-accept gets honest.** The Promoted-tier auto-install criterion is
  currently *"validation passed"* ([`spec` L4, ~2355](../../agent-runtime-spec.md)).
  It becomes *"validation passed **`pass^k ≥ threshold`**."* A skill that works
  1-in-1 but 22-in-30 should **not** silently auto-install — under today's gate
  it would. This is squarely the §8.security safety property, not gold-plating:
  it closes a real auto-accept hole for LLM-generated artifacts.

### 3.3 Reliability as a promotion signal + a self-correcting loop

- A **reliability score gates tier promotion** (Novice → Promoted, §15a sharing
  tiers). *"Passed 28/30 trials at p95 cost X"* is a far more honest promotion
  gate than *"ran once in the canvas."*
- **The generate → eval → regenerate loop closes.** If `pass^k` is low, feed the
  failing trials back to the generator to revise (the `CLAUDE.md` §5 "eval-first
  ladder", applied to generated artifacts). The **provenance block already
  records `generator` + `model` + `prompt_hash`**
  ([`spec` L5, ~2373](../../agent-runtime-spec.md)) — so a reliability score
  attaches to a generation lineage for free.

---

## 4. Honest assessment

### 4.1 What genuinely fits

- The **isolation-per-trial** property (separate SQLite per test session) is the
  expensive prerequisite, and it is already specced and shipping in v0.1.
- The **state-oracle** (DB/VDR end-state) is already produced per test session —
  it is the right success predicate, and it avoids an LLM judge that would
  Monte-Carlo its own noise floor.
- The **generate-then-validate** path (Phase 8 → L3) is already a one-shot
  version of exactly this loop. Multi-trial is a parameter change plus
  aggregation, not new architecture.
- **Differentiation:** the multi-trial reliability literature (τ-bench,
  ReliabilityBench) lives in *offline benchmark harnesses*. No agent-*builder*
  product I found bakes a `pass^k` gate into the "before you promote this skill"
  flow. The runtime's (isolated Tester) + (tier-promotion gate) + (framework-
  supplied verify oracle) combination is unusually well-positioned to be first —
  the pieces are individually specced; this composes them.

### 4.2 What does not fit / honest limits

- **Cost.** `N` live trials = `N`× real Anthropic calls per validation. L3 is
  "always-on"; making it `k`× by default would blow up generation cost. High-`k`
  belongs at the L4 promotion gate, not the always-on L3 path.
- **Declarative-only artifacts.** Generators emit *declarative* artifacts, never
  executable code ([`spec` §8, ~2259](../../agent-runtime-spec.md)). "Running a
  trial" means exercising the skill instruction-set / tool-binding **through the
  model** — which is what costs the calls. Correct, just named.
- **Methodology, not dataset.** τ-bench's domains are customer-service; the
  runtime's agents are workflow/coding. Borrow the *three mechanisms*
  (simulated-user, state-oracle, `pass^k`); do **not** promise τ-bench's
  leaderboard or suites. Overselling this as a τ-bench drop-in would be a
  category error.
- **Scope locks.** Multi-trial brushes v0.1's single-session, STANDARD-mode, and
  Anthropic-only locks (§0d). This is a v1.0+ feature; it does not get smuggled
  into the current M-series.

---

## 5. Open questions (for the v1.0 scope conversation)

1. **Default `k` and threshold per tier.** What `pass^k` floor gates Promoted
   auto-accept? (τ-bench reports `pass^8`; the runtime's tasks differ — the
   threshold is an empirical, per-domain call, not a constant to hardcode.)
2. **Where the simulated user lives.** A second Claude session as the sim-user
   doubles per-trial cost and adds its own variance. Is the sim-user opt-in
   (multi-turn skills only), or the default test harness?
3. **L3 always-on cost ceiling.** Keep always-on L3 at `k = 1` and reserve
   high-`k` for an explicit "Reliability Test" / promotion action? Or a small
   default `k` (e.g. 3–5) always-on with a spend cap?
4. **Perturbation set.** Which perturbations are in scope — paraphrase the task
   (sim-user), inject tool-failure (Rails/Verify hooks), both? How are they
   declared (framework JSON vs Tester UI)?
5. **State-oracle generality.** τ-bench's DB-state comparison assumes a
   well-defined goal state. For open-ended workflow agents, what is the
   equivalent annotated goal? (VDR assertions? framework-supplied Verify hooks
   as the predicate?)

---

## 6. References

### τ-bench / `pass^k` (the source of the metric + methodology)

- [τ-bench: A Benchmark for Tool-Agent-User Interaction in Real-World Domains (arXiv 2406.12045)](https://arxiv.org/abs/2406.12045)
- [Sierra — τ-bench: shaping the development and evaluation of AI agents](https://sierra.ai/blog/tau-bench-shaping-development-evaluation-agents)
- [τ²-bench (GitHub, Sierra Research)](https://github.com/sierra-research/tau2-bench)
- [τ²-bench Telecom leaderboard (Artificial Analysis)](https://artificialanalysis.ai/evaluations/tau2-bench)

### Stochasticity + reliability in agent evals (the "why single-run lies" evidence)

- [On Randomness in Agentic Evals (arXiv 2602.07150)](https://arxiv.org/html/2602.07150)
- [ReliabilityBench: Evaluating LLM Agent Reliability Under Production-Like Stress Conditions (arXiv 2601.06112)](https://arxiv.org/pdf/2601.06112)
- [Harness-Bench: Measuring Harness Effects across Models in Realistic Agent Workflows (arXiv 2605.27922)](https://arxiv.org/html/2605.27922v1)
- [Understanding AI Benchmarks — Shrivu Shankar](https://blog.sshh.io/p/understanding-ai-benchmarks)
- [The Reliability Gap: Agent Benchmarks for Enterprise — Paul Simmering](https://simmering.dev/blog/agent-benchmarks/)

### Internal anchors

- [`agent-runtime-spec.md`](../../agent-runtime-spec.md) — §8 Generators + §8.security L3/L4/L5 (~2255–2384), §9 Visual Canvas and Tester (~2495–2539), §4a Verify & Rails (~1744), §15a sharing tiers
- [`CLAUDE.md`](../../CLAUDE.md) — §5 eval-first ladder / behavior-over-tautology testing discipline
- [`docs/proposals/0001-interaction-layer-roadmap.md`](./0001-interaction-layer-roadmap.md) — sibling back-pocket post-v0.1 proposal (format precedent)

---

## 7. Next steps

1. Maintainer review of this proposal.
2. If accepted: the Reliability Tester advances to the v1.0 scope discussion at
   the post-M11 architecture review; 3.2's `pass^k`-gated L4 is the highest-value
   slice (closes an LLM-generated-artifact auto-accept hole) and could be
   considered first.
3. If rejected: archive this file with a one-line note explaining the rejection
   rationale; no further action.
4. If partially accepted: identify which of 3.1 / 3.2 / 3.3 advance vs stay
   back-pocket.

Until then, this file is a back-pocket artifact — referenced when the post-v0.1
scope conversation opens, not driving any in-flight milestone work.
