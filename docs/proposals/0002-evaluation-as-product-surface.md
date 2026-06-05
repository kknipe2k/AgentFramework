# Proposal 0002 — Evaluation as a Product Surface (Behavioral Regression Across Model Releases)

> **Status:** Proposed (back-pocket → v1.0 scope-candidate). Graduates [`0001-interaction-layer-roadmap.md`](./0001-interaction-layer-roadmap.md) §3.3 from a back-pocket item into a scoped surface, and reframes its open "benchmark vs. demonstrate" question.
> **Trigger:** Han Lee, "Hidden Technical Debt of AI Systems: Agent Harness" (2026-05-08), third in the series; directed review against this runtime. Thesis: production harness scaffolds dissolve into the next model, and **what lasts is the evaluation/task/environment substrate that lets you rebuild harnesses as models change**.
> **Scope:** Post-v0.1 only. None of this lands before M11 ship. Per [§0d Release Scope Matrix](../../agent-runtime-spec.md) additions to v0.1 require equivalent removals — this does not qualify.
> **Author:** kknipe2k (back-pocket exploration; Claude-drafted from a directed review).
> **Tags:** architecture, evaluation, harness-debt, regression, vdr, observability

---

## 1. Context

Lee's "Agent Harness" post is the same genre as the two harness theses already reviewed in this tree — [`0001-interaction-layer-roadmap.md`](./0001-interaction-layer-roadmap.md) (Thinking Machines "the wrapping harness is a dead end") and [`harness-review-takeaways.md`](./harness-review-takeaways.md) (iii + Hermes). All three argue that orchestration scaffolds are a *current-capability* artifact the next model subsumes. Lee's specific, load-bearing claim:

> "What lasts are the training and evaluation data, environments, tasks, and infrastructure: the durable substrate that lets you rebuild harnesses as models change."

A file:line pass over the runtime against that claim found the project is **on the right durable axis** (capability sandbox, live-graph observability, gap-detection/clean-suspend — the narrow-production-harness pattern Lee explicitly endorses; least privilege, deny by default, observe everything). But one axis Lee calls *the most durable* is **present as internal discipline and absent as a product surface**: evaluation that detects behavioral regression across model releases.

This proposal graduates `0001` §3.3 (the "Thought-to-Action correctness benchmark") from back-pocket, and answers its decision-point 3 ("marketing artifact vs. regression-detection engineering artifact") with Lee's framing: it is the **engineering artifact** — the bridge that keeps a user's framework working "when the next model lands."

**Net conclusion: no v0.1 change; a v1.0 scope-candidate to weigh at the post-M11 architecture review.**

---

## 2. The gap, precisely

There are two different objects, and the runtime has only the first:

1. **Inward eval (have it, rigorously).** [`execution-status.md`](../execution-status.md) is the "paints vs. executes" ledger; the `E-NN` evals are assembled-app regressions that flip a primitive to `executes — observed`; CLAUDE.md §4.11 (grounded-claims) forces behavior-over-structure. This is the runtime's-own-code regression surface. It is exactly the eval instinct Lee says is durable — pointed **inward**.

2. **Eval as a product surface (don't have it).** A way for a *user* to pin a task set against *their* framework and detect how its behavior drifts when the model changes. The M8 **Tester** is the embryo, not this: its acceptance criteria (`docs/MVP-v0.1.md` §M8) run **one** task in an isolated session and surface `graph + VDR + token spend + pass/fail` — a single try-it run, not a curated task set, not a baseline, not a cross-version diff.

**Why it matters for a *runtime* specifically.** When Anthropic ships the next model, a user's framework behavior shifts — success rate, capability-violation rate, gap rate, cost, latency all move. Today the runtime has **no surface that tells the user how it shifted**. Lee's point is that this regression-detection surface is what separates an agent product that "still works when the next model lands" from one whose owner spends a quarter discovering the drift by hand.

This is **not a code bug to fix in v0.1** — it is a scope/positioning decision against §0d. Recording it as a gap-analysis-class omission (product↔spec, per CLAUDE.md §20) is the immediate action; building it is a v1.0 question.

---

## 3. Why the runtime is unusually well-positioned

The expensive part of an eval harness is the run substrate — isolated, forkable, replayable execution with full measurement. The runtime already has it:

- **The measurement signals already exist.** §0a row 7 (Decision trace → built-in VDR projection from the event stream) + row 8 (Signal Schema v2, 8 signal types) + §2b signals/VDR. Per-task metrics an eval needs — success, capability-violation count, gap count, tool-hallucination count (a call to a non-existent tool), audit completeness, tokens, wall-time — are a **projection over signals already emitted**, not new instrumentation.
- **Forkable / snapshottable / replayable state.** §1b recovery rebuilds from append-only SHA-chained drone snapshots without re-executing tools. Lee's training/eval-harness column lists "State: forkable, snapshottable, replayable" as the property evals want; the runtime already has it for production.
- **Isolated, deterministic-where-possible runs.** The capability sandbox (§7/§8) is the per-run isolation an eval set wants; the Tester already runs in a separate SQLite session.
- **The Tester is the embryo.** The surface is "Tester **× a pinned task set × across model versions × with a baseline diff**," not net-new plumbing.

This is the same split [`runtime-capabilities-roadmap.md`](./runtime-capabilities-roadmap.md) §5 already endorses — orchestration + safety + **observability** live in the runtime. Evaluating behavior over time is the observability axis extended across the time dimension.

---

## 4. The surface (sketch, not commitment)

Per the `0001` §3 convention, this is an external-pattern sketch for the implementing milestone, not a validated design:

- **Eval-set artifact.** A pinned, versioned list of tasks — `{ task description, optional fixture workspace, expected-outcome assertion or judge rubric }` — stored alongside the framework. Candidate `schemas/eval.v1.json` (typify-generated types per §14; ADR per §11 for a new schema). This *is* Lee's "tasks/environments as durable substrate," made a first-class artifact.
- **Runner.** Execute the framework's agents against the eval set in the Tester's isolated-session mode; collect the VDR-derived metric vector per task.
- **Baseline + diff.** Persist a run as a baseline; re-run on a new `model` id (or after a framework edit); diff the metric vectors; surface regressions. This reuses the `E-NN` "assert the observed behavior, not the painted event" discipline — turned outward.
- **Verifier modes** (Lee's "verifier" row). Programmatic assertions where deterministic; LLM-as-judge rubric where not. Judges run under the same capability posture as any other run (no privileged escape; injection-hygiene per §8.security).
- **Optional public dogfood (the other half of `0001` §3.3).** Ship a reference eval set over `examples/aria/` and run it per release tag, so the project itself regression-tests behavior across Anthropic model releases. Useful, but **the user-facing per-framework surface is the load-bearing part**; the public benchmark is a follow-on.

---

## 5. Why this is the *appreciating* axis (and de-risks everything else)

Lee's verdict on this product class: orchestration scaffolds (planner FSM, multi-agent graph, generators) dissolve; governance + observability + **eval substrate** appreciate. This surface is the appreciating one — and it is also the **enabling condition** for treating the rest of the harness as a 90-day artifact:

> "organize their code so they can [delete most of it on a model release] without flinching."

You can only delete a scaffold on a model boundary "without flinching" if something tells you whether the deletion broke observed behavior. A per-framework eval/regression surface is that something. So this proposal is not only the most durable item — it is the precondition that makes the project's *other* (correct) instinct to keep scaffolds removable actually safe to act on. Concretely, it is what would let a future cycle retire or thin the `drive_plan` FSM, the fixed `examples/aria` agent pipeline, or the M9 generator-as-batch-authoring step against evidence rather than hope.

---

## 6. Out-of-scope locks (do NOT smuggle into v0.1, or let this proposal drift into)

- **No training / RL loop.** This is *evaluation* — read-only over runs. It must not become a self-tuning loop; "the runtime executes what exists, it doesn't modify itself" ([`runtime-capabilities-roadmap.md`](./runtime-capabilities-roadmap.md) §3.4). Self-improvement-on-eval-scores stays a v2.0+ ADR question.
- **No new always-on telemetry.** Evals are local, user-invoked, no phone-home (CLAUDE.md §4.4 / spec §13). Results live in the user's data dir like any other run output.
- **Cross-version diff respects the Anthropic-only lock.** Cross-*Anthropic-version* diff (e.g., the model id of the day vs. its successor) is feasible inside v0.1's provider lock; cross-*vendor* diff waits on v1.0 multi-provider ([`runtime-capabilities-roadmap.md`](./runtime-capabilities-roadmap.md) §3.1).
- **Does not displace [`execution-status.md`](../execution-status.md).** That ledger stays the runtime's-own-code surface (does primitive X execute). This is the user's-framework surface (did framework Y's behavior regress). Two different objects; both wanted.

---

## 7. Decision points before formal proposal

These need maintainer adjudication before this graduates to a milestone phase doc:

1. **v1.0 commitment vs. back-pocket.** Recommendation: scope *eval-set schema + Tester-over-a-set runner + baseline/diff* as an **early v1.0 milestone** — it is the lowest-cost, highest-durability item, and it de-risks every scaffold-deletion decision after it.
2. **Eval-set schema home.** New `schemas/eval.v1.json` (separate, independently versioned — recommended; Lee wants tasks as a first-class durable artifact) vs. a framework-JSON sub-block. Either way, a schema change is an ADR (§11/§14).
3. **Runner reuse vs. headless path.** Reuse the M8 Tester modal vs. a headless `agent-runtime-cli eval` invocation (pairs with the deferred headless CLI — `docs/MVP-v0.1.md` §M7 out-of-scope). The headless path is what makes the per-release CI dogfood (§4 last bullet) clean.
4. **Public dogfood benchmark.** Ship + run a reference eval set over `examples/aria/` per release? Recommendation: yes, but as a follow-on to the user-facing surface, not before it.

---

## 8. References

### Trigger

- [Han Lee — "Hidden Technical Debt of AI Systems: Agent Harness" (2026-05-08)](https://leehanchung.github.io/blogs/2026/05/08/hidden-technical-debt-agent-harness/) — this proposal's trigger.
- [Han Lee — "Hidden Technical Debt of AI Systems: Agent Runtime" (2026-04-24)](https://leehanchung.github.io/blogs/2026/04/24/hidden-technical-debt-agent-runtime/) — the preceding post in the series (runtime + sandbox primitives).

### Internal cross-references

- [`0001-interaction-layer-roadmap.md`](./0001-interaction-layer-roadmap.md) §3.3 — the back-pocket item this graduates; decision-point 3 ("benchmark vs. demonstrate").
- [`harness-review-takeaways.md`](./harness-review-takeaways.md) — sibling external-harness-thesis review (iii + Hermes).
- [`runtime-capabilities-roadmap.md`](./runtime-capabilities-roadmap.md) §3.1 (multi-provider), §3.4 (no self-modification), §5 (orchestration + safety + observability split).
- [`../execution-status.md`](../execution-status.md) — the inward "paints vs. executes" / `E-NN` eval ledger this surface is the outward analog of.
- `agent-runtime-spec.md` §0a rows 7–8 (VDR + Signal Schema v2), §1b (recovery/replay), §2b (signals/VDR), §7/§8 (sandbox/capability), §13 (privacy/no telemetry).
- `docs/MVP-v0.1.md` §M8 (Tester — the single-run embryo), §M7 (deferred headless CLI).

---

## 9. Next steps

1. Maintainer review of this proposal.
2. If accepted: record the omission as a gap-analysis-class entry (product↔spec, CLAUDE.md §20) against §0d, and add this surface to the post-M11 v1.0 scope discussion.
3. If accepted as a v1.0 milestone: draft `schemas/eval.v1.json` + the ADR (§11/§14) at the scoping cycle; size the runner against the existing M8 Tester.
4. If rejected: archive with a one-line rationale; no further action.

Until then this is a back-pocket artifact — referenced when the post-v0.1 scope conversation opens, not driving any in-flight milestone work.
