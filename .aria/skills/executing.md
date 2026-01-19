# Executing Skill

> Implement tasks from an approved plan with verification gates

## Prerequisites

Before executing:
1. Plan must exist at `.aria/state/current-plan.json`
2. Plan status must be `approved`
3. If no approved plan, use planning skill first

---

## Agent Loop (STANDARD+ Mode)

For implementation isolation, use the Boris Cherny Pattern 3 agent loop:

```
For each task:
1. analyzer   → Read code, understand patterns (read-only)
2. implementer → Make targeted changes (single file focus)
3. verify-app  → Test functionality (E2E)
4. verify.sh   → Run linting, tests, accessibility
5. Commit if passed
```

**Invoking agents via Task tool:**
```typescript
// Step 1: Analyze
Task({ subagent_type: "analyzer", prompt: "Analyze patterns in src/api/ for retry logic" })

// Step 2: Implement
Task({ subagent_type: "implementer", prompt: "Add retry wrapper to src/api/client.ts using pattern from utils/retry.ts" })

// Step 3: Verify
Task({ subagent_type: "verify-app", prompt: "Test the API client retry behavior works correctly" })

// Step 4: verify.sh (in main session)
bash .aria/verify.sh
```

**Why subagents:**
- Fresh context per task (no pollution from previous failures)
- Isolated implementation (can't drift)
- Main session stays in control (orchestrator role)

**When to use:**
- LITE mode: Skip subagents (direct implementation)
- STANDARD mode: Use subagents for implementation
- FULL/FULL+ mode: Use subagents for all implementation tasks

---

## Execution Loop

For each task in order:

```
1. ANNOUNCE  → "Starting task N: {title}"
2. CHECK     → Is this a HITL task? If yes, get approval first
3. IMPLEMENT → Write the code
4. VERIFY    → Run: bash .aria/verify.sh
5. COMMIT    → If verify passes, git commit
6. UPDATE    → Mark task done in plan
7. NEXT      → Continue to next task
```

---

## Step-by-Step

### 1. Announce

Before starting each task:
```
═══════════════════════════════════════════════════════════
Starting Task 2/5: Implement game logic
═══════════════════════════════════════════════════════════
Files: src/game.ts, src/types.ts
Estimated: 30 min
```

### 2. HITL Check

If task has `"hitl": true`:
```
HITL CHECKPOINT: This task involves {hitl_reason}

About to: {task description}
Files affected: {file list}

Proceed? [y]es / [n]o / [s]kip task
```

Wait for approval. Do NOT proceed without it.

### 3. Implement

Write the code. Follow these rules:
- Match existing code style
- Write tests alongside implementation
- Keep changes focused on this task only
- Log important decisions to `.aria/design-notes.md`

### 4. Verify

After implementation, ALWAYS run:
```bash
bash .aria/verify.sh
```

**For prototype builds, also run:**
```bash
# After HTML/CSS/JS prototype is created
python .aria/tests/run-tests.py unit --offline
```

**For slide generation:**
```bash
# Verify signals were emitted during slide generation
python .aria/scripts/verify-slide-signals.py
```

**If verification PASSES:**
- Continue to commit step

**If verification FAILS:**
- STOP immediately
- Report what failed
- Wait for guidance
- Do NOT try to fix silently and continue

### 5. Commit

If verification passes, commit the changes:
```bash
git add -A
git commit -m "feat: {task title}

Task {N}/{total} from plan {plan_id}"
```

### 6. Update Plan

Update `.aria/state/current-plan.json`:
- Set task status to `done`
- Add completion timestamp

### 7. Progress Report

After each task:
```
Task 2/5 complete ✓
Progress: ██████░░░░ 40%
Next: Task 3 - Add UI components
```

---

## Handling Failures

### Verification Fails

```
VERIFICATION FAILED

Issue: Tests failing in game.ts
- Expected: X wins when 3 in row
- Actual: Returns undefined

Options:
[f]ix the issue (explain what went wrong)
[s]kip this task
[a]bort execution

What would you like to do?
```

### Task Blocked

If a task can't proceed:
1. Mark task as `blocked` in plan
2. Note the blocker
3. Ask if should continue to next task or stop

### HITL Rejected

If user rejects a HITL checkpoint:
1. Mark task as `skipped`
2. Note the reason
3. Continue to next non-dependent task

---

## State Updates

Keep plan state current:

```json
{
  "tasks": [
    {
      "id": 1,
      "status": "done",
      "completed_at": "2026-01-11T14:45:00Z"
    },
    {
      "id": 2,
      "status": "in_progress",
      "started_at": "2026-01-11T14:45:30Z"
    },
    {
      "id": 3,
      "status": "pending"
    }
  ]
}
```

---

## Completion

When all tasks are done:

```
═══════════════════════════════════════════════════════════
EXECUTION COMPLETE
═══════════════════════════════════════════════════════════

Tasks: 5/5 completed
Duration: 47 minutes
Commits: 5

Summary:
✓ Task 1: Set up project structure
✓ Task 2: Implement game logic
✓ Task 3: Add UI components
✓ Task 4: Write tests
✓ Task 5: Add documentation

All verifications passed.

Next steps:
- Review the implementation
- Run full test suite: npm test
- Consider creating PR
```

Update plan status to `complete`.

---

## Design Notes

Log reasoning to `.aria/design-notes.md`:

```markdown
## Task 2: Implement game logic

### Assumptions
- Using TypeScript for type safety
- Board represented as 2D array

### Decisions
- Chose functional approach over class-based
- Separated win detection into pure function for testability

### Concerns
- [LOW] Edge case: simultaneous wins - handled by turn order
```

---

## Rules

1. **Never skip verification** - It's the enforcement layer
2. **Never proceed past failure** - Stop and report
3. **Always wait for HITL** - Don't assume approval
4. **One task at a time** - Complete before starting next
5. **Commit often** - Each task = one commit
6. **Log reasoning** - Transparency via design notes
