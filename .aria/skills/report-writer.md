# Report Writer Skill

> Generate final summary reports with metrics and dashboard integration

---
version: 1.0.0
modes: [LITE, STANDARD, FULL, FULL+]
triggers: [workflow complete, HITL confirms done, user requests summary]
inputs: [progress.json, decisions.jsonl, signals.jsonl, token_usage.json]
outputs: [summary report, metrics comparison, dashboard offer]
dependencies: [tracking]
---

## When to Use

**Automatically triggered when:**
- All tasks in plan marked complete
- HITL confirms "done" at end of workflow
- User says "summary", "report", "metrics", "how did we do"

**Manually via:**
- `/aria-summary` slash command
- "Show me the summary"

---

## End-of-Workflow Summary

When workflow completes, generate this report:

```markdown
## Session Complete

### Summary
- **Mode:** [LITE|STANDARD|FULL|FULL+]
- **Tasks:** X/Y completed (Z skipped)
- **Duration:** Xh Ym
- **Commits:** N

### Metrics Comparison

| Metric | Estimated | Actual | Variance |
|--------|-----------|--------|----------|
| Tasks | X | Y | ±Z |
| Time | X min | Y min | ±Z% |
| Tokens | X | Y | ±Z% |

### Decision Trace
- Decisions made: N
- Avg confidence: 0.XX
- HITL checkpoints: N

### Key Decisions
1. [Most impactful decision with rationale]
2. [Second most impactful]
3. [Third]

### Files Changed
- `path/to/file.ts` - [what changed]
- `path/to/other.ts` - [what changed]

### Variance Analysis
[If variance > 20%, explain why]

---

**View full lineage?**
→ Run: `python .aria/scripts/serve-dashboard.py`
→ Open: http://localhost:8420
```

---

## Mode-Specific Reports

### LITE Mode

Minimal report:
```
✓ Done (X tasks)
Duration: Xm
```

Skip metrics comparison, decision trace, dashboard offer.

### STANDARD Mode

Standard report with:
- Task completion summary
- Basic metrics (time, tokens)
- Key decisions (top 3)
- Dashboard offer

### FULL/FULL+ Mode

Full report with:
- Everything in STANDARD
- Complete metrics comparison table
- All decisions with confidence scores
- Variance analysis
- Files changed with diffs summary
- Automatic dashboard offer

---

## Automatic Dashboard Offer

At end of STANDARD/FULL/FULL+ workflows:

```
HITL: Session complete. View dashboard?
[y]es - Open dashboard in browser
[n]o - Show text summary only
[s]ave - Export report to .aria/reports/
```

If user selects `[y]es`:
1. Start dashboard server (if not running)
2. Open browser to http://localhost:8420
3. Dashboard shows hierarchical lineage for this session

If user selects `[s]ave`:
1. Save report to `.aria/reports/SESSION-[date].md`
2. Include all metrics, decisions, file changes

---

## Data Sources

Read from these files to generate report:

| File | Data |
|------|------|
| `.aria/state/progress.json` | Task completion, timing |
| `.aria/state/current-plan.json` | Original estimates |
| `.aria/state/decisions.jsonl` | Decision traces |
| `.aria/state/signals.jsonl` | Tool calls, file changes |
| `.aria/logs/token_usage.json` | Token metrics |
| `.aria/logs/hitl_interactions.json` | HITL checkpoints |

---

## Report Templates

### Research Flow Report

```markdown
## Research Complete: [Topic]

### Artifacts Generated
- `.aria/docs/IDEA.md` - Research synthesis
- `.aria/outputs/FOCUS.md` - Core ideas matrix
- `.aria/outputs/slides-[topic].pptx` - Presentation

### Sources Analyzed
1. [Source 1]
2. [Source 2]

### Key Insights
1. [Insight 1]
2. [Insight 2]
3. [Insight 3]

### Prototype
[If built: location and how to run]
[If skipped: "User opted to skip prototype"]

---

**View decision lineage?** → /aria-dashboard
```

### Build/Modify Flow Report

```markdown
## Build Complete: [Feature]

### Implementation Summary
- Tasks: X/Y complete
- Tests: [passing/failing]
- Build: [passing/failing]

### Changes
| File | Lines | Type |
|------|-------|------|
| src/foo.ts | +45/-12 | Feature |
| src/bar.ts | +8/-3 | Refactor |

### Commits
1. `abc123` - [message]
2. `def456` - [message]

### Metrics
[Standard metrics table]

---

**View decision lineage?** → /aria-dashboard
```

---

## Integration Points

### Workflow End Detection

Detect workflow complete when:
1. All tasks in `current-plan.json` have `status: "completed"`
2. User confirms "done" at HITL checkpoint
3. No more pending tasks

### Dashboard Launch

To launch dashboard programmatically:
```bash
# Check if already running
pgrep -f "serve-dashboard.py" || python .aria/scripts/serve-dashboard.py &

# Open browser (cross-platform)
open http://localhost:8420 2>/dev/null || \
xdg-open http://localhost:8420 2>/dev/null || \
start http://localhost:8420
```

---

## Example Output

```
════════════════════════════════════════════════════════════
                    SESSION COMPLETE
════════════════════════════════════════════════════════════

Mode: STANDARD
Duration: 47 minutes
Tasks: 8/8 completed

┌──────────┬───────────┬────────┬──────────┐
│ Metric   │ Estimated │ Actual │ Variance │
├──────────┼───────────┼────────┼──────────┤
│ Tasks    │ 8         │ 8      │ 0        │
│ Time     │ 60 min    │ 47 min │ -22%     │
│ Tokens   │ 45K       │ 38K    │ -16%     │
└──────────┴───────────┴────────┴──────────┘

Key Decisions:
1. Used existing retry pattern (0.85) - consistency
2. Skipped auth refactor (0.72) - out of scope
3. Added error boundary (0.90) - user safety

Commits: 4
Files changed: 12

════════════════════════════════════════════════════════════

HITL: View dashboard for full lineage?
[y]es / [n]o / [s]ave report
```

---

## Tips

- Always offer dashboard at end of STANDARD+ workflows
- Include variance analysis if any metric >20% off
- Link key decisions to their supporting signals
- Save reports for retrospectives
- Use consistent formatting across modes
