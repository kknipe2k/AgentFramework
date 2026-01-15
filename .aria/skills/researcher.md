# Researcher Skill

> Extract implementable concepts from article summaries

## Purpose

Given an article summary (from NotebookLM or manual input), identify:
1. Key technical concepts that can be coded
2. Specific examples to implement
3. Dependencies and prerequisites
4. Potential challenges

## Input

Article summary text, typically from:
- NotebookLM export
- Manual copy/paste
- URL fetch + summarization

## Output Format

```json
{
  "title": "Article title or topic",
  "source": "URL or reference",
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
  "recommended_order": ["C1", "C2", "..."],
  "questions_for_hitl": [
    "Which concepts should we prioritize?",
    "What language/framework preference?",
    "Any concepts to skip?"
  ]
}
```

## Prompt Template

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

## Technical Papers with Math

When processing papers with mathematical formulas:

**Always expand formulas for new learners:**

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

3. **In IDEA.md output:**
   - Include "Math Concepts" section
   - Each formula gets full expansion
   - Link formulas to code implementation

**Add to JSON output:**
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

## HITL Checkpoint

After extraction, pause for human review:
- Confirm concept selection
- Adjust priorities
- Add constraints or preferences
- Skip irrelevant concepts

## Integration

Called by: Research flow in `CLAUDE.md`, brainstorming skill
Output to: `.aria/docs/research-output.json`
Feeds into: `brainstorming.md` → `IDEA.md`

---

*Workflow inspired by article-to-code patterns. Clean-room implementation for ARIA.*
