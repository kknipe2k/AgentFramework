# ARIA: Agentic Rail-based Intent Architecture

> *Autonomous AI development with safety rails, human oversight, and adaptive learning.*

## What is ARIA?

ARIA is a comprehensive orchestration system for autonomous AI-assisted development. It combines:

- **Ralph's Autonomous Loop** - PRD-driven, fresh-context iteration
- **Boris Cherny's Patterns** - Verification, subagents, structured prompts
- **Safety Rails** - Hard blocks on dangerous operations
- **Human-in-the-Loop** - Intervention when automation fails
- **Adaptive Learning** - Model selection that improves over time

## Quick Start

```bash
# 1. Initialize a feature
./.aria/ralph/ralph.sh init "Add user authentication"

# 2. Edit the PRD with your user stories
vim .aria/ralph/prd.json

# 3. Run the autonomous loop
./.aria/ralph/ralph.sh run 25

# 4. Monitor progress (in another terminal)
tail -f .aria/ralph/progress.txt
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         DEVELOPER INTERFACE LAYER                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │ Claude Code │  │  VS Code    │  │ Git Hooks   │  │   Shell     │        │
│  │  /aria      │  │   Tasks     │  │ pre-commit  │  │  Aliases    │        │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘        │
└─────────┴────────────────┴────────────────┴────────────────┴────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              ARIA ENGINE                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│  Ralph Loop    │  Safety Rails   │  Model Selector  │  Git Ops   │  HITL   │
│  - PRD-driven  │  - Hard blocks  │  - Learning      │  - Checkpoint│ - Notify│
│  - Iterations  │  - Verification │  - Budget        │  - Rollback  │ - Wait  │
│  - Progress    │  - Executors    │  - Complexity    │  - Auto-PR   │ - Guide │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Key Features

| Feature | Description |
|---------|-------------|
| **PRD-Driven** | Clear goals with acceptance criteria |
| **Fresh Context** | Each iteration starts clean |
| **Safety Rails** | Blocks secrets, broken tests, bad patterns |
| **Verification** | Types, lint, tests, build, E2E |
| **Checkpoints** | Rollback to any point |
| **HITL** | Human help when needed |
| **Model Selection** | Opus/Sonnet/Haiku based on task |
| **Learning** | Improves model selection over time |
| **Auto-PR** | Creates PR when feature complete |

## Documentation

| Document | Description |
|----------|-------------|
| **[User Guide](.aria/docs/USER-GUIDE.md)** | Complete usage guide - start here |
| [Architecture](.aria/docs/ARIA-SYSTEM-ARCHITECTURE.md) | System design and components |
| [Boris Cherny Patterns](.aria/docs/BORIS-CHERNY-PATTERNS.md) | Verification and subagent patterns |
| [Ralph Autonomous Loop](.aria/docs/RALPH-AUTONOMOUS-LOOP.md) | PRD-driven iteration pattern |

## Commands

```bash
# Ralph Loop
./.aria/ralph/ralph.sh init "description"   # Initialize feature
./.aria/ralph/ralph.sh run [iterations]     # Run autonomous loop
./.aria/ralph/ralph.sh status               # Show status

# Verification
./.aria/verify-executor.sh quick            # Types + lint
./.aria/verify-executor.sh standard         # + tests + build
./.aria/verify-executor.sh full             # + integration + E2E

# Git Operations
./.aria/git-ops.sh checkpoint [name]        # Save checkpoint
./.aria/git-ops.sh rollback checkpoint X    # Rollback
./.aria/git-ops.sh pr create                # Create PR

# Model Selection
./.aria/model-selector.sh status            # Show budget/usage
./.aria/model-selector.sh stats             # Show learning data

# Human-in-the-Loop
./.aria/hitl.sh status                      # Show pending requests
./.aria/hitl.sh respond "guidance"          # Provide guidance
```

## IDE Integration

**Claude Code:** `/aria`, `/aria-verify`, `/aria-start`, `/aria-status`

**VS Code:** `Cmd+Shift+P` → "Tasks: Run Task" → ARIA tasks

**Git Hooks:** `./.aria/hooks/install.sh install`

## Directory Structure

```
.aria/
├── aria-engine.sh          # Main orchestrator
├── ralph/                  # Autonomous loop
│   ├── ralph.sh           # Loop runner
│   ├── prd.json           # Product requirements
│   └── progress.txt       # Progress log
├── verify-executor.sh      # Verification pipeline
├── rails-executor.sh       # YAML rails executor
├── model-selector.sh       # Intelligent model selection
├── git-ops.sh             # Checkpoint/rollback/PR
├── hitl.sh                # Human-in-the-loop
├── hooks/                 # Git hooks
├── rails/                 # YAML rail definitions
└── docs/                  # Documentation
```

## Two Entry Points

ARIA supports two operational modes:

| Mode | Entry Point | Use Case |
|------|-------------|----------|
| **External** | `ralph.sh` | Terminal-based autonomous loop |
| **Hybrid** | `CLAUDE.md` | Inside Claude Code / VS Code |

**External Mode** (this README): Shell scripts orchestrate Claude as subprocess. Best for autonomous batch work.

**Hybrid Mode** ([CLAUDE.md](./CLAUDE.md)): Skills-based system with ARIA rules embedded in Claude's context. Best for interactive development. See also:
- [Cheatsheet](.aria/docs/CHEATSHEET.md) - Quick reference
- [Skill Registry](.aria/skills/REGISTRY.md) - Available skills

## Archive

Prior work on an ARIA programming language concept is archived in [`archive/aria-language/`](./archive/aria-language/).

---

*ARIA: Autonomous, safe, and intelligent AI-assisted development.*
