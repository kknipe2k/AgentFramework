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

### Comments and prose in code

- **No comments by default.** Code expresses what; comments explain *why*.
- A comment is justified when:
  - There's a hidden constraint (e.g., "must run before X because Y").
  - There's a subtle invariant (e.g., "this map can be empty briefly during Z; downstream handles that").
  - There's a workaround for a specific bug (link the bug).
  - The behavior would surprise a reader on first read.
- A comment is **not** justified when it just restates what the code does. Don't write `// Increment counter` above `counter += 1`.
- **No marketing language.** Code comments and commit messages don't have "🚀", "blazing fast", "revolutionary." Plain technical prose only.
- **No "TODO: optimize later" without a linked issue.** Either open the issue and reference it (`// TODO: #N — improve hot path`) or don't write the TODO.

### Naming

- **Rust:** `snake_case` for functions/vars; `CamelCase` for types; `SCREAMING_SNAKE_CASE` for constants. Module names are short, lowercase. Crate names are kebab-case (`runtime-core`). File names mirror module names.
- **TypeScript:** `camelCase` for vars/functions; `PascalCase` for types/components; `SCREAMING_SNAKE_CASE` for constants. File names: `PascalCase.tsx` for React components, `camelCase.ts` for utility modules.
- **Skill / tool / agent .md files:** kebab-case (`code-simplifier.md`, `git-checkpoint.md`).
- **Schema files:** `<name>.v<major>.json` (`framework.v1.json`).

### Names should describe what, not how

- `get_current_user_email` is better than `get_email_from_session_via_db_lookup`.
- `compute_capability_intersection` is better than `loop_over_capability_arrays_and_filter`.
- The "how" lives in the function body; the name lives at the call site.

### Function design

- Functions do one thing. If a function name has "and" in it, split it.
- Functions should be ≤50 lines. Beyond that, decompose.
- Functions should have ≤3 parameters. Beyond that, introduce a struct.
- Pure functions are preferred over functions with side effects. Effects (file I/O, network, time) live in well-named functions at the edge of the call graph.

### Errors

- **Rust:** `Result<T, E>` everywhere it can fail. Use `thiserror` for library error types; `anyhow` for application error types at the boundary; `?` for propagation. No `panic!` in library code; `panic!` is for "this is impossible and represents a bug." Use `unwrap_or` / `unwrap_or_else` / `expect("...")` with a real error message when needed.
- **TypeScript:** throw `Error` subclasses for exceptional conditions; return discriminated unions (`{ ok: true; value: T } | { ok: false; error: E }`) for expected-failure paths in domain logic.
- Capture root cause, not just symptoms. Error messages should let a user fix the issue without reading the source.

### Anti-patterns (project-wide)

- Hidden AI usage. Disclose AI assistance in commits and PRs.
- Magic numbers. Name them with constants.
- Stringly-typed APIs in Rust. Use enums.
- `any` in TypeScript. Use `unknown` and narrow.
- `#[allow(clippy::...)]` without an issue link or comment explaining why.
- `// @ts-ignore` / `// @ts-expect-error` without a comment + issue link.
- Tests that depend on implementation details (private fields, internal call counts) instead of observable behavior.
- Functions named `helper`, `util`, `do_thing`, `process`. Be specific.
- Catching errors and silently dropping them.
- Adding dependencies for one-line utilities you could write in 3 lines (e.g., adding `is-odd` to npm).
- Premature abstraction. Three similar lines is better than a wrong abstraction. Wait for the fourth before extracting.

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

The spec is large and the project covers a lot of ground. These are traps that have already bitten the work or are predictable based on the design:

1. **Tool ≠ Skill ≠ Agent.** Three distinct concepts (§0b). Tools are called. Skills are loaded into context. Agents are spawned. Don't conflate them; the schemas, file formats, and runtime mechanics are different.
2. **Capability narrowing on Agent→Agent edges.** A child agent's `allowed_tools` and `allowed_skills` cannot exceed the parent's. The Builder Canvas (Phase 9) enforces this automatically; manual JSON editing must respect it.
3. **v0.1 ships STANDARD mode hardcoded.** No mode router (§3b). The framework JSON's `modes` field still exists in schema but is not evaluated at runtime in v0.1.
4. **v0.1 ships `fresh_context_per_task` only.** The continuous loop policy (Ralph-style) is in the schema but not implemented. `examples/ralph/framework.json` exists but won't run on v0.1.
5. **v0.1 ships Novice + Promoted tiers only.** No Operator tier. Promoted is blocked from auto-accepting `shell: true` and `network: ["*"]` artifacts even though the tier's general behavior is auto-accept-when-validated.
6. **v0.1 is single-session.** §1c Multi-session is v1.0. Do not write multi-session code paths in v0.1; they create surface area without benefit.
7. **v0.1 is Windows-only.** Not because Tauri is Windows-only — Tauri is cross-platform — but because we test only on Windows in v0.1, and macOS/Linux ports come at v1.0. CI still runs on all three OSes to catch drift early.
8. **No telemetry, ever.** No analytics, no crash reporter, no "anonymous metrics," no phone-home. Per §13 of spec. Adding any requires an ADR with public dashboard plan + opt-in mechanism.
9. **Anthropic API is hit directly.** No `@anthropic-ai/sdk` dep, no `anthropic-rs`. `reqwest` + `eventsource-stream` only. The API surface is small and stable; direct HTTP keeps the dependency surface minimal.
10. **Tauri allowlist is the security boundary.** The renderer has no Node API. Anything the renderer needs from Rust goes through a typed `#[tauri::command]`. Don't widen the allowlist without considering capability implications.
11. **Drone ≠ Main ≠ Sandbox.** Three Rust processes. Drone owns SQLite + snapshots + recovery (per session). Main owns the agent loop, MCP, providers, framework loader, capability enforcer. Sandbox is per-artifact, OS-isolated, used for L3 validation. Don't blur these.
12. **IPC is two layers.** Renderer↔Main is Tauri typed IPC. Main↔Drone is framed JSON over Unix socket / named pipe. Different mechanisms with different semantics. Don't try to use Tauri IPC for drone communication.
13. **SQLite WAL pragmas matter.** `PRAGMA journal_mode = WAL`, `PRAGMA synchronous = NORMAL`, `PRAGMA busy_timeout = 5000`, `PRAGMA foreign_keys = ON`. Set them in this order at every connection open. Missing busy_timeout causes flaky tests under contention.
14. **Snapshots are append-only.** Drone never updates a snapshot row; new snapshot = new row. State_hash deduplication happens at read time, not write time.
15. **Resume rebuilds history, doesn't re-execute.** Tool calls in flight at crash time are flagged `tool_call_uncertain` and surfaced to the user. Don't replay tool calls automatically.
16. **`request_capability` is a meta-tool.** It's auto-injected into every agent's tool list. When the model calls it, the runtime translates to `tool_missing` or `skill_missing` and routes through gap flow. Agents can decline `skill_load_requested` events but typically comply.
17. **Mode-variant skills filter sections.** A `skill.md` with `mode_variants: { LITE: { include_sections: ["quick"] }, ... }` has its body filtered by section header at load time. The full markdown is on disk; the model sees only the slice for the active mode.
18. **JSONLogic for triggers.** Programmatic skill triggers use a JSONLogic-style expression language. Operators allowed in v0.1: `var`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `and`, `or`, `not`, `in`, `+`, `-`, `*`, `/`. Adding operators requires extending the evaluator; do not silently extend.
19. **Capability declarations are mandatory for generated artifacts.** Hand-authored artifacts can omit the `capabilities` block, but they default to Operator-tier-only loading. Generated artifacts (Phase 8) must declare capabilities; the validator rejects missing blocks.
20. **DCO sign-off is mandatory.** `git commit -s`. Without it, the commit is rejected by the CI hook (once configured at M1+).

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
| Architecture decisions | `docs/adr/` |
| Schemas (source of truth) | `schemas/*.v1.json` |
| Reference framework (archetype proof) | `examples/aria/` |
| Sibling framework (Ralph) | `examples/ralph/` |
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

**Every milestone session produces a retrospective.** Claude maintains it during the session and surfaces it alongside the PR. The user reviews the retrospective the same way they review the code diff. The user does **not** fill in retrospective fields.

This protocol exists because Claude has the live context (friction events, ambiguities, self-correction iterations); the user only sees the final PR. Asking the user to score what they didn't observe is asking them to reconstruct context they never had. Claude self-assessing is more honest about who has the information.

### Per-stage deliverable

For every milestone-stage session, Claude creates a retrospective file at:

`docs/build-prompts/retrospectives/M[NN].<X>-retrospective.md`

where `<X>` is `A`, `B`, etc. (or `M[NN]-retrospective.md` for parent milestones small enough to fit one prompt under the 250-line / 12-hour rule in `TEMPLATE.md`).

Copied from `docs/build-prompts/retrospectives/RETROSPECTIVE-TEMPLATE.md`.

**Per-milestone-as-PR pattern:** stages are commits on one feature branch (`claude/m[nn]-<title>`); the M[NN] PR drafts only at the end of the final stage. Each stage commit lands only after user approval. The PR opens with all stage commits + all stage retrospectives + the parent-milestone summary.

### Workflow within a stage session

1. **Read prior stage retrospectives BEFORE writing code (mandatory for stages B onward).** The first action in any non-first stage is to read every prior stage's retrospective in this milestone. Focus on the `[END] Decisions for the next stage` section and any `[LIVE]` friction events flagged as carrying forward. **Apply those decisions** — that's what they exist for. Also re-read the most recent `docs/gap-analysis.md` entry's Carry-forward section for items targeting this stage (e.g., a Pre-M01 anticipated-friction note about `typify oneOf` should be acted on at Stage B start, not re-discovered the hard way). For Stage A of M01: skip — no prior retrospective exists yet. The CLI prompt for each stage (X.5) embeds this read step explicitly; this protocol rule is the backstop in case a future milestone prompt forgets it.
2. **At session start (after the prior-retrospective read)**, immediately after stating the deliverable + test plan, copy `RETROSPECTIVE-TEMPLATE.md` to the per-stage path. Set the header (parent-milestone, stage letter, branch, starting commit, estimated effort).
3. **During the session**, fill in the live observation log AS friction surfaces. Don't summarize at the end — details fade. Specifically:
   - Add a row to the friction-events table the moment a friction event occurs.
   - Add a row to the ambiguity-events table when contradictions or unclear guidance is encountered.
   - Add a row to the surface-events table whenever a decision is surfaced to the user.
   - Add a row to the protocol-drift table if you almost broke a Hard Rule (§4) — and alert the user immediately, not at session end.
   - Add a row to the surprise-events table when something unexpected (good or bad) happens.
4. **At stage end** (when all stage acceptance criteria pass and gates are green):
   - Score the three-axis retrospective (1–5 per row per `PROCESS-VALIDATION.md`)
   - Evaluate threshold gates (5 hard + 5 soft)
   - Mark the outcome (Sound / Sound-but-rough / Friction-heavy / Not-ready)
   - Fill in the Decisions section with specific updates for the next stage (or next parent milestone if final stage). **Be specific** — these decisions get *read* by the next stage's session, so generic notes ("be careful with X") waste the channel; cite file:line, name the exact change to apply, name the gate to re-run.
5. **Surface to the user.** For non-final stages: surface the diff stat + gate results + retrospective + draft commit message. For the **final stage** of the milestone: also draft the M[NN] PR description and create `M[NN]-summary.md` aggregating across stages.
6. **State explicitly:** *"Stage `<X>` is ready. I will not commit until you approve. Please review the retrospective and the diff."* For the final stage: *"M[NN] is ready. I will not commit Stage `<X>`, push, or open the PR until you approve. Please review the retrospective, the M[NN] summary, and the PR description."*
7. **On approval**, the retrospective is committed alongside the stage's code on the parent-milestone feature branch. Push waits for the final stage; PR opens only on the final stage's approval.

### What the user reviews

Two artifacts:

1. **The PR code diff** — does the milestone deliver?
2. **The filled-in retrospective** — does Claude's self-assessment match observable evidence?

User especially validates **Hard Gate G1** (do-not-commit-until-approved): if the retrospective claims it passed but the git log shows a commit before the user said "approved," that's a flag. User pushes back; Claude revises.

### Honest self-assessment is mandatory

The retrospective must reflect what actually happened, not a sanitized version. If Claude self-corrected through 4 rounds when 3 was the budget, that goes in the retrospective. If a friction event was severity 4, score it 4 — don't downgrade to make the totals look better.

A retrospective that claims everything was 5/5 with no friction events is itself a flag. Real sessions have friction.

### Per-parent-milestone summary

After the **final stage** of a parent milestone (e.g., after M01 Stage D for M01), Claude creates `docs/build-prompts/retrospectives/M[NN]-summary.md` from `docs/build-prompts/retrospectives/SUMMARY-TEMPLATE.md`. This aggregates findings across the milestone's stage retrospectives and is part of the M[NN] PR alongside all stage commits and stage retrospectives. The summary verdict gates whether the next parent milestone can start.

If a parent milestone is not staged (small enough per the `TEMPLATE.md` scope-split rule), this summary file isn't needed — the single retrospective IS the summary.

### Outcome routing

Per the outcome marked in the retrospective:

- **Sound** — proceed to next stage in a fresh session (or next parent milestone if this was the final stage). Apply `CLAUDE.md` / `TEMPLATE.md` updates from the Decisions section in a follow-up commit if substantive.
- **Sound but rough** — spend a brief session updating `CLAUDE.md` / `TEMPLATE.md` per Decisions, THEN proceed.
- **Friction-heavy** — stop. Iterate on protocol before next stage.
- **Not ready** — a hard gate failed. Diagnose. Recovery session may be needed. May require an ADR if a primitive protocol change is needed.

### Cross-milestone trends

A `docs/build-prompts/retrospectives/TRENDS.md` is optional and grows over time. When patterns emerge across multiple parent milestones (e.g., "Claude consistently asks about X" or "the time-box estimate is always 1.5× off for Rust-heavy milestones"), Claude updates TRENDS.md. This is the project's quality history; future contributors read it to understand how the build pattern evolved.

### Why this is enforced as a project-wide protocol

- **Catches friction early.** A per-stage retrospective surfaces protocol problems after ~5–8 hours of work, not after 30+ hours of compounding error.
- **Honest about who has context.** Claude logs what Claude saw; user reviews what user can verify.
- **Builds the project's quality history.** After M11, the chain of retrospectives is part of the project's documentation. Someone reading the project a year from now sees how it actually got built — friction included.
- **Aligns with CLAUDE-Code's actual capabilities.** Claude can fill in tables in real time as work happens; the user can't. Use the right tool for the job.

See `docs/build-prompts/PROCESS-VALIDATION.md` for the framework reference (axes, scoring rubric, threshold gates) and `docs/build-prompts/retrospectives/` for the templates and accumulated retrospectives.

---

## 20. Gap Analysis Protocol (append-only, per-milestone)

**Every parent milestone produces a Gap Analysis entry**, separate from the per-stage retrospectives. The retrospectives evaluate the build *process* (did the prompt-driven workflow work?). The gap analysis evaluates the build *product*: does the code match the spec, what did the spec get wrong, what's the prioritized fix backlog?

The single source for this is `docs/gap-analysis.md`. It is **append-only**.

### The append-only rule (Hard Rule)

Per §10 Don't-touch zones, **no prior entry in `docs/gap-analysis.md` may be edited, reordered, or deleted.** This is non-negotiable. The file is the project's audit trail of how the codebase and spec evolved together. Editing prior entries would erase that history and turn the file into another cleaned-up doc.

If a prior milestone's finding is later resolved or invalidated:
- Do NOT modify the original entry.
- The current milestone's "Carry-forward" section states the resolution by referencing the prior entry's milestone tag.
- Example: an M01 critical fix that lands during M02 work gets a line in M02's Carry-forward section: `M01 critical "X" — resolved at <crate/file.rs:line>`. M01's entry stays as it was at M01 PR time.

CI enforces this with a diff check (added in M01 Stage E's edit to `.github/workflows/ci.yml`, alongside the first entry): if any line in `docs/gap-analysis.md` that existed on the PR base branch is missing or changed in HEAD, CI fails. New milestones may only add content at the bottom.

### When the gap analysis runs

After the **final stage** of a parent milestone commits (e.g., after M01 Stage D), and after the per-parent-milestone summary lands, but **before** the milestone PR is opened:

1. Claude runs the **Phase Closeout — Gap Analysis** step (Stage E in M01; documented in each milestone's prompt and in `TEMPLATE.md`).
2. Output: a new entry appended to `docs/gap-analysis.md` per the entry template in that file.
3. Claude surfaces the new entry alongside the milestone PR description and the parent-milestone summary.
4. The gap analysis commit is the **final commit on the parent-milestone branch**. The PR pushes only after this commit is approved by the user. This means the gap analysis gates the PR.

### What the entry covers

Every entry has six sections (template lives in `docs/gap-analysis.md`):

1. **Codebase deep dive** — narrative review of the *cumulative* code shipped to date, not just this milestone. 200–500 words. Surface structural concerns that will compound.
2. **Adherence to spec** — for each area touched, classify ✅ / ⚠️ / ❌ with file:line citations on both spec and code sides.
3. **Spec review (forward-looking)** — missing items, contradictions, ambiguity, open questions, recommended spec changes. Cumulative; re-read prior sections with fresh eyes.
4. **Fix backlog** — code AND spec fixes, prioritized 🔴 Critical / 🟡 Important / 🟢 Nice-to-have. Severity is non-elastic — if everything is "important," the prioritization is meaningless.
5. **Carry-forward from prior milestones** — status of every unresolved fix-backlog item from prior entries. Resolved / still open / deferred to <milestone>. **Never modifies prior entries.**
6. **Sign-off** — Claude attestation + timestamp.

Sections with nothing to report write **"None observed."** — never omit a section.

### What the entry is NOT

- Not a per-stage retrospective. Stage-level process feedback lives in `docs/build-prompts/retrospectives/M[NN].<X>-retrospective.md`.
- Not a parent-milestone process summary. That lives in `docs/build-prompts/retrospectives/M[NN]-summary.md` and aggregates the stage retrospectives.
- Not the changelog. `CHANGELOG.md` lists what shipped; gap analysis evaluates how it relates to the spec and what's still wrong.

### User review and approval

User reviews three artifacts together at PR time:

1. The PR code diff — does the milestone deliver?
2. The filled-in retrospectives + parent-milestone summary — was the process sound?
3. The new gap analysis entry — is the cumulative product↔spec assessment honest, and is the fix backlog prioritized correctly?

If user pushes back on any of the three, Claude revises before the PR opens. **The gap analysis cannot be edited after the milestone PR merges** — only future milestones' Carry-forward sections can update its status.

### Why this is a Hard Rule

- **Audit trail integrity.** The chain of M01 → M02 → ... → M11 entries is the project's record of how product↔spec drifted and converged. Editing prior entries breaks that chain.
- **Honest assessment.** Knowing the entry is permanent forces Claude to assess accurately the first time, not optimistically with the option to "fix it later."
- **Forces forward-looking carry-forward.** Resolution of a prior finding has to be stated in the *new* milestone's entry, which puts the resolution alongside its date and commit context.
- **Spec drift detection.** When M07's gap analysis says "spec §3a contradicts §6a" and M03's gap analysis already said "spec §3a is ambiguous about <X>," the through-line is visible and actionable. Editing prior entries hides through-lines.

See `docs/gap-analysis.md` for the entry template and the (initially empty) milestone log.
