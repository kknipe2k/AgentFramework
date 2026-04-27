---
name: discovery
version: 1.0.0
description: Explore an unfamiliar codebase. Identify entry points, conventions, test infrastructure, and "don't touch" zones before any planning.

triggers:
  semantic:
    - "explore"
    - "discover"
    - "what is this codebase"
    - "/discovery"
  programmatic:
    - event: session_start
      when: { "==": [{ "var": "session.workflow" }, "modify"] }

mode_variants:
  LITE:     { include_sections: ["entry_points"] }
  STANDARD: { include_sections: ["entry_points", "conventions"] }
  FULL:     { include_sections: ["entry_points", "conventions", "test_infra", "dont_touch"] }
  FULL+:    { include_sections: ["entry_points", "conventions", "test_infra", "dont_touch", "architecture"] }

required_tools: ["Read", "Glob", "Grep"]
required_skills: []

capabilities:
  tools_called:    ["Read", "Glob", "Grep"]
  skills_loaded:   []
  file_access:     { read: ["**/*"], write: [".aria-runtime/state/discovery-*.md"] }
  network:         []
  shell:           false
  spawn_agents:    []

provenance:
  generator:    "hand-authored"
  source:       ".aria/skills/discovery.md (ported)"
  authored_at:  "2026-04-18T00:00:00Z"
  content_hash: "sha256:placeholder-replace-on-first-load"
---

# Discovery Skill

Map the territory before drawing the route. Skip discovery on a brand-new codebase you'll regret it later.

## entry_points

Find what runs:

- `package.json` → `scripts.start | dev | build | test`
- `Makefile` / `justfile` → top-level targets
- `README.md` → documented setup steps
- `Dockerfile` / `docker-compose.yml` → containerized entry
- `bin/` / `cmd/` / top-level executable scripts

Output: a short list — "the app starts via X; tests run via Y; build pipeline is Z."

## conventions

Read 3–5 representative files in the largest source directory. Extract:

- **File naming.** kebab-case? snake_case? PascalCase? Test files: `*.test.ts`, `*_test.go`, `tests/`, `__tests__/`?
- **Module organization.** Flat? Domain-per-folder? Feature slices?
- **Import style.** Relative paths? Path aliases (`@/foo`)? Default vs named exports?
- **Error handling.** Throw? Return Result/Either? Callbacks? Error type defined?
- **Async style.** async/await? Promises? Streams?
- **Comment density.** Heavy doc comments? Minimal? JSDoc/TSDoc?

Output: a one-paragraph "house style" summary.

## test_infra

Identify:

- Test framework (Jest / Vitest / Mocha / Pytest / Go test / Cargo test)
- Coverage tool and current threshold
- Mocking style (manual mocks, library, stub vs spy)
- Fixture organization
- E2E framework if any (Playwright / Cypress / Selenium)
- CI test command

Output: "Tests run via `<command>`. Mocking with `<library>`. Coverage: `<X%>`. E2E: `<framework or none>`."

## dont_touch

Identify candidates for the framework's `dont_touch` config (or confirm existing entries):

- Lockfiles (`package-lock.json`, `yarn.lock`, `Cargo.lock`)
- Generated code (`.next/`, `dist/`, `build/`, `target/`, `__pycache__/`)
- State directories (`.aria-runtime/state/`, `.aria/state/`)
- Migration files (often append-only by team policy)
- Production config (`config/production.json`, secrets stores)
- Vendored dependencies (`vendor/`, `node_modules/`)

Output: list of paths/globs the framework should add to `dont_touch`. Surface to user for approval before adding.

## architecture

For FULL+ only: produce an architecture map.

- Module dependency graph (top 20 nodes)
- Public API surface (exported symbols across the codebase)
- Data flow for a representative user journey
- Cross-cutting concerns (auth, logging, telemetry, feature flags) and where they're applied

Saved to `.aria-runtime/state/discovery-architecture.md`. Read by the planner before plan creation.

---

## Outputs

- (LITE) Inline summary in the agent's response
- (STANDARD+) `.aria-runtime/state/discovery-{plan_id}.md` with structured findings
- (FULL+) `.aria-runtime/state/discovery-architecture.md`

## Failure modes

- Codebase too large to read meaningfully → focus on the directories the user's request implicates; skip the rest.
- Multi-language repo → run discovery per language; don't try to generalize.
- No tests / no docs / no clear entry → flag this prominently to the user; planning under these conditions is high-risk.
