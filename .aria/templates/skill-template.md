# [Skill Name]

> [One-line purpose - what this skill accomplishes]

---
version: 1.0.0
modes: [LITE, STANDARD, FULL, FULL+]
triggers: [keywords or conditions that activate this skill]
inputs: [required context, files, or data]
outputs: [files created, state changes, artifacts]
dependencies: [other skills that must run first, or "none"]
---

## When to Use

Use this skill when:
- [Condition 1]
- [Condition 2]
- [Condition 3]

**Skip when:**
- [Condition to skip]

---

## Workflow

### Step 1: [Name]

**Goal:** [What this step accomplishes]

**Actions:**
1. [Action 1]
2. [Action 2]
3. [Action 3]

**Output:** [What this step produces]

---

### Step 2: [Name]

**Goal:** [What this step accomplishes]

**Actions:**
1. [Action 1]
2. [Action 2]

---

## Mode Variations

### LITE Mode
[Abbreviated version for quick tasks]

### STANDARD Mode
[Normal workflow]

### FULL/FULL+ Mode
[Extended workflow with additional steps]

---

## HITL Checkpoints

Before these actions, stop and confirm:

- [ ] [Action requiring approval]
- [ ] [Risky action]

**Format:**
```
HITL CHECKPOINT: About to [action]
Proceed? [y]es / [n]o / [e]xplain
```

---

## Output Format

[Templates, schemas, or examples of what this skill produces]

```json
{
  "example": "output format"
}
```

---

## Handoff

**To next skill:**
- [Data or file to pass]
- [Context to preserve]

**Receives from previous skill:**
- [Expected input]

---

## Tips

- [Best practice 1]
- [Best practice 2]
- [Common mistake to avoid]

---

*See [REGISTRY.md](../skills/REGISTRY.md) for skill index*
