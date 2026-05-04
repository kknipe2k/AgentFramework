import { useEffect, useReducer, useState } from 'react';
import { initialState, reducer } from './lib/eventReducer';
import { invokeRunSmokeSession, invokeSetApiKey, subscribeAgentEvents } from './lib/ipc';
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
    await invokeSetApiKey(key);
    setHasKey(true);
  }

  async function handleSmoke(): Promise<void> {
    dispatch({ type: 'clear' });
    dispatch({ type: 'started' });
    try {
      await invokeRunSmokeSession();
    } catch (e) {
      dispatch({
        type: 'error',
        message: e instanceof Error ? e.message : String(e),
      });
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
