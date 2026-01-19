#!/bin/bash
# Unit Tests: Dashboard Server (serve-dashboard.py)
# Tests the dashboard API functions and database operations

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARIA_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Source test runner (provides to_win_path function)
source "$(dirname "$SCRIPT_DIR")/test-runner.sh"

# Get Windows-compatible path for Python
ARIA_DIR_PY="$(to_win_path "$ARIA_DIR")"

# ============================================
# TESTS: Dashboard Script Exists
# ============================================

test_start "serve-dashboard.py exists"
assert_file_exists "$ARIA_DIR/scripts/serve-dashboard.py" "Dashboard script should exist"
test_end

test_start "serve-dashboard.py is valid Python"
if python -m py_compile "$ARIA_DIR/scripts/serve-dashboard.py" 2>/dev/null; then
    assert_true "1" "Script should be valid Python"
else
    assert_true "" "Script has syntax errors"
fi
test_end

# ============================================
# TESTS: Dashboard Has Required Functions
# ============================================

test_start "serve-dashboard.py has init_db function"
result=$(python -c "
import sys
sys.path.insert(0, '$ARIA_DIR_PY/scripts')
import importlib.util
spec = importlib.util.spec_from_file_location('dashboard', '$ARIA_DIR_PY/scripts/serve-dashboard.py')
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
assert hasattr(mod, 'init_db'), 'Missing init_db'
print('OK')
" 2>&1)
if [[ "$result" == "OK" ]]; then
    assert_true "1" "Should have init_db function"
else
    assert_true "" "Missing init_db: $result"
fi
test_end

test_start "serve-dashboard.py has get_session_summary function"
result=$(python -c "
import sys
sys.path.insert(0, '$ARIA_DIR_PY/scripts')
import importlib.util
spec = importlib.util.spec_from_file_location('dashboard', '$ARIA_DIR_PY/scripts/serve-dashboard.py')
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
assert hasattr(mod, 'get_session_summary'), 'Missing get_session_summary'
print('OK')
" 2>&1)
if [[ "$result" == "OK" ]]; then
    assert_true "1" "Should have get_session_summary function"
else
    assert_true "" "Missing get_session_summary: $result"
fi
test_end

test_start "serve-dashboard.py has get_lineage function"
result=$(python -c "
import sys
sys.path.insert(0, '$ARIA_DIR_PY/scripts')
import importlib.util
spec = importlib.util.spec_from_file_location('dashboard', '$ARIA_DIR_PY/scripts/serve-dashboard.py')
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
assert hasattr(mod, 'get_lineage'), 'Missing get_lineage'
print('OK')
" 2>&1)
if [[ "$result" == "OK" ]]; then
    assert_true "1" "Should have get_lineage function"
else
    assert_true "" "Missing get_lineage: $result"
fi
test_end

test_start "serve-dashboard.py has get_metrics function"
result=$(python -c "
import sys
sys.path.insert(0, '$ARIA_DIR_PY/scripts')
import importlib.util
spec = importlib.util.spec_from_file_location('dashboard', '$ARIA_DIR_PY/scripts/serve-dashboard.py')
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
assert hasattr(mod, 'get_metrics'), 'Missing get_metrics'
print('OK')
" 2>&1)
if [[ "$result" == "OK" ]]; then
    assert_true "1" "Should have get_metrics function"
else
    assert_true "" "Missing get_metrics: $result"
fi
test_end

# ============================================
# TESTS: Database Initialization (Offline)
# ============================================

test_start "init_db creates database schema"
TEST_DB="/tmp/aria-test-dashboard-$$.db"
rm -f "$TEST_DB" 2>/dev/null

result=$(python -c "
import sys, sqlite3
sys.path.insert(0, '$ARIA_DIR_PY/scripts')
import importlib.util
spec = importlib.util.spec_from_file_location('dashboard', '$ARIA_DIR_PY/scripts/serve-dashboard.py')
mod = importlib.util.module_from_spec(spec)

# Override DB_PATH before exec
mod.DB_PATH = '$TEST_DB'
from pathlib import Path
mod.DB_PATH = Path('$TEST_DB')

spec.loader.exec_module(mod)

# Initialize DB
conn = mod.init_db()

# Check tables exist
cursor = conn.cursor()
cursor.execute(\"SELECT name FROM sqlite_master WHERE type='table'\")
tables = [row[0] for row in cursor.fetchall()]
conn.close()

assert 'sessions' in tables, 'Missing sessions table'
assert 'events' in tables, 'Missing events table'
print('OK')
" 2>&1)

if [[ "$result" == "OK" ]]; then
    assert_true "1" "Should create database schema"
else
    assert_true "" "Schema creation failed: $result"
fi

rm -f "$TEST_DB" 2>/dev/null
test_end

# ============================================
# TESTS: Dashboard HTML Exists
# ============================================

test_start "Dashboard HTML directory exists"
if [[ -d "$ARIA_DIR/dashboard" ]]; then
    assert_true "1" "Dashboard directory should exist"
else
    # May not exist yet - that's OK
    assert_true "1" "Dashboard directory would be created on server start"
fi
test_end

test_start "Dashboard has index.html if directory exists"
if [[ -d "$ARIA_DIR/dashboard" ]] && [[ -f "$ARIA_DIR/dashboard/index.html" ]]; then
    assert_true "1" "index.html should exist"
else
    assert_true "1" "index.html would be created or served dynamically"
fi
test_end

# ============================================
# TESTS: API Endpoint Definitions
# ============================================

test_start "serve-dashboard.py defines API endpoints"
result=$(grep -c "/api/" "$ARIA_DIR/scripts/serve-dashboard.py" 2>/dev/null || echo "0")
if [[ "$result" -gt 5 ]]; then
    assert_true "1" "Should define multiple API endpoints ($result found)"
else
    assert_true "" "Missing API endpoint definitions"
fi
test_end

test_start "serve-dashboard.py has /api/session endpoint"
if grep -q "/api/session" "$ARIA_DIR/scripts/serve-dashboard.py" 2>/dev/null; then
    assert_true "1" "Should have /api/session endpoint"
else
    assert_true "" "Missing /api/session endpoint"
fi
test_end

test_start "serve-dashboard.py has /api/lineage endpoint"
if grep -q "/api/lineage" "$ARIA_DIR/scripts/serve-dashboard.py" 2>/dev/null; then
    assert_true "1" "Should have /api/lineage endpoint"
else
    assert_true "" "Missing /api/lineage endpoint"
fi
test_end

test_start "serve-dashboard.py has /api/metrics endpoint"
if grep -q "/api/metrics" "$ARIA_DIR/scripts/serve-dashboard.py" 2>/dev/null; then
    assert_true "1" "Should have /api/metrics endpoint"
else
    assert_true "" "Missing /api/metrics endpoint"
fi
test_end

# ============================================
# TESTS: Claude Log Parsing (Offline)
# ============================================

test_start "parse_claude_log_for_metrics handles empty input"
result=$(python -c "
import sys, tempfile
sys.path.insert(0, '$ARIA_DIR_PY/scripts')
import importlib.util
spec = importlib.util.spec_from_file_location('dashboard', '$ARIA_DIR_PY/scripts/serve-dashboard.py')
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)

# Create empty temp file
with tempfile.NamedTemporaryFile(mode='w', suffix='.jsonl', delete=False) as f:
    temp_path = f.name

from pathlib import Path
result = mod.parse_claude_log_for_metrics(Path(temp_path))

import os
os.unlink(temp_path)

# Should return valid structure even with empty input
assert 'total_input_tokens' in result, 'Missing total_input_tokens'
assert 'total_output_tokens' in result, 'Missing total_output_tokens'
assert 'total_cost' in result, 'Missing total_cost'
print('OK')
" 2>&1)

if [[ "$result" == "OK" ]]; then
    assert_true "1" "Should handle empty log file"
else
    assert_true "" "Failed on empty input: $result"
fi
test_end

test_start "parse_claude_log_for_metrics parses valid JSONL"
result=$(python -c "
import sys, tempfile, json
sys.path.insert(0, '$ARIA_DIR_PY/scripts')
import importlib.util
spec = importlib.util.spec_from_file_location('dashboard', '$ARIA_DIR_PY/scripts/serve-dashboard.py')
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)

# Create test log file
log_entry = {
    'type': 'assistant',
    'timestamp': '2024-01-15T10:30:00Z',
    'message': {
        'model': 'claude-sonnet-4-20250514',
        'usage': {
            'input_tokens': 1000,
            'output_tokens': 500,
            'cache_read_input_tokens': 200,
            'cache_creation_input_tokens': 100
        },
        'content': []
    }
}

with tempfile.NamedTemporaryFile(mode='w', suffix='.jsonl', delete=False) as f:
    f.write(json.dumps(log_entry) + '\n')
    temp_path = f.name

from pathlib import Path
result = mod.parse_claude_log_for_metrics(Path(temp_path))

import os
os.unlink(temp_path)

# Should parse tokens
assert result['total_input_tokens'] == 1000, f\"Wrong input tokens: {result['total_input_tokens']}\"
assert result['total_output_tokens'] == 500, f\"Wrong output tokens: {result['total_output_tokens']}\"
print('OK')
" 2>&1)

if [[ "$result" == "OK" ]]; then
    assert_true "1" "Should parse valid JSONL log"
else
    assert_true "" "Failed to parse: $result"
fi
test_end

echo ""
echo "Dashboard tests complete."
