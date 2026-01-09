# Create Implementation Plan

Create a detailed plan before implementing. Don't write code yet.

## Pre-computed context

```bash
echo "=== Project Structure ==="
find . -type f -name "*.js" -o -name "*.ts" -o -name "*.py" 2>/dev/null | grep -v node_modules | head -20

echo ""
echo "=== Package Dependencies ==="
cat package.json 2>/dev/null | grep -A 50 '"dependencies"' | head -30
```

## Instructions

Based on the user's request, create a detailed plan with:

1. **Goal Summary**
   - What we're building in one sentence

2. **Requirements**
   - Must have (required features)
   - Must not (things to avoid)
   - Nice to have (if time permits)

3. **Files to Create/Modify**
   - List each file with what changes
   - New files with purpose
   - Existing files with what to add/change

4. **Implementation Steps**
   - Numbered steps in order
   - Each step should be small and testable
   - Include test steps

5. **Verification Plan**
   - How to test each step
   - How to verify the final result

6. **Risks/Considerations**
   - What could go wrong
   - Dependencies to be aware of

## Output Format

```markdown
# Plan: [Feature Name]

## Goal
[One sentence]

## Requirements
### Must Have
- [ ] Requirement 1
- [ ] Requirement 2

### Must Not
- [ ] Anti-requirement 1

## Files
| File | Action | Purpose |
|------|--------|---------|
| src/x.js | Create | ... |
| src/y.js | Modify | ... |

## Steps
1. [ ] Step 1
   - Test: ...
2. [ ] Step 2
   - Test: ...

## Verification
- [ ] All tests pass
- [ ] Feature works in browser
- [ ] No regressions

## Risks
- Risk 1: mitigation
```

**Do not implement yet.** Just create the plan and ask user to approve.
