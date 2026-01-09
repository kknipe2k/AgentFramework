# Boris Cherny Patterns for AI-Assisted Development

## Overview

Boris Cherny's patterns represent a pragmatic approach to maximizing the effectiveness of AI coding assistants. These patterns emerged from real-world experience building production software with Claude and similar tools, focusing on **verification**, **context management**, and **structured decomposition**.

The core insight: AI assistants are powerful but need guardrails, context, and verification to produce reliable, production-quality code.

---

## Pattern 1: CLAUDE.md - Codebase Context File

### The Problem

AI assistants start each session with zero knowledge of your specific codebase. They don't know your:
- Architecture decisions
- Naming conventions
- File organization
- Testing patterns
- Domain-specific rules

This leads to code that works but doesn't fit your project.

### The Solution

Create a `CLAUDE.md` file at your repository root that provides essential context:

```markdown
# Project: MyApp

## Architecture
- Frontend: React + TypeScript in /src/components
- Backend: Node.js + Express in /server
- Database: PostgreSQL with Prisma ORM

## Conventions
- Use functional components with hooks
- Name files: PascalCase for components, camelCase for utilities
- Tests go in __tests__ folders adjacent to source
- Use absolute imports from @/

## Commands
- npm test - Run all tests
- npm run typecheck - Check types
- npm run lint - Run ESLint
- npm run build - Production build

## Important Patterns
- All API calls go through /src/api/client.ts
- Use the Result<T, E> type for error handling
- Never throw exceptions in business logic
```

### Why It Works

1. **Immediate Context**: AI starts with understanding, not guessing
2. **Consistency**: Generated code matches existing patterns
3. **Reduced Back-and-Forth**: Fewer corrections needed
4. **Team Alignment**: Documents conventions for humans too

### Best Practices

- Keep it under 500 lines (context window efficiency)
- Update when major decisions change
- Include "anti-patterns" - what NOT to do
- Add examples of good code from your codebase

---

## Pattern 2: Comprehensive Verification

### The Problem

AI-generated code often:
- Has subtle type errors
- Breaks existing tests
- Introduces linting violations
- Works in isolation but fails integration

Catching these issues late is expensive.

### The Solution

Implement a multi-layer verification pipeline that runs after every significant change:

```
┌─────────────────────────────────────────────────────────┐
│                  VERIFICATION PIPELINE                   │
├─────────────────────────────────────────────────────────┤
│  Layer 1: Static Analysis                               │
│  ├── TypeScript type checking (tsc --noEmit)           │
│  ├── ESLint (code style + potential bugs)              │
│  └── Prettier (formatting)                              │
├─────────────────────────────────────────────────────────┤
│  Layer 2: Unit Tests                                    │
│  ├── Jest/Vitest for logic                             │
│  ├── React Testing Library for components              │
│  └── Coverage thresholds                                │
├─────────────────────────────────────────────────────────┤
│  Layer 3: Integration Tests                             │
│  ├── API endpoint tests                                 │
│  ├── Database integration                               │
│  └── Service interactions                               │
├─────────────────────────────────────────────────────────┤
│  Layer 4: End-to-End Tests                              │
│  ├── Playwright/Cypress for UI flows                   │
│  ├── Critical user journeys                            │
│  └── Cross-browser verification                         │
├─────────────────────────────────────────────────────────┤
│  Layer 5: Build Verification                            │
│  ├── Production build succeeds                         │
│  ├── Bundle size within limits                         │
│  └── No console errors in build                        │
└─────────────────────────────────────────────────────────┘
```

### Verification Levels

**Quick** (< 30 seconds): Types + Lint + Fast unit tests
- Run after every file change
- Catches obvious errors immediately

**Standard** (1-5 minutes): Quick + All unit tests + Build
- Run before committing
- Ensures nothing is broken

**Full** (5-15 minutes): Standard + Integration + E2E
- Run before merging PRs
- Complete confidence

### Implementation

```bash
#!/bin/bash
# verify.sh - Boris Cherny style verification

LEVEL=${1:-standard}

echo "Running $LEVEL verification..."

# Layer 1: Static Analysis (always)
npm run typecheck || exit 1
npm run lint || exit 1

if [[ "$LEVEL" == "quick" ]]; then
    exit 0
fi

# Layer 2: Unit Tests
npm test || exit 1

# Layer 3: Build
npm run build || exit 1

if [[ "$LEVEL" == "standard" ]]; then
    exit 0
fi

# Layer 4: Integration (full only)
npm run test:integration || exit 1

# Layer 5: E2E (full only)
npm run test:e2e || exit 1

echo "All verifications passed!"
```

### Why It Works

1. **Fast Feedback**: Catch errors in seconds, not hours
2. **Confidence**: Know the code actually works
3. **AI Accountability**: AI must produce code that passes
4. **Incremental**: Quick checks during development, thorough before merge

---

## Pattern 3: Subagent Architecture

### The Problem

Complex tasks overwhelm AI assistants:
- Context limits are exceeded
- Focus is lost across many concerns
- Quality degrades on long tasks
- Errors compound

### The Solution

Decompose complex work into specialized subagents, each with:
- A single responsibility
- Limited context (only what it needs)
- Clear inputs and outputs
- Verification of its specific work

```
┌─────────────────────────────────────────────────────────┐
│                    ORCHESTRATOR                          │
│         (Coordinates subagents, maintains plan)          │
└─────────────────────────┬───────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
        ▼                 ▼                 ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐
│   ANALYZER    │ │  IMPLEMENTER  │ │   VERIFIER    │
│               │ │               │ │               │
│ - Read code   │ │ - Write code  │ │ - Run tests   │
│ - Understand  │ │ - Follow spec │ │ - Check types │
│ - Plan changes│ │ - Single file │ │ - Validate    │
└───────────────┘ └───────────────┘ └───────────────┘
```

### Subagent Types

**Analyzer Agent**
- Input: Question about codebase
- Output: Analysis with file references
- Tools: Read, Grep, Glob
- No write access

**Implementer Agent**
- Input: Specific implementation task
- Output: Code changes
- Tools: Read, Edit, Write
- Limited to specified files

**Verifier Agent**
- Input: Changes to verify
- Output: Pass/fail with details
- Tools: Bash (test commands)
- No write access

**Simplifier Agent**
- Input: Working but complex code
- Output: Cleaner version
- Runs after implementation passes tests
- Maintains functionality

### Example: Adding a Feature

```
1. ORCHESTRATOR receives: "Add user authentication"

2. ANALYZER examines codebase:
   - Finds existing auth patterns
   - Identifies files to modify
   - Returns analysis

3. ORCHESTRATOR creates plan:
   - Task 1: Add auth middleware
   - Task 2: Create login endpoint
   - Task 3: Add protected routes
   - Task 4: Write tests

4. For each task:
   a. IMPLEMENTER writes code
   b. VERIFIER checks it works
   c. If fail, IMPLEMENTER fixes
   d. If pass, continue

5. SIMPLIFIER reviews final code

6. ORCHESTRATOR reports completion
```

### Why It Works

1. **Focus**: Each agent does one thing well
2. **Bounded Context**: No overwhelming context
3. **Quality Control**: Verification between steps
4. **Recovery**: Failures are isolated and recoverable

---

## Pattern 4: Structured Prompts

### The Problem

Unstructured prompts lead to:
- Inconsistent outputs
- Missing requirements
- Misunderstood context
- Variable quality

### The Solution

Use structured prompt templates with clear sections:

```markdown
# Task: [Clear, Specific Title]

## Context
[What the agent needs to know about the current state]

## Objective
[Single, clear goal to accomplish]

## Constraints
- [Constraint 1]
- [Constraint 2]
- [Things NOT to do]

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Tests pass
- [ ] Types check

## Examples
[Good examples of similar work]

## Output Format
[What the response should look like]
```

### Template: Bug Fix

```markdown
# Task: Fix [Bug Description]

## Context
- File: [path/to/file.ts]
- Current behavior: [what happens now]
- Expected behavior: [what should happen]
- Reproduction: [how to trigger the bug]

## Objective
Fix the bug while maintaining existing functionality.

## Constraints
- Do not change the public API
- Do not modify unrelated code
- Add a test that would have caught this bug

## Acceptance Criteria
- [ ] Bug is fixed
- [ ] New test covers the fix
- [ ] All existing tests pass
- [ ] No type errors

## Related Code
[Paste relevant code snippets]
```

### Template: New Feature

```markdown
# Task: Implement [Feature Name]

## Context
- Location: [where this fits in the codebase]
- Dependencies: [what this feature needs]
- Related features: [similar existing code]

## Objective
[Detailed description of the feature]

## Constraints
- Follow existing patterns in [reference file]
- Use [specific libraries/patterns]
- Do not modify [protected areas]

## Acceptance Criteria
- [ ] Feature works as specified
- [ ] Edge cases handled
- [ ] Error states handled
- [ ] Tests written and passing
- [ ] Types complete

## User Stories
1. As a [user], I want to [action] so that [benefit]
2. ...
```

### Why It Works

1. **Clarity**: No ambiguity about what's needed
2. **Completeness**: All requirements captured
3. **Consistency**: Same structure every time
4. **Measurability**: Clear success criteria

---

## Putting It Together

Boris Cherny's patterns form an integrated system:

```
┌─────────────────────────────────────────────────────────┐
│                    CLAUDE.md                            │
│              (Provides codebase context)                │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│              STRUCTURED PROMPT                          │
│         (Clear task with constraints)                   │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│              SUBAGENT EXECUTION                         │
│      (Decomposed, focused implementation)               │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│              VERIFICATION PIPELINE                      │
│         (Types, Tests, Build, E2E)                      │
└─────────────────────────────────────────────────────────┘
```

This creates a reliable, repeatable process for AI-assisted development that produces production-quality code.

---

## Key Takeaways

1. **Context is King**: CLAUDE.md eliminates guessing
2. **Trust but Verify**: Multi-layer verification catches errors
3. **Divide and Conquer**: Subagents handle complexity
4. **Structure Enables Quality**: Templates ensure completeness

These patterns aren't just about making AI better—they improve the entire development process, with or without AI assistance.

---

## References

- Boris Cherny's writings on AI-assisted development
- TypeScript best practices
- Modern verification pipeline design
- Agent architecture patterns
