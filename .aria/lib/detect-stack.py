#!/usr/bin/env python3
"""
Stack Detection Library for ARIA Test Generation

Detects project type and recommends appropriate testing tools.
"""

import json
import sys
from pathlib import Path
from typing import Optional


def detect_stack(project_dir: Optional[Path] = None) -> dict:
    """
    Detect the project stack from files present.

    Returns a dict with:
    - primary_stack: The main technology stack
    - detected: Dict of boolean flags for each technology
    - recommended_tools: Testing tools to use
    - files_found: List of relevant files found
    """
    if project_dir is None:
        project_dir = Path.cwd()
    else:
        project_dir = Path(project_dir)

    # Detection flags
    detected = {
        "has_package_json": False,
        "has_typescript": False,
        "has_react": False,
        "has_vue": False,
        "has_angular": False,
        "has_python": False,
        "has_rust": False,
        "has_go": False,
        "has_java": False,
        "has_html": False,
        "has_existing_tests": False,
    }

    files_found = []

    # Check for Node.js/JavaScript/TypeScript
    package_json = project_dir / "package.json"
    if package_json.exists():
        detected["has_package_json"] = True
        files_found.append("package.json")

        try:
            with open(package_json, 'r', encoding='utf-8') as f:
                pkg = json.load(f)
                deps = {**pkg.get("dependencies", {}), **pkg.get("devDependencies", {})}

                if "react" in deps or "react-dom" in deps:
                    detected["has_react"] = True
                if "vue" in deps:
                    detected["has_vue"] = True
                if "@angular/core" in deps:
                    detected["has_angular"] = True
                if "typescript" in deps:
                    detected["has_typescript"] = True

                # Check for existing test setup
                if any(test_pkg in deps for test_pkg in ["jest", "vitest", "mocha", "@testing-library/react", "playwright"]):
                    detected["has_existing_tests"] = True
        except (json.JSONDecodeError, IOError):
            pass

    # Check for TypeScript config
    if (project_dir / "tsconfig.json").exists():
        detected["has_typescript"] = True
        files_found.append("tsconfig.json")

    # Check for Python
    if (project_dir / "requirements.txt").exists():
        detected["has_python"] = True
        files_found.append("requirements.txt")
    if (project_dir / "pyproject.toml").exists():
        detected["has_python"] = True
        files_found.append("pyproject.toml")
    if (project_dir / "setup.py").exists():
        detected["has_python"] = True
        files_found.append("setup.py")

    # Check for existing pytest
    if (project_dir / "pytest.ini").exists() or (project_dir / "conftest.py").exists():
        detected["has_existing_tests"] = True
        files_found.append("pytest.ini" if (project_dir / "pytest.ini").exists() else "conftest.py")

    # Check for Rust
    if (project_dir / "Cargo.toml").exists():
        detected["has_rust"] = True
        files_found.append("Cargo.toml")

    # Check for Go
    if (project_dir / "go.mod").exists():
        detected["has_go"] = True
        files_found.append("go.mod")

    # Check for Java
    if (project_dir / "pom.xml").exists() or (project_dir / "build.gradle").exists():
        detected["has_java"] = True
        files_found.append("pom.xml" if (project_dir / "pom.xml").exists() else "build.gradle")

    # Check for HTML files (static site or prototype)
    html_files = list(project_dir.glob("*.html")) + list(project_dir.glob("**/*.html"))
    # Filter out node_modules
    html_files = [f for f in html_files if "node_modules" not in str(f)]
    if html_files:
        detected["has_html"] = True
        files_found.append(f"{len(html_files)} HTML file(s)")

    # Check for existing tests directory
    if (project_dir / "tests").is_dir() or (project_dir / "test").is_dir() or (project_dir / "__tests__").is_dir():
        detected["has_existing_tests"] = True

    # Determine primary stack
    primary_stack = determine_primary_stack(detected)

    # Recommend tools
    recommended_tools = recommend_tools(detected, primary_stack)

    return {
        "primary_stack": primary_stack,
        "detected": detected,
        "recommended_tools": recommended_tools,
        "files_found": files_found,
        "project_dir": str(project_dir),
    }


def determine_primary_stack(detected: dict) -> str:
    """Determine the primary technology stack."""

    # React/TypeScript is most specific
    if detected["has_react"] and detected["has_typescript"]:
        return "react-typescript"

    if detected["has_react"]:
        return "react-javascript"

    if detected["has_vue"] and detected["has_typescript"]:
        return "vue-typescript"

    if detected["has_vue"]:
        return "vue-javascript"

    if detected["has_angular"]:
        return "angular"

    if detected["has_typescript"] and detected["has_package_json"]:
        return "node-typescript"

    if detected["has_package_json"]:
        return "node-javascript"

    if detected["has_python"]:
        return "python"

    if detected["has_rust"]:
        return "rust"

    if detected["has_go"]:
        return "go"

    if detected["has_java"]:
        return "java"

    if detected["has_html"]:
        return "html-css"

    return "unknown"


def recommend_tools(detected: dict, primary_stack: str) -> dict:
    """Recommend testing tools based on stack."""

    tools = {
        "test_framework": None,
        "linter": None,
        "type_checker": None,
        "e2e": None,
        "coverage": None,
    }

    if primary_stack in ["react-typescript", "react-javascript", "vue-typescript", "vue-javascript"]:
        tools["test_framework"] = "vitest"
        tools["linter"] = "eslint"
        tools["e2e"] = "playwright"
        tools["coverage"] = "v8"
        if "typescript" in primary_stack:
            tools["type_checker"] = "typescript"

    elif primary_stack in ["node-typescript", "node-javascript"]:
        tools["test_framework"] = "vitest"
        tools["linter"] = "eslint"
        tools["coverage"] = "v8"
        if "typescript" in primary_stack:
            tools["type_checker"] = "typescript"

    elif primary_stack == "angular":
        tools["test_framework"] = "karma"
        tools["linter"] = "eslint"
        tools["type_checker"] = "typescript"
        tools["e2e"] = "playwright"
        tools["coverage"] = "istanbul"

    elif primary_stack == "python":
        tools["test_framework"] = "pytest"
        tools["linter"] = "ruff"
        tools["type_checker"] = "mypy"
        tools["coverage"] = "pytest-cov"

    elif primary_stack == "rust":
        tools["test_framework"] = "cargo-test"
        tools["linter"] = "clippy"
        tools["coverage"] = "tarpaulin"

    elif primary_stack == "go":
        tools["test_framework"] = "go-test"
        tools["linter"] = "golangci-lint"
        tools["coverage"] = "go-cover"

    elif primary_stack == "java":
        tools["test_framework"] = "junit"
        tools["linter"] = "checkstyle"
        tools["coverage"] = "jacoco"

    elif primary_stack == "html-css":
        tools["test_framework"] = "playwright"
        tools["linter"] = "htmlhint"
        tools["e2e"] = "playwright"

    return tools


def print_report(result: dict) -> None:
    """Print a human-readable report."""
    print("=" * 60)
    print("  STACK DETECTION REPORT")
    print("=" * 60)
    print()
    print(f"  Project: {result['project_dir']}")
    print(f"  Primary Stack: {result['primary_stack']}")
    print()

    print("  Detected:")
    for key, value in result["detected"].items():
        if value:
            clean_key = key.replace("has_", "").replace("_", " ").title()
            print(f"    [x] {clean_key}")
    print()

    print("  Files Found:")
    for f in result["files_found"]:
        print(f"    - {f}")
    print()

    print("  Recommended Tools:")
    for key, value in result["recommended_tools"].items():
        if value:
            clean_key = key.replace("_", " ").title()
            print(f"    {clean_key}: {value}")
    print()
    print("=" * 60)


def main():
    """Main entry point."""
    import argparse

    parser = argparse.ArgumentParser(description="Detect project stack for test generation")
    parser.add_argument("path", nargs="?", default=".", help="Project directory to analyze")
    parser.add_argument("--json", action="store_true", help="Output as JSON")

    args = parser.parse_args()

    result = detect_stack(Path(args.path))

    if args.json:
        print(json.dumps(result, indent=2))
    else:
        print_report(result)

    return 0


if __name__ == "__main__":
    sys.exit(main())
