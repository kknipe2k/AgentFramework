# CLAUDE.md — Agent Runtime project memory

> **Read this first.** Every session in this repository should load and follow this file. It defines what the project is, how work proceeds, what tests must pass, and the explicit rules for committing and PR'ing. Per-milestone prompts in `docs/build-prompts/` add scope-specific guidance on top of these constants — they don't replace them.

This file describes the **runtime** project (Tauri + Rust desktop runtime for agentic AI workflows). The existing shell-based ARIA framework keeps its own project memory at `.aria/CLAUDE.md`, which Claude Code auto-loads when the working directory is `.aria/`. The two products coexist; the runtime is the active build, ARIA is reference material.

---

## 1. Project identity

**What this is**
A local Tauri desktop runtime for agentic AI workflows. Live graph of agent execution, capability sandboxing, gap detection that suspends the session cleanly when an agent needs something it doesn't have, and a workbench where novices and experienced users build agentic processes the same way.

**What this isn't**
A chatbot wrapper. A framework. A general-purpose terminal. A low-code tool for non-technical users in v1. The runtime executes what exists; it doesn't modify itself mid-run.

**Stack (locked)**
- **Shell:** Tauri 2.x (uses OS webview)
- **Backend:** Rust 1.80+ (workspace at `crates/`)
- **Async runtime:** tokio
- **Frontend:** React 18 + TypeScript + React Flow + Tailwind + Vite (in `src/`)
- **LLM client:** direct HTTP+SSE to Anthropic via `reqwest` + `eventsource-stream` (no third-party SDK)
- **Persistence:** SQLite via `rusqlite` (WAL mode)
- **IPC:** Tauri typed IPC (renderer ↔ main); Unix socket / Windows named pipe with framed JSON (main ↔ drone)

Stack rationale lives in **ADR-0002**.

**License:** Apache 2.0. Contributions via DCO sign-off (`git commit -s`). See `CONTRIBUTING.md`.

**Status:** pre-implementation. The runtime binary does not yet exist. M1 (Foundation) is the next deliverable — see `docs/MVP-v0.1.md` and `docs/build-prompts/M01-foundation.md`.

---

## 2. Read-first list (orient before any work)

In a fresh session, read these in order before writing any code or making decisions:

| # | File | Read for |
|---|---|---|
| 1 | `CLAUDE.md` (this file) | The constants — protocol, gates, anti-patterns, decision rules |
| 2 | `docs/build-prompts/M[N]-*.md` | The milestone you're working on (if applicable) |
| 3 | `agent-runtime-spec.md` §0–§0d | Project positioning, capability matrix, three-concept model, dev loop, release scope. **Always read.** |
| 4 | `agent-runtime-spec.md` (relevant phases) | Whatever phase the milestone touches (§1 drone, §2 SDK, §3 graph, §3a plan, §3b mode, §4 gap, §4a verify+rails, §5 MCP, §6 framework, §7 registry, §8 generators, §11 reconciliation, §12 charter, §13 privacy, §14 first-run) |
| 5 | `docs/MVP-v0.1.md` §M[N] | Milestone-specific scope and acceptance criteria |
| 6 | `docs/adr/` | ADRs by topic; at minimum 0001 (archetype), 0002 (Tauri), 0003 (charter), 0004 (code-signing) |
| 7 | `schemas/*.v1.json` | When touching framework JSON or artifact shapes |
| 8 | `examples/aria/` | Reference framework reconstruction (the archetype proof) |

**Rule:** if the spec, MVP doc, an ADR, and this file disagree, surface the contradiction and ask. Don't pick. The spec is the contract; this file is the execution protocol; both are intended to be consistent. Drift is a bug.

---

## 3. Project state (current, as of last update)

- **Stack locked:** Tauri + Rust + TS/React (ADR-0002).
- **Scope locked:** §0d Release Scope Matrix in spec. v0.1 is Windows-only, single-session, Novice + Promoted tiers, STANDARD mode hardcoded, `fresh_context_per_task` loop policy, Anthropic-only.
- **Next milestone:** M1 Foundation — Cargo workspace + drone + runtime-core types + CI green. Detailed prompt at `docs/build-prompts/M01-foundation.md`.
- **What's authored:** spec, schemas, two reference frameworks (`examples/aria/`, `examples/ralph/`), MVP build checklist, ADRs 0001–0004, OSS scaffolding (LICENSE, SECURITY, CoC, CONTRIBUTING, .github), launch communication drafts.
- **What's NOT authored yet:** any Rust crate, any TypeScript code, any actual binary, any GitHub Release.

The implication: most early sessions are **build-from-scratch sessions**. Each milestone produces real code, tests, and CI changes — not just docs. Documentation-only work continues only if a session uncovers a spec gap that must be closed before code can land.

---

## 4. Hard rules (do not violate)

These are non-negotiable. Violations require an explicit user override before proceeding.

1. **Do not commit any code without user approval.** When work is done, draft the PR description and surface it. Wait for explicit approval. Then commit and push. **Never** auto-commit. See §10 below for the workflow.
2. **Do not push to `main`.** Develop on feature branches off `main`. Merge via PR with at least one maintainer review (per `.github/CODEOWNERS` and §12).
3. **Do not skip CI gates.** No `--no-verify`. No commenting out failing tests "to come back to later." If a gate fails, fix or surface — not bypass.
4. **Do not add telemetry, analytics, or crash reporters.** Per §13 of the spec, the runtime collects nothing about the user. Adding any phone-home requires an ADR with public dashboard plan + opt-in mechanism. Default: don't.
5. **Do not write hand-rolled types that should be generated from schemas.** `runtime-core` types come from `schemas/*.v1.json` via `typify`. TS types come from the same via `json-schema-to-typescript`. CI fails if committed types differ from regenerated. To change a type, change the schema (and bump version per the schema versioning policy in `schemas/README.md`).
6. **Do not introduce new third-party dependencies without `cargo deny check` passing.** License compatibility (no GPL/AGPL), supply-chain hygiene (no unmaintained crates per RustSec), no duplicate major versions.
7. **Do not write `unsafe` code outside `crates/runtime-sandbox/`.** Workspace lints `forbid(unsafe_code)` everywhere else. The sandbox needs unsafe for seccomp / landlock / Job Objects integration; every block in there requires a `// SAFETY:` comment naming invariants. Other crates: no unsafe, period.
8. **Do not modify capability enforcement, drone, sandbox, or providers without surfacing a plan first.** These paths are CODEOWNERS-flagged. Even when working solo, treat them as if a security reviewer would block — write the plan, run it past the spec/ADRs, then code.
9. **Do not modify `.aria/` or `archive/aria-shell/`.** That's the shell ARIA reference material. It moves to `archive/aria-shell/` at v0.1 ship time but stays untouched as code/docs. If the runtime needs to interact with the shell version (e.g., to import its signals), the work happens in the runtime crates, not by editing the shell tree.
10. **Do not invent project scope.** v0.1 is what §0d says it is. Adding features means equivalent removals or pushing to v1.0+. Out-of-scope PRs get queued, not merged.

---

## 5. TDD discipline

Tests are the contract. Write them first. Code follows.

### The cycle

For every behavior change:

1. **Red.** Write a single failing test that captures the next bit of behavior. Run it. Confirm it fails for the *right reason* (the assertion you care about, not a setup error).
2. **Green.** Write the minimum code that makes the test pass. Not the cleanest. Not the most general. The minimum.
3. **Refactor.** With the test passing, clean up. The test pins behavior; refactor freely.

A micro-cycle should take 5–15 minutes. If a cycle is longer, the test is too big — split it.

### What counts as a real test

- **Asserts something specific.** A test that calls a function and doesn't assert anything is decoration, not a test.
- **Fails when the production code is wrong.** If you delete a key invariant from the production code and the test still passes, the test is missing the assertion that matters. Verify by mutation: try breaking the production code and confirm tests fail.
- **Doesn't tautologically restate the implementation.** `fn add(a, b) { a + b }` paired with `assert_eq!(add(2, 2), 4)` is a tautology test if the production code is just `a + b`. Tests should assert behavior the user observes — error cases, edge inputs, boundary conditions, integration outcomes — not internal structure.
- **Hard-fails on missing exports / dependencies.** A behavioral test must fail loudly when a required production export, function, or fixture is missing — **never silently skip, never tautologically pass via mocking around the gap.** First run after writing the test should fail with `cannot find function X` / `unresolved import` / `module not found`, not pass-by-skip. Skipping a behavioral test because "the function isn't implemented yet" defeats TDD's red phase. Use real imports; if the import is wrong, surface it immediately.
- **Reproducible.** No reliance on wall-clock time, network, or random seeds without explicit seeding. Use `tokio::time::pause()` for time-dependent tests.

### Coverage thresholds (per §12)

- **≥80% line coverage** on all new code (Rust: `cargo-llvm-cov`; TS: `vitest --coverage`).
- **100% on safety primitives:** drone (`crates/runtime-drone/`), capability enforcer (`crates/runtime-main/src/capability/`), plan state machine, snapshot/recovery code paths.
- Coverage drops vs prior `main` block PR merge. CI computes the delta.

### Test types and when they apply

| Type | Tool | Required from |
|---|---|---|
| Unit (Rust) | `cargo test` | M1 onward |
| Property (Rust) | `proptest` | M1 onward — required for: serde round-trips, state machine transitions, IPC frame encoders |
| Fuzz (Rust) | `cargo-fuzz` | When parsers exist (M1 has the IPC frame parser; M2 has SSE; M6 has MCP JSON-RPC) — short fuzz on PR, long fuzz nightly |
| Integration (Rust) | `cargo test --features integration` + `wiremock` | M2 onward |
| Unit (TS) | Vitest | M3 onward |
| E2E (Playwright) | `npm run test:e2e` against built app | M3 onward — required from when the renderer can run a session |
| Doc tests | `cargo test --doc` + `tsc` on TS examples | Any milestone that adds public API |

### Behavior tests vs implementation tests

- **Behavior tests** assert what the user observes. "When the user clicks Test in the canvas, the live graph renders an `agent_spawned` node." That's a Playwright test that drives the UI, not a unit test that calls a graph reducer.
- **Implementation tests** assert how the code works internally. "The `handle_event` function calls `dispatch` with this argument." Useful for catching regressions in pure logic but invisible to the user.
- **Both are needed.** Heavy on behavior tests at integration boundaries (M3+ Playwright; M5 capability enforcement; M9 generators); heavy on implementation tests for pure-logic crates (`runtime-core` types, capability evaluator, plan state machine).
- **A test suite that is 90% implementation tests is a smell.** Means you're testing the code you wrote, not the contract the code was supposed to satisfy. Add behavior tests until the suite catches a class of bug that pure unit tests would miss.

### Mutation testing (advisory)

`cargo-mutants` runs nightly on `main` and surfaces "code mutations the test suite didn't catch." Treat its findings as test-quality issues to address, not failures to fix immediately. A mutation that survives means there's an assertion your tests aren't making.

---

## 6. Quality gates (the must-pass list)

Before any commit lands, **all** of these must pass locally and on CI. No exceptions.

### Rust gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --doc
cargo doc --workspace --no-deps
cargo audit
cargo deny check
cargo llvm-cov --workspace --fail-under-lines 80
```

CI runs all of these on Linux/macOS/Windows × stable + MSRV.

### Frontend gates (when `package.json` exists)

```bash
npm ci
npx prettier --check '**/*.{ts,tsx,js,jsx,json,md,yml,yaml}'
npx eslint .
npx tsc --noEmit
npm run test
npm audit --audit-level=high
```

### E2E gates (when renderer can run a session, M3+)

```bash
npm run test:e2e   # Playwright against the built Tauri app
```

### Schema gates (always — already in CI)

```bash
# All examples/*/framework.json validate against schemas/framework.v1.json
# All skill/agent/tool frontmatter validates against the right schema
# Cross-validated by CI; a Python script in .github/workflows/ci.yml does it
```

### Pre-commit hook

Install once: `lefthook install` (configured via `lefthook.yml`; chosen for single-binary deployment with no Python dependency — see `docs/build-prompts/M01-foundation.md` Stage A).

The hook runs the fast subset of gates locally on every `git commit`. CI mirrors the hook to prevent `--no-verify` bypass.

---

## 7. The self-correction loop

When gates fail, work through them deterministically.

### Algorithm

```
1. Run all applicable gates (Rust, frontend, schema, E2E if relevant).
2. Collect every failure in one pass — don't fix the first one and re-run blindly.
3. For each failure:
   a. Parse the error (Cargo / clippy / Vitest / Playwright output).
   b. State a hypothesis for the cause in 1 sentence.
   c. Make the smallest fix that addresses the hypothesis.
   d. Re-run only that one gate; confirm green.
4. After all individual fixes: re-run ALL gates from scratch.
   (A fix can break something that was passing.)
5. Iterate. Maximum 3 rounds before escalating to the user.
```

### When to escalate

After 3 rounds without all-green, **stop and surface**:

- What you tried (1 line per attempt: hypothesis + fix + result)
- Current failures (full error output, not summarized)
- Your best current hypothesis
- What you would try next, if anything

Do not silently try a fourth round. The user prefers a 90-second pause to discuss over a 3-hour rabbit hole.

### Anti-patterns in self-correction

- **"This test is flaky, I'll just retry."** Flaky tests are failing tests. Diagnose the source of nondeterminism (clock, network, ordering) and fix it. Never `#[ignore]` a flaky test as a "fix."
- **"Let me just bump the timeout."** Sometimes correct, often hides a real issue (deadlock, slow path, missing await). Investigate before bumping.
- **"This warning is harmless."** Then the lint should be configured to allow it (with rationale). Don't `#[allow(...)]` ad-hoc; configure once in `Cargo.toml` or the rustfmt config.
- **"I'll come back to this."** TODOs without linked issues become permanent. Either fix now or open an issue and reference it from the TODO comment.

---

## 8. PR + commit workflow (CRITICAL — read carefully)

**The single most important rule:** Claude does not commit without explicit user approval.

### What "done" looks like

A unit of work is "done" when:

1. All applicable acceptance criteria for the milestone (per `docs/MVP-v0.1.md` §M[N] and the milestone prompt) are checked.
2. All quality gates pass locally.
3. CI would pass (predict: every gate has been run; nothing skipped).
4. Documentation updated where the change touches public surface or §0a primitives.
5. ADR filed if required (see §11 below).
6. CHANGELOG.md `[Unreleased]` section updated.
7. AI-assistance disclosure prepared (see §13).

### When done, draft the PR — don't commit yet

When a unit of work satisfies "done," do this:

```
1. Run `git status` to confirm what's staged, unstaged, untracked.
2. Run `git diff --stat HEAD` to summarize the diff.
3. Re-run ALL quality gates one final time. Capture exact results.
4. Draft the PR description following .github/PULL_REQUEST_TEMPLATE.md.
   Include:
     - What this PR does (one paragraph)
     - Linked issue / ADR
     - Type of change
     - Scope check (which §0d row)
     - Tests added/updated (with coverage delta)
     - Quality gate results (each gate, pass/fail, key numbers)
     - Capability/security review (when relevant)
     - DCO sign-off plan
     - AI assistance disclosure
     - Documentation updated
     - Breaking changes (if any)
5. Surface to the user:
     - PR title (Conventional Commits format)
     - PR description (markdown, ready to paste)
     - Diff stat output
     - Quality gate results
     - State explicitly: "I will not commit until you approve."
6. Wait. Do not commit, do not push, do not create a PR.
```

### What the user does next

- **Approve as-is** — proceed to commit + push.
- **Request changes** — iterate on the work, then re-surface.
- **Approve with edits** — user may edit the PR description; use what they provide.
- **Abort** — discard or shelve the work.

### After approval

```
1. Stage exactly what's intended. Use specific filenames; not `git add -A` for surprise commits.
2. `git commit -s -m "..."` — DCO sign-off mandatory.
   Commit message: Conventional Commits (feat / fix / docs / refactor / etc.).
   Include the standard session URL footer.
3. `git push -u origin <feature-branch>`.
4. If the user wants a PR opened: use the GitHub MCP `mcp__github__create_pull_request`
   tool with the description drafted in step 4 above.
   Don't open the PR unless the user has explicitly asked for one.
5. After push, run `git status` to confirm clean working tree.
```

### Commit message template

```
type(scope): summary in imperative present tense (≤72 chars)

Optional body explaining what and why (not how — code shows how).
Wrap at 72 chars per line. Use bullet lists when describing multiple changes.

- Specific change 1
- Specific change 2

References:
Refs: #123 (issue)
Closes: #456 (when this PR closes the issue)

https://claude.ai/code/session_<id>
```

### Branch hygiene

- Develop on a feature branch off `main`. Never commit on `main` directly.
- Branch naming: `claude/<short-kebab-description>` for Claude-driven work.
- One branch per logical unit of work. M1 might be one branch with several commits, or several branches if M1 splits naturally.
- Squash-merge for small, single-concept PRs.
- Merge-commit (preserve history) for milestone PRs that have valuable commit-level history (e.g., "drone heartbeat lands → snapshot lands → recovery lands" in three commits is worth preserving).
- Maintainer chooses squash vs merge per PR.
- Never force-push to `main`. Force-push to a feature branch is acceptable if you're the only contributor to it.
- Delete merged branches; keep the branch list clean.

### What "do not commit" means specifically

- Don't run `git commit` — period — until user has seen the PR description and approved.
- Don't run `git push` until user has approved.
- Don't open a GitHub PR until user has explicitly asked.
- Don't auto-merge, don't auto-squash, don't auto-rebase.
- Don't `git checkout` to a different branch with uncommitted changes (it can lose them or carry them).
- Don't `git stash` to "set aside" work; surface it.

When unsure: `git status`. State what you see. Wait.

---

## 9. Style and naming

Full conventions live in **`docs/style.md`** — read that file for comments, naming (Rust + TS + skill files + schemas), function design, error handling, and the project-wide anti-patterns list. Summary of the load-bearing rules:

- **No comments by default.** Comments explain *why* (hidden constraint, subtle invariant, workaround), not *what*. No marketing language.
- **Naming is conventional per language** (snake_case in Rust, camelCase in TS, kebab-case for `.md` artifacts). Names describe what, not how.
- **Functions do one thing**, ≤50 lines, ≤3 params. Pure preferred; effects at the edges.
- **Errors:** Rust `Result<T, E>` (thiserror in libs, anyhow at boundaries); TS throw `Error` subclasses or return discriminated unions. Capture root cause.
- **Anti-patterns to avoid:** hidden AI usage, magic numbers, stringly-typed APIs, `any`, ad-hoc `#[allow(...)]` / `// @ts-ignore`, implementation-detail tests, generic names (`helper`, `util`, `process`), silent error drops, one-line dep adds, premature abstraction (wait for the fourth).

---

## 10. Don't-touch zones and capability adherence

### Don't-touch (hands off)

- `LICENSE` — legally important. Changes require maintainer + ADR.
- `NOTICE` — same.
- `.git/` — never via Edit. Use `git` commands.
- `archive/aria-shell/` — reference material. Read-only after v0.1 ship.
- `.aria/` — same as `archive/aria-shell/` after the move (currently `.aria/` because v0.1 hasn't shipped yet; treat as read-only either way).
- Other contributors' open PRs — don't rebase or rewrite without their explicit ask.
- `schemas/*.v1.json` — changes require schema versioning bump (new file `*.v2.json`) + ADR. Don't edit existing `v1` files except for clarification that doesn't change validation behavior.
- `examples/aria/framework.json` and `examples/ralph/framework.json` — these are the archetype proofs. Changes here should be reviewed against §0a Capability Matrix; if a change makes a matrix row no longer reconstructible, that's a regression that blocks v0.1.
- `docs/gap-analysis.md` **prior entries** — append-only per §20. Never edit, reorder, or delete an entry once committed. New milestones add a section at the bottom only; status updates on prior findings go in the current milestone's "Carry-forward" section. CI enforces this via diff check (added M01 Stage E alongside the first entry).

### Capability adherence

The §8.security model in the spec applies to the project's own code, not just user-generated artifacts:

- **Built-in tools declare their capabilities.** Even runtime-built-in tools (`Read`, `Write`, `Bash`) get capability declarations enforced.
- **Tests respect capabilities.** A test that exercises a tool that should not access network shouldn't accidentally make a network call. Use `wiremock` or in-process fakes; never let a test reach the real internet without `--features integration` and explicit opt-in.
- **Sandbox process is the boundary.** Anything that runs untrusted code (L3 validation, generated artifact execution) goes through `crates/runtime-sandbox/`. The main process never directly executes untrusted input.
- **Don't loosen capabilities to make a test pass.** If a test fails because a capability is too narrow, the right fix is usually narrowing the implementation, not widening the capability.

### Read-only paths during a session

- Anything outside `crates/`, `src/`, `src-tauri/`, `docs/`, `schemas/`, `examples/`, `.github/`, and the root config files (`Cargo.toml`, `package.json`, `tauri.conf.json`, `tsconfig.json`, `rust-toolchain.toml`, `.gitignore`, `.gitattributes`) — don't write there.
- The shell ARIA tree (`.aria/` or `archive/aria-shell/` post-move) is read-only.
- `.git/` is read-only via Edit/Write; use `git` commands instead.

---

## 11. ADRs (Architecture Decision Records)

Per §12 Engineering Charter, an ADR is required for any change that:

- Adds, modifies, or removes a §0a Capability Matrix primitive
- Changes any `schemas/*.v*.json` file (new major version requires new file + ADR)
- Adds a new `LLMProvider` impl
- Changes capability enforcement behavior (any §8.security L1–L5 layer)
- Changes the IPC protocol between main, drone, or sandbox
- Adopts a new core dependency (anything that becomes a runtime dependency, not dev-only)
- Significantly changes scope of v0.1 / v1.0 / v2.0 per §0d

Smaller decisions (refactors, internal abstractions that don't cross primitive boundaries, minor optimizations) don't require an ADR — a clear PR description is enough.

### How to file an ADR

1. Copy `docs/adr/0000-template.md` to `docs/adr/NNNN-short-title.md`. Use the next available number.
2. Fill in every section. Status starts `Proposed`.
3. PR includes the ADR + the change it documents.
4. On merge, status flips to `Accepted` (do this in the PR before merging).
5. ADRs are immutable once accepted. To change, file a new ADR that supersedes the old one (and add `Superseded by ADR-XXXX` to the old one's Status line in the same PR).

Existing ADRs:
- **0001** ARIA as Archetype — positioning decision; ARIA reconstructed inside the runtime, not bundled as default
- **0002** Tauri + Rust over Electron — stack choice
- **0003** Engineering Charter adoption — process gates
- **0004** Defer paid code-signing for v0.1 — distribution integrity via SHA-256 + Sigstore

---

## 12. When to ask vs when to proceed

### Operating mode (default)

The user is the project's product owner / VP, not a hands-on engineer. They direct via spec, PRs, and one-word approvals. **Default to executing, not consulting.** Specifically:

- When the next action is clear from prior direction, **do it** — don't ask for sub-step approval. Surface the outcome (diff, PR, gate result) for a single approval.
- Don't propose options when the action is obvious. Propose options only when the choice is genuinely the user's to make (scope, priority, an irreversible architectural decision).
- Never ask the user to run diagnostic commands they don't want to run. If something needs investigation, do it from the agent side. If something must happen on the user's machine that can't be done remotely (e.g., Windows-side merges, fresh-session prompt pastes), give **one command** they can paste — not a flow.
- For anything that can stay on the agent side (commits, pushes, PRs, merges to feature branches, doc updates), the agent does it autonomously and surfaces the result. The Hard Rules in §4 still apply (do-not-commit-without-approval, don't-push-to-main, etc.) — that's *outcome* approval, not *step* approval.
- The user approves outcomes; the agent figures out steps.

This applies to both this Claude session (orchestration / spec / docs / GitHub) and to Claude-on-the-build-machine (which does code/test/commit work for stages A–E and surfaces a single approval-ready bundle per stage).

### Proceed without asking

- Routine TDD steps within the milestone's stated scope
- Adding a dependency that's already named in the spec (`reqwest`, `eventsource-stream`, `rusqlite`, `keyring`, `tokio`, etc.)
- Refactors that don't change observable behavior (covered by existing tests)
- Documentation updates that match landed code
- Test additions for existing code
- Fixing a clippy warning that the lint config flags
- Renaming a private function for clarity

### Ask first

- Any spec ambiguity or contradiction
- Any feature that would land outside the milestone's stated `Out of scope`
- Adding a new dependency NOT named in spec, MVP doc, or any ADR
- Any change to capability enforcement, drone protocol, sandbox boundary, or LLM provider integration
- Any schema change
- Removing or relaxing a quality gate
- Any change that touches §0a Capability Matrix primitives
- Anything that takes longer than expected (>3 self-correction iterations)
- Anything that requires a new ADR

### How to ask

- State the situation in 1-3 sentences.
- State the options (usually 2-3).
- State your recommendation.
- Ask for the decision.

Don't ask without recommending. Don't recommend without options. Don't dump a wall of context — the user has the spec; reference the relevant section.

---

## 13. AI-assistance disclosure

This project is largely Claude-written; that's disclosed publicly. Every commit and PR makes the disclosure explicit.

### Commit messages

Every Claude-authored commit ends with the session URL footer:

```
https://claude.ai/code/session_<id>
```

The session URL is set per session by the harness; use whatever URL is current. If working in a multi-session continuation, use the latest session's URL.

### PR descriptions

Per `.github/PULL_REQUEST_TEMPLATE.md`, the AI-assistance disclosure section is required:

```markdown
## AI assistance disclosure

- [x] AI tools used; described in commit messages and here:
      Claude Code wrote the implementation; reviewed and edited by maintainer.
      Direction, scope, and final acceptance by human.
```

If a PR is genuinely unassisted, check the "No AI tools used" box. Don't claim manual work that wasn't manual.

### Code comments

Don't add `// Generated by Claude` or similar comments to source files. The disclosure lives at the commit/PR level, not in the code. Code is code.

---

## 14. Schemas as source of truth

Per §12 Engineering Charter, schemas in `schemas/` are the **single source of truth** for the shapes of artifacts the runtime consumes. Rust types and TS types are *generated*, not hand-written.

### Generation pipeline

```
schemas/framework.v1.json  ──┬──> crates/runtime-core/src/framework.rs   (typify)
                             └──> src/types/framework.ts                 (json-schema-to-typescript)

schemas/common.v1.json     ──┬──> crates/runtime-core/src/common.rs
                             └──> src/types/common.ts

(same pattern for skill, tool, agent)
```

### Rules

- **Don't hand-write types** that should come from schemas. CI runs the generators and fails if committed types differ from regenerated.
- **To change a type, change the schema.** Bump version per `schemas/README.md` versioning policy. File an ADR.
- **Adding a field** is usually a minor bump (`v1.1`). The `$id` URL doesn't change for minor bumps; the file is updated in-place.
- **Removing or restricting** a field is a major bump. New file `*.v2.json`. Old `*.v1.json` stays for back-compat.
- **The runtime supports the major versions it ships with.** Loaders dispatch by `$schema` URL in the loaded document.

### Regeneration command

```bash
# When a schema changes, regenerate types:
cargo xtask regenerate-types
```

(This `xtask` doesn't exist yet at M0. M1 sets it up; subsequent milestones use it.)

---

## 15. Common gotchas (lessons learned)

The full numbered list of 20 traps lives in **`docs/gotchas.md`** — read it once at session start when working in unfamiliar areas. The cluster:

- **Concept boundaries:** Tool ≠ Skill ≠ Agent; Drone ≠ Main ≠ Sandbox; capability narrowing on Agent→Agent edges.
- **v0.1 scope locks:** STANDARD mode only, `fresh_context_per_task` only, Novice + Promoted tiers only, single-session, Windows-only (CI on all three OSes).
- **Hard rules from the spec:** no telemetry ever, Anthropic API hit directly (no SDK), Tauri allowlist is the security boundary.
- **IPC + persistence:** two IPC layers (Tauri ↔ framed JSON), SQLite WAL pragmas in order, snapshots append-only, resume rebuilds (doesn't re-execute), `tool_call_uncertain` flag.
- **Runtime mechanics:** `request_capability` meta-tool, mode-variant skill section filtering, JSONLogic operator allowlist, generated-artifact capability declarations mandatory.
- **Process:** DCO sign-off (`git commit -s`) mandatory.

When in doubt, open `docs/gotchas.md` and find the relevant trap before writing code that touches the area.

---

## 16. Session-start checklist

When opening a fresh session in this repository, before any code is written:

- [ ] Read `CLAUDE.md` (this file). Confirm understanding by stating the rules in §4 (Hard rules) at session start.
- [ ] Identify the milestone. Which M[N] in `docs/MVP-v0.1.md`? If working a milestone, read `docs/build-prompts/M[N]-*.md`.
- [ ] Read the relevant spec sections (always: §0–§0d; per milestone: phases referenced in the milestone prompt).
- [ ] Read relevant ADRs (the milestone prompt names them).
- [ ] Run `git status` and `git log --oneline -5` to understand current branch state.
- [ ] Confirm the branch matches the work. If unclear, ask before proceeding.
- [ ] State the deliverable in 1-3 sentences before writing code.
- [ ] State the test plan in 3-5 bullets before writing code.

This checklist is the orientation before TDD's "Red" phase. Skip it and you'll discover halfway through that the work is on the wrong branch, scope, or premise.

---

## 17. Reference index (where things live)

| Need | File |
|---|---|
| What we're building | `agent-runtime-spec.md` (especially §0–§0d) |
| Current scope and milestones | `docs/MVP-v0.1.md` |
| Per-milestone prompt | `docs/build-prompts/M[NN]-*.md` |
| Per-milestone template | `docs/build-prompts/TEMPLATE.md` |
| Per-stage retrospectives | `docs/build-prompts/retrospectives/` |
| Cumulative gap analysis (append-only) | `docs/gap-analysis.md` (per §20) |
| Persistence architecture (HLA) | `docs/persistence-architecture.md` |
| Architecture decisions | `docs/adr/` |
| Schemas (source of truth) | `schemas/*.v1.json` |
| Reference framework (archetype proof) | `examples/aria/` |
| Sibling framework (Ralph) | `examples/ralph/` |
| Style and naming conventions | `docs/style.md` |
| Common gotchas (20 traps) | `docs/gotchas.md` |
| Engineering charter | `agent-runtime-spec.md` §12 |
| Privacy & telemetry | `agent-runtime-spec.md` §13 |
| First-run UX | `agent-runtime-spec.md` §14 |
| Security disclosure flow | `SECURITY.md` |
| Code of Conduct | `CODE_OF_CONDUCT.md` |
| Contributor guide | `CONTRIBUTING.md` |
| License | `LICENSE` (Apache 2.0) |
| Changelog | `CHANGELOG.md` |
| CI workflow | `.github/workflows/ci.yml` |
| Release workflow | `.github/workflows/release.yml` |
| Issue / PR templates | `.github/ISSUE_TEMPLATE/`, `.github/PULL_REQUEST_TEMPLATE.md` |
| CODEOWNERS | `.github/CODEOWNERS` |
| Shell ARIA reference (read-only) | `.aria/` (currently) → `archive/aria-shell/` (post-v0.1) |
| Shell ARIA project memory | `.aria/CLAUDE.md` |

---

## 18. Versioning of this document

This file changes when:
- The spec changes in a way that affects how Claude works (rare; spec is the contract)
- Quality gates change (e.g., adding a new lint, raising coverage threshold)
- The PR/commit workflow changes
- New common gotchas are discovered

Substantive changes are committed with a clear `docs(claude-md): ...` commit message and noted in `CHANGELOG.md`. The commit history of this file is a useful audit of "how the project's working agreements evolved."

If this file disagrees with the spec or an ADR, the spec/ADR wins; this file is the execution protocol layered on top, not a source of truth for design decisions.

---

## 19. Retrospective protocol (Claude-driven)

**Every milestone session produces a retrospective.** Claude maintains it during the session and surfaces it alongside the PR. The user reviews — does not fill in fields.

Full protocol (per-stage workflow steps, scoring rubric, threshold gates, outcome routing, cross-milestone trends) lives in:

- **`docs/build-prompts/PROCESS-VALIDATION.md`** — framework reference (3 axes, 5 hard + 5 soft gates, outcome matrix)
- **`docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md`** — per-stage shape Claude fills in
- **`docs/build-prompts/retrospectives/SUMMARY-TEMPLATE.md`** — per-parent-milestone roll-up shape

### The non-negotiable rules

1. **Stage B onward must read prior stage retrospectives before writing code.** First action in any non-first stage: read every prior stage's `[END] Decisions for the next stage` section and the most recent `docs/gap-analysis.md` Carry-forward section. **Apply those decisions** — that's why they exist. Stage X.5 CLI prompts embed this read step; this rule is the backstop.
2. **Fill in the live observation log AS friction surfaces.** Friction, ambiguity, surface, protocol-drift, surprise events get logged in real time — not summarized at session end. Details fade.
3. **Honest self-assessment.** If you self-corrected through 4 rounds when 3 was the budget, log it. If a friction event was severity 4, score it 4. A retrospective claiming everything 5/5 with no friction is itself a flag.
4. **Stage end:** score 3 axes per `PROCESS-VALIDATION.md`, evaluate threshold gates, mark outcome (Sound / Sound-but-rough / Friction-heavy / Not-ready), write specific Decisions for the next stage (cite file:line, name the change, name the gate).
5. **Final stage of a parent milestone:** also write `M[NN]-summary.md` aggregating across stages and draft the PR description.
6. **Surface and wait.** Per `CLAUDE.md` §8, do not commit until user approves. State explicitly: *"Stage `<X>` is ready. I will not commit until you approve."* User especially validates Hard Gate G1 against the git log.

### Outcome routing (after user approval)

- **Sound** → proceed (apply minor `CLAUDE.md` / `TEMPLATE.md` updates from Decisions if substantive).
- **Sound but rough** → brief protocol-iteration session first, then proceed.
- **Friction-heavy** → stop; iterate on protocol before next stage.
- **Not ready** → hard gate failed; diagnose, possibly file ADR, possibly recovery session.

---

## 20. Gap Analysis Protocol (append-only, per-milestone)

**Every parent milestone produces a Gap Analysis entry** in `docs/gap-analysis.md`, separate from per-stage retrospectives. Retrospectives evaluate the build *process*; gap analysis evaluates the build *product* (does code match spec, what did spec get wrong, prioritized fix backlog).

Full entry template, append-only enforcement details, and the running milestone log live in **`docs/gap-analysis.md`**.

### The non-negotiable rules

1. **Append-only — Hard Rule.** Per §10, no prior entry may be edited, reordered, or deleted. Resolution of a prior finding goes in the *current* milestone's Carry-forward section, referencing the prior entry's milestone tag. Example: `M01 critical "X" — resolved at <crate/file.rs:line>`. CI enforces this via diff check (added in M01 Stage E).
2. **When it runs.** After the final stage of a parent milestone commits and the per-parent-milestone summary lands, but **before** the milestone PR opens. The gap-analysis commit is the **final commit on the parent-milestone branch** and gates the PR push.
3. **Six sections per entry, none optional** (write "None observed." rather than omit): (1) Codebase deep dive — cumulative, 200–500 words; (2) Adherence to spec — ✅ / ⚠️ / ❌ with file:line; (3) Spec review forward-looking — missing/contradicted/ambiguous; (4) Fix backlog — 🔴 Critical / 🟡 Important / 🟢 Nice-to-have, severity non-elastic; (5) Carry-forward from prior milestones; (6) Sign-off.
4. **What it is NOT.** Not a retrospective (process). Not the changelog (what shipped). It's product↔spec evaluation, cumulative.
5. **Three-artifact PR review.** User reviews code diff + retrospectives/summary + gap-analysis entry together. Pushback on any of the three blocks the PR until Claude revises.

### Why append-only is a Hard Rule

Audit trail integrity (the M01→M11 chain documents drift); honest assessment (knowing it's permanent forces accuracy); forces forward-looking carry-forward (resolution lives next to its date and commit context); spec-drift detection across milestones stays visible.
