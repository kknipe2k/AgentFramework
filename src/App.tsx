import { useEffect, useReducer, useState } from 'react';
import { initialState, reducer } from './lib/eventReducer';
import {
  invokeRunSmokeSession,
  invokeSetApiKey,
  subscribeAgentEvents,
  unwrapCmdError,
} from './lib/ipc';
import { EventList } from './components/EventList';
import { SetupPanel } from './components/SetupPanel';
import { SmokeButton } from './components/SmokeButton';
import './styles.css';

export function App(): JSX.Element {
  const [state, dispatch] = useReducer(reducer, initialState);
  const [hasKey, setHasKey] = useState(false);

  useEffect(() => {
    const unsubscribePromise = subscribeAgentEvents((event) => {
      dispatch({ type: 'event_received', event });
      if (event.type === 'agent_complete' || event.type === 'agent_error') {
        dispatch({ type: 'completed' });
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
      dispatch({ type: 'error', message: unwrapCmdError(e) });
    }
  }

  async function handleSmoke(): Promise<void> {
    dispatch({ type: 'clear' });
    dispatch({ type: 'started' });
    try {
      await invokeRunSmokeSession();
    } catch (e) {
      // Always log structured errors to DevTools console — `state.error`
      // carries the user-facing string but the full object is needed for
      // diagnostics (per docs/gotchas.md #29 keyring-stub case the renderer
      // alone showed "[object Object]" with no usable signal).
      console.error('Smoke test error:', e);
      dispatch({ type: 'error', message: unwrapCmdError(e) });
    }
  }

  return (
    <main>
      <h1>Agent Runtime — M02 smoke</h1>
      <SetupPanel onSave={handleSetKey} />
      <SmokeButton disabled={!hasKey || state.running} onClick={handleSmoke} />
      {state.error && <p className="error">{state.error}</p>}
      <EventList events={state.events} />
    </main>
  );
}
