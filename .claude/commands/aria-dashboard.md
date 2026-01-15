# ARIA Dashboard

Launch the ARIA decision lineage dashboard.

## Instructions

Start the dashboard server and open it in the browser:

### 1. Start Server

Run in background:
```bash
python .aria/scripts/serve-dashboard.py &
```

Or if already running, just report the URL.

### 2. Open Dashboard

The dashboard is available at: http://localhost:8420

### 3. What You'll See

**Hierarchical Lineage View:**
- SESSION at top level
- SKILLS as collapsible containers
- DECISIONS nested under skills with confidence scores
- SIGNALS (tool calls) nested under decisions
- COMMITS with linked decision traces

**Summary Metrics:**
- Total skills, decisions, signals loaded
- HITL interactions count
- Test runs and commits

### 4. Data Sources

The dashboard reads from:
- `.aria/state/signals.jsonl` - Tool call signals
- `.aria/state/decisions.jsonl` - Decision traces
- `.aria/state/progress.json` - Task progress

### Output

Report:
```
ARIA Dashboard
==============
Server: http://localhost:8420
Status: Running (PID: XXXX)

Data:
- Signals: N entries
- Decisions: N entries
- Skills loaded: [list]

Open in browser to view hierarchical lineage.
```
