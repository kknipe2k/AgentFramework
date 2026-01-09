# ARIA Verify

Run ARIA verification pipeline: $ARGUMENTS

## Instructions

Run the ARIA verification executor at the specified level.

### Levels
- `quick` - Types + Lint only (fastest, <30s)
- `standard` - Quick + Tests + Build (default, 1-5min)
- `full` - Standard + Integration + E2E (thorough, 5-15min)

### Execution

1. Run: `./.aria/verify-executor.sh ${ARGUMENTS:-standard}`
2. Report results clearly with pass/fail status
3. If failures occur, identify specific issues and suggest fixes
4. Show summary: X passed, Y failed

If verification passes, confirm the code is ready for the next step.
If verification fails, list what needs to be fixed before proceeding.
