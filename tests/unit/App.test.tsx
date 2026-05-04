import { beforeEach, describe, expect, it, vi } from 'vitest';

// Hoisted mocks for the @tauri-apps/api modules so App's IPC layer
// resolves to test-controlled functions before App is imported.
const invokeMock = vi.fn(async (..._args: unknown[]) => undefined);
const listenMock =
  vi.fn<(channel: string, cb: (e: { payload: AgentEvent }) => void) => Promise<() => void>>();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));
vi.mock('@tauri-apps/api/event', () => ({
  listen: (channel: string, cb: (e: { payload: AgentEvent }) => void) => listenMock(channel, cb),
}));

import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { App } from '../../src/App';
import type { AgentEvent } from '../../src/types/agent_event';

describe('App (renderer-level state machine)', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
    // Default the listen mock to a no-op subscription so App's useEffect
    // resolves cleanly when a test doesn't override it.
    listenMock.mockImplementation(async () => () => undefined);
    invokeMock.mockResolvedValue(undefined);
  });

  it('save_key_then_run_smoke_renders_event_list', async () => {
    let registeredHandler: ((e: { payload: AgentEvent }) => void) | undefined;
    listenMock.mockImplementation(
      async (_channel: string, cb: (e: { payload: AgentEvent }) => void): Promise<() => void> => {
        registeredHandler = cb;
        return () => undefined;
      },
    );
    invokeMock.mockResolvedValue(undefined);

    const user = userEvent.setup();
    render(<App />);
    await waitFor(() => expect(registeredHandler).toBeDefined());

    // Save key first — Run button is disabled until the key save resolves.
    await user.type(screen.getByLabelText(/anthropic api key/i), 'sk-ant-fixture');
    await user.click(screen.getByRole('button', { name: /save key/i }));
    // Wait for the "stored in OS keychain" indicator — proves setHasKey(true)
    // has run and the React tree has re-rendered before we touch Run.
    await screen.findByLabelText(/saved/i);

    const runButton = await screen.findByRole('button', {
      name: /run smoke test/i,
    });
    await waitFor(() => expect(runButton).toBeEnabled());
    await user.click(runButton);
    await waitFor(
      () => {
        const calls = invokeMock.mock.calls.map((c) => String(c[0]));
        expect(calls).toContain('run_smoke_session');
      },
      { timeout: 3000 },
    );

    // Simulate the runtime emitting the canonical M02 happy-path sequence.
    const sequence: AgentEvent[] = [
      {
        type: 'agent_spawned',
        agent_id: 'a1',
        agent_name: 'smoke',
        parent_id: null,
        session_id: 's1',
      },
      { type: 'stream_text', agent_id: 'a1', text: 'hi' },
      { type: 'stream_text', agent_id: 'a1', text: ' there' },
      { type: 'agent_complete', agent_id: 'a1', result: 'hi there' },
    ];
    for (const e of sequence) {
      registeredHandler!({ payload: e });
    }

    await waitFor(() => expect(screen.getAllByRole('listitem').length).toBeGreaterThanOrEqual(4));
    const last = screen.getAllByRole('listitem').at(-1)!;
    expect(last).toHaveAttribute('data-event-type', 'agent_complete');
  });

  it('surfaces_command_error_via_error_paragraph', async () => {
    listenMock.mockImplementation(async () => () => undefined);
    invokeMock.mockReset();
    invokeMock
      .mockResolvedValueOnce(undefined) // set_api_key OK
      .mockRejectedValueOnce(new Error('API key not set'));

    const user = userEvent.setup();
    render(<App />);
    await user.type(screen.getByLabelText(/anthropic api key/i), 'sk-ant-fixture');
    await user.click(screen.getByRole('button', { name: /save key/i }));

    await waitFor(() =>
      expect(screen.getByRole('button', { name: /run smoke test/i })).toBeEnabled(),
    );
    await user.click(screen.getByRole('button', { name: /run smoke test/i }));

    const err = await screen.findByText(/API key not set/i);
    expect(err).toBeInTheDocument();
  });
});
