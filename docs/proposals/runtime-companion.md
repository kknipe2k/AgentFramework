# Runtime Companion — forward-looking design proposal

**Status:** Proposed (idea-stage; v1.0+ scope)
**Date:** 2026-05-24
**Author:** @kknipe2k (showerthought captured 2026-05-24)
**Tags:** product, architecture, v1.0+, multi-session, conversation, self-improvement

## The idea (verbatim from the conversation that captured it)

> So LLM will help build — but can you layer in an LLM conversation node?
> This can run as pause — and also can run as overlay while runtime runs.
> Also can we have multiple sessions running in the same runtime? (I guess
> that's just multiple agents.) The idea is to have a conversation with
> the runtime — the runtime has embedded LLM that is self-aware of the
> app and specific runtime — can diagnose — can halt all or some — can
> create new paths on the fly. Crazy but eventually this is the thing.

## Articulated

A **Runtime Companion**: an embedded LLM with privileged read access to
the running runtime's full state and capability-bounded write access to
modify that state in flight. The user converses with it; it diagnoses,
halts, redirects, and (eventually) generates new tools / agents /
skills on the fly.

Two operating modes:

- **Pause mode** — full runtime halt; user opens conversation; companion
  examines state + proposes changes + user approves + changes applied +
  runtime resumes. The "stop and ask" altitude.
- **Overlay mode** — runtime keeps executing; companion observes +
  can alert the user proactively + can suspend individual agents without
  full halt. The "watching alongside" altitude.

Multi-session: multiple frameworks running concurrently as distinct
sessions. v1.0: per-session companion. v1.1+: cross-session companion
(Operator-tier).

## Why this matters

The runtime today is "configure once, run, watch." The companion makes
it "configure, run, and *converse*." Two unlocks:

1. **Live diagnosis at the right altitude.** Instead of digging through
   audit log + signals + graph to figure out why an agent did X, ask the
   companion. It synthesizes from the same state the user would
   manually inspect — but without the user becoming an audit-log
   archaeologist.

2. **Live evolution.** Instead of stop → edit framework → restart,
   the companion proposes + applies changes in-flight. Capability-
   bounded. Audited. User-approved. The framework improves as the user
   works with it.

Together: the runtime stops being a thing you *use* and starts being a
thing you *collaborate with*. Same shift as Claude Code vs an IDE.

## Architectural prerequisites (NOT a v0.1 feature)

The companion is well beyond v0.1's §0d scope locks. Prerequisites:

- **M8.6 (Framework Representation)** — canonical multi-file loader
  with one resolution boundary. The companion reads + writes framework
  state through a single canonical surface. Without this, the
  companion fights schema/path divergence.
- **M9 (Generators)** — for "create new paths on the fly." Without M9
  the companion can only modify EXISTING tools / agents / skills, not
  generate new ones. M9 is the unlock.
- **Multi-session runtime** — v0.1 is single-session per §0d; the
  companion's conversation lives in its own session distinct from the
  agent sessions it observes. v1.0+ scope.
- **Operator tier** — currently §0d "out of scope for v0.1, planned
  v1.0+." The companion's capability set is elevated above Promoted;
  it needs an explicit tier (or a sub-role within Promoted with
  audit-only write affordances). v1.0+ scope.
- **Conversation primitive** — spec §0a has Tool, Skill, Agent, Plan,
  HITL, Audit, Capability, Gap. "Conversation" is not yet a primitive.
  Either: HITL evolves to cover it (HITL is per-step approval today; a
  conversation is per-turn), OR a new Conversation primitive joins the
  matrix. ADR-class decision.
- **New ADR for the security model** — the companion's "modify
  in-flight" capability is the most powerful affordance in the
  runtime. Needs explicit threat model (rogue companion;
  compromised LLM provider; capability escalation chains), narrowing
  rules (what CAN'T the companion do, even at Operator-Companion
  tier?), audit guarantees (every companion action recorded with
  rationale).

## What the companion needs to read

All accessible today (M01-M08 milestones):

- **Live graph** (M3) — current agent/tool/skill nodes + execution
  state.
- **Audit log + signals** (M2 + M4) — every decision + every event.
- **Capability map** (M5) — what every agent can do.
- **Plan state** (M4) — current plan, completed steps, pending
  HITL checkpoints, budget.
- **MCP server inventory** (M6) — installed servers + their tool
  schemas.
- **Skills/tools registry** (M7) — installed artifacts + their
  capabilities + their provenance.
- **Framework structure** (M8.6) — canonical multi-file representation.

What the companion needs to write (with capability narrowing + audit):

- `pause_agent(agent_id)` — suspend one agent at its current step.
- `pause_all()` — full runtime halt (pause mode entry).
- `resume(agent_id?)` — resume one or all.
- `suspend_plan_step(step_id)` — skip a step pending user direction.
- `propose_change(change_spec)` — surfaces a proposed framework change
  to the user; user approves; change applied at next resume.
- `inject_tool(tool_md)` / `inject_skill(skill_md)` / `inject_agent(agent_json)` —
  add a new artifact mid-run (gated by M9 Generators).
- `redirect(agent_id, new_task)` — change what an agent is working on.

Every write goes through the existing capability enforcer + audit
writer; the companion's tier defines what it can call.

## Modes — concretely

### Pause mode

- User clicks "Talk to runtime" (a chrome button always visible).
- All agents halt at current step (their executor checks the suspend
  flag at the next step boundary; in-flight LLM/tool calls finish or
  cancel cleanly).
- Conversation panel opens. Companion has full read access; user has
  full write access (to the conversation, not the runtime — write to
  runtime is companion-mediated).
- User: "Why did the analyzer agent decide to read `foo.rs` first?"
- Companion: synthesizes from audit log + the analyzer's stated
  rationale + the plan step it was on. Answers in plain English with
  citations into the audit log + graph.
- User: "Add a tool that summarizes this file."
- Companion: drafts a `tool.md` + capability summary; user reviews;
  user approves; tool added to framework (gated by M9 Generators +
  L3 sandbox validation).
- User: "Resume."
- Runtime resumes with the new tool available.

### Overlay mode

- Runtime keeps executing.
- Companion runs as an observer agent (capabilities: read-only on
  graph/audit/signals; write only on `pause_agent` + `propose_change`).
- User can ping at any time without halting; companion answers from
  current state.
- Companion can proactively alert: "Agent X has been retrying for 3
  minutes — want to investigate?" Alerts surface as toasts /
  notifications.
- User can promote any companion intervention to a pause-mode
  conversation.

Default for v1.0: overlay mode opt-in (token cost), pause mode
always available (chrome button). User picks.

## The "this is the thing" eventuality

The companion + DESIGN.md + Stage D + M9 Generators together make the
runtime **self-improving in conversation with the user**. The
trajectory:

- v0.1: configure + run + watch.
- v1.0: configure + run + watch + converse + diagnose.
- v1.1+: converse + diagnose + propose changes + apply changes.
- v1.5+: companion suggests proactive changes the user accepts/rejects;
  the framework becomes a co-design with the runtime as a partner.
- v2.0+: companion observes patterns across runs + suggests structural
  framework refactors; the runtime becomes self-aware of *its own
  effectiveness*.

This is the framework-author's leverage. They build the initial
framework; the companion + their conversation evolves it.

## Connection to the AI-finance trust-gap post

The companion is the structural answer to the "AI quietly does the
technically defensible but actually wrong thing" class of bug:

- The companion observes (overlay mode) — when an agent makes a
  categorization decision that diverges from a recorded business rule
  (in a skill or DESIGN.md), the companion alerts the user.
- The companion has FULL access to the user's stated rules
  (DESIGN.md, chart_of_accounts.md, business-rules.md) AND full
  access to what the agent actually did. It synthesizes the
  divergence: "Agent X moved `account-42` from Operating to Other
  Expenses; per `chart_of_accounts.md` this account belongs in
  Operating; the agent's stated rationale was 'GAAP best practice
  for non-recurring items.' Want to revert + add a rule?"
- The user accepts; companion injects the rule + redirects the agent.
- Next time, the agent reads the rule, doesn't make the divergence.

The companion + the framework author's domain skill libraries close
the trust gap: the AI's "confidently wrong" decisions become
*surfaced disagreements between the agent's training and your stated
rules*, with a conversation path to resolution.

## Open questions

- **Capability model.** Is "companion access" a new tier
  (Operator-Companion?) or a special role within Promoted? Bias:
  separate tier — the companion's affordances are structurally
  different from agent affordances.
- **Conversation persistence.** Does the companion remember across
  runtime restarts? Probably yes; conversation is session-scoped
  audit data + replayable per spec §11 reconciliation.
- **Multi-user implications.** In a future multi-user runtime, do
  multiple users share the companion or each get their own? v1.0
  is single-user per §0d; defer.
- **Companion's own LLM provider.** Same as the agents (probably)
  or different? Probably user-choosable (e.g., companion = Opus
  for reasoning depth, agents = Sonnet for speed/cost).
- **Cost.** Always-running companion in overlay mode burns tokens.
  Pause-only default + opt-in overlay = pragmatic. Per-user budget
  cap applies (M4 budget primitive).
- **Conflict resolution.** What if companion's proposed change
  conflicts with another companion proposal (multi-session,
  cross-session)? Companion has lock semantics (one in-flight
  proposal per session?). v1.1+ design.

## When to design this seriously

Post-M11 (v0.1 ship). Before v1.0 milestone planning. The companion
ADR should be authored BEFORE v1.0 phase docs lock in the
multi-session + Operator-tier + Conversation-primitive architecture.

Roughly: v0.1 ships → 2-4 week pause → v1.0 planning → companion ADR
+ multi-session ADR + Operator-tier ADR + Conversation-primitive ADR
land together as a v1.0 design pre-flight → v1.0 milestone breakdown
incorporates the companion as a first-class deliverable.

## Related (existing)

- Spec §0a primitives: Plan, HITL, Audit, Capability, Gap, Tool, Skill,
  Agent — companion respects all of them.
- Spec §6a HITL notifier — pause-mode is HITL at a different altitude
  (per-conversation-turn vs per-step).
- Spec §11 reconciliation — companion conversation is replayable per
  the same signal-log discipline.
- M3 live graph — the read surface the companion reads from.
- M4 plan + HITL + budget — companion respects plan + budget.
- M8.6 framework representation — required prerequisite (canonical
  surface).
- M9 Generators — required prerequisite (on-the-fly artifact creation).
- DESIGN.md + Stage D Design Review — companion benefits from design
  quality lens (UX of the conversation panel + alert surfaces).
- `docs/post-m08.6-critical-review.md` — the architectural audit's "AI
  for finance trust gap" is what the companion is the structural
  answer to.

## Status + tracking

This is a forward-design proposal, not a committed roadmap item. It
lives in `docs/proposals/` (the starter-kit's convention, adopted here
post-M08.5.5). Re-evaluate at v0.1 ship to decide whether the
companion is the v1.0 headline feature or v1.1+ scope.
