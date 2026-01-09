#!/bin/bash
# ARIA Discovery Agent
# Onboards existing projects by scanning, asking questions, and building context

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="${1:-$(pwd)}"
CONTEXT_FILE="$SCRIPT_DIR/project-context.md"
QUESTIONS_FILE="$SCRIPT_DIR/state/discovery-questions.json"

source "$SCRIPT_DIR/common.sh" || { echo "Failed to load common.sh"; exit 1; }
aria_check_deps jq || exit 1

mkdir -p "$SCRIPT_DIR/state"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# ============================================================================
# CODEBASE SCANNING
# ============================================================================

scan_tech_stack() {
    echo -e "${BLUE}Scanning tech stack...${NC}"

    local stack=""

    # Node.js
    if [[ -f "$PROJECT_DIR/package.json" ]]; then
        stack+="Node.js"
        local node_version=$(jq -r '.engines.node // "unspecified"' "$PROJECT_DIR/package.json" 2>/dev/null)
        [[ "$node_version" != "unspecified" ]] && stack+=" ($node_version)"
        stack+="\n"

        # Check for TypeScript
        if jq -e '.devDependencies.typescript or .dependencies.typescript' "$PROJECT_DIR/package.json" >/dev/null 2>&1; then
            stack+="TypeScript\n"
        fi

        # Check for frameworks
        if jq -e '.dependencies.react' "$PROJECT_DIR/package.json" >/dev/null 2>&1; then
            stack+="React\n"
        fi
        if jq -e '.dependencies.next' "$PROJECT_DIR/package.json" >/dev/null 2>&1; then
            stack+="Next.js\n"
        fi
        if jq -e '.dependencies.express' "$PROJECT_DIR/package.json" >/dev/null 2>&1; then
            stack+="Express\n"
        fi
        if jq -e '.dependencies.fastify' "$PROJECT_DIR/package.json" >/dev/null 2>&1; then
            stack+="Fastify\n"
        fi

        # Check for ORMs
        if jq -e '.dependencies.prisma or .devDependencies.prisma' "$PROJECT_DIR/package.json" >/dev/null 2>&1; then
            stack+="Prisma ORM\n"
        fi
        if jq -e '.dependencies.typeorm' "$PROJECT_DIR/package.json" >/dev/null 2>&1; then
            stack+="TypeORM\n"
        fi

        # Check for testing
        if jq -e '.devDependencies.jest' "$PROJECT_DIR/package.json" >/dev/null 2>&1; then
            stack+="Jest (testing)\n"
        fi
        if jq -e '.devDependencies.vitest' "$PROJECT_DIR/package.json" >/dev/null 2>&1; then
            stack+="Vitest (testing)\n"
        fi
    fi

    # Python
    if [[ -f "$PROJECT_DIR/requirements.txt" ]] || [[ -f "$PROJECT_DIR/pyproject.toml" ]]; then
        stack+="Python\n"
        if [[ -f "$PROJECT_DIR/pyproject.toml" ]]; then
            if grep -q "django" "$PROJECT_DIR/pyproject.toml" 2>/dev/null; then
                stack+="Django\n"
            fi
            if grep -q "fastapi" "$PROJECT_DIR/pyproject.toml" 2>/dev/null; then
                stack+="FastAPI\n"
            fi
            if grep -q "pytest" "$PROJECT_DIR/pyproject.toml" 2>/dev/null; then
                stack+="pytest (testing)\n"
            fi
        fi
    fi

    # Go
    if [[ -f "$PROJECT_DIR/go.mod" ]]; then
        stack+="Go\n"
    fi

    # Rust
    if [[ -f "$PROJECT_DIR/Cargo.toml" ]]; then
        stack+="Rust\n"
    fi

    # Docker
    if [[ -f "$PROJECT_DIR/Dockerfile" ]] || [[ -f "$PROJECT_DIR/docker-compose.yml" ]]; then
        stack+="Docker\n"
    fi

    echo -e "$stack"
}

scan_structure() {
    echo -e "${BLUE}Scanning directory structure...${NC}"

    local structure=""

    # Common directories
    [[ -d "$PROJECT_DIR/src" ]] && structure+="src/ - Source code\n"
    [[ -d "$PROJECT_DIR/lib" ]] && structure+="lib/ - Libraries\n"
    [[ -d "$PROJECT_DIR/app" ]] && structure+="app/ - Application code\n"
    [[ -d "$PROJECT_DIR/api" ]] && structure+="api/ - API endpoints\n"
    [[ -d "$PROJECT_DIR/pages" ]] && structure+="pages/ - Page components (Next.js style)\n"
    [[ -d "$PROJECT_DIR/components" ]] && structure+="components/ - UI components\n"
    [[ -d "$PROJECT_DIR/services" ]] && structure+="services/ - Service layer\n"
    [[ -d "$PROJECT_DIR/models" ]] && structure+="models/ - Data models\n"
    [[ -d "$PROJECT_DIR/utils" ]] && structure+="utils/ - Utilities\n"
    [[ -d "$PROJECT_DIR/helpers" ]] && structure+="helpers/ - Helper functions\n"
    [[ -d "$PROJECT_DIR/tests" ]] && structure+="tests/ - Test files\n"
    [[ -d "$PROJECT_DIR/__tests__" ]] && structure+="__tests__/ - Jest test files\n"
    [[ -d "$PROJECT_DIR/test" ]] && structure+="test/ - Test files\n"
    [[ -d "$PROJECT_DIR/spec" ]] && structure+="spec/ - Spec files\n"
    [[ -d "$PROJECT_DIR/config" ]] && structure+="config/ - Configuration\n"
    [[ -d "$PROJECT_DIR/scripts" ]] && structure+="scripts/ - Build/deploy scripts\n"
    [[ -d "$PROJECT_DIR/docs" ]] && structure+="docs/ - Documentation\n"
    [[ -d "$PROJECT_DIR/prisma" ]] && structure+="prisma/ - Prisma schema and migrations\n"
    [[ -d "$PROJECT_DIR/migrations" ]] && structure+="migrations/ - Database migrations\n"
    [[ -d "$PROJECT_DIR/public" ]] && structure+="public/ - Static assets\n"
    [[ -d "$PROJECT_DIR/static" ]] && structure+="static/ - Static files\n"

    echo -e "$structure"
}

scan_patterns() {
    echo -e "${BLUE}Scanning code patterns...${NC}"

    local patterns=""

    # Naming conventions
    if find "$PROJECT_DIR" -name "*.tsx" -o -name "*.jsx" 2>/dev/null | head -1 | grep -q .; then
        local sample=$(find "$PROJECT_DIR" -name "*.tsx" -o -name "*.jsx" 2>/dev/null | head -1)
        if [[ -n "$sample" ]]; then
            local basename=$(basename "$sample" | sed 's/\.[^.]*$//')
            if [[ "$basename" =~ ^[A-Z] ]]; then
                patterns+="Components: PascalCase naming\n"
            elif [[ "$basename" =~ ^[a-z] ]]; then
                patterns+="Components: camelCase naming\n"
            fi
        fi
    fi

    # Check for barrel exports
    if find "$PROJECT_DIR" -name "index.ts" -o -name "index.js" 2>/dev/null | head -1 | grep -q .; then
        patterns+="Uses barrel exports (index.ts/js)\n"
    fi

    # Check for environment handling
    if [[ -f "$PROJECT_DIR/.env.example" ]] || [[ -f "$PROJECT_DIR/.env.template" ]]; then
        patterns+="Environment: .env files with template\n"
    fi

    # Check for config files
    [[ -f "$PROJECT_DIR/tsconfig.json" ]] && patterns+="TypeScript configured\n"
    [[ -f "$PROJECT_DIR/.eslintrc.js" ]] || [[ -f "$PROJECT_DIR/.eslintrc.json" ]] && patterns+="ESLint configured\n"
    [[ -f "$PROJECT_DIR/.prettierrc" ]] || [[ -f "$PROJECT_DIR/.prettierrc.json" ]] && patterns+="Prettier configured\n"
    [[ -f "$PROJECT_DIR/jest.config.js" ]] || [[ -f "$PROJECT_DIR/jest.config.ts" ]] && patterns+="Jest configured\n"

    echo -e "$patterns"
}

scan_testing() {
    echo -e "${BLUE}Scanning test coverage...${NC}"

    local testing=""

    # Count test files
    local test_count=$(find "$PROJECT_DIR" \( -name "*.test.ts" -o -name "*.test.js" -o -name "*.spec.ts" -o -name "*.spec.js" -o -name "*_test.py" -o -name "test_*.py" \) 2>/dev/null | wc -l)
    testing+="Test files found: $test_count\n"

    # Count source files for rough coverage estimate
    local src_count=$(find "$PROJECT_DIR" \( -name "*.ts" -o -name "*.js" -o -name "*.py" \) -not -path "*/node_modules/*" -not -name "*.test.*" -not -name "*.spec.*" 2>/dev/null | wc -l)
    testing+="Source files: $src_count\n"

    if [[ $src_count -gt 0 ]]; then
        local ratio=$((test_count * 100 / src_count))
        testing+="Test/Source ratio: ~${ratio}%\n"

        if [[ $ratio -lt 10 ]]; then
            testing+="⚠️  Low test coverage\n"
        fi
    fi

    # Check for E2E
    if [[ -d "$PROJECT_DIR/e2e" ]] || [[ -d "$PROJECT_DIR/cypress" ]] || [[ -d "$PROJECT_DIR/playwright" ]]; then
        testing+="E2E tests: Present\n"
    else
        testing+="E2E tests: Not found\n"
    fi

    echo -e "$testing"
}

scan_docs() {
    echo -e "${BLUE}Scanning documentation...${NC}"

    local docs=""

    [[ -f "$PROJECT_DIR/README.md" ]] && docs+="README.md: Present\n"
    [[ -f "$PROJECT_DIR/CONTRIBUTING.md" ]] && docs+="CONTRIBUTING.md: Present\n"
    [[ -f "$PROJECT_DIR/CHANGELOG.md" ]] && docs+="CHANGELOG.md: Present\n"
    [[ -d "$PROJECT_DIR/docs" ]] && docs+="docs/ directory: Present\n"

    # Check for ADRs
    if [[ -d "$PROJECT_DIR/docs/adr" ]] || [[ -d "$PROJECT_DIR/adr" ]]; then
        docs+="Architecture Decision Records: Present\n"
    fi

    # Check for API docs
    if [[ -f "$PROJECT_DIR/openapi.yaml" ]] || [[ -f "$PROJECT_DIR/swagger.json" ]]; then
        docs+="API specification: Present\n"
    fi

    echo -e "$docs"
}

# ============================================================================
# QUESTION GENERATION
# ============================================================================

generate_questions() {
    echo -e "${BLUE}Generating questions...${NC}"

    local questions=()

    # Always ask these
    questions+=("What is the main purpose of this project?")
    questions+=("Are there any areas of the codebase I should NOT modify?")
    questions+=("What is the deployment process? (PR → staging → prod?)")

    # Conditional questions
    if [[ ! -f "$PROJECT_DIR/README.md" ]]; then
        questions+=("There's no README. Can you describe the project architecture?")
    fi

    local test_count=$(find "$PROJECT_DIR" \( -name "*.test.*" -o -name "*.spec.*" \) 2>/dev/null | wc -l)
    if [[ $test_count -lt 5 ]]; then
        questions+=("Test coverage appears low. Should new code include tests? Any specific testing requirements?")
    fi

    if [[ -d "$PROJECT_DIR/src/legacy" ]] || [[ -d "$PROJECT_DIR/legacy" ]]; then
        questions+=("Found a 'legacy' directory. What's the status of this code?")
    fi

    # Check for multiple similar directories
    if [[ -d "$PROJECT_DIR/api" ]] && [[ -d "$PROJECT_DIR/services" ]]; then
        questions+=("Found both 'api/' and 'services/'. What's the difference? Where should new endpoints go?")
    fi

    # Output as JSON
    printf '%s\n' "${questions[@]}" | jq -R . | jq -s '.' > "$QUESTIONS_FILE"

    echo "${#questions[@]} questions generated"
}

# ============================================================================
# HITL Q&A FLOW
# ============================================================================

run_qa_session() {
    if [[ ! -f "$QUESTIONS_FILE" ]]; then
        echo "No questions file. Run scan first."
        return 1
    fi

    local questions=$(cat "$QUESTIONS_FILE")
    local count=$(echo "$questions" | jq 'length')
    local answers=()

    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}              PROJECT DISCOVERY Q&A                        ${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Please answer these questions to help ARIA understand your project."
    echo "Type 'skip' to skip a question, 'done' to finish early."
    echo ""

    for ((i=0; i<count; i++)); do
        local question=$(echo "$questions" | jq -r ".[$i]")
        echo -e "${YELLOW}Q$((i+1))/${count}: $question${NC}"
        read -p "A: " answer

        if [[ "$answer" == "done" ]]; then
            break
        fi

        if [[ "$answer" != "skip" ]] && [[ -n "$answer" ]]; then
            answers+=("$(jq -n --arg q "$question" --arg a "$answer" '{question: $q, answer: $a}')")
        fi
        echo ""
    done

    # Save answers
    printf '%s\n' "${answers[@]}" | jq -s '.' > "$SCRIPT_DIR/state/discovery-answers.json"

    echo -e "${GREEN}Answers saved. Run 'discover build' to create project context.${NC}"
}

# ============================================================================
# BUILD CONTEXT
# ============================================================================

build_context() {
    echo -e "${BLUE}Building project context...${NC}"

    local tech_stack=$(scan_tech_stack)
    local structure=$(scan_structure)
    local patterns=$(scan_patterns)
    local testing=$(scan_testing)
    local docs=$(scan_docs)

    # Load answers if available
    local qa_section=""
    if [[ -f "$SCRIPT_DIR/state/discovery-answers.json" ]]; then
        qa_section="## Questions Answered\n\n"
        while IFS= read -r item; do
            local q=$(echo "$item" | jq -r '.question')
            local a=$(echo "$item" | jq -r '.answer')
            qa_section+="**Q:** $q\n**A:** $a\n\n"
        done < <(jq -c '.[]' "$SCRIPT_DIR/state/discovery-answers.json" 2>/dev/null)
    fi

    # Build the context file
    cat > "$CONTEXT_FILE" << EOF
# Project Context

*Generated by ARIA Discovery Agent on $(date '+%Y-%m-%d %H:%M')*

This file captures what ARIA learned about this project. Review and edit as needed.

---

## Tech Stack

$(echo -e "$tech_stack" | sed 's/^/- /')

## Directory Structure

$(echo -e "$structure" | sed 's/^/- /')

## Code Patterns

$(echo -e "$patterns" | sed 's/^/- /')

## Testing

$(echo -e "$testing" | sed 's/^/- /')

## Documentation

$(echo -e "$docs" | sed 's/^/- /')

$(echo -e "$qa_section")

## Don't Touch

*Add areas that should not be modified:*

- (none specified yet)

## Special Instructions

*Add any special instructions for ARIA:*

- (none specified yet)

---

## Ready for ARIA

This project has been onboarded. You can now use:

\`\`\`bash
# Start planning with project context
./.aria/ralph/ralph.sh plan "Your feature request"
\`\`\`

ARIA will reference this file when making decisions.
EOF

    echo -e "${GREEN}Project context saved to: $CONTEXT_FILE${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Review and edit $CONTEXT_FILE"
    echo "  2. Add 'Don't Touch' areas and special instructions"
    echo "  3. Start using ARIA: ralph plan \"your feature\""
}

# ============================================================================
# MAIN
# ============================================================================

usage() {
    cat << EOF
ARIA Discovery Agent - Onboard existing projects

Usage: discover.sh <command> [project_dir]

Commands:
  scan [dir]     Scan codebase and generate questions
  qa             Answer questions about the project
  build          Build project-context.md from scan + answers
  full [dir]     Run complete discovery (scan → qa → build)
  status         Show discovery status

Examples:
  discover.sh scan /path/to/project
  discover.sh qa
  discover.sh build
  discover.sh full .
EOF
}

main() {
    local cmd="${1:-}"
    shift || true

    case "$cmd" in
        scan)
            PROJECT_DIR="${1:-$(pwd)}"
            echo ""
            echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
            echo -e "${CYAN}         ARIA DISCOVERY - Scanning $PROJECT_DIR${NC}"
            echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
            echo ""
            scan_tech_stack
            echo ""
            scan_structure
            echo ""
            scan_patterns
            echo ""
            scan_testing
            echo ""
            scan_docs
            echo ""
            generate_questions
            echo ""
            echo -e "${GREEN}Scan complete. Run 'discover.sh qa' to answer questions.${NC}"
            ;;
        qa)
            run_qa_session
            ;;
        build)
            build_context
            ;;
        full)
            PROJECT_DIR="${1:-$(pwd)}"
            echo ""
            echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
            echo -e "${CYAN}         ARIA DISCOVERY - Full Onboarding${NC}"
            echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
            echo ""
            scan_tech_stack
            echo ""
            scan_structure
            echo ""
            scan_patterns
            echo ""
            scan_testing
            echo ""
            scan_docs
            echo ""
            generate_questions
            echo ""
            run_qa_session
            echo ""
            build_context
            ;;
        status)
            echo "Discovery Status:"
            [[ -f "$QUESTIONS_FILE" ]] && echo "  Questions: Generated" || echo "  Questions: Not generated"
            [[ -f "$SCRIPT_DIR/state/discovery-answers.json" ]] && echo "  Answers: Provided" || echo "  Answers: Not provided"
            [[ -f "$CONTEXT_FILE" ]] && echo "  Context: Built" || echo "  Context: Not built"
            ;;
        *)
            usage
            exit 1
            ;;
    esac
}

main "$@"
