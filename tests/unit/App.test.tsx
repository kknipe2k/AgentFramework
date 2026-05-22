import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// Hoisted mocks for the @tauri-apps/api modules so App's IPC layer
// resolves to test-controlled functions before App is imported. The
// return is `unknown` so a per-command mockImplementation can resolve
// non-undefined values (e.g. has_api_key → boolean).
const invokeMock = vi.fn(async (..._args: unknown[]) => undefined as unknown);
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
    // Stage E added replay-on-mount that fires `invokeReplaySession` when
    // localStorage.lastSessionId is set. Clean it before each test so the
    // mount-time IPC call sequence is deterministic; tests that exercise
    // the replay path set the value explicitly.
    localStorage.removeItem('lastSessionId');
  });

  afterEach(() => {
    useGraphStore.getState().clear();
    localStorage.removeItem('lastSessionId');
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
    // Order-independent per command — the mount-time has_api_key probe
    // and the set_api_key save both resolve; only run_smoke_session
    // rejects (a `mockResolvedValueOnce` chain would be consumed by the
    // mount probe and mis-route the rejection).
    invokeMock.mockImplementation(async (...args: unknown[]) => {
      if (args[0] === 'run_smoke_session') {
        throw new Error('API key not set');
      }
      return undefined;
    });

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

  // ---- Stage E: replay-on-mount + session-id persistence

  it('mount_calls_replay_session_with_stored_lastSessionId', async () => {
    localStorage.setItem('lastSessionId', 'prior-session-xyz');
    listenMock.mockImplementation(async () => () => undefined);
    invokeMock.mockResolvedValue(undefined);
    render(<App />);
    await waitFor(() => {
      const calls = invokeMock.mock.calls.map((c) => [String(c[0]), c[1]]);
      expect(calls).toContainEqual(['replay_session', { sessionId: 'prior-session-xyz' }]);
    });
    localStorage.removeItem('lastSessionId');
  });

  it('session_start_event_persists_session_id_to_localStorage', async () => {
    let registeredHandler: ((e: { payload: AgentEvent }) => void) | undefined;
    listenMock.mockImplementation(
      async (_channel: string, cb: (e: { payload: AgentEvent }) => void): Promise<() => void> => {
        registeredHandler = cb;
        return () => undefined;
      },
    );
    localStorage.removeItem('lastSessionId');
    render(<App />);
    await waitFor(() => expect(registeredHandler).toBeDefined());
    act(() => {
      registeredHandler!({
        payload: {
          type: 'session_start',
          session_id: 'new-session-abc',
          framework: 'aria',
          model: 'haiku',
        },
      });
    });
    expect(localStorage.getItem('lastSessionId')).toBe('new-session-abc');
    localStorage.removeItem('lastSessionId');
  });

  // ---- M08.A: has_api_key startup read (M07-IRL #7) ----
  // A key entered once must survive an app restart. The root cause was
  // the absent startup read — App.tsx hardcoded `hasKey` false and only
  // flipped it inside handleSetKey. App now reads the keychain on mount.

  it('app_calls_has_api_key_on_mount_and_enables_smoke_button_when_present', async () => {
    listenMock.mockImplementation(async () => () => undefined);
    invokeMock.mockImplementation(async (...args: unknown[]) =>
      args[0] === 'has_api_key' ? true : undefined,
    );
    render(<App />);
    await waitFor(() => {
      const calls = invokeMock.mock.calls.map((c) => String(c[0]));
      expect(calls).toContain('has_api_key');
    });
    const runButton = await screen.findByRole('button', { name: /run smoke test/i });
    await waitFor(() => expect(runButton).toBeEnabled());
  });

  it('app_keeps_smoke_button_disabled_when_has_api_key_returns_false', async () => {
    listenMock.mockImplementation(async () => () => undefined);
    invokeMock.mockImplementation(async (...args: unknown[]) =>
      args[0] === 'has_api_key' ? false : undefined,
    );
    render(<App />);
    await waitFor(() => {
      const calls = invokeMock.mock.calls.map((c) => String(c[0]));
      expect(calls).toContain('has_api_key');
    });
    expect(screen.getByRole('button', { name: /run smoke test/i })).toBeDisabled();
  });

  // ---- M08.A: stale Test error banner (M06.5 IRL 🟡-3) ----
  // handleSmoke clears `error` at run start; the banner is bound to the
  // same App `error` slot and no racing handler re-sets it. A stale
  // banner from a prior failed run is gone before the next run begins.

  it('smoke_error_banner_cleared_when_new_run_starts', async () => {
    listenMock.mockImplementation(async () => () => undefined);
    let smokeCalls = 0;
    invokeMock.mockImplementation(async (...args: unknown[]) => {
      if (args[0] === 'run_smoke_session') {
        smokeCalls += 1;
        if (smokeCalls === 1) {
          throw new Error('first run boom');
        }
        return undefined;
      }
      return undefined;
    });
    const user = userEvent.setup();
    render(<App />);
    await user.type(screen.getByLabelText(/anthropic api key/i), 'sk-ant-fixture');
    await user.click(screen.getByRole('button', { name: /save key/i }));
    const runButton = await screen.findByRole('button', { name: /run smoke test/i });
    await waitFor(() => expect(runButton).toBeEnabled());

    // First run fails — the error banner surfaces.
    await user.click(runButton);
    expect(await screen.findByText(/first run boom/i)).toBeInTheDocument();

    // A subsequent run clears the stale banner at run start.
    await user.click(runButton);
    await waitFor(() => expect(screen.queryByText(/first run boom/i)).toBeNull());
  });
});
