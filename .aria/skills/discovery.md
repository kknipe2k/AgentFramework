# Discovery Skill

> Explore unfamiliar codebase and document patterns before making changes

---
version: 1.0.0
modes: [LITE, STANDARD, FULL, FULL+]
triggers: [new codebase, "explore", "understand", start of Modify flow]
inputs: [codebase access]
outputs: [project-context.md]
dependencies: []
---

## When to Use

Use this skill when:
- Entering an unfamiliar codebase
- Starting Modify flow (before planning)
- User asks "what does this do" or "how is this structured"
- Need to identify "don't touch" areas

**Skip when:**
- You created the codebase in this session
- User says "I'll explain the structure"

---

## Workflow

### Step 1: Identify Project Type

Check for framework/language indicators:

```
CHECK ORDER:
1. package.json → Node/JS project
2. requirements.txt / pyproject.toml → Python
3. Cargo.toml → Rust
4. go.mod → Go
5. *.csproj → .NET
6. pom.xml / build.gradle → Java
```

**Output:** Project type + primary language

---

### Step 2: Map Directory Structure

Read top-level structure:

```bash
ls -la
```

**Look for:**
| Directory | Likely Purpose |
|-----------|----------------|
| `src/` | Source code |
| `lib/` | Libraries/utilities |
| `tests/` / `__tests__/` | Test files |
| `docs/` | Documentation |
| `scripts/` | Build/deploy scripts |
| `config/` | Configuration |
| `.github/` | CI/CD workflows |

**Output:** Structure map with purposes

---

### Step 3: Find Entry Points

Identify where execution starts:

| Project Type | Entry Points |
|--------------|--------------|
| Node | `package.json` → main, scripts |
| React/Next | `pages/`, `app/`, `index.tsx` |
| Python | `__main__.py`, `app.py`, `main.py` |
| CLI | `bin/`, command definitions |
| API | Route definitions, controllers |

**Output:** List of entry points

---

### Step 4: Identify Patterns

Look for architectural patterns:

```
PATTERNS TO DETECT:
[ ] MVC / MVVM / Clean Architecture
[ ] Monorepo / Multi-package
[ ] Microservices / Monolith
[ ] REST / GraphQL / tRPC
[ ] ORM (Prisma, SQLAlchemy, etc.)
[ ] State management (Redux, Zustand, etc.)
[ ] Testing framework (Jest, Pytest, etc.)
```

**Read these files:**
- README.md - stated architecture
- Config files - dependencies reveal patterns
- A few source files - coding conventions

**Output:** Detected patterns

---

### Step 5: Find "Don't Touch" Areas

Identify sensitive/stable code:

```
DON'T TOUCH INDICATORS:
- Auth/security modules
- Payment processing
- Database migrations (in production)
- Generated files (*.generated.*, dist/)
- Vendor/third-party code
- Files with "DO NOT EDIT" comments
- Core infrastructure (unless that's the task)
```

**Output:** List of areas requiring HITL approval

---

### Step 6: Document in project-context.md

Create `.aria/project-context.md`:

```markdown
# Project Context: [Name]

## Overview
- **Type:** [e.g., Next.js web app]
- **Language:** [e.g., TypeScript]
- **Created:** [date discovered]

## Structure
```
[directory tree]
```

## Entry Points
- [file]: [purpose]

## Key Patterns
- [pattern]: [where used]

## Dependencies (Notable)
- [dep]: [what it does]

## Don't Touch (HITL Required)
- [ ] [area]: [reason]

## Testing
- Framework: [name]
- Run with: [command]

## Build/Run
- Dev: [command]
- Build: [command]
- Test: [command]

## Notes
- [Any quirks or important context]
```

---

## Mode Variations

### LITE Mode

Quick scan (2-3 minutes):
- Project type
- Entry point
- Test command
- Skip detailed documentation

### STANDARD Mode

Standard discovery:
- Full workflow
- Create project-context.md
- Identify don't-touch areas

### FULL/FULL+ Mode

Comprehensive discovery:
- Full workflow
- Document all patterns
- Read key source files
- Map dependencies
- Note code quality observations

---

## Output

After discovery:
1. **project-context.md** created in `.aria/`
2. **Don't touch areas** identified for HITL
3. **Ready for planning** with context

---

## Tips

- **Don't read everything** - Sample strategically
- **Trust README** - But verify claims
- **Check recent commits** - Shows active areas
- **Note test coverage** - Indicates stability
- **Ask if stuck** - User knows their codebase
