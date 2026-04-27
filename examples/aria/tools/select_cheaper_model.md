---
name: select_cheaper_model
version: 1.0.0
description: Budget downshift hook — given current model and remaining budget, return the model the runtime should switch to.

input_schema:
  type: object
  properties:
    current_model:    { type: string,  description: "Current model identifier (e.g., claude-opus-4-7)" }
    spent_usd:        { type: number }
    cap_usd:          { type: number }
    avg_task_cost:    { type: number,  description: "Mean task cost so far (USD)" }
    tasks_remaining:  { type: number }
  required: ["current_model", "spent_usd", "cap_usd"]

output_schema:
  type: object
  properties:
    selected_model:  { type: string }
    rationale:       { type: string }
    confidence:      { type: number }

mcp_binding: null

shell_binding: null

inline_implementation:
  type: declarative_decision_table
  rules:
    - if: { "==": [{ "var": "current_model" }, "claude-opus-4-7"] }
      then: { selected_model: "claude-sonnet-4-6", rationale: "Opus -> Sonnet on first downshift trigger.", confidence: 0.95 }
    - if:
        and:
          - { "==": [{ "var": "current_model" }, "claude-sonnet-4-6"] }
          - or:
              - { "<": [{ "-": [{ "var": "cap_usd" }, { "var": "spent_usd" }] }, { "*": [{ "var": "cap_usd" }, 0.10] }] }
              - { ">": [{ "var": "avg_task_cost" }, { "/": [{ "-": [{ "var": "cap_usd" }, { "var": "spent_usd" }] }, 3] }] }
      then: { selected_model: "claude-haiku-4-5", rationale: "Sonnet -> Haiku: <10% budget remaining or per-task cost > 1/3 of remainder.", confidence: 0.85 }
    - if: { "==": [{ "var": "current_model" }, "claude-haiku-4-5"] }
      then: { selected_model: "claude-haiku-4-5", rationale: "Already at cheapest tier; no further downshift available. Runtime should escalate to HITL.", confidence: 1.0 }
    - default: { selected_model: "${input.current_model}", rationale: "No downshift rule matched.", confidence: 0.5 }

capabilities:
  tools_called:    []
  skills_loaded:   []
  file_access:     { read: [], write: [] }
  network:         []
  shell:           false
  spawn_agents:    []

provenance:
  generator:    "hand-authored"
  source:       "ports model-selector.sh budget-aware tier policy"
  authored_at:  "2026-04-18T00:00:00Z"
  content_hash: "sha256:placeholder-replace-on-first-load"

tags: ["budget", "model", "downshift", "declarative"]
---

## Description

Pure-decision tool with zero capabilities. Used by the runtime when `budget.downshift_at_percent` triggers. Given the current model and budget state, returns the next-cheaper model to use.

This tool ports ARIA's `model-selector.sh` budget-aware tier policy into the runtime as a declarative decision table — no executable code, no shell, no I/O. Pure JSONLogic evaluation.

Because all `capabilities.*` fields are empty, this tool installs under any tier (Novice/Promoted/Operator) without warnings. It is the safest possible tool: pure data transformation.

## Implementation

The runtime evaluates the `inline_implementation.rules` array as a JSONLogic decision table:

1. Match each rule's `if` condition against the input.
2. First match wins; emit `then` as output.
3. If no rule matches, emit `default`.

This pattern (declarative inline implementation) is available for any tool whose logic is a pure function of input. It's preferred over shell or MCP bindings whenever feasible — no execution, no security surface.

## Examples

```yaml
input:
  current_model:   "claude-opus-4-7"
  spent_usd:       3.75
  cap_usd:         5.00
  avg_task_cost:   0.15
  tasks_remaining: 4
output:
  selected_model: "claude-sonnet-4-6"
  rationale:      "Opus -> Sonnet on first downshift trigger."
  confidence:     0.95
```

```yaml
input:
  current_model:   "claude-sonnet-4-6"
  spent_usd:       4.60
  cap_usd:         5.00
  avg_task_cost:   0.20
  tasks_remaining: 2
output:
  selected_model: "claude-haiku-4-5"
  rationale:      "Sonnet -> Haiku: <10% budget remaining or per-task cost > 1/3 of remainder."
  confidence:     0.85
```

## Replacing this tool

Frameworks that want learning-based selection (e.g., a port of ARIA's offline-RL `model-selector.sh`) replace this tool with one that calls out to an external Python process via shell binding, or to an MCP server hosting the learned policy. The runtime's `budget.downshift_hook` field references whichever tool is configured.

## Why declarative

A learning-based selector ports cleanly later. Starting declarative:
- Removes execution risk during early adoption.
- Makes the behavior auditable (the rules are visible in the tool definition).
- Gives a baseline against which the learning version can be A/B tested.
