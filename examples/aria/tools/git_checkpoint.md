---
name: git_checkpoint
version: 1.0.0
description: Create a named checkpoint of the current working tree (git stash + tag) that can be restored later via git_rollback.

input_schema:
  type: object
  properties:
    name:
      type: string
      description: "Human-readable checkpoint name (e.g., 'pre-refactor', 'before-task-3')"
    include_untracked:
      type: boolean
      default: true
  required: ["name"]

output_schema:
  type: object
  properties:
    checkpoint_id:   { type: string, description: "Stable handle for git_rollback" }
    name:            { type: string }
    git_stash_ref:   { type: string }
    files_count:     { type: number }
    created_at:      { type: string }

mcp_binding: null

shell_binding:
  command: |
    git stash push ${input.include_untracked == true ? "-u" : ""} \
      -m "aria-checkpoint:${input.name}:$(date +%s)" && \
    git stash list -1 --format=%H
  cwd:     "${session.workspace_root}"
  timeout_ms: 30000
  parse_output:
    - { match: "^([a-f0-9]+)$", capture_to: "git_stash_ref" }

capabilities:
  tools_called:    []
  skills_loaded:   []
  file_access:
    read:  [".git/**", "**/*"]
    write: [".git/**"]
  network:         []
  shell:           true
  spawn_agents:    []

provenance:
  generator:    "hand-authored"
  source:       "wrapper around git stash + .aria/git-ops.sh checkpoint logic"
  authored_at:  "2026-04-18T00:00:00Z"
  content_hash: "sha256:placeholder-replace-on-first-load"

tags: ["git", "checkpoint", "rollback"]
---

## Description

Saves a snapshot of the working tree as a named checkpoint. Useful before risky operations (refactors, automated edits) so the runtime can roll back if verify fails.

The runtime auto-fires `git_checkpoint` at task boundaries when `task_defaults.post_hooks` includes a hook with `on_failure: rollback` — so manual invocation is rare. This tool exists so frameworks and agents can checkpoint at non-task boundaries (e.g., before a multi-task refactor begins).

## Implementation

A thin wrapper around `git stash push` with a structured naming convention:

```
aria-checkpoint:<name>:<unix-timestamp>
```

The timestamp suffix ensures uniqueness even if the same `name` is reused. The stash ref (commit hash) is the stable `checkpoint_id` returned to the caller.

## Examples

```yaml
input: { name: "pre-refactor", include_untracked: true }
output:
  checkpoint_id: "a1b2c3d4..."
  name: "pre-refactor"
  git_stash_ref: "a1b2c3d4..."
  files_count: 12
  created_at: "2026-04-18T15:00:00Z"
```

## Error handling

- Working tree clean (nothing to stash) → returns `checkpoint_id: null`, `files_count: 0`. No error. Caller can treat as "nothing to roll back to" if needed.
- Git not installed → tool fails L3 validation at framework load.
- Stash conflict on save (rare) → emit `tool_error` with git's error message; runtime surfaces via `tool_missing` flow.

## Pairs with

- `git_rollback` — restores a checkpoint by `checkpoint_id`.
- Drone snapshot system — checkpoints in git complement drone snapshots in SQLite. Drone snapshots cover runtime state; git checkpoints cover working-tree state.
