# Build Prompts

This directory holds **per-milestone prompts** for Claude Code sessions. Each milestone prompt is **self-contained**: it can be pasted as the opening message of a fresh, cleared Claude Code session and Claude will know exactly what to read, what to deliver, what tests to run, and what NOT to do.

## How this works

The project uses a **two-layer prompt structure**:

| Layer | File | When loaded | Content |
|---|---|---|---|
| **Layer 1: Constants** | `CLAUDE.md` (repo root) | Auto-loaded by Claude Code in every session | Protocol — TDD discipline, quality gates, PR workflow, anti-patterns, hard rules, schemas-as-source-of-truth, capability adherence. The "how Claude works in this project" doc. |
| **Layer 2: Per-milestone scope** | `docs/build-prompts/M[NN]-*.md` | Pasted as the opening message of a fresh session | The "what THIS milestone delivers" doc — scope, reading list, TDD plan specific to the milestone, acceptance criteria, milestone-specific gotchas. |

The per-milestone prompt always references `CLAUDE.md` as the protocol; it doesn't repeat the protocol verbatim. This keeps the prompts tight and the protocol DRY.

## Files

| File | Status | Purpose |
|---|---|---|
| `README.md` (this file) | Stable | Index + how-to-use |
| `TEMPLATE.md` | Stable | Per-milestone shape; copy this when authoring M12+ |
| `M01-foundation.md` | Authored | M1 (weeks 1–2): Cargo workspace + drone + runtime-core types + CI green |
| `M02-event-pipeline.md` | TODO | M2 (weeks 3–4): SDK + AnthropicProvider + Tauri shell + event flow |
| `M03-live-graph.md` | TODO | M3 (weeks 5–6): React Flow + node types + VDR projection |
| `M04-plan-verify-hitl-budget.md` | TODO | M4 (weeks 7–8): §3a + §4a + §6a + §2a + §1b |
| `M05-gap-and-capability.md` | TODO | M5 (weeks 9–10): §4b + §8.security L1+L2a+L3+L4(N+P)+L5 |
| `M06-mcp-basic.md` | TODO | M6 (weeks 11–12): MCP add/connect/list + per-server auth |
| `M07-registry-import.md` | TODO | M7 (week 13): import-by-URL + import-by-file + skills.lock |
| `M08-workbench.md` | TODO | M8 (weeks 14–17): Phase 9 Builder Canvas + Tester |
| `M09-generators.md` | TODO | M9 (weeks 18–20): Phase 8a/b/c with tier-gated install |
| `M10-first-run-polish.md` | TODO | M10 (weeks 21–22): §14 onboarding + Settings + Help |
| `M11-ship-prep.md` | TODO | M11 (weeks 23–24): unsigned .msi + SHA-256 + Sigstore + release |

`M01-foundation.md` is the **proof-of-concept**. After M1 actually runs, lessons learned go into `CLAUDE.md` (where appropriate) and `TEMPLATE.md`, then M02–M11 are generated in one batch using the validated template.

## How to use a milestone prompt

1. Open a **fresh** Claude Code session in this repository (cleared context — don't continue from prior session work).
2. Copy the entire contents of `M[NN]-<milestone>.md` into the opening message.
3. Add any session-specific overrides at the top: branch name, time-box if applicable, anything that's true for this run but not the milestone in general. Keep it minimal — the prompt is intentionally complete.
4. Send. Claude will read `CLAUDE.md` (auto-loaded), the milestone prompt, and the files the milestone prompt names — in that order.
5. Claude does TDD work, runs gates, drafts the PR description, surfaces it. **Claude does not commit.** You review, approve, and Claude then commits + pushes.
6. After the milestone PR merges, the next session starts fresh with the next milestone's prompt.

## Authoring new milestone prompts

Use `TEMPLATE.md`. Sections are not optional — even when "None applies" or "N/A", state that explicitly. The template is annotated to explain why each section exists and what makes a good vs a poor entry.

When a milestone prompt is authored:
1. Mark its row in the table above as "Authored"
2. Reference it from `docs/MVP-v0.1.md` §M[N]
3. Commit per `CLAUDE.md` §8 PR workflow

## Versioning

Milestone prompts are versioned implicitly via git history. If a milestone is re-scoped after work begins (rare; should require an ADR per `CLAUDE.md` §11), update the milestone prompt and note the change in `CHANGELOG.md`.

The two-layer separation means common protocol changes go to `CLAUDE.md` and don't require updating 11 milestone prompts. That's the point.

## Why two layers (and not just one giant file)

- **DRY.** Protocol changes happen in one place.
- **Tight prompts.** A milestone prompt is ~200–400 lines instead of 800+. Easier to fit in context, easier to read, easier to revise.
- **Drop-in droppability.** A fresh session reads `CLAUDE.md` automatically; pasting the milestone prompt completes the orientation.
- **Survives clearing.** Each milestone prompt is self-contained relative to the protocol. No conversational state required.

## Why this isn't just `CLAUDE.md`

`CLAUDE.md` is for things that are constant across all work in the repo. The per-milestone prompts are for things that change per milestone — what to build, what to read, which tests apply, milestone-specific traps. Mixing them would either bloat `CLAUDE.md` with milestone detail or scatter protocol across milestone files. Two layers is the cleanest factoring.
