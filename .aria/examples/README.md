# ARIA: Paper to Interactive Explainer

> **Give ARIA your paper. Get an interactive HTML explainer.**

---

## Quick Start

### Option 1: Full Flow (Recommended)

Give ARIA a paper or article, and it will:
1. Extract key concepts
2. Generate IDEA.md synthesis
3. Create interactive HTML explainer

```
You: "Research this paper and create an interactive explainer"
     [paste paper content or provide URL]

ARIA:
- Extracts concepts → .aria/docs/research-output.json
- Synthesizes → .aria/docs/IDEA.md
- Generates → explainer-[topic].html
```

### Option 2: Direct Generation

If you already have an IDEA.md:

```bash
python .aria/scripts/generate-explainer.py your-IDEA.md output.html
```

---

## How It Works

### Step 1: Research Phase

ARIA's researcher skill extracts from your paper:
- Key concepts with definitions
- Core mechanisms and how they work
- Why it matters (significance)
- Relationships between concepts

Output: `.aria/docs/research-output.json`

### Step 2: Synthesis Phase

ARIA's brainstorming skill synthesizes:
- Executive summary
- Structured concept breakdowns
- Synthesis matrix (concept → problem → mechanism → benefit)
- Key takeaways

Output: `.aria/docs/IDEA.md`

### Step 3: Generation Phase

The explainer generator creates:
- Interactive HTML with tabbed navigation
- Concept cards with visual hierarchy
- Synthesis tables
- Dark theme, responsive design
- No dependencies (single HTML file)

Output: `explainer-[topic].html`

---

## IDEA.md Format

The generator expects this structure:

```markdown
# Title

> Subtitle or tagline

## Summary

Overview paragraph...

## Key Concepts

- **Concept Name**: Description of the concept...

- **Another Concept**: Another description...

## How It Works

Explanation section...

## Synthesis Matrix

| Component | Problem Solved | Mechanism | Benefit |
|-----------|---------------|-----------|---------|
| X | Y | Z | W |

## Key Takeaways

1. First takeaway
2. Second takeaway

## Sources

- Source 1
- Source 2
```

See `sample-IDEA.md` for a complete example.

---

## Example Output

The `sample-output.html` file was generated from `sample-IDEA.md` (explaining the Transformer architecture).

Open it in a browser to see:
- Tabbed navigation (Overview, Concepts, sections, Synthesis, Takeaways)
- Concept cards with numbered icons
- Responsive dark theme
- Clean typography

---

## Customization

### Adding More Sections

Any `## Heading` in IDEA.md becomes a navigable section.

### Synthesis Matrix

Use a markdown table with any columns. Common patterns:
- Concept | Problem | Solution | Benefit
- Component | Input | Output | Purpose
- Feature | Before | After | Impact

### Styling

Edit the `<style>` block in `generate-explainer.py` or modify the output HTML directly.

---

## Full Workflow Example

```
You: Research this paper and create an interactive explainer:

     "Attention Is All You Need" introduces the Transformer architecture...
     [full paper text]

ARIA:
SIZE: MEDIUM
MODE: STANDARD
Reason: Research flow, multiple artifacts

Extracting concepts...
✓ Identified 5 key concepts
✓ Mapped relationships

Synthesizing...
✓ Generated IDEA.md

HITL: Generate interactive HTML explainer?
[y]es / [n]o

You: y

ARIA:
✓ Generated explainer-attention-is-all-you-need.html
  Open in browser to view.
```

---

## Files in This Directory

| File | Purpose |
|------|---------|
| `sample-IDEA.md` | Example input showing expected format |
| `sample-output.html` | Generated output from sample |
| `aria-demo.html` | ARIA framework explainer (meta-demo) |
| `FOCUS.md` | NotebookLM input for slide generation |

---

## NotebookLM Slides

For richer slides with audio:
1. Upload `FOCUS.md` to NotebookLM
2. Request presentation generation
3. Download and share

---

## Technical Details

### Generator Script

```
.aria/scripts/generate-explainer.py

Usage:
  python generate-explainer.py <input> [output.html]

Inputs:
  - IDEA.md (markdown)
  - research-output.json (JSON)

Output:
  - Single self-contained HTML file
  - No external dependencies
  - Works offline
```

### What Gets Parsed

From IDEA.md:
- `# Title` → Page title
- `> Quote` → Subtitle
- `## Summary` / `## Overview` → Overview section
- `## Key Concepts` / `## Concepts` → Concept cards
- `## Synthesis` / `## Matrix` → Table
- `## Takeaways` / `## Conclusions` → Highlighted list
- `## Sources` / `## References` → Footer citations
- Any other `## Section` → Additional tab

---

*ARIA: Turning research into understanding.*
