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

### Core Skills
| Say This | Skill Invoked |
|----------|---------------|
| "plan this", "design" | planning |
| "implement", "build it" | executing |
| (test fails) | debugging |
| "what does this codebase do" | discovery |
| "write tests first", "tdd" | tdd |

### Extended Skills (STANDARD+)
| Say This | Skill Invoked |
|----------|---------------|
| "explore options", "brainstorm" | brainstorming |
| "show mockup", "prototype" | prototyping |
| (during/after execution) | tracking |
| (3+ failures, between phases) | context-refresh |
| (all tasks complete) | report-writer |

### Research Skills
| Say This | Skill Invoked |
|----------|---------------|
| "analyze this paper" | researcher |
| "generate slides", "presentation" | slide-generation |
| "show summary", "metrics" | report-writer |

---

## Workflows

**Build:** brainstorm → prototype? → plan → execute → verify → report → dashboard?

**Bug Fix:** debug → plan(lite) → execute → verify

**Modify:** discovery → plan → execute → verify → report → dashboard?

**Research:** researcher → brainstorm → IDEA.md → slides? → prototype? → report

**End of Workflow (STANDARD+):**
```
All tasks complete → Summary report → HITL: View dashboard? [y/n/s]
```

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
| `/aria-summary` | Generate session summary |
| `/aria-dashboard` | Launch lineage dashboard |

## Scripts

| Script | Action |
|--------|--------|
| `python .aria/scripts/serve-dashboard.py` | Open dashboard at :8420 |
| `python .aria/scripts/generate-slides.py` | Generate slides from IDEA.md |
| `.aria/scripts/setup-project.sh <name>` | Create isolated project workspace |

## Workspace Setup

Keep ARIA pristine, create isolated test workspaces:

```bash
# Clone once
git clone https://github.com/kknipe2k/AgentFramework.git ~/aria-test

# Create project workspace
~/aria-test/.aria/scripts/setup-project.sh SVM

# Work in project folder
code ~/aria/eval/SVM
```

Results stay in project folder. ARIA stays clean.

---

## Quick Answers

**How to skip brainstorming?** → Say "just plan it"

**How to force HITL?** → Mark task with `"hitl": true` in plan

**How to skip a task?** → Say "skip" at HITL checkpoint

**How to abort?** → Say "abort" at any checkpoint

**How to change mode?** → Say "switch to FULL mode"

---

*Full docs: See `CLAUDE.md` and `.aria/skills/REGISTRY.md`*
