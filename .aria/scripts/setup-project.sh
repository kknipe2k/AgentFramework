#!/bin/bash
# ARIA Project Workspace Setup (Mac/Linux)
# Usage: setup-project.sh <project-name> [aria-path] [eval-path]
# Example: setup-project.sh SVM
# Example: setup-project.sh SVM ~/aria-test ~/aria/eval

set -e

PROJECT_NAME="$1"
ARIA="${2:-$HOME/aria-test}"
EVAL="${3:-$HOME/aria/eval}"

if [ -z "$PROJECT_NAME" ]; then
    echo "Usage: setup-project.sh <project-name> [aria-path] [eval-path]"
    echo ""
    echo "Examples:"
    echo "  setup-project.sh SVM"
    echo "  setup-project.sh SVM ~/aria-test ~/aria/eval"
    echo ""
    echo "Defaults:"
    echo "  aria-path: ~/aria-test"
    echo "  eval-path: ~/aria/eval"
    exit 1
fi

PROJECT="$EVAL/$PROJECT_NAME"

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

# Create project directory structure
mkdir -p "$PROJECT/.aria/state"
mkdir -p "$PROJECT/.aria/docs"
mkdir -p "$PROJECT/.aria/outputs"

# Symlink immutable framework files
echo "Linking framework files..."
ln -s "$ARIA/CLAUDE.md" "$PROJECT/CLAUDE.md"
ln -s "$ARIA/.aria/skills" "$PROJECT/.aria/skills"
ln -s "$ARIA/.aria/scripts" "$PROJECT/.aria/scripts"
ln -s "$ARIA/.aria/templates" "$PROJECT/.aria/templates" 2>/dev/null || true
ln -s "$ARIA/.aria/dashboard" "$PROJECT/.aria/dashboard" 2>/dev/null || true

# Copy mutable state files (fresh per project)
echo "Copying state templates..."
if ls "$ARIA/.aria/state/"*.json 1>/dev/null 2>&1; then
    cp "$ARIA/.aria/state/"*.json "$PROJECT/.aria/state/" 2>/dev/null || true
fi

# Create empty state files if they don't exist
[ -f "$PROJECT/.aria/state/progress.json" ] || echo '{"tasks": []}' > "$PROJECT/.aria/state/progress.json"
[ -f "$PROJECT/.aria/state/decisions.jsonl" ] || touch "$PROJECT/.aria/state/decisions.jsonl"
[ -f "$PROJECT/.aria/state/signals.jsonl" ] || touch "$PROJECT/.aria/state/signals.jsonl"

# Copy verify.sh if it exists
[ -f "$ARIA/.aria/verify.sh" ] && cp "$ARIA/.aria/verify.sh" "$PROJECT/.aria/verify.sh"

# Create README for the project
cat > "$PROJECT/README.md" << EOF
# $PROJECT_NAME

ARIA workspace created: $(date)

## Usage

1. Drop your source materials (papers, docs, repos) here
2. Open this folder in VS Code
3. Run ARIA - outputs will be saved in .aria/outputs/
EOF

echo ""
echo "========================================"
echo "Project ready: $PROJECT"
echo "========================================"
echo ""
echo "Next steps:"
echo "  1. Drop your source materials in: $PROJECT"
echo "  2. Open VS Code: code \"$PROJECT\""
echo "  3. Run ARIA research workflow"
echo ""
echo "Results will be saved in:"
echo "  - .aria/docs/IDEA.md"
echo "  - .aria/outputs/FOCUS.md"
echo "  - .aria/outputs/slides-*.pptx"
echo ""
