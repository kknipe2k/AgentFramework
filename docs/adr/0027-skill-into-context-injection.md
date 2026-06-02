# ADR-0027: Skill-into-context injection model — tool-result, persisted via message history

**Status:** Accepted
**Date:** 2026-05-31
**Deciders:** @kknipe2k
**Tags:** capability, skill, system-prompt, scope, runtime

## Context

M08.7 rung 3 (Stage C) builds the `LoadSkill` handler — the first
production emitter of `AgentEvent::SkillLoaded` (the variant is
schema-defined at `crates/runtime-core/src/event.rs:279` and
`replay.rs:146` can already *project* it, but nothing emits it). A skill
in an agent's `allowed_skills` is never injected into the agent's
context; the agent cannot follow skill instructions. This is the only
true greenfield in M08.7a.

`LoadSkill` must make a skill's instruction body reach the model **for
subsequent turns** so the skill changes observable behavior. Three facts
ground the decision (all verified at the stage's `ground_at_red`):

- **Load policy is tool-invoked, not framework-auto-load.** Spec §0b:
  a skill is a "Context-loaded instruction set… Runtime-injected
  `LoadSkill` tool" (line 107); "LoadSkill runtime tool (auto-injected
  into every agent's tool list)" (line 166). The `LoadSkill` input is
  `{skill_name, reason}` (lines 175-176). The capability gate is the
  agent's `allowed_skills: Vec<String>`.

- **The resolved skill body is NOT carried on `Framework`.**
  `FrameworkSkillsItem` is reference-only — `{name, path?, registry_id?,
  source}`, no body field (`generated/framework.rs:2786`). Per ADR-0022
  the loader *does* resolve skill companions to bodies, but into a
  **separate** `LoadedFramework.companions[] = {file_name, body}`
  structure, not into `Framework.skills`. The run path
  (`run_test_session_with(framework: &Framework)` →
  `CapabilityWiring.framework: Arc<Framework>` → `drive_stream`) carries
  only `Framework`, so the already-resolved body is **not reachable**
  from the run loop today. Rung 3 must thread the already-resolved body
  into the run path (it does NOT re-read skill files — ADR-0022's
  single-resolution-boundary holds).

- **A tool result persists across turns in this codebase.**
  `AgentSdk::run_agent` accumulates `config.messages` and re-sends
  `config.clone()` on every subsequent turn
  (`agent_sdk.rs:258-297`); a `ToolResult` block appended on turn 1 is
  present in every turn 2..N. This is the exact feedback contract rung 1
  (built-in tools) and MCP dispatch already rely on.

The §0b system-prompt-assembly job is partial (proposal
`harness-review-takeaways.md` Item 2 — the Hermes three-tier
stable/context/volatile model is the forward blueprint); rung 3 needs
the minimal injection that makes a skill change behavior without
committing to that refactor (the stage scope lock).

## Decision

**We inject a loaded skill's body as the `LoadSkill` tool's `tool_result`,
which persists in the agent's message history for all subsequent turns —
the spec §0b model. We do NOT introduce a system-prompt-section injection
channel for v0.1.**

Concretely:

- A `LoadSkill` `ToolUse` is intercepted in `drive_stream` by a branch
  analogous to the rung-1 built-in branch. The branch calls the pure
  `sdk::load_skill::load_skill(skill_name, allowed_skills,
  resolved_skills)` handler, which (1) checks `skill_name` is in the
  agent's `allowed_skills` (the capability gate — analogous to the
  executor's `file_access` check; a skill not in `allowed_skills` is
  denied), (2) looks up the already-resolved body from the threaded
  `resolved_skills` map (no re-resolution), and returns
  `LoadedSkill { name, body }`.

- On success the branch emits `SkillLoaded { agent_id, skill_name, mode:
  None }` and emits the `ToolInvoked` / `ToolResult` pair (the result
  payload carries the skill body), and returns a `DispatchedTool` whose
  value is the skill body so `run_agent` appends it as a `ToolResult`
  block to `config.messages`. Because messages are re-sent every turn,
  the body **persists for the rest of the session**.

- **The resolved body reaches the run loop by threading.** A
  `resolved_skills: BTreeMap<String, String>` (skill name → body) is
  carried on `CapabilityWiring` (set via an additive
  `with_resolved_skills` builder so existing constructors are
  byte-stable). The Tester's production wrapper builds the map by joining
  `framework.skills[].path` ↔ `LoadedFramework.companions[].file_name`
  (the body was resolved once, at load — ADR-0022). The assembled test
  passes the map directly through a `run_test_session_with_skills` seam.

- **Composition is additive.** A second `LoadSkill` appends a second
  `ToolResult`; both bodies are in the message history, so multiple
  skills compose. No skill replaces another.

- **The grounded close is behavioral.** Per CLAUDE.md §4 rule 11 /
  gotcha #66, the `SkillLoaded` event firing licenses only "the event
  fired." The CI assembled test proves *injection-into-context* (the
  skill body is present in the turn-2 `AgentConfig`, observed on the real
  assembled config); the **IRL gate** proves *behavior-change* (a real
  Anthropic model loads a "reply in ALL CAPS" skill and replies in all
  caps).

- **No schema change.** `SkillLoaded` already exists. The capability gate
  is `allowed_skills`. In-process only — no Bash / sandbox.

## Consequences

### Positive

- Spec-compliant (§0b: "Tool result returns the skill body").
- Maximally minimal: the `LoadSkill` branch mirrors the proven rung-1
  `dispatch_builtin` feedback contract; no new persistence channel, no
  `AgentConfig` structural change.
- The body persists across turns automatically via the existing
  message-history accumulation.
- Capability-gated by `allowed_skills`; a skill not granted is denied.

### Negative

- The skill body lives in the message history, not a stable system-prompt
  prefix, so it is NOT in a cache-friendly stable prefix (a forward cost
  consideration once Anthropic prompt-caching is wired — not used today).
- `mode_variants` section filtering (spec §0b "sections filtered by
  `${session.mode}`") is NOT applied in v0.1 — the full body is injected.
  STANDARD mode is hardcoded for v0.1 (§0d), so mode-filtering is a
  forward item, not a v0.1 gap.
- The resolved-skills map must be threaded from the loader's companions
  into the run path; the production join lives in the Tauri-shell Tester
  wrapper (the `&Path`/shell-resolves archetype, CLAUDE.md §9).

### Neutral / future implications

- **Forward path — the three-tier assembly.** When the §0b
  system-prompt-assembly job lands (harness-review Item 2; Hermes
  stable/context/volatile), loaded skills become **stable-tier** content
  and migrate from the message history into the stable system-prompt
  prefix wholesale — a cache-friendly, semantically cleaner home. This
  ADR's tool-result mechanism is the grounded v0.1 step toward that, not
  a dead end: the handler + the resolved-skills threading are reused; only
  the *injection target* moves. A future ADR supersedes this one when
  that refactor is scheduled.

## Alternatives Considered

### Alternative A — system-prompt-section injection (the M08.7 phase-doc sketch)

Append the skill body to a dedicated loaded-skills section of
`AgentConfig.system_prompt` via a new `TurnFeedback.loaded_skills`
channel that `run_agent` drains before the next turn.

**Rejected for v0.1 because:** it diverges from spec §0b (which says the
tool result returns the body); it needs a second feedback channel beyond
the existing `dispatched` one (more wiring) AND still must feed a
`tool_result` for the `LoadSkill` `ToolUse` id (the Anthropic
continuation contract requires every `tool_use` to be answered), so it is
strictly *more* than the tool-result approach; and its motivating
rationale in the phase-doc sketch ("a tool result vanishes after the turn
that loaded it") is factually wrong for this codebase — `run_agent`
re-sends the accumulated `config.messages` every turn, so a tool result
persists. The semantic appeal (instructions belong in the system prompt)
is real and is captured as the forward path above, gated on the
three-tier assembly.

### Alternative B — framework auto-load of all `allowed_skills` at session start

Inject every `allowed_skills` body into context at session start, no
tool call.

**Rejected because:** spec §0b is explicit that skills are loaded via the
runtime-injected `LoadSkill` tool, on the agent's decision (semantic +
programmatic triggers, §0b). Auto-loading every skill defeats the
context-economy purpose of on-demand skill loading and contradicts the
schema text. Noted as a forward option, not adopted.

## Related

- Spec: §0b (Tool/Skill/Agent — the LoadSkill runtime tool, "Tool result
  returns the skill body", the Available-skills block); §0d (STANDARD
  mode hardcoded for v0.1)
- Prior ADRs: ADR-0022 (canonical framework representation — the loader
  resolves skill companions to bodies into `LoadedFramework.companions`,
  the single-resolution-boundary rung 3 reads from); ADR-0028 (the
  built-in tool execution contract — the feedback contract this mirrors);
  ADR-0019 (the Tester isolated-session model the assembled test runs in)
- Proposals: `docs/proposals/harness-review-takeaways.md` Item 2
  (system-prompt-assembly tiering — the forward path)
- Build prompt: `docs/build-prompts/M08.7-execution-engine.md` Stage C
- Event: `crates/runtime-core/src/event.rs:279` (`SkillLoaded`)

## Notes

Filed `Proposed` in the M08.7.C red commit. Flips
`Proposed → Accepted` in the M08.7 PR (the stage that implements the ADR
flips it — the ADR-0026 / M08.6.A precedent). The injection-model
contradiction between spec §0b (tool-result) and the phase-doc ADR-0027
sketch (system-prompt-section) was surfaced at `ground_at_red` per
CLAUDE.md §2 (surface contradictions, do not pick); the maintainer
selected the spec-compliant tool-result model.
