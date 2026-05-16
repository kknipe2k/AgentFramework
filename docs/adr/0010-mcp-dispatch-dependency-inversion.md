# ADR-0010: MCP dispatch via dependency inversion (runtime-main trait, runtime-mcp impl)

**Status:** Accepted
**Date:** 2026-05-15
**Deciders:** @kknipe2k
**Tags:** ipc, capability, scope, mcp

## Context

M06 Stage D wires MCP tool dispatch through the §8.security L1+L4 capability
gates and implements §5a tool namespace resolution. The `docs/build-prompts/
M06-mcp-basic.md` Stage D design (D.3.2 / D.3.3) places the concrete
`McpDispatcher` at `crates/runtime-main/src/sdk/mcp_dispatch.rs`, importing
`McpClient` + `NamespaceResolver` from `runtime-mcp`, and directs
`runtime-main/Cargo.toml` to add a path-dependency on `runtime-mcp`.

That is impossible to build. M06 Stage C established `runtime-mcp` →
`runtime-main` (`crates/runtime-mcp/src/client/mod.rs:40` consumes
`runtime_main::audit` for the M05.E audit writer; `crates/runtime-mcp/
Cargo.toml:15` declares `runtime-main = { workspace = true }`). Adding
`runtime-main` → `runtime-mcp` closes a Cargo **circular package
dependency**; `cargo` refuses to resolve it. The Stage D prompt's
`<phase_doc_inventory_audit verified="true">` for the runtime-main
dispatcher path and its `<dependency_audit_check>` directing the cycle-
causing dep are both factually wrong.

Doing nothing blocks the entire M06.D deliverable. The crate-boundary
choice has architectural weight (it changes module placement, the
`mcp_dispatch_integration.rs` test-file location, and introduces a new
cross-crate seam), so it is recorded here rather than picked silently.

## Decision

We resolve the cycle by **dependency inversion**, keeping the established
`runtime-mcp` → `runtime-main` direction intact.

`crates/runtime-main/src/sdk/mcp_dispatch.rs` defines the dispatch *seam*
the SDK run loop calls — the `McpToolDispatch` trait plus the
`McpDispatchOutcome` value type — and carries **no** dependency on
`runtime-mcp`. The SDK (`runtime-main`) holds an
`Option<Arc<dyn McpToolDispatch>>` and intercepts `ProviderEvent::ToolUse`
through it before the existing Stage A non-MCP L1 path.

`runtime-mcp` hosts the concrete implementation: `NamespaceResolver`
(`crates/runtime-mcp/src/namespace/`) and `McpDispatcher`
(`crates/runtime-mcp/src/dispatch.rs`), the latter implementing
`runtime_main::sdk::McpToolDispatch`. The Tauri shell (`src-tauri`,
which already depends on both crates) constructs the concrete
`McpDispatcher` and injects it into the SDK as `Arc<dyn McpToolDispatch>`.

This mirrors the codebase's established seam pattern (`Arc<dyn
Connection>`, `Arc<dyn SecretStore>`, `Arc<dyn Transport>`,
`Arc<AuditWriter>` injected at the shell layer). The end-to-end dispatch
integration test moves to `crates/runtime-mcp/tests/
mcp_dispatch_integration.rs` (where the concrete impl lives); a
trait-level wire test against a mock `McpToolDispatch` covers the
runtime-main SDK interception.

## Consequences

### Positive
- Compiles. No circular package dependency.
- Idiomatic — reuses the existing dependency-inversion seam archetype the
  codebase already applies at every cross-crate / OS-call boundary.
- Both ≥95% per-crate coverage gates (`runtime-mcp`, `runtime-main`) still
  apply; the seam is small + fully testable via a mock impl.
- The SDK has no compile-time knowledge of MCP — future transport/dispatch
  changes stay inside `runtime-mcp`.

### Negative
- Deviates from the literal phase-doc placement (concrete dispatcher in
  `runtime-main`). Documented as a Stage D drift in the retrospective.
- One extra indirection (`dyn McpToolDispatch`) on the tool-dispatch path.
  Negligible — dispatch is already async + I/O-bound.

### Neutral / future implications
- `src-tauri` becomes the single wiring point for MCP dispatch (consistent
  with how it already wires audit + keychain + drone IPC).
- M06.E renderer + M06.V Wire pass trace the seam, not a direct call.

## Alternatives Considered

### Alternative A: Relocate `CapabilityEnforcer` + `audit` to `runtime-core`
**Rejected because:** large blast radius across all of M05/M06 (every
capability/audit import path changes), touches CODEOWNERS-flagged
capability code for a structural-only reason, and inverts the intended
layering (`runtime-core` is schema-generated types + thin helpers, not the
security enforcer's home). High risk for no product benefit.

### Alternative B: Concrete dispatcher entirely in `runtime-mcp`, no runtime-main seam
**Rejected because:** drops the Stage D SDK-wire deliverable (the
`ProviderEvent::ToolUse` → resolve → check → dispatch trace) — the SDK run
loop in `runtime-main` could not reach the dispatcher at all without the
trait seam, deferring the user-visible wire indefinitely.

## Related

- Spec sections: §5 (MCP Manager), §5a (Tool Namespace Resolution),
  §8.security (L1/L4)
- Prior ADRs: ADR-0006 (mcp-servers-schema), ADR-0007 (in-process HITL
  seam architecture — same shell-injected-seam archetype), ADR-0009
  (L1+L2a SDK wire-up — the Stage A wire this extends)
- Phase doc: `docs/build-prompts/M06-mcp-basic.md` Stage D
- External references: Dependency Inversion Principle (Martin, 1996)

## Notes

Surfaced to the maintainer before the Stage D red phase via the
`AskUserQuestion` decision gate; maintainer selected "Dependency
inversion (Recommended)". Status set to Accepted in the same PR per
CLAUDE.md §11 step 4.
