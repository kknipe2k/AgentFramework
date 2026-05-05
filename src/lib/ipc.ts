import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { AgentEvent } from '../types/agent_event';

export async function invokeRunSmokeSession(): Promise<void> {
  await invoke('run_smoke_session');
}

export async function invokeSetApiKey(key: string): Promise<void> {
  await invoke('set_api_key', { key });
}

export async function subscribeAgentEvents(
  handler: (event: AgentEvent) => void,
): Promise<UnlistenFn> {
  return listen<AgentEvent>('agent_event', (e) => handler(e.payload));
}

/**
 * Wire shape of `CmdError` from `src-tauri/src/commands.rs` —
 * `serde(tag = "type", rename_all = "snake_case")` puts the variant name in
 * `type` and (for struct variants) the human message in `message`.
 *
 * Variants (per M02 Stage E):
 * - `setup_required` — unit; no `message` (action: prompt user to set API key)
 * - `provider`       — has `message`
 * - `drone`          — has `message`
 * - `key_store`      — has `message`
 * - `internal`       — has `message`
 */
export interface CmdError {
  type:
    | 'setup_required'
    | 'provider'
    | 'drone'
    | 'key_store'
    | 'internal';
  message?: string;
}

/**
 * Unwrap an unknown error thrown across the Tauri bridge into a
 * user-facing string. Replaces `String(e)` which yields "[object Object]"
 * for serde-tagged enums (M02 Stage E friction r-?, fixed in this PR).
 *
 * Order of preference:
 * 1. `Error` instances → `e.message`
 * 2. `CmdError` shape (object with `type`, optional `message`) →
 *    user-friendly message per variant
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
  if (e !== null && typeof e === 'object') {
    const obj = e as Record<string, unknown>;
    const t = typeof obj.type === 'string' ? obj.type : undefined;
    const m = typeof obj.message === 'string' ? obj.message : undefined;
    if (t === 'setup_required') {
      return 'API key not set. Click "Save key" to set it (it stores in the OS keychain).';
    }
    if (m !== undefined && m.length > 0) {
      return t !== undefined ? `${t}: ${m}` : m;
    }
    if (t !== undefined) {
      return t;
    }
  }
  return String(e);
}
