# Proposal — Data / ML / EDA framework (v1.0 vertical blueprint)

> **Status:** Proposed — forward-looking, **not** a committed plan. The runtime
> ships its v0.1 lighthouse (the software-development loop) first; this is a
> candidate **v1.0 fast-follow framework** that proves the runtime is not a
> one-trick pony.
>
> **Provenance.** Extracted (2026-06-07) from a working "capability dictionary"
> thinking-tool before that tool was discarded. It is **point-in-time** — when the
> ML framework is actually built, re-validate every mapping against the *then-current*
> primitives (they will have moved). Treat the difficulty/status notes as the
> 2026-06-07 read, superseded by `docs/execution-status.md` + the spec.
>
> **What this is NOT.** New runtime (L1) work. The thesis below is precisely that a
> data/ML/EDA product is an **L2 framework** composed of **L3** tools + skills +
> agents on primitives the runtime already has.

---

## The three layers (so the thesis is self-contained)

| Layer | What it is | Built in | By whom |
|---|---|---|---|
| **L1 — Runtime primitives** | The engine: drone, event pipeline, capability enforcer, plan FSM, hooks, HITL, budget, gap detection, sandbox. Generic; no opinion about *what* the agent does. | Rust (`crates/`) | Project, once |
| **L2 — Framework composition** | Declarative wiring of primitives into a product. | `framework.json` + companion `.md` artifacts | Per product |
| **L3 — Domain capabilities** | The actual work: query a DB, profile a dataset, train a model. | Tools (MCP/inline) + Skills (markdown) + Agents (JSON) | A user in the workbench |

**The thesis:** "DB management / ML model train-test / EDA" is **not** a set of new
runtime parts to build. It is **one L2 framework** (`examples/data-science/framework.json`)
made of L3 tools + skills + agents running on the primitives already shipped. The
runtime's job is to make those capabilities **safe, observable, resumable, and
budgeted** — which is exactly what the existing primitives do.

---

## What the framework decomposes into

**Agents** (L3 — JSON entries, capability-narrowed):

- `data-orchestrator` — root; plans the analysis, spawns specialists.
- `eda-agent` — profiles datasets, surfaces distributions / correlations / outliers.
- `feature-agent` — cleaning, encoding, feature engineering.
- `modeling-agent` — selects, trains, tunes models.
- `eval-agent` — holdout/CV metrics, error analysis, reports (a natural critic / reflection agent).

**Tools** (L3 — mostly MCP-bound; **no new runtime primitive**):

- DB access → a **Postgres/SQLite MCP server** (`query`, `list_schemas`, `describe_table`). Read-only by capability default.
- Dataframe profiling → a tool that runs `pandas-profiling` / `ydata-profiling` **inside `runtime-sandbox`**.
- Train / test → a code-execution tool that runs sklearn / XGBoost / PyTorch **in the sandbox**, returns metrics JSON.
- Plotting / EDA artifacts → a tool that writes charts to a `file_access`-scoped artifacts dir.

**Skills** (L3 — markdown methodology, *read* not *run*):

- `eda-methodology` — how to approach an unseen dataset (the analyst's playbook).
- `model-selection` — when to reach for which model family.
- `train-test-discipline` — leakage avoidance, CV, holdout hygiene.

---

## How existing primitives make it safe & sane (zero new engine code)

| Concern in data/ML work | Handled by (already shipped) |
|---|---|
| "Don't let the agent drop a prod table / touch raw data" | `dont_touch` globs + hard **rails**; read-only DB capability |
| "A training run could cost $$$ / burn hours" | **Budget** scopes + `hitl_at_percent` + `hard_stop` |
| "Approve before we kick off the expensive train job" | **HITL** `on_risky_tool` / `on_plan_approval` |
| "Verify the model beats the baseline before accepting" | **post_task hook** (`category: verify`) on metrics; `on_failure: rollback` |
| "A train script crashed the session" | **drone** survives; **snapshot/resume** continues |
| "What did the modeling run actually decide and why?" | **signals + VDR** (the decision log) |
| "Run untrusted training code safely" | **runtime-sandbox** (OS fences) + capability enforcement |
| "It needs a DB tool it doesn't have" | **gap detection** + `request_capability` → suspend → install |
| "Show me the pipeline as it runs" | the **live graph** (agents → tools → verify nodes) |

> **Note on the verify gate.** "Is this model good?" has no exit code — so the
> data/ML framework leans on **HITL judgment gates** at review points (the `eval-agent`
> output a human approves), with **objective sub-gates** as verify hooks where they
> *do* exist (trains without error / beats a baseline / a no-leakage check). This is
> the fuzzy-front / objective-back balance, and it is exactly what the HITL primitive
> is for.

---

## What this vertical *would* newly want (and the cheapest path)

| Want | Cheapest delivery (not an L1 primitive unless noted) |
|---|---|
| Persistent DB connections + schema awareness | A DB **MCP server** (L3). M. |
| Reusable artifacts across turns (datasets, fitted models) | Scratchpad dir via `file_access` + optional artifact registry (M). |
| "Remember last run's findings" | A `memory` **MCP server** (L3) — *much* cheaper than an L1 memory primitive. M. |
| Long EDA over big context | Either `fresh_context_per_task` summaries or context compaction (M). |
| Compare 20 model configs and score | The **eval harness** — the one piece worth promoting toward L1. L. |
| Notebook-style interactive exploration | A code-exec tool with a persistent kernel (L) — *real* new work; defer. |

---

## Bottom line

~90% of the data/ML/EDA vertical is **L2/L3 composition** on what already exists. The
only items that are genuinely new *runtime* capabilities are **(a)** an **eval harness**
and **(b)** optionally a **retrieval/memory** primitive — and even (b) is better started
as an **MCP server** (L3) than as engine code. Build the engine to *run things*; let the
data domain live in a framework authored in the workbench.
