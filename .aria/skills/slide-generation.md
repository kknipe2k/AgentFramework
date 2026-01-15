# Slide Generation Skill

Generate presentation decks from research artifacts using a two-phase approach.

---

## When to Use

- After IDEA.md is created
- After research JSON is generated  
- Before prototype decision
- User requests slides/presentation

---

## Workflow

```
IDEA.md + Sources → Focus Doc → Slides
                         ↓
              [NotebookLM or pptx]
```

---

## Phase 1: Create Focus Document

**Prompt Template:**

```
Analyze all sources and provide a structured synthesis:

a. The Core Ideas: Identify the top 3 foundational elements 
   (concepts, entities, or arguments) that are absolutely 
   required to understand this corpus. Define them briefly.

b. The Synthesis Matrix: Coalesce the findings by listing 
   the top 5-10 unifying ideas or themes that connect the 
   "Core Ideas" together. Explain how these ideas turn the 
   separate elements into a cohesive whole.
```

**Input Sources:**
- IDEA.md (summary)
- Original paper/whitepaper (depth)
- Additional docs (repos, READMEs, etc.)

**Output:** `.aria/outputs/FOCUS.md`

**HITL Checkpoint:** Confirm sources before generation
```
Sources for Focus doc:
1. .aria/docs/IDEA.md
2. [original paper path]
3. [additional docs...]

Proceed? [y]es / [e]dit sources / [c]ancel
```

---

## Phase 2: Generate Slides

**Prompt Template:**

```
Intent: Explain in detail the key ideas from these docs.
USE THE Focus doc as the guide to highlight each important 
aspect we need to bring forth and explain.

Must DO:
- Provide detailed learning deck to explain the workflow
- Be verbose
- Use unconventional spatial and verbal slide design 
  techniques rooted in cognitive science for maximum learning
- Make this a long deck (20+ slides if necessary)
- Use charts, graphs, flow diagrams, and other visuals 
  liberally to get the message across
- Break concepts down for ease of intake for new learners 
  or people unfamiliar with the content
- Use diagrams liberally
- Capture main steps in all workflows
- Ensure the high level process is clear
- Provide a clear concise view at the end of complete flow
```

**Input Sources:**
- FOCUS.md (structure/guide)
- IDEA.md (summarized content)
- Original paper (depth)

**HITL Checkpoint:** Choose output method
```
Generate slides via:
[1] NotebookLM (richer design, requires auth)
[2] Local pptx (reliable, no external dependency)
```

---

## NotebookLM Path

**Setup (one-time):**
```bash
pip install "notebooklm-py[browser]"
playwright install chromium
notebooklm login  # Opens browser for Google auth
```

**API Usage:**
```python
from notebooklm import NotebookLMClient

async with await NotebookLMClient.from_storage() as client:
    # 1. Create notebook
    notebook = await client.notebooks.create("Research Slides")

    # 2. Add sources
    await client.sources.add_file(notebook.id, "FOCUS.md")
    await client.sources.add_file(notebook.id, "IDEA.md")
    await client.sources.add_file(notebook.id, "paper.pdf")

    # 3. Generate slides with prompt
    await client.chat.ask(notebook.id, SLIDES_PROMPT)
    status = await client.artifacts.generate_slides(notebook.id)
    await client.artifacts.wait_for_completion(notebook.id, status.task_id)

    # 4. Download
    await client.artifacts.download_slides(notebook.id, ".aria/outputs/")
```

**Output:** `.aria/outputs/slides-[topic]-[date].pdf`

---

## pptx Fallback Path

Uses: `python-pptx` (no external auth)

**Structure from Focus doc:**
1. Title slide (topic from IDEA.md)
2. Overview slide (Core Ideas summary)
3. Core Idea slides (1 per concept, with diagrams)
4. Synthesis Matrix slides (themes with connections)
5. Workflow slides (process diagrams)
6. Deep dive slides (from original paper)
7. Summary slide (complete flow view)

**Output:** `.aria/outputs/slides-[topic]-[date].pptx`

---

## File Locations

| Artifact | Path |
|----------|------|
| Focus doc | `.aria/outputs/FOCUS.md` |
| NotebookLM slides | `.aria/outputs/slides-*.pdf` |
| pptx slides | `.aria/outputs/slides-*.pptx` |

---

## Integration Point

In ARIA research flow, after IDEA.md:

```
... → IDEA.md created → JSON created →

HITL: Generate presentation?
[y]es / [n]o, continue to prototype

If yes:
  → Phase 1: Focus doc
  → Phase 2: Slides (NBLM or pptx)
  
→ Prototype decision
```

---
