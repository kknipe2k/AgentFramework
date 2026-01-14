# Debugging Skill

> Systematic approach to diagnosing and fixing failures

---
version: 1.0.0
modes: [LITE, STANDARD, FULL, FULL+]
triggers: [test failure, runtime error, unexpected behavior]
inputs: [error message, failing test, reproduction steps]
outputs: [root cause, fix, verification]
dependencies: []
---

## When to Use

Use this skill when:
- Test fails during execution
- Runtime error occurs
- Behavior doesn't match expectation
- User reports "it's broken"

**Invoked by:** `executing` skill on task failure

---

## Workflow: REPRODUCE → ISOLATE → HYPOTHESIZE → TEST → FIX → VERIFY

### Step 1: Reproduce

**Goal:** Confirm the failure happens consistently.

```
REPRODUCE CHECKLIST:
[ ] Can trigger failure on demand?
[ ] Same error every time?
[ ] Environment-specific? (node version, OS, etc.)
```

**Actions:**
1. Run the failing command/test exactly as reported
2. Note the exact error message
3. Check if failure is consistent or intermittent

**If can't reproduce:**
```
HITL: Cannot reproduce the failure.

Tried:
- [what you tried]

Need more info:
- [specific questions]

[p]rovide more details / [s]kip this issue
```

---

### Step 2: Isolate

**Goal:** Find the smallest reproduction case.

```
ISOLATE CHECKLIST:
[ ] Which file(s) are involved?
[ ] Which function(s)?
[ ] What input triggers it?
[ ] What's the minimal test case?
```

**Actions:**
1. Read the stack trace (if available)
2. Identify the failing line/function
3. Check recent changes (`git diff`, `git log`)
4. Create minimal reproduction if complex

**Output:**
```
ISOLATED TO:
- File: src/utils/parser.ts
- Function: parseConfig()
- Line: 47
- Trigger: Empty string input
```

---

### Step 3: Hypothesize

**Goal:** Form testable theories about the cause.

Generate 2-3 hypotheses, ranked by likelihood:

```
HYPOTHESES:

1. [HIGH] Missing null check on line 47
   - Evidence: Error is "Cannot read property of undefined"
   - Test: Add console.log before line 47

2. [MEDIUM] Race condition in async call
   - Evidence: Intermittent failure
   - Test: Add await or check promise chain

3. [LOW] Dependency version mismatch
   - Evidence: Works locally, fails in CI
   - Test: Compare package-lock.json
```

**Rules:**
- Start with simplest explanation
- Prefer recent changes as cause
- Consider edge cases (null, empty, boundary)

---

### Step 4: Test Hypotheses

**Goal:** Validate or eliminate each hypothesis.

For each hypothesis:
1. Add diagnostic code (console.log, debugger, assertions)
2. Run the failing test
3. Observe behavior
4. Confirm or eliminate hypothesis

```
TESTING HYPOTHESIS 1:

Added: console.log(input) before line 47
Result: input is undefined
Confirmed: Missing input validation

Root cause identified ✓
```

**If all hypotheses eliminated:**
- Generate new hypotheses
- Expand search (check dependencies, environment)
- Ask for help (HITL)

---

### Step 5: Fix

**Goal:** Implement the minimal fix.

**Fix Principles:**
1. **Minimal change** - Fix only what's broken
2. **No side effects** - Don't refactor while fixing
3. **Add guard** - Prevent same bug recurring
4. **Document** - Comment why fix was needed

**Fix Template:**
```typescript
// Before
function parseConfig(input) {
  return input.split(',');  // Crashes on undefined
}

// After
function parseConfig(input) {
  if (!input) return [];  // Guard: handle missing input
  return input.split(',');
}
```

**Log the fix:**
```
FIX APPLIED:
- File: src/utils/parser.ts
- Change: Added null check for input parameter
- Reason: Function was called with undefined during initialization
```

---

### Step 6: Verify

**Goal:** Confirm fix works and doesn't break anything.

```
VERIFY CHECKLIST:
[ ] Original test passes?
[ ] Related tests still pass?
[ ] Full test suite passes?
[ ] Manual verification (if applicable)?
```

**Actions:**
1. Run the specific failing test
2. Run related tests
3. Run full test suite
4. If UI: manual check

**If verification fails:**
- Return to Step 3 (new hypothesis)
- Consider if fix introduced new bug

---

## Mode Variations

### LITE Mode

Abbreviated debugging for simple fixes:

```
LITE DEBUGGING:
1. Read error message
2. Check obvious causes (typo, missing import, null)
3. Fix and verify
4. Done

Skip: Deep hypothesis testing, extensive logging
```

### STANDARD Mode

Full workflow with basic logging:

```
STANDARD DEBUGGING:
- Full 6-step workflow
- Log root cause to design-notes.md
- Single retry then escalate
```

### FULL/FULL+ Mode

Comprehensive debugging with documentation:

```
FULL DEBUGGING:
- Full 6-step workflow
- Document all hypotheses tested
- Log to design-notes.md with learnings
- Add regression test for the bug
- Update project-context.md if systemic issue
```

---

## Common Bug Patterns

Quick checks before deep debugging:

| Symptom | Common Cause | Quick Fix |
|---------|--------------|-----------|
| "undefined is not a function" | Missing import/export | Check imports |
| "Cannot read property of undefined" | Null reference | Add null check |
| "Module not found" | Wrong path, missing dep | Check path, npm install |
| Test timeout | Async not awaited | Add await |
| Works locally, fails CI | Env difference | Check node version, env vars |
| Intermittent failure | Race condition | Check async order |
| Type error | Wrong type passed | Check function signature |

---

## Escalation

If debugging fails after reasonable effort:

```
DEBUGGING ESCALATION

Tried:
- [Hypothesis 1]: [Result]
- [Hypothesis 2]: [Result]
- [Hypothesis 3]: [Result]

Stuck because: [Reason]

Options:
[r]etry with fresh approach
[s]earch for similar issues online
[a]sk user for help
[k]ip this task (mark as blocked)
```

---

## Output

After successful debugging:

1. **Fix applied** and verified
2. **Root cause** documented in design-notes.md
3. **Regression test** added (FULL mode)
4. **Return to executing** to continue

```markdown
## Debugging Session: [Issue]

**Symptom:** [What failed]
**Root Cause:** [Why it failed]
**Fix:** [What was changed]
**Prevention:** [How to avoid in future]
```

---

## Tips

- **Don't guess** - Follow the steps, gather evidence
- **Binary search** - Comment out half the code, narrow down
- **Check recent changes** - `git log -5` is your friend
- **Rubber duck** - Explain the problem out loud
- **Take breaks** - Fresh eyes find bugs faster
- **One change at a time** - Multiple changes obscure cause
