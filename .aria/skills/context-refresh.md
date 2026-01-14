# Context Refresh Skill

> Reset context while preserving progress during long sessions

---
version: 1.0.0
modes: [STANDARD, FULL, FULL+]
triggers: [between phases/epics, 3+ consecutive failures, extended session]
inputs: [current progress, plan state]
outputs: [handoff summary, preserved state]
dependencies: [tracking]
---

## When to Use

Use this skill when:
- Completing a phase (STANDARD) or major step (FULL/FULL+)
- 3+ consecutive failures occur
- Session exceeds ~2 hours of active work
- Context feels "drifted" (repeating mistakes, forgetting earlier decisions)

**Skip when:**
- LITE mode (sessions are short)
- Task is almost complete
- User says "keep going"

---

## Triggers by Mode

| Mode | Automatic Refresh Points |
|------|-------------------------|
| LITE | Never (manual only) |
| STANDARD | Between phases |
| FULL | Between major steps + phases |
| FULL+ | Between epics + major steps + phases |

---

## Workflow

### Step 1: Announce Refresh Point

```
CONTEXT REFRESH POINT

Completed: [Phase/Step/Epic name]
Progress: [X/Y tasks, Z%]

Options:
[c]ontinue with fresh context
[k]eep current context
[p]ause for break
```

Wait for user response.

---

### Step 2: Save State

Before refresh, preserve:

```json
{
  "refresh_point": "after_phase_2",
  "timestamp": "ISO-8601",
  "plan_id": "plan-YYYYMMDD-HHMMSS",
  "progress": {
    "completed_tasks": ["1.1", "1.2", "2.1"],
    "current_task": "2.2",
    "remaining_tasks": ["2.3", "3.1", "3.2"]
  },
  "key_decisions": [
    "Using sessions for auth (not JWT)",
    "PostgreSQL for database"
  ],
  "files_modified": [
    "src/auth.ts",
    "src/db.ts"
  ],
  "blockers": [],
  "notes": "Phase 2 took longer due to auth complexity"
}
```

Save to: `.aria/state/refresh-checkpoint.json`

---

### Step 3: Create Handoff Summary

Generate summary for new context:

```markdown
## Context Handoff

### Project
[Name] - [Brief description]

### Progress
- Completed: Phases 1-2 (8/15 tasks)
- Current: Task 2.2 - Implement login endpoint
- Remaining: 7 tasks

### Key Files
- `src/auth.ts` - Authentication logic
- `src/db.ts` - Database connection
- `.aria/state/current-plan.json` - Full plan

### Key Decisions Made
1. Using session-based auth (simpler for monolith)
2. PostgreSQL with Prisma ORM
3. Jest for testing

### Don't Touch
- `src/config.ts` - Production secrets
- `migrations/` - Already applied

### Next Action
Continue with task 2.2: Implement login endpoint

### Commands
- Test: `npm test`
- Dev: `npm run dev`
```

---

### Step 4: Re-Read Essential Files

After refresh, re-read:

1. `.aria/state/current-plan.json` - Current plan
2. `.aria/project-context.md` - Codebase knowledge
3. `.aria/state/refresh-checkpoint.json` - Where we left off
4. Recent files modified (from checkpoint)

---

## Failure-Triggered Refresh

When 3+ consecutive failures occur:

```
CONTEXT REFRESH: Failure Escalation

3 consecutive failures on: [issue description]

This may indicate context drift.

Options:
[r]efresh context and retry
[d]ifferent approach (same context)
[s]kip this task
[a]bort execution
```

If user chooses refresh:
1. Save current state
2. Create handoff summary
3. Clear working memory
4. Re-read essential files
5. Retry failed task with fresh perspective

---

## What to Preserve vs Clear

| Preserve | Clear |
|----------|-------|
| Plan and progress | Failed approach details |
| Key decisions | Stale assumptions |
| File locations | Intermediate reasoning |
| User preferences | Error context (summarize only) |
| Don't-touch areas | Temporary workarounds |

---

## Handoff File Location

Save handoff to: `.aria/state/handoff-[timestamp].md`

Keep last 3 handoffs (delete older ones to avoid clutter).

---

## Tips

- **Don't over-refresh** - Only at natural break points
- **Summarize, don't dump** - Handoff should be scannable
- **Preserve decisions** - Most valuable context to keep
- **Note the "why"** - Why decisions were made, not just what
- **Test after refresh** - Run verification to confirm state
