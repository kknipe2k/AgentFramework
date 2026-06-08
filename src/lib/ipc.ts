import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import type { AgentEvent, TierRef } from '../types/agent_event';
import type { CapabilityDeclaration } from '../types/capability';
import type { CmdError } from '../types/error';
import type { Framework } from '../types/framework';
import type { McpServerConfig } from '../types/mcp';

/**
 * One tool a connected MCP server exposes. Mirrors the serde shape of
 * `runtime_mcp::transport::McpTool` (NOT schema-generated — the struct
 * lives in `crates/runtime-mcp/src/transport/mod.rs:62` and crosses the
 * Tauri bridge as-is). `mcp_test_connection` returns `McpTool[]`.
 */
export interface McpTool {
  name: string;
  description?: string;
  input_schema: unknown;
}

/**
 * One registered MCP server's summary row. Mirrors the serde shape of
 * `runtime_mcp::client::McpServerSummary`
 * (`crates/runtime-mcp/src/client/mod.rs:60`). `mcp_list_servers`
 * returns `McpServerSummary[]`.
 */
export interface McpServerSummary {
  name: string;
  transport: string;
  has_auth: boolean;
  status: string;
}

export async function invokeRunSmokeSession(): Promise<void> {
  await invoke('run_smoke_session');
}

export async function invokeSetApiKey(key: string): Promise<void> {
  await invoke('set_api_key', { key });
}

/**
 * Whether an Anthropic API key is present in the OS keychain. The App
 * mount reads this so a key entered once survives an app restart
 * (M07-IRL #7). Resolves `false` on any keychain error — the renderer
 * treats "can't tell" as "absent" (the user can re-enter the key).
 */
export async function invokeHasApiKey(): Promise<boolean> {
  return await invoke<boolean>('has_api_key');
}

/**
 * Run a SELECT-only query against the session database. The drone-side
 * validator rejects DDL/DML/PRAGMA + compound statements; rejection
 * surfaces as a `CmdError::Internal`-shape rejection (use
 * `unwrapCmdError` to render).
 *
 * Returns rows as JSON objects keyed by column name.
 */
export async function invokeQuerySessionDb(sql: string): Promise<Record<string, unknown>[]> {
  return await invoke<Record<string, unknown>[]>('query_session_db', { sql });
}

/**
 * Replay a prior session by id. Main reads the signal log via drone
 * IPC, translates each signal into an `AgentEvent`, and re-emits via
 * the existing `agent_event` channel so `graphStore.applyEvent`
 * reconstructs the graph identically.
 */
export async function invokeReplaySession(sessionId: string): Promise<void> {
  await invoke('replay_session', { sessionId });
}

/**
 * Reconstruct the most-recent persisted session's graph — the
 * reload-after-restart fallback for {@link invokeReplaySession} (TD-044).
 *
 * `lastSessionId` in localStorage survives a soft reload but a full app
 * restart comes up on a fresh WebView profile that wipes it, so the
 * renderer loses the prior session id. The backend owns persistence: it
 * reads the latest session WITH signals back from the signal log and
 * replays it through the existing `agent_event` channel. Resolves the
 * replayed session id, or `null` when no prior session has signals.
 */
export async function invokeReplayLatestSession(): Promise<string | null> {
  return await invoke<string | null>('replay_latest_session');
}

/**
 * Approve a pending plan (M04 Stage C). Resolves the in-process
 * `ApprovalSeam` (Tauri-managed-state) with `ApprovalDecision::Approved`.
 * The SDK's awaiting plan_loop wakes and emits `plan_approved`, which
 * the renderer re-receives via the existing `agent_event` subscription.
 *
 * Spec §3a Approval-gate primitive.
 */
export async function invokeApprovePlan(planId: string): Promise<void> {
  await invoke('approve_plan', { planId });
}

/**
 * Submit user-typed revisions to a pending plan. The string is passed
 * through opaque per CLAUDE.md §8.security; the SDK / framework JSON
 * downstream sanitizes before re-prompting the planner agent.
 */
export async function invokeRevisePlan(planId: string, revisions: string): Promise<void> {
  await invoke('revise_plan', { planId, revisions });
}

/**
 * Cancel a pending plan with a free-text reason. Resolves the seam
 * with `ApprovalDecision::Aborted` so the SDK's awaiting plan_loop
 * unblocks and emits `plan_aborted`.
 */
export async function invokeAbortPlan(planId: string, reason: string): Promise<void> {
  await invoke('abort_plan', { planId, reason });
}

/**
 * Resolve a pending HITL prompt with the user's choice (M04 Stage E).
 * Resolves the in-process `HitlSeam` (Tauri-managed-state); the SDK's
 * awaiting HITL gate wakes and the plan loop routes per the chosen token.
 *
 * Spec §6a HITL Policy primitive.
 */
export async function invokeRespondHitl(promptId: string, choice: string): Promise<void> {
  await invoke('respond_hitl', { promptId, choice });
}

/**
 * Recovered session state from {@link invokeRequestResume}.
 * Mirrors `runtime_main::recovery::ResumePlan` wire format.
 */
export interface ResumePlan {
  snapshot_id: string | null;
  plans: Record<string, unknown>[];
  tasks: Record<string, unknown>[];
  uncertain_tool_invocations: string[];
  has_state: boolean;
}

/**
 * Request a session resume (M04 Stage F — spec §1b). Reads the latest
 * snapshot + projected plan/task state + uncertain tool-invocation ids
 * from the drone. Tools are NOT re-invoked (gotcha #15); the SDK will
 * rebuild message history from the snapshot's signal log and generate
 * the next turn fresh.
 */
export async function invokeRequestResume(sessionId: string): Promise<ResumePlan> {
  return await invoke<ResumePlan>('request_resume', { sessionId });
}

/**
 * The four spec §1b actions a user can pick for an uncertain tool invocation.
 */
export type UncertaintyAction = 'retry' | 'skip' | 'mark_complete' | 'abort';

/**
 * Result of recording an uncertainty resolution. Mirrors
 * `runtime_main::recovery::UncertaintyResolution`.
 */
export interface UncertaintyResolution {
  signal_id: string;
  action: UncertaintyAction;
  invocation_id: string;
}

/**
 * Record the user's resolution for an uncertain tool invocation
 * (M04 Stage F — spec §1b). Writes a `tool_call_uncertainty_resolved`
 * decision signal to the VDR via drone IPC.
 */
export async function invokeRespondUncertainty(
  sessionId: string,
  invocationId: string,
  action: UncertaintyAction,
  agentId?: string,
): Promise<UncertaintyResolution> {
  return await invoke<UncertaintyResolution>('respond_uncertainty', {
    sessionId,
    invocationId,
    action,
    agentId: agentId ?? null,
  });
}

/**
 * Store the user-configured per-day global budget cap (M04 Stage F —
 * spec §2a). v0.1 holds the value in process memory only; M10
 * first-run UX persists it. Pass `0` to disable the global cap.
 */
export async function invokeSetGlobalBudget(usdCap: number): Promise<void> {
  await invoke('set_global_budget', { usdCap });
}

/**
 * Read the user's current (persisted/enforced) tier — M08.8.C.fix
 * (#19 display desync). Wraps the EXISTING `get_current_tier` Tauri
 * command (`src-tauri/src/commands.rs:633` → `Tier`, serde lowercase =
 * {@link TierRef}). Takes ZERO JS args; the backend reads its in-memory
 * `CurrentTierState` (seeded from `<app_data_dir>/tier.json` at setup).
 *
 * The App mount seeds the store's `currentTier` from this so the Settings
 * display matches the ENFORCED tier across an app restart — the renderer
 * previously defaulted `currentTier` to `'novice'` and wrote it ONLY from
 * `tier_transition` events, so a restart with a Promoted backend showed
 * Novice while the run enforced Promoted. Mirrors the `invokeHasApiKey`
 * startup seed (App.tsx). DIRECT `TierRef`, no mapping. The seed REFLECTS
 * the enforced tier; it never widens it.
 */
export async function getCurrentTier(): Promise<TierRef> {
  return await invoke<TierRef>('get_current_tier');
}

/**
 * Request a Novice ↔ Promoted tier transition (M05 Stage D — spec
 * §8.security L4). Wraps the EXISTING `request_tier_transition` Tauri
 * command (`src-tauri/src/commands.rs:573`) — M08 Stage G surfaces it,
 * it does NOT reimplement tier logic.
 *
 * The backend persists the new tier to `<app_data_dir>/tier.json`,
 * updates its in-memory cache, and emits a `tier_transition` event on
 * the `agent_event` channel — which `graphStore.applyEvent` already
 * reduces into `currentTier` (graphStore.ts:1549). The Settings panel's
 * displayed tier therefore updates through the EXISTING event path; the
 * wrapper does not return the new tier and the caller does not set it.
 *
 * Idempotent when `targetTier` equals the current tier (the backend
 * returns `Ok` for the no-op — commands.rs:621), so the panel may call
 * freely without a pre-check.
 *
 * `targetTier` is the generated {@link TierRef} ('novice' | 'promoted')
 * — byte-identical to the Rust `Tier` enum's serde form. Operator is
 * NOT a `TierRef` member (v1.0, §0d). Errors surface as the Tauri
 * `CmdError` shape — render via {@link unwrapCmdError}.
 */
export async function requestTierTransition(targetTier: TierRef, reason: string): Promise<void> {
  await invoke('request_tier_transition', { targetTier, reason });
}

/**
 * Register a new MCP server (M06 Stage E → Stage C `mcp_add_server`).
 * `config` is the schema-generated {@link McpServerConfig}; `auth` is
 * the optional per-server secret (null for unauthenticated servers).
 * Errors surface as the Tauri `CmdError` shape — render via
 * {@link unwrapCmdError}.
 */
export async function mcpAddServer(config: McpServerConfig, auth: string | null): Promise<void> {
  await invoke('mcp_add_server', { config, auth });
}

/**
 * Remove a registered MCP server by name (Stage C `mcp_remove_server`).
 */
export async function mcpRemoveServer(name: string): Promise<void> {
  await invoke('mcp_remove_server', { name });
}

/**
 * Test a server connection without persisting (Stage C
 * `mcp_test_connection`). Takes the full {@link McpServerConfig} — the
 * Stage C command connects + `list_tools` + disconnects from the config
 * directly (it does NOT take a server name; the E.3.4 phase-doc
 * pseudocode drifted — reconciled against `commands.rs:821`).
 */
export async function mcpTestConnection(config: McpServerConfig): Promise<McpTool[]> {
  return await invoke<McpTool[]>('mcp_test_connection', { config });
}

/**
 * List registered MCP servers + their current state (Stage C
 * `mcp_list_servers`).
 */
export async function mcpListServers(): Promise<McpServerSummary[]> {
  return await invoke<McpServerSummary[]>('mcp_list_servers');
}

/**
 * List a *registered* MCP server's tools by name (M09.C
 * `mcp_list_server_tools`). Read-only — resolves the server through the
 * registry + lists its tools (the `mcpTestConnection` `Vec<McpTool>`
 * bridge, but keyed on an installed server name rather than an inline
 * config). The Palette's Tools tab fetches this per `mcpListServers`
 * entry to surface `source:'mcp'` items.
 */
export async function mcpListServerTools(name: string): Promise<McpTool[]> {
  return await invoke<McpTool[]>('mcp_list_server_tools', { name });
}

/** L3 sandbox report — rides nested inside {@link ImportOutcome}. */
export interface L3ReportWire {
  report_id: string;
  passed: boolean;
  reasons: string[];
}

/**
 * Outcome of `import_artifact` / `complete_import_artifact` (M07.5 /
 * ADR-0017 — the install-after-confirm split that closes M07.V 🔴 #1).
 * Discriminated on `status`, hand-mirrored from the serde
 * `#[serde(tag = "status")]` enum in
 * `src-tauri/src/commands.rs::ImportOutcome` (the `McpTool` /
 * `ResumePlan` precedent — not schema-generated; the `src-tauri`
 * `import_outcome_*` in-source tests pin the JSON keys this union
 * mirrors).
 *
 * `pending` — Novice: the backend ran the pipeline through the
 * tier-gate and is HOLDING; nothing is installed or locked. Carry
 * `pending_review_id` to `completeImportArtifact` / `cancelPendingImport`.
 * `installed` — terminal: installed + hash-locked (Promoted L4
 * auto-accept, or a completed Novice review). Keys are snake_case
 * (serde default). `share_provenance` is `null` when unexported; the
 * panel renders the "No provenance" state from `null` (ADR-0005).
 */
export type ImportOutcome =
  | {
      status: 'pending';
      pending_review_id: string;
      lock_key: string;
      capabilities: string[];
      l3_report: L3ReportWire;
      requires_secrets: string[];
      /** ADR-0005 trust block when present; `null` when unexported. */
      share_provenance: unknown;
    }
  | {
      status: 'installed';
      lock_key: string;
      capabilities: string[];
      l3_report: L3ReportWire;
      requires_secrets: string[];
      /** ADR-0005 trust block when present; `null` when unexported. */
      share_provenance: unknown;
    };

/** `ImportSource::Url` vs `ImportSource::File` — the shipped wire. */
export type ImportSourceKind = 'url' | 'file';

/** `ArtifactKind` — the shipped `import_artifact` command's third arg. */
export type ImportArtifactKind = 'skill' | 'tool' | 'agent' | 'mcp_server';

/**
 * Import an artifact (skill / tool / agent / MCP-server config) by raw
 * URL or local file path — M07 Stage C `import_artifact`. Params are
 * PINNED to the SHIPPED command signature at `src-tauri/src/commands.rs`
 * (three flat camelCased args; Tauri auto-converts snake_case Rust args
 * to camelCase JS keys), NOT the phase-doc-assumed `{ src, kind }` —
 * the v1.8 `<wire_signature_audit>` drift caught at Stage E authoring.
 */
export async function importArtifact(
  sourceKind: ImportSourceKind,
  location: string,
  artifactKind: ImportArtifactKind,
): Promise<ImportOutcome> {
  return await invoke<ImportOutcome>('import_artifact', {
    sourceKind,
    location,
    artifactKind,
  });
}

/**
 * Confirm a Novice import at the tier-gate review (M07.5 / ADR-0017).
 * Runs the install half the backend held back; resolves to the terminal
 * `installed` outcome. `pendingReviewId` is PINNED to the A.fix-shipped
 * `complete_import_artifact` command param (Tauri auto-converts the
 * snake_case Rust `pending_review_id`).
 */
export async function completeImportArtifact(pendingReviewId: string): Promise<ImportOutcome> {
  return await invoke<ImportOutcome>('complete_import_artifact', { pendingReviewId });
}

/**
 * Reject a Novice import at the tier-gate review (M07.5 / ADR-0017).
 * Drops the held pending import — because the install half never ran,
 * nothing is rolled back: no `skills.lock` entry and no MCP registry row
 * was ever written (the M07.V 🔴 #1 fix). Idempotent. `pendingReviewId`
 * is PINNED to the A.fix-shipped `cancel_pending_import` command param.
 */
export async function cancelPendingImport(pendingReviewId: string): Promise<void> {
  await invoke('cancel_pending_import', { pendingReviewId });
}

/**
 * One installed / imported artifact row. Mirrors the serde shape of
 * `runtime_main::builder::InstalledArtifact` (M08 Stage B — NOT
 * schema-generated; the struct crosses the Tauri bridge as-is, the
 * `McpServerSummary` / `McpTool` precedent). `list_installed_artifacts`
 * returns `InstalledArtifact[]`. The field set is PINNED to the Stage
 * B-shipped struct: `{ key, kind, source, installed_at }`.
 */
export interface InstalledArtifact {
  /** The `name@version` skills.lock key. */
  key: string;
  /** Artifact kind — the skills.lock `ArtifactKind`. */
  kind: 'skill' | 'tool' | 'agent' | 'mcp_server';
  /**
   * Where the artifact was imported from (the lock entry's `Source` — a
   * typify `oneOf`). Opaque here; the Palette keys items on `kind`.
   */
  source: unknown;
  /** RFC-3339 install timestamp (from the lock entry's `installed_at`). */
  installed_at: string;
}

/**
 * List artifacts recorded in `skills.lock` — M08 Stage B's
 * `list_installed_artifacts`, the first production `skills.lock` reader.
 *
 * The command takes ZERO JS args: the Tauri shell resolves
 * `<app_local_data_dir>/skills.lock` internally (wire PINNED to the
 * shipped Stage B signature `list_installed_artifacts(app: AppHandle)`).
 * An absent lock resolves to `[]` (Stage B: absent → empty, not an
 * error). The Palette + the ImportPanel call this on mount so installed
 * artifacts survive an app restart (M07-IRL #6).
 */
export async function listInstalledArtifacts(): Promise<InstalledArtifact[]> {
  const result = await invoke<InstalledArtifact[]>('list_installed_artifacts');
  // The command always resolves an array (Stage B — Result<Vec<_>>);
  // coerce defensively so a malformed bridge payload cannot crash a
  // consumer's `.length` / `.filter` / `.map`.
  return Array.isArray(result) ? result : [];
}

/**
 * One validation problem keyed to the offending node / JSON-path.
 * Mirrors `runtime_main::builder::NodeError` (M08 Stage B).
 */
export interface NodeError {
  /**
   * JSON-path or node id the error attaches to (`(root)` for a
   * whole-document schema-shape failure).
   */
  node_path: string;
  message: string;
}

/**
 * One Agent→Agent (`spawns`) edge's §8.security L2a narrowing decision.
 * Mirrors the serde shape of `runtime_main::builder::SpawnEdgeNarrowing`
 * (M08 Stage B — hand-mirrored, the `McpServerSummary` precedent; the
 * struct derives `serde::Serialize` and crosses the Tauri bridge as-is).
 *
 * The decision is computed by `capability/narrowing.rs::narrow()` in the
 * Rust main process (M05.B). Spec §9 forbids a second copy of that
 * intersection in TS — D2 SURFACES this record, it never recomputes it.
 *
 * `narrowed_caps` is a serde-serialized Rust `Result`, externally
 * tagged: `{ Ok: [...] }` carries the surviving set (L2a is
 * all-or-nothing — `Ok` is the child's declared set verbatim, there is
 * no partial clamp), `{ Err: "..." }` names the capability the parent
 * does not hold (Stage B folds that `Err` into `capability_errors`).
 */
export interface SpawnEdgeNarrowing {
  parent_id: string;
  child_id: string;
  parent_caps: CapabilityDeclaration[];
  child_declared_caps: CapabilityDeclaration[];
  narrowed_caps: { Ok: CapabilityDeclaration[] } | { Err: string };
}

/**
 * The whole-framework capability picture (spec Phase 9 Inspector).
 * Mirrors the serde shape of
 * `runtime_main::builder::FrameworkCapabilitySummary` (M08 Stage B).
 * Rides on `FrameworkValidationReport.capability_summary`; D2 reads its
 * `spawn_edges` for the Agent→Agent narrowing notice, and Stage E reads
 * the whole-framework totals — both off the one report, no separate
 * command.
 */
export interface FrameworkCapabilitySummary {
  files_read: string[];
  files_written: string[];
  network_hosts: string[];
  any_shell: boolean;
  spawn_edges: SpawnEdgeNarrowing[];
}

/**
 * The structured framework-validation report. Mirrors the serde shape of
 * `runtime_main::builder::FrameworkValidationReport` (M08 Stage B —
 * hand-mirrored, the `McpServerSummary` precedent). Stage B's
 * `validate_framework` command returns it; D2's continuous validation
 * and E's Validate button consume it. `capability_summary` rides on this
 * report (Stage B B.3.4) and is `null` when schema validation fails (no
 * parsed framework to summarize).
 */
export interface FrameworkValidationReport {
  schema_errors: NodeError[];
  capability_errors: NodeError[];
  ok: boolean;
  capability_summary: FrameworkCapabilitySummary | null;
}

/**
 * Validate an in-progress framework document against the schema-derived
 * types + the capability primitive — M08 Stage B's `validate_framework`
 * command. The report is keyed to offending node paths; D2's continuous
 * debounced pass and Stage E's explicit Validate button both call this
 * (one Rust validator, two triggers — spec §9, no TS duplication). `doc`
 * is the canvas's serialized `framework.json` candidate; it MAY be
 * invalid — that is the point of continuous validation. The command
 * returns the report synchronously (Stage B's §12-owned wire decision —
 * the `import_artifact` precedent).
 */
export async function validateFramework(doc: unknown): Promise<FrameworkValidationReport> {
  return await invoke<FrameworkValidationReport>('validate_framework', { doc });
}

/**
 * One inline-defined artifact's companion markdown file. Mirrors the
 * serde shape of `runtime_main::builder::Companion`
 * (`crates/runtime-main/src/builder/persist.rs` — M08 Stage B;
 * hand-mirrored, the `McpServerSummary` precedent). It crosses the
 * Tauri bridge both ways: a `save_framework` argument and a
 * `load_framework` return field.
 */
export interface Companion {
  /** File name relative to the framework directory (e.g.
   *  `summarize.skill.md`). */
  file_name: string;
  /** Full markdown body (frontmatter + content), written verbatim. */
  body: string;
}

/**
 * A framework reloaded from disk — Stage B's `load_framework` return.
 * Mirrors the serde shape of `runtime_main::builder::LoadedFramework`
 * (M08 Stage B). The renderer feeds `framework` to
 * `builderStore.replaceFramework`; the canvas re-derives (ADR-0020).
 */
export interface LoadedFramework {
  /** The parsed `framework.json`. */
  framework: Framework;
  /** The companion `.md` files found alongside `framework.json`. */
  companions: Companion[];
}

/**
 * Write `framework.json` + companion `.md` files to `dir` — M08 Stage B
 * `save_framework`. Params are PINNED to the SHIPPED command signature
 * `save_framework(dir, framework, companions)` at
 * `src-tauri/src/commands.rs` — NOT the phase-doc-assumed `{ dir, fw }`
 * (the v1.8 `<wire_signature_audit>` drift caught at Stage E authoring;
 * the `importArtifact` / `mcpTestConnection` reconciliations above).
 *
 * `dir` is the directory the `@tauri-apps/plugin-dialog` picker
 * returned (Stage C); the backend persistence is path-agnostic
 * (CLAUDE.md §9). `companions` defaults to `[]` — the v0.1 canvas
 * authors no inline markdown bodies (M09's Generators will). Errors
 * surface as the Tauri `CmdError` shape — render via
 * {@link unwrapCmdError}.
 */
export async function saveFramework(
  dir: string,
  framework: Framework,
  companions: Companion[] = [],
): Promise<void> {
  await invoke('save_framework', { dir, framework, companions });
}

/**
 * Read `framework.json` + its companion `.md` files from `dir` — M08
 * Stage B `load_framework`. Param PINNED to the shipped signature
 * `load_framework(dir)`. A save→load→save cycle is byte-stable (Stage
 * B B.3.2). Errors surface as the Tauri `CmdError` shape — render via
 * {@link unwrapCmdError}.
 */
export async function loadFramework(dir: string): Promise<LoadedFramework> {
  return await invoke<LoadedFramework>('load_framework', { dir });
}

/**
 * Mirrors serde's wire form for a Rust `std::time::Duration` — a
 * `#[derive(Serialize)]` struct field of type `Duration` crosses the
 * Tauri bridge as `{ secs, nanos }`, NOT a bare millisecond count.
 * Rides on {@link TestOutcome.timing}.
 */
export interface WireDuration {
  /** Whole seconds. */
  secs: number;
  /** Sub-second nanoseconds (0–999_999_999). */
  nanos: number;
}

/**
 * One §8.security L2 capability violation observed in a Tester run.
 * Mirrors the serde shape of `runtime_main::builder::CapabilityFailure`
 * (M08 Stage F1 — NOT schema-generated; the struct crosses the Tauri
 * bridge as-is, the `McpTool` / `McpServerSummary` precedent). F2
 * renders each as a test-failure line, never a HITL prompt (spec
 * Phase 9; F1.3.3).
 */
export interface CapabilityFailure {
  /** The runtime agent id that attempted the denied action. */
  agent_id: string;
  /** The capability that was missing/denied (human-readable). */
  needed: string;
  /** The enforcer's reason string. */
  reason: string;
}

/**
 * One §8.security L4 tier block observed in a Tester run — an action the
 * user's tier forbade. Mirrors the serde shape of
 * `runtime_main::builder::TierBlock` (M08.9.A — producer-driven mirror,
 * gotcha #94; the `CapabilityFailure` precedent). A tier block is NOT a
 * framework defect (ADR-0030); it drives the `tier_limited` verdict.
 */
export interface TierBlock {
  /** The runtime agent id whose dispatch the L4 tier gate rejected. */
  agent_id: string;
  /** The coarse capability kind the tier excluded (e.g. `write`). */
  kind: string;
  /** Plain-English description of what the agent attempted. */
  attempted_action: string;
}

/**
 * The Tester's truthful, UI-facing verdict — mirrors the `snake_case`
 * serde of `runtime_main::builder::TestVerdict` (M08.9.A). Distinct from
 * {@link TestOutcome.passed}: a `tier_limited` run has `passed === true`
 * but must NOT read as a clean PASS (TD-047 / ADR-0030).
 */
export type TestVerdict = 'pass' | 'fail' | 'tier_limited';

/**
 * Token in / out / total for a Tester run. Mirrors
 * `runtime_main::builder::TokenSpend` (M08 Stage F1).
 */
export interface TokenSpend {
  /** Input tokens summed across the run. */
  input: number;
  /** Output tokens summed across the run. */
  output: number;
  /** `input + output`. */
  total: number;
}

/**
 * The result of one Tester run. Mirrors the serde shape of
 * `runtime_main::builder::TestOutcome` (M08 Stage F1 — hand-mirrored,
 * the `McpTool` / `McpServerSummary` precedent; not schema-generated).
 * `test_framework` returns it; F2's modal renders every field.
 *
 * `passed === false` covers BOTH a capability violation / integrity
 * block AND a clean run the user judged wrong — a failed test is never
 * a thrown error (those are infrastructure-only).
 */
export interface TestOutcome {
  /**
   * Whether the run completed with no capability failure / integrity
   * block. A *tier* block does NOT flip this to `false` (tier ≠ defect;
   * ADR-0030) — the UI-facing distinction lives on {@link verdict}.
   */
  passed: boolean;
  /**
   * The truthful, 3-state verdict (`pass` / `fail` / `tier_limited`). The
   * modal renders this instead of the binary `passed`: a tier-blocked run
   * reads TIER-LIMITED, never a clean PASS (TD-047).
   */
  verdict: TestVerdict;
  /**
   * §8.security L2 violations observed during the run. F2 surfaces these
   * as test failures, never as live HITL prompts. Non-empty ⇒ `passed`
   * is `false`.
   */
  capability_failures: CapabilityFailure[];
  /**
   * §8.security L4 tier blocks observed during the run — actions the
   * user's tier forbade. Non-empty drives the `tier_limited` verdict but
   * does NOT force `passed === false` (the framework is fine; ADR-0030).
   */
  tier_blocks: TierBlock[];
  /** Token spend for the run (in / out / total). */
  token_spend: TokenSpend;
  /** Wall-clock duration — serde's Duration shape (see {@link WireDuration}). */
  timing: WireDuration;
  /** The Verification & Decision Record the run produced, or `null`. */
  vdr: unknown;
  /**
   * The full ordered `AgentEvent` trace — F2 reduces it into the scoped
   * test-session graph after the run resolves.
   */
  trace: AgentEvent[];
}

/**
 * Run the Builder's Tester against a candidate framework — M08 Stage F1
 * `test_framework`. The candidate `framework` crosses the wire straight
 * from the canvas (spec Phase 9 — no disk round-trip). Params are
 * PINNED to the SHIPPED F1 command signature
 * `test_framework(app, framework_doc, task)` at
 * `src-tauri/src/commands.rs` — Tauri auto-converts the snake_case Rust
 * `framework_doc` to the camelCase JS key `frameworkDoc` (the
 * `importArtifact` precedent).
 *
 * A *failed test* resolves `TestOutcome { passed: false, .. }`; only an
 * infrastructure failure (drone spawn / temp-DB setup) throws a
 * `CmdError` — render it via {@link unwrapCmdError}.
 */
export async function testFramework(framework: Framework, task: string): Promise<TestOutcome> {
  return await invoke<TestOutcome>('test_framework', { frameworkDoc: framework, task });
}

/**
 * Open the native file picker for a local artifact file (M07.V 🟡 #4 —
 * `@tauri-apps/plugin-dialog`). Returns the chosen absolute path, or
 * `null` when the user cancels — a cancel is a normal user action, not
 * an error, so the caller short-circuits on `null`. The caller passes
 * the path to `importArtifact('file', path, kind)`; the backend already
 * accepts `ImportSource::File` — only this renderer surface was missing.
 */
export async function pickLocalArtifactFile(): Promise<string | null> {
  const picked = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'Artifact', extensions: ['json', 'md'] }],
  });
  return typeof picked === 'string' ? picked : null;
}

export async function subscribeAgentEvents(
  handler: (event: AgentEvent) => void,
): Promise<UnlistenFn> {
  return listen<AgentEvent>('agent_event', (e) => handler(e.payload));
}

/**
 * Type guard: `value` matches the generated `CmdError` discriminator.
 *
 * The generated `CmdError` (typify-emitted from `schemas/error.v1.json`,
 * paralleled by `src/types/error.ts`) is a discriminated union over a
 * `type` literal: `'setup_required' | 'provider' | 'drone' | 'key_store'
 * | 'internal'`. Only `setup_required` is a unit variant; the rest carry
 * a `message: string` body.
 */
function isCmdError(value: unknown): value is CmdError {
  if (value === null || typeof value !== 'object') return false;
  const t = (value as { type?: unknown }).type;
  return (
    t === 'setup_required' ||
    t === 'provider' ||
    t === 'drone' ||
    t === 'key_store' ||
    t === 'internal'
  );
}

/**
 * Unwrap an unknown error thrown across the Tauri bridge into a
 * user-facing string. Replaces `String(e)` which yields "[object Object]"
 * for serde-tagged enums (M02 Stage E friction; gotcha #30).
 *
 * Order of preference:
 * 1. `Error` instances → `e.message`
 * 2. Generated `CmdError` discriminated-union shape → user-friendly
 *    message per variant (M04 Stage A2: consumes `src/types/error.ts`
 *    rather than the M02 hand-maintained interface)
 * 3. Plain object with `message` field → `String(obj.message)`
 * 4. Fallback → `String(e)` (last-resort; preserves prior behavior)
 *
 * The helper is exported so future renderer surfaces (M03 graph,
 * M04+ command surfaces) reuse it instead of re-implementing.
 */
export function unwrapCmdError(e: unknown): string {
  if (e instanceof Error) {
    return e.message;
  }
  if (isCmdError(e)) {
    if (e.type === 'setup_required') {
      return 'API key not set. Click "Save key" to set it (it stores in the OS keychain).';
    }
    // Every non-unit variant carries a `message: ErrorMessage` (the
    // typify-generated `ErrorMessage` newtype validates `minLength: 1`,
    // so the string is always present and non-empty).
    return `${e.type}: ${e.message}`;
  }
  if (e !== null && typeof e === 'object') {
    const obj = e as Record<string, unknown>;
    const m = typeof obj.message === 'string' ? obj.message : undefined;
    if (m !== undefined && m.length > 0) {
      return m;
    }
  }
  return String(e);
}
