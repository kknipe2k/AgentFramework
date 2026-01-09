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
- yq (YAML processor, for rails)
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

### 3. Create Your First Feature

```bash
# Initialize a new feature
./.aria/ralph/ralph.sh init "Add user authentication"

# This creates:
# - .aria/ralph/prd.json (edit this!)
# - .aria/ralph/progress.txt
# - .aria/ralph/prompt.md
```

### 4. Edit the PRD

Open `.aria/ralph/prd.json` and define your user stories:

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

### 5. Run ARIA

```bash
# Run the autonomous loop (max 25 iterations)
./.aria/ralph/ralph.sh run 25

# Or use the engine wrapper
./.aria/aria-engine.sh ralph run 25
```

### 6. Monitor Progress

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
| `rails [file]` | Execute YAML rails | `aria rails .aria/rails/security.yaml` |
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
| `init "description"` | Initialize new feature |
| `run [max_iterations]` | Run the autonomous loop |
| `status` | Show current status |
| `help` | Show help |

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

### Standard Feature Development

```bash
# 1. Initialize
./.aria/ralph/ralph.sh init "My new feature"

# 2. Edit PRD with your stories
vim .aria/ralph/prd.json

# 3. Run ARIA
./.aria/ralph/ralph.sh run 50

# 4. Review PR when complete
# (auto-created if ARIA_RALPH_AUTO_PR=true)
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

### Rail File Structure

```yaml
# .aria/rails/my-rails.yaml
rails:
  - id: unique_id
    description: "Human-readable description"
    type: hard  # hard (block) or soft (warn)
    check: |
      # Bash command that returns 0 (pass) or non-0 (fail)
      ! grep -r "TODO" src/
    message: "Error message shown when rail fails"
    auto_fix: |
      # Optional: command to fix the issue
      echo "Run: remove-todos.sh"
```

### Example: Security Rails

```yaml
# .aria/rails/security.yaml
rails:
  - id: no_eval
    description: "Block use of eval()"
    type: hard
    check: |
      ! grep -r "eval(" src/ --include="*.ts" --include="*.js"
    message: "eval() is a security risk. Use safer alternatives."

  - id: no_innerhtml
    description: "Block innerHTML assignments"
    type: hard
    check: |
      ! grep -r "innerHTML\s*=" src/ --include="*.ts" --include="*.tsx"
    message: "innerHTML can cause XSS. Use textContent or React."

  - id: sql_parameterized
    description: "Ensure parameterized SQL queries"
    type: soft
    check: |
      # Check for string concatenation in SQL
      ! grep -rE "\.(query|execute)\s*\(\s*['\`].*\+.*['\`]" src/
    message: "Use parameterized queries to prevent SQL injection."
```

### Running Rails

```bash
# Run specific rail file
./.aria/aria-engine.sh rails .aria/rails/security.yaml

# Run all rails in directory
for f in .aria/rails/*.yaml; do
  ./.aria/rails-executor.sh "$f"
done
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

### 7. Custom Notifications
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

## Next Steps

1. Try the Quick Start above
2. Read the deep-dive docs for concepts you want to understand
3. Customize configuration for your project
4. Create project-specific agents and rails
5. Iterate and improve your PRD writing skills

Happy autonomous coding!
