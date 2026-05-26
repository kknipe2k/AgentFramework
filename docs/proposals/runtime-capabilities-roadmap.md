# Runtime Capabilities Roadmap

> **Status: forward-looking scope clarification.** Captures architectural answers to three recurring questions about what the runtime can and will support across major version horizons: agent swarms, data integration, and non-Anthropic AI model invocation. Drafted 2026-05-26 after the M08.6 / M9 mentor-pattern scoping discussion to give the M9 phase doc author + forward planning sessions a stable reference.
>
> Not a commitment to ship; not a proposal to debate. A current-best answer to "could this runtime do X?" for the three dimensions most often asked about. Where the answer is "yes, in version N+1", it's an architecturally-feasible target, not a roadmap promise.

---

## 1. Agent swarms

The runtime is designed multi-agent-first. ARIA already proves the pattern (8 agents wired by `spawns` / `allowed_*` relationships; capability narrowing on Agent→Agent edges). What "swarm" means determines what's already supported vs what needs new architecture.

### 1.1 Hierarchical multi-agent — supported (M9+)

Parent spawns children, children return results to parent. The framework `agents[]` array + spawn relationships + capability-narrowing-on-spawn primitives are the load-bearing pieces. M9 Agent Composer generates these.

ARIA = 8 agents, hierarchical pattern, works today through M8's loader (M08.6 closes load + render + persist).

### 1.2 Concurrent multi-agent — v1.0+ (architecture-ready, scope-locked at v0.1)

Multiple agents running in parallel. v0.1 is single-session, single-active-agent per §0d scope lock. Lifting that lock is M11+.

What's already there: capability sandboxing supports per-agent process isolation (each agent in its own seccomp/landlock/Job Objects boundary).

What's missing: shared workspace coordination. Filesystem races, git races, shared state contention. Multi-agent concurrent access to the same project directory needs explicit coordination primitives (file locks, queue-based serialization of mutating operations, conflict detection on git operations). Achievable additive work, not a re-architecture.

### 1.3 Peer-to-peer swarm — v1.5+ (needs new IPC topology)

Agents message each other directly, not just parent-child. Current agent-to-agent comms is one-shot spawn: parent calls child with input, child returns output, no ongoing channel.

True peer-to-peer needs broadcast, subscription, or pub/sub primitives. The drone IPC layer + capability matrix would need extension (per-edge permission rules for which agents can message which agents on which topics). Not trivial — affects the spec's whole agent-relationship model — but architecturally compatible with the runtime's safety-first design.

### 1.4 Cross-runtime swarms — v3.0+ (out of scope until inter-runtime trust model exists)

Your agent stack talks to your colleague's agent stack. Requires inter-runtime trust model, identity, message routing across machine boundaries, cross-org capability federation. Big architecture, not roadmap-relevant pre-v2.

### 1.5 Why this runtime is well-positioned for swarms

The capability sandbox + plan state machine + HITL discipline scale BETTER for swarms than most agent frameworks (LangGraph, CrewAI, AutoGen). Those bolt safety on AFTER the swarm primitives; this runtime was designed safety-first, with swarms as the natural composition pattern. The harder swarm problems (capability containment, audit trail integrity, plan reconciliation when one agent diverges) are solved at architecture level, not patched per-feature.

---

## 2. Data integration

The runtime ships zero built-in data layer beyond its own operational SQLite (sessions, audit, snapshots, MCP server registry). Data access is deliberately delegated to user-owned resources.

### 2.1 What's delegated to MCP / tools / hooks

- **MCP servers** for most integrations. There are community + vendor MCP servers for Postgres, Notion, Google Drive, Slack, Linear, GitHub, vector DBs (Pinecone, Weaviate, Chroma), graph DBs (Neo4j), and growing. The user points the runtime at the MCP server's stdio/HTTP endpoint; runtime calls it via the MCP protocol.
- **RAG** is typically a vector-DB MCP server the agent queries (the embedding + indexing infrastructure lives in the user's MCP-wrapped vector store, not in the runtime).
- **Tools** for ad-hoc integrations — `inline_implementation` (declarative decision table) or `mcp_binding` (wraps an MCP call). User authors when no MCP exists for the integration they need.
- **Hooks** at SessionStart / UserPromptSubmit / PreToolUse / etc. — fetch context from external resources before the agent runs. M9+ has hooks as a first-class ingredient class.

### 2.2 What the runtime deliberately does NOT ship

- Built-in vector DB
- Built-in graph DB
- Built-in RAG pipelines (chunker / embedder / retriever)
- Built-in knowledge-graph extraction
- Built-in document indexing

### 2.3 Why this scope lock

Three reasons:

- **Tight runtime.** Adding a data layer doubles the scope of what we own + maintain. Vector DBs alone are a research area with quarterly state-of-the-art shifts; embedding model choices change yearly; chunking strategies are domain-dependent. Owning any of these means perpetual catch-up work that distracts from the orchestration + safety mission.
- **Real data sovereignty.** If the runtime owned the data layer, user data would traverse our SQLite + our embedding pipeline + our retrieval. "Your data doesn't leave your machine" stops being true. By delegating, the user keeps their data inside whatever vector/graph/document store they already trust (or run locally).
- **Avoid losing competition we can't win.** Pinecone, Weaviate, pgvector, Neo4j, Postgres + RAG MCPs are mature. We can't ship a better vector store than Pinecone or a better graph store than Neo4j. Trying would be a strategic mistake. The orchestration + safety layer is where we can win.

### 2.4 Realistic future additions

- **Lightweight built-in vector index via SQLite (sqlite-vec).** "Quick-start RAG" for novice users who want to demo a knowledge-bound agent without standing up a vector DB. Small additive scope; real value for first-time UX. Conditional on user research showing MCP friction is blocking adoption. Probably v1.0 or v1.5.
- **Curated starter MCP examples** for the common data integrations (a sample "RAG MCP server" the mentor can scaffold for novice users; same for "structured data lookup MCP" against Postgres). Ships as kit content, not runtime code.

Full RAG pipeline (chunker + embedder + retriever as built-in features) stays MCP-delegated by design.

---

## 3. Invoking non-Anthropic AI / ML models

Four layers, four answers.

### 3.1 Other LLMs (OpenAI, Gemini, Llama, Mistral, etc.) — v1.0+ (architecture-ready)

The runtime's `LLMProvider` trait is pluggable (M2 architecture). v0.1 is Anthropic-only per §0d scope lock — deliberate, not architectural.

Adding providers is config + impl per provider. Agent frameworks could route per-task: Opus for deep reasoning, GPT-4o for code generation, local Llama for fast routes / privacy-sensitive operations, Gemini for vision, etc. Per-task model routing is already in the framework schema (M9 Agent Composer generates this); it's just gated to Anthropic models in v0.1.

v1.0+ scope expansion: add `OpenAIProvider`, `GeminiProvider`, `OllamaProvider` (or similar local-model provider) impls; mentor's model-routing config gets multi-vendor options; cost model accounts for per-vendor pricing differences.

### 3.2 Non-LLM ML (vision models, classical ML, custom-trained models) — v0.1 via tools

Already supported. An agent calls a tool that wraps any model behind any inference interface:

- Python service the user runs locally (PyTorch / TensorFlow / sklearn behind FastAPI)
- Cloud ML endpoint (SageMaker, Vertex AI, Replicate, Hugging Face Inference, Modal)
- Custom local inference server (vLLM, TGI, llama.cpp, ollama)
- Local executable (sandbox runs binaries with declared capabilities; e.g., a CLI ML tool)

The agent doesn't care whether the tool wraps an LLM, a CNN, a regression model, or a rule engine. Input → output. The capability sandbox scopes what the tool can access (filesystem, network range, env vars) so an ML tool can't exfiltrate data outside its declared boundaries.

### 3.3 Agent invokes another agent system (Claude Code, OpenAI Assistants, Anthropic Computer Use) — v0.1 via MCP or tool

Recursive agency. Agent A calls a tool that IS itself an agent system. Examples:

- Tool wraps Claude Code's headless mode (`claude -p`) — Agent A delegates a sub-task to a Claude Code session
- Tool wraps OpenAI Assistants API — Agent A invokes an OpenAI assistant for specialized tasks
- MCP server wraps Anthropic's Computer Use API — Agent A can drive a desktop session for tasks the runtime's own tools can't handle

The runtime's capability scoping is what makes this safe: Agent A doesn't get Agent B's transitive permissions unless explicitly granted. If Claude Code can write files but Agent A's invocation of it shouldn't write files, the tool's declared capabilities narrow Claude Code's effective permissions for that call.

### 3.4 Self-modifying agents (RL on eval scores, agents that improve themselves mid-run) — NO, by deliberate design

The spec is explicit: "The runtime executes what exists; it doesn't modify itself mid-run."

Improving an agent means re-generating its body, re-running evals, re-shipping. Offline-only improvement loop. The reasons are not technical — they're safety and debuggability:

- **Audit trail integrity.** If an agent mid-run modifies its own prompt or capability rules, the audit trail of "what executed and why" becomes meaningless. You can't retroactively reproduce a session if the agent's definition changed mid-session.
- **Capability scoping becomes dynamic.** Static capability narrowing is what makes the runtime's safety story tractable. If capabilities can be granted at runtime by the agent itself, the entire safety model collapses (the agent could grant itself any capability that's expressible).
- **Debugging becomes a nightmare.** "Why did agent X do Y?" requires "X's definition at time T was Z." If Z changes mid-run, root-cause analysis becomes archaeology.

A future "agent self-tunes via offline RLHF on eval scores between runs" pattern is architecturally conceivable as a v2.0+ ADR, but it's a major addition with serious safety implications and probably requires its own audit/governance story. Not on the v0.1-v1.5 roadmap.

---

## 4. Summary matrix

| Dimension | v0.1 (MVP) | v1.0+ | v1.5+ | v2.0+ |
|---|---|---|---|---|
| Hierarchical multi-agent | ✓ (M9+) | ✓ | ✓ | ✓ |
| Concurrent multi-agent | — (single-session lock) | ✓ (M11+) | ✓ | ✓ |
| Peer-to-peer swarm | — | — | ✓ (new IPC) | ✓ |
| Cross-runtime swarm | — | — | — | maybe v3.0+ |
| Built-in vector index (SQLite-vec) | — | — / ✓ | ✓ | ✓ |
| Built-in graph DB | — | — | — | unlikely |
| Built-in RAG pipeline | — | — | — | unlikely (stays MCP) |
| MCP / tool / hook data integration | ✓ | ✓ | ✓ | ✓ |
| Anthropic LLM provider | ✓ | ✓ | ✓ | ✓ |
| OpenAI / Gemini / Llama LLM providers | — (scope lock) | ✓ | ✓ | ✓ |
| Non-LLM ML via tool wrappers | ✓ | ✓ | ✓ | ✓ |
| Agent-invokes-other-agent-system | ✓ (via MCP/tool) | ✓ | ✓ | ✓ |
| Self-modifying agents | ✗ (by design) | ✗ | ✗ | maybe v3.0+ ADR |

---

## 5. Strategic framing

The runtime's positioning is **the orchestration + safety layer**, not the data layer, not the model layer, not the inference stack.

What stays user-owned (sovereign):
- Data (via MCP / tools / user's databases)
- Models (LLM via configurable providers; non-LLM via tool wrappers to user's infra)
- Inference infrastructure (user's GPUs, user's cloud accounts, user's local servers)

What lives in the runtime (disciplined):
- Orchestration (the plan state machine, the spawn graph, the handoff protocol)
- Safety (capability narrowing, OS-level sandboxing, HITL gates)
- Observability (the live event graph, audit trail, snapshots, plan reconciliation)
- Build-time mentorship (M9+ — the runtime-creation framework that helps users build runtimes correctly)

This split is a strategic call. It makes the runtime a partner to the broader AI ecosystem rather than a competitor to mature vendors (vector DBs, graph DBs, alternate LLM providers, ML inference platforms). It also makes the user's data + model + infrastructure choices durable — the runtime adapts to what the user already runs, rather than asking them to migrate.

The bet is that the orchestration + safety + mentorship layer is where teams will pay for quality, while the data + model + inference layers are where users will choose the best-of-breed components they already trust.

---

## 6. When this doc gets updated

- When a v1.0+ provider impl lands (update §3.1 from architecture-ready to shipped)
- When a multi-session architecture ADR lands (update §1.2 from architecture-ready to shipped)
- When user research clarifies the built-in-vector-index demand (update §2.4)
- When a peer-to-peer swarm ADR is drafted (update §1.3)
- When a self-modifying-agents ADR is proposed (update §3.4)

Until then, this is the stable forward-look for the three most-asked architecture questions.
