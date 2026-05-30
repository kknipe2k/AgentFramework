# Harness Review Takeaways — iii + Hermes

**Status:** Proposed (idea-stage). Two items in-scope-soon (compaction, prompt assembly); one v1.0+ evaluation.
**Date:** 2026-05-30
**Author:** @kknipe2k (Claude-drafted from a directed review of two external harness write-ups)
**Tags:** architecture, context-management, system-prompt, hitl, harness-comparison

## Context

Reviewed two external harness write-ups against this runtime:

1. **iii — "build your own agent harness"** (worker-composition substrate). Thesis: a harness is ~15 separable jobs; ship each as an independently-versioned worker on a shared bus so "build your own" means "swap a worker," not "fork a framework."
2. **Hermes (NousResearch)** — implementation deep-dive against a nine-component harness model (outer loop, context compression, tools, subagents, built-in skills, session persistence, system-prompt assembly, lifecycle hooks, permission/safety).

Both decompose a harness into the same underlying jobs. Same genre as the existing
[`0001-interaction-layer-roadmap.md`](./0001-interaction-layer-roadmap.md)
(reviewing the Thinking Machines "the wrapping harness is a dead end" thesis).

**Net conclusion: no architecture change.** Our deliberate bets hold — local single-user *runtime + workbench* (not a distributed substrate), and security-first locks where the articles advocate maximal swappability (capability enforcement / drone / sandbox / providers are CODEOWNERS-gated Hard-Rule paths per CLAUDE.md §4.8; the iii model treats those as hot-swappable workers, which for our threat model is an anti-goal). A file:line pass over the tree found 13 of iii's 15 jobs present in code, 1 partial (system-prompt assembly), 1 absent (context compaction).

> Build-state caveat: the pass confirmed *where each job's code lives*, not that each is verified working in the assembled app (CLAUDE.md §4.11 — code presence ≠ runtime behavior).

What follows are the items worth tracking. The blueprints below are **external implementation choices** (per the reviewed write-ups), not patterns we've validated — they're starting points for the implementing milestone, not commitments.

---

## Item 1 — Context compaction (gap) + Hermes's compression blueprint

**The gap.** The May 2026 review found **no compaction/pruning/rolling-window code** in the tree; v0.1 ships unbounded message history. Compaction is **not listed in the §0d Release Scope Matrix**. Budget downshift (opus→sonnet→haiku, §2a) is *not* a substitute — it doesn't bound context growth.

**Action:** confirm this is a conscious deferral vs an omission. If deferred, record the milestone home + acceptance criteria (gap-analysis-class item).

**The blueprint (Hermes), for when it's scheduled:**

- Summarize older turns with an auxiliary model — not a naive trim.
- Protect head + tail segments by token budget; prune tool outputs older than a threshold *before* summarizing.
- Size the summary at ~20% of compressed content, with a 2k-token floor and 12k-token ceiling.
- **Compression as a session lifecycle event:** close the current session, spawn a *child* session seeded by the summary, rotate the session ID, record parent→child lineage. Result is a lineage chain, not one repeatedly-rewritten transcript.

**Why it fits us.** Our drone already writes append-only, SHA-256-chained snapshots (`crates/runtime-drone/src/snapshot.rs`) and recovery rebuilds from the latest snapshot without re-executing tools (§1b). Today that chain is linear within a session; the child-session-on-compression idea is a *fork-on-compress* extension that sits naturally on that substrate. Design touch-points: §1b recovery semantics (resume must understand a compression boundary) and §2b signals/VDR (the boundary needs to be an event).

**Priority:** medium. Becomes load-bearing as soon as real sessions run long enough to fill the context window (M9 generated frameworks running multi-agent; sooner if ARIA reconstruction hits long runs).

---

## Item 2 — System-prompt assembly tiering (partial job) + Hermes's three-tier model

**The state.** Job #6 is **partial**: `AgentConfig::system_prompt` exists (`crates/runtime-main/src/providers/mod.rs`), the smoke session hardcodes `None`, and framework-driven assembly is not yet wired (we're mid-M08). §0b already specifies the pieces (Available-skills block, identity preamble, mode paragraph).

**The blueprint (Hermes):** compose the prompt in three explicit tiers so invariants are easy to reason about and prefixes stay cache-friendly:

- **stable** — identity, tool guidance for *enabled tools only*, skills index, environment hints.
- **context** — project/working-dir files; **prompt-injection-scanned before loading**.
- **volatile** — memory/profile blocks, timestamp + model/provider line.

Rebuild is tied to compression / invalidation points, not every turn.

**Two specifics that land for us:**

1. **Cache-friendly prefixes.** We hit Anthropic over raw HTTP+SSE (§2c); a stable→context→volatile ordering makes the stable prefix cacheable, which is direct cost savings via Anthropic prompt caching once assembly is wired. (We do not cache today — this is a forward benefit of the ordering.)
2. **Injection-scan project-context files before injecting.** Reading project files (the runtime's analog to Hermes loading `CLAUDE.md`/`AGENTS.md`) into the prompt is an untrusted-input path; scanning before injection fits our §8.security posture directly.

**Where it lands:** M08/M09 when framework-driven prompt assembly is wired, per §0b.

**Priority:** medium — it's already on the path; adopt the tiering when implementing rather than retrofitting later.

---

## Item 3 — Reactive single-trigger approval pattern (low-priority eval)

iii collapsed per-call approval-resume registrations into **one reactive trigger** that wakes the right session on an approval write — no per-call resume functions to register, no startup re-scan to recover pending approvals. Our HITL is a oneshot-channel `HitlSeam` (`crates/runtime-main/src/hitl/seam.rs`) + 9-trigger policy + recovery-from-snapshot.

**Action:** evaluate the single-reactive-trigger shape as a simplification of HITL-resume — specifically because we care about clean resume after suspend, and "no startup re-scan to recover pending approvals" is a property our recovery path would want.

**Priority:** low. Pure internal simplification; revisit if HITL-resume complexity bites.

---

## Validated — no action

Two external references independently confirm existing designs:

- **Tool registration vs. exposure are separate** (Hermes: central registry vs. per-run visible set, narrowed for delegated runs) = our capability model + Agent→Agent capability narrowing (`crates/runtime-main/src/capability/`). Already done.
- **Hooks run independently of model cooperation** (Hermes) for policy/audit = our default-deny capability enforcement (§8 L2a). Already done.

Explicit anti-goal recorded: iii's maximal-swappability substrate (swap the policy engine / capability gate / credential vault as workers, any language, thin-vs-thick as a config slider) is the **opposite** of our bet. For a security-first local runtime those layers are the boundary, not interchangeable parts — and "trust the model" thin mode (no enforcement) is not a configuration we ship.

## Out of scope / different product class

Hermes's gateway (Telegram/Slack/…), multi-surface session plane, FTS5 `session_search`, profiles, and cron-as-first-class target a *persistent, multi-channel, unattended* agent server. They map to things we've deferred: multi-session → v1.0, continuous/unattended loop → v1.0 (examples/ralph/), multi-provider → v2.0 (§0d). One inversion: the **durable child-run control plane** Hermes says it *lacks* (children die with the parent) is something our **drone** — an out-of-process lifecycle owner that survives main crash — is unusually well-positioned to provide later. Still v1.0+ (concurrent multi-agent), per [`runtime-capabilities-roadmap.md`](./runtime-capabilities-roadmap.md) §1.2.

## Web research to do at implementation time

**Do NOT pre-research now** (per CLAUDE.md §12 + gotcha #32). When the implementing cycle is dispatched, web-verify current shapes:

- **Compaction:** Anthropic `POST /v1/messages/count_tokens` current shape (§2c.3 swaps the stub to it at M04); summarization-model choice + cost; whether a child-session fork interacts with any Anthropic conversation-state assumptions.
- **Prompt tiering:** Anthropic prompt-caching `cache_control` current API + breakpoint rules + minimum cacheable prefix size; verify the stable/context/volatile boundary aligns with cache breakpoints.

## Related

- §0d Release Scope Matrix — scope horizons (single-session, Anthropic-only, no continuous loop in v0.1).
- §0b — Tool/Skill/Agent + system-prompt pieces (Available-skills block, identity preamble).
- §1b recovery + §2b signals/VDR — compaction touch-points.
- §2c LLMProvider — provider seam (our analog to iii's "provider router"); §2c.3 token tracking.
- §8.security — why the capability/enforcement layers are locked, not swappable.
- [`0001-interaction-layer-roadmap.md`](./0001-interaction-layer-roadmap.md) — sibling external-harness-thesis review.
- [`runtime-capabilities-roadmap.md`](./runtime-capabilities-roadmap.md) §1.2 — concurrent multi-agent / durable child runs (v1.0+).

## Status + tracking

Forward-design note (idea-stage). Re-evaluate Items 1 & 2 when the compaction decision and the M08/M09 prompt-assembly work are dispatched; fold the compaction deferral decision into the relevant milestone's gap-analysis entry.
