import { useEffect, useState } from 'react';
import { GraphCanvas } from './components/GraphCanvas';
import { InspectorPanel } from './components/InspectorPanel';
import { SetupPanel } from './components/SetupPanel';
import { SmokeButton } from './components/SmokeButton';
import {
  invokeRunSmokeSession,
  invokeSetApiKey,
  subscribeAgentEvents,
  unwrapCmdError,
} from './lib/ipc';
import { useGraphStore } from './lib/graphStore';
import './styles.css';

export function App(): JSX.Element {
  const [hasKey, setHasKey] = useState(false);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const unsubscribePromise = subscribeAgentEvents((event) => {
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
      </div>
    </main>
  );
}
