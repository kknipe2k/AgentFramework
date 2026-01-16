# Ralph: The Autonomous Loop Pattern

## Overview

Ralph is an autonomous execution pattern that enables AI agents to work on complex, multi-step projects over extended periods. Unlike single-session interactions, Ralph provides a framework for **iterative, self-correcting, goal-driven development** where each iteration starts fresh but builds on accumulated progress.

The core insight: AI agents can accomplish large tasks if given a clear goal (PRD), progress tracking, and the ability to iterate with fresh context.

---

## The Problem Ralph Solves

### Traditional AI Session Limitations

```
Session 1: "Add authentication"
├── Agent starts working
├── Context fills up
├── Quality degrades
├── Session ends incomplete
└── Progress lost

Session 2: "Continue authentication"
├── Agent has no memory
├── Must re-explain everything
├── May contradict Session 1
└── Duplicated effort
```

Issues:
1. **Context Exhaustion**: Long tasks exceed context limits
2. **Memory Loss**: Each session starts from zero
3. **No Continuity**: Work doesn't persist across sessions
4. **Drift**: Without clear goals, agents wander

### Ralph's Solution

```
┌─────────────────────────────────────────────────────────┐
│                        PRD.json                         │
│              (Persistent goal definition)               │
└─────────────────────────┬───────────────────────────────┘
                          │
    Iteration 1           │           Iteration N
         │                │                │
         ▼                ▼                ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ Fresh Agent │    │ Fresh Agent │    │ Fresh Agent │
│ + PRD       │    │ + PRD       │    │ + PRD       │
│ + Progress  │───▶│ + Progress  │───▶│ + Progress  │
│ + Learnings │    │ + Learnings │    │ + Learnings │
└─────────────┘    └─────────────┘    └─────────────┘
         │                │                │
         ▼                ▼                ▼
┌─────────────────────────────────────────────────────────┐
│                   Progress.txt                          │
│            (Accumulated work log)                       │
└─────────────────────────────────────────────────────────┘
```

---

## Core Concepts

### 1. Product Requirements Document (PRD)

The PRD is the **source of truth** for what needs to be built. It's a JSON file that:
- Defines the feature being built
- Lists user stories with acceptance criteria
- Tracks completion status
- Prioritizes work

```json
{
  "feature": "User Authentication System",
  "branchName": "feature/auth-system",
  "createdAt": "2024-01-15T10:00:00Z",
  "userStories": [
    {
      "id": "US-001",
      "title": "User can register with email",
      "description": "New users can create an account...",
      "acceptanceCriteria": [
        "Registration form validates email format",
        "Password must be 8+ characters",
        "Duplicate emails rejected",
        "Welcome email sent",
        "Tests pass"
      ],
      "priority": 1,
      "passes": false,
      "notes": ""
    },
    {
      "id": "US-002",
      "title": "User can log in",
      "priority": 2,
      "passes": false
    }
  ]
}
```

### Why PRD Works

1. **Clear Goal**: Agent knows exactly what to build
2. **Measurable Progress**: Stories are either done or not
3. **Prioritization**: Work on most important things first
4. **Persistence**: Survives across sessions
5. **Accountability**: Acceptance criteria define "done"

---

### 2. Fresh Context Per Iteration

Each Ralph iteration starts a **new agent session** with:
- The current PRD
- Recent progress log
- Accumulated learnings
- The next story to work on

```
┌─────────────────────────────────────────────────────────┐
│                ITERATION PROMPT                          │
├─────────────────────────────────────────────────────────┤
│  ## Your Task                                           │
│  Complete the next user story from the PRD.             │
│                                                         │
│  ## Current PRD                                         │
│  [Full PRD JSON]                                        │
│                                                         │
│  ## Progress So Far                                     │
│  [Last 100 lines of progress.txt]                       │
│                                                         │
│  ## Learnings                                           │
│  [Codebase patterns discovered]                         │
│                                                         │
│  ## Current Story: US-002                               │
│  Focus on this story. Mark passes:true when done.       │
└─────────────────────────────────────────────────────────┘
```

### Why Fresh Context Works

1. **No Degradation**: Each iteration has full context capacity
2. **Clean Slate**: Previous mistakes don't compound
3. **Focused**: Only relevant context included
4. **Recoverable**: Bad iteration doesn't ruin everything

---

### 3. Progress Tracking

The `progress.txt` file is an append-only log of all work done:

```markdown
# ARIA-RALPH Progress Log
Started: 2024-01-15 10:00
Feature: User Authentication System

## Learnings

### Architecture Patterns
- Auth middleware goes in /src/middleware
- Use bcrypt for password hashing
- JWT tokens stored in httpOnly cookies

### Testing Patterns
- Mock auth middleware in integration tests
- Use test JWT tokens from /test/fixtures

### Gotchas
- Must set NODE_ENV=test for test database

---

## 2024-01-15 10:05 - Iteration 1 - US-001
- Status: ATTEMPTED
- Duration: 120s
- Created /src/auth/register.ts
- Added validation schemas

## 2024-01-15 10:08 - Iteration 2 - US-001
- Status: ATTEMPTED
- Duration: 95s
- Fixed email validation regex
- Added duplicate email check

## 2024-01-15 10:12 - Iteration 3 - US-001
- Status: COMPLETE
- Duration: 80s
- All tests passing
- Story marked as passes:true
```

### Why Progress Tracking Works

1. **Visibility**: See exactly what happened
2. **Context**: New iterations know what was tried
3. **Debugging**: Trace back to when things broke
4. **Metrics**: Track velocity and patterns

---

### 4. Learnings Accumulation

As Ralph works, it discovers patterns in the codebase and records them:

```markdown
# Learnings

## Architecture Patterns
- Services are in /src/services and export default class
- All database queries go through Prisma client
- Use dependency injection via constructor

## Testing Patterns
- Mock Prisma with jest.mock('@prisma/client')
- Use factories in /test/factories for test data
- Integration tests use test database

## Gotchas
- Must run prisma generate after schema changes
- Auth middleware expects req.user to be set
- Rate limiting is per-IP, not per-user
```

### Why Learnings Work

1. **Transfer**: Knowledge persists across iterations
2. **Efficiency**: Don't rediscover the same patterns
3. **Quality**: Follow existing conventions
4. **Documentation**: Creates useful codebase docs

---

## The Ralph Loop

```
┌─────────────────────────────────────────────────────────┐
│                    START                                 │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│              1. PRE-FLIGHT CHECKS                        │
│  - PRD exists?                                          │
│  - On correct branch?                                   │
│  - Clean git state?                                     │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│              2. CHECK COMPLETION                         │
│  - Any stories left?                                    │
│  - If no: EXIT (done!)                                  │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│              3. SELECT NEXT STORY                        │
│  - Sort by priority                                     │
│  - Pick first incomplete                                │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│              4. BUILD PROMPT                             │
│  - Include PRD                                          │
│  - Include progress (last 100 lines)                    │
│  - Include learnings                                    │
│  - Specify current story                                │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│              5. RUN AGENT                                │
│  - Execute with full prompt                             │
│  - Agent works on story                                 │
│  - Agent updates PRD when done                          │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│              6. LOG PROGRESS                             │
│  - Record iteration outcome                             │
│  - Capture any learnings                                │
│  - Update metrics                                       │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│              7. SLEEP & REPEAT                           │
│  - Brief pause between iterations                       │
│  - Go to step 2                                         │
└─────────────────────────────────────────────────────────┘
```

---

## Configuration

Ralph is configured via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `ARIA_RALPH_AGENT` | `claude` | Agent to use (claude, amp) |
| `ARIA_RALPH_SLEEP` | `5` | Seconds between iterations |
| `ARIA_RALPH_MAX_FAILURES` | `3` | Failures before intervention |
| `ARIA_RALPH_AUTO_PR` | `true` | Create PR on completion |
| `ARIA_RALPH_CHECKPOINT` | `true` | Save checkpoint each iteration |

---

## Example Session

```bash
# Initialize a new feature
$ aria ralph init "User profile management"

# Edit the PRD to add your stories
$ vim .aria/ralph/prd.json

# Run the loop (max 25 iterations)
$ aria ralph run 25

═══════════════════════════════════════════════════════════
          ARIA-RALPH: Autonomous Execution Loop
═══════════════════════════════════════════════════════════

Agent:          claude
Max iterations: 25
PRD:            .aria/ralph/prd.json

Running pre-flight checks...
Pre-flight checks passed

═══════════════════════════════════════════════════════════
                    ITERATION 1 / 25
═══════════════════════════════════════════════════════════
Remaining stories: 4
Next story: US-001
Model: sonnet (type: feature, complexity: 6, failures: 0)

Running agent...
[Agent output...]

✅ Story US-001 marked as complete

═══════════════════════════════════════════════════════════
                    ITERATION 2 / 25
═══════════════════════════════════════════════════════════
Remaining stories: 3
Next story: US-002
...
```

---

## Key Innovations

### 1. Goal-Driven, Not Task-Driven

Traditional: "Do this specific thing"
Ralph: "Achieve this outcome (PRD), however you need to"

The agent has autonomy to figure out how to complete stories.

### 2. Self-Correcting

If an iteration fails:
- Progress is logged
- Next iteration sees what went wrong
- Agent can try a different approach
- After N failures, human is consulted

### 3. Checkpoint & Resume

Ralph can be stopped and resumed:
- Git checkpoints at each iteration
- PRD reflects current state
- Progress log provides context
- Pick up exactly where you left off

### 4. Observable

Everything is logged:
- Each iteration's duration and outcome
- What the agent attempted
- Why things failed
- What was learned

---

## When to Use Ralph

**Good for:**
- Multi-day feature development
- Complex refactoring projects
- Systematic migrations
- Features with clear acceptance criteria

**Not ideal for:**
- Quick one-off tasks
- Exploratory coding
- Tasks without clear requirements
- Highly interactive development

---

## Comparison: Single Session vs Ralph

| Aspect | Single Session | Ralph |
|--------|---------------|-------|
| Duration | Minutes to hours | Hours to days |
| Context | Fixed, degrades | Fresh each iteration |
| Progress | In-memory | Persisted |
| Recovery | Start over | Resume from checkpoint |
| Goal | Implicit | Explicit (PRD) |
| Verification | Manual | Integrated |
| Learnings | Lost | Accumulated |

---

## Key Takeaways

1. **PRD is Essential**: Clear, measurable goals drive everything
2. **Fresh Context Scales**: Iteration beats one long session
3. **Progress Must Persist**: Log everything, lose nothing
4. **Learnings Compound**: Each iteration is smarter than the last
5. **Observable is Debuggable**: See what happened, fix what broke

Ralph transforms AI coding assistance from a tool you use into a system that works for you—autonomously, reliably, and at scale.
