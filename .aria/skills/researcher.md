# Researcher Skill

> Extract implementable concepts from articles and research papers

---
version: 1.0.0
modes: [STANDARD, FULL, FULL+]
triggers: [research flow, article analysis, paper review]
inputs: [article summary, paper URL, source documents]
outputs: [research-output.json, feeds into IDEA.md]
dependencies: [none - first skill in research flow]
---

## When to Use

Use this skill when:
- User provides an article, paper, or research document
- Research flow is initiated in CLAUDE.md
- User asks to analyze or extract concepts from a source

**Skip when:**
- LITE mode (minimal research, direct to implementation)
- User just wants a quick summary without structured extraction
- Source is code-only (use discovery skill instead)

---

## Workflow

```
Source Document → Extract Concepts → research-output.json
                                           ↓
                              [brainstorming skill → IDEA.md]
```

---

## Input Sources

Article summary text, typically from:
- NotebookLM export
- Manual copy/paste
- URL fetch + summarization
- PDF content extraction

---

## Output Format

Save to: `.aria/docs/research-output.json`

```json
{
  "title": "Article title or topic",
  "source": "URL or reference",
  "extracted_at": "ISO timestamp",
  "concepts": [
    {
      "id": "C1",
      "name": "Concept name",
      "description": "What it does",
      "implementable": true,
      "complexity": "low|medium|high",
      "dependencies": ["list", "of", "deps"],
      "example_scope": "What a working example would demonstrate"
    }
  ],
  "math_concepts": [],
  "recommended_order": ["C1", "C2", "..."],
  "questions_for_hitl": [
    "Which concepts should we prioritize?",
    "What language/framework preference?",
    "Any concepts to skip?"
  ]
}
```

---

## Extraction Prompt Template

```
You are a technical researcher. Given the following article summary, extract implementable concepts.

## Article Summary
{summary}

## Your Task

1. Identify 3-7 key technical concepts from this article
2. For each concept, assess:
   - Can it be implemented as working code?
   - What's the complexity (low/medium/high)?
   - What dependencies would it need?
   - What would a minimal example demonstrate?

3. Recommend an implementation order (dependencies first)

4. Generate 2-3 questions for the human to clarify scope

Output as JSON matching the schema above.

Focus on concepts that:
- Have clear, testable outcomes
- Can be implemented in under 1 hour each
- Demonstrate the core idea without production complexity
```

---

## Technical Papers with Math

When processing papers with mathematical formulas:

### Always Expand Formulas for New Learners

1. **Break down each formula:**
   - State what it calculates in plain English
   - Define every variable/symbol
   - Explain the intuition behind the formula
   - Provide a concrete numeric example

2. **Example expansion:**
   ```
   Formula: L = -Σ yᵢ log(ŷᵢ)

   Plain English: Cross-entropy loss measures how wrong our predictions are

   Variables:
   - L = the loss value (lower is better)
   - yᵢ = actual label (0 or 1)
   - ŷᵢ = predicted probability (0 to 1)
   - Σ = sum over all samples

   Intuition: Penalizes confident wrong predictions heavily

   Example: If actual=1 and predicted=0.9, loss = -1×log(0.9) = 0.105 (small, good!)
            If actual=1 and predicted=0.1, loss = -1×log(0.1) = 2.303 (large, bad!)
   ```

3. **In output, include "math_concepts" section:**
   ```json
   {
     "math_concepts": [
       {
         "formula": "L = -Σ yᵢ log(ŷᵢ)",
         "name": "Cross-entropy loss",
         "plain_english": "Measures prediction error",
         "variables": {"L": "loss", "y": "actual", "ŷ": "predicted"},
         "intuition": "Penalizes confident wrong predictions",
         "code_mapping": "torch.nn.CrossEntropyLoss()"
       }
     ]
   }
   ```

---

## Mode Variations

### LITE Mode
Skip detailed extraction. Quick summary only:
- 2-3 key concepts
- No math expansion
- Minimal questions

### STANDARD Mode
Full extraction as documented:
- 3-7 concepts
- Math expansion if formulas present
- HITL checkpoint for priorities

### FULL/FULL+ Mode
Deep extraction with additional depth:
- All identifiable concepts
- Complete math expansion
- Multiple HITL checkpoints
- Cross-reference with related papers if available

---

## HITL Checkpoint

After extraction, pause for human review:

```
EXTRACTION COMPLETE

Concepts found: [N]
Math formulas: [M]

Review:
- Confirm concept selection
- Adjust priorities
- Add constraints or preferences
- Skip irrelevant concepts

[a]pprove / [e]dit / [s]kip concepts
```

---

## Integration

| Direction | Target |
|-----------|--------|
| Called by | Research flow in CLAUDE.md |
| Output to | `.aria/docs/research-output.json` |
| Feeds into | `brainstorming.md` → `IDEA.md` |
| Then | `slide-generation.md` (optional) |

---

## Traceability

Emit signals at key points:

```bash
emit_signal "research_started" "researcher" "extraction" \
    "source=$source_path"

emit_signal "research_complete" "researcher" "extraction" \
    "concepts_count=$N" \
    "math_concepts_count=$M" \
    "output=.aria/docs/research-output.json"
```

---

## Example: Complete Flow

**Input:** User provides ML paper URL

**Step 1:** Fetch and parse content
```bash
emit_signal "research_started" "researcher" "extraction" "source=paper.pdf"
```

**Step 2:** Run extraction prompt, get JSON

**Step 3:** If math found, expand formulas

**Step 4:** Save to research-output.json

**Step 5:** HITL checkpoint
```
Extracted 5 concepts, 3 math formulas.
Recommended order: C1 → C3 → C2 → C4 → C5

[a]pprove / [e]dit / [s]kip
```

**Step 6:** On approval, emit completion signal
```bash
emit_signal "research_complete" "researcher" "extraction" \
    "concepts_count=5" "math_concepts_count=3"
```

**Step 7:** Hand off to brainstorming skill

---

## Tips

- Focus on implementable concepts, not just theoretical ideas
- Math expansion is critical for technical papers - don't skip it
- Keep extraction focused on 1-hour implementation chunks
- Questions for HITL should help scope the prototype
- Emit signals for traceability throughout

---

*Clean-room implementation for ARIA research workflow.*
