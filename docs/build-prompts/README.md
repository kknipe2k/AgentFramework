# Build Prompts

This directory holds **per-milestone prompts** for Claude Code sessions. Each milestone prompt is **self-contained**: it can be pasted as the opening message of a fresh, cleared Claude Code session and Claude will know exactly what to read, what to deliver, what tests to run, and what NOT to do.

## How this works

The project uses a **two-layer prompt structure**:

| Layer | File | When loaded | Content |
|---|---|---|---|
| **Layer 1: Constants** | `CLAUDE.md` (repo root) | Auto-loaded by Claude Code in every session | Protocol — TDD discipline, quality gates, PR workflow, anti-patterns, hard rules, schemas-as-source-of-truth, capability adherence. The "how Claude works in this project" doc. |
| **Layer 2: Per-milestone document** | `docs/build-prompts/M[NN]-*.md` | Read by user end-to-end; per-stage CLI prompts pasted as fresh-session opening messages | The milestone specification + per-stage prompts — header (background, design decisions), stages A–D each with X.1 Problem / X.2 Files / X.3 Detailed Changes / X.4 Tests / X.5 CLI Prompt / X.6 Commit Message, summary table, verification checklist. |

The per-milestone prompt always references `CLAUDE.md` as the protocol; it doesn't repeat the protocol verbatim. This keeps the prompts tight and the protocol DRY.

## Files

| File | Status | Purpose |
|---|---|---|
| `README.md` (this file) | Stable | Index + how-to-use |
| `TEMPLATE.md` | Stable | Per-milestone shape; includes the **scope-split rule** for milestones >250 prompt-lines or >12h work |
| `PROCESS-VALIDATION.md` | Stable | Framework reference for evaluating whether the prompt-driven pattern works (axes, scoring, threshold gates) |
| `../persistence-architecture.md` | Stable | High-level architecture: how memory and instructions persist across sessions / stages / milestones (layers, lifecycle, mutability matrix) |
| `retrospectives/` | Live | Per-session retrospectives Claude fills in during/after every milestone; see `retrospectives/README.md` |
| `../gap-analysis.md` | **Live, append-only** | Cumulative product↔spec audit. Every parent milestone appends one entry in its Phase Closeout (final stage) per `CLAUDE.md` §20. **Prior entries are immutable** — CI enforces. |
| **M01 — staged into A/B/C/D plus E (Phase Closeout); one PR for the parent milestone** | | |
| `M01-foundation.md` | Authored | M1 (weeks 1–2; 5 stages, ~29–46h total). Stage A workspace skeleton + Stage B type generation + Stage C drone Phase 1 + Stage D fuzz + polish + Stage E gap analysis. Each stage commits on the parent-milestone branch; M1 PR drafts at end of Stage E. |
| **M02–M11 — generated after M01 summary** | | |
| `M02-event-pipeline.md` | Authored | M2 (weeks 3–4; 6 stages, ~13h actual calibrated): Stage A build hygiene + scaffolding (M01 carry-forward + signal.rs + HeartbeatStatus + mcp_servers); Stage B LLMProvider trait + AnthropicProvider stub; Stage C real HTTP+SSE impl with `*_with` SSE pattern + wiremock + ≥95% coverage; Stage D AgentSdk + main↔drone IPC client + ProviderEvent→AgentEvent translation; Stage E Tauri shell + skeleton React renderer + frontend CI gates + Playwright; Stage F Phase Closeout. Each stage commits on parent-milestone branch; M2 PR drafts at end of Stage F. |
| `M03-live-graph.md` | Authored | M3 (weeks 5–6; 6 stages A–F, ~25–31h calibrated): Stage A build hygiene + carry-forward closures (delete `src/counter.{js,test.js}`, retrofit drone integration tests to `current_exe()`, add `event.v1.json` schema + xtask TS codegen, vitest --coverage default, Vite/keyring/secrecy re-eval, add React Flow + Zustand deps); Stage B React Flow + Zustand foundation + 3 basic node types (Agent/Tool/Skill); Stage C remaining 8 node types (MCP/Gap/HITL/Plan/Task/Verify/Hook/Framework) + animated edges + color encoding; Stage D click-to-inspect side panel + token-spend node weight + zoom/pan; Stage E VDR projection + SQL inspector + graph persistence to SQLite; Stage F Tauri 2.x desktop-shell E2E (tauri-driver + WebdriverIO; Linux+Windows matrix; macOS unsupported) + Phase Closeout. First milestone authored on the v1.2 XML stage-prompt protocol per `STAGE-PROMPT-PROTOCOL.md`. Each stage commits on parent-milestone branch; M3 PR drafts at end of Stage F. |
| `M03.5-pre-m04-prep.md` | Authored | M3.5 (week 6/7 prep): combined doc PR (post-M03 spec polish + M02 carry-forward + 8 gotchas graduation + new `schemas/error.v1.json` + CLAUDE.md count refresh) + STAGE-PROMPT-PROTOCOL.md v1.3 iteration (5 new tags + 3 anti-patterns). Two stages on parent-milestone branch; PR drafts at end of Stage B. Doc/protocol-only — no source code touched, no gap-analysis entry per CLAUDE.md §20. Sets the stage for M04 prompt authoring on v1.3. |
| `M04-plan-verify-hitl-budget.md` | TODO | M4 (weeks 7–8): §3a + §4a + §6a + §2a + §1b. First milestone authored on the v1.3 XML stage-prompt protocol. |
| `M05-gap-and-capability.md` | TODO | M5 (weeks 9–10): §4b + §8.security L1+L2a+L3+L4(N+P)+L5 |
| `M06-mcp-basic.md` | TODO | M6 (weeks 11–12): MCP add/connect/list + per-server auth |
| `M07-registry-import.md` | TODO | M7 (week 13): import-by-URL + import-by-file + skills.lock |
| `M08-workbench.md` | TODO | M8 (weeks 14–17): Phase 9 Builder Canvas + Tester |
| `M09-generators.md` | TODO | M9 (weeks 18–20): Phase 8a/b/c with tier-gated install |
| `M10-first-run-polish.md` | TODO | M10 (weeks 21–22): §14 onboarding + Settings + Help |
| `M11-ship-prep.md` | TODO | M11 (weeks 23–24): unsigned .msi + SHA-256 + Sigstore + release |

**M01 is the proof-of-concept**, staged into A/B/C/D within a single milestone document to avoid the prompt-too-long failure mode (an unstaged 540-line prompt is too much for a fresh-session opening message; see TEMPLATE.md scope-split rule). The four stages commit sequentially on one feature branch; the M1 PR drafts only at the end of Stage D, including all stage commits and per-stage retrospectives + parent-milestone summary per `CLAUDE.md` §19. After M1 runs, lessons learned go into `CLAUDE.md` (where appropriate) and `TEMPLATE.md`. Many of M02–M11 will themselves stage per the rule; staging happens at authoring time, not run time.

## How to use a milestone document

1. **First**, read the entire `M[NN]-<title>.md` document end-to-end — it's your spec, design rationale, and stage roadmap in one file.
2. **For each stage**, open a **fresh** Claude Code session (cleared context — don't continue from prior session work).
3. Copy the **stage's CLI Prompt** (section X.5 of the milestone document, where X is `A`/`B`/...) into the fresh session as the opening message. The stage prompt instructs Claude to read CLAUDE.md (auto-loaded), the stage's X.1–X.4 sections, and any other files the prompt names.
4. Add any session-specific overrides at the top: branch name (default `claude/m[nn]-<title>`), time-box if applicable. Keep it minimal — the prompt is intentionally complete.
5. Claude does TDD work for the stage, runs gates, fills in the per-stage retrospective, drafts the stage commit message, surfaces it all. **Claude does not commit.** You review, approve, and Claude then commits the stage on the parent-milestone branch (does NOT push between stages).
6. **After the final WORK stage** (e.g., Stage D in M01), Claude creates the parent-milestone summary (`M[NN]-summary.md`).
7. **Phase Closeout (final stage; Stage E in M01)** runs the gap analysis pass per `CLAUDE.md` §20: append a new entry to `docs/gap-analysis.md` (append-only — prior entries immutable). This commit is the final commit on the parent-milestone branch and gates the PR push.
8. On approval of the Phase Closeout commit, Claude pushes the branch and (if explicitly requested) drafts the M[NN] PR description.
9. After the milestone PR merges, the next milestone starts fresh with its own document and stage prompts.

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
