# ARIA Cheatsheet

> One-page quick reference

---

## Mode Selection

```
SMALL (1-5 tasks)     → LITE      → Fast, minimal overhead
MEDIUM (6-15 tasks)   → STANDARD  → Normal workflow
LARGE (16-40 tasks)   → FULL      → Maximum oversight
X-LARGE (40+ tasks)   → FULL+     → Epic-level management
```

**Override:** "use FULL mode" or "keep it lite"

---

## Skill Triggers

| Say This | Skill Invoked |
|----------|---------------|
| "plan this", "design" | planning |
| "explore options", "brainstorm" | brainstorming |
| "show mockup", "prototype" | prototyping |
| "what does this codebase do" | discovery |
| (test fails) | debugging |
| "analyze this paper" | researcher |
| "generate slides", "presentation" | slide-generation |

---

## Workflows

**Build:** brainstorm → prototype? → plan → execute → report

**Bug Fix:** debug → plan(lite) → execute

**Modify:** discovery → plan → execute

**Research:** researcher → brainstorm → IDEA.md → slides? → prototype? → plan → execute

---

## HITL Checkpoints

Stop and ask before:
- Deleting files
- Modifying auth/payments/security
- Changing config files (package.json, etc.)
- Installing dependencies
- Anything in "don't touch" areas

---

## Verification

After every task:
```bash
bash .aria/verify.sh
```

**No verify.sh?** Fallback: npm test → pytest → cargo test → lint → tsc

**LITE exception:** Skip if no tests, but warn

---

## Failure Handling

| Mode | Threshold | Action |
|------|-----------|--------|
| LITE | 1 failure | Stop, report |
| STANDARD | 2 failures | Stop, report |
| FULL/FULL+ | 3 failures | Escalation prompt |

---

## Key Files

| File | Purpose |
|------|---------|
| `CLAUDE.md` | Entry point, mode definitions |
| `.aria/state/current-plan.json` | Active plan |
| `.aria/state/progress.json` | Task status |
| `.aria/project-context.md` | Codebase knowledge |
| `.aria/design-notes.md` | Decision log |
| `.aria/docs/IDEA.md` | Research synthesis |
| `.aria/outputs/FOCUS.md` | Slide outline |
| `.aria/outputs/slides-*.pptx` | Generated slides |

---

## Commands

| Command | Action |
|---------|--------|
| `/aria:plan` | Start planning |
| `/aria:status` | Show progress |
| `/aria:verify` | Run verification |

## Scripts

| Script | Action |
|--------|--------|
| `python .aria/scripts/serve-dashboard.py` | Open dashboard at :8420 |
| `python .aria/scripts/generate-slides.py` | Generate slides from IDEA.md |

---

## Quick Answers

**How to skip brainstorming?** → Say "just plan it"

**How to force HITL?** → Mark task with `"hitl": true` in plan

**How to skip a task?** → Say "skip" at HITL checkpoint

**How to abort?** → Say "abort" at any checkpoint

**How to change mode?** → Say "switch to FULL mode"

---

*Full docs: See `CLAUDE.md` and `.aria/skills/REGISTRY.md`*
