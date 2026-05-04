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

import { invokeRunSmokeSession, invokeSetApiKey, subscribeAgentEvents } from '../../src/lib/ipc';
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
});
