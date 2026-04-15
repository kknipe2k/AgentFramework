#!/usr/bin/env bash
#
# Reference verification script for the Loom project.
#
# This file is a TEMPLATE. Copy to scripts/verify.sh in the new repo and
# adapt as needed. The npm run verify command should call this script.
#
# The script must:
#   1. Fail fast — first failure exits non-zero
#   2. Report which gate failed
#   3. Be runnable on Windows (via Git Bash or WSL) and Linux
#   4. Mirror exactly what CI runs, so "green locally" means "green in CI"
#
# Every commit runs this. Every milestone's definition-of-done includes
# this passing.

set -euo pipefail

# Colors (disable if NO_COLOR is set or stdout is not a tty)
if [[ -t 1 && -z "${NO_COLOR:-}" ]]; then
  RED=$'\033[0;31m'
  GREEN=$'\033[0;32m'
  YELLOW=$'\033[0;33m'
  BLUE=$'\033[0;34m'
  BOLD=$'\033[1m'
  RESET=$'\033[0m'
else
  RED=""
  GREEN=""
  YELLOW=""
  BLUE=""
  BOLD=""
  RESET=""
fi

# Track which step is running so we can report failure clearly
CURRENT_STEP=""

log_step() {
  CURRENT_STEP="$1"
  echo ""
  echo "${BOLD}${BLUE}[verify]${RESET} ${BOLD}$1${RESET}"
}

log_ok() {
  echo "${GREEN}  ✓ $1${RESET}"
}

log_fail() {
  echo ""
  echo "${RED}${BOLD}[verify] FAILED at: ${CURRENT_STEP}${RESET}" >&2
  echo "${RED}  $1${RESET}" >&2
  exit 1
}

trap 'log_fail "Unexpected error — check output above"' ERR

START_TIME=$(date +%s)

echo "${BOLD}======================================${RESET}"
echo "${BOLD} Loom verification gate${RESET}"
echo "${BOLD}======================================${RESET}"

# ---------------------------------------------------------------------------
# Step 1: Dependency sanity check
# ---------------------------------------------------------------------------
log_step "1/7 Dependency sanity"

if [[ ! -d node_modules ]]; then
  log_fail "node_modules not found. Run: npm ci"
fi

# Fail if package-lock.json is older than package.json
if [[ package.json -nt package-lock.json ]]; then
  log_fail "package.json is newer than package-lock.json. Run: npm install"
fi

log_ok "Dependencies look fresh"

# ---------------------------------------------------------------------------
# Step 2: Lint
# ---------------------------------------------------------------------------
log_step "2/7 Lint (ESLint, zero warnings allowed)"

# --max-warnings 0 makes ANY warning a failure.
npm run lint -- --max-warnings 0
log_ok "Zero lint warnings"

# ---------------------------------------------------------------------------
# Step 3: Format check
# ---------------------------------------------------------------------------
log_step "3/7 Format (Prettier)"

npm run format -- --check
log_ok "Formatted correctly"

# ---------------------------------------------------------------------------
# Step 4: Type check
# ---------------------------------------------------------------------------
log_step "4/7 Type check (tsc --noEmit, strict mode)"

npm run typecheck
log_ok "Types clean"

# ---------------------------------------------------------------------------
# Step 5: Unit tests with coverage gates
# ---------------------------------------------------------------------------
log_step "5/7 Unit tests (coverage gates enforced)"

# Coverage gates are configured in vitest.config.ts via
# coverage.thresholds.lines, branches, functions, statements per module.
# If any gate is not met, vitest exits non-zero.
npm run test:unit
log_ok "Unit tests green, coverage gates met"

# ---------------------------------------------------------------------------
# Step 6: Integration tests
# ---------------------------------------------------------------------------
log_step "6/7 Integration tests"

npm run test:integration
log_ok "Integration tests green"

# ---------------------------------------------------------------------------
# Step 7: Behavioral tests
# ---------------------------------------------------------------------------
log_step "7/7 Behavioral tests"

npm run test:behavioral
log_ok "Behavioral tests green"

# ---------------------------------------------------------------------------
# Optional: E2E tests (Playwright Electron)
# ---------------------------------------------------------------------------
# E2E is opt-in because it's slow. Enable by setting VERIFY_WITH_E2E=1.
if [[ "${VERIFY_WITH_E2E:-0}" == "1" ]]; then
  log_step "bonus: E2E tests (Playwright Electron)"
  npm run test:e2e
  log_ok "E2E green"
fi

# ---------------------------------------------------------------------------
# Success
# ---------------------------------------------------------------------------
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo "${BOLD}${GREEN}======================================${RESET}"
echo "${BOLD}${GREEN} VERIFY PASSED (${DURATION}s)${RESET}"
echo "${BOLD}${GREEN}======================================${RESET}"
echo ""
echo "  Ready to commit."
echo ""
