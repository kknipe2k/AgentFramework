# Agent Runtime — Full Code & Design Review

**Date:** 2026-06-09 · **Tree:** `main` @ `c612f7c` (post-M08.9, pre-M09) · **Reviewer:** Claude (external-review posture, maintainer-commissioned)
**Method:** 7 parallel deep-read review passes (core execution, persistence/IPC/drone, sandbox/MCP, schemas/codegen, Tauri shell/frontend, tests/CI/supply-chain, design/docs coherence) + current Anthropic API reference cross-check. Every 🔴 finding below was independently line-verified against the source before inclusion.

**Tracking:** the eight Critical findings (§2) are logged as **TD-050 … TD-057** in `docs/tech-debt.md` as the orchestrator's actionable queue. This document is the full evidence + rationale behind those entries.

**Honesty scope (rule 11):** This is static review. The assembled app was NOT run; no test suite or CI gate was executed in this session. Behavioral findings are code-grounded hypotheses unless they cite the project's own IRL/eval evidence. The tests/CI domain was covered by direct targeted verification (the dedicated review agent was cut off mid-run by a rate limit); the planned fresh-web best-practice sweep was likewise cut off and replaced by a current (2026-05) Anthropic API reference plus model knowledge for Tauri/MCP/sandbox guidance.

---

## 1. Executive verdict

**Architecture: sound. Trust boundaries: leaky. API client: works but below best practice on robustness and cost. Process: exceptional, with two integrity gaps of its own.**

The big architectural calls all hold up in mid-2026: the 3-process model (main/drone/sandbox), schema-as-source-of-truth with a real, tested drift gate, direct HTTP+SSE to Anthropic behind a clean `LLMProvider` trait, the modular framework representation (ADR-0022), the Tester isolation model (ADR-0019/0030), and especially the ADR-0032 vertical re-cut — which is the textbook-correct response to the "paints, not executes" discovery. Nothing found in this review requires a redesign.

What the review did find is a consistent pattern: **the product's *internal* security model (capability enforcement, tier gates, SSRF egress, key hygiene) is unusually strong, while the *perimeter* trust boundaries (WebView CSP, local IPC endpoints, renderer-supplied paths, model-supplied paths, sandbox network egress) are weak**. All perimeter gaps are bounded, low-effort fixes — but several should land before M09 ships a runtime that authors and runs real workflows.

---

## 2. Critical findings (all line-verified) → TD-050 … TD-057

### C1 (TD-050) — WebView ships with no CSP, and all four Zustand stores are exposed on `window` unconditionally
`src-tauri/tauri.conf.json:23` → `"csp": null`; `src/App.tsx:77-82` → `window.__graphStore`, `__builderStore`, `__testGraphStore`, `__toastStore` assigned with no dev-gate.
Tauri 2's hardening guidance treats CSP as the baseline mitigation. With `csp: null`, any script injection into the WebView runs unimpeded — and the exposed stores give injected code a typed, direct write path into runtime state (including forging `AgentEvent`s the graph reducer trusts). Today's exploit surface is narrow (local app, no remote content, no `dangerouslySetInnerHTML` anywhere), but M09–M13 progressively render more model-generated content. **Fix:** set a restrictive CSP (`default-src 'self'`; style allowances as needed), gate the store exposure behind `import.meta.env.DEV`, and consider Tauri's isolation pattern.

### C2 (TD-051) — Renderer-supplied paths reach the filesystem unconfined
`src-tauri/src/commands.rs:1508-1538` (`save_framework`/`load_framework` accept raw `dir: String`), `commands.rs:1333-1350` (`import_artifact` file source).
The OS file dialog is a UX affordance, not an enforcement boundary — `invoke("save_framework", {dir: "../../anywhere"})` is accepted. Combined with C1 this is a real chain: script injection → arbitrary file write of attacker-shaped framework JSON. **Fix:** confine to dialog-returned paths (Tauri's dialog plugin can return validated handles), or canonicalize + require containment under an allow-listed root.

### C3 (TD-052) — Built-in `Read`/`Write` tools never canonicalize the model-supplied path
`crates/runtime-main/src/sdk/builtin_tools.rs:98,119` — the raw LLM-provided path string is used for **both** the capability scope check and the actual `fs::read_to_string`/`fs::write`.
Symlinks defeat the scope check: with grant `read: /workdir/**`, a symlink `/workdir/link → /etc/passwd` passes the glob and reads outside the granted scope; `..` traversal segments are similarly unnormalized. This undermines the capability model — the product's central promise — for the two tools that already "execute — observed" (E-01/E-02). **Fix:** canonicalize before the check and perform IO on the canonicalized path (and decide the symlink policy explicitly); add the adversarial cases to the E-02 eval.

### C4 (TD-053) — All three IPC layers use unbounded `LinesCodec`; the drone's Unix socket is world-connectable
13 sites: `LinesCodec::new()` (max length = `usize::MAX`) in `runtime-drone/src/ipc.rs`, `runtime-sandbox/src/ipc.rs`, `runtime-main/src/{drone_ipc,sandbox_ipc}/connection.rs`. Plus `runtime-drone/src/ipc.rs:82` — `UnixListener::bind` with no `set_permissions` (umask-dependent, typically world-accessible), and the Windows named-pipe DACL hardening is still marked deferred ("to M05" — long past).
Any local process can connect to the drone socket and issue `QuerySessionDb` (read the session DB), `SnapshotNow`, or `GracefulShutdown`; a peer (or corrupted pipe) writing bytes without a newline buffers unbounded memory. The `MaxLineLengthExceeded` error arms already exist — they're just unreachable. **Fix:** `LinesCodec::new_with_max_length(4 MiB)` everywhere; `chmod 0600` the socket after bind; set an explicit DACL on the Windows pipe (v0.1 is Windows-first — the pipe is the production surface).

### C5 (TD-054) — No timeout anywhere on the provider HTTP/SSE path, and no retry on 500/529
`crates/runtime-main/src/providers/anthropic.rs:55-58` — `reqwest::Client::builder().pool_max_idle_per_host(2).build()`; no `connect_timeout`, no `read_timeout`, no `tokio::time::timeout` around `stream.next()` in the run loop.
A stalled connection (proxy idle-close, network partition) parks the session task forever — an unkillable session in a product whose core claims include "suspends cleanly." Separately, 429 is mapped (with `retry-after`) but never retried, and 500/529 (`overloaded_error`) get no backoff at all; Anthropic's guidance (and every SDK's default) is exponential backoff on 429/5xx/529. **Fix:** connect timeout + per-read idle timeout (60–120s) on the SSE stream; provider-level bounded retry with jitter honoring `retry-after`.

### C6 (TD-055) — Sandbox seccomp allowlist permits `AF_INET`/`AF_INET6` sockets — an exfiltration channel, and a blocker for M12 shell-exec
`crates/runtime-sandbox/src/seccomp.rs:104-117` — `socket`, `bind`, `sendto`, `recvfrom`, etc. allowed with **no argument filtering**; the "Unix domain sockets for IPC" intent (comment at :103) is not enforced. `connect` is absent (good), but `bind`+`sendto` UDP exfiltration is reachable, and landlock does not cover network. Tolerable for today's validate-only sandbox; **must close before `SandboxRequest::Execute`** (ADR-0032/M12): add `SCMP_A0 == AF_UNIX` conditional rules on `socket`/`bind`/`sendto`.

### C7 (TD-056) — Process-integrity: the claimed nightly mutation-testing gate does not exist
CLAUDE.md §5 states "`cargo-mutants` runs nightly on `main`" and makes mutation gates blocking at cluster-close. `.github/workflows/` contains only `ci.yml`, `fuzz-nightly.yml`, `release.yml`; zero references to `mutants` in workflows or lefthook. By the project's own CI-parity and grounded-claims standards, a documented gate that is not implemented is a 🔴. Either add the nightly workflow or amend §5 to say the gate is manual/at-cluster-close-only.

### C8 (TD-057) — Design-level: prompt injection is absent from the threat model
No spec section, ADR, or `docs/SECURITY.md` content addresses the dominant 2026 attack vector for tool-wielding agents: attacker-controlled content (a fetched web page, an MCP tool result or *tool description* — `runtime-mcp/src/transport/mod.rs:148-152` passes server-supplied descriptions through untouched) steering the agent into misusing its granted capabilities. Capability enforcement (L1–L5) bounds the blast radius but is not a substitute: an injected agent can still do anything *within* its grants (e.g., exfiltrate file contents through any granted network tool). M09 (real MCP tool + file write) makes this surface live. **Fix:** a threat-model addendum + concrete mitigations (tool-description length caps/sanitization plus provenance display in the UI, injection-aware HITL prompts for sensitive capability use) and an explicit statement of what is *not* defended. This belongs in the M12 H-ladder security ADR — which ADR-0032 requires and which has not been authored yet.

---

## 3. Anthropic API integration — correctness & best-practice assessment

Verified against the current (2026-05) API reference. The wire basics are right: `anthropic-version: 2023-06-01`, `x-api-key` via `SecretString` (no Debug leaks anywhere — genuinely well done), correct SSE event grammar (`message_start`, `content_block_start/delta/stop` incl. `input_json_delta`/`thinking_delta`/`signature_delta`, `message_delta`, `error`, `ping`), and the pricing table in `estimate_cost` (Opus $5/$25, Sonnet $3/$15, Haiku $1/$5; cache ×1.25/×2.0/×0.1) is **accurate**.

Gaps, in priority order:

1. **🟡 `temperature` passthrough will 400 on the default model family.** `anthropic.rs` sends `temperature` when set; Opus 4.7 (in the production model list) rejects `temperature`/`top_p`/`top_k` with 400 — sampling params are removed on Opus 4.7+. Latent until some config sets it; strip or gate per-model.
2. **🟡 `stop_reason` is a raw string passthrough; `max_tokens`, `refusal`, `model_context_window_exceeded`, `pause_turn` are not handled** (`anthropic_sse.rs:264-291` → `AgentComplete{result}`). A truncated or refused run currently *reads as a normal completion* — a labels-tell-the-truth (DESIGN.md rule 8) violation at the engine level. Make it a typed enum with explicit arms.
3. **🟡 No prompt caching.** The run loop re-sends the full conversation each turn at full input price. `cache_control` on the system prompt + last conversation block would serve cached history at ~0.1× — for a product with a *budget enforcer as a headline feature*, this is the single biggest cost/latency lever available, and it's unused. (Also fits the loop: stable system prompt first, volatile content last.)
4. **🟡 `count_tokens` omits `system` and `tools`** (`anthropic.rs:183-224`) — budget pre-flight estimates are systematically understated for exactly the frameworks the product targets.
5. **🟡 No context-window management.** Long sessions will eventually hit the window; with #2 unhandled this fails silently. Plan for it (truncation policy now; server-side compaction beta later).
6. **🟡 Malformed streamed tool-input JSON is silently coerced to a `Value::String`** and dispatched (`anthropic_sse.rs:252-257`) — should be a provider error, not a garbled tool call. Tool results are also fed back as stringified text only — consider structured `tool_result` content and `is_error: true` for failed tools.
7. **🟢 Model catalog freshness:** list is `opus-4-7`/`sonnet-4-6`/`haiku-4-5` — all valid, but Opus 4.8 (same price, current Opus tier) is out. Recommendation: query `GET /v1/models` at startup for catalog + capabilities rather than hardcoding (the runtime is a *product*; hardcoded model lists rot).

---

## 4. Important findings by domain

### Persistence / recovery (runtime-drone, runtime-main)
- **🟡 Uncertain-tool-call pairing has a false-negative** (`runtime-drone/src/snapshot.rs:209-268`): results matched by `HashSet<(agent_id, tool_name)>` — a repeated call to the same tool lets one result vouch for two invocations, so a genuinely-uncertain invocation can be marked certain. This degrades the `tool_call_uncertain` recovery invariant (spec §1b). Use a count-based multiset or pair by signal id.
- **🟡 `signals.timestamp` is `TEXT`** (`migrations/000_initial.sql:38`) while every other timestamp is `INTEGER`; ordering is coincidentally correct (13-digit epoch strings) but numeric comparisons in SQL will mislead. Migrate.
- **🟡 `is_select_only` (VDR read-only SQL gate) is a lexical prefix check** bypassable in principle via comment tricks (`runtime-drone/src/vdr.rs:270-303`); actual risk is mitigated because execution uses single-statement `prepare`, but the shell layer adds no second validation (`SqlInspector` → `query_session_db` passes raw SQL through). Add comment-stripping or an EXPLAIN-based check, plus comment-injection test vectors.
- **🟡 Synchronous rusqlite calls run on tokio runtime threads** under the connection mutex (`command_handler.rs:143-179`, heartbeat) — fine at today's microsecond scale; move to `spawn_blocking`/dedicated thread before projections get heavier (M10 plans).
- **🟢 Events emit `timestamp: 0`** for `SnapshotWritten` (`shutdown.rs:81`, `command_handler.rs:315`) — DB row is correct, broadcast isn't.

### Sandbox / MCP
- **🟡 No per-call timeout on MCP calls** — `McpError::Timeout` exists but nothing produces it; a stuck server hangs the health-ping task forever.
- **🟡 `StdioConnection::shutdown` is a no-op**; with `Arc` clones held (health-ping snapshots), child MCP processes can outlive `remove_server` until app exit.
- **🟡 `http://` accepted for remote MCP servers** — auth headers in cleartext; require/warn on non-`https`.
- **🟢 `job_objects.rs` module comment inverts `BREAKAWAY_OK` semantics** (the flag *permits* breakaway; re-check the flag choice against intent when M12 hardening lands) and two `/ SAFETY:` typos. Otherwise: 13/13 unsafe blocks carry substantive SAFETY comments — full compliance — and the landlock-before-seccomp install ordering is correct *and documented*.

### Schemas / codegen / types
- **🟡 `$id`↔`$schema` URL inconsistency**: schema `$id` is `…/framework.v1.json`; both examples declare `…/framework/v1.json`. Verified non-fatal today (runtime validates by serde-deserializing into the generated `Framework`, not URL dispatch; the schema's own `$schema` property is an unconstrained string) — but it's a trap for the planned URL-dispatch loader. Pick one form; constrain the property.
- **🟡 Runtime validation is serde-shape-only** — JSON-Schema constraints that typify can't encode are unenforced at runtime (CI validates *examples* only). Verify which constraints survive into generated types; consider a runtime `jsonschema` pass for user-authored documents.
- **🟡 Capability strings are unconstrained** in `common.v1.json` (`spawn_agents: ["*"]` passes schema; enforcement lives only in the loader), and **glob narrowing is equality-only** (`declaration.rs:63-67`) — a parent `**/*` does not subsume child `src/**`, silently rejecting legitimate narrowing. Both deserve tracked entries.
- **🟡 The hand-mirror surface is growing**: 13 hand-written interfaces in `src/lib/ipc.ts` + 5 mirrored enums in `runtime-core/src/event.rs` (typify cross-schema-ref limitation). The newest cluster (`TestOutcome`) lacks Rust-side serde pin tests. **Recommendation:** ts-rs/specta derive on the Tauri-bridge structs, generated into `src/lib/ipc-generated.ts` via xtask — closes the side-channel without schema promotion.

### Frontend / shell
- **🟡 Zero error boundaries** in the React tree — one render error in model-fed components (Inspector, drill-down) unmounts the whole app. Add top-level + per-pane boundaries before M09.
- **🟡 `BuilderCanvas.onDrop` blind-casts `JSON.parse`** of the DND payload (`BuilderCanvas.tsx:92`).
- **🟢** Toast timers never cancelled; inline error `<p>` lacks `aria-live`; `McpServerSummary.status` typed `string` not a union; Inspector save path missing its DESIGN-rule-1 success toast.

### Tests / CI / supply chain
- Verified **parity holds** between CLAUDE.md §6 coverage commands and `ci.yml` (byte-identical flags for the gates checked), 16 CI jobs including dco, gitleaks (full-history), schema validation, append-only diff enforcement, stage-prompt validation, e2e (Playwright) + e2e-tauri-driver, fuzz-smoke + nightly fuzz; top-level `permissions: contents: read`. All 14 `#[ignore]`s have documented reasons (live-API / keychain). proptest is real where claimed.
- **🟡 Only one fuzz target exists** (`drone_command_decode`); §5's own policy names the SSE parser (M2) — the hand-written `anthropic_sse.rs` is the parser most exposed to hostile-ish input and is unfuzzed.
- **🟡 Actions are tag-pinned (`@v4`, `@v2`)**, not SHA-pinned — post-2025 supply-chain guidance (tj-actions compromise) is to SHA-pin third-party actions (`markdownlint-cli2-action`, `lychee-action`, `gitleaks-action` especially).
- **🟡 `.expect("capabilities_for_tool returns ≥1 decl")`** in the L1 gate (`event_pipeline.rs:230-232`) — invariant held only by another module's contract; a future refactor turns it into a session-killing panic. Make it an error path.
- **🟡 HITL `prompt_id` for capability violations is session-scoped** (`agent_sdk.rs:847-858`) — concurrent violations all resolve on the first user response. Known-simplified inline; needs per-violation ids before M10's HITL milestone.

### Docs / process
- **🔴 CLAUDE.md §1 Status is five milestones stale** ("M1/M2 merged; M3 next") — and violates its own §3 rule against snapshotting live state. Every fresh session loads this as ground truth. Replace with pointers. *(Not in scope for the commit that introduced this report; flagged for the orchestrator.)*
- **🟡** README roadmap shows pre-ADR-0031 numbering and "ADRs 0001–0029"; ADR-0032 still `Proposed` while governing the active roadmap (ADR-0031 already says "superseded in part by 0032"); MVP index one sub-milestone behind; CLAUDE.md says Rust 1.80+ vs pinned 1.95.0.
- **Process assessment:** the execution-status ledger + grounded-claims rule + tauri-driver real-app gate are the three highest-value mechanisms (each traceable to a failure class it now prevents). The drag: triple overlapping append-only ledgers (gap-analysis / tech-debt / execution-status) without cross-indexing, M08.x doc sprawl (6+ IRL files), and a monotonically growing mandatory read-set. Consolidation is worth one deliberate pass post-M09.

---

## 5. Strengths worth naming (all verified or evidence-cited)

1. **SSRF egress defense is production-quality** (`import/egress.rs`): IPv4-mapped-IPv6 unwrapping, CGNAT/ULA/link-local ranges, DNS pinning, redirect re-validation, body cap, fully unit-testable resolver seam. Better than most production code.
2. **Secret hygiene is correct end-to-end** — `SecretString` everywhere, no Debug/log leaks found on any reviewed path; keyring ops correctly `spawn_blocking`'d.
3. **The capability enforcer is a real single enforcement point** with correct L4-before-L1 ordering, proptest-verified narrowing asymmetry, and MCP dispatch gated server-independently.
4. **The codegen drift gate is real** — byte-compare regeneration with mutation-detection tests for both Rust and TS outputs; `cmd_error_ext` shows the right pattern for extending generated types.
5. **Persistence invariants are structural**: WAL pragma ordering textbook-correct and tested; snapshot append-only enforced by the absence of any UPDATE path, not by convention.
6. **The Tauri command layer is uniformly seam-tested** (26 commands, every one behind a `*_with` seam) with a genuinely minimal capability grant set, no `dangerouslySetInnerHTML`, module-scope React Flow `nodeTypes`, and WAI-APG-correct modals/toasts.
7. **The process machinery demonstrably learns**: the paints→executes ledger, IRL gates, and assembled-app regression mandate are a novel, working answer to the central failure mode of AI-built software — and the project's own retros prove the loop closes.

---

## 6. Prioritized recommendations

**Before M09 merges (perimeter + provider robustness):**
1. CSP + dev-gate the `window.__*Store` exposure (C1/TD-050); confine `save_framework`/`load_framework`/`import_artifact` paths (C2/TD-051).
2. Canonicalize built-in tool paths before check **and** IO; add symlink/`..` adversarial cases to E-02 (C3/TD-052).
3. `LinesCodec::new_with_max_length`; socket `0600`; Windows pipe DACL (C4/TD-053).
4. reqwest connect + SSE idle timeouts; bounded retry w/ jitter on 429/500/529 (C5/TD-054).
5. Replace the L1 `.expect()` with an error path; fix CLAUDE.md §1 + README staleness (cheap, high-confusion-cost items).

**M10 window (correctness + cost):**
6. Typed `stop_reason` handling (refusal / max_tokens / context-exceeded / pause_turn) surfaced honestly in the verdict pipeline.
7. Prompt caching on system+history; include `system`+`tools` in `count_tokens`; context-window policy.
8. Per-violation HITL prompt ids; uncertain-tool multiset pairing; `signals.timestamp` migration; MCP per-call timeouts + real shutdown.
9. React error boundaries; onDrop validation; ts-rs for the IPC mirror cluster.

**M12 gate (before any `Execute`):**
10. seccomp socket-family argument filters (C6/TD-055); per-exec landlock profile; the H-ladder security ADR **including the prompt-injection threat model** (C8/TD-057) and MCP tool-description handling.

**Process:**
11. Add the cargo-mutants nightly workflow or amend §5 (C7/TD-056); add an SSE fuzz target; SHA-pin third-party actions; flip ADR-0032 to Accepted; plan one ledger-consolidation pass.

**Product/API freshness:**
12. Model catalog via `GET /v1/models` (+ add Opus 4.8); drop/gate `temperature` for 4.7+ models.

---

## 7. Already-tracked vs net-new

The project's ledgers (tech-debt, gap-analysis, execution-status, IRL findings) accurately capture the *execution-wiring* gaps (sub-agents/plans/hooks paint-only, gap-resume missing, budget persistence, save_framework companions, palette integrity). **None of the Critical findings above were tracked** — they cluster in exactly the areas the IRL process doesn't exercise: adversarial inputs, trust boundaries, network failure modes, and cost optimization. That's the blind spot of an IRL-driven verification culture (it verifies the happy path *really* runs), and worth a standing "adversarial pass" in the process — this review can serve as its first instance.
