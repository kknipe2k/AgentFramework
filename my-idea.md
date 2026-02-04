# How AI Coding Agents Actually Work

> Under the hood of Claude Code, Codex, and other autonomous programming tools

## Summary

AI coding agents from OpenAI, Anthropic, and Google can now work on software projects for hours, writing apps, running tests, and fixing bugs. But they're not magic—they're sophisticated orchestration systems built on LLMs with specific architectural patterns, context management tricks, and known limitations. Understanding how they work helps developers know when to use them and avoid common pitfalls.

## Key Concepts

- **Large Language Model (LLM)**: The core technology—a neural network trained on vast text and code. It's a pattern-matching machine that extracts compressed statistical representations from training data. Can interpolate across domains (useful inferences) or confabulate (errors).

- **Context Window**: The LLM's "short-term memory" limiting how much data it can process before forgetting. Every response adds to one giant prompt including conversation history, code, and reasoning tokens. Processing cost increases quadratically with size.

- **Context Rot**: As tokens in the context window increase, the model's ability to accurately recall information decreases. Every new token depletes the "attention budget."

- **Context Compression (Compaction)**: When nearing context limits, agents summarize history—preserving architectural decisions and unresolved bugs while discarding redundant outputs. The agent periodically "forgets" but can re-orient by reading code, notes, and change logs.

- **Orchestrator-Worker Pattern**: A lead agent coordinates the process, spawning specialized subagents that work in parallel. Subagents act as intelligent filters, returning only relevant information rather than full context.

- **CLAUDE.md / AGENTS.md**: External documentation files that guide agent actions between context refreshes. They document commands, core files, code style, and testing instructions—acting as persistent memory.

## How It Works

### The Basic Loop

Anthropic describes the core pattern as: **"gather context, take action, verify work, repeat."**

1. Supervising LLM interprets tasks from human user
2. Assigns subtasks to parallel worker LLMs with tool access
3. Workers can write files, run commands, fetch websites, download software
4. Supervisor interrupts and evaluates results
5. Cycle continues until task complete

### Tool Use to Save Tokens

Rather than feeding large files through the LLM (expensive, inaccurate), agents write code to outsource work:
- Python scripts to extract data from images
- Targeted database queries instead of loading full datasets
- Bash commands like `head` and `tail` to analyze large files

### Multi-Agent Economics

| Interaction Type | Token Usage |
|-----------------|-------------|
| Chatbot conversation | 1x (baseline) |
| Single agent | ~4x |
| Multi-agent system | ~15x |

This only makes economic sense for high-value tasks.

## Why It Matters

### The Promise
- Hours of autonomous work on real codebases
- Parallel exploration of solutions
- Sandboxed execution environments
- Persistent memory through documentation files

### The Pitfalls
- Context rot degrades accuracy over time
- "Vibe coding" (not understanding the output) creates technical debt
- METR study: Experienced devs took 19% longer with AI tools
- Security risks from unreviewed code
- No accountability—AI has no actual agency

## Best Practices

The article and Anthropic documentation recommend:

1. **Plan before coding**: Ask agent to read files and make a plan first. Without this, LLMs jump to quick solutions that break later.

2. **Use CLAUDE.md files**: Document bash commands, core files, code style, testing instructions. Acts as external memory.

3. **Incremental development**: One feature at a time, test before moving on.

4. **Human oversight**: You bear responsibility for proving code works. Shipping unreviewed AI code to production is risky.

5. **Know your architecture**: Guide the LLM toward modular, expandable designs.

## Synthesis Matrix

| Component | Problem Solved | Mechanism | Limitation |
|-----------|---------------|-----------|------------|
| LLM Core | Code generation | Pattern matching on training data | Confabulation errors |
| Context Window | Memory for conversation | Giant prompt with full history | Quadratic cost, rot |
| Compaction | Context limits | Summarize, preserve key details | Loses information |
| Multi-Agent | Complex tasks | Orchestrator + parallel workers | 15x token cost |
| CLAUDE.md | Context refresh | External documentation | Manual maintenance |
| Tool Use | Large data handling | Write scripts, run commands | Security risks |

## Key Takeaways

1. AI coding agents are orchestration wrappers around multiple LLMs, not magic
2. Context is finite and degrades—every token depletes "attention budget"
3. Agents periodically forget but re-orient via code and documentation
4. Multi-agent systems burn tokens 15x faster than chat
5. Human planning and oversight remain essential—"vibe coding" is dangerous
6. Experienced developers may actually be slower with AI tools on familiar codebases
7. Best use case today: proof-of-concept demos and internal tools

## Sources

- Ars Technica: "How AI coding agents work under the hood"
- Anthropic engineering documentation
- METR randomized controlled trial (July 2025)
