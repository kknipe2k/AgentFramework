# Tracking Skill

> Monitor progress, time, tokens, and HITL interactions

## When to Use

Use this skill:
- During execution to track progress
- After tasks complete to log metrics
- At project end for summary comparison
- STANDARD/FULL/FULL+ modes (optional for LITE)

---

## Mode-Specific Tracking

> **Note:** See `CLAUDE.md` → "Mode Definitions" for full mode behavior.
> This table shows tracking-specific differences only.

| Mode | Token Tracking | Time Tracking | HITL Logging | Progress File |
|------|----------------|---------------|--------------|---------------|
| **LITE** | Skip | Skip | Destructive only | Optional |
| **STANDARD** | Basic (totals) | Per-task | All interactions | Required |
| **FULL** | Detailed + checkpoints | Per-task + phases | All + wait times | Required |
| **FULL+** | Full + per-epic | All + projections | All + analytics | Required |

### LITE Mode Tracking

In LITE mode, tracking is minimal to maintain speed:

```
LITE MODE: Tracking disabled (speed priority)

What to track:
✓ Task completion (just done/not done)
✓ Any destructive HITL interactions
✓ Final outcome (success/failure)

What to skip:
✗ Token counting
✗ Time per task
✗ Detailed progress.json
✗ ccusage integration
```

**LITE Progress Announcement:**
```
✓ Done (3/3 tasks)
```

**When to upgrade tracking in LITE:**
- User explicitly requests metrics
- Task takes unexpectedly long (>30 min)
- Multiple failures occur

---

## Token Tracking (ccusage)

### About ccusage

[ccusage](https://github.com/ryoppippi/ccusage) analyzes Claude Code token usage from local JSONL log files. It reads from `~/.config/claude/projects/` automatically.

### Setup

No installation required - use npx:
```bash
# Recommended: always use @latest
npx ccusage@latest

# Or install globally
npm install -g ccusage
```

> **Note:** ccusage is an npm package, NOT a Python package. `pipx install` will not work.

### Usage

ccusage reads existing log files (not real-time tracking):
```bash
# Daily usage report (default)
npx ccusage@latest

# Monthly aggregated report
npx ccusage@latest monthly

# Usage by conversation session
npx ccusage@latest session

# 5-hour billing windows
npx ccusage@latest blocks
```

### Integration Points

**Check usage during work:**
```bash
npx ccusage@latest session
```

**At project end:**
```bash
npx ccusage@latest monthly > .aria/logs/token-report.txt
```

### Fallback: When ccusage Unavailable

If ccusage is not available or you're in a web environment:

1. **Estimate tokens manually:**
   - ~4 chars = 1 token (English text)
   - Code is ~1.5x more tokens than equivalent text
   - Large file reads: estimate lines × 10 tokens

2. **Track in progress.json:**
   ```json
   {
     "tokens": {
       "estimated_total": 50000,
       "tracking_method": "manual_estimate"
     }
   }
   ```

3. **Use Claude's built-in usage** (if available in your environment)

### Log Format

Save to `.aria/logs/token_usage.json`:
```json
{
  "session_id": "project-20260112",
  "started": "2026-01-12T10:00:00Z",
  "ended": "2026-01-12T12:30:00Z",
  "total_tokens": {
    "input": 45000,
    "output": 32000,
    "total": 77000
  },
  "checkpoints": [
    {
      "name": "Phase 1 complete",
      "timestamp": "2026-01-12T10:45:00Z",
      "tokens_so_far": 25000
    }
  ],
  "estimated_cost": "$0.23"
}
```

---

## Time Tracking

### Per-Task Timing

Track actual vs estimated time:

```json
{
  "task_id": "1.2.3",
  "title": "Implement login form",
  "estimated_minutes": 30,
  "actual_minutes": 42,
  "started": "2026-01-12T10:15:00Z",
  "ended": "2026-01-12T10:57:00Z",
  "variance": "+12 min (+40%)"
}
```

### Aggregate Timing

In `.aria/state/progress.json`:
```json
{
  "timing": {
    "estimated_total_minutes": 180,
    "actual_total_minutes": 210,
    "variance_percent": "+16.7%",
    "tasks_completed": 12,
    "tasks_remaining": 3,
    "avg_task_minutes": 17.5
  }
}
```

### Time Logging Rules

1. **Start timer** when announcing task start
2. **Stop timer** when verification passes
3. **Pause timer** during HITL waits (don't count user think time)
4. **Log variance** if >20% over estimate

---

## HITL Interaction Logging

### What to Log

Every HITL interaction should be recorded:

| Field | Description |
|-------|-------------|
| timestamp | When checkpoint triggered |
| type | approval / question / pause / escalation |
| context | What triggered it |
| user_response | What user decided |
| wait_time | How long until response |

### Log Format

Save to `.aria/logs/hitl_interactions.json`:
```json
{
  "interactions": [
    {
      "id": 1,
      "timestamp": "2026-01-12T10:30:00Z",
      "type": "approval",
      "context": "Plan review - 12 tasks",
      "options_presented": ["approve", "revise", "cancel"],
      "user_response": "approve",
      "wait_seconds": 45,
      "notes": null
    },
    {
      "id": 2,
      "timestamp": "2026-01-12T11:15:00Z",
      "type": "question",
      "context": "Auth implementation approach",
      "options_presented": ["JWT", "Sessions", "Hybrid"],
      "user_response": "Sessions",
      "wait_seconds": 120,
      "notes": "User preferred simpler approach"
    },
    {
      "id": 3,
      "timestamp": "2026-01-12T11:45:00Z",
      "type": "escalation",
      "context": "3 failures on database connection",
      "options_presented": ["retry", "fresh", "skip", "abort"],
      "user_response": "fresh",
      "wait_seconds": 30,
      "notes": "Context refresh resolved issue"
    }
  ],
  "summary": {
    "total_interactions": 3,
    "total_wait_seconds": 195,
    "by_type": {
      "approval": 1,
      "question": 1,
      "escalation": 1
    }
  }
}
```

### HITL Types

| Type | When | Expected Response |
|------|------|-------------------|
| `approval` | Plan review, HITL tasks | approve/revise/cancel |
| `question` | Decision needed | Option selection |
| `pause` | User requested stop | resume/abort |
| `escalation` | 3+ failures | retry/fresh/skip/abort |
| `checkpoint` | Phase/epic complete | continue/review |

---

## Progress Tracking

### Progress File

Maintain `.aria/state/progress.json`:

```json
{
  "plan_id": "plan-20260112-100000",
  "status": "in_progress",
  "started": "2026-01-12T10:00:00Z",
  "last_updated": "2026-01-12T11:30:00Z",

  "completion": {
    "tasks_total": 15,
    "tasks_done": 8,
    "tasks_in_progress": 1,
    "tasks_blocked": 0,
    "tasks_skipped": 1,
    "percent_complete": 53
  },

  "current": {
    "phase": "2",
    "phase_name": "Core Implementation",
    "task": "2.3",
    "task_name": "Build API endpoints"
  },

  "timing": {
    "estimated_total_minutes": 180,
    "actual_so_far_minutes": 95,
    "projected_total_minutes": 179,
    "on_track": true
  },

  "tokens": {
    "estimated_total": 50000,
    "actual_so_far": 28000,
    "projected_total": 52800
  },

  "failures": {
    "total": 2,
    "consecutive": 0,
    "last_failure": "Task 2.1 - type error",
    "resolved": true
  },

  "hitl_interactions": 3,

  "refresh_points_hit": [
    "after_phase_1"
  ],

  "notes": [
    "Phase 1 took longer due to unfamiliar codebase",
    "Skipped task 1.4 - user said not needed"
  ]
}
```

### Progress Announcements

**STANDARD mode:** After each task
```
✓ Task 2.3 complete (8/15 tasks, 53%)
  Time: 12 min (est: 15 min)
  Next: Task 2.4 - Connect to database
```

**FULL/FULL+ mode:** After each task with estimates
```
✓ Task 2.3 complete
  Progress: 8/15 tasks (53%)
  Time: 95 min elapsed, ~84 min remaining
  Tokens: ~28K used, ~22K remaining
  On track: Yes ✓

  Next: Task 2.4 - Connect to database (~20 min)
```

---

## Metrics Comparison (End of Project)

### Final Report Metrics

Compare estimated vs actual:

```markdown
## Metrics Summary

| Metric | Estimated | Actual | Variance |
|--------|-----------|--------|----------|
| Tasks | 15 | 14 (1 skipped) | -1 |
| Time | 180 min | 195 min | +8% |
| Tokens | 50,000 | 52,800 | +6% |
| HITL interactions | 4 | 6 | +2 |
| Failures | 0 | 3 | +3 |

### Variance Analysis
- Time +8%: Phase 1 took longer (unfamiliar codebase)
- Extra HITL: 2 questions about auth approach
- Failures: Database connection issues (resolved with context refresh)
```

---

## File Locations

| File | Purpose |
|------|---------|
| `.aria/state/progress.json` | Current progress state |
| `.aria/logs/token_usage.json` | Token tracking from ccusage |
| `.aria/logs/hitl_interactions.json` | HITL interaction log |
| `.aria/logs/session-[date].json` | Full session export |

---

## Tips

- **Don't over-track in LITE** - Just note completion, skip detailed metrics
- **Checkpoint frequently in LARGE+** - Helps with context refresh decisions
- **Log HITL wait time** - Helps identify where users need more info
- **Track variance reasons** - Useful for future estimates
- **Export at end** - Full session data for retrospectives
