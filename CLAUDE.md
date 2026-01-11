# ARIA

> VS Code entry point for ARIA orchestration

This file enables ARIA to run inside Claude Code / VS Code using the Task tool for fresh context per task.

---

## Workflow

```
1. Plan    → Create tasks, get HITL approval
2. Execute → Task tool subagent per task (fresh context)
3. Verify  → Run verify-executor.sh after each task
4. Commit  → Git commit if verification passes
5. Repeat  → Next task until done
```

---

## Planning Phase

Before implementing anything:

1. Ask 2-3 clarifying questions (max)
2. Break work into tasks (15-30 min each)
3. Save plan to `.aria/state/current-plan.json`
4. Present to user:

```
Plan: [title]
Tasks: [N] tasks, ~[M] minutes

1. [ ] Task 1 - description
2. [ ] Task 2 - description
3. [ ] Task 3 - description

[a]pprove / [r]evise / [c]ancel?
```

**Wait for approval. Do not proceed without it.**

---

## Execution Phase

For each task in the approved plan:

### 1. Announce
```
═══════════════════════════════════════
Task [N]/[Total]: [title]
═══════════════════════════════════════
```

### 2. Spawn Subagent (Fresh Context)

Use Task tool with `subagent_type="general-purpose"`:

```
Prompt to subagent:
- Task: [description]
- Files to read: [relevant files]
- Output: Implement this task only. Return files changed and status.
```

**Why:** Fresh context per task. No pollution from previous failures.

### 3. Verify

After subagent returns, run:

```bash
bash .aria/verify-executor.sh standard
```

**If PASSES:** Continue to commit
**If FAILS:** STOP. Report failure. Wait for guidance.

### 4. Commit

```bash
git add -A
git commit -m "feat: [task title]"
```

### 5. Update Progress

Update `.aria/state/current-plan.json` - mark task as done.

### 6. Next Task

Continue to next pending task. Repeat until all done.

---

## HITL Checkpoints

Before these actions, STOP and ask user:

- Deleting files
- Modifying configuration (package.json, tsconfig, etc.)
- Installing dependencies
- Any action that feels risky

Format:
```
HITL: About to [action]
Proceed? [y]es / [n]o
```

---

## Failure Handling

**Verification fails:**
```
VERIFICATION FAILED: [reason]

Options:
[r]etry task
[s]kip task
[a]bort

What would you like to do?
```

**3 consecutive failures:**
```
ESCALATION: 3 consecutive failures

Consider:
- Fresh session (context may be drifted)
- Different approach
- User guidance

[r]etry / [f]resh session / [a]bort?
```

---

## State Files

| File | Purpose |
|------|---------|
| `.aria/state/current-plan.json` | Tasks and status |
| `.aria/design-notes.md` | AI reasoning (optional) |
| `.aria/project-context.md` | Project knowledge |

---

## Rules

1. **Plan before code** - No implementation without approved plan
2. **Fresh context per task** - Always use Task tool for implementation
3. **Verify after every task** - Run verify-executor.sh, no exceptions
4. **Stop on failure** - Do not proceed past failed verification
5. **HITL for risky actions** - When in doubt, ask

---

*ARIA - Fresh context, file state, verification gates*
