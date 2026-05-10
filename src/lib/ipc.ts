import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { AgentEvent } from '../types/agent_event';
import type { CmdError } from '../types/error';

export async function invokeRunSmokeSession(): Promise<void> {
  await invoke('run_smoke_session');
}

export async function invokeSetApiKey(key: string): Promise<void> {
  await invoke('set_api_key', { key });
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
