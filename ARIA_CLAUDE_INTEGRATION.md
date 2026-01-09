# ARIA + Claude Code Integration

How ARIA plugs into Claude Code's existing architecture.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     CLAUDE CODE                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  USER: "Add authentication to my app"                          │
│                         │                                       │
│                         ▼                                       │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              ARIA SKILL (auto-triggered)                 │   │
│  │                                                          │   │
│  │   1. Analyze codebase context                           │   │
│  │   2. Generate ARIA plan                                  │   │
│  │   3. Show plan to user for approval                     │   │
│  │   4. Execute via ARIA Executor                          │   │
│  │                                                          │   │
│  └─────────────────────────────────────────────────────────┘   │
│                         │                                       │
│            ┌────────────┼────────────┐                         │
│            ▼            ▼            ▼                         │
│       ┌────────┐  ┌────────┐  ┌────────┐                      │
│       │ HOOKS  │  │ AGENTS │  │ TOOLS  │                      │
│       │        │  │        │  │        │                      │
│       │PreTool │  │ Task   │  │ Bash   │                      │
│       │PostTool│  │subagent│  │ Edit   │                      │
│       │Stop    │  │        │  │ Write  │                      │
│       └────────┘  └────────┘  └────────┘                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 1. ARIA Skill Definition

`.claude/skills/aria/SKILL.md`:

```markdown
---
name: aria-orchestrator
description: |
  Automatically generates and executes structured ARIA plans for complex tasks.
  Triggers when: multi-step implementation, feature addition, refactoring, bug fixes.
allowed-tools: Bash, Edit, Write, Read, Glob, Grep, Task
---

# ARIA Orchestrator

When the user requests a task that involves multiple files or steps:

1. **Analyze** the codebase to understand structure
2. **Generate** an ARIA plan capturing intent and steps
3. **Present** the plan to user for approval
4. **Execute** with verification at each gate

## Plan Generation Rules

- Always capture intent with must_have/must_not
- Break into atomic phases
- Add verification gates after each phase
- Include tests before marking complete
- Create checkpoints for rollback capability

## Execution Rules

- Never skip gates
- On gate failure, rollback to checkpoint
- Always verify intent at end
- Generate docs and commit on success

## Example Trigger

User: "Add user authentication"

Response: Generate ARIA plan, show to user, execute on approval.
```

---

## 2. Hook Integration

### PreToolUse Hook - Plan Validation

`.claude/hooks/aria-pre-tool.sh`:

```bash
#!/bin/bash
# Validates that complex operations go through ARIA planning

TOOL="$1"
INPUT="$2"

# If editing multiple files without an active ARIA plan, warn
if [[ "$TOOL" == "Edit" || "$TOOL" == "Write" ]]; then
  if [[ ! -f ".aria/current-plan.yaml" ]]; then
    EDIT_COUNT=$(cat /tmp/aria-edit-count 2>/dev/null || echo 0)
    EDIT_COUNT=$((EDIT_COUNT + 1))
    echo $EDIT_COUNT > /tmp/aria-edit-count

    if [[ $EDIT_COUNT -gt 3 ]]; then
      echo '{"warning": "Multiple edits without ARIA plan. Consider: @aria plan"}'
      # Don't block, just warn
    fi
  fi
fi

exit 0
```

### PostToolUse Hook - Gate Checking

`.claude/hooks/aria-post-tool.sh`:

```bash
#!/bin/bash
# Runs ARIA gates after tool execution

TOOL="$1"
RESULT="$2"

# Check if we're in an ARIA execution
if [[ -f ".aria/current-plan.yaml" ]]; then
  PHASE=$(cat .aria/current-phase)

  # Run phase gates
  aria check-gates --phase "$PHASE"

  if [[ $? -ne 0 ]]; then
    echo '{"error": "ARIA gate failed", "action": "review_required"}'
    exit 2  # Block further execution
  fi
fi

exit 0
```

### Stop Hook - Final Verification

`.claude/hooks/aria-stop.sh`:

```bash
#!/bin/bash
# End-of-turn verification

if [[ -f ".aria/current-plan.yaml" ]]; then
  # Run all pending gates
  RESULT=$(aria verify-intent)

  if [[ "$RESULT" == *"DRIFT"* ]]; then
    echo "Intent drift detected. Review changes."
    echo "$RESULT"
  fi

  # Check if plan is complete
  if aria is-complete; then
    aria finalize  # Generate docs, commit, cleanup
    rm -f .aria/current-plan.yaml
  fi
fi
```

---

## 3. Subagent Architecture

### ARIA Planner Agent

`.claude/agents/aria-planner.md`:

```markdown
---
name: aria-planner
description: Generates ARIA execution plans from user intent
model: opus
tools: [Read, Glob, Grep]
---

You are the ARIA Plan Generator. Given a user's intent and codebase context,
generate a complete ARIA plan.

## Your Process

1. Understand the intent fully
2. Explore relevant code
3. Identify all required changes
4. Generate ARIA plan with:
   - Clear intent (must_have, must_not)
   - Atomic phases
   - Verification gates
   - Tests
   - Checkpoints

## Output Format

Return ONLY valid ARIA syntax. No explanation. No markdown. Just the plan.

## Quality Rules

- Every file edit needs a gate
- Every feature needs a test
- Every phase needs a checkpoint
- Intent must be checkable at end
```

### ARIA Gate Checker Agent

`.claude/agents/aria-gate-checker.md`:

```markdown
---
name: aria-gate-checker
description: Verifies ARIA gates during execution
model: haiku
tools: [Read, Bash, Grep]
---

You verify ARIA gates. Given a gate definition and current state,
determine if the gate passes.

## Gate Types

- `contains(file, pattern)` - Check file contains pattern
- `!contains(file, pattern)` - Check file doesn't contain pattern
- `exists(path)` - Check path exists
- `command == N` - Check command exit code
- `llm "question"` - You answer the question

## Output

Return JSON:
```json
{
  "gate": "gate_name",
  "passed": true/false,
  "reason": "brief explanation"
}
```
```

### ARIA Intent Verifier Agent

`.claude/agents/aria-intent-verifier.md`:

```markdown
---
name: aria-intent-verifier
description: Verifies implementation matches original intent
model: sonnet
tools: [Read, Grep, Glob]
---

You are the final arbiter of intent satisfaction.

Given:
- Original intent (must_have, must_not)
- Files created/modified
- Test results

Determine if the intent is satisfied.

## Process

1. Check each must_have requirement
2. Verify no must_not violations
3. Review test coverage
4. Make final determination

## Output

```json
{
  "satisfied": true/false,
  "must_have_results": [
    {"requirement": "...", "met": true/false, "evidence": "..."}
  ],
  "must_not_results": [
    {"requirement": "...", "violated": true/false, "evidence": "..."}
  ],
  "overall_confidence": 0.95,
  "recommendation": "proceed/review/rollback"
}
```
```

---

## 4. Execution Flow

### User Triggers ARIA

```
User: "Add rate limiting to the API"

Claude: I'll create an ARIA plan for this.

[Spawns aria-planner agent]

Agent returns:
```

```aria
@plan "Add rate limiting to API"

@intent
  + All endpoints rate limited
  + Returns 429 when exceeded
  + Configurable limits per endpoint
  - No breaking changes to existing API

@require express-rate-limit

---

@phase setup
  > npm install express-rate-limit
  ? installed: package.json has express-rate-limit
  * checkpoint

@phase middleware
  @ src/middleware/rateLimit.js
    ```js
    const rateLimit = require('express-rate-limit');

    const createLimiter = (max, window) => rateLimit({
      windowMs: window,
      max: max,
      message: { error: 'Too many requests' }
    });

    module.exports = { createLimiter };
    ```
  * checkpoint

@phase apply
  @ src/app.js
    - const app = express();
    + const app = express();
    + const { createLimiter } = require('./middleware/rateLimit');
    + app.use('/api/', createLimiter(100, 15 * 60 * 1000));
  ? has_limiter: contains(app.js, "createLimiter")
  * checkpoint

@phase test
  @ tests/rateLimit.test.js
    ```js
    test('returns 429 when rate limited', async () => {
      for (let i = 0; i < 101; i++) {
        await request(app).get('/api/test');
      }
      const res = await request(app).get('/api/test');
      expect(res.status).toBe(429);
    });
    ```
  ? tests: `npm test` == 0
  * checkpoint

@phase verify
  ? intent: llm "Rate limiting added, 429 on exceed, configurable?"

@done
  > git commit -m "feat: add API rate limiting"
```

```
Claude: Here's the ARIA plan. Shall I execute it?

User: Yes

[Executes plan phase by phase, checking gates]

Claude: Complete. Rate limiting added.
- Created: src/middleware/rateLimit.js
- Modified: src/app.js
- Tests: 5/5 passing
- Committed: feat: add API rate limiting
```

---

## 5. CLI Commands

```bash
# Generate plan without executing
claude "Add caching" --aria-plan-only

# Execute existing plan
claude --aria-execute .aria/plans/caching.aria

# Check current plan status
claude --aria-status

# Rollback to checkpoint
claude --aria-rollback setup

# Verify intent
claude --aria-verify

# List available plans
claude --aria-list
```

---

## 6. State Management

### Plan State File

`.aria/current-plan.yaml`:

```yaml
plan: add-rate-limiting
started: 2024-01-15T10:30:00Z
current_phase: apply
checkpoints:
  - name: setup
    timestamp: 2024-01-15T10:31:00Z
    git_sha: abc123
  - name: middleware
    timestamp: 2024-01-15T10:32:00Z
    git_sha: def456
gates_passed:
  - installed
  - has_limiter
gates_failed: []
intent_hash: a3f2b1c9
```

### Checkpoint Storage

`.aria/checkpoints/`:

```
checkpoints/
├── setup/
│   ├── files.tar.gz
│   └── metadata.json
├── middleware/
│   ├── files.tar.gz
│   └── metadata.json
└── apply/
    ├── files.tar.gz
    └── metadata.json
```

---

## 7. Error Recovery

### Gate Failure

```
[Executing phase: test]
> npm test

GATE FAILED: tests
Exit code: 1
3 tests failed

Options:
1. View failures
2. Rollback to checkpoint
3. Fix and retry
4. Abort plan

Claude: Tests failed. Let me analyze the failures...

[Reads test output, identifies issue]

Claude: The rate limit test is flaky due to timing.
I'll fix it and retry.

[Edits test, reruns]

GATE PASSED: tests
```

### Intent Drift

```
[Executing phase: verify]

INTENT CHECK:
  + All endpoints rate limited ✓
  + Returns 429 when exceeded ✓
  + Configurable limits per endpoint ✗
  - No breaking changes ✓

DRIFT DETECTED: "Configurable limits" not fully implemented.
Current: Single global limit
Required: Per-endpoint configuration

Options:
1. Rollback and revise plan
2. Add missing feature
3. Accept with note

Claude: I missed per-endpoint configuration. Let me add that...
```

---

## 8. Metrics & Observability

### Execution Metrics

```json
{
  "plan_id": "add-rate-limiting",
  "duration_ms": 45000,
  "phases_executed": 5,
  "gates_checked": 8,
  "gates_passed": 8,
  "gates_failed": 0,
  "rollbacks": 0,
  "intent_satisfaction": 1.0,
  "tokens_used": 12500,
  "model_calls": 15
}
```

### Dashboard Integration

```
ARIA Execution Summary
═══════════════════════════════════════════════════════
Plan: add-rate-limiting
Status: COMPLETE ✓

Phases:     ████████████████████ 5/5
Gates:      ████████████████████ 8/8
Intent:     ████████████████████ 100%

Duration:   45s
Tokens:     12.5k
Commits:    1

Files Changed:
  + src/middleware/rateLimit.js (new)
  ~ src/app.js (modified)
  + tests/rateLimit.test.js (new)
═══════════════════════════════════════════════════════
```

---

## Benefits of Integration

| Without ARIA | With ARIA |
|--------------|-----------|
| LLM might forget requirements | Intent locked at start |
| No verification until end | Gates verify each step |
| No rollback on failure | Checkpoints enable recovery |
| Hope tests pass | Tests required before done |
| Unclear what changed | Full audit trail |
| Manual doc updates | Auto-generated docs |

**ARIA makes Claude Code deterministic and auditable.**
