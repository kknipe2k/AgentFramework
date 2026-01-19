#!/usr/bin/env python3
"""
ARIA Test Suite Runner
Cross-platform test runner for Windows/Claude CLI compatibility.

Usage:
    python .aria/tests/run-tests.py [category] [--verbose] [--output FILE]

Categories: unit, integration, validation, all (default)
"""

import os
import sys
import json
import time
import subprocess
import argparse
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Tuple, Optional

# Configuration
SCRIPT_DIR = Path(__file__).parent
ARIA_DIR = SCRIPT_DIR.parent
PROJECT_ROOT = ARIA_DIR.parent

# Test directories
TEST_DIRS = {
    "unit": SCRIPT_DIR / "unit",
    "integration": SCRIPT_DIR / "integration",
    "validation": SCRIPT_DIR / "validation",
}

# Colors for terminal output
class Colors:
    HEADER = '\033[1;36m'  # Cyan bold
    SUCCESS = '\033[0;32m'  # Green
    ERROR = '\033[0;31m'    # Red
    WARNING = '\033[0;33m'  # Yellow
    INFO = '\033[0;34m'     # Blue
    RESET = '\033[0m'

def color(text: str, color_code: str) -> str:
    """Apply color if terminal supports it."""
    if sys.stdout.isatty():
        return f"{color_code}{text}{Colors.RESET}"
    return text

def print_header(text: str):
    print(color(text, Colors.HEADER))

def print_success(text: str):
    print(color(text, Colors.SUCCESS))

def print_error(text: str):
    print(color(text, Colors.ERROR))

def print_info(text: str):
    print(color(text, Colors.INFO))

def detect_platform() -> Dict:
    """Detect platform capabilities."""
    info = {
        "os": sys.platform,
        "python": f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}",
        "has_bash": False,
        "has_git_bash": False,
        "has_wsl": False,
        "has_claude_cli": False,
        "bash_path": None,
    }

    # Check for Git Bash on Windows
    git_bash_paths = [
        r"C:\Program Files\Git\bin\bash.exe",
        r"C:\Program Files (x86)\Git\bin\bash.exe",
        os.path.expandvars(r"%LOCALAPPDATA%\Programs\Git\bin\bash.exe"),
    ]

    for path in git_bash_paths:
        if os.path.exists(path):
            info["has_git_bash"] = True
            info["has_bash"] = True
            info["bash_path"] = path
            break

    # Check for WSL
    try:
        result = subprocess.run(
            ["wsl", "--status"],
            capture_output=True,
            timeout=5
        )
        info["has_wsl"] = result.returncode == 0
    except:
        pass

    # Check for Claude CLI
    try:
        result = subprocess.run(
            ["claude", "--version"],
            capture_output=True,
            timeout=5
        )
        info["has_claude_cli"] = result.returncode == 0
    except:
        pass

    return info

def discover_tests(test_dir: Path) -> List[Path]:
    """Discover test files in directory."""
    tests = []

    if not test_dir.exists():
        return tests

    # Find bash tests
    for f in sorted(test_dir.glob("test-*.sh")):
        tests.append(f)
    for f in sorted(test_dir.glob("test_*.sh")):
        tests.append(f)

    # Find Python tests
    for f in sorted(test_dir.glob("test_*.py")):
        tests.append(f)
    for f in sorted(test_dir.glob("test-*.py")):
        tests.append(f)

    return tests

def run_bash_test(test_file: Path, bash_path: str, verbose: bool = False) -> Tuple[bool, float, str]:
    """Run a bash test file."""
    start_time = time.time()

    try:
        # Convert Windows path to Unix-style for Git Bash
        unix_path = str(test_file).replace("\\", "/")

        # Set locale to avoid grep -P issues
        env = os.environ.copy()
        env["LC_ALL"] = "C.UTF-8"
        env["LANG"] = "C.UTF-8"

        result = subprocess.run(
            [bash_path, unix_path],
            capture_output=True,
            text=True,
            timeout=120,
            cwd=str(PROJECT_ROOT),
            env=env
        )

        elapsed = time.time() - start_time

        if result.returncode == 0:
            return True, elapsed, result.stdout
        else:
            error_output = result.stderr or result.stdout
            return False, elapsed, error_output

    except subprocess.TimeoutExpired:
        elapsed = time.time() - start_time
        return False, elapsed, "Test timed out after 120 seconds"
    except Exception as e:
        elapsed = time.time() - start_time
        return False, elapsed, str(e)

def run_python_test(test_file: Path, verbose: bool = False) -> Tuple[bool, float, str]:
    """Run a Python test file."""
    start_time = time.time()

    try:
        result = subprocess.run(
            [sys.executable, str(test_file)],
            capture_output=True,
            text=True,
            timeout=120,
            cwd=str(PROJECT_ROOT)
        )

        elapsed = time.time() - start_time

        if result.returncode == 0:
            return True, elapsed, result.stdout
        else:
            error_output = result.stderr or result.stdout
            return False, elapsed, error_output

    except subprocess.TimeoutExpired:
        elapsed = time.time() - start_time
        return False, elapsed, "Test timed out after 120 seconds"
    except Exception as e:
        elapsed = time.time() - start_time
        return False, elapsed, str(e)

def run_test_suite(
    categories: List[str],
    platform_info: Dict,
    verbose: bool = False
) -> Dict:
    """Run all tests in specified categories."""
    results = {
        "timestamp": datetime.now().isoformat(),
        "platform": platform_info,
        "suites": [],
        "total": 0,
        "passed": 0,
        "failed": 0,
        "skipped": 0,
    }

    for category in categories:
        test_dir = TEST_DIRS.get(category)
        if not test_dir:
            continue

        suite_result = {
            "name": category.capitalize(),
            "tests": [],
            "total": 0,
            "passed": 0,
            "failed": 0,
            "skipped": 0,
        }

        print_header(f"\n{'='*60}")
        print_header(f"  {category.upper()} TESTS")
        print_header(f"{'='*60}")

        tests = discover_tests(test_dir)

        # Also check root tests directory for legacy tests
        if category == "unit":
            legacy_tests = discover_tests(SCRIPT_DIR)
            # Filter to only include actual test files, not the runner
            legacy_tests = [t for t in legacy_tests if t.name != "test-runner.sh"]
            tests = legacy_tests + tests

        for test_file in tests:
            test_name = test_file.name
            print_info(f"Running: {test_name}")

            if test_file.suffix == ".sh":
                if not platform_info["has_bash"]:
                    print_error(f"  SKIP {test_name} (no bash available)")
                    suite_result["skipped"] += 1
                    continue

                success, elapsed, output = run_bash_test(
                    test_file,
                    platform_info["bash_path"],
                    verbose
                )
            else:
                success, elapsed, output = run_python_test(test_file, verbose)

            if success:
                print_success(f"  PASS {test_name} ({elapsed:.2f}s)")
                suite_result["passed"] += 1
            else:
                print_error(f"  FAIL {test_name} ({elapsed:.2f}s)")
                # Show first few lines of error
                error_lines = output.strip().split('\n')[:3]
                for line in error_lines:
                    if line.strip():
                        print_error(f"       {line.strip()}")
                suite_result["failed"] += 1

            suite_result["tests"].append({
                "name": test_name,
                "passed": success,
                "elapsed": elapsed,
                "output": output if not success else None,
            })
            suite_result["total"] += 1

        results["suites"].append(suite_result)
        results["total"] += suite_result["total"]
        results["passed"] += suite_result["passed"]
        results["failed"] += suite_result["failed"]
        results["skipped"] += suite_result["skipped"]

    return results

def print_summary(results: Dict):
    """Print test summary."""
    print_header(f"\n{'='*60}")
    print_header("  TEST SUMMARY")
    print_header(f"{'='*60}")

    for suite in results["suites"]:
        if suite["failed"] > 0:
            print_error(f"  [FAIL] {suite['name']}: {suite['passed']}/{suite['total']} passed")
        else:
            print_success(f"  [PASS] {suite['name']}: {suite['passed']}/{suite['total']} passed")

    print_info("")
    print_info(f"  Total:   {results['total']}")
    print_success(f"  Passed:  {results['passed']}")
    if results['failed'] > 0:
        print_error(f"  Failed:  {results['failed']}")
    else:
        print_info(f"  Failed:  {results['failed']}")
    if results['skipped'] > 0:
        print_info(f"  Skipped: {results['skipped']}")

def main():
    parser = argparse.ArgumentParser(description="ARIA Test Suite Runner")
    parser.add_argument(
        "category",
        nargs="?",
        default="all",
        choices=["unit", "integration", "validation", "all"],
        help="Test category to run"
    )
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    parser.add_argument("--output", "-o", type=str, help="Save results to JSON file")

    args = parser.parse_args()

    # Detect platform
    platform_info = detect_platform()

    # Print header
    print_header(f"\n{'='*60}")
    print_header("  ARIA TEST SUITE")
    print_header(f"{'='*60}")
    print_info(f"  Platform: {platform_info['os']}")
    print_info(f"  Python: {platform_info['python']}")
    print_info(f"  Bash: {'Yes' if platform_info['has_bash'] else 'No'}")
    print_info(f"  Git Bash: {'Yes' if platform_info['has_git_bash'] else 'No'}")
    print_info(f"  WSL: {'Yes' if platform_info['has_wsl'] else 'No'}")
    print_info(f"  Claude CLI: {'Yes' if platform_info['has_claude_cli'] else 'No'}")

    # Determine categories to run
    if args.category == "all":
        categories = ["unit", "integration", "validation"]
    else:
        categories = [args.category]

    # Run tests
    results = run_test_suite(categories, platform_info, args.verbose)

    # Print summary
    print_summary(results)

    # Save results
    output_file = args.output or str(ARIA_DIR / "state" / "test-results.json")
    os.makedirs(os.path.dirname(output_file), exist_ok=True)
    with open(output_file, "w") as f:
        json.dump(results, f, indent=2)
    print_info(f"\nResults saved to: {output_file}")

    # Exit with appropriate code
    sys.exit(0 if results["failed"] == 0 else 1)

if __name__ == "__main__":
    main()
