#!/bin/bash
# ARIA Project Workspace Setup (Mac/Linux)
# Usage: setup-project.sh <project-path> <aria-path>
# Example: setup-project.sh ~/aria-eval/Projects/SVM ~/aria-test

set -e

PROJECT="$1"
ARIA="$2"

if [ -z "$PROJECT" ] || [ -z "$ARIA" ]; then
    echo "Usage: setup-project.sh <project-path> <aria-path>"
    echo ""
    echo "Examples:"
    echo "  setup-project.sh ~/aria-eval/Projects/SVM ~/aria-test"
    echo "  setup-project.sh /tmp/my-test ~/aria-test"
    echo ""
    echo "Arguments:"
    echo "  project-path: Full path to create workspace"
    echo "  aria-path:    Path to cloned ARIA framework"
    exit 1
fi

# Extract project name from path for display
PROJECT_NAME=$(basename "$PROJECT")

# Check ARIA source exists
if [ ! -f "$ARIA/CLAUDE.md" ]; then
    echo "ERROR: ARIA not found at $ARIA"
    echo "Clone it first: git clone -b main https://github.com/kknipe2k/AgentFramework.git $ARIA"
    exit 1
fi

# Check project doesn't already exist
if [ -d "$PROJECT" ]; then
    echo "ERROR: Project already exists: $PROJECT"
    echo "Delete it first or choose a different name."
    exit 1
fi

echo "Creating project workspace: $PROJECT"
echo "ARIA source: $ARIA"
echo ""

# ============================================
# Create project directory structure
# ============================================
mkdir -p "$PROJECT/.aria/state"
mkdir -p "$PROJECT/.aria/docs"
mkdir -p "$PROJECT/.aria/outputs"
mkdir -p "$PROJECT/.aria/prototypes"
mkdir -p "$PROJECT/.aria/logs"
mkdir -p "$PROJECT/.aria/reports"
mkdir -p "$PROJECT/sources"

# ============================================
# Symlink immutable framework files
# ============================================
echo "Linking framework files..."

# Root CLAUDE.md
ln -s "$ARIA/CLAUDE.md" "$PROJECT/CLAUDE.md"

# Skill definitions (read-only)
ln -s "$ARIA/.aria/skills" "$PROJECT/.aria/skills"

# Scripts (read-only)
ln -s "$ARIA/.aria/scripts" "$PROJECT/.aria/scripts"

# Templates (read-only)
[ -d "$ARIA/.aria/templates" ] && ln -s "$ARIA/.aria/templates" "$PROJECT/.aria/templates"

# Dashboard (read-only)
[ -d "$ARIA/.aria/dashboard" ] && ln -s "$ARIA/.aria/dashboard" "$PROJECT/.aria/dashboard"

# Git hooks (read-only)
[ -d "$ARIA/.aria/hooks" ] && ln -s "$ARIA/.aria/hooks" "$PROJECT/.aria/hooks"

# Safety rails (read-only)
[ -d "$ARIA/.aria/rails" ] && ln -s "$ARIA/.aria/rails" "$PROJECT/.aria/rails"

# Planner (read-only)
[ -d "$ARIA/.aria/planner" ] && ln -s "$ARIA/.aria/planner" "$PROJECT/.aria/planner"

# Ralph executor (read-only)
[ -d "$ARIA/.aria/ralph" ] && ln -s "$ARIA/.aria/ralph" "$PROJECT/.aria/ralph"

# Claude IDE integration (read-only)
[ -d "$ARIA/.claude" ] && ln -s "$ARIA/.claude" "$PROJECT/.claude"

# ============================================
# Symlink core shell scripts
# ============================================
echo "Linking core scripts..."
for script in verify.sh common.sh git-ops.sh hitl.sh aria-engine.sh \
              verify-executor.sh rails-executor.sh model-selector.sh \
              agent-runner.sh design-notes.sh discover.sh pause.sh; do
    [ -f "$ARIA/.aria/$script" ] && ln -s "$ARIA/.aria/$script" "$PROJECT/.aria/$script"
done

# ============================================
# Create empty state files
# ============================================
echo "Initializing state files..."

# progress.json - task tracking
cat > "$PROJECT/.aria/state/progress.json" << 'STATEJSON'
{
  "tasks": [],
  "mode": null,
  "started": null,
  "completed": null
}
STATEJSON

# current-plan.json - active plan (created by planning skill)
cat > "$PROJECT/.aria/state/current-plan.json" << 'PLANJSON'
{
  "id": null,
  "title": null,
  "status": "empty",
  "created": null,
  "tasks": []
}
PLANJSON

# Empty JSONL files for tracing
touch "$PROJECT/.aria/state/decisions.jsonl"
touch "$PROJECT/.aria/state/signals.jsonl"

# ============================================
# Create project-context.md template
# ============================================
cat > "$PROJECT/.aria/project-context.md" << 'CTXEOF'
# Project Context

*Edit this file to capture project-specific knowledge for ARIA.*

---

## Tech Stack

- [List your technologies here]

## Directory Structure

- `sources/` - Input materials (papers, docs, repos)
- `.aria/` - ARIA framework and outputs

## Don't Touch

- [List areas that should NOT be modified without approval]

## Special Instructions

- [Any project-specific rules or patterns]

---

## Ready for ARIA

Run discovery to auto-populate this file:
```bash
# ARIA will analyze sources/ and fill in details
```
CTXEOF

# ============================================
# Create design-notes.md template
# ============================================
cat > "$PROJECT/.aria/design-notes.md" << 'NOTESEOF'
# Design Notes

*AI reasoning log - decisions and rationale*

---

## Session Log

<!-- ARIA will append decisions here during execution -->
NOTESEOF

# ============================================
# Create project README
# ============================================
cat > "$PROJECT/README.md" << EOF
# $PROJECT_NAME

ARIA workspace created: $(date)

## Structure

\`\`\`
sources/              ← Drop papers, docs, repos here
.aria/
├── state/            ← JSON state (plan, progress, traces)
├── docs/             ← IDEA.md, research synthesis
├── outputs/          ← Slides, FOCUS.md
├── prototypes/       ← Working demos
├── logs/             ← Token usage, tracking
└── reports/          ← Final reports
\`\`\`

## Workflows

**Research:** Drop paper in sources/ → IDEA.md → slides → prototype
**Build:** Plan → execute → verify → commit
**Modify:** Discovery → plan → execute → verify

## Usage

1. Drop source materials in \`sources/\`
2. Open this folder in VS Code
3. Run ARIA workflow (research, build, or modify)
4. Choose prototype variant when prompted (research flow)

## Mode Selection

ARIA auto-selects mode based on task size:
- **LITE** (1-5 tasks): Fast, minimal overhead
- **STANDARD** (6-15 tasks): Normal workflow with verification
- **FULL** (16-40 tasks): Maximum oversight, design notes
- **FULL+** (40+ tasks): Epic-level management, design doc required
EOF

# ============================================
# Summary
# ============================================
echo ""
echo "========================================"
echo "Project ready: $PROJECT"
echo "========================================"
echo ""
echo "Structure created:"
echo "  sources/              - Input materials"
echo "  .aria/state/          - JSON state files"
echo "  .aria/docs/           - Research outputs (IDEA.md)"
echo "  .aria/outputs/        - Slides, FOCUS.md"
echo "  .aria/prototypes/     - Working demos"
echo "  .aria/logs/           - Token tracking"
echo "  .aria/reports/        - Final reports"
echo ""
echo "State files initialized:"
echo "  - progress.json       (task tracking)"
echo "  - current-plan.json   (active plan)"
echo "  - decisions.jsonl     (decision trace)"
echo "  - signals.jsonl       (tool signals)"
echo "  - project-context.md  (project knowledge)"
echo "  - design-notes.md     (reasoning log)"
echo ""
echo "Next steps:"
echo "  1. Drop source materials in: $PROJECT/sources/"
echo "  2. Open VS Code: code \"$PROJECT\""
echo "  3. Run ARIA workflow"
echo ""
echo "Modes: LITE | STANDARD | FULL | FULL+"
echo "Flows: research | build | modify"
echo ""
