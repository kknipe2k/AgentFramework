# Slide Generation Skill

> Generate presentation decks from research artifacts using a two-phase approach

---
version: 1.0.0
modes: [STANDARD, FULL, FULL+]
triggers: [after IDEA.md, user requests slides, research complete]
inputs: [IDEA.md, original sources, FOCUS.md]
outputs: [FOCUS.md, NotebookLM URL or slides-*.pptx]
dependencies: [researcher, brainstorming]
---

## When to Use

Use this skill when:
- IDEA.md has been created from research
- User explicitly requests slides/presentation
- Research flow reaches slide decision point

**Skip when:**
- LITE mode (speed priority, no slides)
- User declines slide generation at HITL checkpoint
- No IDEA.md exists yet

---

## Workflow

```
IDEA.md + Sources → Focus Doc → Slides
                         ↓
              [NotebookLM or pptx]
```

**Two phases:**
1. **Focus Document** - Synthesize sources into structured Core + Synthesis
2. **Slide Generation** - Send to NotebookLM with verbatim prompt

---

## Phase 1: Create Focus Document

### HITL Checkpoint: Confirm Sources

```
Sources for Focus doc:
1. .aria/docs/IDEA.md
2. [original paper/article path]
3. [additional sources...]

Proceed? [y]es / [e]dit sources / [c]ancel
```

### Focus Prompt (VERBATIM - do not modify)

```
Analyze all sources and provide a structured synthesis:
1. The Core: Identify the top 3 foundational elements (concepts, entities, or arguments) that are absolutely required to understand this corpus. Define them briefly.
2. The Synthesis: Coalesce the findings by listing the top 5-10 unifying ideas or themes that connect the "Core Trinity" together. Explain how these ideas turn the separate elements into a cohesive whole.
```

### FOCUS.md Format Requirements (CRITICAL)

**ASCII-ONLY Characters Required** - NotebookLM upload fails on Windows cp1252 encoding.

| DO NOT USE | USE INSTEAD |
|------------|-------------|
| Box drawing (┌─┐└┘│) | Plain text borders (+--+) or none |
| Arrows (←→↔⇒) | Text arrows (->, <-, <->) |
| Bullets (•●○) | Dash (-) or asterisk (*) |
| Em dash (—) | Double dash (--) |
| Ellipsis (…) | Three dots (...) |
| Smart quotes ("") | Straight quotes ("") |

**Valid FOCUS.md structure:**
```markdown
# FOCUS: [Topic Name]

## The Core (Trinity)

1. **[Concept 1]**: Definition here
2. **[Concept 2]**: Definition here
3. **[Concept 3]**: Definition here

## The Synthesis

1. **[Theme 1]**: How it connects core ideas
2. **[Theme 2]**: How it connects core ideas
...
```

**Before generating FOCUS.md, validate:**
- No Unicode box-drawing characters
- No special arrows or bullets
- No smart quotes or em dashes
- Plain ASCII markdown only

**Input Sources:**
- IDEA.md (summary)
- Original paper/whitepaper (depth)
- Additional docs (repos, READMEs, etc.)

**Output:** `.aria/outputs/FOCUS.md`

---

## Phase 2: Generate Slides via NotebookLM

### HITL Checkpoint: Choose Method

```
Generate slides via:
[1] NotebookLM (richer design, requires auth) - RECOMMENDED
[2] Local pptx (reliable, no external dependency)
```

### Slide Prompt (VERBATIM - do not modify)

```
Intent: explain in detail the key ideas from these docs - USE THE [FOCUS doc] as the guide to highlight each important aspect we need to bring forth and explain

Must DO: provide detailed learning deck to explain the workflow - be verbose - use unconventional spatial and verbal slide design techniques rooted in cognitive science for maximum learning - make this a long deck 20 plus slides if necessary. Use charts, graphs, flow diagrams, and other visuals liberally to get the message across. Break concepts down for ease of intake for new learners or people unfamiliar with the content - again use diagrams liberally - capture main steps in all workflows - ensure the high level process is clear. Provide a clear concise view at the end of complete flow
```

**Input Sources to NotebookLM:**
- FOCUS.md (structure/guide)
- IDEA.md (summarized content)
- Original paper (depth)
- Any additional source documents

---

## NotebookLM Path

**IMPORTANT: Use the generate-slides.py script - do NOT implement manually.**

### Run Command

```bash
python .aria/scripts/generate-slides.py \
  --focus .aria/outputs/FOCUS.md \
  --idea .aria/docs/IDEA.md \
  --sources sources/[original-paper.pdf] \
  --method nblm
```

### What the Script Does

1. Creates a new NotebookLM notebook
2. Uploads FOCUS.md, IDEA.md, and original paper
3. Sends the verbatim slide generation prompt
4. Starts slide deck generation
5. Returns the notebook URL to user

### Expected Output

```
============================================================
SLIDE GENERATION STARTED
============================================================

Notebook URL: https://notebooklm.google.com/notebook/{id}

Next steps:
  1. Open the URL above in your browser
  2. Wait 5-10 minutes for slide deck to generate
  3. Download from NotebookLM when ready
============================================================
```

**ARIA's role ends here.** User downloads slides from NotebookLM when ready.

### DO NOT

- Manually create files and tell user to upload them
- Try to implement the NotebookLM API yourself
- Skip running the script
- Track slide completion status (user handles this)

### Setup (one-time, if not done)

```bash
pip install "notebooklm-py[browser]"
playwright install chromium
notebooklm login  # Opens browser for Google auth
```

---

## pptx Fallback Path

Uses: `python-pptx` (no external auth)

Only use if NotebookLM is unavailable.

**Structure from Focus doc:**
1. Title slide (topic from IDEA.md)
2. Overview slide (Core summary)
3. Core slides (1 per concept, with diagram placeholders)
4. Synthesis slides (themes with connections)
5. Workflow slides (process diagram placeholders)
6. Deep dive slides (from original paper)
7. Summary slide (complete flow view)

**Output:** `.aria/outputs/slides-[topic]-[date].pptx`

**Note:** pptx output contains placeholder diagrams. For richer visuals, use NotebookLM.

---

## Mode Variations

### LITE Mode
Skip slides entirely (speed priority).

### STANDARD Mode
Offer slides at HITL checkpoint. Default to NotebookLM if available.

### FULL/FULL+ Mode
Always offer slides. Include additional review checkpoint after generation.

---

## Integration Point

In ARIA research flow, after IDEA.md:

```
... → IDEA.md created → research-output.json created →

HITL: Generate presentation?
[y]es / [n]o, continue to prototype

If yes:
  → Phase 1: Create Focus doc (with HITL source confirmation)
  → Phase 2: Generate slides (NotebookLM or pptx)
  → Return notebook URL to user

→ Continue to prototype decision
```

---

## File Locations

| Artifact | Path |
|----------|------|
| Focus doc | `.aria/outputs/FOCUS.md` |
| NotebookLM slides | User downloads from notebook URL |
| pptx slides | `.aria/outputs/slides-*.pptx` |

---

## Traceability & Runtime Verification

The `generate-slides.py` script automatically emits signals to `.aria/state/signals.jsonl` for full traceability.

### Signals Emitted (NotebookLM Path)

| Signal | When | Verifies |
|--------|------|----------|
| `nblm_generation_start` | Script starts | Method selection |
| `nblm_notebook_created` | Notebook created | Notebook ID assigned |
| `nblm_prompt_sending` | Before prompt sent | Full prompt content logged |
| `nblm_prompt_sent` | After prompt delivered | Prompt reached NotebookLM |
| `nblm_deck_generation_started` | Deck task started | Generation initiated |
| `nblm_generation_complete` | All steps done | Full success with details |
| `nblm_generation_failed` | On error | Error type and details |

### Signals Emitted (pptx Path)

| Signal | When | Verifies |
|--------|------|----------|
| `pptx_generation_start` | Script starts | Method selection |
| `pptx_generation_complete` | File saved | Output path, slide count |
| `pptx_import_failed` | Library missing | Expected when not installed |

### Runtime Verification

After slide generation, verify signals were emitted:

```bash
python .aria/scripts/verify-slide-signals.py --verbose
```

**What it checks:**
- All required signals present
- Prompt was actually sent to NotebookLM
- Notebook ID and URL captured
- Generation task started successfully

**Example output:**
```
============================================================
  SLIDE GENERATION SIGNAL VERIFICATION
============================================================

Found 6 slide generation signal(s)

NotebookLM Signal Verification:
----------------------------------------
  [✓] nblm_generation_start
  [✓] nblm_notebook_created
  [✓] nblm_prompt_sending
  [✓] nblm_prompt_sent
  [✓] nblm_deck_generation_started
  [✓] nblm_generation_complete

Prompt Details:
  Length: 423 chars
  Preview: Intent: explain in detail the key ideas from these docs...

============================================================
  VERIFICATION PASSED
  Slide generation signals verified successfully.
============================================================
```

### Why This Matters

- **Audit trail**: Proves prompts were sent, not just claimed
- **Debugging**: Identify exactly where failures occur
- **Testing**: Integration tests can verify runtime behavior
- **Compliance**: Full traceability for research workflows

---

## Tips

- Always confirm sources before generating Focus doc
- Use NotebookLM when available - produces richer visuals
- Don't wait for slide completion - return URL immediately
- pptx is fallback only - contains placeholders, not rich diagrams
- FOCUS.md is the key artifact - slides depend on its quality

---

*Prompts are verbatim from user requirements. Do not modify without explicit approval.*
