# ARIA Skill & Agent Registry

**Purpose:** Complete documentation of every skill and agent, when/why/how they are called, and professional assessment grades.

---

## Quick Reference

| Skill | Grade | Category | Trigger Keywords |
|-------|-------|----------|------------------|
| planning.md | A- | Core | "plan", "/plan", mode start |
| executing.md | B+ | Core | Plan approval |
| debugging.md | B | Core | Test failure, error |
| discovery.md | B- | Core | "explore", new codebase |
| tdd.md | A | Core | "tdd", "test first" |
| context-refresh.md | C+ | Core | Long session, 3+ failures |
| brainstorming.md | B+ | Creative | "brainstorm", unclear approach |
| prototyping.md | B | Creative | Prototype decision |
| researcher.md | C+ | Research | Article/paper URL |
| slide-generation.md | C | Research | HITL slides decision |
| report-writer.md | B | Meta | Workflow completion |
| tracking.md | B+ | Meta | Every task (parallel) |
| REGISTRY.md | A- | Meta | Skill lookup |
| COMPOSITION.md | A | Meta | Workflow understanding |

---

## Detailed Skill Documentation

### planning.md

**Grade:** A-

| Attribute | Value |
|-----------|-------|
| **Category** | Core |
| **File Size** | 9.0 KB |
| **Completeness** | 95% |
| **Professional Standard** | Production-ready |

#### When Called
- User requests planning: "create a plan", "plan this feature"
- Mode start (after sizing/routing)
- After brainstorming when approach is chosen
- Slash command: `/plan`, `/aria:plan`

#### Why Called
To break down work into discrete, verifiable tasks with HITL checkpoints before execution begins. Essential for:
- Establishing verification gates
- Setting user expectations
- Enabling progress tracking
- Defining rollback points

#### How Called
```markdown
Load skill: Read .aria/skills/planning.md
Follow instructions for current mode (LITE/STANDARD/FULL/FULL+)
```

**From CLAUDE.md:**
```
1. Understand requirements (ask clarifying questions)
2. Create a plan (save to `.aria/state/current-plan.json`)
3. Get HITL approval: `[a]pprove / [r]evise / [c]ancel`
```

#### Outputs
- `.aria/state/current-plan.json` - JSON plan with tasks
- HITL checkpoint for approval

#### Grading Breakdown

| Criterion | Score | Notes |
|-----------|-------|-------|
| Clarity | 10/10 | Clear instructions, well-structured |
| Completeness | 9/10 | Missing complex plan examples |
| Mode Variations | 9/10 | All modes documented |
| Error Handling | 8/10 | Replanning mentioned but light |
| Integration | 9/10 | Good handoff to executing |

#### What's Missing
- Example of multi-phase FULL+ plan
- Error recovery when planning fails
- Maximum task count guidance

---

### executing.md

**Grade:** B+

| Attribute | Value |
|-----------|-------|
| **Category** | Core |
| **File Size** | 4.7 KB |
| **Completeness** | 85% |
| **Professional Standard** | Good, needs expansion |

#### When Called
- After plan is approved
- Automatically follows planning
- Never called standalone

#### Why Called
To implement tasks from approved plan with verification after each task. Core execution engine for:
- Code modifications
- Running verification gates
- Git commits
- Progress tracking

#### How Called
```markdown
Load skill: Read .aria/skills/executing.md
For each task in current-plan.json:
  - Announce task
  - Implement
  - Run verify.sh
  - Commit if passed
```

#### Outputs
- Code changes
- Git commits
- Progress updates to `.aria/state/progress.json`
- Design notes (FULL mode)

#### Grading Breakdown

| Criterion | Score | Notes |
|-----------|-------|-------|
| Clarity | 9/10 | Clear execution loop |
| Completeness | 7/10 | Mode variations need detail |
| Mode Variations | 6/10 | LITE/STANDARD/FULL differences unclear |
| Error Handling | 8/10 | Debugging skill invocation mentioned |
| Integration | 9/10 | Good verify.sh integration |

#### What's Missing
- Detailed mode-specific behavior
- Subagent isolation instructions
- Rollback on failure guidance

---

### debugging.md

**Grade:** B

| Attribute | Value |
|-----------|-------|
| **Category** | Core |
| **File Size** | 6.3 KB |
| **Completeness** | 80% |
| **Professional Standard** | Adequate |

#### When Called
- Test failure during execution
- Error encountered
- User reports bug
- Explicit request: "debug this", "fix the failing test"

#### Why Called
To systematically diagnose and fix issues:
- Reproduce the problem
- Isolate the cause
- Hypothesize solutions
- Implement fix
- Verify resolution

#### How Called
```markdown
Load skill: Read .aria/skills/debugging.md
Follow debugging loop:
  1. Reproduce
  2. Isolate
  3. Hypothesize
  4. Fix
  5. Verify
```

**Invoked from executing:**
```
Task 3 fails → debugging invoked → fix applied → retry Task 3
```

#### Outputs
- Fix implementation
- Design notes entry explaining the bug
- (Optional) Test case for regression

#### Grading Breakdown

| Criterion | Score | Notes |
|-----------|-------|-------|
| Clarity | 8/10 | Good structure |
| Completeness | 7/10 | Missing troubleshooting trees |
| Mode Variations | 7/10 | Present but light |
| Error Handling | 6/10 | No guidance for unfixable bugs |
| Integration | 8/10 | Good handoff back to executing |

#### What's Missing
- Decision trees for common error types
- When to escalate to HITL vs keep trying
- Performance debugging guidance

---

### tdd.md

**Grade:** A (EXEMPLARY)

| Attribute | Value |
|-----------|-------|
| **Category** | Core |
| **File Size** | 13 KB |
| **Completeness** | 98% |
| **Professional Standard** | Best in collection |

#### When Called
- User requests: "write tests first", "tdd approach"
- Critical functionality (auth, payments, data integrity)
- Task marked for TDD in plan
- High-confidence requirements

#### Why Called
To ensure code correctness through test-driven development:
- Write failing test (RED)
- Implement to pass (GREEN)
- Clean up (REFACTOR)
- Repeat

#### How Called
```markdown
Load skill: Read .aria/skills/tdd.md
For each requirement:
  1. RED: Write failing test
  2. GREEN: Minimal implementation
  3. REFACTOR: Clean code
  4. Commit when green
```

#### Outputs
- Test files
- Implementation code
- Coverage report (STANDARD+)

#### Grading Breakdown

| Criterion | Score | Notes |
|-----------|-------|-------|
| Clarity | 10/10 | Excellent explanations |
| Completeness | 10/10 | Nothing missing |
| Mode Variations | 10/10 | LITE/STANDARD/FULL all detailed |
| Error Handling | 9/10 | Good guidance |
| Integration | 9/10 | Good handoffs |
| Examples | 10/10 | Multiple languages, patterns |

#### What Makes This Exemplary
- Complete TDD cycle explanation with diagrams
- Code examples in multiple languages
- Anti-patterns documented
- Coverage guidelines per mode
- Test naming conventions
- Integration with other skills

---

### researcher.md

**Grade:** C+ (NEEDS EXPANSION)

| Attribute | Value |
|-----------|-------|
| **Category** | Research |
| **File Size** | 3.6 KB |
| **Completeness** | 70% |
| **Professional Standard** | TOO LIGHT |

#### When Called
- User provides article/paper URL
- Research workflow: "analyze this paper"
- Repository analysis (explicit research intent)

#### Why Called
To extract concepts and insights from research materials:
- Parse document
- Extract key concepts
- Structure findings
- Hand off to brainstorming

#### How Called
```markdown
Load skill: Read .aria/skills/researcher.md
Follow extraction process:
  1. Read/fetch source material
  2. Extract key concepts
  3. Generate concepts.json
  4. Hand off to brainstorming for IDEA.md
```

#### Outputs
- `.aria/docs/research-output.json` (concepts)
- Handoff to brainstorming

#### Grading Breakdown

| Criterion | Score | Notes |
|-----------|-------|-------|
| Clarity | 7/10 | Basic structure |
| Completeness | 5/10 | Too brief for complex task |
| Mode Variations | 6/10 | Minimal |
| Error Handling | 4/10 | No guidance |
| Integration | 7/10 | Mentions brainstorming handoff |
| Examples | 3/10 | None |

#### What's Missing
- Output format examples
- Handling large documents
- Multi-source research
- Different document types (PDF, HTML, repo)
- Error handling for inaccessible sources
- Integration details with slide-generation

---

### slide-generation.md

**Grade:** C (INCOMPLETE)

| Attribute | Value |
|-----------|-------|
| **Category** | Research |
| **File Size** | 4.6 KB |
| **Completeness** | 65% |
| **Professional Standard** | INCOMPLETE |

#### When Called
- HITL checkpoint after IDEA.md: "Generate slides? [y/n]"
- User requests presentation from research

#### Why Called
To create presentation materials from research synthesis:
- Generate FOCUS.md (core ideas)
- Create slide deck (PPTX or PDF)
- Support two methods: NotebookLM or local

#### How Called
```markdown
Load skill: Read .aria/skills/slide-generation.md
1. Generate FOCUS.md from IDEA.md
2. HITL: Choose method (NotebookLM/Local)
3. Generate slides
```

#### Outputs
- `.aria/outputs/FOCUS.md`
- `.aria/outputs/slides-*.pptx` or `.pdf`

#### Grading Breakdown

| Criterion | Score | Notes |
|-----------|-------|-------|
| Clarity | 6/10 | Concept clear, details missing |
| Completeness | 5/10 | References scripts not verified |
| Mode Variations | 5/10 | Minimal |
| Error Handling | 4/10 | None |
| Integration | 6/10 | Vague NotebookLM integration |
| Examples | 4/10 | No sample outputs |

#### What's Missing
- FOCUS.md template
- generate-slides.py documentation
- NotebookLM authentication guide
- Example slide output
- Error handling for missing dependencies

---

### context-refresh.md

**Grade:** C+ (UNDERDEVELOPED)

| Attribute | Value |
|-----------|-------|
| **Category** | Core |
| **File Size** | 4.0 KB |
| **Completeness** | 70% |
| **Professional Standard** | NEEDS SIGNIFICANT EXPANSION |

#### When Called
- Long session (extended work)
- 3+ consecutive failures
- Between phases in FULL/FULL+ mode
- Between epics in FULL+ mode

#### Why Called
To prevent context drift and maintain AI accuracy:
- Summarize current state
- Clear stale context
- Hand off to fresh session
- Preserve critical decisions

#### How Called
```markdown
Load skill: Read .aria/skills/context-refresh.md
1. Create handoff summary
2. Save to progress.json
3. Signal need for refresh
```

**NOT ACTUALLY IMPLEMENTED:** The skill exists but no code triggers it.

#### Outputs
- Handoff summary
- Preserved state in progress.json

#### Grading Breakdown

| Criterion | Score | Notes |
|-----------|-------|-------|
| Clarity | 7/10 | Clear purpose |
| Completeness | 5/10 | Implementation missing |
| Mode Variations | 6/10 | FULL+ variation mentioned |
| Error Handling | 4/10 | None |
| Integration | 5/10 | Not integrated with code |
| Examples | 4/10 | No handoff format example |

#### What's Missing
- Actual implementation in scripts
- Handoff format specification
- Metrics for when to trigger
- State preservation details
- Integration with ralph.sh

---

## Agent Documentation

### Quick Reference (Agents)

| Agent | Location | Grade | Boris Pattern |
|-------|----------|-------|---------------|
| analyzer | `.claude/agents/analyzer.md` | B+ | Pattern 3: Analyzer |
| implementer | `.claude/agents/implementer.md` | B+ | Pattern 3: Implementer |
| verify-app | `.claude/agents/verify-app.md` | B | Pattern 3: Verifier |
| code-simplifier | `.claude/agents/code-simplifier.md` | B | Pattern 3: Simplifier |

All four Boris Cherny Pattern 3 agents are now implemented.

---

### analyzer Agent

**Location:** `.claude/agents/analyzer.md`

| Attribute | Value |
|-----------|-------|
| **Type** | Subagent |
| **Tools** | Read, Glob, Grep (read-only) |
| **Grade** | B+ |
| **Boris Pattern** | Pattern 3: Analyzer |

#### When Called
- Understanding codebase before changes
- Analyzing existing patterns
- Planning modifications
- Code review tasks

#### Why Called
To read and understand code without modifying it:
- Explore codebase structure
- Find existing patterns
- Identify dependencies
- Report findings

#### How Called
```typescript
Task({
  subagent_type: "analyzer",
  prompt: "Analyze how auth is implemented in this codebase"
})
```

#### Assessment
- Read-only tools (safe)
- Clear single responsibility
- Pattern-following emphasis
- Good for codebase exploration

---

### implementer Agent

**Location:** `.claude/agents/implementer.md`

| Attribute | Value |
|-----------|-------|
| **Type** | Subagent |
| **Tools** | Read, Edit, Write, Glob |
| **Grade** | B+ |
| **Boris Pattern** | Pattern 3: Implementer |

#### When Called
- Implementing specific tasks from plan
- Making targeted code changes
- File-level modifications

#### Why Called
To write code following specifications:
- Implement single task
- Follow existing patterns
- Limited scope changes
- No architectural decisions

#### How Called
```typescript
Task({
  subagent_type: "implementer",
  prompt: "Implement retry logic in src/api/client.ts following the pattern in utils/retry.ts"
})
```

#### Assessment
- Focused write access
- Follows analyzer findings
- Single-task implementation
- Pattern compliance emphasis

---

### code-simplifier Agent

**Location:** `.claude/agents/code-simplifier.md`

| Attribute | Value |
|-----------|-------|
| **Type** | Subagent |
| **Tools** | Read, Edit, Glob, Grep |
| **Grade** | B |

#### When Called
- After code changes to simplify/clean
- User requests code simplification
- Refactoring tasks

#### Why Called
To improve code quality without changing functionality:
- Remove duplication
- Simplify logic
- Improve naming
- Clean up formatting

#### How Called
```typescript
// From Task tool
Task({
  subagent_type: "code-simplifier",
  prompt: "Simplify the code in src/utils.ts"
})
```

#### Assessment
- Clear purpose
- Limited tool access (good for safety)
- Missing examples of expected output

---

### verify-app Agent

**Location:** `.claude/agents/verify-app.md`

| Attribute | Value |
|-----------|-------|
| **Type** | Subagent |
| **Tools** | Bash, Read, Glob |
| **Grade** | B |

#### When Called
- After changes to verify application works
- E2E verification tasks
- Before deployment/PR

#### Why Called
To test the application end-to-end:
- Run application
- Verify functionality
- Check for errors
- Report results

#### How Called
```typescript
Task({
  subagent_type: "verify-app",
  prompt: "Test the application works correctly after changes"
})
```

#### Assessment
- Clear purpose
- Appropriate tool access
- Missing detailed verification steps

---

## Skill Composition Patterns

### Build Workflow
```
brainstorming → planning → executing → tracking → report-writer
     ↓              ↓           ↓
  IDEA.md    current-plan.json  commits
```

### Bug Fix Workflow
```
debugging → planning(lite) → executing → verify
    ↓            ↓              ↓
hypothesis  1-3 tasks         fix
```

### Research Workflow
```
researcher → brainstorming → slide-generation → prototyping → report-writer
     ↓            ↓               ↓                 ↓             ↓
concepts.json  IDEA.md      FOCUS.md, slides   prototype     REPORT.md
```

### TDD Build
```
planning → tdd → executing → verify
    ↓       ↓        ↓
 tasks   RED→GREEN  commits
         →REFACTOR
```

---

## Skill Quality Standards

### What Makes a Professional Skill

1. **Clear Triggers** - When is this skill invoked?
2. **Explicit Outputs** - What files/artifacts are created?
3. **Mode Variations** - Different behavior for LITE/STANDARD/FULL/FULL+
4. **Error Handling** - What to do when things fail
5. **Examples** - Concrete samples of expected behavior
6. **Integration Points** - Handoffs to/from other skills
7. **HITL Checkpoints** - When to ask user

### Grading Criteria

| Grade | Description |
|-------|-------------|
| A | Complete, professional, exemplary |
| B | Good, functional, minor gaps |
| C | Adequate, needs expansion |
| D | Incomplete, significant issues |
| F | Non-functional or missing |

---

## Recommendations

### Skills Needing Expansion

1. **researcher.md** (C+ → B+)
   - Add output format examples
   - Document multi-source handling
   - Add error handling

2. **slide-generation.md** (C → B)
   - Add FOCUS.md template
   - Document generate-slides.py
   - Add working examples

3. **context-refresh.md** (C+ → B+)
   - Implement actual triggering
   - Add handoff format spec
   - Add metrics/thresholds

4. **executing.md** (B+ → A-)
   - Add mode-specific details
   - Add subagent isolation instructions
   - Add rollback guidance

### Skills Meeting Standard

- planning.md (A-)
- tdd.md (A)
- brainstorming.md (B+)
- tracking.md (B+)
- COMPOSITION.md (A)
- REGISTRY.md (A-)

---

*This registry should be updated whenever skills are modified or new skills are added.*
