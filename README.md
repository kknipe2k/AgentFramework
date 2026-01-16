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

## Workspace Setup

For testing and evaluation, keep ARIA pristine and create isolated project workspaces:

```bash
# Windows
.aria\scripts\setup-project.bat SVM

# Mac/Linux
.aria/scripts/setup-project.sh SVM
```

This creates `~/aria/eval/SVM` (or `c:\aria\eval\SVM`) with:
- Symlinks to ARIA framework files (immutable)
- Fresh state directories (per-project)
- Ready for your source materials

**Workflow:**
1. Clone ARIA once: `git clone ... ~/aria-test`
2. Create project: `setup-project.sh MyResearch`
3. Drop papers/docs into `~/aria/eval/MyResearch`
4. Open VS Code there, run ARIA
5. Results stay in project folder, ARIA stays clean

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
| **Dashboard** | Web UI for decision lineage tracking |
| **Slide Generation** | Create presentations from research |

## Documentation

| Document | Description |
|----------|-------------|
| **[User Guide](.aria/docs/USER-GUIDE.md)** | Complete usage guide - start here |
| [Cheatsheet](.aria/docs/CHEATSHEET.md) | Quick reference for all modes/skills |
| [Skill Registry](.aria/skills/REGISTRY.md) | All 13 skills with triggers |
| [Observability](.aria/docs/OBSERVABILITY.md) | Decision tracing and dashboard |
| [Architecture](.aria/docs/CONCEPT-aria-architecture.md) | System design and components |
| [Boris Cherny Patterns](.aria/docs/CONCEPT-boris-cherny-patterns.md) | Verification and subagent patterns |
| [Ralph Autonomous Loop](.aria/docs/CONCEPT-ralph-autonomous-loop.md) | PRD-driven iteration pattern |
| [Parking Lot](.aria/docs/PARKING-LOT.md) | Future ideas (whisper, metrics) |

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

# Dashboard & Research
python .aria/scripts/serve-dashboard.py     # Open dashboard at :8420
python .aria/scripts/generate-slides.py     # Generate slides from IDEA.md

# Workspace Setup
.aria/scripts/setup-project.sh <name>       # Create isolated project workspace
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
├── scripts/               # Utility scripts
│   ├── serve-dashboard.py # Decision lineage dashboard
│   ├── generate-slides.py # Slide generation
│   ├── setup-project.sh   # Workspace setup (Mac/Linux)
│   └── setup-project.bat  # Workspace setup (Windows)
├── skills/                # Skill definitions
├── outputs/               # Generated artifacts (slides, etc.)
├── dashboard/             # Dashboard web UI
└── docs/                  # Documentation
```

## Two Entry Points

ARIA supports two operational modes:

| Mode | Entry Point | Use Case |
|------|-------------|----------|
| **External** | `ralph.sh` | Terminal-based autonomous loop |
| **Hybrid** | `/aria-start` | Inside Claude Code / VS Code |

**External Mode** (this README): Shell scripts orchestrate Claude as subprocess. Best for autonomous batch work.

**Hybrid Mode** ([CLAUDE.md](./CLAUDE.md)): Skills-based system with ARIA rules embedded in Claude's context. Best for interactive development.

**Hybrid Quick Start:**
```
/aria-start → Dashboard launches → Select: [b]uild / [m]odify / [r]esearch
```

See also:
- [Cheatsheet](.aria/docs/CHEATSHEET.md) - Quick reference
- [Skill Registry](.aria/skills/REGISTRY.md) - Available skills

## Archive

Prior work on an ARIA programming language concept is archived in [`archive/aria-language/`](./archive/aria-language/).

---

*ARIA: Autonomous, safe, and intelligent AI-assisted development.*
