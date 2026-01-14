# TDD Skill

> Write tests first, then make them pass - systematic test-driven development

---
version: 1.0.0
modes: [LITE, STANDARD, FULL, FULL+]
triggers: ["write tests first", "tdd", "test-driven", "red-green-refactor"]
inputs: [requirements, feature specification, acceptance criteria]
outputs: [test files, implementation, coverage report]
dependencies: [planning]
---

## When to Use

Use this skill when:
- Building new functionality that needs high confidence
- Requirements are clear enough to express as tests
- Working on critical paths (auth, payments, data integrity)
- User explicitly requests TDD approach
- Modifying code with poor test coverage

**Skip when:**
- Exploratory coding / prototyping
- One-off scripts that won't be maintained
- UI layout work (hard to test meaningfully)
- Requirements are too vague to write tests

---

## The TDD Cycle: RED → GREEN → REFACTOR

```
    ┌─────────────────────────────────────────────────────────────────┐
    │                                                                 │
    │    ┌─────────┐         ┌─────────┐         ┌──────────┐        │
    │    │  RED    │ ──────▶ │  GREEN  │ ──────▶ │ REFACTOR │        │
    │    │         │         │         │         │          │        │
    │    │ Write   │         │ Make    │         │ Clean    │        │
    │    │ failing │         │ test    │         │ up code  │        │
    │    │ test    │         │ pass    │         │          │        │
    │    └─────────┘         └─────────┘         └──────────┘        │
    │         ▲                                        │              │
    │         │                                        │              │
    │         └────────────────────────────────────────┘              │
    │                    Next requirement                              │
    └─────────────────────────────────────────────────────────────────┘
```

**The Three Laws of TDD:**
1. Write NO production code except to pass a failing test
2. Write only ENOUGH test to demonstrate failure
3. Write only ENOUGH production code to pass the test

---

## Workflow

### Step 1: RED - Write a Failing Test

**Goal:** Express ONE requirement as a test that fails.

**Actions:**
1. Pick the smallest testable requirement
2. Write a test that expresses expected behavior
3. Run the test - it MUST fail
4. Confirm it fails for the RIGHT reason

```
RED CHECKLIST:
[ ] Test name describes the behavior, not implementation?
[ ] Test is specific and focused (tests ONE thing)?
[ ] Test runs and fails?
[ ] Failure message is clear and helpful?
[ ] Test fails for the right reason (not syntax error)?
```

**Test Naming Convention:**
```
Good: "returns empty array when input is null"
Good: "throws error when user not authenticated"
Bad:  "test1"
Bad:  "should work correctly"
```

**Example - RED:**
```typescript
// test/calculator.test.ts
describe('Calculator', () => {
  it('adds two positive numbers', () => {
    const calc = new Calculator();
    expect(calc.add(2, 3)).toBe(5);
  });
});

// RUN: npm test
// RESULT: ReferenceError: Calculator is not defined ✓ (fails correctly)
```

---

### Step 2: GREEN - Make the Test Pass

**Goal:** Write the MINIMUM code to make the test pass.

**Actions:**
1. Write the simplest code that passes the test
2. Don't over-engineer - "fake it till you make it"
3. Run the test - it MUST pass
4. Commit when green

```
GREEN CHECKLIST:
[ ] Implementation is minimal (no extra features)?
[ ] Test passes?
[ ] No other tests broken?
[ ] Code committed?
```

**Green Rules:**
- **Do:** Return a hardcoded value if it passes the test
- **Do:** Write "obvious" implementation if clear
- **Don't:** Add error handling not tested yet
- **Don't:** Optimize prematurely
- **Don't:** Add features not covered by tests

**Example - GREEN:**
```typescript
// src/calculator.ts
export class Calculator {
  add(a: number, b: number): number {
    return a + b;  // Simplest thing that works
  }
}

// RUN: npm test
// RESULT: PASS ✓
```

---

### Step 3: REFACTOR - Clean Up

**Goal:** Improve code quality WITHOUT changing behavior.

**Actions:**
1. Look for code smells (duplication, unclear names, long methods)
2. Make small, safe improvements
3. Run tests after EACH change - must stay green
4. Commit when satisfied

```
REFACTOR CHECKLIST:
[ ] Any duplication to remove?
[ ] Names clear and intention-revealing?
[ ] Functions small and focused?
[ ] Tests still passing after each change?
```

**Safe Refactorings:**
- Rename variables/functions for clarity
- Extract helper methods
- Remove duplication (DRY)
- Simplify conditionals
- Improve formatting

**Unsafe (Don't Do in Refactor):**
- Add new functionality
- Change public interfaces
- "While I'm here..." changes

**Example - REFACTOR:**
```typescript
// Before refactoring
export class Calculator {
  add(a: number, b: number): number {
    return a + b;
  }
}

// After refactoring (if needed)
// In this case, code is already clean - no refactor needed
// Don't refactor for the sake of refactoring!
```

---

### Step 4: Repeat

**Goal:** Continue the cycle for next requirement.

```
CYCLE PROGRESS:
[x] RED:      "adds two positive numbers" - PASS
[ ] RED:      "adds negative numbers"
[ ] RED:      "adds zero"
[ ] RED:      "throws on non-numeric input"
```

**Pick next test by:**
1. Simplest case first
2. Happy path before edge cases
3. Edge cases before error cases
4. Build complexity gradually

---

## Mode Variations

### LITE Mode

Quick TDD for simple features:

```
LITE TDD:
1. Write 1-3 core tests (happy path)
2. Implement to pass
3. Quick cleanup (optional)
4. Done

Skip:
- Exhaustive edge case testing
- Comprehensive refactoring
- Coverage reports
```

**When:** Bug fixes, small utilities, time pressure

---

### STANDARD Mode

Full TDD cycle with reasonable coverage:

```
STANDARD TDD:
1. Write tests covering:
   - Happy path (required)
   - Key edge cases (required)
   - Error cases (as needed)
2. Full RED → GREEN → REFACTOR cycle
3. Coverage check (aim for >70% on new code)
4. Document test decisions in design-notes.md
```

**When:** New features, API endpoints, business logic

---

### FULL/FULL+ Mode

Comprehensive TDD with documentation:

```
FULL TDD:
1. Write test plan before coding:
   - List all cases to test
   - Identify boundary conditions
   - Document testing strategy
2. Full RED → GREEN → REFACTOR for each
3. Coverage requirement: >80% on new code
4. Write tests at multiple levels:
   - Unit tests
   - Integration tests
   - Contract tests (if APIs)
5. Document all test decisions
6. Add test documentation for complex cases
```

**When:** Critical systems, payment flows, auth logic

---

## Test Writing Guidelines

### Test Structure: Arrange-Act-Assert (AAA)

```typescript
it('calculates total with tax', () => {
  // Arrange - set up test data
  const cart = new ShoppingCart();
  cart.addItem({ price: 100 });

  // Act - perform the action
  const total = cart.calculateTotal(taxRate: 0.1);

  // Assert - verify the result
  expect(total).toBe(110);
});
```

### What Makes a Good Test

| Good Test | Bad Test |
|-----------|----------|
| Tests ONE behavior | Tests multiple things |
| Descriptive name | Vague name like "test1" |
| Independent (no order dependency) | Depends on other tests |
| Fast (<100ms per test) | Slow (network, DB) |
| Deterministic (same result always) | Flaky (sometimes fails) |
| Tests behavior, not implementation | Tests internal details |

### Test Naming Patterns

```
"returns X when Y"           - for return values
"throws X when Y"            - for error cases
"calls X when Y"             - for side effects
"does not X when Y"          - for negative cases
"X given Y"                  - BDD style
```

---

## HITL Checkpoints

Before these actions, stop and confirm:

- [ ] Deleting existing tests
- [ ] Skipping tests (marking as .skip)
- [ ] Changing test assertions (weakening tests)
- [ ] Testing private/internal methods directly

**Format:**
```
HITL CHECKPOINT: About to [action]
Reason: [why this might be needed]
Proceed? [y]es / [n]o / [e]xplain
```

---

## Coverage Guidelines

| Mode | Target | Requirement |
|------|--------|-------------|
| LITE | No target | Skip coverage |
| STANDARD | 70%+ | New code only |
| FULL | 80%+ | New code required |
| FULL+ | 90%+ | Plus integration |

**Coverage Commands:**
```bash
# JavaScript/TypeScript
npx jest --coverage
npx c8 npm test

# Python
pytest --cov=src --cov-report=term

# Go
go test -cover ./...
```

**Coverage Caveats:**
- High coverage ≠ good tests
- 100% is usually not worth the effort
- Focus on critical paths first
- Don't test getters/setters for coverage

---

## Common Testing Patterns

### Testing Async Code

```typescript
// Async/await
it('fetches user data', async () => {
  const user = await fetchUser(1);
  expect(user.name).toBe('Alice');
});

// Promises
it('fetches user data', () => {
  return fetchUser(1).then(user => {
    expect(user.name).toBe('Alice');
  });
});
```

### Testing Errors

```typescript
it('throws on invalid input', () => {
  expect(() => divide(1, 0)).toThrow('Division by zero');
});

// Async errors
it('rejects on network failure', async () => {
  await expect(fetchUser(-1)).rejects.toThrow('Not found');
});
```

### Testing with Mocks

```typescript
it('sends notification on signup', () => {
  const mockNotify = jest.fn();
  const service = new SignupService(mockNotify);

  service.signup({ email: 'test@test.com' });

  expect(mockNotify).toHaveBeenCalledWith('test@test.com');
});
```

---

## Integration with Execution

TDD integrates with `executing` skill:

```
For each task in plan:
1. If task creates new functionality:
   - Enter TDD mode
   - RED: Write failing test
   - GREEN: Implement
   - REFACTOR: Clean up
   - Run verify.sh
2. If task modifies existing:
   - Run existing tests first (baseline)
   - Add test for new behavior if needed
   - Implement change
   - Verify all tests pass
```

---

## Handoff

**From planning:**
- Requirements broken into testable units
- Acceptance criteria (become test cases)

**To debugging (on failure):**
- Failing test as reproduction
- Expected vs actual behavior

**To executing:**
- Tested, working implementation
- Test suite as safety net

---

## Output

After TDD session:

```markdown
## TDD Session: [Feature]

**Tests Written:**
- [test 1 - description]
- [test 2 - description]
- [test 3 - description]

**Coverage:** X% (new code)

**Implementation Notes:**
- [key decision 1]
- [key decision 2]

**Remaining Test Cases:**
- [future test 1 - if deferred]
```

---

## Anti-Patterns to Avoid

| Anti-Pattern | Problem | Instead |
|--------------|---------|---------|
| Test after code | Fits tests to implementation | Write test first |
| Testing implementation | Brittle, breaks on refactor | Test behavior |
| Giant tests | Hard to debug failures | One assertion per test |
| Copy-paste tests | Maintenance nightmare | Use parameterized tests |
| Ignoring failing tests | Rot sets in | Fix or delete immediately |
| 100% coverage obsession | Diminishing returns | Focus on critical paths |
| Mocking everything | Tests prove nothing | Mock at boundaries |

---

## Tips

- **Start simple** - First test should be the easiest case
- **Trust the cycle** - Don't skip steps, even when tempted
- **One assertion per test** - Makes failures clear
- **Test behavior, not implementation** - Tests survive refactoring
- **Name tests as documentation** - Future you will thank you
- **Keep tests fast** - Slow tests don't get run
- **Delete tests that don't add value** - Dead tests are worse than none
- **When stuck, write a simpler test** - Back up and try again
- **Refactor tests too** - Test code is real code

---

*See [REGISTRY.md](./REGISTRY.md) for skill index*
