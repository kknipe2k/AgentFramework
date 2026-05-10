import { useEffect, useState } from 'react';
import { ApprovalPanel } from './components/ApprovalPanel';
import { GraphCanvas } from './components/GraphCanvas';
import { InspectorPanel } from './components/InspectorPanel';
import { SetupPanel } from './components/SetupPanel';
import { SmokeButton } from './components/SmokeButton';
import { SqlInspector } from './components/SqlInspector';
import {
  invokeReplaySession,
  invokeRunSmokeSession,
  invokeSetApiKey,
  subscribeAgentEvents,
  unwrapCmdError,
} from './lib/ipc';
import { useGraphStore } from './lib/graphStore';
import './styles.css';

const LAST_SESSION_KEY = 'lastSessionId';

// Renderer-level Playwright affordance — exposes the Zustand store on
// `window.__graphStore` so `tests/e2e/plan_approval.spec.ts` can drive
// graph state without spinning up an SDK + Anthropic. Module mocking
// across the @tauri-apps/api ESM boundary doesn't work in Playwright
// (Vitest covers the click→invoke linkage); this affordance lets the
// E2E spec exercise the surface-on-state-change + dismiss-on-state-change
// flow that only renders correctly inside a real browser layout.
//
// Exported unconditionally rather than gated on `import.meta.env.DEV` —
// the store carries no secrets, the same data is already inspectable via
// React DevTools, and CLAUDE.md §9 anti-patterns calls out feature-flag
// shims that don't earn their cost.
declare global {
  interface Window {
    __graphStore?: typeof useGraphStore;
  }
}
if (typeof window !== 'undefined') {
  window.__graphStore = useGraphStore;
}

export function App(): JSX.Element {
  const [hasKey, setHasKey] = useState(false);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Replay-on-mount: if a previous session id was stashed in
    // localStorage by a prior session_start, ask main to read its
    // signal log and re-emit AgentEvents through the existing
    // `agent_event` channel. graphStore.applyEvent's idempotence
    // guarantees the reconstructed graph matches the original.
    const lastSessionId = localStorage.getItem(LAST_SESSION_KEY);
    if (lastSessionId !== null && lastSessionId.length > 0) {
      void invokeReplaySession(lastSessionId).catch((e) => {
        console.error('Replay session error:', e);
      });
    }

    const unsubscribePromise = subscribeAgentEvents((event) => {
      if (event.type === 'session_start') {
        localStorage.setItem(LAST_SESSION_KEY, event.session_id);
      }
      useGraphStore.getState().applyEvent(event);
      if (event.type === 'agent_complete' || event.type === 'agent_error') {
        setRunning(false);
      }
    });
    return () => {
      void unsubscribePromise.then((unsub) => unsub());
    };
  }, []);

  async function handleSetKey(key: string): Promise<void> {
    try {
      await invokeSetApiKey(key);
      setHasKey(true);
    } catch (e) {
      console.error('Set API key error:', e);
      setError(unwrapCmdError(e));
    }
  }

  async function handleSmoke(): Promise<void> {
    useGraphStore.getState().clear();
    setRunning(true);
    setError(null);
    try {
      await invokeRunSmokeSession();
    } catch (e) {
      // Always log structured errors to DevTools console — `error` carries
      // the user-facing string but the full object is needed for
      // diagnostics (per docs/gotchas.md #29 keyring-stub case the renderer
      // alone showed "[object Object]" with no usable signal).
      console.error('Smoke test error:', e);
      setError(unwrapCmdError(e));
      setRunning(false);
    }
  }

  return (
    <main>
      <h1>Agent Runtime — M03 live graph</h1>
      <SetupPanel onSave={handleSetKey} />
      <SmokeButton disabled={!hasKey || running} onClick={handleSmoke} />
      {error && <p className="error">{error}</p>}
      <div className="graph-layout">
        <GraphCanvas />
        <InspectorPanel />
        <ApprovalPanel />
      </div>
      <SqlInspector />
    </main>
  );
}
