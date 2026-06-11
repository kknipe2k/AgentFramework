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
import { useToastStore } from '../../src/lib/toastStore';
import type { AgentEvent } from '../../src/types/agent_event';

// The serde wire form of `CmdError::SetupRequired` — the unit-variant
// case serializes to `{"type":"setup_required"}` with no message key
// (pinned by commands.rs::cmd_error_setup_required_serializes_with_type_tag_only).
// Typed `unknown` because that IS the contract: the Tauri bridge rejects
// with a plain tagged object, never an `Error` instance.
const SETUP_REQUIRED_REJECTION: unknown = { type: 'setup_required' };

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

  // ---- M08.8.C.fix: tier display seed (#19) + tier-change toast (#20) ----
  // The renderer never seeded currentTier from the backend: it defaulted
  // 'novice' and was written ONLY by the tier_transition reducer. After a
  // restart with a Promoted backend the Settings display read Novice while
  // the run enforced Promoted (the #19 desync). App now reads
  // get_current_tier on mount and seeds the store; a tier_transition event
  // pushes a feedback toast (#20).

  it('app_seeds_current_tier_from_get_current_tier_on_mount', async () => {
    // Force a clean Novice baseline so the seed-to-Promoted flip is the
    // observed effect (clear() preserves currentTier — a prior test could
    // leave it promoted).
    act(() => {
      useGraphStore.setState({ currentTier: 'novice' });
    });
    listenMock.mockImplementation(async () => () => undefined);
    invokeMock.mockImplementation(async (...args: unknown[]) =>
      args[0] === 'get_current_tier' ? 'promoted' : undefined,
    );
    render(<App />);
    await waitFor(() => {
      const calls = invokeMock.mock.calls.map((c) => String(c[0]));
      expect(calls).toContain('get_current_tier');
    });
    // The seed reflects the enforced tier — without it the store stays at
    // its 'novice' default (the #19 desync).
    await waitFor(() => expect(useGraphStore.getState().currentTier).toBe('promoted'));
  });

  it('tier_transition_event_pushes_a_feedback_toast', async () => {
    act(() => {
      useToastStore.setState({ toasts: [] });
    });
    let registeredHandler: ((e: { payload: AgentEvent }) => void) | undefined;
    listenMock.mockImplementation(
      async (_channel: string, cb: (e: { payload: AgentEvent }) => void): Promise<() => void> => {
        registeredHandler = cb;
        return () => undefined;
      },
    );
    invokeMock.mockResolvedValue(undefined);
    render(<App />);
    await waitFor(() => expect(registeredHandler).toBeDefined());

    act(() => {
      registeredHandler!({
        payload: {
          type: 'tier_transition',
          previous: 'novice',
          current: 'promoted',
          reason: 'user requested promoted via Settings',
        },
      });
    });

    // DESIGN.md principle 1 (feedback): the tier change surfaces a toast
    // that names the new tier.
    await waitFor(() => {
      const toasts = useToastStore.getState().toasts;
      expect(toasts.length).toBeGreaterThan(0);
      expect(toasts.some((t) => t.title.toLowerCase().includes('promoted'))).toBe(true);
    });
    act(() => {
      useToastStore.setState({ toasts: [] });
    });
  });

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

  // ---- M09.5.F: honest key chip (maintainer-IRL truthful-labels) ----
  // The topbar key chip was seeded at mount + set in handleSetKey, but no
  // run handler ever updated it: a run failing SetupRequired (the run
  // loop's own read_api_key resolved no key) left the chip reading "key
  // active" beside the failure it contradicted — a DESIGN.md principle-8
  // violation (labels tell the truth).

  it('setup_required_run_failure_flips_key_chip_to_no_key', async () => {
    listenMock.mockImplementation(async () => () => undefined);
    invokeMock.mockImplementation(async (...args: unknown[]) => {
      if (args[0] === 'has_api_key') return true;
      if (args[0] === 'run_smoke_session') throw SETUP_REQUIRED_REJECTION;
      return undefined;
    });
    const user = userEvent.setup();
    render(<App />);
    const chip = screen.getByTestId('topbar-key-chip');
    await waitFor(() => expect(chip).toHaveTextContent('key active'));
    const runButton = screen.getByRole('button', { name: /run smoke test/i });
    await waitFor(() => expect(runButton).toBeEnabled());

    await user.click(runButton);

    // The run loop's own read just proved no key resolves — the chip
    // must flip honest, not stay green beside the failed run.
    await waitFor(() => expect(chip).toHaveTextContent('no key'));
  });

  it('setup_required_flip_is_not_overridden_by_a_true_has_api_key_repoll', async () => {
    // The rider-1 invariant (the vanish case): the keychain probe may
    // STILL report a key present while the run loop's own read resolved
    // none. The settled-run re-poll must be SKIPPED on a SetupRequired
    // failure — otherwise the async poll wins and recreates the exact
    // lie this stage kills (a failed-for-no-key run beside a green chip).
    listenMock.mockImplementation(async () => () => undefined);
    invokeMock.mockImplementation(async (...args: unknown[]) => {
      if (args[0] === 'has_api_key') return true;
      if (args[0] === 'run_smoke_session') throw SETUP_REQUIRED_REJECTION;
      return undefined;
    });
    const user = userEvent.setup();
    render(<App />);
    const chip = screen.getByTestId('topbar-key-chip');
    await waitFor(() => expect(chip).toHaveTextContent('key active'));
    const runButton = screen.getByRole('button', { name: /run smoke test/i });
    await waitFor(() => expect(runButton).toBeEnabled());

    await user.click(runButton);

    await waitFor(() => expect(chip).toHaveTextContent('no key'));
    // The authoritative flip was not chased by a re-poll: the mount seed
    // is the only has_api_key call on the wire.
    const hasKeyCalls = invokeMock.mock.calls.filter((c) => String(c[0]) === 'has_api_key');
    expect(hasKeyCalls).toHaveLength(1);
  });

  it('settled_run_repolls_has_api_key_and_the_chip_reflects_the_result', async () => {
    listenMock.mockImplementation(async () => () => undefined);
    let hasKeyProbes = 0;
    invokeMock.mockImplementation(async (...args: unknown[]) => {
      if (args[0] === 'has_api_key') {
        hasKeyProbes += 1;
        // Present at mount; gone by the post-run re-poll (the
        // out-of-band-change case the re-poll exists to surface).
        return hasKeyProbes === 1;
      }
      return undefined;
    });
    const user = userEvent.setup();
    render(<App />);
    const chip = screen.getByTestId('topbar-key-chip');
    await waitFor(() => expect(chip).toHaveTextContent('key active'));
    const runButton = screen.getByRole('button', { name: /run smoke test/i });
    await waitFor(() => expect(runButton).toBeEnabled());

    await user.click(runButton);

    // Every settled run re-polls has_api_key, so an out-of-band keychain
    // change surfaces on the chip instead of going stale-green until the
    // next app restart.
    await waitFor(() => expect(chip).toHaveTextContent('no key'));
    expect(hasKeyProbes).toBeGreaterThanOrEqual(2);
  });
});
