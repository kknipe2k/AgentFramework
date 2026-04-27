---
name: git_rollback
version: 1.0.0
description: Restore a previously-created git checkpoint, discarding any current working-tree changes.

input_schema:
  type: object
  properties:
    checkpoint_id:
      type: string
      description: "checkpoint_id returned by git_checkpoint"
    discard_current:
      type: boolean
      default: false
      description: "If true, discard current uncommitted changes before restoring; if false, fail when conflicts exist"
  required: ["checkpoint_id"]

output_schema:
  type: object
  properties:
    restored:    { type: boolean }
    files_count: { type: number, description: "Files restored from checkpoint" }
    discarded:   { type: array, items: { type: string }, description: "Paths whose current changes were discarded" }
    conflicts:   { type: array, items: { type: string }, description: "Paths with unresolved conflicts (rollback aborted)" }

mcp_binding: null

shell_binding:
  command: |
    if [ "${input.discard_current}" = "true" ]; then
      git checkout -- . && git clean -fd
    fi && \
    git stash apply ${input.checkpoint_id} --index
  cwd:     "${session.workspace_root}"
  timeout_ms: 30000
  parse_output:
    - { match: "^CONFLICT.*: (.+)$",  capture_to: "conflicts", flags: "gm" }
    - { match: "^Removing (.+)$",      capture_to: "discarded", flags: "gm" }

capabilities:
  tools_called:    []
  skills_loaded:   []
  file_access:
    read:  [".git/**", "**/*"]
    write: [".git/**", "**/*"]
  network:         []
  shell:           true
  spawn_agents:    []

provenance:
  generator:    "hand-authored"
  source:       "wrapper around git stash apply + .aria/git-ops.sh rollback"
  authored_at:  "2026-04-18T00:00:00Z"
  content_hash: "sha256:placeholder-replace-on-first-load"

tags: ["git", "checkpoint", "rollback", "destructive"]
---

## Description

Restores a checkpoint created by `git_checkpoint`. Destructive when `discard_current: true` — any uncommitted current work is lost.

This tool is automatically invoked by the runtime when a `post_task` hook with `on_failure: rollback` fails. Manual invocation should be rare and always gated by HITL.

Because of `shell: true` AND broad `file_access.write`, this tool requires Operator tier to invoke autonomously. Promoted tier requires HITL approval for each call. Novice tier blocks autonomous invocation entirely.

## Implementation

1. If `discard_current: true`, run `git checkout -- . && git clean -fd` to clear the working tree.
2. Apply the stash referenced by `checkpoint_id` with `--index` to preserve staged-vs-unstaged distinction.
3. Parse output for conflicts; if any, abort and report.

## Examples

Successful rollback after task failure:

```yaml
input:  { checkpoint_id: "a1b2c3d4...", discard_current: true }
output:
  restored: true
  files_count: 12
  discarded: ["src/buggy_change.ts", "tests/incomplete.test.ts"]
  conflicts: []
```

Conflict (rollback aborted):

```yaml
input:  { checkpoint_id: "e5f6...", discard_current: false }
output:
  restored: false
  files_count: 0
  discarded: []
  conflicts: ["src/utils.ts", "tests/utils.test.ts"]
```

## HITL integration

When triggered by a hook with `on_failure: rollback`, the runtime surfaces a HITL prompt before executing:

> Verify failed for task `{task.title}`. Roll back working-tree changes to checkpoint `{checkpoint.name}`?
> [y]es / [n]o (keep changes, mark task failed) / [s]ee diff first

## Capability enforcement

- `shell: true` — declared because `git` invocation is a shell command.
- `file_access.write: ["**/*"]` — broad because rollback can touch any file. This is the broadest write surface in `examples/aria/`; that's why the tool is gated behind tier + HITL.

## Pairs with

- `git_checkpoint` — creates the checkpoints this tool restores.
- Drone snapshot revert (`revert_to_snapshot`) — drone snapshots cover runtime state in SQLite; git rollback covers working-tree files. Both fire together when a post_task hook fails with `on_failure: rollback`.
