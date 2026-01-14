# Parking Lot

> Ideas to integrate - with implementation status

---

## Implementation Status Summary

| Item | Status | Notes |
|------|--------|-------|
| Brainstorming skill | ✅ DONE | `.aria/skills/brainstorming.md` |
| Systematic Debugging | ✅ DONE | `.aria/skills/debugging.md` |
| Researcher skill | ✅ DONE | `.aria/skills/researcher.md` |
| Report Writer skill | ✅ DONE | `.aria/skills/report-writer.md` |
| Context building | ✅ DONE | `.aria/skills/discovery.md` |
| Clarifying questions | ✅ DONE | In planning.md workflow |
| ccusage integration | ✅ DONE | In tracking.md |
| Context refresh | ✅ DONE | `.aria/skills/context-refresh.md` |
| Skill registry | ✅ DONE | `.aria/skills/REGISTRY.md` |
| Skill composition | ✅ DONE | `.aria/skills/COMPOSITION.md` |
| Cheatsheet | ✅ DONE | `.aria/docs/CHEATSHEET.md` |
| TDD skill | ✅ DONE | `.aria/skills/tdd.md` |
| ARIA CLI wrapper | ⏳ TODO | Future consideration |
| NotebookLM automation | ⏳ TODO | Local only, low priority |
| Publisher skills | ⏳ TODO | Extension, not core |

---

## 1. Superpowers - Brainstorming Phase ✅ IMPLEMENTED

**Source:** https://github.com/obra/superpowers

**Status:** Core concepts implemented

| Concept | Implementation |
|---------|----------------|
| Brainstorming before planning | ✅ `brainstorming.md` |
| Socratic questioning | ✅ In brainstorming workflow |
| Systematic Debugging | ✅ `debugging.md` |
| TDD workflow | ✅ `tdd.md` |
| Code Review workflow | ⏳ Not yet |

**Remaining from Superpowers:**
- Code review skill (cross-model review)
- Parallel agent dispatching (already using Task tool)

---

## 2. BuildFlow - Article to Code ✅ IMPLEMENTED

**Source:** https://github.com/BowTiedSwan/buildflow

**Status:** Core workflow implemented

| Component | Implementation |
|-----------|----------------|
| Researcher skill | ✅ `researcher.md` |
| Report Writer | ✅ `report-writer.md` |
| Research flow | ✅ In CLAUDE.md Use Cases |
| Math formula expansion | ✅ In researcher.md |

**Clean-room implementation** - concepts only, no copied code.

---

## 3. Plan Mode - Context Building ✅ IMPLEMENTED

**Source:** Matt Pocock (@mattpocockuk)

**Status:** Key insights implemented

| Insight | Implementation |
|---------|----------------|
| Read-only exploration first | ✅ `discovery.md` |
| Iterate until plan approved | ✅ In planning.md |
| Context carries to execution | ✅ `context-refresh.md` |
| Clarifying questions | ✅ In planning.md Step 1 |
| Summary at end | ✅ In planning.md format |

**Not implemented:**
- Dictation workflow docs (user-specific, not ARIA core)
- AGENTS.md config (using CLAUDE.md instead)

---

## 4. ccusage - Usage Tracking ✅ IMPLEMENTED

**Source:** https://github.com/ryoppippi/ccusage

**Status:** Documented in tracking.md

| Feature | Implementation |
|---------|----------------|
| Install instructions | ✅ Corrected (npm, not pipx) |
| Usage commands | ✅ Documented |
| Fallback when unavailable | ✅ Manual estimation |

---

## 5. ARIA CLI / Wrapper ⏳ NOT STARTED

**Concept:** Package ARIA as installable tool

**Status:** Parked - focus on testing current implementation first

**When to revisit:** After validating ARIA Hybrid works well in real projects

---

## 6. Claude Skills Review ⏳ PARTIALLY DONE

### 6.1 Anthropic Official Skills
**Status:** Not reviewed yet
**Priority:** Low - current skills are working

### 6.2 Planning-with-files (Manus Method)
**Status:** Not evaluated
**Priority:** Low - ARIA planning is functional

### 6.3 X-Article-Publisher-Skill
**Status:** Not implemented
**Priority:** Low - extension, not core

### 6.4 NotebookLM Skill
**Status:** Documented limitation (LOCAL ONLY)
**Priority:** Low - manual upload is acceptable

---

## What's Left (Prioritized)

### Should Do (High Value)
1. **Real-world testing** - Validate on actual projects

### Could Do (Medium Value)
2. **Code review skill** - Cross-model verification
3. **Anthropic skills review** - Check for patterns to adopt

### Won't Do (Low Value / Bloat)
- ~~ARIA CLI wrapper~~ - Premature optimization
- ~~NotebookLM automation~~ - Local only, manual works
- ~~Publisher skills~~ - Not core functionality
- ~~Dictation docs~~ - User-specific

---

## Removed from Consideration

| Item | Reason |
|------|--------|
| TROUBLESHOOTING.md | Bloat - debugging.md covers this |
| examples/ directory | Bloat - examples rot fast |
| metrics-dashboard.md | Over-engineering |
| scaffolding.md | One-time use, script instead |
| git-workflow.md | Project-specific, not ARIA |

---

## Done! Current Skill Set

**Core (7):**
- planning.md
- executing.md
- debugging.md
- discovery.md
- tdd.md
- context-refresh.md
- tracking.md

**Creative (2):**
- brainstorming.md
- prototyping.md

**Research (2):**
- researcher.md
- report-writer.md

**Meta (3):**
- REGISTRY.md
- COMPOSITION.md
- CHEATSHEET.md

**Total: 14 files** - Lean and complete.

---

*Last updated: 2026-01-14 - Major implementation pass complete*
