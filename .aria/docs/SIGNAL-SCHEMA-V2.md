# ARIA Signal Schema v2 - Full Traceability

## Design Goals

1. **Complete Lineage** - Track every tool call, skill load, agent spawn from start to finish
2. **Failure Forensics** - When something fails, see exactly what, when, why, and what happened next
3. **Duration Tracking** - Know how long each operation took
4. **Rich Context** - File previews, command outputs, error messages
5. **Correlation** - Link related events (pre/post, parent/child, retry chains)

---

## Signal Types

### 1. Tool Signals (Read, Edit, Write, Bash, Glob, Grep, Task)

```json
{
  "id": "sig-1705512345678",
  "type": "tool",
  "event": "pre|post",
  "timestamp": "2026-01-17T10:00:00.000Z",
  "duration_ms": null,

  "tool": {
    "name": "Read|Edit|Write|Bash|Glob|Grep|Task",
    "input": {
      "file_path": "/path/to/file.ts",
      "command": "npm test",
      "pattern": "*.ts",
      "subagent_type": "Explore"
    }
  },

  "result": {
    "success": true|false,
    "exit_code": 0,
    "error": null|"Error message",
    "output_preview": "First 500 chars of output...",
    "lines_read": 150,
    "files_matched": 12,
    "bytes_written": 1024
  },

  "context": {
    "type": "skill|framework|code|search|verify|commit|subagent",
    "name": "planning|executing|tdd|...",
    "file_category": "source|test|config|doc|skill|template",
    "parent_signal_id": "sig-xxx"
  },

  "correlation": {
    "pre_signal_id": "sig-xxx",
    "retry_of": "sig-xxx",
    "retry_count": 0
  }
}
```

### 2. Skill Signals

```json
{
  "id": "skill-1705512345678",
  "type": "skill",
  "event": "loaded|completed|failed",
  "timestamp": "2026-01-17T10:00:00.000Z",
  "duration_ms": 45000,

  "skill": {
    "name": "planning",
    "path": ".aria/skills/planning.md",
    "version": "1.0.0",
    "mode": "STANDARD"
  },

  "result": {
    "success": true|false,
    "output_summary": "Created plan with 5 tasks",
    "artifacts": [".aria/state/current-plan.json"],
    "error": null
  }
}
```

### 3. Agent/Subagent Signals

```json
{
  "id": "agent-1705512345678",
  "type": "agent",
  "event": "spawned|completed|failed",
  "timestamp": "2026-01-17T10:00:00.000Z",
  "duration_ms": 120000,

  "agent": {
    "type": "Explore|analyzer|implementer|verify-app",
    "prompt_preview": "First 200 chars of prompt...",
    "prompt_length": 1500,
    "model": "haiku|sonnet|opus"
  },

  "result": {
    "success": true|false,
    "output_preview": "First 500 chars of result...",
    "output_length": 5000,
    "files_touched": ["src/main.ts", "src/utils.ts"],
    "tools_used": {"Read": 5, "Grep": 3, "Edit": 2},
    "error": null
  },

  "context": {
    "parent_skill": "executing",
    "task_id": 3,
    "task_title": "Implement retry logic"
  }
}
```

### 4. Decision Signals

```json
{
  "id": "dec-1705512345678",
  "type": "decision",
  "timestamp": "2026-01-17T10:00:00.000Z",

  "decision": {
    "action": "Add retry wrapper to API client",
    "context": "Read utils/retry.ts, saw 3 similar patterns",
    "rationale": "Consistency with existing codebase",
    "alternatives": ["Custom retry logic", "No retry", "External library"],
    "confidence": 0.85
  },

  "supporting_signals": ["sig-123", "sig-124", "sig-125"],
  "verified": false,
  "verification_signal": null
}
```

### 5. Verification Signals

```json
{
  "id": "verify-1705512345678",
  "type": "verify",
  "event": "started|passed|failed",
  "timestamp": "2026-01-17T10:00:00.000Z",
  "duration_ms": 15000,

  "verify": {
    "command": "npm test",
    "type": "unit|integration|lint|typecheck|e2e"
  },

  "result": {
    "success": true|false,
    "exit_code": 0|1,
    "tests_passed": 45,
    "tests_failed": 2,
    "tests_skipped": 0,
    "coverage": 78.5,
    "output_preview": "First 1000 chars...",
    "failing_tests": [
      {"name": "auth.test.ts > login > should validate", "error": "Expected 200, got 401"}
    ]
  },

  "triggered_by": "task_completion|pre_commit|manual"
}
```

### 6. Error/Failure Signals

```json
{
  "id": "error-1705512345678",
  "type": "error",
  "timestamp": "2026-01-17T10:00:00.000Z",

  "error": {
    "category": "tool_failure|test_failure|hook_block|timeout|permission",
    "message": "Command failed with exit code 1",
    "stack_preview": "First 500 chars of stack trace...",
    "tool": "Bash",
    "command": "npm test"
  },

  "context": {
    "signal_id": "sig-xxx",
    "skill": "executing",
    "task_id": 3,
    "consecutive_failures": 2
  },

  "resolution": {
    "action": "retry|escalate|skip|abort",
    "next_signal_id": "sig-yyy",
    "user_intervention": false
  }
}
```

### 7. HITL Signals

```json
{
  "id": "hitl-1705512345678",
  "type": "hitl",
  "event": "prompted|responded",
  "timestamp": "2026-01-17T10:00:00.000Z",
  "wait_duration_ms": 30000,

  "hitl": {
    "checkpoint": "Delete legacy auth module",
    "reason": "destructive_action|risky_change|user_requested",
    "options": ["yes", "no", "explain"]
  },

  "response": {
    "choice": "yes",
    "user_input": null,
    "timestamp": "2026-01-17T10:00:30.000Z"
  },

  "context": {
    "skill": "executing",
    "task_id": 5,
    "blocked_signal_id": "sig-xxx"
  }
}
```

### 8. Session Signals

```json
{
  "id": "session-1705512345678",
  "type": "session",
  "event": "started|ended|context_refresh",
  "timestamp": "2026-01-17T10:00:00.000Z",

  "session": {
    "id": "session-20260117-100000",
    "mode": "STANDARD",
    "workflow": "BUILD|MODIFY|RESEARCH"
  },

  "metrics": {
    "duration_ms": 3600000,
    "signals_total": 150,
    "tools_used": {"Read": 45, "Edit": 20, "Bash": 15},
    "files_touched": 12,
    "decisions_made": 8,
    "errors_encountered": 3,
    "hitl_checkpoints": 2
  },

  "tokens": {
    "input": 125000,
    "output": 35000,
    "cache_read": 50000,
    "cache_write": 75000
  },

  "cost": {
    "total_usd": 2.45,
    "by_model": {
      "opus": 2.10,
      "sonnet": 0.30,
      "haiku": 0.05
    }
  }
}
```

---

## Timeline View Requirements

The dashboard timeline should show:

```
[10:00:00] SESSION STARTED - STANDARD mode, MODIFY workflow
           │
[10:00:05] SKILL loaded: planning (.aria/skills/planning.md)
           │
[10:00:10] READ .aria/project-context.md (245 lines)
           │  Context: framework, understanding codebase
           │
[10:00:15] GREP "authentication" in src/ (12 files matched)
           │  Context: search, finding auth code
           │
[10:00:20] READ src/auth/login.ts (89 lines)
           │  Preview: "export async function login(credentials)..."
           │
[10:00:25] DECISION: Use existing auth pattern
           │  Confidence: 0.85
           │  Rationale: Consistency with 3 similar implementations
           │
[10:00:30] SKILL loaded: executing (.aria/skills/executing.md)
           │
[10:00:35] AGENT spawned: implementer
           │  Task: "Add retry logic to login"
           │  Model: sonnet
           │
[10:01:30] │  EDIT src/auth/login.ts (+15 lines)
           │  │  Added retry wrapper at line 45
           │  │
[10:01:45] │  EDIT src/auth/login.test.ts (+25 lines)
           │  │  Added retry tests
           │  │
[10:02:00] AGENT completed (85s)
           │  Files: 2, Tools: {Read: 3, Edit: 2}
           │
[10:02:05] VERIFY: npm test
           │
[10:02:20] VERIFY FAILED (15s)
           │  Exit code: 1
           │  Failed: auth.test.ts > retry > should retry 3 times
           │  Error: "Expected 3 retries, got 2"
           │
[10:02:25] ERROR logged
           │  Category: test_failure
           │  Consecutive failures: 1
           │
[10:02:30] AGENT spawned: implementer (RETRY)
           │  Task: "Fix retry count bug"
           │  Retry of: agent-xxx
           │
[10:03:00] AGENT completed (30s)
           │
[10:03:05] VERIFY: npm test
           │
[10:03:20] VERIFY PASSED (15s)
           │  Tests: 47 passed, 0 failed
           │
[10:03:25] COMMIT: abc123 "Add retry logic to auth"
           │  Files: 2, Decisions linked: 1
           │
[10:03:30] SESSION metrics
           │  Duration: 3m 30s
           │  Tokens: 15K in, 4K out
           │  Cost: $0.45
```

---

## Implementation Plan

1. **Rewrite aria-rails.sh** - Capture rich data on pre/post hooks
2. **Add post-tool result parsing** - Extract exit codes, line counts, errors
3. **Track durations** - Store pre timestamp, calculate on post
4. **Read Claude's native logs** - Get tokens/cost from JSONL
5. **Update dashboard** - Rich timeline with expandable details
6. **Add failure chain tracking** - Link retries, escalations
