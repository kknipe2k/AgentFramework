# Advanced Test Suite Generation

> Future skill: Auto-generate thorough test suites for any app built with ARIA

## Principle

**If it's an app, it needs thorough testing - no matter how small.**

Broken is broken. A 50-line prototype and a 50,000-line enterprise app both need to know when something breaks.

## Test Coverage by App Type

| App Type | Test Suite |
|----------|------------|
| **Python** | pytest, type checking (mypy), linting (ruff/flake8) |
| **React/JS/TS** | Jest/Vitest, ESLint, TypeScript strict, Playwright for UI |
| **HTML/CSS** | HTML validation, CSS linting, accessibility (axe), Playwright |
| **API** | Endpoint tests, schema validation, error handling, auth flows |
| **CLI** | Command tests, flag combinations, error cases, help output |

## What Gets Generated

For ANY prototype or app:

1. **Test runner** - Cross-platform, discovers and runs all tests
2. **Unit tests** - Every function/component gets tested
3. **Integration tests** - Data flows, API calls, state management
4. **E2E tests** - User journeys via Playwright (if UI exists)
5. **Linting** - Code quality enforcement
6. **Type checking** - Static analysis where applicable
7. **Accessibility** - WCAG compliance for UI apps

## Size Affects Quantity, Not Coverage

- **Small app** = Fewer tests (less code to test)
- **Large app** = More tests (more code to test)
- **Both** = 100% coverage of what exists

## Stack Detection

The skill will auto-detect the stack:

```
package.json exists     → Node/React/JS tests
requirements.txt exists → Python tests
Cargo.toml exists       → Rust tests
go.mod exists           → Go tests
*.html files exist      → HTML/Playwright tests
```

## Generated Structure

```
tests/
├── run-tests.py          # Cross-platform runner
├── unit/                  # Unit tests
│   ├── test_*.py         # Python
│   └── *.test.ts         # TypeScript
├── integration/           # Integration tests
│   └── test_*.py
├── e2e/                   # End-to-end (Playwright)
│   ├── playwright.config.js
│   └── *.spec.js
├── fixtures/              # Test data
└── README.md              # How to run tests
```

## CI Integration

Auto-generate GitHub Actions workflow:

```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run tests
        run: python tests/run-tests.py
```

## Trigger Points

Generate test suite:
- After prototype is built (Research flow)
- After initial implementation (Build flow)
- When `verify.sh` doesn't exist
- On user request: `/aria:generate-tests`

## Future Skill

This will become `.aria/skills/test-generation.md` that:

1. Detects app type from file structure
2. Generates appropriate test infrastructure
3. Creates initial test cases based on code analysis
4. Integrates with verification gate

## Status

**Documented** - Awaiting implementation as skill

---

*ARIA Advanced Test Suite - Thorough testing for every app*
*Created: 2026-01-19*
