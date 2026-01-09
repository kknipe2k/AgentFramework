# ARIA Rails - Actual Implementation

Not theory. Actual scripts that **block me** from bad behavior.

## What It Does

```
┌─────────────────────────────────────────────────────────────┐
│  RAIL 1: No edits without intent                           │
│          Can't touch code until .aria/intent.md exists     │
├─────────────────────────────────────────────────────────────┤
│  RAIL 2: Max 3 edits without testing                       │
│          BLOCKED after 3 file changes until tests run      │
├─────────────────────────────────────────────────────────────┤
│  RAIL 3: Max 5 edits without commit                        │
│          BLOCKED after 5 file changes until committed      │
├─────────────────────────────────────────────────────────────┤
│  RAIL 4: Tests must pass before commit                     │
│          Can't commit if last test run failed              │
└─────────────────────────────────────────────────────────────┘
```

## Installation

Already in `.claude/` directory:

```
.claude/
├── settings.json          # Wires hooks to Claude Code
└── hooks/
    ├── aria-rails.sh      # The actual rails (PreToolUse/PostToolUse)
    └── aria                # CLI to manage state
```

## Usage

### Start a Task

```bash
# Initialize with intent
./aria init "Add user authentication with JWT"

# Edit the intent file to add requirements
cat .aria/intent.md
```

`.aria/intent.md`:
```markdown
# Intent: Add user authentication with JWT

## Must Have:
- User registration endpoint
- Login endpoint returning JWT
- Middleware to protect routes
- Password hashing

## Must Not:
- Plain text passwords
- Tokens in logs or URLs
```

### During Work

```bash
# Check status anytime
./.claude/hooks/aria status

═══════════════════════════════════
         ARIA STATUS
═══════════════════════════════════
Intent: Add user authentication with JWT

Edits total:        7
Edits since test:   2/3
Edits since commit: 2/5
Tests passing:      YES
═══════════════════════════════════
```

### What Happens When Rails Trigger

**After 3 edits without testing:**
```
BLOCKED: 3 edits without testing. Run tests before continuing.
```
I literally cannot edit another file until I run tests.

**After 5 edits without commit:**
```
BLOCKED: 5 edits without commit. Commit checkpoint before continuing.
```
I literally cannot edit another file until I commit.

**Trying to commit with failing tests:**
```
BLOCKED: Cannot commit with failing tests. Fix tests first.
```

### Manual Overrides

```bash
# If you ran tests manually outside Claude
./.claude/hooks/aria pass

# If you committed manually
./.claude/hooks/aria reset commit

# Reset everything
./.claude/hooks/aria reset all
```

### Complete a Task

```bash
./.claude/hooks/aria done

═══════════════════════════════════
         INTENT VERIFICATION
═══════════════════════════════════

# Intent: Add user authentication with JWT

## Must Have:
- User registration endpoint
- Login endpoint returning JWT
- Middleware to protect routes
- Password hashing

## Must Not:
- Plain text passwords
- Tokens in logs or URLs

═══════════════════════════════════

Does the implementation satisfy this intent? [y/N]
```

## How the Rails Work

### Hook Flow

```
Claude tries to Edit a file
         │
         ▼
    PreToolUse Hook
         │
         ├─► Check: Intent exists?
         │   NO → BLOCKED
         │
         ├─► Check: < 3 edits since test?
         │   NO → BLOCKED
         │
         ├─► Check: < 5 edits since commit?
         │   NO → BLOCKED
         │
         ▼
    Edit proceeds
         │
         ▼
    PostToolUse Hook
         │
         └─► Increment edit counter
```

### State Files

```
.aria/
├── intent.md       # The sacred contract
├── edit_count      # Total edits made
├── last_test       # Edit count when tests last ran
├── last_commit     # Edit count when last committed
└── tests_failed    # Marker if tests are failing
```

## Why This Works

| Text instruction | Script rail |
|------------------|-------------|
| "Remember to test" | **Cannot proceed** without testing |
| "Commit often" | **Cannot proceed** without committing |
| "Define intent first" | **Cannot start** without intent |
| I can ignore it | I literally cannot bypass it |

## Tuning the Rails

Edit `aria-rails.sh` to adjust:

```bash
# Change from 3 to 5 edits before forced test
if [[ $edits_since_test -ge 3 ]]; then  # ← change this number

# Change from 5 to 10 edits before forced commit
if [[ $edits_since_commit -ge 5 ]]; then  # ← change this number
```

## Limitations

1. **Only counts Claude's edits** - Manual edits outside Claude not tracked
2. **Test detection is pattern-based** - Looks for `npm test`, `pytest`, etc.
3. **No automatic rollback** - Just blocks, doesn't revert
4. **Single project** - State is per-directory

## Future Improvements

- [ ] Detect test frameworks automatically
- [ ] Track specific files changed for smarter rollback
- [ ] Integration with git hooks for commit verification
- [ ] Web dashboard for status
- [ ] Slack/Discord notifications on blocks

---

**This is the minimum viable rails. Not pretty, but it WORKS.**
