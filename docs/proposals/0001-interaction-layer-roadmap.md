# Proposal 0001 — Interaction Layer Roadmap (Post-v0.1 Scaffolding)

> **Status:** Proposed (back-pocket).
> **Trigger:** Thinking Machines Lab released "Interaction Models" on 2026-05-11; thesis is "the wrapping harness is the wrong layer to bet on."
> **Scope:** Post-v0.1 only. None of the items here land before M11 ship. See [§0d Release Scope Matrix](../../agent-runtime-spec.md) — additions to v0.1 require equivalent removals, and these items don't qualify.
> **Author:** kknipe2k (back-pocket exploration; Claude-drafted with web-research backing).

---

## 1. Context

On 2026-05-11, Mira Murati's Thinking Machines Lab released **Interaction Models** + a model called TML-Interaction-Small (276B-param MoE; 12B active). Their explicit thesis:

> "The prevailing approach to interactive AI, in which a turn-based language model is wrapped in a harness that handles speech detection, interruption and latency tricks, is a dead end."
>
> "As the language model improves, the gap between what the model could do conversationally and what the surrounding harness lets it do widens."

Their architecture is a custom-trained model where listen / speak / see / pause are token-level decisions inside the model, with 200ms micro-turns replacing request-response. Two-model split: an "interaction model" stays live with the user; a "background model" handles reasoning + tool use asynchronously, sharing full context.

**Critical distinction for this project:** Thinking Machines is competing on the MODEL ARCHITECTURE. The 200ms micro-turn is a property of the model, not of any wrapping infrastructure. The runtime in this repo wraps Claude (an existing turn-based model). The mimicry options are wrapper-layer patterns that work AROUND an existing model, NOT model-architecture replacements.

This proposal evaluates what's legitimately mimicable, calls out marketing-language vapor, and frames the strategic positioning question for the runtime's post-v0.1 trajectory.

---

## 2. Honest assessment

### 2.1 Mimicable in a wrapping runtime (real, researched, evidence-backed)

1. **Background-Critic sidecar pattern.** Spawning a parallel agent session (a second Claude instance) that observes the main agent's live event stream and emits its own events (`critic_disagreed`, `critic_suggested_pivot`). Empirical evidence from Berkeley BAIR's *Adaptive Parallel Reasoning* (May 2026) and Nebius's *Reasoning critics enable better parallel search for software engineering agents*:
    - Parallel verification loops outperform chain-of-thought by 37% on complex reasoning benchmarks.
    - Catch 52% more logical errors.
    - Converge 3× faster.
   The runtime's existing main+drone architecture maps directly — a sidecar Claude session is an additional process, not a model change.

2. **Streamed-observation + parallel tool dispatch during a turn.** The runtime already has SSE-stream observation via M02's `anthropic_sse`. Adding hooks where the runtime can dispatch parallel tools (linter, type-check, capability check, partial-result validators) WHILE the main stream is in flight is a legitimate wrap-layer mimicry. Not the same as Thinking Machines' intra-model interruption — Claude's turn boundary is the boundary — but you can cancel-and-reissue with updated context, which is the wrap-around equivalent.

3. **Context-compression / pivot triggers.** Forcing summarization or pivot when context approaches limits is standard practice (Anthropic Claude Code has compaction built in). The runtime already has snapshots + projection from M01/M04 — the pivot trigger is mostly UI surface + threshold-based hook + an existing tools to compress.

### 2.2 Vapor / marketing-language (do NOT chase)

- **"Single clock cycle for input/output"** — hardware/model property, not wrappable. Misleading.
- **"Self-aware markdown scaffolding"** — this is marketing language for a state machine that tracks agent context and adapts. Standard pattern; the project already has this via the SQLite signals table + projection + IPC events. Not novel.
- **"Memory pressure system that forces the agent to summarize or pivot"** — re-statement of context compression / hook triggers. Standard.
- **"MD scaffolding as the 2026 standard protocol"** — Microsoft Aspire's blog explicitly counters: "Markdown — skills, prompts, custom instructions — are unpredictable, require constant maintenance, can't be compiled or tested, and cost tokens just to parse." Credible counter-argument worth weighing before betting harder on MD.
- **"Open-source as the standard protocol"** — aspirational marketing, not technical strategy.
- **"X video titled 'Why Thinking Machines is right'"** — marketing tactic, not architecture.
- **"Months 1-6 release plan"** — generic open-source playbook; not specific to the runtime's design.

### 2.3 Strategic risk — the harness-bet

Thinking Machines explicitly argues the scaffolding/harness IS the wrong layer to bet on long-term. If this project competes on "best scaffolding for Claude," it's betting that frontier models WON'T eventually subsume the scaffolding's role via native interaction abilities. That's a real bet that should be conscious.

**Counter-positioning option to weigh at v1.0 architecture review:** the runtime is "the model-agnostic harness that survives the harness-being-subsumed trend by being the right harness right now AND a smooth migration target when frontier models grow native interaction." That framing keeps the runtime relevant whether Thinking Machines' model bet pays off or not — but requires investment in model-agnostic interfaces (post-v0.1 multi-provider work, per §0d v1.0 scope).

---

## 3. Roadmap items

Each item is sized as a post-v0.1 feature; none land before M11 ship. Detailed phase docs would land at the milestone where the item is scoped.

### 3.1 Background-Critic sidecar (target: v1.0+)

**What:** A second Claude session running in parallel with the main agent loop, observing the live event stream emitted by `runtime-main` and emitting its own events (`critic_observation`, `critic_disagreed`, `critic_suggested_pivot`, `critic_blocked`).

**Why:** Parallel-critic patterns improve correctness measurably (52% more logical errors caught per BAIR + Nebius research). The runtime is the right substrate — main+drone+sandbox architecture extends naturally to main+drone+sandbox+critic. Mirrors Thinking Machines' interaction-vs-background-model split at the wrapper layer.

**Scope:**

- New crate `runtime-critic` (or `runtime-main::critic` module — TBD by architectural review).
- Subscribes to the same event stream the renderer + audit log consume (no new IPC primitive).
- Spawns + manages a separate Claude session via `AnthropicProvider`.
- Emits a new event variant family (`critic_*`) extending `schemas/event.v1.json`.
- Renderer surface: a `CriticPanel` showing critic observations alongside the main agent stream.

**Dependencies:**

- M02 (event pipeline) ✓ landed
- M04 (HITL primitives — critic suggestions may route through `on_critic_disagreed` trigger) ✓ landed
- M11 (v0.1 ship) — must precede
- Possibly post-v1.0 multi-provider (if the critic uses a different model than the main agent for diversity)

**Cost:** ~one milestone (5-7 stages; estimate similar to M05 or M06 scope). Roughly 30-40 hours code execution.

**Open question:** does the critic share context with the main agent, or does it run in isolation observing only the emitted event stream? TML's interaction+background split SHARES full context; the runtime's drone-as-audit-log pattern (ADR-0007) suggests EVENT-STREAM-ONLY would fit better architecturally. Decision deferred to the milestone phase doc.

### 3.2 Streamed-observation hooks (target: post-M11; possibly bolt onto M02-equivalent in v1.0)

**What:** Hook surface at the M02 `anthropic_sse` boundary where the runtime can dispatch parallel tools (linter, type-check, capability re-verification, partial-result validators) WHILE the main Claude stream is in flight.

**Why:** Closes the "linter result lands AFTER the agent already wrote the next block" gap. Doesn't require model changes. Existing SSE plumbing is the dispatch point; existing tool-invocation infrastructure is the consumer.

**Scope:**

- New `StreamObserver` trait in `runtime-main::sdk` accepting partial-token batches.
- Built-in observers: capability-violation pre-check, type-check trigger (if the stream is writing code), partial-result diff against the prior turn.
- Surface to the renderer: observer findings appear as ephemeral notifications, not committed to the audit log unless they trigger a HITL.
- Integration with the existing HitlSeam — observers can request user attention via the M04.E `on_observer_finding` trigger (new trigger variant).

**Dependencies:**

- M02 anthropic_sse ✓ landed
- M04 HitlSeam ✓ landed
- M05 capability enforcer ✓ landed
- M11 (v0.1 ship) — must precede

**Cost:** ~one milestone (smaller than 3.1; ~20-25 hours).

**Open question:** how aggressive should the observer be? Cancel-and-restart-stream on a finding is heavy; surface-and-continue is light but may miss the chance to redirect. v1.0 default surface-and-continue; v2.0 explore cancel-and-restart for specific finding classes.

### 3.3 Thought-to-Action correctness benchmark (target: post-M11, public)

**What:** A reproducible benchmark suite comparing the runtime's loop-hallucination / wrong-tool-call / capability-violation rates against raw Claude prompting (no runtime, just the Anthropic API directly).

**Why:** This is the only "best path forward" item from the pasted text with genuine technical substance. Public benchmarks are how harness-layer projects prove value to skeptical maintainers + the broader community. Anthropic's own TDD guidance (verified at M06.C TDD-discipline research) cites measurable defect-rate improvements as the bar; this benchmark would put the runtime on the same scale.

**Scope:**

- Curated task suite: 50-100 tasks across categories (file editing, multi-step refactor, capability-respecting operations, gap-detection scenarios, etc.).
- Two execution modes: (a) raw Anthropic API directly, (b) through the runtime.
- Measurement: success rate, hallucination rate (calls to non-existent tools), capability-violation rate, audit completeness, time-to-completion.
- Reproducible CI workflow that runs the benchmark on every release tag.
- Public results page (likely hosted alongside the docs site).

**Dependencies:**

- Runtime stable enough to run the benchmark (post-M11 v0.1 ship).
- Anthropic API access (already in scope).
- Curated task suite authoring — significant upfront cost; needs maintainer-led design.

**Cost:** Two-part. (a) Initial benchmark suite authoring + harness: ~one milestone (~25-30 hours). (b) Ongoing maintenance: per-release CI run + result analysis.

**Open question:** task curation methodology. Existing benchmarks (SWE-bench, HumanEval, etc.) are mostly model-centric, not harness-centric. The runtime's value-add is capability-respecting + audit-producing + recovery-capable execution; the benchmark must measure those specifically.

---

## 4. Out-of-scope locks (do NOT smuggle these into v0.1)

- 200ms micro-turns or any intra-model timing knob — Claude's turn boundary is the boundary; not negotiable from the wrapper layer.
- Custom model training — Thinking Machines' bet; not this project's.
- Multi-provider support — locked to Anthropic-only in v0.1 per §0d; expand only at v1.0 architectural review.
- Voice / vision modalities — out of v0.1 + v1.0 scope; text-only.
- Multi-session — out of v0.1; v1.0 architecture call.

---

## 5. Decision points before formal proposal

These need maintainer adjudication before any of the three roadmap items graduate to a milestone phase doc:

1. **Harness-bet vs. counter-positioning.** Does the project formally adopt the "model-agnostic harness with migration path" framing at v1.0 architecture review, or stay with "best harness for Claude"? This decision shapes whether 3.1 and 3.2 are short-term value or long-term defensive moats.

2. **Critic-share-context vs. event-stream-only.** Affects 3.1's architectural shape. The drone-as-audit-log pattern (ADR-0007) argues for event-stream-only; TML's interaction/background architecture argues for shared context.

3. **Benchmark vs. demonstrate.** Is 3.3 a marketing artifact (compare-favorably-to-raw-prompting for public proof) or an engineering artifact (regression detection per release)? Both are valuable; the curation + investment differs significantly.

---

## 6. References

### Source — Thinking Machines Lab (2026-05-11 release)

- [Thinking Machines Lab — Interaction Models (canonical blog post)](https://thinkingmachines.ai/blog/interaction-models/)
- [MarkTechPost — Mira Murati's Thinking Machines Lab Introduces Interaction Models](https://www.marktechpost.com/2026/05/13/mira-muratis-thinking-machines-lab-introduces-interaction-models-a-native-multimodal-architecture-for-real-time-human-ai-collaboration/)
- [TechCrunch — Thinking Machines wants to build an AI that actually listens while it talks](https://techcrunch.com/2026/05/11/thinking-machines-wants-to-build-an-ai-that-actually-listens-while-it-talks/)
- [VentureBeat — Thinking Machines preview of near-realtime AI voice and video](https://venturebeat.com/technology/thinking-machines-shows-off-preview-of-near-realtime-ai-voice-and-video-conversation-with-new-interaction-models)
- [The Decoder — Thinking Machines argues interactivity is what OpenAI gets wrong](https://the-decoder.com/thinking-machines-lab-ships-its-first-model-and-argues-interactivity-is-what-openai-gets-wrong-about-voice/)

### Parallel-reasoning + critic patterns (empirical evidence)

- [Berkeley BAIR — Adaptive Parallel Reasoning (2026-05-08)](https://bair.berkeley.edu/blog/2026/05/08/adaptive-parallel-reasoning/)
- [Nebius — Reasoning critics enable better parallel search for software engineering agents](https://nebius.com/blog/posts/reasoning-critics-parallel-search-for-agents)
- [AI Crucible — Parallel Verification Loops: The Future of AI Reasoning](https://ai-crucible.com/articles/parallel-verification-loops/)
- [MarkTechPost — Streaming Decision Agent with Partial Reasoning, Online Replanning, Reactive Mid-Execution Adaptation (2026-03)](https://www.marktechpost.com/2026/03/11/how-to-design-a-streaming-decision-agent-with-partial-reasoning-online-replanning-and-reactive-mid-execution-adaptation-in-dynamic-environments/)

### Harness engineering perspectives

- [LangChain — The Anatomy of an Agent Harness](https://www.langchain.com/blog/the-anatomy-of-an-agent-harness)
- [OpenAI — Harness Engineering: leveraging Codex in an agent-first world](https://openai.com/index/harness-engineering/)
- [Microsoft Aspire — Agentic development aspirations: build, run, observe — without more Markdown](https://devblogs.microsoft.com/aspire/agentic-dev-aspirations/) (credible counter-argument to MD-heavy scaffolding)

---

## 7. Next steps

1. Maintainer review of this proposal.
2. If accepted: each roadmap item gets formally tracked in a v1.0 / v2.0 scope discussion at the appropriate architecture review point (post-M11 ship for v1.0 planning).
3. If rejected: archive this file with a one-line note explaining the rejection rationale; no further action.
4. If partially accepted: identify which items advance to scope-discussion + which stay back-pocket.

Until then, this file is a back-pocket artifact — referenced for context when the post-v0.1 scope conversation opens, not driving any in-flight milestone work.
