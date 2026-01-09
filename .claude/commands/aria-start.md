# ARIA Start Feature

Initialize and optionally run ARIA for a new feature: $ARGUMENTS

## Instructions

This command sets up ARIA for a new feature development cycle.

### If arguments provided (feature description):

1. Initialize: `./.aria/ralph/ralph.sh init "$ARGUMENTS"`
2. Show the created PRD template
3. Explain that user should edit `.aria/ralph/prd.json` to add user stories
4. Provide example of a good user story structure

### If no arguments:

Ask the user to describe the feature they want to build.

### After initialization:

Remind the user:
- Edit the PRD at `.aria/ralph/prd.json`
- Add user stories with clear acceptance criteria
- Run `/aria ralph run 25` to start the autonomous loop

### PRD Best Practices to Share:
- Each story should be completable in 1-3 iterations
- Include "Tests pass" in acceptance criteria
- Set priorities (1 = highest)
- Be specific about what "done" means
