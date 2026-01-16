# Design Notes

This document captures AI reasoning, research, and design decisions.

---

## Current Session


### [ASSUMPTION] Framework Test
*2026-01-09 21:51*

Testing the ARIA framework end-to-end

---

### [RESEARCH] Agent Patterns
*2026-01-09 21:51*

Two-agent architecture (planner + executor) is common in production AI systems

---

### [DECISION] JSON vs YAML
*2026-01-09 21:51*

**Chosen:** JSON

**Alternatives considered:**
YAML, TOML

**Reasoning:** jq is reliable, YAML parsing in bash is fragile

---

### [DECISION] CLAUDE.md Size Acceptance
*2026-01-16*

**Chosen:** Accept CLAUDE.md exceeding 500-line guideline

**Alternatives considered:**
- Option A: Split into multiple files
- Option B: Refactor to reduce size

**Reasoning:**
The CLAUDE.md file serves as a comprehensive single-source-of-truth for ARIA mode definitions and workflow guidance. While it exceeds the typical 500-line guideline:

1. **Single-source-of-truth value** - Mode definitions are canonical here, other files reference this
2. **Coherent context** - Keeping all mode info together helps AI understand the full system
3. **Reduced cross-file navigation** - Splitting would require more file reads per session
4. **Traceability** - One place to update when modes change

**Mitigation:** Use clear section headers and table of contents for navigation.

---
