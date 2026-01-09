#!/bin/bash
# ARIA Git Hooks Installer
# Installs ARIA hooks into .git/hooks

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GIT_DIR="$(git rev-parse --git-dir 2>/dev/null)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

if [[ -z "$GIT_DIR" ]]; then
    echo -e "${RED}Error: Not in a git repository${NC}"
    exit 1
fi

HOOKS_DIR="$GIT_DIR/hooks"

echo -e "${BLUE}ARIA Git Hooks Installer${NC}"
echo ""

# Create hooks directory if it doesn't exist
mkdir -p "$HOOKS_DIR"

# Available hooks
HOOKS=("pre-commit" "pre-push" "commit-msg")

install_hook() {
    local hook_name="$1"
    local source="$SCRIPT_DIR/$hook_name"
    local target="$HOOKS_DIR/$hook_name"

    if [[ ! -f "$source" ]]; then
        echo -e "${YELLOW}  Skip: $hook_name (not found)${NC}"
        return
    fi

    if [[ -f "$target" ]]; then
        # Check if it's already an ARIA hook
        if grep -q "ARIA" "$target" 2>/dev/null; then
            echo -e "${YELLOW}  Update: $hook_name (replacing existing ARIA hook)${NC}"
        else
            # Backup existing hook
            local backup="$target.backup.$(date +%Y%m%d_%H%M%S)"
            cp "$target" "$backup"
            echo -e "${YELLOW}  Backup: $hook_name (saved to $backup)${NC}"
        fi
    fi

    cp "$source" "$target"
    chmod +x "$target"
    echo -e "${GREEN}  Install: $hook_name${NC}"
}

uninstall_hook() {
    local hook_name="$1"
    local target="$HOOKS_DIR/$hook_name"

    if [[ -f "$target" ]] && grep -q "ARIA" "$target" 2>/dev/null; then
        rm "$target"
        echo -e "${GREEN}  Remove: $hook_name${NC}"

        # Restore backup if exists
        local backup=$(ls -t "$target.backup."* 2>/dev/null | head -1)
        if [[ -n "$backup" ]]; then
            mv "$backup" "$target"
            echo -e "${YELLOW}  Restore: $hook_name (from backup)${NC}"
        fi
    else
        echo -e "${YELLOW}  Skip: $hook_name (not an ARIA hook)${NC}"
    fi
}

case "${1:-install}" in
    "install")
        echo "Installing ARIA git hooks..."
        echo ""
        for hook in "${HOOKS[@]}"; do
            install_hook "$hook"
        done
        echo ""
        echo -e "${GREEN}ARIA hooks installed!${NC}"
        echo ""
        echo "Hooks will run automatically on git operations."
        echo "Bypass with --no-verify flag if needed."
        ;;

    "uninstall")
        echo "Uninstalling ARIA git hooks..."
        echo ""
        for hook in "${HOOKS[@]}"; do
            uninstall_hook "$hook"
        done
        echo ""
        echo -e "${GREEN}ARIA hooks uninstalled${NC}"
        ;;

    "status")
        echo "ARIA Git Hooks Status:"
        echo ""
        for hook in "${HOOKS[@]}"; do
            local target="$HOOKS_DIR/$hook"
            if [[ -f "$target" ]] && grep -q "ARIA" "$target" 2>/dev/null; then
                echo -e "  ${GREEN}✓${NC} $hook (installed)"
            elif [[ -f "$target" ]]; then
                echo -e "  ${YELLOW}○${NC} $hook (other hook installed)"
            else
                echo -e "  ${RED}✗${NC} $hook (not installed)"
            fi
        done
        ;;

    "help"|*)
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  install    Install ARIA git hooks (default)"
        echo "  uninstall  Remove ARIA git hooks"
        echo "  status     Show hook installation status"
        echo "  help       Show this help"
        echo ""
        echo "Hooks installed:"
        echo "  pre-commit   Quick verification before commit"
        echo "  pre-push     Standard verification before push"
        echo "  commit-msg   Commit message validation"
        ;;
esac
