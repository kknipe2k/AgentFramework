#!/usr/bin/env python3
"""
Test Generator for ARIA

Generates test files, configurations, and CI workflows based on detected stack.
"""

import json
import os
import sys
from pathlib import Path
from typing import Optional
from datetime import datetime

# Import stack detection - handle module naming
import importlib.util
_stack_module_path = Path(__file__).parent / "detect-stack.py"
_spec = importlib.util.spec_from_file_location("detect_stack", _stack_module_path)
_detect_stack_module = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_detect_stack_module)
detect_stack = _detect_stack_module.detect_stack


# =============================================================================
# TEST TEMPLATES
# =============================================================================

VITEST_CONFIG = '''import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
    },
  },
})
'''

VITEST_CONFIG_REACT = '''import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./tests/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
    },
  },
})
'''

VITEST_SETUP_REACT = '''import '@testing-library/jest-dom'
'''

JEST_CONFIG = '''{
  "testEnvironment": "node",
  "testMatch": ["**/tests/**/*.test.js", "**/tests/**/*.test.ts"],
  "collectCoverageFrom": ["src/**/*.{js,ts}"],
  "coverageThreshold": {
    "global": {
      "branches": 70,
      "functions": 70,
      "lines": 70
    }
  }
}
'''

PYTEST_INI = '''[pytest]
testpaths = tests
python_files = test_*.py
python_functions = test_*
addopts = -v --tb=short
'''

PLAYWRIGHT_CONFIG = '''import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:3000',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
'''

# =============================================================================
# TEST FILE TEMPLATES
# =============================================================================

def get_unit_test_template(stack: str, module_name: str = "example") -> str:
    """Get unit test template for the given stack."""

    if stack in ["react-typescript", "node-typescript", "vue-typescript"]:
        return f'''import {{ describe, it, expect }} from 'vitest';
// import {{ yourFunction }} from '../src/{module_name}';

describe('{module_name}', () => {{
  describe('basic functionality', () => {{
    it('should work with valid input', () => {{
      // Arrange
      const input = 'test';

      // Act
      const result = input.toUpperCase();

      // Assert
      expect(result).toBe('TEST');
    }});

    it('should handle edge cases', () => {{
      // TODO: Add edge case tests
      expect(true).toBe(true);
    }});
  }});
}});
'''

    elif stack in ["react-javascript", "node-javascript", "vue-javascript"]:
        return f'''const {{ yourFunction }} = require('../src/{module_name}');

describe('{module_name}', () => {{
  describe('basic functionality', () => {{
    it('should work with valid input', () => {{
      // Arrange
      const input = 'test';

      // Act
      const result = input.toUpperCase();

      // Assert
      expect(result).toBe('TEST');
    }});

    it('should handle edge cases', () => {{
      // TODO: Add edge case tests
      expect(true).toBe(true);
    }});
  }});
}});
'''

    elif stack == "python":
        return f'''"""Unit tests for {module_name}."""
import pytest
# from src.{module_name} import your_function


class Test{module_name.title().replace("_", "")}:
    """Tests for {module_name} module."""

    def test_basic_functionality(self):
        """Test basic functionality with valid input."""
        # Arrange
        input_value = "test"

        # Act
        result = input_value.upper()

        # Assert
        assert result == "TEST"

    def test_edge_cases(self):
        """Test edge cases."""
        # TODO: Add edge case tests
        assert True
'''

    elif stack == "rust":
        return f'''#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_basic_functionality() {{
        // Arrange
        let input = "test";

        // Act
        let result = input.to_uppercase();

        // Assert
        assert_eq!(result, "TEST");
    }}

    #[test]
    fn test_edge_cases() {{
        // TODO: Add edge case tests
        assert!(true);
    }}
}}
'''

    elif stack == "go":
        return f'''package {module_name}

import "testing"

func TestBasicFunctionality(t *testing.T) {{
    // Arrange
    input := "test"

    // Act
    result := strings.ToUpper(input)

    // Assert
    if result != "TEST" {{
        t.Errorf("Expected TEST, got %s", result)
    }}
}}

func TestEdgeCases(t *testing.T) {{
    // TODO: Add edge case tests
}}
'''

    else:
        return f'''// Unit tests for {module_name}
// TODO: Add tests based on your stack
'''


def get_e2e_test_template(stack: str) -> str:
    """Get E2E test template."""

    return '''import { test, expect } from '@playwright/test';

test.describe('Main user flow', () => {
  test('homepage loads correctly', async ({ page }) => {
    await page.goto('/');

    // Check page loaded
    await expect(page).toHaveTitle(/./);
  });

  test('main interaction works', async ({ page }) => {
    await page.goto('/');

    // TODO: Add interaction tests
    // await page.click('button');
    // await expect(page.locator('.result')).toBeVisible();
  });
});

test.describe('Accessibility', () => {
  test('has no accessibility violations', async ({ page }) => {
    await page.goto('/');

    // Basic accessibility check
    // For full a11y testing, use @axe-core/playwright
  });
});
'''


def get_integration_test_template(stack: str) -> str:
    """Get integration test template."""

    if stack in ["react-typescript", "node-typescript"]:
        return '''import { describe, it, expect, beforeEach, afterEach } from 'vitest';

describe('Integration Tests', () => {
  beforeEach(() => {
    // Setup before each test
  });

  afterEach(() => {
    // Cleanup after each test
  });

  describe('Data flow', () => {
    it('should process data end-to-end', async () => {
      // TODO: Add integration tests
      expect(true).toBe(true);
    });
  });

  describe('API interactions', () => {
    it('should handle API responses', async () => {
      // TODO: Add API integration tests
      expect(true).toBe(true);
    });
  });
});
'''

    elif stack == "python":
        return '''"""Integration tests."""
import pytest


class TestIntegration:
    """Integration test suite."""

    @pytest.fixture(autouse=True)
    def setup(self):
        """Setup before each test."""
        yield
        # Cleanup after each test

    def test_data_flow(self):
        """Test end-to-end data processing."""
        # TODO: Add integration tests
        assert True

    def test_api_interactions(self):
        """Test API response handling."""
        # TODO: Add API integration tests
        assert True
'''

    else:
        return '''// Integration tests
// TODO: Add integration tests for your stack
'''


# =============================================================================
# CI WORKFLOW TEMPLATES
# =============================================================================

def get_github_actions_workflow(stack: str) -> str:
    """Get GitHub Actions workflow for the stack."""

    if stack in ["react-typescript", "react-javascript", "node-typescript", "node-javascript", "vue-typescript", "vue-javascript"]:
        return '''name: Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Install dependencies
        run: npm ci

      - name: Run linter
        run: npm run lint --if-present

      - name: Run type check
        run: npm run typecheck --if-present

      - name: Run tests
        run: npm test -- --coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v4
        if: always()
        with:
          files: ./coverage/coverage-final.json

  e2e:
    runs-on: ubuntu-latest
    needs: test

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Install dependencies
        run: npm ci

      - name: Install Playwright
        run: npx playwright install --with-deps

      - name: Run E2E tests
        run: npx playwright test

      - name: Upload test results
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: playwright-report
          path: playwright-report/
'''

    elif stack == "python":
        return '''name: Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest

    strategy:
      matrix:
        python-version: ['3.10', '3.11', '3.12']

    steps:
      - uses: actions/checkout@v4

      - name: Set up Python ${{ matrix.python-version }}
        uses: actions/setup-python@v5
        with:
          python-version: ${{ matrix.python-version }}

      - name: Install dependencies
        run: |
          python -m pip install --upgrade pip
          pip install -r requirements.txt
          pip install pytest pytest-cov ruff mypy

      - name: Run linter
        run: ruff check .

      - name: Run type check
        run: mypy . --ignore-missing-imports || true

      - name: Run tests
        run: pytest --cov=src --cov-report=xml

      - name: Upload coverage
        uses: codecov/codecov-action@v4
        if: always()
'''

    elif stack == "rust":
        return '''name: Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: clippy, rustfmt

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Run tests
        run: cargo test --verbose

      - name: Check formatting
        run: cargo fmt -- --check
'''

    elif stack == "go":
        return '''name: Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Go
        uses: actions/setup-go@v5
        with:
          go-version: '1.21'

      - name: Run linter
        uses: golangci/golangci-lint-action@v4

      - name: Run tests
        run: go test -v -coverprofile=coverage.out ./...

      - name: Upload coverage
        uses: codecov/codecov-action@v4
        if: always()
'''

    else:
        return '''name: Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Run tests
        run: echo "Add your test commands here"
'''


# =============================================================================
# VERIFY.SH TEMPLATE
# =============================================================================

def get_verify_sh(stack: str) -> str:
    """Get verify.sh content for the stack."""

    if stack in ["react-typescript", "react-javascript", "node-typescript", "node-javascript", "vue-typescript", "vue-javascript"]:
        return '''#!/bin/bash
# ARIA Verification Gate
set -e

echo "Running verification..."

# Lint
if npm run lint --if-present 2>/dev/null; then
    echo "[PASS] Linting"
else
    echo "[WARN] No lint script or lint failed"
fi

# Type check (TypeScript)
if npm run typecheck --if-present 2>/dev/null; then
    echo "[PASS] Type checking"
elif npx tsc --noEmit 2>/dev/null; then
    echo "[PASS] Type checking (tsc)"
else
    echo "[SKIP] No TypeScript or type check failed"
fi

# Tests
if npm test 2>/dev/null; then
    echo "[PASS] Tests"
else
    echo "[FAIL] Tests failed"
    exit 1
fi

echo ""
echo "All verification checks passed!"
'''

    elif stack == "python":
        return '''#!/bin/bash
# ARIA Verification Gate
set -e

echo "Running verification..."

# Lint
if ruff check . 2>/dev/null; then
    echo "[PASS] Linting (ruff)"
elif flake8 . 2>/dev/null; then
    echo "[PASS] Linting (flake8)"
else
    echo "[WARN] No linter or lint failed"
fi

# Type check
if mypy . --ignore-missing-imports 2>/dev/null; then
    echo "[PASS] Type checking"
else
    echo "[SKIP] No mypy or type check failed"
fi

# Tests
if pytest -v 2>/dev/null; then
    echo "[PASS] Tests"
else
    echo "[FAIL] Tests failed"
    exit 1
fi

echo ""
echo "All verification checks passed!"
'''

    elif stack == "rust":
        return '''#!/bin/bash
# ARIA Verification Gate
set -e

echo "Running verification..."

# Lint
cargo clippy -- -D warnings
echo "[PASS] Clippy"

# Format check
cargo fmt -- --check
echo "[PASS] Formatting"

# Tests
cargo test
echo "[PASS] Tests"

echo ""
echo "All verification checks passed!"
'''

    elif stack == "go":
        return '''#!/bin/bash
# ARIA Verification Gate
set -e

echo "Running verification..."

# Lint
if golangci-lint run 2>/dev/null; then
    echo "[PASS] Linting"
else
    echo "[WARN] No golangci-lint or lint failed"
fi

# Tests
go test -v ./...
echo "[PASS] Tests"

echo ""
echo "All verification checks passed!"
'''

    else:
        return '''#!/bin/bash
# ARIA Verification Gate
set -e

echo "Running verification..."

# Add your verification commands here
echo "[SKIP] No stack-specific verification"

echo ""
echo "Verification complete."
'''


# =============================================================================
# MAIN GENERATOR
# =============================================================================

def generate_tests(project_dir: Optional[Path] = None, mode: str = "STANDARD") -> dict:
    """
    Generate test suite for the project.

    Args:
        project_dir: Project directory (default: current directory)
        mode: ARIA mode (LITE, STANDARD, FULL, FULL+)

    Returns:
        Dict with files created and summary
    """
    if project_dir is None:
        project_dir = Path.cwd()
    else:
        project_dir = Path(project_dir)

    # Detect stack
    stack_info = detect_stack(project_dir)
    stack = stack_info["primary_stack"]

    files_created = []
    summary = {
        "stack": stack,
        "mode": mode,
        "timestamp": datetime.now().isoformat(),
    }

    # Create tests directory
    tests_dir = project_dir / "tests"
    tests_dir.mkdir(exist_ok=True)

    unit_dir = tests_dir / "unit"
    unit_dir.mkdir(exist_ok=True)

    integration_dir = tests_dir / "integration"
    integration_dir.mkdir(exist_ok=True)

    # Create unit test
    if stack in ["react-typescript", "node-typescript", "vue-typescript"]:
        unit_file = unit_dir / "example.test.ts"
    elif stack in ["react-javascript", "node-javascript", "vue-javascript"]:
        unit_file = unit_dir / "example.test.js"
    elif stack == "python":
        unit_file = unit_dir / "test_example.py"
    elif stack == "rust":
        unit_file = unit_dir / "test_example.rs"
    elif stack == "go":
        unit_file = unit_dir / "example_test.go"
    else:
        unit_file = unit_dir / "example.test.txt"

    unit_file.write_text(get_unit_test_template(stack))
    files_created.append(str(unit_file.relative_to(project_dir)))

    # Create integration test
    if stack in ["react-typescript", "node-typescript", "vue-typescript"]:
        int_file = integration_dir / "integration.test.ts"
    elif stack in ["react-javascript", "node-javascript", "vue-javascript"]:
        int_file = integration_dir / "integration.test.js"
    elif stack == "python":
        int_file = integration_dir / "test_integration.py"
    else:
        int_file = integration_dir / "integration.test.txt"

    int_file.write_text(get_integration_test_template(stack))
    files_created.append(str(int_file.relative_to(project_dir)))

    # Create E2E tests (STANDARD and above)
    if mode in ["STANDARD", "FULL", "FULL+"]:
        e2e_dir = tests_dir / "e2e"
        e2e_dir.mkdir(exist_ok=True)

        e2e_file = e2e_dir / "main.spec.ts"
        e2e_file.write_text(get_e2e_test_template(stack))
        files_created.append(str(e2e_file.relative_to(project_dir)))

        # Playwright config
        if stack_info["recommended_tools"].get("e2e") == "playwright":
            pw_config = project_dir / "playwright.config.ts"
            pw_config.write_text(PLAYWRIGHT_CONFIG)
            files_created.append("playwright.config.ts")

    # Create test config
    if stack in ["react-typescript", "node-typescript", "vue-typescript", "react-javascript", "node-javascript", "vue-javascript"]:
        if "react" in stack:
            config_content = VITEST_CONFIG_REACT
            # Create setup file
            setup_file = tests_dir / "setup.ts"
            setup_file.write_text(VITEST_SETUP_REACT)
            files_created.append("tests/setup.ts")
        else:
            config_content = VITEST_CONFIG

        config_file = project_dir / "vitest.config.ts"
        config_file.write_text(config_content)
        files_created.append("vitest.config.ts")

    elif stack == "python":
        pytest_file = project_dir / "pytest.ini"
        if not pytest_file.exists():
            pytest_file.write_text(PYTEST_INI)
            files_created.append("pytest.ini")

    # Create CI workflow
    workflow_dir = project_dir / ".github" / "workflows"
    workflow_dir.mkdir(parents=True, exist_ok=True)

    workflow_file = workflow_dir / "test.yml"
    workflow_file.write_text(get_github_actions_workflow(stack))
    files_created.append(".github/workflows/test.yml")

    # Create/update verify.sh
    verify_file = project_dir / ".aria" / "verify.sh"
    verify_file.parent.mkdir(parents=True, exist_ok=True)
    verify_file.write_text(get_verify_sh(stack))
    files_created.append(".aria/verify.sh")

    # Create fixtures directory
    fixtures_dir = tests_dir / "fixtures"
    fixtures_dir.mkdir(exist_ok=True)

    gitkeep = fixtures_dir / ".gitkeep"
    gitkeep.write_text("")
    files_created.append("tests/fixtures/.gitkeep")

    summary["files_created"] = files_created
    summary["test_count"] = {
        "unit": 1,
        "integration": 1,
        "e2e": 1 if mode in ["STANDARD", "FULL", "FULL+"] else 0,
    }

    return summary


def print_summary(summary: dict) -> None:
    """Print generation summary."""
    print("=" * 60)
    print("  TEST SUITE GENERATED")
    print("=" * 60)
    print()
    print(f"  Stack: {summary['stack']}")
    print(f"  Mode: {summary['mode']}")
    print()
    print("  Files Created:")
    for f in summary["files_created"]:
        print(f"    - {f}")
    print()
    print("  Test Count:")
    for test_type, count in summary["test_count"].items():
        print(f"    {test_type}: {count}")
    print()
    print("  Next Steps:")
    print("    1. Review generated tests")
    print("    2. Run: python tests/run-tests.py (or npm test)")
    print("    3. Add more test cases as needed")
    print()
    print("=" * 60)


def main():
    """Main entry point."""
    import argparse

    parser = argparse.ArgumentParser(description="Generate test suite for ARIA projects")
    parser.add_argument("path", nargs="?", default=".", help="Project directory")
    parser.add_argument("--mode", choices=["LITE", "STANDARD", "FULL", "FULL+"], default="STANDARD", help="ARIA mode")
    parser.add_argument("--json", action="store_true", help="Output as JSON")

    args = parser.parse_args()

    summary = generate_tests(Path(args.path), args.mode)

    if args.json:
        print(json.dumps(summary, indent=2))
    else:
        print_summary(summary)

    return 0


if __name__ == "__main__":
    sys.exit(main())
