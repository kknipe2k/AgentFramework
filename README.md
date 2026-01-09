# ARIA: Agentic Rail-based Intent Architecture

> *"The LLM writes the recipe. The rails ensure it's followed."*

## TL;DR - Working Implementation

**Scripts that BLOCK Claude from bad behavior:**

```bash
# Start a task
./.claude/hooks/aria init "Add user authentication"

# Claude works... but after 3 edits:
# BLOCKED: 3 edits without testing. Run tests before continuing.

# After 5 edits:
# BLOCKED: 5 edits without commit. Commit checkpoint before continuing.

# Trying to commit with failing tests:
# BLOCKED: Cannot commit with failing tests. Fix tests first.

# When done:
./.claude/hooks/aria done  # Forces intent verification
```

**See [ARIA_RAILS.md](./ARIA_RAILS.md) for the actual working implementation.**

---

## What is ARIA?

ARIA is **not a programming language** - it's rails that make LLM-driven development **deterministic and verifiable**.

```
INTENT LOCKED → RAILS BLOCK BAD BEHAVIOR → GATES VERIFY → DONE
```

## The Problem

| Problem | What Happens |
|---------|--------------|
| **Intent drift** | LLM forgets original goal halfway through |
| **No verification** | Hope it worked, find bugs later |
| **No rollback** | Stuck with broken state |
| **Missed requirements** | "Oh, I forgot to add tests" |

## The ARIA Solution

### Actual Rails (Working Now)

```
┌─────────────────────────────────────────────────────────────┐
│  RAIL 1: No edits without intent                           │
│          Can't touch code until intent defined             │
├─────────────────────────────────────────────────────────────┤
│  RAIL 2: Max 3 edits without testing                       │
│          BLOCKED until tests run                           │
├─────────────────────────────────────────────────────────────┤
│  RAIL 3: Max 5 edits without commit                        │
│          BLOCKED until committed (checkpoint)              │
├─────────────────────────────────────────────────────────────┤
│  RAIL 4: Tests must pass before commit                     │
│          Can't commit broken code                          │
└─────────────────────────────────────────────────────────────┘
```

### Future: Structured Plans

```aria
@plan "Add user authentication"

@intent
  + User can register with email/password
  + User can login and get JWT token
  - No plain text passwords
  - No tokens in URLs

---

@phase model
  @ src/models/User.js
    ```js
    const bcrypt = require('bcrypt');
    class User {
      static async create(email, pass) {
        return { email, hash: await bcrypt.hash(pass, 10) };
      }
    }
    ```
  ? no_plaintext: !contains(User.js, "password =")
  * checkpoint

@phase verify
  ? tests: `npm test` == 0
  ? intent: llm "Does this satisfy the original intent?"

@done
  > git commit -m "feat: add JWT auth"
```

**Every step verified. Every requirement checked. Every failure recoverable.**

## Core Concepts

### 1. INTENT - The Sacred Contract
```aria
@intent
  + must have this feature
  - must NOT have this anti-pattern
```
Checked at every gate. Drift = stop and review.

### 2. RAILS - Constrained Actions
```aria
> npm install express     # Run command
@ src/file.js             # Create/edit file
? name: condition         # Verification gate
* checkpoint              # Save state for rollback
```
Atomic, verifiable, reversible.

### 3. GATES - Verification Checkpoints
```aria
? tests: `npm test` == 0                    # Command check
? secure: !contains(file.js, "password")    # Content scan
? intent: llm "Is the goal met?"            # LLM verification
```
Block progression until conditions met.

### 4. CHECKPOINTS - Rollback Points
```aria
* checkpoint
# If next gate fails, rollback here
```
Never stuck with broken state.

## Documentation

| Document | Description |
|----------|-------------|
| **[ARIA_RAILS.md](./ARIA_RAILS.md)** | **Working implementation - start here** |
| [ARIA_ORCHESTRATION.md](./ARIA_ORCHESTRATION.md) | Full architecture and concepts |
| [ARIA_SYNTAX.md](./ARIA_SYNTAX.md) | Concise syntax reference (future) |
| [ARIA_CLAUDE_INTEGRATION.md](./ARIA_CLAUDE_INTEGRATION.md) | Claude Code hooks/skills integration |

## Syntax Quick Reference

```aria
@plan "name"           # Plan title
@intent                # Requirements block
  + must have          # Required feature
  - must not           # Anti-requirement
@require pkg1, pkg2    # Dependencies
@env VAR1              # Required env vars

@phase name            # Execution phase
  > command            # Run shell command
  @ path/file.js       # Create/edit file
  ? gate: condition    # Verification
  * checkpoint         # Save state

@done                  # On success
@fail                  # On failure
```

## How It Works

1. **User states intent** → "Add authentication"
2. **LLM generates ARIA plan** → Structured recipe with gates
3. **User approves plan** → Or requests changes
4. **Executor runs plan** → Phase by phase
5. **Gates verify each step** → Block on failure
6. **Intent verified at end** → Did we achieve the goal?
7. **Auto-generate artifacts** → Docs, commits, changelog

## Why ARIA?

| Aspect | Traditional LLM | ARIA |
|--------|-----------------|------|
| Intent tracking | Hopes to remember | Locked and checked |
| Verification | End-to-end prayer | Every step |
| Failure recovery | Start over | Rollback to checkpoint |
| Audit trail | Chat history | Structured logs |
| Test coverage | If you're lucky | Required gate |

## Integration

ARIA integrates with Claude Code via:
- **Skills** - Auto-trigger on complex tasks
- **Hooks** - PreToolUse/PostToolUse verification
- **Agents** - Specialized planner/verifier subagents

See [ARIA_CLAUDE_INTEGRATION.md](./ARIA_CLAUDE_INTEGRATION.md) for details.

---

## Archive: Hybrid Language Design

The original exploration of a hybrid programming language combining Python, JavaScript, and HTML features is archived in:
- [ARIA_LANGUAGE_SPEC.md](./ARIA_LANGUAGE_SPEC.md) - Language specification
- [IMPLEMENTATION_ROADMAP.md](./IMPLEMENTATION_ROADMAP.md) - Implementation guide
- [examples/](./examples/) - Syntax examples

---

*ARIA: Making agentic coding deterministic, verifiable, and recoverable.*
