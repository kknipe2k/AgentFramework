# ARIA Parking Lot

Future ideas and features to consider.

---

## Voice Integration (Whisper)

**Source:** Tim Dettmers article on agent automation

**Why:**
- Inspect outputs while narrating (hands free)
- Faster than typing for guidance/direction
- Enables parallel session management
- Accessibility (carpal tunnel, etc.)

**What:**
- Voice-to-text for prompts
- Could integrate with existing tools
- Custom voice tool per Dettmers recommendation

**Priority:** Medium - quality of life, not blocking

---

## Process Optimization Metrics

**Source:** Tim Dettmers article

**Why:**
- Before automating, map current process
- Calculate: (time saved) vs (time to automate + overhead)
- Track if automation actually helped

**What:**
- Time tracking per task type
- Automation ROI calculator
- "Was this worth it?" retrospective

**Priority:** Low - nice to have for analysis

---

## Layered Skills (Progressive Disclosure)

**Source:** Effect Layers pattern, Vercel agent-skills repo

**Why:**
- Large skills could blow up context
- Priority ordering (CRITICAL → LOW) helps focus
- Only load detailed refs when needed

**What:**
- Skill summary (always loaded) + references/ folder (on demand)
- Priority tags on sections
- Agent fetches detail when needed

**Trigger:** When a skill exceeds ~500 lines

**Priority:** Low - YAGNI until skills get too big

---
