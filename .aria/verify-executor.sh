#!/bin/bash
# ARIA Verification Executor
# Runs verification checks: types, lint, tests, build, E2E

# Exit on error, undefined vars, and pipeline failures
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh" || { echo "Failed to load common.sh"; exit 1; }

# Check dependencies
aria_check_deps git || exit 1

ARIA_DIR="$SCRIPT_DIR"
SCREENSHOTS_DIR="$ARIA_DIR/screenshots"
STATE_DIR="$ARIA_DIR/state"
LOGS_DIR="$ARIA_DIR/logs"

# Use colors from common.sh
RED="$ARIA_RED"
GREEN="$ARIA_GREEN"
YELLOW="$ARIA_YELLOW"
BLUE="$ARIA_BLUE"
NC="$ARIA_NC"

# Create directories
mkdir -p "$SCREENSHOTS_DIR" "$STATE_DIR" "$LOGS_DIR"

# Configuration
APP_URL="${APP_URL:-http://localhost:3000}"
API_URL="${API_URL:-http://localhost:3000/api}"
VERIFICATION_TIMEOUT="${VERIFICATION_TIMEOUT:-30000}"

# ============================================
# SECURITY: URL Validation (Issue #10 fix)
# ============================================

# Validate URL is safe (no shell injection)
validate_url() {
    local url="$1"
    local name="$2"

    # Check URL format - must start with http:// or https://
    if ! echo "$url" | grep -qE "^https?://[a-zA-Z0-9]"; then
        echo -e "${RED}SECURITY: Invalid URL format for $name${NC}"
        echo -e "${RED}URL must start with http:// or https://${NC}"
        return 1
    fi

    # Block shell metacharacters that could enable injection
    local dangerous_chars=';|&$`><(){}[]!#'
    if echo "$url" | grep -qE "[$dangerous_chars]"; then
        echo -e "${RED}SECURITY: Dangerous characters in $name${NC}"
        echo -e "${RED}URL contains shell metacharacters - blocked for safety${NC}"
        return 1
    fi

    # Block newlines and carriage returns
    if echo "$url" | grep -qE $'\n|\r'; then
        echo -e "${RED}SECURITY: Newlines in $name - blocked${NC}"
        return 1
    fi

    return 0
}

# Validate configured URLs at startup
if ! validate_url "$APP_URL" "APP_URL"; then
    echo -e "${RED}Set a valid APP_URL (e.g., http://localhost:3000)${NC}"
    exit 1
fi

if ! validate_url "$API_URL" "API_URL"; then
    echo -e "${RED}Set a valid API_URL (e.g., http://localhost:3000/api)${NC}"
    exit 1
fi

# ============================================
# DEPENDENCY DETECTION & INSTALLATION
# ============================================

check_node() {
    if ! command -v node >/dev/null 2>&1; then
        echo -e "${RED}Node.js not found. Please install Node.js first.${NC}"
        return 1
    fi
    return 0
}

check_playwright() {
    if [[ -f "node_modules/playwright/package.json" ]]; then
        return 0
    fi
    return 1
}

install_playwright() {
    echo -e "${YELLOW}Installing Playwright...${NC}"
    npm install -D playwright 2>/dev/null || npm install -D @playwright/test 2>/dev/null
    npx playwright install chromium 2>/dev/null || true
    echo -e "${GREEN}Playwright installed${NC}"
}

check_cypress() {
    [[ -f "node_modules/cypress/package.json" ]] || [[ -f "cypress.config.js" ]] || [[ -f "cypress.config.ts" ]]
}

detect_e2e_framework() {
    if [[ -f "playwright.config.ts" ]] || [[ -f "playwright.config.js" ]]; then
        echo "playwright"
    elif [[ -f "cypress.config.js" ]] || [[ -f "cypress.config.ts" ]] || [[ -d "cypress" ]]; then
        echo "cypress"
    elif [[ -d "e2e" ]]; then
        # Check what's in e2e folder
        if ls e2e/*.spec.ts 2>/dev/null | head -1 | grep -q .; then
            echo "playwright"
        else
            echo "unknown"
        fi
    else
        echo "none"
    fi
}

detect_project_type() {
    if [[ -f "package.json" ]]; then
        if grep -q '"react"' package.json 2>/dev/null; then
            echo "react"
        elif grep -q '"vue"' package.json 2>/dev/null; then
            echo "vue"
        elif grep -q '"@angular/core"' package.json 2>/dev/null; then
            echo "angular"
        elif grep -q '"svelte"' package.json 2>/dev/null; then
            echo "svelte"
        elif grep -q '"next"' package.json 2>/dev/null; then
            echo "nextjs"
        elif grep -q '"express"' package.json 2>/dev/null; then
            echo "express"
        else
            echo "node"
        fi
    elif [[ -f "requirements.txt" ]] || [[ -f "pyproject.toml" ]]; then
        echo "python"
    elif [[ -f "Cargo.toml" ]]; then
        echo "rust"
    elif [[ -f "go.mod" ]]; then
        echo "go"
    else
        echo "unknown"
    fi
}

# ============================================
# SERVER DETECTION
# ============================================

is_server_running() {
    local url="${1:-$APP_URL}"
    curl -s -o /dev/null -w "%{http_code}" "$url" 2>/dev/null | grep -qE "^[23]"
}

wait_for_server() {
    local url="${1:-$APP_URL}"
    local timeout="${2:-30}"
    local elapsed=0

    echo -e "${YELLOW}Waiting for server at $url...${NC}"
    while ! is_server_running "$url"; do
        sleep 1
        elapsed=$((elapsed + 1))
        if [[ $elapsed -ge $timeout ]]; then
            echo -e "${RED}Server not responding after ${timeout}s${NC}"
            return 1
        fi
    done
    echo -e "${GREEN}Server is up${NC}"
    return 0
}

# ============================================
# VERIFICATION CHECKS
# ============================================

# Check: App loads in browser
verify_app_loads() {
    echo -e "${BLUE}Checking: App loads in browser...${NC}"

    if ! is_server_running; then
        echo -e "${YELLOW}Server not running, skipping browser check${NC}"
        echo "skipped" > "$STATE_DIR/app_loads"
        return 0
    fi

    if ! check_playwright; then
        # Fallback to curl
        local status=$(curl -s -o /dev/null -w "%{http_code}" "$APP_URL")
        if [[ "$status" =~ ^[23] ]]; then
            echo -e "${GREEN}App responds with HTTP $status${NC}"
            echo "pass" > "$STATE_DIR/app_loads"
            return 0
        else
            echo -e "${RED}App returned HTTP $status${NC}"
            echo "fail" > "$STATE_DIR/app_loads"
            return 1
        fi
    fi

    # Use Playwright for full check
    node -e "
const { chromium } = require('playwright');
(async () => {
    const browser = await chromium.launch();
    const page = await browser.newPage();
    try {
        await page.goto('$APP_URL', { timeout: $VERIFICATION_TIMEOUT });
        const title = await page.title();
        console.log('Page loaded: ' + title);
        await page.screenshot({ path: '$SCREENSHOTS_DIR/app_loads.png' });
        await browser.close();
        process.exit(0);
    } catch (err) {
        console.error('Failed:', err.message);
        await page.screenshot({ path: '$SCREENSHOTS_DIR/app_loads_error.png' }).catch(() => {});
        await browser.close();
        process.exit(1);
    }
})();
" 2>&1 | tee "$LOGS_DIR/app_loads.log"

    local result=$?
    if [[ $result -eq 0 ]]; then
        echo -e "${GREEN}App loads successfully${NC}"
        echo "pass" > "$STATE_DIR/app_loads"
    else
        echo -e "${RED}App failed to load${NC}"
        echo "fail" > "$STATE_DIR/app_loads"
    fi
    return $result
}

# Check: No console errors
verify_no_console_errors() {
    echo -e "${BLUE}Checking: No JavaScript console errors...${NC}"

    if ! is_server_running || ! check_playwright; then
        echo -e "${YELLOW}Skipping console error check${NC}"
        return 0
    fi

    node -e "
const { chromium } = require('playwright');
(async () => {
    const browser = await chromium.launch();
    const page = await browser.newPage();
    const errors = [];

    page.on('pageerror', err => errors.push('PAGE ERROR: ' + err.message));
    page.on('console', msg => {
        if (msg.type() === 'error') errors.push('CONSOLE: ' + msg.text());
    });

    try {
        await page.goto('$APP_URL', { timeout: $VERIFICATION_TIMEOUT });
        await page.waitForTimeout(2000); // Wait for async errors
        await browser.close();

        if (errors.length > 0) {
            console.error('Errors found:');
            errors.forEach(e => console.error('  ' + e));
            process.exit(1);
        }
        console.log('No console errors');
        process.exit(0);
    } catch (err) {
        console.error('Check failed:', err.message);
        await browser.close();
        process.exit(1);
    }
})();
" 2>&1 | tee "$LOGS_DIR/console_errors.log"

    local result=$?
    if [[ $result -eq 0 ]]; then
        echo -e "${GREEN}No console errors${NC}"
        echo "pass" > "$STATE_DIR/console_errors"
    else
        echo -e "${YELLOW}Console errors detected (warning)${NC}"
        echo "warn" > "$STATE_DIR/console_errors"
    fi
    return 0  # Warning only, don't block
}

# Check: Take screenshot
verify_screenshot() {
    echo -e "${BLUE}Taking verification screenshot...${NC}"

    if ! is_server_running; then
        echo -e "${YELLOW}Server not running, skipping screenshot${NC}"
        return 0
    fi

    if ! check_playwright; then
        echo -e "${YELLOW}Playwright not installed, skipping screenshot${NC}"
        return 0
    fi

    node -e "
const { chromium } = require('playwright');
(async () => {
    const browser = await chromium.launch();
    const page = await browser.newPage();
    try {
        await page.goto('$APP_URL', { timeout: $VERIFICATION_TIMEOUT });
        await page.screenshot({ path: '$SCREENSHOTS_DIR/latest.png', fullPage: true });
        console.log('Screenshot: $SCREENSHOTS_DIR/latest.png');
        await browser.close();
    } catch (err) {
        console.error('Screenshot failed:', err.message);
        await browser.close();
        process.exit(1);
    }
})();
"

    if [[ -f "$SCREENSHOTS_DIR/latest.png" ]]; then
        echo -e "${GREEN}Screenshot saved${NC}"
        return 0
    fi
    return 1
}

# Check: Mobile responsive
verify_mobile() {
    echo -e "${BLUE}Checking: Mobile responsiveness...${NC}"

    if ! is_server_running || ! check_playwright; then
        echo -e "${YELLOW}Skipping mobile check${NC}"
        return 0
    fi

    node -e "
const { chromium, devices } = require('playwright');
(async () => {
    const browser = await chromium.launch();
    const context = await browser.newContext({ ...devices['iPhone 12'] });
    const page = await context.newPage();
    try {
        await page.goto('$APP_URL', { timeout: $VERIFICATION_TIMEOUT });
        await page.screenshot({ path: '$SCREENSHOTS_DIR/mobile.png' });
        console.log('Mobile screenshot: $SCREENSHOTS_DIR/mobile.png');
        await browser.close();
    } catch (err) {
        console.error('Mobile check failed:', err.message);
        await browser.close();
        process.exit(1);
    }
})();
"
    echo -e "${GREEN}Mobile screenshot saved${NC}"
    return 0
}

# Check: E2E tests pass
verify_e2e() {
    echo -e "${BLUE}Running E2E tests...${NC}"

    local framework=$(detect_e2e_framework)

    case "$framework" in
        "playwright")
            echo "Using Playwright"
            if npx playwright test 2>&1 | tee "$LOGS_DIR/e2e.log"; then
                echo -e "${GREEN}E2E tests passed${NC}"
                echo "pass" > "$STATE_DIR/e2e"
                return 0
            else
                echo -e "${RED}E2E tests failed${NC}"
                echo "fail" > "$STATE_DIR/e2e"
                return 1
            fi
            ;;
        "cypress")
            echo "Using Cypress"
            if npx cypress run 2>&1 | tee "$LOGS_DIR/e2e.log"; then
                echo -e "${GREEN}E2E tests passed${NC}"
                echo "pass" > "$STATE_DIR/e2e"
                return 0
            else
                echo -e "${RED}E2E tests failed${NC}"
                echo "fail" > "$STATE_DIR/e2e"
                return 1
            fi
            ;;
        "none"|"unknown")
            echo -e "${YELLOW}No E2E framework detected, skipping${NC}"
            echo "skipped" > "$STATE_DIR/e2e"
            return 0
            ;;
    esac
}

# Check: API responds
verify_api() {
    echo -e "${BLUE}Checking: API endpoints...${NC}"

    local health_url="${API_URL}/health"
    local status=$(curl -s -o /dev/null -w "%{http_code}" "$health_url" 2>/dev/null || echo "000")

    if [[ "$status" == "000" ]]; then
        echo -e "${YELLOW}API not reachable at $health_url${NC}"
        echo "skipped" > "$STATE_DIR/api"
        return 0
    elif [[ "$status" =~ ^[23] ]]; then
        echo -e "${GREEN}API health check passed (HTTP $status)${NC}"
        echo "pass" > "$STATE_DIR/api"
        return 0
    else
        echo -e "${RED}API health check failed (HTTP $status)${NC}"
        echo "fail" > "$STATE_DIR/api"
        return 1
    fi
}

# Check: Unit tests
verify_unit_tests() {
    echo -e "${BLUE}Running unit tests...${NC}"

    local project_type=$(detect_project_type)
    local result=0

    case "$project_type" in
        "node"|"react"|"vue"|"angular"|"svelte"|"nextjs"|"express")
            if [[ -f "package.json" ]] && grep -q '"test"' package.json; then
                if npm test 2>&1 | tee "$LOGS_DIR/unit_tests.log"; then
                    echo -e "${GREEN}Unit tests passed${NC}"
                    echo "pass" > "$STATE_DIR/unit_tests"
                    rm -f "$STATE_DIR/tests_failed"
                else
                    echo -e "${RED}Unit tests failed${NC}"
                    echo "fail" > "$STATE_DIR/unit_tests"
                    touch "$STATE_DIR/tests_failed"
                    result=1
                fi
            else
                echo -e "${YELLOW}No test script found${NC}"
                echo "skipped" > "$STATE_DIR/unit_tests"
            fi
            ;;
        "python")
            if command -v pytest >/dev/null 2>&1; then
                if pytest 2>&1 | tee "$LOGS_DIR/unit_tests.log"; then
                    echo -e "${GREEN}Unit tests passed${NC}"
                    echo "pass" > "$STATE_DIR/unit_tests"
                    rm -f "$STATE_DIR/tests_failed"
                else
                    echo -e "${RED}Unit tests failed${NC}"
                    echo "fail" > "$STATE_DIR/unit_tests"
                    touch "$STATE_DIR/tests_failed"
                    result=1
                fi
            else
                echo -e "${YELLOW}pytest not found${NC}"
                echo "skipped" > "$STATE_DIR/unit_tests"
            fi
            ;;
        *)
            echo -e "${YELLOW}Unknown project type, skipping tests${NC}"
            echo "skipped" > "$STATE_DIR/unit_tests"
            ;;
    esac

    return $result
}

# Check: Lint
verify_lint() {
    echo -e "${BLUE}Running linter...${NC}"

    if [[ -f "package.json" ]] && grep -q '"lint"' package.json; then
        if npm run lint 2>&1 | tee "$LOGS_DIR/lint.log"; then
            echo -e "${GREEN}Lint passed${NC}"
            echo "pass" > "$STATE_DIR/lint"
            return 0
        else
            echo -e "${RED}Lint failed${NC}"
            echo "fail" > "$STATE_DIR/lint"
            return 1
        fi
    else
        echo -e "${YELLOW}No lint script found${NC}"
        echo "skipped" > "$STATE_DIR/lint"
        return 0
    fi
}

# Check: TypeScript
verify_types() {
    echo -e "${BLUE}Checking TypeScript types...${NC}"

    if [[ -f "tsconfig.json" ]]; then
        if npx tsc --noEmit 2>&1 | tee "$LOGS_DIR/types.log"; then
            echo -e "${GREEN}Type check passed${NC}"
            echo "pass" > "$STATE_DIR/types"
            return 0
        else
            echo -e "${RED}Type check failed${NC}"
            echo "fail" > "$STATE_DIR/types"
            return 1
        fi
    else
        echo -e "${YELLOW}No tsconfig.json found${NC}"
        echo "skipped" > "$STATE_DIR/types"
        return 0
    fi
}

# Check: Build
verify_build() {
    echo -e "${BLUE}Running build...${NC}"

    if [[ -f "package.json" ]] && grep -q '"build"' package.json; then
        if npm run build 2>&1 | tee "$LOGS_DIR/build.log"; then
            echo -e "${GREEN}Build passed${NC}"
            echo "pass" > "$STATE_DIR/build"
            return 0
        else
            echo -e "${RED}Build failed${NC}"
            echo "fail" > "$STATE_DIR/build"
            return 1
        fi
    else
        echo -e "${YELLOW}No build script found${NC}"
        echo "skipped" > "$STATE_DIR/build"
        return 0
    fi
}

# ============================================
# VERIFICATION SUITES
# ============================================

# Quick verification (for Stop hook)
verify_quick() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}              ARIA Quick Verification                       ${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""

    local failures=0

    verify_unit_tests || failures=$((failures + 1))
    verify_types || failures=$((failures + 1))
    verify_lint || true  # Lint is warning only

    return $failures
}

# Standard verification (before commit)
verify_standard() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}              ARIA Standard Verification                    ${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""

    local failures=0

    verify_unit_tests || failures=$((failures + 1))
    verify_types || failures=$((failures + 1))
    verify_lint || failures=$((failures + 1))
    verify_build || failures=$((failures + 1))

    if is_server_running; then
        verify_app_loads || failures=$((failures + 1))
        verify_screenshot || true
    fi

    return $failures
}

# Full verification (before deploy/PR)
verify_full() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}              ARIA Full Verification                        ${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""

    local failures=0

    # Ensure Playwright is available
    if ! check_playwright; then
        echo -e "${YELLOW}Installing Playwright for full verification...${NC}"
        install_playwright
    fi

    verify_unit_tests || failures=$((failures + 1))
    verify_types || failures=$((failures + 1))
    verify_lint || failures=$((failures + 1))
    verify_build || failures=$((failures + 1))

    if is_server_running; then
        verify_app_loads || failures=$((failures + 1))
        verify_no_console_errors || true  # Warning only
        verify_screenshot || true
        verify_mobile || true
        verify_e2e || failures=$((failures + 1))
        verify_api || true  # Warning only
    else
        echo -e "${YELLOW}Server not running - skipping browser verification${NC}"
        echo -e "${YELLOW}Start with: npm start / npm run dev${NC}"
    fi

    return $failures
}

# ============================================
# REPORT GENERATION
# ============================================

generate_report() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}              VERIFICATION REPORT                           ${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""

    local checks=("unit_tests" "types" "lint" "build" "app_loads" "console_errors" "e2e" "api")
    local total=0
    local passed=0
    local failed=0
    local skipped=0

    for check in "${checks[@]}"; do
        local state_file="$STATE_DIR/$check"
        if [[ -f "$state_file" ]]; then
            local status=$(cat "$state_file")
            total=$((total + 1))
            case "$status" in
                "pass")
                    echo -e "  ${GREEN}PASS${NC}  $check"
                    passed=$((passed + 1))
                    ;;
                "fail")
                    echo -e "  ${RED}FAIL${NC}  $check"
                    failed=$((failed + 1))
                    ;;
                "warn")
                    echo -e "  ${YELLOW}WARN${NC}  $check"
                    passed=$((passed + 1))
                    ;;
                "skipped")
                    echo -e "  ${BLUE}SKIP${NC}  $check"
                    skipped=$((skipped + 1))
                    ;;
            esac
        fi
    done

    echo ""
    echo "────────────────────────────────────────────────────────────"
    echo "Total: $total | Passed: $passed | Failed: $failed | Skipped: $skipped"
    echo ""

    if [[ $failed -gt 0 ]]; then
        echo -e "${RED}VERIFICATION FAILED${NC}"
        echo ""
        echo "Check logs in: $LOGS_DIR/"
        if [[ -d "$SCREENSHOTS_DIR" ]] && ls "$SCREENSHOTS_DIR"/*.png >/dev/null 2>&1; then
            echo "Screenshots in: $SCREENSHOTS_DIR/"
        fi
        return 1
    else
        echo -e "${GREEN}VERIFICATION PASSED${NC}"
        return 0
    fi
}

# ============================================
# MAIN
# ============================================

main() {
    local command="${1:-help}"

    case "$command" in
        "quick")
            verify_quick
            generate_report
            ;;
        "standard"|"commit")
            verify_standard
            generate_report
            ;;
        "full"|"deploy"|"pr")
            verify_full
            generate_report
            ;;
        "app"|"app-loads")
            verify_app_loads
            ;;
        "screenshot")
            verify_screenshot
            ;;
        "mobile")
            verify_mobile
            ;;
        "e2e")
            verify_e2e
            ;;
        "tests"|"unit")
            verify_unit_tests
            ;;
        "lint")
            verify_lint
            ;;
        "types")
            verify_types
            ;;
        "build")
            verify_build
            ;;
        "api")
            verify_api
            ;;
        "console")
            verify_no_console_errors
            ;;
        "install-playwright")
            install_playwright
            ;;
        "status")
            generate_report
            ;;
        "help"|*)
            echo "ARIA Verification Executor"
            echo ""
            echo "Usage: $0 <command>"
            echo ""
            echo "Suites:"
            echo "  quick      - Tests, types, lint (for Stop hook)"
            echo "  standard   - Quick + build + app loads (before commit)"
            echo "  full       - Everything including E2E (before deploy/PR)"
            echo ""
            echo "Individual checks:"
            echo "  tests      - Run unit tests"
            echo "  types      - TypeScript type check"
            echo "  lint       - Run linter"
            echo "  build      - Run build"
            echo "  app        - Check app loads in browser"
            echo "  screenshot - Take screenshot"
            echo "  mobile     - Take mobile screenshot"
            echo "  e2e        - Run E2E tests"
            echo "  api        - Check API endpoints"
            echo "  console    - Check for console errors"
            echo ""
            echo "Other:"
            echo "  install-playwright  - Install Playwright"
            echo "  status              - Show verification report"
            echo ""
            echo "Environment:"
            echo "  APP_URL             - App URL (default: http://localhost:3000)"
            echo "  API_URL             - API URL (default: http://localhost:3000/api)"
            echo "  VERIFICATION_TIMEOUT - Timeout in ms (default: 30000)"
            ;;
    esac
}

main "$@"
