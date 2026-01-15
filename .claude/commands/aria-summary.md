# ARIA Summary

Generate end-of-session summary report with metrics and dashboard offer.

## Instructions

Read state files and generate comprehensive summary:

### 1. Read State Files

Load data from:
- `.aria/state/progress.json` - Task completion
- `.aria/state/current-plan.json` - Original estimates
- `.aria/state/decisions.jsonl` - Decision traces
- `.aria/state/signals.jsonl` - Tool calls
- `.aria/logs/token_usage.json` - Token metrics (if exists)

### 2. Generate Summary Report

```
════════════════════════════════════════════════════════════
                    SESSION SUMMARY
════════════════════════════════════════════════════════════

Mode: [LITE|STANDARD|FULL|FULL+]
Duration: X minutes
Tasks: X/Y completed (Z skipped)

┌──────────┬───────────┬────────┬──────────┐
│ Metric   │ Estimated │ Actual │ Variance │
├──────────┼───────────┼────────┼──────────┤
│ Tasks    │ X         │ Y      │ ±Z       │
│ Time     │ X min     │ Y min  │ ±Z%      │
│ Tokens   │ XK        │ YK     │ ±Z%      │
└──────────┴───────────┴────────┴──────────┘

Key Decisions:
1. [Decision] (confidence: 0.XX)
2. [Decision] (confidence: 0.XX)
3. [Decision] (confidence: 0.XX)

Files Changed: N
Commits: N
HITL Checkpoints: N

════════════════════════════════════════════════════════════
```

### 3. Offer Dashboard

After showing summary:

```
HITL: View full lineage in dashboard?
[y]es - Launch dashboard (http://localhost:8420)
[n]o - Done
[s]ave - Export report to .aria/reports/
```

### 4. Handle Response

**If [y]es:**
```bash
python .aria/scripts/serve-dashboard.py &
# Then open browser
```

**If [s]ave:**
Save to `.aria/reports/SESSION-[date].md`

### Output Format

Present the summary with clear formatting, then the HITL prompt.
If no state files exist, report "No session data found."
