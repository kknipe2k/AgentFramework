# Open Questions

> Things that came up during planning but weren't decided. Each one blocks
> or shapes some later phase. Ordered roughly by when they need an answer.

---

## Blocking v0.1 start

These three need answers before the first line of code gets written.

### 1. Product name

The product needs a repo name, an app name, and a one-line pitch.

Candidates discussed:
- **Loom** — "weaves agentic workflows"; short, clean, clear metaphor
- **Atelier** — French for "workshop"; gestures at craft, maybe too fancy
- **Forge** — direct, workshop-coded, slightly overused in dev tools
- **Weft** — another weaving metaphor, less common than Loom
- Something entirely user-chosen

**Status:** Undecided. Lean: Loom, pending user preference.

### 2. Repo location

- Personal GitHub account (`username/<productname>`) — fastest, most
  personal, easiest to transfer later.
- New org — signals "this is a real project," slightly more setup.
- Subdirectory of this `AgentFramework` repo — fastest of all, but
  muddles the ARIA repo with the new product.

**Status:** Undecided. Recommendation: personal GitHub account for v0.1,
move to an org if/when it takes off.

### 3. Start order

- **[A] Skeleton-first** — Drone Core + Supervisor + plumbing before any
  UI. Architecturally rigorous. Nothing clickable for 2-3 sessions.
- **[B] Slice-first** — thin end-to-end user experience (open app, see one
  card, click it, type a task, watch narration, see cost). Fake the
  plumbing; rewrite it properly after the feel is validated.

**Status:** Undecided. Recommendation: **[B] slice-first.**

---

## Blocking later phases (can decide when we get there)

### 4. Outcome labeling UX details (Phase 2)

How exactly do users rate runs so the Optimizer has training signal?
Thumbs at end of run? Hotkey during run? Passive inference from re-runs
and aborts? Combination with weighting?

### 5. Embedding model choice for Corpus (Phase 5)

Anthropic embeddings API when available vs. a small local model via
`transformers.js`. Trade-off: quality vs. offline operation vs. setup
friction.

### 6. Secrets injection model (Phase 5)

How does a secret (API key, MCP auth token) reach an agent's context
without leaking into logs or snapshots? keytar for storage is decided;
injection mechanism is not.

### 7. `.workflow` bundle format (Phase 6)

Conceptually agreed: manifest + skill references + MCP requirements.
File structure not drawn. Open questions: single-file JSON vs. zip with
manifest + assets? How does bundle versioning work? How does loading
handle missing skills (prompt to install?)?

### 8. First-Run Experience wizard content (Phase 0)

What are the three sentences that introduce the app? What are the
starter tasks for each framework card? What's the onboarding flow for
API keys? This is UX-heavy work that needs user testing.

### 9. Orchestration strategy library v1 (Phase 6)

Which ~6 strategies ship in the executor. Proposed set:
1. ReAct loop
2. Plan-then-execute
3. Single-shot
4. Jury (committee)
5. Debate (committee)
6. Chain-of-verification (committee)

Needs confirmation. May need a seventh for "multi-step pipeline with
explicit state transitions" if any v0.1 framework needs it.

### 10. Sandboxing for custom skills (Phase 8)

What's the sandbox boundary when running a Python skill written by the
Skill Writer? Options: subprocess with restricted env, container
(Docker/Podman), WASM, or trust the user. v0.1 doesn't need this
(Quick Agent uses built-in skills only), but it blocks Phase 8.

### 11. Builder graph editor UX (Phase 7)

Wire it into React Flow like the runtime graph? Separate editor? Custom
component? This is the single biggest UX design problem in the whole
project and deserves its own session.

---

## Strategic / business questions (not technical)

### 12. Monetization model

Not needed for v0.1. Options on the table (in rough order of realism):

- **Hosted version** — free to self-host, paid SaaS with cloud sync,
  collaboration, zero install.
- **Open core** — core free; team collab, audit logs, SSO, enterprise
  policy are paid.
- **Marketplace cut** — take a percentage on paid skills/frameworks.
- **Support contracts** — free software, paid enterprise support.
- **Sponsorships / GitHub Sponsors** — low-ticket but real.

Decision can wait until ~1,000 stars or ~100 active users.

### 13. License

**Decision:** Apache 2.0.

Rationale: maximally permissive, widely trusted, no legal friction for
enterprises, doesn't lock out contributors. Alternative would be BSL if
we wanted to prevent hosted-competitor forks — not needed for a desktop
app.

Revisit if hosted version becomes the monetization model and we need to
protect against AWS-style forks.

### 14. Mac support

**Decision for v0.1:** Windows-only tested. Code written platform-agnostic.
README says "macOS untested, PRs welcome."

Mac support becomes real only when a Mac collaborator appears and
contributes.

### 15. Relationship to Claude Code

**Decision:** Build *on top of* Anthropic primitives (SDK, MCP, skills
format) rather than alongside. Positioning: "the visual workbench that
runs Claude Agent SDK workflows." Compete where Anthropic will be
slowest: non-coder accessibility, visual-first, framework-agnostic.

### 16. Demand validation

Originally proposed: 20-person interview round before building. Revised
given that the AI is doing the building: **the prototype is the demand
test.** Ship v0.1 publicly on GitHub, see who shows up.

If v0.1 gets silence, iterate or move on. If it gets traction, continue
to Phase 1b and beyond.

---

## Questions the planning session produced but didn't address

These came up in passing and deserve their own follow-up eventually.

### 17. Semantic vs. key-based memory

v0.1 decision: memory is KV; semantic memory goes in a corpus. This is
slightly awkward for "remember this user's preferences semantically" but
keeps the mental model clean. Revisit if users complain.

### 18. What happens when multiple frameworks share a namespace

Memory namespaces could collide across framework instances. Current plan:
scope memory namespaces by framework instance, not globally. Needs to be
enforced in the `MemoryClient`.

### 19. Anthropic API rate limits and cap-interaction edge cases

What happens when a run is paused mid-committee because a rate limit was
hit? Budget cap reached mid-parallel-execution? These are implementation
details that will surface during Phase 6.5 testing.

### 20. Upgrading framework manifests when the schema changes

If a v1.2 manifest is loaded into a v2.0 workbench, what happens? Auto-
migrate? Warn user? Refuse? Manifest has `bundle_version` but the
migration story is undefined.

### 21. Privacy story for the Optimizer

Optimizer is local-only in v0.1. If we ever add a "share improvements"
or "learn across users" feature, it must be strictly opt-in with
transparent disclosure. No telemetry of any kind in v0.1. Documented here
so it doesn't get lost.

---

*Maintained alongside `DESIGN-DECISIONS.md`. When an open question is
answered, move its conclusion into `DESIGN-DECISIONS.md` and delete it
from here.*
