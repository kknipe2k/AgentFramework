# Style and Naming

Project-wide style and naming conventions. CLAUDE.md §9 references this file as the source. Changes here apply to all Rust and TypeScript code in the runtime.

## Comments and prose in code

- **No comments by default.** Code expresses what; comments explain *why*.
- A comment is justified when:
  - There's a hidden constraint (e.g., "must run before X because Y").
  - There's a subtle invariant (e.g., "this map can be empty briefly during Z; downstream handles that").
  - There's a workaround for a specific bug (link the bug).
  - The behavior would surprise a reader on first read.
- A comment is **not** justified when it just restates what the code does. Don't write `// Increment counter` above `counter += 1`.
- **No marketing language.** Code comments and commit messages don't have "🚀", "blazing fast", "revolutionary." Plain technical prose only.
- **No "TODO: optimize later" without a linked issue.** Either open the issue and reference it (`// TODO: #N — improve hot path`) or don't write the TODO.

## Naming

- **Rust:** `snake_case` for functions/vars; `CamelCase` for types; `SCREAMING_SNAKE_CASE` for constants. Module names are short, lowercase. Crate names are kebab-case (`runtime-core`). File names mirror module names.
- **TypeScript:** `camelCase` for vars/functions; `PascalCase` for types/components; `SCREAMING_SNAKE_CASE` for constants. File names: `PascalCase.tsx` for React components, `camelCase.ts` for utility modules.
- **Skill / tool / agent .md files:** kebab-case (`code-simplifier.md`, `git-checkpoint.md`).
- **Schema files:** `<name>.v<major>.json` (`framework.v1.json`).

## Names should describe what, not how

- `get_current_user_email` is better than `get_email_from_session_via_db_lookup`.
- `compute_capability_intersection` is better than `loop_over_capability_arrays_and_filter`.
- The "how" lives in the function body; the name lives at the call site.

## Function design

- Functions do one thing. If a function name has "and" in it, split it.
- Functions should be ≤50 lines. Beyond that, decompose.
- Functions should have ≤3 parameters. Beyond that, introduce a struct.
- Pure functions are preferred over functions with side effects. Effects (file I/O, network, time) live in well-named functions at the edge of the call graph.

## Errors

- **Rust:** `Result<T, E>` everywhere it can fail. Use `thiserror` for library error types; `anyhow` for application error types at the boundary; `?` for propagation. No `panic!` in library code; `panic!` is for "this is impossible and represents a bug." Use `unwrap_or` / `unwrap_or_else` / `expect("...")` with a real error message when needed.
- **TypeScript:** throw `Error` subclasses for exceptional conditions; return discriminated unions (`{ ok: true; value: T } | { ok: false; error: E }`) for expected-failure paths in domain logic.
- Capture root cause, not just symptoms. Error messages should let a user fix the issue without reading the source.

## Anti-patterns (project-wide)

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
