# Meta Reasoning Engine Skill

> **Grade: A** | Core skill for systematic decision-making with offline reinforcement learning

## Overview

The Meta Reasoning Engine provides a structured approach to problem-solving that:
1. Maps the solution space before committing to an approach
2. Selects paths using learned priors from past sessions
3. Monitors execution for dead-end signals
4. Extracts meta-insights for future learning

## When to Use

- **Always** at the start of complex tasks (STANDARD/FULL/FULL+ modes)
- When facing multiple viable approaches
- After 2+ failures on the same task
- When entering unfamiliar code areas

## The Four Phases

### 1. MAP THE SOLUTION SPACE

Before solving, enumerate possible approaches:

```
SOLUTION SPACE for "Add retry logic to API client":

1. Extend existing utility (60% success probability)
   - Modify: src/utils/api-client.ts
   - Risk: May break existing callers
   - Similar past: Task Y succeeded with this

2. Create wrapper function (75% success probability)
   - Create: src/utils/retry-wrapper.ts
   - Risk: Code duplication
   - Similar past: Task Z used this pattern

3. Use existing retry library (80% success probability)
   - Install: p-retry
   - Risk: New dependency
   - Similar past: No prior experience
```

**Key Questions:**
- What approaches could solve this?
- Which approaches have I seen fail before?
- What's the probability each approach succeeds?

### 2. SELECT THE PATH

Choose approach using learned priors:

```
CHOSEN PATH: Create wrapper function

RATIONALE:
- Learned: 75% success rate over 12 past observations
- Matches existing patterns in src/utils/
- No new dependencies (risk reduction)
- Lower coupling than extending existing client

CONFIDENCE: 0.75 (calibrated from 0.8 based on learned adjustment)
```

**Selection Criteria:**
- Learned success rates from past sessions
- Risk factors and dependencies
- Alignment with existing codebase patterns
- Cost-benefit trade-offs

### 3. EXECUTE WITH AWARENESS

Track progress against checkpoints:

```
EXECUTION LOG:

[Checkpoint 1: After reading code]
✓ Approach still makes sense
  - Found similar pattern in src/utils/cache-wrapper.ts
  - Confirmed no circular dependency risk

[Checkpoint 2: After first edit]
✓ Solving the right problem
  - Added RetryWrapper class
  - Matches acceptance criteria

[Dead-end Signal Detected!]
⚠ Same file edited 4 times
  - src/utils/retry-wrapper.ts
  - ACTION: Backtrack, consider different approach

[Backtrack]
- Abandoning wrapper approach
- Trying: Extend existing utility instead
- Reason: Wrapper creating too much indirection
```

**Dead-End Signals:**
- Same file edited 3+ times
- Test flip-flopping (pass→fail→pass)
- Increasing complexity instead of decreasing
- Circular dependency warnings

### 4. META-ANALYZE

Extract learnings for future sessions:

```
META-INSIGHT:

What worked:
- Using existing pattern from cache-wrapper.ts
- Keeping wrapper simple (single responsibility)

What didn't work:
- First attempt with complex retry configuration
- Trying to handle too many edge cases initially

Pattern discovered:
- In this codebase, wrappers should be thin
- Configuration should be injected, not hardcoded

Recommendation for future:
- When adding utilities, check existing *-wrapper.ts files first
- Start with minimal implementation, add complexity only when needed
```

## Integration with Offline Learning

### How Learning Works

```
┌─────────────────────────────────────────────────────────────────┐
│                     LEARNING LOOP                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  SESSION N                                                      │
│  ┌──────────────────┐                                          │
│  │ 1. Load policy   │ ← .aria/learned/policy.json              │
│  │ 2. Make decisions│   (model selection, strategy, etc.)      │
│  │ 3. Execute tasks │                                          │
│  │ 4. Record outcomes│ → .aria/logs/model_learning.json        │
│  │ 5. Log decisions │ → .aria/state/decisions.jsonl            │
│  └──────────────────┘                                          │
│                                                                 │
│  BETWEEN SESSIONS                                               │
│  ┌──────────────────┐                                          │
│  │ python offline-learner.py learn                             │
│  │                                                              │
│  │ - Extract episodes from logs                                │
│  │ - Calculate rewards (success/fail + cost)                   │
│  │ - Update Beta priors using Thompson Sampling                │
│  │ - Export improved policy                                    │
│  └──────────────────┘                                          │
│           │                                                     │
│           ▼                                                     │
│  SESSION N+1                                                    │
│  ┌──────────────────┐                                          │
│  │ Uses IMPROVED    │ ← Better model selection                 │
│  │ decision-making  │ ← Calibrated confidence                  │
│  │                  │ ← Strategy recommendations               │
│  └──────────────────┘                                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Data Sources Used for Learning

| Source | Location | What's Learned |
|--------|----------|----------------|
| Model outcomes | `logs/model_learning.json` | Which models succeed for which task types |
| Decisions | `state/decisions.jsonl` | Confidence calibration, decision quality |
| Signals | `state/signals.jsonl` | Tool call patterns, failure sequences |
| Progress | `state/progress.json` | Estimation accuracy, timing patterns |

### Reward Signal Definition

```python
# Model Selection Reward
if task_succeeded:
    reward = 1.0
else:
    reward = -0.5

# Cost penalty (normalized)
reward -= 0.1 * (model_cost / avg_cost)

# Bonus for staying under budget
if under_token_budget:
    reward += 0.2

# Normalize to [0, 1] for Beta distribution
reward = (reward + 0.6) / 1.7
```

### Using Learned Policy

```bash
# In your scripts, source the meta-reasoning library
source .aria/lib/meta-reasoning.sh

# Get model recommendation
result=$(meta_select_model "feature" 6 "api")
# Output: sonnet|0.78|Learned from 15 past observations for feature|medium|api

# Get strategy recommendation
result=$(meta_select_strategy "bugfix" 2 true)
# Output: tdd_approach|0.82|After failures, TDD provides better feedback loop

# Calibrate confidence
calibrated=$(meta_calibrate_confidence 0.8 "architecture")
# Output: 0.72 (if agents tend to be overconfident on architecture decisions)

# Full meta-reasoning cycle
meta_reason "Implement retry logic for API calls" "feature" 6
```

## Output Structure

When using this skill, structure your output as:

```
## META REASONING

### SOLUTION SPACE
[List 2-4 approaches with probabilities]

### CHOSEN PATH
[Selected approach with rationale]
[Confidence score with calibration note if adjusted]

### EXECUTION CHECKPOINTS
[ ] Checkpoint 1: [validation point]
[ ] Checkpoint 2: [validation point]
[ ] Checkpoint 3: [validation point]

### DEAD-END SIGNALS TO WATCH
- [Signal 1]
- [Signal 2]

---
[Proceed with implementation]
---

### EXECUTION LOG
[Track checkpoints, note any backtracking]

### META-INSIGHT
[What this taught about reasoning]
```

## Shell Commands

```bash
# Trigger learning after session
python .aria/lib/offline-learner.py learn

# View current policy
python .aria/lib/offline-learner.py export-policy

# Query for specific task
python .aria/lib/offline-learner.py query feature 7 auth

# View learning statistics
python .aria/lib/offline-learner.py stats

# Using shell functions
source .aria/lib/meta-reasoning.sh
meta_stats          # Show learning stats
meta_learn          # Trigger learning
meta_reason "task" "type" complexity  # Full reasoning cycle
```

## Example: Complete Meta-Reasoning Cycle

```
============================================
META REASONING ENGINE
============================================

TASK: Add rate limiting to API endpoints
TYPE: feature | COMPLEXITY: 6

1. SOLUTION SPACE
-----------------
[
  {"name": "direct_implementation", "probability": 0.6, "description": "Implement directly"},
  {"name": "tdd_approach", "probability": 0.82, "description": "Write tests first"},
  {"name": "refactor_first", "probability": 0.65, "description": "Clean up related code"},
  {"name": "minimal_spike", "probability": 0.75, "description": "Build minimal prototype"}
]

2. CHOSEN PATH
--------------
Model: sonnet (confidence: 0.78)
  Rationale: Learned from 15 past observations for feature|medium|api

Strategy: tdd_approach (confidence: 0.82)
  Rationale: Learned from 8 observations - TDD has highest success rate

3. EXECUTION CHECKPOINTS
------------------------
  [ ] After reading code: Does approach still make sense?
  [ ] After first edit: Is this solving the right problem?
  [ ] Before committing: Does this match acceptance criteria?

4. DEAD-END SIGNALS
-------------------
  - Same file edited 3+ times
  - Test flip-flopping (pass→fail→pass)
  - Increasing complexity instead of decreasing

5. LEARNED RECOMMENDATIONS
--------------------------
  - Use sonnet for feature|medium|api (learned: 78% success over 15 tasks)
  - Avoid haiku for feature tasks (learned: only 35% success over 12 tasks)
  - Agents are overconfident on architecture decisions (adjust by -8%)

============================================
```

## Mode Variations

| Mode | Meta Reasoning Depth |
|------|---------------------|
| LITE | Skip (speed over traceability) |
| STANDARD | Solution space + chosen path only |
| FULL | Full cycle with execution log |
| FULL+ | Full cycle + mandatory meta-insight extraction |

## Best Practices

1. **Always map before choosing** - Even if the path seems obvious
2. **Trust the learned priors** - They're based on actual outcomes
3. **Record outcomes honestly** - Learning depends on accurate data
4. **Check for dead-ends early** - Backtrack is cheaper than persisting
5. **Extract meta-insights** - Future you will thank present you

## Related Skills

- `planning.md` - Uses meta-reasoning for task decomposition
- `debugging.md` - Uses dead-end detection for troubleshooting
- `tracking.md` - Provides data for learning pipeline
- `context-refresh.md` - Triggered when meta-reasoning detects confusion
