# ARIA Command

Run ARIA system commands: $ARGUMENTS

## Instructions

Execute the ARIA engine command specified in the arguments. If no arguments provided, show ARIA status.

### Available Commands

**Verification:**
- `verify [quick|standard|full]` - Run verification pipeline

**Ralph Loop:**
- `ralph init "description"` - Initialize new feature
- `ralph run [max_iterations]` - Run autonomous loop
- `ralph status` - Show current status

**Model Selection:**
- `model status` - Show usage and budget
- `model stats` - Show learning statistics
- `model budget [amount]` - Get/set budget

**Human-in-the-Loop:**
- `hitl status` - Show pending requests
- `hitl respond "message"` - Respond to request
- `hitl approve` - Approve pending request

**Git Operations:**
- `checkpoint [name]` - Save checkpoint
- `rollback checkpoint <name>` - Rollback to checkpoint
- `pr create` - Create pull request

### Execution

Run the command via `./.aria/aria-engine.sh` and report results clearly.

If the command fails, explain what went wrong and suggest fixes.

If no arguments: run `./.aria/ralph/ralph.sh status` to show current state.
