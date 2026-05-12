# ADR-0008: Milestone Stage V Verifier (fresh-context contract-fidelity check)

**Status:** Accepted
**Date:** 2026-05-12
**Deciders:** @kknipe2k (maintainer)
**Tags:** process, milestone-protocol, testing, gates

## Context

M04 shipped with five bugs caught only by post-merge IRL manual testing:

1. Drone IPC reader single-use → SQL inspector + replay + recovery silently broken after one read
2. `AgentNode` read `tokensIn + tokensOut` instead of `tokensTotal` → visual scaling stuck at floor
3. `BudgetHeaderBar` shipped without its CSS rules → status changes invisible
4. `TaskNode` rendered blank when `title` field missing → "untitled" in inspector + canvas
5. `<main>` element constrained to 720px → graph canvas couldn't fill the window

Each passed its existing unit tests. The common pattern: **implementation tests green; contract tests missing**. The test suite verified the code-as-written; the contract — *"does the user-observable behavior match the spec?"* — was never asserted.

The milestone protocol at the time of M04 had four quality gates:

| Layer | What it catches | What slipped through |
|---|---|---|
| Per-stage retrospective | Process friction | Behavior gaps |
| Coverage gates (≥80% / ≥95%) | Lines exercised | Whether the line does the right thing |
| Gap analysis (per milestone) | Product↔spec at a high level | Per-component "does it work" |
| IRL test | Catches reality — but only after merge | Too late |

All four agents (per-stage build agents, per-stage retro agents, closeout agent for gap analysis) share confirmation bias: *"I/we just shipped this; the tests pass; it's done."* A clean session, knowing only what the spec says, would have asked *"can I query the DB twice in sequence"* within five minutes.

A milestone-level verification stage that runs in a **fresh CLI session** with **deliberately limited context** (phase doc + spec sections + code — *not* retros) closes this gap.

## Decision

**Adopt Stage V as a required stage between the milestone's last work stage and the closeout stage.** Stage V is a third schema variant alongside `<work_stage_prompt>` and `<closeout_stage_prompt>` — a fresh-context contract-fidelity check that runs four passes against the milestone's deliverables.

Concretely:

- **Schema variant** `<verifier_stage_prompt>` in `STAGE-PROMPT-PROTOCOL.md` v1.5. Declares its own required tags + sub-tags for the four passes.
- **Sequence:** `Stage A → B → C → D (work) → V (verifier) → [D.fix if 🔴] → E (closeout)`. V runs between last work stage and closeout. If V surfaces 🔴 findings, a `D.fix` stage authored on demand resolves them; V re-runs after the fix. Maximum 2 D.fix iterations before escalation.
- **Fresh-context paste pattern.** User clears the CLI session and pastes the V XML prompt fresh. The prompt's `<read_first>` deliberately omits prior retrospectives (the agent's bias against re-evaluating its own work). The honor-system clear-and-paste is the bias guard — no separate hook script needed.
- **Four passes:**
  1. **Inventory** — every file the phase doc said would ship exists and matches shape
  2. **Wire** — every spec claim has a verifiable code path (5-step data-path tracing, not bare grep)
  3. **Behavior** — runtime-render checks: actually exercise each primitive and observe DOM / IPC / state, not just static reads
  4. **Multi-call invariants** — every public API / IPC method / Tauri command survives "called twice in sequence"
- **Severity model aligns with gap-analysis:** 🔴 blocks merge (triggers D.fix); 🟡 carries forward to the next milestone's Stage A; 🟢 lands in `docs/tech-debt.md` (new append-only ledger distinct from gap-analysis and gotchas).
- **Waiver path.** If V flags 🔴 but the build agent disputes on interpretation grounds (spec ambiguity, architectural reframing), the build agent files an ADR-class waiver at `docs/adr/NNNN-waiver-M[NN]-finding-N.md` with one-paragraph reasoning; maintainer adjudicates. Same machinery as any other ADR — no new artifact class.
- **Tier conditioning:** N/A for this project (single-tier v0.1 runtime). All four passes are required for every milestone going forward (M05+).
- **Validator** (the schema validator shipped in PR #63) extends to validate `<verifier_stage_prompt>` blocks alongside the existing two variants.
- **Grandfathering:** M01–M04 predate the protocol. They are NOT retroactively run through V; their gap-analysis entries (already merged, append-only) stand. M05 is the first milestone shipped under V.

The phase-doc Stage V section has the standard X.1–X.6 shape:
- V.1 Problem statement (what this verification covers)
- V.2 Scope to verify (files / spec sections)
- V.3 Verification passes (per-pass details: specific multi-call invariants, specific hooks to trace)
- V.4 Findings format (the structured output Stage V produces)
- V.5 CLI prompt (the XML `<verifier_stage_prompt>`)
- V.6 Commit message (`verify(MNN): findings — N🔴 N🟡 N🟢`)

## Consequences

### Positive

- **Closes the M04-class bug pattern.** All five M04 bugs would have been caught by V's Behavior pass (runtime DOM/state inspection) or Multi-call pass (sequential drone IPC calls).
- **Fresh-context bias guard structurally enforced** via the paste pattern. No reliance on the build agent ignoring retros voluntarily.
- **Separation of cognitive modes.** Closeout stays focused on cumulative review + gap-analysis ledger; V handles contract fidelity. Mixing them would compromise both — closeout would context-switch mid-session between "ignore retros" and "read everything cumulatively."
- **Aligns with existing protocol shape.** V is a third XML schema variant, not a new artifact class. Validator extends naturally. Retrospective + commit conventions follow the established A/E pattern.
- **🟢 / tech-debt ledger separates "not-a-bug-but-noted" from gap-analysis (product↔spec drift) and gotchas (don't-do-this patterns).** Each artifact has a single clear purpose.
- **Forward-applicable to the framework / playbook side project.** The verifier shape generalizes; the agent-runtime instance is the first concrete deployment.

### Negative

- **Per-milestone cost: 2–4 hours of fresh-CLI verifier time.** Add 1–2 hours for D.fix iterations when 🔴 findings surface. Acceptable relative to the cost of post-merge hotfixes (~5+ PRs in M04's case).
- **Calibrated to one sample (M04).** The four passes are fitted to the bug classes M04 exhibited. M05's first V run will surface new bug classes; protocol will refine. Bug classes V may still miss (until the protocol iterates):
  - Race conditions / concurrency (sequential ≠ concurrent)
  - Error paths (happy path passes; error variant broken)
  - Resource lifecycle (memory leaks, file handle exhaustion)
  - Cross-platform divergence (test runs on one OS; bug in `cfg(other)`)
  - Performance regressions (slower than last milestone)
- **D.fix loop has a 2-iteration cap, then escalation to maintainer.** If V finds 🔴 repeatedly across iterations, the cap is the structural signal that the fix needs design, not patching.
- **Honor-system limitation on the read-first guard.** Build agent could in theory load retros despite the V prompt's `<read_first>` omitting them. The paste pattern reduces but does not eliminate the bias. Mitigated by the explicit `<read_first>` listing what TO read, plus the X.5 CLI prompt's `<context>` reiterating the fresh-eyes mandate.
- **Cognitive cost of the protocol grows.** v1.4 was ~80KB of stage-prompt content; v1.5 adds another ~3KB schema definition + ~3KB section to `BUILD-PLAYBOOK`-equivalents. Each iteration adds friction; weigh marginal addition against marginal value.

### Neutral / future implications

- **First V run after this ADR lands is retroactive against M04.** That run produces the first verifier retrospective (`M04.V-retrospective.md`) and surfaces whether V catches the five M04 bugs that triggered this ADR. If V finds them: protocol validated. If V misses any: refine the passes before M05.
- **Multi-tier conditioning is deferred.** Future framework deployments may want Lite / Standard / Full tier mappings for the four passes; this project ships single-tier and all four required.
- **Cross-stack runtime harness for the Behavior pass.** Vitest + jsdom covers renderer behavior; existing Rust integration tests cover IPC. v0.1 ships with this harness coverage; v1.0+ may add Playwright visual regression for the Visual sub-pass when GraphCanvas grows further.
- **The waiver-as-ADR pattern is reusable** for any "build-agent disputes verifier finding on interpretation grounds" scenario. The first such waiver (if it lands) tests the maintainer adjudication flow.

## Alternatives Considered

### Alternative A: Strengthen closeout instead of adding Stage V

Fold the four-pass verification into the existing closeout stage; require closeout to run with fresh CLI and retros excluded from its `<read_first>` list. One ceremony, one schema variant, one retrospective.

**Rejected because:** closeout has two distinct cognitive modes — *"fresh-eyes contract fidelity"* and *"cumulative cross-milestone review"*. Combining them forces the closeout agent to context-switch mid-session: the second mode requires reading retros and prior gap-analysis entries; the first mode requires NOT reading them. The bias guard only works if the session is structurally separated. A two-phase single-ceremony closeout was considered but the read-list guard becomes honor-system-within-a-session, which is exactly the failure mode this ADR is trying to eliminate.

### Alternative B: Continuous verifier hooks after each work stage

Run a mini-verifier pass after each of stages A, B, C, D. Catches bugs earlier; smaller per-pass scope.

**Rejected because:** per-stage verifier multiplies the friction the user has already complained about. M04 had four work stages; four verifier ceremonies plus four work stages plus closeout = nine ceremonies per milestone. End-of-milestone single-V is the compromise between bug-detection latency and ceremony overhead. If post-M05 retros show V at end is too late for some bug class, revisit with a smaller per-stage hook.

### Alternative C: Rely on automated coverage + IRL test catching everything

Don't add a verifier stage; instead, raise coverage thresholds, add visual regression tests, add multi-call integration tests, and require IRL test before milestone PR merge.

**Rejected because:** the M04 bugs surface only when the *user-observable behavior* is checked, not the *code paths*. Coverage at ≥80% on every M04-touched file would not catch BudgetHeaderBar's missing CSS rules (the JavaScript path is correct; only the CSS file is incomplete). Higher coverage gates miss the class of bug entirely. IRL test catches it, but the IRL test is unstructured and depends on the maintainer remembering to run it pre-merge — V structures the same check and gates the merge.

## Related

- **Spec sections:** N/A — this is a process / protocol change, not a §0a Capability Matrix change.
- **Prior ADRs:** ADR-0003 (Engineering Charter adoption) is the parent context that V extends. Not superseded; V is additive to the charter.
- **Companion docs:** `STAGE-PROMPT-PROTOCOL.md` v1.5 (defines the schema); `CLAUDE.md` §19 (retrospective protocol, extended with verifier-retro shape); `CLAUDE.md` §20 (gap-analysis protocol, references V findings); `docs/build-prompts/STAGE-V-VERIFIER-PROMPT-TEMPLATE.md` (the parameterized prompt template); `docs/build-prompts/retrospectives/VERIFIER-RETROSPECTIVE-TEMPLATE.md` (the per-V retro shape); `docs/tech-debt.md` (the 🟢 findings ledger).
- **External references:** Inspired by the multi-session design review with the framework/playbook project's protocol author (round 4) — the load-bearing insight was the fresh-CLI clear-and-paste pattern as a structural bias guard, not a hook-based enforcement.

## Notes

The framework/playbook side project (separate repo) discussed the same pattern at greater depth, including tier conditioning (Lite / Standard / Full) and a SessionStart hook for mode-aware read-list selection. This ADR is the agent-runtime instance — single-tier, no SessionStart hook, paste-pattern bias guard. The framework-side implementation may diverge in detail; the core shape (fresh-context, four passes, severity model, waiver-as-ADR) is shared.

Calibration: the four passes were arrived at across a four-round design review. Round 1 proposed three passes (inventory + hooks + multi-call) fitted to M04's bugs. Round 4 added a fourth pass (Behavior / runtime / visual) after the BudgetHeaderBar-CSS bug class was identified as static-uncatchable. Round 4 also tightened Pass 2 (Hooks) from "grep-verify" to "5-step data-path tracing" after the AgentNode-tokensTotal example showed grep was inferential, not deterministic. Both round-4 refinements are baked into the v1.5 schema.

First retroactive V run is scheduled for M04 post-merge as the protocol's first real-world test. Findings land in `M04.V-retrospective.md`; gap analysis remains immutable per CLAUDE.md §20 (append-only).
