import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// Mock @tauri-apps/api modules at the test boundary so the IPC wrapper
// can be exercised in isolation. The wrappers are thin; the value of the
// tests is verifying the call shape (command name + arg shape) and the
// subscriber lifecycle (subscribe → emit → unsubscribe).

const invokeMock = vi.fn();
const listenMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: (...args: unknown[]) => listenMock(...args),
}));

import {
  invokeQuerySessionDb,
  invokeReplaySession,
  invokeRunSmokeSession,
  invokeSetApiKey,
  subscribeAgentEvents,
  unwrapCmdError,
} from '../../src/lib/ipc';
import type { AgentEvent } from '../../src/types/agent_event';

describe('ipc', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('invokeRunSmokeSession_calls_invoke_with_correct_command_name', async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    await invokeRunSmokeSession();
    expect(invokeMock).toHaveBeenCalledTimes(1);
    expect(invokeMock).toHaveBeenCalledWith('run_smoke_session');
  });

  it('invokeSetApiKey_passes_key_arg', async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    await invokeSetApiKey('sk-ant-test');
    expect(invokeMock).toHaveBeenCalledWith('set_api_key', { key: 'sk-ant-test' });
  });

  it('subscribeAgentEvents_returns_unlisten_fn', async () => {
    const unlisten = vi.fn();
    listenMock.mockResolvedValueOnce(unlisten);
    const handler = vi.fn();
    const result = await subscribeAgentEvents(handler);
    expect(result).toBe(unlisten);
    expect(listenMock).toHaveBeenCalledWith('agent_event', expect.any(Function));
  });

  it('subscribeAgentEvents_handler_called_on_emit', async () => {
    const unlisten = vi.fn();
    let registeredHandler: ((e: { payload: AgentEvent }) => void) | undefined;
    listenMock.mockImplementationOnce(
      async (_channel: string, cb: (e: { payload: AgentEvent }) => void): Promise<() => void> => {
        registeredHandler = cb;
        return unlisten;
      },
    );

    const handler = vi.fn();
    await subscribeAgentEvents(handler);
    expect(registeredHandler).toBeDefined();

    const event: AgentEvent = {
      type: 'stream_text',
      agent_id: 'a1',
      text: 'hi',
    };
    registeredHandler!({ payload: event });

    expect(handler).toHaveBeenCalledWith(event);
  });

  it('unsubscribe_stops_handler_calls', async () => {
    const unlisten = vi.fn();
    listenMock.mockResolvedValueOnce(unlisten);
    const handler = vi.fn();
    const stop = await subscribeAgentEvents(handler);
    expect(handler).not.toHaveBeenCalled();
    stop();
    expect(unlisten).toHaveBeenCalledTimes(1);
  });

  it('invokeQuerySessionDb_passes_sql_arg_and_returns_rows', async () => {
    invokeMock.mockResolvedValueOnce([{ id: 1 }, { id: 2 }]);
    const rows = await invokeQuerySessionDb('SELECT id FROM signals');
    expect(invokeMock).toHaveBeenCalledWith('query_session_db', {
      sql: 'SELECT id FROM signals',
    });
    expect(rows).toEqual([{ id: 1 }, { id: 2 }]);
  });

  it('invokeReplaySession_passes_session_id_arg', async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    await invokeReplaySession('s-xyz');
    expect(invokeMock).toHaveBeenCalledWith('replay_session', { sessionId: 's-xyz' });
  });

  // ── unwrapCmdError — M04 Stage A2 consumes generated CmdError shape. ──

  it('unwrapCmdError_setup_required_returns_user_facing_message', () => {
    // The unit variant has no `message` field; the helper substitutes a
    // user-actionable instruction so the renderer doesn't surface a bare
    // discriminator string. Wire shape from src/types/error.ts.
    const e = { type: 'setup_required' };
    const out = unwrapCmdError(e);
    expect(out).toContain('API key not set');
    expect(out).toContain('Save key');
  });

  it('unwrapCmdError_provider_renders_type_and_message', () => {
    // Generated CmdError tuple variants serialize as
    // {"type":"provider","message":"..."} per #[serde(content="message")].
    const e = { type: 'provider', message: 'auth failed' };
    expect(unwrapCmdError(e)).toBe('provider: auth failed');
  });

  it('unwrapCmdError_drone_renders_type_and_message', () => {
    const e = { type: 'drone', message: 'subprocess died' };
    expect(unwrapCmdError(e)).toBe('drone: subprocess died');
  });

  it('unwrapCmdError_key_store_renders_type_and_message', () => {
    const e = { type: 'key_store', message: 'keychain locked' };
    expect(unwrapCmdError(e)).toBe('key_store: keychain locked');
  });

  it('unwrapCmdError_internal_renders_type_and_message', () => {
    const e = { type: 'internal', message: 'channel closed' };
    expect(unwrapCmdError(e)).toBe('internal: channel closed');
  });

  it('unwrapCmdError_error_instance_returns_error_message', () => {
    // A raw JS Error (e.g., a network failure inside the @tauri-apps
    // bridge that didn't reach the CmdError path) should still surface
    // its `message` rather than collapsing to "[object Object]".
    const e = new Error('network unavailable');
    expect(unwrapCmdError(e)).toBe('network unavailable');
  });

  it('unwrapCmdError_unknown_object_with_message_returns_message', () => {
    // Compatibility path: arbitrary error-shape objects from elsewhere in
    // the renderer (not generated CmdError but with a `message` field).
    const e = { message: 'something else' };
    expect(unwrapCmdError(e)).toBe('something else');
  });

  it('unwrapCmdError_falls_back_to_string_for_unknown_types', () => {
    // Last-resort fallback preserves M02 Stage E behavior — anything
    // not matching the typed paths goes through String() so the user
    // sees *something* rather than nothing.
    expect(unwrapCmdError(42)).toBe('42');
    expect(unwrapCmdError(null)).toBe('null');
  });

  it('unwrapCmdError_object_without_recognized_type_or_message_falls_through', () => {
    // An object with neither a recognized `type` discriminator nor a
    // `message` field falls through to the last-resort String() branch.
    const e = { foo: 'bar' };
    expect(unwrapCmdError(e)).toBe('[object Object]');
  });
});
