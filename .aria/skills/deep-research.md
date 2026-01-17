# Deep Research Skill

> Systematic web research with iterative refinement, source tracking, and HITL gates

---
version: 1.0.0
modes: [STANDARD, FULL, FULL+]
triggers: [research question, "deep research", "investigate", "find out about"]
inputs: [research question, optional context/constraints]
outputs: [research-output.json, IDEA.md, optional FOCUS.md/slides]
dependencies: [WebSearch tool, brainstorming skill]
---

## When to Use

Use this skill when:
- User has a research question (not code-related)
- Topic requires multiple sources and synthesis
- User says "research", "investigate", "deep dive", "find out about"
- Question goes beyond simple factual lookup

**Skip when:**
- Simple factual question (use WebSearch directly)
- Code/architecture research (use discovery skill)
- Analyzing a specific article/paper (use researcher skill)
- LITE mode unless explicitly requested

---

## Workflow Overview

```
Question → Depth Selection → Query Formulation → Search Loop → Synthesis → Output
              ↓                    ↓                  ↓            ↓
           [HITL]              [HITL]             [HITL]       [HITL]
```

**Key difference from simple search:** Iterative refinement with human checkpoints.

---

## HITL Gate 1: Research Depth Selection

Before starting, determine research depth:

```
RESEARCH DEPTH SELECTION

Question: "[user's question]"

Choose research depth:

[1] Quick (5-10 min)
    - 2-3 searches, top sources only
    - Brief synthesis, no deep verification
    - Best for: Background info, quick answers

[2] Standard (15-30 min) - RECOMMENDED
    - 5-8 searches, multiple source types
    - Cross-reference key claims
    - Best for: Most research questions

[3] Deep (30-60 min)
    - 10-15 searches, exhaustive source coverage
    - Verify claims across 3+ sources
    - Best for: Important decisions, complex topics

[4] Exhaustive (60+ min)
    - Comprehensive literature review
    - Academic + industry + community sources
    - Confidence scoring on all claims
    - Best for: Critical decisions, novel domains
```

**Mode defaults:**
- STANDARD mode → defaults to [2] Standard
- FULL/FULL+ mode → defaults to [3] Deep

---

## HITL Gate 2: Query Strategy Selection

After depth selection, formulate search strategy:

```
QUERY STRATEGY

Topic: "[extracted topic]"
Depth: [selected depth]

Choose search strategy:

[a] Broad Scan (recommended for unfamiliar topics)
    - Start wide, then narrow based on findings
    - Good for: Discovering the landscape

[b] Focused Drill (recommended when you know what you need)
    - Start with specific queries
    - Good for: Known unknowns

[c] Comparative (recommended for decisions)
    - Search for multiple perspectives/options
    - Good for: "X vs Y", "best approach for Z"

[d] Temporal (recommended for evolving topics)
    - Track how topic has changed over time
    - Good for: "Current state of X", trends

[e] Custom
    - Define your own search sequence
```

---

## HITL Gate 3: Query Approval

Present initial queries for approval:

```
PROPOSED SEARCH QUERIES

Based on: [strategy] strategy, [depth] depth

Round 1 queries:
1. "[query 1]" - [what we expect to find]
2. "[query 2]" - [what we expect to find]
3. "[query 3]" - [what we expect to find]

[a]pprove / [e]dit queries / [c]hange strategy
```

---

## Search Execution Loop

### Per-Round Process

```
For each search round:
  1. Execute queries (parallel where possible)
  2. Evaluate source quality
  3. Extract key findings
  4. Identify gaps/contradictions
  5. Formulate follow-up queries
  6. HITL checkpoint (if configured)
```

### Source Quality Evaluation

Rate each source:

| Score | Criteria | Examples |
|-------|----------|----------|
| **A** | Official/authoritative | Docs, academic papers, official blogs |
| **B** | Reputable secondary | Tech blogs, established publications |
| **C** | Community knowledge | Stack Overflow, Reddit (high-vote) |
| **D** | Unverified | Random blogs, low-vote answers |
| **F** | Unreliable | SEO farms, outdated, contradicts evidence |

### Finding Extraction Format

```json
{
  "finding_id": "F1",
  "claim": "What the source says",
  "source_url": "https://...",
  "source_quality": "A|B|C|D",
  "confidence": 0.0-1.0,
  "corroborated_by": ["F3", "F7"],
  "contradicted_by": [],
  "notes": "Context or caveats"
}
```

---

## HITL Gate 4: Mid-Research Checkpoint

After initial rounds, present findings:

```
RESEARCH CHECKPOINT

Rounds completed: [N] of [expected]
Sources consulted: [M]
Findings extracted: [K]

Key findings so far:
1. [High-confidence finding] (confidence: 0.9, sources: 3)
2. [Medium-confidence finding] (confidence: 0.7, sources: 2)
3. [Needs verification] (confidence: 0.5, sources: 1)

Gaps identified:
- [Gap 1] - proposed query: "[query]"
- [Gap 2] - proposed query: "[query]"

Contradictions found:
- [Source A] says X, [Source B] says Y

Options:
[c]ontinue with proposed queries
[r]edirect - change focus based on findings
[d]eepen - more queries on specific finding
[s]ynthesize now - enough information gathered
[a]bort - stop research
```

**When to trigger:**
- Quick: After round 1
- Standard: After rounds 2-3
- Deep: After rounds 3-5
- Exhaustive: After every 3-4 rounds

---

## HITL Gate 5: Synthesis Approach

Before synthesizing, confirm approach:

```
SYNTHESIS OPTIONS

Total findings: [N]
High-confidence: [X]
Needs verification: [Y]

Choose synthesis approach:

[1] Executive Summary
    - 1-page synthesis with key takeaways
    - Best for: Quick decisions, sharing with others

[2] Structured Analysis (RECOMMENDED)
    - Full IDEA.md with sections
    - Findings organized by theme
    - Confidence levels on each claim

[3] Comparative Matrix
    - Side-by-side comparison of options/approaches
    - Best for: Decision-making between alternatives

[4] Annotated Bibliography
    - Source-by-source summary
    - Best for: Further research, academic use

[5] All of the above
    - Complete package
    - Best for: Important topics, future reference
```

---

## Output Formats

### research-output.json

```json
{
  "id": "research-YYYYMMDD-HHMMSS",
  "question": "Original research question",
  "depth": "quick|standard|deep|exhaustive",
  "strategy": "broad|focused|comparative|temporal|custom",
  "started_at": "ISO timestamp",
  "completed_at": "ISO timestamp",
  "rounds": [
    {
      "round": 1,
      "queries": ["query1", "query2"],
      "sources_found": 12,
      "sources_used": 5,
      "findings_extracted": 8
    }
  ],
  "sources": [
    {
      "url": "https://...",
      "title": "Source title",
      "quality": "A",
      "accessed_at": "ISO timestamp",
      "findings": ["F1", "F3"]
    }
  ],
  "findings": [
    {
      "id": "F1",
      "claim": "...",
      "confidence": 0.85,
      "sources": ["url1", "url2"],
      "corroborated": true
    }
  ],
  "contradictions": [
    {
      "claim_a": "...",
      "source_a": "...",
      "claim_b": "...",
      "source_b": "...",
      "resolution": "Claim A appears more current/authoritative"
    }
  ],
  "gaps": ["Things we couldn't find"],
  "synthesis_approach": "structured|executive|matrix|bibliography",
  "hitl_decisions": [
    {"gate": "depth", "choice": "standard", "timestamp": "..."},
    {"gate": "strategy", "choice": "broad", "timestamp": "..."}
  ]
}
```

### IDEA.md (Structured Analysis)

```markdown
# Research: [Topic]

**Question:** [Original question]
**Depth:** [Level] | **Strategy:** [Type] | **Date:** [Date]

## Executive Summary

[2-3 paragraph synthesis of key findings]

## Key Findings

### [Theme 1]

**Finding:** [Claim]
**Confidence:** [HIGH|MEDIUM|LOW]
**Sources:** [List with links]

[Details and context]

### [Theme 2]
...

## Contradictions & Debates

| Topic | View A | View B | Assessment |
|-------|--------|--------|------------|
| ... | ... | ... | ... |

## Gaps & Unknowns

- [What we couldn't determine]
- [Areas needing more research]

## Sources

### Tier A (Authoritative)
- [Source](url) - [What it contributed]

### Tier B (Reputable)
- ...

### Tier C (Community)
- ...

## Methodology

- Rounds: [N]
- Queries: [List]
- Total sources consulted: [M]
- Date range of sources: [Range]
```

---

## Search Patterns

### Query Templates by Strategy

**Broad Scan:**
```
Round 1: "[topic] overview", "[topic] introduction", "what is [topic]"
Round 2: "[topic] best practices", "[topic] common approaches"
Round 3: "[topic] [specific aspect from R1]", "[topic] [another aspect]"
```

**Focused Drill:**
```
Round 1: "[specific question]", "[topic] [specific aspect]"
Round 2: "[detail from R1] how to", "[detail from R1] examples"
Round 3: "[edge case]", "[topic] limitations"
```

**Comparative:**
```
Round 1: "[A] vs [B]", "[A] pros cons", "[B] pros cons"
Round 2: "[A] [specific criterion]", "[B] [specific criterion]"
Round 3: "[A] [B] comparison [year]", "when to use [A] vs [B]"
```

**Temporal:**
```
Round 1: "[topic] [current year]", "[topic] latest"
Round 2: "[topic] history", "[topic] evolution"
Round 3: "[topic] future", "[topic] trends [current year]"
```

### Domain-Specific Modifiers

| Domain | Add to queries |
|--------|----------------|
| Technical | "documentation", "RFC", "specification" |
| Business | "case study", "market analysis", "industry report" |
| Scientific | "research paper", "study", "peer reviewed" |
| Legal | "regulation", "compliance", "legal requirements" |
| Security | "OWASP", "CVE", "security advisory" |

---

## Confidence Scoring

### Claim Confidence Formula

```
base_confidence = source_quality_score (A=1.0, B=0.8, C=0.6, D=0.4)

adjustments:
  +0.1 per corroborating source (max +0.3)
  -0.2 if contradicted by higher-quality source
  -0.1 if source is >2 years old (for fast-moving topics)
  +0.1 if from official/primary source

final_confidence = min(1.0, base_confidence + adjustments)
```

### Confidence Labels

| Score | Label | Meaning |
|-------|-------|---------|
| 0.9+ | VERY HIGH | Multiple authoritative sources agree |
| 0.7-0.89 | HIGH | Good sources, some corroboration |
| 0.5-0.69 | MEDIUM | Limited sources or some uncertainty |
| 0.3-0.49 | LOW | Single source or quality concerns |
| <0.3 | UNVERIFIED | Treat as hypothesis, needs verification |

---

## Mode Variations

### LITE Mode

Skip deep research. Use simple WebSearch flow:
- Single search round
- No HITL gates (except destructive actions)
- Brief summary output
- No confidence scoring

### STANDARD Mode

Full workflow with streamlined HITL:
- Gate 1 (depth): Show, default to Standard
- Gate 2 (strategy): Show, recommend based on question
- Gate 3 (queries): Show first round only
- Gate 4 (checkpoint): After round 2-3
- Gate 5 (synthesis): Show, recommend Structured

### FULL/FULL+ Mode

Maximum oversight:
- All HITL gates active
- Show all query rounds for approval
- Checkpoint after every 2 rounds
- Full source quality audit
- Contradiction resolution required before synthesis

---

## Integration with ARIA Research Flow

After deep research completes:

```
Deep Research Complete
         ↓
   research-output.json saved
   IDEA.md generated
         ↓
┌─────────────────────────────────────┐
│ HITL: Continue to slides/prototype? │
│ [s]lides - generate presentation    │
│ [p]rototype - build working demo    │
│ [b]oth - slides then prototype      │
│ [d]one - research complete          │
└─────────────────────────────────────┘
         ↓
   If slides: → slide-generation skill
   If prototype: → prototyping skill
   If done: → report-writer skill
```

---

## Traceability

### Signal Emissions

```bash
# Research started
emit_signal "research_started" "deep-research" "init" \
    "question=$QUESTION" \
    "depth=$DEPTH" \
    "strategy=$STRATEGY"

# Search round complete
emit_signal "search_round" "deep-research" "search" \
    "round=$ROUND" \
    "queries=$QUERY_COUNT" \
    "sources_found=$SOURCE_COUNT" \
    "findings=$FINDING_COUNT"

# HITL decision
emit_signal "hitl_decision" "deep-research" "checkpoint" \
    "gate=$GATE_NAME" \
    "choice=$USER_CHOICE"

# Research complete
emit_signal "research_complete" "deep-research" "synthesis" \
    "total_rounds=$ROUNDS" \
    "total_sources=$SOURCES" \
    "total_findings=$FINDINGS" \
    "confidence_avg=$AVG_CONFIDENCE"
```

### Decision Blocks

Emit decisions for:
- Source quality assessments
- Contradiction resolutions
- Query refinement choices
- Synthesis approach selection

---

## Example: Complete Flow

**User:** "What's the current state of WebAssembly for server-side applications?"

**Gate 1 - Depth:**
```
RESEARCH DEPTH SELECTION

Question: "What's the current state of WebAssembly for server-side applications?"

Choose research depth:
[1] Quick (5-10 min)
[2] Standard (15-30 min) - RECOMMENDED
[3] Deep (30-60 min)
[4] Exhaustive (60+ min)

> User selects: [2] Standard
```

**Gate 2 - Strategy:**
```
QUERY STRATEGY

Topic: "WebAssembly server-side"
Depth: Standard

[a] Broad Scan - RECOMMENDED (unfamiliar/evolving topic)
[b] Focused Drill
[c] Comparative
[d] Temporal

> User selects: [a] Broad Scan
```

**Gate 3 - Queries:**
```
PROPOSED SEARCH QUERIES (Round 1)

1. "WebAssembly server-side 2024" - current state overview
2. "WASI WebAssembly System Interface" - core technology
3. "WebAssembly edge computing" - major use case

[a]pprove / [e]dit / [c]hange strategy

> User: [a]pprove
```

**[Search Round 1 executes...]**

**Gate 4 - Checkpoint:**
```
RESEARCH CHECKPOINT

Rounds completed: 1 of 3
Sources consulted: 8
Findings extracted: 12

Key findings so far:
1. WASI is the standard interface for server-side WASM (confidence: 0.9)
2. Major runtimes: Wasmtime, WasmEdge, Wasmer (confidence: 0.85)
3. Production adoption is early but growing (confidence: 0.7)

Gaps identified:
- Performance benchmarks vs native
- Security model details

[c]ontinue / [r]edirect / [d]eepen / [s]ynthesize / [a]bort

> User: [c]ontinue
```

**[Rounds 2-3 execute...]**

**Gate 5 - Synthesis:**
```
SYNTHESIS OPTIONS

Total findings: 28
High-confidence: 18
Needs verification: 4

[1] Executive Summary
[2] Structured Analysis - RECOMMENDED
[3] Comparative Matrix
[4] Annotated Bibliography
[5] All of the above

> User: [2] Structured Analysis
```

**[Synthesis generates IDEA.md]**

**Final HITL:**
```
RESEARCH COMPLETE

Outputs:
- .aria/docs/research-output.json
- .aria/docs/IDEA.md

Continue?
[s]lides - generate presentation
[p]rototype - build working demo
[b]oth
[d]one

> User: [d]one
```

---

## Error Handling

### Search Failures

```
If WebSearch fails:
  1. Retry with exponential backoff (2s, 4s, 8s)
  2. If still failing, try alternative query formulation
  3. If persistent, HITL: "Search unavailable. [r]etry / [s]kip / [a]bort"
```

### Source Access Issues

```
If source URL is inaccessible:
  1. Note in source record: "access_failed": true
  2. Try archive.org fallback
  3. Continue with other sources
  4. Report in final synthesis
```

### Insufficient Sources

```
If <3 quality sources found after 2 rounds:
  HITL: "Limited sources available for this topic."
  Options:
  [b]roaden search terms
  [a]ccept limited sources
  [t]ry different angle
  [s]top research
```

---

## Tips

- **Start broad, then narrow** - Initial queries inform better follow-ups
- **Trust the hierarchy** - A-tier sources trump C-tier, even with corroboration
- **Note contradictions explicitly** - They're often more informative than agreements
- **Time-box appropriately** - Deep doesn't mean infinite
- **Confidence is not certainty** - 0.9 still means 10% chance of being wrong
- **Gaps are findings too** - "We don't know X" is valuable information

---

## Comparison: Deep Research vs Native Claude Search

| Aspect | Native WebSearch | ARIA Deep Research |
|--------|------------------|-------------------|
| Speed | Fast (seconds) | Slower (minutes) |
| Depth | Single query | Iterative refinement |
| Oversight | None | HITL gates |
| Traceability | None | Full decision trail |
| Source tracking | Implicit | Explicit with ratings |
| Confidence | Implicit | Scored per claim |
| Output | Prose response | Structured artifacts |

**Use Native when:** Quick answer, simple factual lookup, exploration
**Use Deep Research when:** Important decisions, complex topics, need to show work

---

*Systematic research with human oversight - find the truth, not just an answer.*
