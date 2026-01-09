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
