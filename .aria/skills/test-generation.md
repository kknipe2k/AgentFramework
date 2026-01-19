# Test Generation Skill

> Auto-generate comprehensive test suites for any app built with ARIA

---
version: 1.0.0
modes: [LITE, STANDARD, FULL, FULL+]
triggers: ["generate tests", "create test suite", "add tests", "/aria:generate-tests"]
inputs: [source code, app type, existing tests]
outputs: [test files, test runner, CI workflow, coverage config]
dependencies: []
---

## Principle

**If it's an app, it needs thorough testing - no matter how small.**

Broken is broken. A 50-line prototype and a 50,000-line enterprise app both need to know when something breaks. Size affects quantity of tests, not coverage of what exists.

---

## When to Use

Use this skill when:
- Building a new prototype (after initial implementation)
- `verify.sh` doesn't exist or is minimal
- Starting work on existing codebase without tests
- User requests test generation
- After Research flow produces a prototype

**Skip when:**
- Tests already exist and are comprehensive
- Pure documentation/config changes
- Exploratory work that won't be kept

---

## Workflow

### Step 1: Detect Stack

Run stack detection to identify the project type:

```bash
python .aria/lib/detect-stack.py
```

**Output:**
```json
{
  "primary_stack": "react-typescript",
  "detected": {
    "has_package_json": true,
    "has_typescript": true,
    "has_react": true,
    "has_python": false,
    "has_rust": false,
    "has_go": false,
    "has_html": true
  },
  "recommended_tools": {
    "test_framework": "vitest",
    "linter": "eslint",
    "type_checker": "typescript",
    "e2e": "playwright"
  }
}
```

**Stack Detection Rules:**

| Files Present | Stack | Test Framework |
|---------------|-------|----------------|
| `package.json` + `tsconfig.json` + react | React/TS | Vitest + Playwright |
| `package.json` + `tsconfig.json` | Node/TS | Vitest |
| `package.json` only | Node/JS | Jest |
| `requirements.txt` or `pyproject.toml` | Python | pytest |
| `Cargo.toml` | Rust | cargo test |
| `go.mod` | Go | go test |
| `*.html` only | HTML/CSS | Playwright |

---

### Step 2: Generate Test Infrastructure

Based on detected stack, generate:

```
[project]/
├── tests/
│   ├── run-tests.py          # Cross-platform runner
│   ├── unit/                  # Unit tests
│   ├── integration/           # Integration tests
│   ├── e2e/                   # E2E tests (if UI)
│   └── fixtures/              # Test data
├── [test config]              # Jest/Vitest/pytest config
└── .github/workflows/test.yml # CI workflow
```

**HITL Checkpoint:**
```
HITL: Test infrastructure will be created:
- tests/ directory with [X] test files
- [config file] configuration
- GitHub Actions workflow

Proceed? [y]es / [n]o / [c]ustomize
```

---

### Step 3: Analyze Code for Test Cases

For each source file, identify:

1. **Functions/Methods** - Generate unit tests
2. **Components** (React) - Generate component tests
3. **API Endpoints** - Generate integration tests
4. **User Flows** - Generate E2E tests

**Analysis Output:**
```
FILE ANALYSIS: src/utils/math.ts

Functions found:
  - add(a, b) → unit test: positive, negative, zero, edge cases
  - divide(a, b) → unit test: normal, divide-by-zero error
  - calculateTax(amount, rate) → unit test: rates, boundaries

Recommended tests: 8 unit tests
```

---

### Step 4: Generate Tests

Generate test files based on analysis:

**Unit Test Template (TypeScript/Vitest):**
```typescript
import { describe, it, expect } from 'vitest';
import { add, divide } from '../src/utils/math';

describe('math utilities', () => {
  describe('add', () => {
    it('adds two positive numbers', () => {
      expect(add(2, 3)).toBe(5);
    });

    it('adds negative numbers', () => {
      expect(add(-2, -3)).toBe(-5);
    });

    it('adds zero', () => {
      expect(add(5, 0)).toBe(5);
    });
  });

  describe('divide', () => {
    it('divides two numbers', () => {
      expect(divide(10, 2)).toBe(5);
    });

    it('throws on division by zero', () => {
      expect(() => divide(10, 0)).toThrow();
    });
  });
});
```

**Unit Test Template (Python/pytest):**
```python
import pytest
from src.utils.math import add, divide

class TestMath:
    def test_add_positive_numbers(self):
        assert add(2, 3) == 5

    def test_add_negative_numbers(self):
        assert add(-2, -3) == -5

    def test_divide_normal(self):
        assert divide(10, 2) == 5

    def test_divide_by_zero_raises(self):
        with pytest.raises(ZeroDivisionError):
            divide(10, 0)
```

---

### Step 5: Generate CI Workflow

**GitHub Actions (.github/workflows/test.yml):**
```yaml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js  # or Python, Rust, etc.
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies
        run: npm ci

      - name: Run tests
        run: npm test

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        if: always()
```

---

### Step 6: Update verify.sh

Add test commands to `.aria/verify.sh`:

```bash
#!/bin/bash
set -e

echo "Running tests..."

# Stack-specific test command
npm test              # Node/React
# pytest              # Python
# cargo test          # Rust
# go test ./...       # Go

echo "Running linter..."
npm run lint          # or equivalent

echo "Type checking..."
npx tsc --noEmit      # TypeScript

echo "All checks passed!"
```

---

## Mode Variations

### LITE Mode

Quick test generation for small apps:

```
LITE TEST GENERATION:
- Unit tests for exported functions only
- Skip E2E tests
- Basic CI workflow
- No coverage requirements

Output: ~5-10 tests
```

### STANDARD Mode

Balanced test coverage:

```
STANDARD TEST GENERATION:
- Unit tests for all functions
- Integration tests for APIs/data flows
- E2E tests for main user flow (if UI)
- 70% coverage target
- Full CI workflow

Output: ~20-50 tests
```

### FULL/FULL+ Mode

Comprehensive test suite:

```
FULL TEST GENERATION:
- Unit tests with edge cases
- Integration tests for all APIs
- E2E tests for all user flows
- Accessibility tests (if UI)
- Performance benchmarks
- 80%+ coverage target
- Multi-platform CI

Output: ~50-100+ tests
```

---

## Test Types by App Category

### Web App (React/Vue/etc.)

| Level | What to Test | Tool |
|-------|--------------|------|
| Unit | Utils, hooks, pure functions | Vitest/Jest |
| Component | React components in isolation | Testing Library |
| Integration | Component interactions, state | Testing Library |
| E2E | Full user flows | Playwright |
| Accessibility | WCAG compliance | axe-playwright |

### API/Backend

| Level | What to Test | Tool |
|-------|--------------|------|
| Unit | Business logic, utilities | pytest/Jest |
| Integration | Database queries, services | pytest/Jest |
| API | Endpoint responses, errors | supertest/httpx |
| Contract | API schema validation | Pact/OpenAPI |

### CLI Tool

| Level | What to Test | Tool |
|-------|--------------|------|
| Unit | Core functions | pytest/Jest |
| CLI | Commands, flags, output | subprocess tests |
| Integration | File I/O, system calls | tmp directories |
| Error | Invalid inputs, edge cases | error assertions |

### HTML/CSS Prototype

| Level | What to Test | Tool |
|-------|--------------|------|
| Validation | HTML validity | html-validate |
| Lint | CSS quality | stylelint |
| Visual | Rendering, layout | Playwright screenshots |
| A11y | Accessibility | axe |
| E2E | User interactions | Playwright |

---

## HITL Checkpoints

Before these actions, stop and confirm:

- [ ] Overwriting existing test files
- [ ] Modifying verify.sh
- [ ] Adding new dependencies
- [ ] Creating CI workflow

**Format:**
```
HITL CHECKPOINT: About to [action]
Files affected: [list]
Proceed? [y]es / [n]o / [e]xplain
```

---

## Generated File Templates

### Cross-Platform Test Runner (run-tests.py)

```python
#!/usr/bin/env python3
"""Cross-platform test runner for ARIA projects."""

import subprocess
import sys
import json
from pathlib import Path

def detect_stack():
    """Detect project stack from files."""
    if Path('package.json').exists():
        return 'node'
    if Path('requirements.txt').exists() or Path('pyproject.toml').exists():
        return 'python'
    if Path('Cargo.toml').exists():
        return 'rust'
    if Path('go.mod').exists():
        return 'go'
    return 'unknown'

def run_tests():
    """Run tests based on detected stack."""
    stack = detect_stack()

    commands = {
        'node': ['npm', 'test'],
        'python': ['pytest', '-v'],
        'rust': ['cargo', 'test'],
        'go': ['go', 'test', './...'],
    }

    cmd = commands.get(stack)
    if not cmd:
        print(f"Unknown stack, cannot run tests")
        return 1

    print(f"Running {stack} tests...")
    result = subprocess.run(cmd)
    return result.returncode

if __name__ == '__main__':
    sys.exit(run_tests())
```

---

## Integration with Other Skills

**From prototyping:**
- Receives completed prototype
- Generates tests for prototype code

**From executing:**
- Called after implementation tasks
- Adds tests for new code

**To debugging:**
- Provides failing test as reproduction
- Enables quick iteration

**To verify.sh:**
- Updates verification gate
- Ensures tests run on every change

---

## Output

After test generation:

```markdown
## Test Suite Generated

**Stack:** [detected stack]
**Tests Created:** [count]

### Files Created:
- tests/unit/test_[module].py (X tests)
- tests/integration/test_[feature].py (Y tests)
- tests/e2e/[flow].spec.ts (Z tests)

### Configuration:
- [config file] created
- verify.sh updated
- .github/workflows/test.yml created

### Coverage Target: [X]%

### Run Tests:
```bash
python tests/run-tests.py
# or
npm test
# or
pytest
```

### Next Steps:
1. Review generated tests
2. Add edge cases as needed
3. Run tests to verify baseline
```

---

## Tips

- **Start with unit tests** - They're fastest to write and run
- **Mock external services** - Don't depend on network in tests
- **Use fixtures** - Share test data across tests
- **Name tests clearly** - Test names are documentation
- **Test behavior, not implementation** - Tests should survive refactoring
- **Keep E2E tests focused** - Test critical paths, not everything
- **Run tests locally first** - Before pushing to CI

---

*See [REGISTRY.md](./REGISTRY.md) for skill index*
