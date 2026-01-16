# ARIA Start Skill

> Session initialization with dashboard launch and workflow routing

## Purpose

This skill is invoked at the start of an ARIA session to:
1. Launch the observability dashboard
2. Present workflow selection HITL
3. Route to the appropriate flow

**Invoke with:** `/aria-start` or automatically on session start

---

## Startup Sequence

```
1. Launch dashboard (background)
2. Wait for dashboard ready
3. Present HITL router
4. Route to selected workflow
```

---

## Step 1: Launch Dashboard

Start the observability dashboard in the background:

**Linux/macOS:**
```bash
python .aria/scripts/serve-dashboard.py &
DASHBOARD_PID=$!
sleep 2
echo "Dashboard: http://localhost:8420 (PID: $DASHBOARD_PID)"
```

**Windows (via aria-start.bat):**
```batch
start "ARIA Dashboard" cmd /c "python .aria\scripts\serve-dashboard.py"
timeout /t 2 /nobreak >nul
start http://localhost:8420
```

**Verify dashboard is running:**
```bash
curl -s http://localhost:8420/api/signals > /dev/null && echo "Dashboard ready" || echo "Dashboard failed to start"
```

---

## Step 2: HITL Router

Present workflow selection:

```
+------------------------------------------+
|         ARIA Session Started             |
|   Dashboard: http://localhost:8420       |
+------------------------------------------+

Select workflow:

[b] BUILD    - Create new application from scratch
[m] MODIFY   - Change existing codebase
[r] RESEARCH - Analyze article/paper, create docs

What would you like to do? [b/m/r]
```

### Router Decision Tree

```
User input → Workflow
-----------------------
b, build     → Build flow
m, modify    → Modify flow
r, research  → Research flow
```

---

## Step 3: Route to Workflow

### [b] BUILD Flow

```
BUILD FLOW SELECTED

→ Router (size project)
→ Brainstorming (IDEA.md)
→ Prototyping (optional, SPEC-*.json)
→ Planning (current-plan.json)
→ Executing (agent loop)
→ Report + Dashboard
```

**Announce:**
```
BUILD MODE

Starting build workflow...
1. First, let's size the project
2. Then brainstorm approach
3. Prototype if needed
4. Plan implementation
5. Execute with verification

Describe what you want to build:
```

### [m] MODIFY Flow

```
MODIFY FLOW SELECTED

→ Discovery (understand codebase)
→ Router (size change)
→ Planning (current-plan.json)
→ Executing (agent loop)
→ Report + Dashboard
```

**Announce:**
```
MODIFY MODE

Starting modify workflow...
1. First, let's understand the codebase
2. Size the change
3. Plan modifications
4. Execute with verification

What changes do you want to make?
```

### [r] RESEARCH Flow

```
RESEARCH FLOW SELECTED

→ Researcher (extract concepts)
→ Brainstorming (IDEA.md)
→ Slides HITL (optional)
→ Prototype HITL (optional)
→ Report + Dashboard
```

**Announce:**
```
RESEARCH MODE

Starting research workflow...
1. Extract key concepts
2. Synthesize into IDEA.md
3. Optional: Generate slides
4. Optional: Create prototype
5. Generate report

Provide article/paper URL or paste content:
```

---

## Workflow Integration

After routing, the selected workflow proceeds with its standard steps:

| Workflow | Next Skill | Reference |
|----------|------------|-----------|
| BUILD | `brainstorming.md` → `prototyping.md` → `planning.md` | CLAUDE.md "Build" section |
| MODIFY | `discovery.md` → `planning.md` | CLAUDE.md "Modify" section |
| RESEARCH | `researcher.md` → `brainstorming.md` | CLAUDE.md "Research" section |

---

## Input Disambiguation

If user provides input before selecting workflow, attempt to infer:

| Input Pattern | Inferred Workflow |
|---------------|-------------------|
| URL to article/paper | RESEARCH |
| "build X", "create X" | BUILD |
| "fix", "refactor", "add feature to" | MODIFY |
| GitHub repo URL | Present HITL (could be any) |
| Ambiguous | Present HITL |

**Example disambiguation:**
```
INPUT: https://arxiv.org/abs/2301.xxxxx

This appears to be a research paper.

[r] Research this paper (recommended)
[b] Build something from this paper
[m] This is not what I meant

Select: [r/b/m]
```

---

## Dashboard Integration

The dashboard at http://localhost:8420 shows:

- **Signals**: Tool calls captured by hooks
- **Decisions**: Reasoning trace
- **Progress**: Task completion status
- **Session Info**: Mode, timing, metrics

**Auto-open browser:**
```bash
# Linux
xdg-open http://localhost:8420 2>/dev/null || true

# macOS
open http://localhost:8420 2>/dev/null || true

# Windows (in batch file)
start http://localhost:8420
```

---

## State Initialization

On session start, initialize state files:

```bash
mkdir -p .aria/state

# Initialize signals if not exists
[ -f .aria/state/signals.jsonl ] || touch .aria/state/signals.jsonl

# Initialize decisions if not exists
[ -f .aria/state/decisions.jsonl ] || touch .aria/state/decisions.jsonl

# Initialize progress
echo '{"session_start": "'$(date -Iseconds)'", "tasks": [], "current_task": null}' > .aria/state/progress.json
```

---

## Session Banner

Display at session start:

```
========================================
       ARIA Development Session
========================================

Dashboard: http://localhost:8420
Mode: [awaiting selection]
Session: [timestamp]

----------------------------------------
```

---

## HITL Checkpoints

| Checkpoint | Required |
|------------|----------|
| Workflow selection | Always |
| Input disambiguation | When ambiguous |

---

## Error Handling

### Dashboard fails to start
```
WARNING: Dashboard failed to start on port 8420
- Port may be in use: lsof -i :8420
- Try manually: python .aria/scripts/serve-dashboard.py

Continue without dashboard? [y/n]
```

### No input provided
```
No input provided. What would you like to do?

[b] BUILD - Create something new
[m] MODIFY - Change existing code
[r] RESEARCH - Analyze an article/paper

Select: [b/m/r]
```

---

## Integration with Claude Code

This skill can be invoked:

1. **Manually:** User types `/aria-start`
2. **Automatically:** Via SessionStart hook (if configured)
3. **Via batch file:** `aria-start.bat` (Windows)
4. **Via shell script:** `aria-start.sh` (Linux/macOS)

---

## Output

After this skill completes:

1. **Dashboard running** at http://localhost:8420
2. **Workflow selected** (BUILD/MODIFY/RESEARCH)
3. **State initialized** (.aria/state/ files)
4. **Ready for input** in selected workflow

The selected workflow skill takes over from here.
