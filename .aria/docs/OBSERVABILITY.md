# ARIA Observability Architecture

> Decision tracing + Signal capture + Reconciliation

This document describes how ARIA captures, stores, and verifies agent decisions for traceability and precedent lookup.

---

## Core Principle

**Traces are artifacts, not working memory.**

```
DURING EXECUTION              AFTER DECISION
─────────────────             ──────────────
Agent reasons                 Emit decision → storage
Makes decision                Log signal → storage
Acts                          Clear from context
                              Move on
```

Context stays clean. History accumulates externally. Agent queries it like any other tool.

---

## Architecture

```
USER REQUEST
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│                         AGENT                               │
│                                                             │
│   System prompt (CLAUDE.md) includes:                       │
│   "Emit <decision> block for consequential actions"         │
│                                                             │
│   Response includes:                                        │
│   - Normal output                                           │
│   - <decision> blocks for key choices                       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
     │                                          │
     │ tool calls                               │ decision blocks
     ▼                                          ▼
┌──────────────┐                         ┌──────────────┐
│    HOOKS     │                         │    PARSER    │
│              │                         │              │
│ PreToolUse   │                         │ Extracts     │
│ PostToolUse  │                         │ <decision>   │
│              │                         │ blocks       │
└──────────────┘                         └──────────────┘
     │                                          │
     │ signals                                  │ decisions
     ▼                                          ▼
┌──────────────┐                         ┌──────────────┐
│  signals.    │                         │  decisions.  │
│  jsonl       │                         │  jsonl       │
└──────────────┘                         └──────────────┘
     │                                          │
     └──────────────────┬───────────────────────┘
                        ▼
              ┌──────────────────┐
              │   RECONCILER     │
              │                  │
              │  reconcile.sh    │
              │                  │
              │  claim: "read    │
              │   utils/retry"   │
              │       ↕          │
              │  signal: Read    │
              │   utils/retry    │
              │       =          │
              │    VERIFIED      │
              └──────────────────┘
```

---

## Components

### 1. Decision Schema (CLAUDE.md)

Agent emits decision blocks for consequential choices:

```xml
<decision>
  <action>what you're doing</action>
  <context>what you looked at to decide</context>
  <rationale>why this approach</rationale>
  <alternatives>what else you considered</alternatives>
  <confidence>0.0-1.0</confidence>
</decision>
```

**When to emit:**
- Modifying files (architectural choices)
- Choosing between alternatives
- Deviating from existing patterns
- Skipping something (and why)

**Skip for:** Trivial reads, routine navigation, obvious single-path actions.

### 2. Signal Capture (Hooks)

The `aria-rails.sh` hook logs every tool call:

```bash
# PreToolUse and PostToolUse both call:
log_signal "pre|post" "$TOOL_NAME" "$TOOL_INPUT"
```

**Output format (signals.jsonl):**
```json
{"id":"sig-1234","timestamp":"2024-01-14T15:30:00Z","event":"pre","tool":"Read","file_path":"utils/retry.ts","command":""}
{"id":"sig-1235","timestamp":"2024-01-14T15:30:01Z","event":"post","tool":"Read","file_path":"utils/retry.ts","command":""}
{"id":"sig-1236","timestamp":"2024-01-14T15:30:05Z","event":"pre","tool":"Edit","file_path":"src/api/client.ts","command":""}
```

### 3. Decision Storage (decisions.jsonl)

Decisions are captured and stored:

```json
{"timestamp":"2024-01-14T15:30:00Z","action":"Add retry wrapper","context":"Read utils/retry.ts, saw 3 similar uses","rationale":"Consistency","alternatives":"custom, none","confidence":"0.85","verified":null}
```

### 4. Reconciliation (reconcile.sh)

Matches decision claims against actual signals:

```
Decision: "Read utils/retry.ts to follow existing patterns"
Signal:   Read(utils/retry.ts) @ 15:30:00
Match:    ✓ VERIFIED
```

---

## Storage Files

| File | Purpose | Format |
|------|---------|--------|
| `.aria/state/signals.jsonl` | Tool call log | JSONL, append-only |
| `.aria/state/decisions.jsonl` | Decision trace | JSONL, append-only |

**Why JSONL:**
- Append-only (no corruption on crash)
- Easy to grep/filter
- No schema migrations
- Works with standard Unix tools

---

## Web Dashboard

Interactive dashboard for exploring session traces.

### Start the Dashboard

```bash
python .aria/scripts/serve-dashboard.py
# Opens at http://localhost:8420
```

### Dashboard Views

**Summary Cards:**
- Signals (tool calls captured)
- Decisions (with verification rate)
- Commits (git commits in session)
- Average confidence
- Files touched
- Tools used

**Timeline Tab:**
- Chronological view of all events
- Color-coded by type (signal, decision, commit, HITL)
- Shows tool calls with file paths/commands

**Decisions Tab:**
- All decisions with confidence scores
- Verification status (verified/unverified/pending)
- Click to expand and see supporting signals
- Context, rationale, alternatives

**Commits Tab:**
- Git commits in session
- Linked decisions that led to each commit
- Tool usage breakdown per commit

### API Endpoints

```
GET /api/session    - Session summary (counts, tools, files)
GET /api/timeline   - Unified event timeline
GET /api/decisions  - Decisions with supporting signals
GET /api/commits    - Commits with linked decisions
```

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Web Dashboard (index.html)                  │
│                         JS/CSS SPA                              │
└─────────────────────────────────────────────────────────────────┘
                              │ fetch /api/*
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                 Python Server (serve-dashboard.py)              │
│                                                                 │
│   - Syncs JSONL → sqlite on each request                       │
│   - Parses git log for commits                                  │
│   - Links decisions to signals by timestamp                     │
│   - Links commits to decisions by time window                   │
└─────────────────────────────────────────────────────────────────┘
                              │ reads
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│   signals.jsonl │ decisions.jsonl │ traces.db │ git log        │
└─────────────────────────────────────────────────────────────────┘
```

**Storage:**
- JSONL files = source of truth (append-only, crash-safe)
- sqlite = query cache (rebuilt on server start)
- No external dependencies (Python stdlib only)

---

## CLI Tools

### trace-view.sh

See recent activity:

```bash
.aria/scripts/trace-view.sh           # Last 20 entries
.aria/scripts/trace-view.sh --last 50 # More entries
.aria/scripts/trace-view.sh --today   # Today only
```

**Output:**
```
═══════════════════════════════════════════════════════════════
                    ARIA Decision Trace
═══════════════════════════════════════════════════════════════

┌─ SIGNALS (Tool Calls) ────────────────────────────────────────┐
│  15:30:00 → Read         utils/retry.ts
│  15:30:01 ✓ Read         utils/retry.ts
│  15:30:05 → Edit         src/api/client.ts
│  15:30:06 ✓ Edit         src/api/client.ts
└────────────────────────────────────────────────────────────────┘

┌─ DECISIONS ───────────────────────────────────────────────────┐
│  15:30:00 ✓ Add retry wrapper to API client (0.85)
└────────────────────────────────────────────────────────────────┘

Summary: 4 signals | 1 decisions | 1 verified
```

### query-decisions.sh

Search past decisions for precedent:

```bash
.aria/scripts/query-decisions.sh auth        # Find auth decisions
.aria/scripts/query-decisions.sh retry -c    # Show with context
.aria/scripts/query-decisions.sh "error" -n 5
```

**Output:**
```
Searching for: auth
═══════════════════════════════════════════════════════════════

Found 2 matching decision(s):

┌─────────────────────────────────────────────────────────────┐
│ 2024-01-12 14:22:00                              VERIFIED
│
│ Action: Implemented JWT validation in middleware
│ Confidence: 0.9
└─────────────────────────────────────────────────────────────┘
```

### reconcile.sh

Verify claims match signals:

```bash
.aria/scripts/reconcile.sh           # Basic reconciliation
.aria/scripts/reconcile.sh -v        # Verbose output
```

**Output:**
```
═══════════════════════════════════════════════════════════════
                 ARIA Decision Reconciliation
═══════════════════════════════════════════════════════════════

✓ VERIFIED: Add retry wrapper to API client
~ PARTIAL: Update error handling
✗ UNVERIFIED: Refactor auth flow

═══════════════════════════════════════════════════════════════
Summary
═══════════════════════════════════════════════════════════════

  Total decisions:  3
  Verified:         1
  Partial:          1
  Unverified:       1

  Verification rate: 33%
```

---

## Mode Variations

| Mode | Decision Tracing | Signals |
|------|-----------------|---------|
| LITE | Skip | Always captured |
| STANDARD | Key decisions | Always captured |
| FULL/FULL+ | All consequential | Always captured |

Signals are always captured (hooks run regardless). Decision emission is mode-dependent.

---

## VS Code vs Terminal

| Environment | Signals | Decisions | Reconciliation |
|-------------|---------|-----------|----------------|
| VS Code (Claude Code) | ✓ via hooks | ✓ | ✓ |
| Terminal (ralph.sh) | ✓ via hooks or wrapper | ✓ | ✓ |

Both environments get full traceability.

---

## Querying Precedent

When agent needs to know "how did I handle this before?":

1. Use query-decisions.sh to search
2. Inject relevant precedent into prompt
3. Make informed decision

**Example workflow:**
```
User: "Add rate limiting to the API"

Agent thinks: "I should check how auth was implemented"
Agent queries: .aria/scripts/query-decisions.sh "auth"
Agent sees: Previous decision used middleware pattern
Agent decides: Follow same pattern for consistency
```

---

## Token Cost

**Decision block:** ~200-400 tokens per decision
**Typical session:** 5-10 decision points = 2-3K tokens
**Session total:** 50-200K tokens

**Overhead:** 1-5%

The trace is emitted once per decision, not per tool call. Minimal impact.

---

## Future Enhancements

1. **--fix flag:** Update decisions.jsonl with verification status
2. **Time-window matching:** Match decisions to signals within N seconds
3. **Cross-session queries:** Search across all sessions
4. **Export to sqlite:** For complex queries
5. **Web UI:** Visual trace exploration

---

*ARIA Observability - Decisions you can audit*
