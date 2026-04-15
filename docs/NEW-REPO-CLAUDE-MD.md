# CLAUDE.md Template for the New Product Repo

> **How to use this file:** When the new product repo is created
> (GitHub: `username/loom` or whatever the name becomes), copy the
> contents below into `CLAUDE.md` at the root of that new repo. This
> is the standing orders file every future Claude Code session will
> read on start. It is NOT the `CLAUDE.md` at the root of
> `AgentFramework/` — that one is ARIA-specific and stays.

---

--- CLAUDE.md START ---

# Loom — Development Standing Orders

> A desktop workbench for agentic-literate non-coders. Read the design
> docs before writing code.

## What this project is

Loom is an Electron desktop app where non-coders assemble, run, and
share agentic workflows. Target user: someone who understands agents
and skills conceptually but does not write code. v0.1 ships Quick
Agent (ReAct strategy) end-to-end on Windows with plain-English
narration, budget caps, and a first-run experience.

Full context: `docs/DESIGN-DECISIONS.md`.

## Read before writing code

**Every session, every time.** In this order:

1. `docs/DESIGN-DECISIONS.md` — what we're building and why
2. `docs/BUILD-SPEC-v0.1-architecture.md` — the structural spec
3. `docs/BUILD-SPEC-v0.1-milestones.md` — the test-first work plan
4. `docs/OPEN-QUESTIONS.md` — things still undecided
5. This file

If you are continuing a specific milestone, also read:

6. `docs/progress/milestone-<N>.md` — if it exists, the in-progress
   notes from previous sessions working on this milestone

## The cardinal rule: test-first, always

**No production code is written before a failing test exists for it.**

Workflow for every change, without exception:

1. Read the milestone in `BUILD-SPEC-v0.1-milestones.md`
2. Write the unit test(s) first. Run them. Confirm they fail for the
   *right* reason (not because of a typo or missing import).
3. Write the integration test(s). Run them. Confirm they fail for
   the right reason.
4. Write the behavioral test(s). Run them. Confirm they fail.
5. Implement the minimum code needed to make the unit tests pass.
6. Make the integration tests pass.
7. Make the behavioral tests pass.
8. Run `npm run verify`. All gates must pass.
9. Commit with the message from the milestone spec.

**Proof of life is not acceptance.** If a test is
`expect(fn()).not.toThrow()`, delete it and write a real one. The
test must assert the *specific, observable* outcome the code is
supposed to produce.

## The verification gate

Every milestone, every commit, every time:

```bash
npm run verify
```

This runs, in order:

1. `npm run lint` — ESLint with zero warnings
2. `npm run typecheck` — `tsc --noEmit` with zero errors
3. `npm run test:unit` — Vitest unit suite with coverage gate
4. `npm run test:integration` — Vitest integration suite
5. `npm run test:behavioral` — Vitest behavioral suite
6. `npm run test:e2e` — Playwright Electron suite (if applicable
   to this milestone)

If any step fails, you **do not proceed.** You fix it first. No
commits on red.

### Exception: intentional red state during TDD

It is correct and expected for tests to fail during the red phase of
TDD. Commits made during red should be clearly labeled and rare. The
standard flow is: red → green → commit. Not: red → commit → green.

## Handling ambiguity

If the spec is unclear or a choice exists:

1. **First:** web search for current best practice. Document what you
   found.
2. **Second:** if the web doesn't resolve it, ask the user. Frame the
   question with your recommended default so they can approve quickly.
3. **Never:** silently guess. Every invented decision is a future bug.

Examples of things you should web-search before committing to, every
time you encounter them:

- Current version of any dependency before adding it
- Current best practice for an Electron security setting
- Current MCP protocol version when touching MCP code
- Current Anthropic SDK streaming pattern when adding a new provider
  call
- Any deprecation warning you see in the terminal

## What NOT to touch without explicit approval

The following areas are protected. Modifying them requires the user
saying "go ahead with X":

- **Drone Core (`src/main/drone/core/**`)** — the survival layer.
  Breaking this loses user work. Any change needs approval.
- **Process Supervisor (`src/main/drone/supervisor/**`)** — same
  reasoning.
- **Budget enforcement (`src/main/runtime/budget.ts`)** — breaking this
  means users get surprise bills. Approval required.
- **Cost math (`src/main/runtime/cost.ts`)** — same reasoning.
- **`PolicyEnforcer` in `RuntimeContext`** — the security boundary.
  Approval required.
- **API key handling (`src/main/secrets/**`)** — touches keytar /
  OS keychain. Approval required.
- **DB migrations** — data loss risk. Approval required.
- **CI configuration** — breaks shared signal. Approval required.

Adding tests to any of these is always allowed without approval.

## Commits

- Small, per-milestone or per-test cycle
- Conventional Commits: `feat(scope):`, `fix(scope):`, `test(scope):`,
  `docs(scope):`, `refactor(scope):`, `chore(scope):`
- Commit message body explains the *why*, not the *what* (the diff
  shows the what)
- Never `--amend` a commit that's been pushed
- Never `--no-verify` without explicit user instruction
- Never force-push `main`

## Branches

- `main` is always in a state where `npm run verify` passes
- Feature work happens on `feature/milestone-<N>-<slug>` branches
- Branch from `main`, PR back to `main`, merge after review (or
  solo-merge if user has approved the flow)
- Delete merged branches

## Linting and typing

Enforced, non-negotiable:

- ESLint with `@typescript-eslint/recommended-type-checked`
- TypeScript `strict: true`, `noUncheckedIndexedAccess: true`,
  `noImplicitOverride: true`, `exactOptionalPropertyTypes: true`
- Zero `any` in committed code (use `unknown` + narrowing)
- Zero `// @ts-ignore` (use `// @ts-expect-error` with a reason comment)
- Zero ESLint warnings in committed code
- Zero `console.log` in committed production code (use the logger)
- Zero skipped tests (`it.skip`, `test.skip`) in committed code unless
  paired with a GitHub issue link explaining why

## Testing philosophy

Three tiers, all required:

1. **Unit tests** — fast (<10ms each), isolated, mocked deps. One
   function or class under test. Cover happy path, edge cases, and
   every error branch.
2. **Integration tests** — multiple modules together, real SQLite
   (in-memory or temp file), mocked LLM. Exercise actual data flow.
3. **Behavioral tests** — user-visible outcomes. When X happens,
   Y is observably true in the DB, the UI, the event stream, or
   the rendered output. These are the tests the user cares about.

**A milestone is not complete until all three tiers pass on the
specific functionality it introduces.** Unit tests alone are never
sufficient proof.

### Tests that are unacceptable

These patterns are wrong and should be deleted or rewritten if
found in the codebase:

- `expect(fn()).not.toThrow()` — proves the happy path doesn't blow
  up; asserts nothing meaningful
- `expect(result).toBeDefined()` / `toBeTruthy()` / `not.toBeNull()`
  as the sole assertion — proves only that something exists
- `expect(mock).toHaveBeenCalled()` without checking *what it was
  called with* — proves the code path is reached, not that it's
  correct
- Snapshot tests of arbitrarily large objects — they rot and get
  rubber-stamped; only snapshot small, stable, human-inspectable
  things like narration strings
- Tests that require network access to a live API — flaky, slow,
  and expose secrets; mock the API

### Tests that are acceptable

- Assert exact values: `expect(result).toEqual({status: 'ok'})`
- Assert exact shapes with precision: `expect(event).toMatchObject({
    type: 'agent_spawned', parent_id: null })`
- Assert sequence and timing: *"events arrive in this order, within
  this window, with no duplicates"*
- Assert side effects: *"after calling this, the DB has a row with
  these specific values"*
- Assert error paths: *"when the LLM returns 429, a retry is scheduled
  with exponential backoff, the first retry happens in 1–2 seconds,
  and the cap event fires after 5 failed retries"*
- Assert UI outcomes via Testing Library: *"after clicking 'Run',
  the narration panel shows a sentence starting with 'Quick Agent
  is...'"*
- Behavioral E2E: *"a full run from the user's POV produces the
  expected final state"*

## Failure handling during a session

If a test fails and you don't understand why:

1. Read the error message fully. Don't skim.
2. Run the failing test in isolation (`vitest <file>`).
3. Look at the actual vs. expected diff.
4. Check if it's a test bug or an implementation bug.
5. If implementation bug: fix it, re-run, don't touch the test.
6. If test bug: fix the test, re-run.
7. If you can't figure it out after ~15 minutes: stop, report to the
   user with what you've tried.

**Never:**

- Delete a failing test to make a commit green
- Add `.skip` to avoid a failing test
- Lower a coverage gate to pass
- Comment out assertions
- Use `try/catch` to swallow errors that should propagate
- Silently catch and log an error that breaks a test

## Progress reporting

During multi-hour sessions, report every milestone boundary:

- What milestone was completed
- Tests added per tier (unit / integration / behavioral)
- Coverage delta (e.g., `drone/core: 92% → 95%`)
- Time spent vs budget
- Open questions or blockers
- Next milestone

Use the todo list tool to track progress visibly.

## When updating docs

If you discover reality diverges from `BUILD-SPEC-v0.1-architecture.md`
or `BUILD-SPEC-v0.1-milestones.md`:

1. Stop coding on that milestone
2. Report the divergence
3. Wait for user guidance
4. Update the spec first, then resume coding

Keeping the spec accurate is higher priority than forward progress on
code. A drifted spec is how projects lose coherence.

## Repository layout (v0.1)

```
loom/
├── CLAUDE.md                    # this file
├── README.md                    # user-facing description
├── LICENSE                      # Apache 2.0
├── package.json
├── tsconfig.json
├── vite.config.ts
├── electron-builder.yml
├── .eslintrc.cjs
├── .prettierrc
├── docs/
│   ├── DESIGN-DECISIONS.md
│   ├── BUILD-SPEC-v0.1-architecture.md
│   ├── BUILD-SPEC-v0.1-milestones.md
│   ├── OPEN-QUESTIONS.md
│   └── progress/
│       └── milestone-<N>.md     # optional in-session notes
├── electron/
│   ├── main.ts
│   ├── preload.ts
│   └── drone/
│       ├── core.ts
│       └── supervisor.ts
├── src/
│   ├── main/
│   │   ├── sdk/
│   │   ├── runtime/
│   │   ├── db/
│   │   ├── secrets/
│   │   └── ipc/
│   └── renderer/
│       ├── App.tsx
│       ├── narration/
│       ├── graph/
│       ├── cost/
│       ├── first-run/
│       └── gallery/
├── tests/
│   ├── unit/
│   ├── integration/
│   ├── behavioral/
│   ├── e2e/
│   └── fixtures/
└── scripts/
    ├── verify.sh
    └── package-windows.sh
```

## Commands quick reference

```bash
# Development
npm run dev              # Vite + Electron in dev mode
npm run dev:main         # main process only (for debugging)
npm run dev:renderer     # renderer only (for component work)

# Testing
npm run test             # all tests
npm run test:unit        # unit suite with coverage
npm run test:integration # integration suite
npm run test:behavioral  # behavioral suite
npm run test:e2e         # Playwright Electron suite
npm run test:watch       # watch mode for TDD

# Quality
npm run lint             # ESLint
npm run lint:fix         # auto-fix what's auto-fixable
npm run typecheck        # tsc --noEmit
npm run format           # Prettier

# The Gate
npm run verify           # lint + typecheck + all tests — REQUIRED BEFORE COMMIT

# Build
npm run build            # production build
npm run package          # electron-builder Windows installer
```

## Known gotchas

- `better-sqlite3` requires native rebuild per Electron version — run
  `npm run rebuild` if tests fail with "invalid ELF header" or
  similar on startup
- Windows Defender may flag unsigned Electron apps — if the installer
  fails SmartScreen, check `docs/BUILD-SPEC-v0.1-architecture.md`
  Section 11 (Security & Secrets) for the signing workflow
- Playwright Electron tests are slow — run them explicitly with
  `npm run test:e2e`, not as part of the watch loop
- `keytar` has historically had maintenance issues — if it's broken,
  fall back to the alternative identified in the architecture spec

## The bottom line

Every session: read the docs, test first, verify, commit small. No
proof-of-life tests. No silent guesses. No scope creep. No commits on
red. Ask when unsure.

--- CLAUDE.md END ---

## Notes for the user (not part of the CLAUDE.md content)

When you create the new repo:

1. Copy everything between `--- CLAUDE.md START ---` and
   `--- CLAUDE.md END ---` into the new repo's `CLAUDE.md`
2. Customize the product name if not "Loom"
3. Delete any section that doesn't apply yet (e.g., the repo layout
   section can be shortened until the layout stabilizes)
4. Commit as the first commit on the new repo, along with an empty
   `docs/` folder structure

This file is the foundation every future Claude Code session will
rely on to stay disciplined. Keep it tight, keep it honest, update
it when reality changes.
