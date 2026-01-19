# Offline Test Mode

Tests that run without network access or API credentials. These tests validate:
- File parsing and transformation logic
- Database schema and queries
- Skill file structure and content
- Configuration validation
- Template rendering

## Running Offline Tests

```bash
# Run all offline tests
python .aria/tests/run-tests.py --offline

# Or run specific offline test suites
bash .aria/tests/unit/test-slide-generation.sh
bash .aria/tests/unit/test-cc-usage.sh
bash .aria/tests/unit/test-deep-research.sh
```

## Test Categories

### Unit Tests (Always Offline)
- `test-test-generation.sh` - Stack detection and test generation
- `test-slide-generation.sh` - FOCUS.md parsing, PPTX generation
- `test-cc-usage.sh` - Claude log parsing, token metrics
- `test-dashboard.sh` - Database schema, API functions
- `test-deep-research.sh` - Skill structure validation

### Integration Tests (May Need Network)
- `test-invoke-agent.sh` - Agent invocation (needs Claude API)
- `test-skill-touch.sh` - Skill loading verification

## Writing Offline Tests

### Pattern 1: File Parsing
Test parsing logic with temp files containing mock data:

```bash
test_start "parse_focus_doc extracts core ideas"
result=$(python -c "
import sys, tempfile
from pathlib import Path

# Create mock input
focus_content = '''# FOCUS Document
## The Core
1. First idea
2. Second idea
'''

with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
    f.write(focus_content)
    temp_path = f.name

# Test the function
sys.path.insert(0, '\$ARIA_DIR/scripts')
from generate_slides import parse_focus_doc
result = parse_focus_doc(Path(temp_path))

import os
os.unlink(temp_path)

assert 'core_ideas' in result
print('OK')
" 2>&1)
```

### Pattern 2: Database Tests
Test database operations with temp SQLite:

```bash
test_start "init_db creates schema"
result=$(python -c "
import tempfile, sqlite3
from pathlib import Path

# Create temp database
db_path = Path(tempfile.mktemp(suffix='.db'))

# Run schema creation
# ... test code ...

# Verify and cleanup
db_path.unlink()
print('OK')
" 2>&1)
```

### Pattern 3: Skill Structure
Validate skill files have required sections:

```bash
test_start "skill has required sections"
skill_file="\$ARIA_DIR/skills/my-skill.md"
has_when=\$(grep -c "## When to Use" "\$skill_file" || echo "0")
has_workflow=\$(grep -c "## Workflow" "\$skill_file" || echo "0")

if [[ "\$has_when" -gt 0 ]] && [[ "\$has_workflow" -gt 0 ]]; then
    assert_true "1" "Has required sections"
else
    assert_true "" "Missing sections"
fi
```

### Pattern 4: JSONL Processing
Test JSONL parsing with mock entries:

```bash
result=$(python -c "
import json, tempfile

# Create mock JSONL
entries = [
    {'type': 'assistant', 'timestamp': '2025-01-15T10:00:00Z'},
    {'type': 'tool_call', 'name': 'Read'}
]

with tempfile.NamedTemporaryFile(mode='w', suffix='.jsonl', delete=False) as f:
    for e in entries:
        f.write(json.dumps(e) + '\n')
    temp_path = f.name

# Test parsing
# ... test code ...

import os
os.unlink(temp_path)
print('OK')
" 2>&1)
```

## Mock Data Fixtures

Common test fixtures are in `.aria/tests/fixtures/`:

- `sample-focus.md` - Mock FOCUS document
- `sample-idea.md` - Mock IDEA document
- `sample-claude-log.jsonl` - Mock Claude log entries
- `sample-decisions.jsonl` - Mock decision trace
- `sample-signals.jsonl` - Mock tool call signals

## Marking Tests Offline vs Online

Tests should handle missing dependencies gracefully:

```bash
# Skip if dependency missing
if ! python -c "import notebooklm" 2>/dev/null; then
    echo "SKIP: notebooklm not installed"
    exit 0
fi

# Or use conditional logic
result=$(python -c "
try:
    import notebooklm
    # ... online test ...
except ImportError:
    print('SKIP: offline mode')
" 2>&1)
```

## CI Integration

The GitHub Actions workflow runs offline tests by default:

```yaml
- name: Run offline tests
  run: python .aria/tests/run-tests.py --offline

- name: Run integration tests
  if: env.ANTHROPIC_API_KEY != ''
  run: python .aria/tests/run-tests.py --integration
```
