# ARIA User Guide

## Quick Start

Get ARIA running in 5 minutes.

### 1. Prerequisites

```bash
# Required
- Git repository (initialized)
- Claude CLI installed and authenticated
- jq (JSON processor)
- bash 4.0+

# Optional but recommended
- Node.js/npm (for JS/TS projects)
- Python 3 (for Python projects)
```

### 2. Initialize ARIA

```bash
# Navigate to your project
cd your-project

# Create ARIA directory structure
mkdir -p .aria/{ralph,rails,logs,state,agents}

# Copy ARIA scripts (or clone from repo)
# ... scripts should be in .aria/

# Make scripts executable
chmod +x .aria/*.sh .aria/ralph/*.sh
```

### 2a. For Existing Projects: Run Discovery First

If you're adding ARIA to an existing/mature project, run discovery to build context:

```bash
# Full onboarding: scan → answer questions → build context
./.aria/discover.sh full .

# Or step by step:
./.aria/discover.sh scan .        # Scan codebase
./.aria/discover.sh qa            # Answer questions about the project
./.aria/discover.sh build         # Generate project-context.md
```

**What discovery does:**
- Scans tech stack (Node, Python, frameworks, ORMs)
- Maps directory structure and patterns
- Assesses test coverage
- Generates project-specific questions
- Creates `.aria/project-context.md` for AI to reference

**After discovery, edit project-context.md to add:**
- "Don't Touch" areas (legacy code, security-critical files)
- Special instructions (naming conventions, deployment notes)
- Tribal knowledge that AI should know

### 2b. For Greenfield Projects: Start Planning Directly

New projects can skip discovery and go straight to planning:

```bash
./.aria/ralph/ralph.sh plan "Build a REST API for user management"
```

---

### 3. Create Your First Feature (Planning-First Workflow)

```bash
# Start the planning agent
./.aria/ralph/ralph.sh plan "Add user authentication"

# The planner will:
# 1. Create a plan with tasks, risks, and questions
# 2. Present it for your review
# 3. You choose: [a]pprove / [r]evise / [e]dit / [c]ancel
# 4. Loop until you approve
```

### 4. Review and Approve the Plan

The planner presents a structured plan:

```
GOAL: Add user authentication

TASKS:
  [pending] 1. Create user model and database schema (simple)
  [pending] 2. Implement registration endpoint (medium)
  [pending] 3. Implement login endpoint (medium)
  [pending] 4. Add JWT token generation (medium)
  [pending] 5. Add tests for all endpoints (simple)

RISKS:
  - Database schema may conflict with existing models

QUESTIONS (need your input):
  ? Should passwords use bcrypt or argon2?
  ? What should token expiration be?

Options:
  [a]pprove  - Start execution
  [r]evise   - Provide feedback for revision
  [e]dit     - Edit plan directly
  [c]ancel   - Abort planning
```

### 5. (Legacy) Manual PRD Mode

For backward compatibility, you can still use manual PRDs:

```json
{
  "feature": "Add user authentication",
  "branchName": "feature/auth",
  "userStories": [
    {
      "id": "US-001",
      "title": "User registration endpoint",
      "description": "Create POST /api/register endpoint",
      "acceptanceCriteria": [
        "Validates email format",
        "Hashes password with bcrypt",
        "Returns 201 on success",
        "Returns 400 on validation error",
        "Tests pass"
      ],
      "priority": 1,
      "passes": false
    },
    {
      "id": "US-002",
      "title": "User login endpoint",
      "description": "Create POST /api/login endpoint",
      "acceptanceCriteria": [
        "Verifies email/password",
        "Returns JWT token",
        "Returns 401 on bad credentials",
        "Tests pass"
      ],
      "priority": 2,
      "passes": false
    }
  ]
}
```

### 6. Run Execution

```bash
# Run with approved plan (requires plan approval first)
./.aria/ralph/ralph.sh run

# Or force run without plan check (legacy mode)
./.aria/ralph/ralph.sh run --force

# Or use the engine wrapper
./.aria/aria-engine.sh ralph run
```

**What happens during execution:**
- Executor runs approved tasks in order
- If stuck (3 consecutive failures), escalates to Planning Agent
- Planning Agent asks you: [r]eplan / [s]kip / [a]bort
- You decide how to proceed, execution continues

### 7. Monitor Progress

In another terminal:
```bash
# Watch progress
tail -f .aria/ralph/progress.txt

# Check status
./.aria/ralph/ralph.sh status
```

---

## Command Reference

### ARIA Engine Commands

```bash
# Main entry point
./.aria/aria-engine.sh <command> [args]
```

| Command | Description | Example |
|---------|-------------|---------|
| `verify [level]` | Run verification | `aria verify standard` |
| `rails [file]` | Execute JSON rails | `aria rails .aria/rails/safety.json` |
| `agent [name]` | Run an agent | `aria agent verify` |
| `hitl <cmd>` | HITL operations | `aria hitl respond "fix the tests"` |
| `checkpoint [name]` | Save checkpoint | `aria checkpoint pre-refactor` |
| `rollback <target>` | Rollback changes | `aria rollback checkpoint pre-refactor` |
| `pr create` | Create pull request | `aria pr create` |
| `model <cmd>` | Model operations | `aria model status` |
| `ralph <cmd>` | Ralph operations | `aria ralph run` |

### Ralph Commands

```bash
./.aria/ralph/ralph.sh <command> [args]
```

| Command | Description |
|---------|-------------|
| `plan "requirements"` | Start planning loop (get approval first) |
| `run [--force]` | Run execution (requires approved plan) |
| `init "description"` | Initialize new PRD (legacy mode) |
| `status` | Show current status (plan + PRD) |
| `help` | Show help |

### Planner Commands

```bash
./.aria/planner/planner.sh <command> [args]
```

| Command | Description |
|---------|-------------|
| `plan "requirements"` | Start planning loop with HITL approval |
| `replan "blocker"` | Re-plan due to execution blocker |
| `status` | Show current plan status |
| `approve` | Mark current plan as approved |
| `reset` | Clear current plan |

### Discovery Commands (for existing projects)

```bash
./.aria/discover.sh <command> [args]
```

| Command | Description |
|---------|-------------|
| `scan [dir]` | Scan codebase, generate questions |
| `qa` | Answer questions about the project |
| `build` | Build project-context.md from scan + answers |
| `full [dir]` | Complete onboarding (scan → qa → build) |
| `status` | Show discovery status |

### Design Notes Commands

```bash
./.aria/design-notes.sh <command> [args]
```

| Command | Description |
|---------|-------------|
| `checkpoint <title> <content>` | Pause for design review |
| `concern <title> <content> [severity]` | Flag a concern (low/medium/high) |
| `assumption <title> <content>` | Log an assumption |
| `research <topic> <findings>` | Log research findings |
| `decision <title> <chosen> <alts> <reason>` | Log a decision |
| `show [lines]` | Show recent notes |
| `clear` | Start fresh session |

### Model Selector Commands

```bash
./.aria/model-selector.sh <command> [args]
```

| Command | Description |
|---------|-------------|
| `select <task> [story_id]` | Select best model |
| `recommend <task>` | Show recommendation with reasoning |
| `status` | Show usage and budget |
| `budget [amount]` | Get/set budget |
| `stats` | Show learning statistics |
| `reset` | Reset usage tracking |
| `learn-reset` | Reset learning data |

### HITL Commands

```bash
./.aria/hitl.sh <command> [args]
```

| Command | Description |
|---------|-------------|
| `help <reason> <context>` | Request help |
| `confirm <action> <details>` | Request confirmation |
| `respond <message>` | Respond to pending request |
| `approve` | Approve pending request |
| `reject <reason>` | Reject pending request |
| `status` | Show pending requests |

### Git Operations Commands

```bash
./.aria/git-ops.sh <command> [args]
```

| Command | Description |
|---------|-------------|
| `checkpoint [name]` | Save checkpoint |
| `checkpoint list` | List checkpoints |
| `rollback checkpoint <name>` | Rollback to checkpoint |
| `rollback commits <n>` | Rollback N commits |
| `rollback success` | Rollback to last success |
| `pr create` | Create pull request |
| `pr status` | Check PR status |

---

## Workflows

### Planning-First Development (Recommended)

```bash
# 1. Start planning with your requirements
./.aria/ralph/ralph.sh plan "Add user dashboard with analytics"

# 2. Review the generated plan
#    - Check tasks make sense
#    - Answer any questions
#    - Provide feedback if needed
#    Options: [a]pprove / [r]evise / [e]dit / [c]ancel

# 3. Once approved, run execution
./.aria/ralph/ralph.sh run

# 4. If execution gets stuck (3 failures):
#    - Planner asks: [r]eplan / [s]kip / [a]bort
#    - Choose how to proceed

# 5. Review PR when complete
```

### Architecture Overview

```
┌──────────────────────────────────────────────────────────┐
│                    User (HITL)                           │
│         approve / revise / provide guidance              │
└──────────────────────┬───────────────────────────────────┘
                       │
         ┌─────────────▼─────────────┐
         │     Planning Agent        │
         │  (.aria/planner/)         │
         │                           │
         │  - Creates plans          │
         │  - Gets HITL approval     │
         │  - Handles replanning     │
         └─────────────┬─────────────┘
                       │ approved plan
         ┌─────────────▼─────────────┐
         │    Execution Agent        │
         │  (.aria/ralph/)           │
         │                           │
         │  - Executes tasks         │
         │  - Tracks progress        │
         │  - Escalates when stuck ──┼──► back to Planner
         └───────────────────────────┘
```

### Onboarding Existing Projects

```bash
# 1. Run full discovery
./.aria/discover.sh full /path/to/existing-project

# 2. Answer the questions:
#    - What is the main purpose?
#    - What areas should NOT be modified?
#    - What's the deployment process?

# 3. Review and edit project-context.md
#    - Add "Don't Touch" areas
#    - Add special instructions
#    - Add tribal knowledge

# 4. Now plan features as usual
./.aria/ralph/ralph.sh plan "Add new dashboard"
```

### Legacy PRD-Based Development

```bash
# 1. Initialize (creates PRD template)
./.aria/ralph/ralph.sh init "My new feature"

# 2. Edit PRD with your stories
vim .aria/ralph/prd.json

# 3. Run ARIA (skip plan check)
./.aria/ralph/ralph.sh run --force

# 4. Review PR when complete
```

### Manual Verification

```bash
# Quick check (types + lint)
./.aria/aria-engine.sh verify quick

# Standard check (+ tests + build)
./.aria/aria-engine.sh verify standard

# Full check (+ integration + e2e)
./.aria/aria-engine.sh verify full
```

### Handling HITL Requests

When ARIA needs human help:

```bash
# Check what's needed
./.aria/hitl.sh status

# Provide guidance
./.aria/hitl.sh respond "Try using the UserService instead of direct DB access"

# Or approve/reject
./.aria/hitl.sh approve
./.aria/hitl.sh reject "This approach won't work because..."
```

### Recovery from Failures

```bash
# List checkpoints
./.aria/git-ops.sh checkpoint list

# Rollback to specific checkpoint
./.aria/git-ops.sh rollback checkpoint iter_5

# Rollback last 3 commits
./.aria/git-ops.sh rollback commits 3

# Rollback to last known good state
./.aria/git-ops.sh rollback success
```

### Budget Management

```bash
# Check current usage
./.aria/model-selector.sh status

# Set budget
./.aria/model-selector.sh budget 20.00

# Check remaining
./.aria/model-selector.sh remaining

# Reset (start fresh)
./.aria/model-selector.sh reset
```

---

## IDE & Workflow Integrations

ARIA integrates with your development environment at multiple levels for seamless workflow.

### Claude Code Slash Commands

ARIA includes slash commands for use within Claude Code sessions.

**Available Commands:**

| Command | Description |
|---------|-------------|
| `/aria [command]` | Run any ARIA command |
| `/aria-verify [level]` | Run verification pipeline |
| `/aria-start [description]` | Initialize new feature |
| `/aria-status` | Show comprehensive status |

**Usage Examples:**

```
# In Claude Code session
/aria ralph run 25
/aria verify standard
/aria-start "Add user dashboard"
/aria-status
```

**Location:** `.claude/commands/`

The slash commands provide natural integration when you're already in a Claude Code session - no need to switch to terminal.

### VS Code Tasks

ARIA provides VS Code tasks for GUI-based operation.

**Access:** `Cmd/Ctrl+Shift+P` → "Tasks: Run Task" → Select ARIA task

**Available Tasks:**

| Task | Description |
|------|-------------|
| ARIA: Verify Quick | Types + lint check |
| ARIA: Verify Standard | Full test suite |
| ARIA: Verify Full | Including E2E tests |
| ARIA: Ralph Run | Start autonomous loop |
| ARIA: Ralph Status | Show current status |
| ARIA: Ralph Init | Initialize new feature |
| ARIA: Model Status | Show budget/usage |
| ARIA: Learning Stats | Show model learning |
| ARIA: Save Checkpoint | Create git checkpoint |
| ARIA: List Checkpoints | Show available checkpoints |
| ARIA: Rollback to Checkpoint | Restore checkpoint |
| ARIA: Create PR | Create pull request |
| ARIA: HITL Status | Show pending requests |
| ARIA: HITL Respond | Respond to request |
| ARIA: HITL Approve | Approve pending |

**Keyboard Shortcuts (optional):**

Add to `keybindings.json`:
```json
{
  "key": "cmd+shift+v",
  "command": "workbench.action.tasks.runTask",
  "args": "ARIA: Verify Standard"
},
{
  "key": "cmd+shift+r",
  "command": "workbench.action.tasks.runTask",
  "args": "ARIA: Ralph Status"
}
```

**Location:** `.vscode/tasks.json`

### Git Hooks

ARIA provides git hooks for automatic verification on commit and push.

**Available Hooks:**

| Hook | Trigger | Verification Level |
|------|---------|-------------------|
| `pre-commit` | Before commit | Quick (types + lint) |
| `pre-push` | Before push | Standard (+ tests + build) |
| `commit-msg` | After message | Message format validation |

**Installation:**

```bash
# Install all ARIA hooks
./.aria/hooks/install.sh install

# Check installation status
./.aria/hooks/install.sh status

# Uninstall hooks
./.aria/hooks/install.sh uninstall
```

**Behavior:**

- **pre-commit:** Runs quick verification. Blocks commit if types or lint fail.
- **pre-push:** Runs standard verification. Blocks push if tests fail.
- **commit-msg:** Validates message format. Suggests conventional commits and story references.

**Bypassing Hooks:**

```bash
# Skip pre-commit hook
git commit --no-verify -m "WIP: work in progress"

# Skip pre-push hook
git push --no-verify
```

**Location:** `.aria/hooks/` (templates), `.git/hooks/` (installed)

### Integration Layers

The recommended setup uses all three integrations:

```
┌─────────────────────────────────────────────────────────┐
│  Layer 1: Git Hooks (Automatic Safety)                  │
│  ├── pre-commit: Quick verify on every commit           │
│  ├── pre-push: Standard verify before push              │
│  └── commit-msg: Message format validation              │
├─────────────────────────────────────────────────────────┤
│  Layer 2: Claude Code Slash Commands (AI Sessions)      │
│  ├── /aria: Full command access                         │
│  ├── /aria-verify: Verification                         │
│  ├── /aria-start: Initialize features                   │
│  └── /aria-status: Dashboard                            │
├─────────────────────────────────────────────────────────┤
│  Layer 3: VS Code Tasks (GUI Access)                    │
│  ├── Quick access via Command Palette                   │
│  ├── Optional keyboard shortcuts                        │
│  └── Visual feedback in terminal panel                  │
├─────────────────────────────────────────────────────────┤
│  Layer 4: Shell Aliases (Terminal Power Users)          │
│  ├── alias aria='./.aria/aria-engine.sh'               │
│  └── alias ralph='./.aria/ralph/ralph.sh'              │
└─────────────────────────────────────────────────────────┘
```

### Quick Setup Script

Run this to set up all integrations:

```bash
#!/bin/bash
# setup-aria-integrations.sh

# Install git hooks
./.aria/hooks/install.sh install

# Add shell aliases (add to your .bashrc or .zshrc)
echo "
# ARIA aliases
alias aria='./.aria/aria-engine.sh'
alias ralph='./.aria/ralph/ralph.sh'
" >> ~/.bashrc

# VS Code tasks are already in .vscode/tasks.json
# Claude Code commands are already in .claude/commands/

echo "ARIA integrations installed!"
echo "  - Git hooks: installed"
echo "  - Shell aliases: added to ~/.bashrc (source it or restart shell)"
echo "  - VS Code tasks: available in Command Palette"
echo "  - Claude Code: use /aria commands in sessions"
```

---

## Configuration

### Environment Variables

Create a `.env` file or export these variables:

```bash
# Ralph Configuration
export ARIA_RALPH_AGENT=claude           # claude or amp
export ARIA_RALPH_SLEEP=5                # Seconds between iterations
export ARIA_RALPH_MAX_FAILURES=3         # Failures before HITL
export ARIA_RALPH_AUTO_PR=true           # Auto-create PR
export ARIA_RALPH_CHECKPOINT=true        # Save checkpoints

# Model Selection
export ARIA_RALPH_AUTO_MODEL=true        # Intelligent model selection
export ARIA_RALPH_FORCE_MODEL=           # Force: opus, sonnet, or haiku
export ARIA_MODEL_BUDGET=10.00           # Budget in dollars

# HITL Configuration
export ARIA_HITL_TIMEOUT=3600            # Timeout in seconds
export ARIA_HITL_NOTIFY="terminal,desktop,sound"
export ARIA_HITL_SLACK_WEBHOOK=          # Optional Slack webhook
export ARIA_HITL_EMAIL=                  # Optional email address
```

### Project-Specific Configuration

Create `.aria/config.sh` for project-specific settings:

```bash
#!/bin/bash
# Project-specific ARIA configuration

# Verification commands (override defaults)
export ARIA_CMD_TYPECHECK="npm run typecheck"
export ARIA_CMD_LINT="npm run lint"
export ARIA_CMD_TEST="npm test"
export ARIA_CMD_BUILD="npm run build"
export ARIA_CMD_E2E="npm run test:e2e"

# Project type (auto-detected if not set)
export ARIA_PROJECT_TYPE=node

# Custom verification level defaults
export ARIA_DEFAULT_VERIFY_LEVEL=standard
```

---

## Writing Good PRDs

### Story Structure

```json
{
  "id": "US-001",
  "title": "Short, descriptive title",
  "description": "Detailed description of what needs to be built",
  "acceptanceCriteria": [
    "Specific, testable criterion 1",
    "Specific, testable criterion 2",
    "All tests pass",
    "No type errors"
  ],
  "priority": 1,
  "passes": false,
  "notes": "Any additional context"
}
```

### Best Practices

1. **Atomic Stories**: Each story should be completable in 1-3 iterations
2. **Clear Criteria**: Acceptance criteria should be verifiable
3. **Priority Order**: Put foundational work first (priority 1)
4. **Include Tests**: Always include "Tests pass" as a criterion
5. **Be Specific**: "Add login endpoint" not "Add auth stuff"

### Anti-Patterns

```json
// BAD: Too vague
{
  "title": "Make authentication work",
  "acceptanceCriteria": ["Auth works"]
}

// GOOD: Specific and testable
{
  "title": "Create JWT-based login endpoint",
  "acceptanceCriteria": [
    "POST /api/login accepts email and password",
    "Returns JWT token on valid credentials",
    "Returns 401 with message on invalid credentials",
    "Token expires in 24 hours",
    "Tests cover success and failure cases"
  ]
}
```

---

## Creating Custom Agents

### Agent File Structure

Create agents in `.claude/agents/`:

```markdown
---
name: my-agent
description: What this agent does
tools: [Read, Edit, Bash]  # Available tools
model: sonnet              # Preferred model
---

# Agent Name

[Instructions for the agent]

## Context
[What the agent needs to know]

## Task
[What the agent should do]

## Output
[Expected output format]
```

### Example: Database Migration Agent

```markdown
---
name: migrate
description: Create and run database migrations
tools: [Read, Edit, Bash, Glob]
model: sonnet
---

# Database Migration Agent

Create safe, reversible database migrations.

## Context
- Using Prisma ORM
- Migrations in /prisma/migrations
- Schema in /prisma/schema.prisma

## Task
1. Analyze requested schema change
2. Update schema.prisma
3. Generate migration: npx prisma migrate dev
4. Verify migration applied correctly

## Output
Report migration name and any manual steps needed.
```

### Running Custom Agents

```bash
# Run by name
./.aria/aria-engine.sh agent migrate

# Or directly
./.aria/agent-runner.sh migrate
```

---

## Creating Custom Rails

### Rail File Structure (JSON)

```json
{
  "rails": [
    {
      "id": "unique_id",
      "description": "Human-readable description",
      "type": "hard",
      "check": "bash command that returns 0 (pass) or non-0 (fail)",
      "message": "Error message shown when rail fails"
    }
  ]
}
```

- **type: "hard"** - Blocks execution if check fails
- **type: "soft"** - Warns but allows continuation

### Example: Security Rails

```json
{
  "rails": [
    {
      "id": "no_secrets",
      "description": "Check for secrets in staged files",
      "type": "hard",
      "check": "test -z \"$(git diff --cached 2>/dev/null)\" || ! git diff --cached 2>/dev/null | grep -qE '(api[_-]?key|secret|password|token)\\s*[=:]\\s*[A-Za-z0-9_-]{16,}'",
      "message": "Potential secret detected in staged changes."
    },
    {
      "id": "no_eval",
      "description": "Block use of eval()",
      "type": "hard",
      "check": "! grep -r 'eval(' src/ --include='*.ts' --include='*.js' 2>/dev/null",
      "message": "eval() is a security risk. Use safer alternatives."
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

### Running Rails

```bash
# Run specific rail file
./.aria/rails-executor.sh .aria/rails/safety.json

# Run via engine
./.aria/aria-engine.sh rails .aria/rails/safety.json
```

---

## Troubleshooting

### ARIA Won't Start

```bash
# Check prerequisites
which claude  # Claude CLI installed?
which jq      # jq installed?
git status    # In a git repo?

# Check permissions
ls -la .aria/*.sh  # Should be executable

# Check PRD exists
cat .aria/ralph/prd.json
```

### Iterations Keep Failing

```bash
# Check progress log
tail -50 .aria/ralph/progress.txt

# Check verification manually
./.aria/aria-engine.sh verify standard

# Check if story is too complex
# Consider breaking into smaller stories
```

### HITL Not Working

```bash
# Check pending requests
./.aria/hitl.sh status

# Check log
cat .aria/logs/hitl.log

# Try responding manually
./.aria/hitl.sh respond "your guidance"
```

### Model Selection Issues

```bash
# Check current state
./.aria/model-selector.sh status

# Check learning data
./.aria/model-selector.sh stats

# Reset if corrupted
./.aria/model-selector.sh reset
./.aria/model-selector.sh learn-reset
```

### Rollback Needed

```bash
# List what's available
./.aria/git-ops.sh checkpoint list

# Rollback
./.aria/git-ops.sh rollback checkpoint <name>

# Nuclear option: rollback all ARIA commits
git log --oneline | head -20  # Find commit to reset to
git reset --hard <commit>
```

---

## Tips and Tricks

### 1. Start Small
Begin with 2-3 simple stories to see how ARIA works before tackling complex features.

### 2. Watch the First Run
Monitor the first few iterations closely to understand the flow:
```bash
# In another terminal
tail -f .aria/ralph/progress.txt
```

### 3. Tune Failure Threshold
If HITL triggers too often, increase the threshold:
```bash
export ARIA_RALPH_MAX_FAILURES=5
```

### 4. Use Checkpoints Liberally
Before any risky operation:
```bash
./.aria/aria-engine.sh checkpoint before_experiment
```

### 5. Keep PRDs Updated
If you manually fix something, update the PRD:
```bash
# Mark story as complete
jq '.userStories[0].passes = true' .aria/ralph/prd.json > tmp && mv tmp .aria/ralph/prd.json
```

### 6. Review Learning Data
Periodically check what ARIA has learned:
```bash
./.aria/model-selector.sh stats
cat .aria/logs/model_learning.json | jq .
```

### 7. Use Meta-Reasoning for Complex Decisions
When facing complex tasks, use the meta-reasoning system:
```bash
source .aria/lib/meta-reasoning.sh
meta_reason "Implement authentication system" "feature" 8
# Output: Model recommendation, confidence score, solution space analysis
```

### 8. Run Offline Learning After Sessions
After completing significant work, run the learning pipeline:
```bash
python .aria/lib/offline-learner.py learn
python .aria/lib/offline-learner.py stats
```

### 9. Use Deep Research for Complex Questions
For thorough web research with oversight:
```bash
# In Claude Code session:
"Research the best practices for implementing rate limiting in Node.js"
# ARIA will use deep-research skill with HITL gates
```

### 10. Custom Notifications
Configure Slack for team visibility:
```bash
export ARIA_HITL_SLACK_WEBHOOK="https://hooks.slack.com/services/xxx"
```

---

## Getting Help

- Check documentation in `.aria/docs/`
- Review progress log for error details
- Use `--help` flag on any script
- File issues at the project repository

---

---

## Offline Reinforcement Learning

ARIA learns from past sessions to improve decision-making over time.

### How It Works

```
SESSION N                         BETWEEN SESSIONS
┌──────────────┐                  ┌──────────────────────┐
│ Execute with │                  │ Learning Pipeline    │
│ current      │──────────────────▶│                      │
│ policy       │   signals.jsonl  │ 1. Extract episodes  │
│              │   decisions.jsonl│ 2. Calculate rewards │
│              │   outcomes       │ 3. Update priors     │
└──────────────┘                  │ 4. Export policy     │
       ▲                          └──────────┬───────────┘
       │                                     │
       └─────────────────────────────────────┘
                  SESSION N+1 uses improved policy
```

### Offline Learner Commands

```bash
# Run learning pipeline
python .aria/lib/offline-learner.py learn

# View current policy
python .aria/lib/offline-learner.py export-policy

# Query for specific recommendation
python .aria/lib/offline-learner.py query feature 7 auth

# View statistics
python .aria/lib/offline-learner.py stats
```

### Meta-Reasoning Functions

```bash
source .aria/lib/meta-reasoning.sh

# Get model recommendation (uses Thompson Sampling)
meta_select_model "feature" 6 "api"
# Output: sonnet|0.78|Learned from 15 past observations

# Full meta-reasoning cycle
meta_reason "Implement retry logic" "feature" 6

# Record outcome for learning
meta_record_outcome "sonnet" "feature" 6 "success" "US-001"
```

### What Gets Learned

| Decision Point | What's Learned | Data Source |
|----------------|----------------|-------------|
| Model selection | Which models succeed for task types | `model_learning.json` |
| Strategy selection | Which approaches work when | `decisions.jsonl` |
| Confidence calibration | Agent over/under-confidence | `decisions.jsonl` |
| Dead-end detection | Patterns that precede failures | `signals.jsonl` |

### Learning Files

| File | Purpose |
|------|---------|
| `.aria/learned/policy.json` | Current learned policy |
| `.aria/learned/priors/model-selection.json` | Beta priors for model × context |
| `.aria/learned/priors/strategy-selection.json` | Beta priors for strategy × context |
| `.aria/learned/history/episodes.jsonl` | Historical (state, action, reward) tuples |

---

## Deep Research

ARIA includes a deep research skill for systematic web research with HITL oversight.

### When to Use

- Complex questions requiring multiple sources
- Need source quality tracking and confidence scores
- Want iterative refinement with human checkpoints

### Workflow

```
STEP 1: Depth Selection (HITL)
        [1] Quick (5-10 min)
        [2] Standard (15-30 min) - RECOMMENDED
        [3] Deep (30-60 min)
        [4] Exhaustive (60+ min)

STEP 2: Strategy Selection (HITL)
        [a] Broad Scan - start wide, narrow
        [b] Focused Drill - specific queries
        [c] Comparative - X vs Y analysis
        [d] Temporal - track changes over time

STEP 3: Query Approval (HITL)
        → Present proposed queries
        → [a]pprove / [e]dit / [c]hange strategy

STEP 4: Search Loop
        → Execute queries (WebSearch tool)
        → Evaluate source quality (A/B/C/D rating)
        → Extract findings with confidence scores

STEP 5: Mid-Research Checkpoint (HITL)
        [c]ontinue / [r]edirect / [d]eepen / [s]ynthesize / [a]bort

STEP 6: Synthesis
        → Generate research-output.json
        → Generate IDEA.md with findings
```

### Confidence Scoring

| Score | Label | Meaning |
|-------|-------|---------|
| 0.9+ | VERY HIGH | Multiple authoritative sources agree |
| 0.7-0.89 | HIGH | Good sources, some corroboration |
| 0.5-0.69 | MEDIUM | Limited sources or some uncertainty |
| 0.3-0.49 | LOW | Single source or quality concerns |
| <0.3 | UNVERIFIED | Treat as hypothesis |

### Outputs

- `.aria/docs/research-output.json` - Full research trace
- `.aria/docs/IDEA.md` - Synthesized findings
- Source quality ratings and confidence scores

---

## Next Steps

1. Try the Quick Start above
2. Read the deep-dive docs for concepts you want to understand
3. Customize configuration for your project
4. Create project-specific agents and rails
5. Iterate and improve your PRD writing skills
6. Run offline learning after sessions to improve over time
7. Use deep research for complex questions requiring oversight

Happy autonomous coding!
