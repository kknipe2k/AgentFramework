# ARIA Status

Show comprehensive ARIA system status.

## Instructions

Gather and display status from all ARIA components:

### 1. Ralph Status
Run: `./.aria/ralph/ralph.sh status`
- Feature being developed
- Stories completed vs remaining
- Current branch

### 2. Model Status
Run: `./.aria/model-selector.sh status`
- Budget used/remaining
- Calls by model
- Learning statistics

### 3. HITL Status
Run: `./.aria/hitl.sh status`
- Any pending requests
- Recent interactions

### 4. Git Status
- Current branch
- Uncommitted changes
- Available checkpoints (if any)

### Output Format

Present a clear dashboard:
```
ARIA System Status
==================

Feature: [name]
Progress: X/Y stories complete

Budget: $X.XX / $Y.YY (Z% remaining)
Model calls: opus: N, sonnet: N, haiku: N

HITL: [No pending requests | N pending]

Git: [branch] - [clean | N uncommitted files]
```

Highlight any issues or recommended actions.
