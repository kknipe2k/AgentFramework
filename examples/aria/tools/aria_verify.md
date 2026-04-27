---
name: aria_verify
version: 1.0.0
description: Run the ARIA verification pipeline at the specified level. Wraps `bash .aria/verify.sh`.

input_schema:
  type: object
  properties:
    level:
      type: string
      enum: ["quick", "standard", "full"]
      default: "standard"
      description: Verification depth — quick (types+lint), standard (+tests+build), full (+integration+e2e)
  required: []

output_schema:
  type: object
  properties:
    passed:        { type: boolean }
    level:         { type: string }
    duration_ms:   { type: number }
    failures:      { type: array, items: { type: string } }
    warnings:      { type: array, items: { type: string } }
    output_tail:   { type: string, description: "Last 2000 chars of verifier output" }

mcp_binding: null

shell_binding:
  command: "bash .aria/verify.sh ${input.level}"
  cwd:     "${session.workspace_root}"
  timeout_ms: 900000
  parse_output:
    - { match: "^FAIL: (.+)$",  capture_to: "failures",  flags: "gm" }
    - { match: "^WARN: (.+)$",  capture_to: "warnings",  flags: "gm" }
    - { match: "passed=(true|false)", capture_to: "passed", transform: "boolean" }

capabilities:
  tools_called:    []
  skills_loaded:   []
  file_access:
    read:  ["**/*"]
    write: [".aria/state/**"]   # verify.sh writes to its own state
  network:         []
  shell:           true          # this tool wraps a shell command
  spawn_agents:    []

provenance:
  generator:    "hand-authored"
  source:       "wrapper around .aria/verify.sh"
  authored_at:  "2026-04-18T00:00:00Z"
  content_hash: "sha256:placeholder-replace-on-first-load"

tags: ["verify", "ci", "wrapper"]
---

## Description

Wraps the existing shell-ARIA verification pipeline as a runtime tool. Enables `examples/aria/` to satisfy matrix row 3 (`verify.sh after every task`) by referring to this tool from `task_defaults.post_hooks`.

The tool is a thin shell binding — no executable code is shipped here, only a declarative reference to `bash .aria/verify.sh ${level}` with an output parser.

## Implementation

The runtime's shell binding executor:

1. Spawns a sandboxed shell process.
2. Substitutes `${input.level}` from the tool input.
3. Sets `cwd` to the session's workspace root.
4. Captures stdout/stderr; matches lines against `parse_output` regexes.
5. Returns a structured result object matching `output_schema`.

`shell: true` is declared in capabilities; the runtime will refuse to load this tool under Novice tier. Promoted/Operator users see a one-time warning at install.

## Examples

```yaml
input:  { level: "standard" }
output:
  passed: true
  level: standard
  duration_ms: 45123
  failures: []
  warnings: ["Coverage below 80% in src/utils/"]
  output_tail: "...All 245 tests passed.\n"
```

```yaml
input:  { level: "full" }
output:
  passed: false
  level: full
  duration_ms: 312000
  failures: ["E2E test 'login flow' timed out after 60s"]
  warnings: []
  output_tail: "...Playwright: 1 test failed, 12 passed.\n"
```

## Error handling

- Timeout (>900s) → `passed: false`, `failures: ["timeout"]`, runtime emits `hook_failed` with reason: timeout.
- Verify script not found → tool installation error at framework load; framework fails L3 validation.
- Verify output not parseable → `passed: null`, `failures: ["unparseable_output"]`, runtime treats as soft warning.

## Why a wrapper, not a direct shell hook?

Two reasons:

1. **Capability disclosure.** A wrapper tool has explicit `capabilities` declared in frontmatter (Phase 8 §8.security L1). A raw shell hook embedded in `task_defaults.post_hooks` does too, but the wrapper makes the contract reusable across hooks, agents, and slash commands.
2. **Result structuring.** The wrapper parses output into a structured shape the dashboard and VDR can consume. Raw shell output is opaque.

The hook field in framework.json can reference this tool via `{ "type": "tool", "tool_name": "aria_verify" }` instead of `{ "type": "shell", ... }` — same effect, but introspectable.
