# Stage 1 — First public signal

**When:** Live-graph + gap-flow demo works end-to-end on your machine. Roughly mid-M5 to early M6 in the build checklist (`docs/MVP-v0.1.md`). Not before.

**Goal:** establish the project exists. Show one specific capability. No date, no promises, no roadmap teasing.

**Format:** one post + a 15–30 second screen recording (no audio, captions if anything).

---

## Draft A — terse, technical

> Building a desktop runtime for agentic AI workflows.
>
> The differentiator: when an agent needs a capability it doesn't have, the session suspends cleanly, shows you exactly what's missing, and resumes when you fix it. No more starting over after an agent flails.
>
> Rust + Tauri. ~10MB binary. Apache 2.0 when it ships. Windows first.
>
> Inspired by ARIA, Boris Cherny's subagent patterns, and the Ralph autonomous-loop pattern.
>
> [15–30s screen recording: agent runs → hits gap → GapPanel opens with capability_name + reason → user installs missing tool → resume → completes]

## Draft B — slightly longer, problem-first

> Today's agent loops burn tokens on failures with no audit trail. When the agent gets stuck, you usually start over.
>
> Working on something different: every decision captured, every missing capability surfaced cleanly instead of letting the agent flail, and a workbench where you fix the gaps without restarting.
>
> Rust + Tauri runtime. Live graph of agent reasoning. Capability sandboxing on every artifact.
>
> Apache 2.0 when shippable. Windows first because that's where I work.
>
> [video]

## Draft C — minimal

> A desktop runtime that suspends an agent session when a tool/skill is missing — instead of letting the agent flail.
>
> [15s clip]
>
> Rust + Tauri. Apache 2.0 at v0.1. Windows first.

---

## Notes per draft

- **A** is the safest. Concise, technical, no overclaiming, prior art credited.
- **B** trades brevity for problem framing. Use this if the audience may not already know why current agent loops are frustrating.
- **C** is the highest-confidence version — works only if the demo speaks for itself.

## What NOT to include

- Project name (don't have one yet — "the runtime" is fine until you do)
- Logo / banner / brand assets
- Repo URL (repo stays private until ship; teasing a private repo just frustrates people)
- Promised features ("coming soon: X, Y, Z")
- Compatibility claims you haven't verified (don't say "works on Mac/Linux too" if you've only tested Windows)
- Comparisons to specific competitors ("better than [project name]")
- Numbers you haven't measured (don't say "10x faster" without a benchmark you can show)

## Replies that will arrive — prep your response stance

- "Open source it now" → "Will be at v0.1 ship. Apache 2.0. ETA TBD; the work is the work."
- "Why not Mac/Linux?" → "Solo maintainer. Windows first. Multi-OS in v1.0. Happy to take a Linux contributor at that point."
- "Does it support [model X]?" → "Anthropic only at v0.1. Provider trait is in place; OpenAI / Google / local-Ollama in v2.0+."
- "Why Tauri not Electron?" → "Bundle size + security. ADR-0002 in the repo when it lands."
- "How is this different from [LangChain / AutoGPT / CrewAI / agent framework X]?" → Don't engage in defining differences in 280 chars; "Different scope. Spec is published when the repo opens. Curious what you'd want compared." Defer to long-form.

Trolls / bad-faith replies: don't engage. Mute, move on.

## After posting

- Don't refresh likes/replies obsessively. Check at end of day.
- Reply substantively to substantive replies; ignore the rest.
- Save thoughtful replies as potential testers / contributors for v0.1 ship.
