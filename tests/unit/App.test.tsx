import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

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

import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { App } from '../../src/App';
import { useGraphStore } from '../../src/lib/graphStore';
import type { AgentEvent } from '../../src/types/agent_event';

describe('App (renderer-level state machine)', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
    listenMock.mockImplementation(async () => () => undefined);
    invokeMock.mockResolvedValue(undefined);
    useGraphStore.getState().clear();
  });

  afterEach(() => {
    useGraphStore.getState().clear();
  });

  it('save_key_then_run_smoke_drives_AgentNode_into_graph_store', async () => {
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

    await user.type(screen.getByLabelText(/anthropic api key/i), 'sk-ant-fixture');
    await user.click(screen.getByRole('button', { name: /save key/i }));
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

    // Simulate the runtime emitting the canonical M02 happy-path
    // sequence with the Stage C addition: session_start lands first
    // and spawns the FrameworkNode root that the AgentNode hangs off.
    const sequence: AgentEvent[] = [
      { type: 'session_start', session_id: 's1', framework: 'aria', model: 'haiku' },
      {
        type: 'agent_spawned',
        agent_id: 'a1',
        agent_name: 'smoke',
        parent_id: null,
        session_id: 's1',
      },
      { type: 'stream_text', agent_id: 'a1', text: 'hi' },
      { type: 'agent_complete', agent_id: 'a1', result: 'hi there' },
    ];
    // Wrap in act() — registeredHandler() updates the Zustand store
    // synchronously, which triggers a React re-render in GraphCanvas;
    // RTL otherwise warns about state updates outside act().
    act(() => {
      for (const e of sequence) {
        registeredHandler!({ payload: e });
      }
    });

    await waitFor(() => {
      const fw = useGraphStore.getState().nodes.find((n) => n.id === 'framework:aria');
      expect(fw).toBeDefined();
      const node = useGraphStore.getState().nodes.find((n) => n.id === 'agent:a1');
      expect(node).toBeDefined();
      expect(node!.data).toMatchObject({ status: 'complete' });
    });
  });

  it('surfaces_command_error_via_error_paragraph', async () => {
    listenMock.mockImplementation(async () => () => undefined);
    invokeMock.mockReset();
    invokeMock.mockResolvedValueOnce(undefined).mockRejectedValueOnce(new Error('API key not set'));

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

  // ---- Stage D: InspectorPanel interaction (click → open → ESC → close)

  it('selecting_a_node_opens_inspector_panel_via_store', async () => {
    listenMock.mockImplementation(async () => () => undefined);
    render(<App />);
    // Drive selection via the store rather than simulating a React Flow
    // node click — the click → selectNode delegation is exercised by
    // GraphCanvas's onNodeClick prop, which happy-dom can't fully drive
    // (no zoom-pane / pointer-event simulation). The store-driven path
    // exercises the same Inspector subscription via Zustand selectors.
    act(() => {
      useGraphStore.getState().applyEvent({
        type: 'agent_spawned',
        agent_id: 'a1',
        agent_name: 'smoke',
        parent_id: null,
        session_id: 's1',
      });
      useGraphStore.getState().selectNode('agent:a1');
    });
    await waitFor(() => expect(screen.getByTestId('inspector-panel')).toBeInTheDocument());
  });

  it('escape_keydown_closes_inspector_panel', async () => {
    listenMock.mockImplementation(async () => () => undefined);
    render(<App />);
    act(() => {
      useGraphStore.getState().applyEvent({
        type: 'agent_spawned',
        agent_id: 'a1',
        agent_name: 'smoke',
        parent_id: null,
        session_id: 's1',
      });
      useGraphStore.getState().selectNode('agent:a1');
    });
    await waitFor(() => expect(screen.getByTestId('inspector-panel')).toBeInTheDocument());
    act(() => {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    });
    await waitFor(() => expect(screen.queryByTestId('inspector-panel')).toBeNull());
  });
});
