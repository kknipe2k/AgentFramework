# Stage 2 — Progress updates

**When:** at concrete milestones from `docs/MVP-v0.1.md` — drone resilient, graph live, workbench loads, first generated skill installed, end-to-end demo. Every 2–3 weeks at most. Skip if there's no milestone; don't post just to be visible.

**Goal:** show forward motion with one specific, verifiable capability per post. No "working on X." Capability lands → post.

**Format:** one paragraph + (optional) 10–20s clip showing the capability in action.

---

## Template

> Update: **[one specific capability that landed]**.
>
> [one sentence on what it means in practice — what the user can now do that they couldn't before].
>
> [optional: one constraint or rough edge — what's still broken or partial].
>
> Next: [the next milestone, named, no date].
>
> [clip]

---

## Drafts by milestone

### M2 — Event pipeline alive

> Update: agent events now flow from the Anthropic API through the Rust main process and out to the renderer in real time.
>
> Click "run smoke session" → the renderer logs every tool_invoked, every tool_result, every stream_text chunk as it happens. No graph yet — just the event log proving the wiring works.
>
> Next: Phase 3, render those events as a live graph.
>
> [10s clip: click run → events scrolling in renderer]

### M3 — Live Graph

> Update: graph is live.
>
> Every agent the session spawns becomes a node; every tool call animates an edge from the agent to the tool; every skill load draws a dashed line. Click any node for the full event payload + the VDR row that recorded the decision.
>
> The graph reconstructs after a page reload — state lives in SQLite, not React's memory.
>
> Next: M4, plan-approval gate + verification hooks + HITL primitive.
>
> [15s clip: agent session → graph populates → click a tool node → side panel with payload]

### M4 — Plan + verify + HITL + budget

> Update: full task lifecycle now works.
>
> Load a framework → orchestrator spawns planner → planner emits a 3-task plan → HITL approval panel surfaces → user approves → tasks execute one at a time. After each task, a `post_task` hook fires the verify pipeline; pass → next task; fail with `on_failure: rollback` → drone reverts to the snapshot taken at task_started, task retries.
>
> Budget tracking: warn at 50%, downshift at 75%, HITL at 90%, hard-stop at 100%.
>
> Next: M5, gap detection + capability enforcement.
>
> [20s clip: plan creation → approval → tasks running → verify hook firing]

### M5 — Gap detection + capability enforcement

> Update: capability enforcement working.
>
> An agent that declares `tools_called: [WebFetch, Read]` and tries to invoke Bash gets blocked. `capability_violation` event fires; HITL prompt asks the user to allow once / block / open builder to update the declaration. The artifact literally cannot exceed what it declared at runtime — that's the layer that makes "auto-accept tested" actually safe later.
>
> Also: `request_capability` meta-tool. Agent realizes it needs a tool it doesn't have → calls request_capability → GapPanel opens with the agent's stated reason → user installs → drone resumes from snapshot.
>
> Next: M6, MCP server connection.
>
> [15s clip: capability_violation → HITL prompt; gap → install → resume]

### M6 — MCP basic

> Update: Connect a Model Context Protocol server, agent uses its tools.
>
> Settings → MCP Servers → Add → URL or local path → Test → list of discovered tools appears. Auth in OS keychain (never written to disk by this runtime). MCPNode renders in the graph; tool calls flow through it as animated edges.
>
> Next: M7, import skills from URL.
>
> [10s clip: add an MCP server → tool list → agent using a tool from it]

### M7 — Registry import

> Update: import a skill from any URL.
>
> Paste a GitHub raw URL of a skill.md → fetched, validated against the JSON schema, run through the L3 sandbox, tier-gated review, installed, hash-locked in skills.lock. Same flow for tool.md and agent.md and MCP server configs.
>
> Skill drift detected: if the file's content_hash differs from skills.lock on next load, load is blocked with a re-install prompt.
>
> Next: M8, the workbench (Builder Canvas).
>
> [15s clip: paste URL → review screen → install → skill in palette]

### M8 — Workbench

> Update: the workbench works.
>
> Drag a Tool / Skill / Agent from the palette. Connect them with edges. Capability narrowing applied automatically when you wire a child agent under a parent. Live framework JSON preview on the right; Validate button runs the schema; Test button opens a sandboxed session with a separate SQLite database.
>
> JSON view + Canvas view share state. Edit either; the other updates.
>
> Next: M9, the generators (write a skill / tool / agent from a natural-language description).
>
> [20s clip: drag-drop → wire edges → Validate green → Test → graph runs]

### M9 — Generators

> Update: generators write a skill / tool / agent from a sentence.
>
> "Generate Tool" → describe what you need → generator produces a tool.md → L3 sandbox validates → review screen with capability disclosure in plain English → Install (Novice tier) or auto-accept toast (Promoted tier within bounds).
>
> Promoted tier blocks auto-accept for `shell: true` and `network: ["*"]` — those drop to Novice review even for a Promoted user. Operator tier (full auto-accept) waits for v1.0 with full OS-level sandboxing.
>
> Next: M10, first-run UX + polish.
>
> [25s clip: Generate Tool → describe → review → install → use in canvas]

### M10 — First-run + polish

> Update: fresh-install path works.
>
> Welcome → API key (test-connect) → "Build my own" or "Use ARIA template" → first session prompt → running session. Total elapsed for someone new: under 10 minutes. Tested on a fresh Windows VM.
>
> Settings panel: tier, privacy (export / delete-all-local), MCP servers, frameworks. Help → 60-second graph tour for the first-time user.
>
> Next: M11, signed installer + repo public + v0.1.0 release.

---

## Anti-patterns (DO NOT)

- "Working on..." — only post when something IS, not when something is being worked on.
- "Soon..." or "Coming...": no date promises.
- Comparisons ("turns out [project X] doesn't do this").
- Sped-up footage. Real time or be honest about timelapse.
- Voiceover. Captions only.
- Multiple unrelated capabilities in one post — split them.
- Posts shorter than 30 seconds apart in time. Let each one breathe.

## Engagement stance

- Reply to substantive technical questions in <48h.
- Direct DMs from interested testers/contributors → respond personally; bookmark for v0.1 launch.
- Trolls → mute, never quote-reply.
- "When will it ship?" → "When it ships." Or specifically: "M11 is the ship gate; current milestone is M[N]." Don't promise dates you don't know.
