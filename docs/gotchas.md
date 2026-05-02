# Common Gotchas (lessons learned)

The spec is large and the project covers a lot of ground. These are traps that have already bitten the work or are predictable based on the design. CLAUDE.md §15 references this file as the source.

1. **Tool ≠ Skill ≠ Agent.** Three distinct concepts (§0b). Tools are called. Skills are loaded into context. Agents are spawned. Don't conflate them; the schemas, file formats, and runtime mechanics are different.
2. **Capability narrowing on Agent→Agent edges.** A child agent's `allowed_tools` and `allowed_skills` cannot exceed the parent's. The Builder Canvas (Phase 9) enforces this automatically; manual JSON editing must respect it.
3. **v0.1 ships STANDARD mode hardcoded.** No mode router (§3b). The framework JSON's `modes` field still exists in schema but is not evaluated at runtime in v0.1.
4. **v0.1 ships `fresh_context_per_task` only.** The continuous loop policy (Ralph-style) is in the schema but not implemented. `examples/ralph/framework.json` exists but won't run on v0.1.
5. **v0.1 ships Novice + Promoted tiers only.** No Operator tier. Promoted is blocked from auto-accepting `shell: true` and `network: ["*"]` artifacts even though the tier's general behavior is auto-accept-when-validated.
6. **v0.1 is single-session.** §1c Multi-session is v1.0. Do not write multi-session code paths in v0.1; they create surface area without benefit.
7. **v0.1 is Windows-only.** Not because Tauri is Windows-only — Tauri is cross-platform — but because we test only on Windows in v0.1, and macOS/Linux ports come at v1.0. CI still runs on all three OSes to catch drift early.
8. **No telemetry, ever.** No analytics, no crash reporter, no "anonymous metrics," no phone-home. Per §13 of spec. Adding any requires an ADR with public dashboard plan + opt-in mechanism.
9. **Anthropic API is hit directly.** No `@anthropic-ai/sdk` dep, no `anthropic-rs`. `reqwest` + `eventsource-stream` only. The API surface is small and stable; direct HTTP keeps the dependency surface minimal.
10. **Tauri allowlist is the security boundary.** The renderer has no Node API. Anything the renderer needs from Rust goes through a typed `#[tauri::command]`. Don't widen the allowlist without considering capability implications.
11. **Drone ≠ Main ≠ Sandbox.** Three Rust processes. Drone owns SQLite + snapshots + recovery (per session). Main owns the agent loop, MCP, providers, framework loader, capability enforcer. Sandbox is per-artifact, OS-isolated, used for L3 validation. Don't blur these.
12. **IPC is two layers.** Renderer↔Main is Tauri typed IPC. Main↔Drone is framed JSON over Unix socket / named pipe. Different mechanisms with different semantics. Don't try to use Tauri IPC for drone communication.
13. **SQLite WAL pragmas matter.** `PRAGMA journal_mode = WAL`, `PRAGMA synchronous = NORMAL`, `PRAGMA busy_timeout = 5000`, `PRAGMA foreign_keys = ON`. Set them in this order at every connection open. Missing busy_timeout causes flaky tests under contention.
14. **Snapshots are append-only.** Drone never updates a snapshot row; new snapshot = new row. State_hash deduplication happens at read time, not write time.
15. **Resume rebuilds history, doesn't re-execute.** Tool calls in flight at crash time are flagged `tool_call_uncertain` and surfaced to the user. Don't replay tool calls automatically.
16. **`request_capability` is a meta-tool.** It's auto-injected into every agent's tool list. When the model calls it, the runtime translates to `tool_missing` or `skill_missing` and routes through gap flow. Agents can decline `skill_load_requested` events but typically comply.
17. **Mode-variant skills filter sections.** A `skill.md` with `mode_variants: { LITE: { include_sections: ["quick"] }, ... }` has its body filtered by section header at load time. The full markdown is on disk; the model sees only the slice for the active mode.
18. **JSONLogic for triggers.** Programmatic skill triggers use a JSONLogic-style expression language. Operators allowed in v0.1: `var`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `and`, `or`, `not`, `in`, `+`, `-`, `*`, `/`. Adding operators requires extending the evaluator; do not silently extend.
19. **Capability declarations are mandatory for generated artifacts.** Hand-authored artifacts can omit the `capabilities` block, but they default to Operator-tier-only loading. Generated artifacts (Phase 8) must declare capabilities; the validator rejects missing blocks.
20. **DCO sign-off is mandatory.** `git commit -s`. Without it, the commit is rejected by the CI hook (once configured at M1+).
