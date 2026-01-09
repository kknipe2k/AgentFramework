# ARIA: Agentic Rail-based Intent Architecture

## Overview

ARIA is a comprehensive orchestration system that combines the best of multiple AI-assisted development paradigms into a unified, production-ready framework. It synthesizes:

- **Boris Cherny's Patterns**: Verification, subagents, structured prompts
- **Ralph's Autonomous Loop**: Fresh context iteration, PRD-driven development
- **Safety Rails**: Hard blocks on dangerous operations
- **Human-in-the-Loop**: Intervention when automation fails
- **Adaptive Learning**: Model selection that improves over time

The result is a system that can autonomously develop features while maintaining safety, quality, and human oversight.

---

## The Complete Picture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         DEVELOPER INTERFACE LAYER                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │ Claude Code │  │  VS Code    │  │ Git Hooks   │  │   Shell     │        │
│  │  /aria      │  │   Tasks     │  │ pre-commit  │  │  Aliases    │        │
│  │  /aria-*    │  │   Panel     │  │ pre-push    │  │  aria/ralph │        │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘        │
└─────────┼────────────────┼────────────────┼────────────────┼────────────────┘
          └────────────────┴────────────────┴────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              ARIA ENGINE                                     │
│                     (Central orchestration layer)                            │
└───────────────────────────────────┬─────────────────────────────────────────┘
                                    │
                 ┌──────────────────┴──────────────────┐
                 │                                     │
                 ▼                                     ▼
┌────────────────────────────┐           ┌────────────────────────────┐
│      PLANNING AGENT        │           │    EXECUTION AGENT         │
│    (.aria/planner/)        │           │    (.aria/ralph/)          │
│                            │           │                            │
│ - Takes requirements       │           │ - Executes approved plans  │
│ - Creates structured plans │  approve  │ - Runs PRD stories         │
│ - Identifies risks/unknowns│ ────────► │ - Tracks progress          │
│ - HITL approval loop       │           │ - Detects stuck state      │
│ - Handles re-planning      │ ◄──────── │ - Escalates to planner     │
│                            │  escalate │                            │
└────────────┬───────────────┘           └─────────────┬──────────────┘
             │                                         │
             └──────────────────┬──────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
┌───────────────┐       ┌───────────────┐       ┌───────────────┐
│ SAFETY RAILS  │       │  INTELLIGENT  │       │     HITL      │
│               │       │    MODEL      │       │               │
│ - Hard blocks │       │   SELECTOR    │       │ - Notify      │
│ - Soft warns  │       │               │       │ - Wait        │
│ - JSON rails  │       │ - Learning    │       │ - Respond     │
│ - Verification│       │ - Budget      │       │ - Approve     │
└───────┬───────┘       │ - Complexity  │       └───────────────┘
        │               └───────────────┘
        │
        ▼
┌───────────────┐       ┌───────────────┐
│   GIT OPS     │       │    AGENTS     │
│               │       │               │
│ - Checkpoint  │       │ - Verify      │
│ - Rollback    │       │ - Simplify    │
│ - Auto-PR     │       │ - Custom      │
└───────────────┘       └───────────────┘
```

### Two-Agent Architecture

ARIA uses a **Planning Agent** and **Execution Agent** working together:

```
User provides requirements
         │
         ▼
┌─────────────────────────────────────────────┐
│              PLANNING AGENT                  │
│                                             │
│  1. Analyze requirements                    │
│  2. Break into atomic tasks                 │
│  3. Identify risks and unknowns             │
│  4. Present plan to HITL                    │
│  5. Loop: approve / revise / edit / cancel  │
└─────────────────────┬───────────────────────┘
                      │ approved plan
                      ▼
┌─────────────────────────────────────────────┐
│             EXECUTION AGENT                  │
│                                             │
│  1. Execute tasks from approved plan        │
│  2. Run safety rails after each task        │
│  3. Track progress and learnings            │
│  4. If stuck (3 failures):                  │
│     → Escalate to Planning Agent            │
│     → Planner asks: replan/skip/abort       │
│  5. Continue until complete                 │
└─────────────────────────────────────────────┘
```

This separation ensures:
- **Plans are reviewed before execution** - No runaway agent
- **Blockers get proper handling** - Not just retry loops
- **User stays in control** - Approve, revise, or abort at any point

---

## Why ARIA is Complete

ARIA addresses every gap in autonomous AI development:

| Gap | Without ARIA | With ARIA |
|-----|--------------|-----------|
| Goals | Vague requests | PRD with acceptance criteria |
| Context | Degrades over time | Fresh each iteration |
| Safety | Hope agent doesn't mess up | Hard rails block bad actions |
| Verification | Manual checking | Automated pipeline |
| Recovery | Start over | Checkpoint and rollback |
| Intervention | Kill and restart | HITL with guidance |
| Cost | Uncontrolled spend | Budget-aware model selection |
| Quality | Variable | Consistent via verification |
| Learning | None | Adapts over time |
| Workflow | Manual CLI | Claude Code, VS Code, Git hooks |

---

## Component Deep Dives

### 1. ARIA Engine (`aria-engine.sh`)

The central command interface that coordinates all ARIA components.

```bash
# Available commands
aria verify [level]      # Run verification (quick/standard/full)
aria rails [rail-file]   # Execute YAML rails
aria agent [agent-name]  # Run a specific agent
aria hitl <command>      # Human-in-the-loop operations
aria checkpoint [name]   # Save git checkpoint
aria rollback [target]   # Rollback to checkpoint
aria pr create           # Create pull request
aria model <command>     # Model selection and tracking
aria ralph <command>     # Ralph loop operations
```

### 2. Ralph Loop Integration

ARIA wraps Ralph with additional capabilities:

```
┌─────────────────────────────────────────────────────────┐
│                   ARIA-RALPH ITERATION                   │
├─────────────────────────────────────────────────────────┤
│  1. Save checkpoint (for rollback safety)               │
│  2. Select model (intelligent, learning-based)          │
│  3. Build prompt (PRD + progress + learnings)           │
│  4. Execute agent (with selected model)                 │
│  5. Check for signals:                                  │
│     - <aria-complete> → All done                        │
│     - <aria-help> → Request human help                  │
│     - <aria-blocked> → Rail triggered                   │
│  6. Run safety verification                             │
│  7. Record learning outcome                             │
│  8. Log progress                                        │
│  9. Check failure threshold → HITL if needed            │
│  10. Sleep and continue                                 │
└─────────────────────────────────────────────────────────┘
```

### 3. Safety Rails System

Rails are defined in JSON and executed automatically:

```json
{
  "rails": [
    {
      "id": "no_secrets",
      "description": "Block commits with secrets",
      "type": "hard",
      "check": "test -z \"$(git diff --cached)\" || ! git diff --cached | grep -qE '(api[_-]?key|secret|password|token)\\s*[=:]\\s*[A-Za-z0-9_-]{16,}'",
      "message": "Detected potential secret in staged changes"
    },
    {
      "id": "no_env_files",
      "description": "Prevent committing .env files",
      "type": "hard",
      "check": "! git diff --cached --name-only | grep -qE '^\\.env'",
      "message": "Cannot commit .env files. Add to .gitignore."
    },
    {
      "id": "no_debug",
      "description": "Check for debug statements",
      "type": "soft",
      "check": "! grep -r 'debugger' src/ 2>/dev/null | grep -v node_modules",
      "message": "Found debugger statements. Consider removing before commit."
    }
  ]
}
```

**Rail Types:**
- **Hard Rails**: Block execution entirely (security issues, breaking changes)
- **Soft Rails**: Warn but allow continuation (style issues, recommendations)

**Rail Executor** parses JSON with `jq` and runs each check:
```bash
aria rails .aria/rails/safety.json
# Executes each rail, blocks or warns as appropriate
```

### 4. Verification Executor

Multi-level verification with auto-detection:

```bash
# Quick: Types + Lint (< 30s)
aria verify quick

# Standard: Quick + Tests + Build (1-5 min)
aria verify standard

# Full: Standard + Integration + E2E (5-15 min)
aria verify full
```

**Auto-Detection:**
- Detects project type (Node, Python, Go, Rust, etc.)
- Finds test framework (Jest, Pytest, etc.)
- Discovers E2E tools (Playwright, Cypress)
- Adapts commands automatically

### 5. Human-in-the-Loop (HITL)

When automation fails, humans are brought in gracefully:

```
┌─────────────────────────────────────────────────────────┐
│                    HITL WORKFLOW                         │
├─────────────────────────────────────────────────────────┤
│  1. Failure threshold reached (3+ consecutive)          │
│  2. HITL system activates                               │
│  3. Notifications sent:                                 │
│     - Terminal bell                                     │
│     - Desktop notification                              │
│     - Sound alert                                       │
│     - Slack message (if configured)                     │
│     - Email (if configured)                             │
│  4. Human reviews situation                             │
│  5. Human provides guidance:                            │
│     - aria hitl respond "guidance text"                 │
│     - aria hitl approve                                 │
│     - aria hitl reject "reason"                         │
│  6. Loop resumes with guidance                          │
└─────────────────────────────────────────────────────────┘
```

**Request Types:**
- `help`: Agent needs assistance
- `confirm`: Approval required for action
- `choice`: Select from options
- `input`: Free-form input needed

### 6. Git Operations

Safe git operations with checkpoint and rollback:

```bash
# Save checkpoint before risky operation
aria checkpoint pre_refactor

# List available checkpoints
aria checkpoint list

# Rollback to checkpoint
aria rollback checkpoint pre_refactor

# Rollback last N commits
aria rollback commits 3

# Rollback to last success
aria rollback success

# Auto-create PR when feature complete
aria pr create
```

**Auto-PR Template:**
```markdown
## Summary
- Implemented user authentication
- Added login/logout endpoints
- Created protected route middleware

## Stories Completed
- [x] US-001: User registration
- [x] US-002: User login
- [x] US-003: Protected routes

## Test Plan
- [ ] Manual login/logout testing
- [ ] Check session persistence
- [ ] Verify protected routes redirect
```

### 7. Intelligent Model Selection

Adaptive model selection based on task and learning:

```
┌─────────────────────────────────────────────────────────┐
│              MODEL SELECTION PIPELINE                    │
├─────────────────────────────────────────────────────────┤
│  1. Check forced model (ARIA_RALPH_FORCE_MODEL)         │
│  2. Query learned data:                                 │
│     - Success rate by task type                         │
│     - Success rate by complexity                        │
│     - Recent outcomes                                   │
│  3. If learned data exists (3+ samples):                │
│     → Use model with best success rate                  │
│  4. Else fall back to heuristics:                       │
│     - Complexity 1-3: haiku                             │
│     - Complexity 4-7: sonnet                            │
│     - Complexity 8-10: opus                             │
│  5. Apply budget constraints:                           │
│     - Budget < 20%: force haiku                         │
│     - Budget < 50%: avoid opus                          │
│  6. Apply failure escalation:                           │
│     - 3+ failures: escalate model tier                  │
│  7. Return selected model                               │
└─────────────────────────────────────────────────────────┘
```

**Learning System:**
```bash
# Record outcome after iteration
aria model outcome sonnet feature 6 success US-001

# View learning statistics
aria model stats
# Output:
# Task Type Success Rates:
# ------------------------------------------------------------
#   bugfix          haiku: -  | sonnet: 80% (5) | opus: 100% (2)
#   feature         haiku: -  | sonnet: 67% (9) | opus: 90% (10)
#   refactoring     haiku: 50% (4) | sonnet: 75% (8) | opus: -
```

### 8. Subagent System

Specialized agents for specific tasks:

```
.claude/agents/
├── verify.md      # Run verification checks
├── simplify.md    # Clean up working code
├── analyze.md     # Analyze codebase questions
└── custom.md      # User-defined agents
```

**Agent Definition:**
```markdown
---
name: verify
description: Run verification and report results
tools: [Bash, Read]
model: haiku
---

# Verify Agent

Run the verification pipeline and report results clearly.

## Instructions
1. Run `aria verify standard`
2. If failures, identify the specific issues
3. Report in structured format

## Output Format
```json
{
  "passed": boolean,
  "failures": ["list of failures"],
  "suggestions": ["how to fix"]
}
```
```

---

## The Complete Workflow

Here's how ARIA handles a complete feature development:

```
User: aria ralph init "Add user dashboard"
       ↓
┌──────────────────────────────────────────────────────────┐
│  PRD created with placeholder story                      │
│  User edits PRD with actual stories                      │
└──────────────────────────────────────────────────────────┘
       ↓
User: aria ralph run 50
       ↓
┌──────────────────────────────────────────────────────────┐
│  ITERATION 1                                             │
│  ├── Checkpoint saved                                    │
│  ├── Model selected: sonnet (feature, complexity 7)      │
│  ├── Agent runs on US-001                                │
│  ├── Safety check: PASSED                                │
│  ├── Learning outcome: recorded                          │
│  └── Status: ATTEMPTED                                   │
└──────────────────────────────────────────────────────────┘
       ↓
┌──────────────────────────────────────────────────────────┐
│  ITERATION 2                                             │
│  ├── US-001 still incomplete                             │
│  ├── Agent continues work                                │
│  ├── Safety check: PASSED                                │
│  ├── US-001 marked complete!                             │
│  └── Learning outcome: SUCCESS                           │
└──────────────────────────────────────────────────────────┘
       ↓
┌──────────────────────────────────────────────────────────┐
│  ITERATION 3                                             │
│  ├── US-002 started                                      │
│  ├── Agent runs                                          │
│  ├── Safety check: FAILED (tests broken)                 │
│  ├── Failure count: 1                                    │
│  └── Learning outcome: FAIL                              │
└──────────────────────────────────────────────────────────┘
       ↓
┌──────────────────────────────────────────────────────────┐
│  ITERATION 4-5: More failures                            │
│  Failure count reaches 3                                 │
│  ↓                                                       │
│  HITL ACTIVATED                                          │
│  ├── Notifications sent                                  │
│  ├── Waiting for human...                                │
│  └── Human provides guidance                             │
└──────────────────────────────────────────────────────────┘
       ↓
┌──────────────────────────────────────────────────────────┐
│  ITERATION 6                                             │
│  ├── Agent has human guidance                            │
│  ├── Fixes issues correctly                              │
│  ├── Safety check: PASSED                                │
│  └── US-002 complete!                                    │
└──────────────────────────────────────────────────────────┘
       ↓
       ... more iterations ...
       ↓
┌──────────────────────────────────────────────────────────┐
│  ALL STORIES COMPLETE                                    │
│  ├── Final checkpoint saved                              │
│  ├── Auto-PR created                                     │
│  └── URL: github.com/user/repo/pull/42                   │
└──────────────────────────────────────────────────────────┘
```

---

## Why This is a Complete System

### 1. Autonomous but Safe
- Can run unattended for hours
- Rails prevent dangerous operations
- HITL catches failures before they compound

### 2. Goal-Driven with Verification
- PRD defines clear success criteria
- Verification ensures code actually works
- Stories aren't "done" until tests pass

### 3. Recoverable and Observable
- Every iteration is checkpointed
- Full progress log for debugging
- Rollback to any point instantly

### 4. Cost-Effective
- Budget tracking prevents runaway costs
- Intelligent model selection uses appropriate tier
- Learning improves efficiency over time

### 5. Human-Integrated
- Humans are consulted when needed
- Guidance improves future iterations
- Not replacing humans, augmenting them

### 6. Developer-Friendly
- Multiple interface options for different workflows
- Integrates with existing tools (VS Code, Git)
- Works within Claude Code sessions naturally

---

## IDE & Workflow Integrations

ARIA provides multiple entry points to fit any development workflow:

### Claude Code Slash Commands

Use ARIA directly within Claude Code sessions:

```
/aria [command]           # Run any ARIA command
/aria-verify [level]      # Run verification pipeline
/aria-start [description] # Initialize new feature
/aria-status              # Comprehensive dashboard
```

**Location:** `.claude/commands/`

**Best for:** AI-assisted development sessions where you're already using Claude.

### VS Code Tasks

GUI access via Command Palette (`Cmd/Ctrl+Shift+P` → "Tasks: Run Task"):

| Task | Action |
|------|--------|
| ARIA: Verify Quick/Standard/Full | Run verification |
| ARIA: Ralph Run/Status/Init | Ralph operations |
| ARIA: Model Status/Learning Stats | Model info |
| ARIA: Save/List/Rollback Checkpoint | Git operations |
| ARIA: HITL Status/Respond/Approve | Human intervention |

**Location:** `.vscode/tasks.json`

**Best for:** Developers who prefer GUI over terminal.

### Git Hooks

Automatic verification on git operations:

| Hook | Trigger | Action |
|------|---------|--------|
| `pre-commit` | `git commit` | Quick verification (types + lint) |
| `pre-push` | `git push` | Standard verification (+ tests) |
| `commit-msg` | After message | Message format validation |

**Installation:**
```bash
./.aria/hooks/install.sh install
```

**Location:** `.aria/hooks/` (templates) → `.git/hooks/` (installed)

**Best for:** Ensuring code quality automatically, team consistency.

### Shell Aliases

Direct terminal access:

```bash
# Add to ~/.bashrc or ~/.zshrc
alias aria='./.aria/aria-engine.sh'
alias ralph='./.aria/ralph/ralph.sh'
```

**Best for:** Power users who prefer terminal workflows.

### Integration Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    ENTRY POINTS                              │
├─────────────────────────────────────────────────────────────┤
│  Git Hooks        → Automatic safety on commit/push         │
│  Claude Code      → Natural AI session integration          │
│  VS Code Tasks    → GUI-based operation                     │
│  Shell Aliases    → Terminal power user access              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    ARIA ENGINE                               │
│              (All paths converge here)                       │
└─────────────────────────────────────────────────────────────┘
```

All integration methods ultimately call the same ARIA scripts, ensuring consistent behavior regardless of how you access the system.

---

## Configuration Summary

```bash
# Environment Variables

# Ralph Configuration
ARIA_RALPH_AGENT=claude       # Agent to use
ARIA_RALPH_SLEEP=5            # Seconds between iterations
ARIA_RALPH_MAX_FAILURES=3     # Failures before HITL
ARIA_RALPH_AUTO_PR=true       # Auto-create PR on completion
ARIA_RALPH_CHECKPOINT=true    # Save checkpoint each iteration

# Model Selection
ARIA_RALPH_AUTO_MODEL=true    # Enable intelligent selection
ARIA_RALPH_FORCE_MODEL=       # Force specific model
ARIA_MODEL_BUDGET=10.00       # Budget in dollars

# HITL
ARIA_HITL_TIMEOUT=3600        # Wait timeout in seconds
ARIA_HITL_NOTIFY=terminal,desktop,sound  # Notification methods
ARIA_HITL_SLACK_WEBHOOK=      # Slack webhook URL
```

---

## Key Takeaways

1. **ARIA = Ralph + Safety + Intelligence**
   - Ralph's autonomy with guardrails
   - Boris Cherny's verification rigor
   - Adaptive learning for efficiency

2. **Complete Lifecycle Coverage**
   - Initialize → Execute → Verify → Ship
   - Every step automated with human fallback

3. **Production-Ready**
   - Not a toy or experiment
   - Real safety, real recovery, real observability

4. **Continuously Improving**
   - Learning system gets smarter
   - Learnings accumulate across runs
   - Model selection optimizes over time

ARIA represents the current state of the art in autonomous AI-assisted development: powerful enough to work independently, safe enough to trust, and smart enough to know when to ask for help.
