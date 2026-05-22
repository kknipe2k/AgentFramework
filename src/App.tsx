import { useEffect, useState } from 'react';
import { ApprovalPanel } from './components/ApprovalPanel';
import { BudgetHeaderBar } from './components/BudgetHeaderBar';
import { BuilderShell } from './components/builder/BuilderShell';
import { ViewSwitch, type AppView } from './components/builder/ViewSwitch';
import { GapPanel } from './components/GapPanel';
import { GraphCanvas } from './components/GraphCanvas';
import { HITLModal } from './components/HITLModal';
import { HITLPanel } from './components/HITLPanel';
import { HITLToast } from './components/HITLToast';
import { ImportPanel } from './components/ImportPanel';
import { InspectorPanel } from './components/InspectorPanel';
import { MCPServerSettings } from './components/MCPServerSettings';
import { RecoveryDialog } from './components/RecoveryDialog';
import { SettingsPanel } from './components/SettingsPanel';
import { SetupPanel } from './components/SetupPanel';
import { SmokeButton } from './components/SmokeButton';
import { SqlInspector } from './components/SqlInspector';
import { UncertaintyPrompt } from './components/UncertaintyPrompt';
import {
  invokeHasApiKey,
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

interface RuntimeLayoutProps {
  hasKey: boolean;
  running: boolean;
  error: string | null;
  onSetKey: (key: string) => Promise<void>;
  onSmoke: () => Promise<void>;
  lastSessionId: string | null;
}

// RuntimeLayout — the live-execution view (SetupPanel + SmokeButton +
// the `graph-layout` panels + the modal/dialog overlays). Extracted
// verbatim from App's return so the Runtime view is a clean unit and the
// M08.C Builder view is a sibling, not a rewrite. Behavior is unchanged:
// App still owns the hasKey / running / error state, the
// subscribeAgentEvents effect, and the replay-on-mount.
function RuntimeLayout({
  hasKey,
  running,
  error,
  onSetKey,
  onSmoke,
  lastSessionId,
}: RuntimeLayoutProps): JSX.Element {
  return (
    <>
      <SetupPanel onSave={onSetKey} />
      <SmokeButton disabled={!hasKey || running} onClick={onSmoke} />
      {error && <p className="error">{error}</p>}
      <div className="graph-layout">
        <GraphCanvas />
        <InspectorPanel />
        <ApprovalPanel />
        <HITLPanel />
        <GapPanel />
        <MCPServerSettings />
        <ImportPanel />
      </div>
      <HITLModal />
      <HITLToast />
      <RecoveryDialog />
      <UncertaintyPrompt sessionId={lastSessionId ?? ''} />
      <SqlInspector />
    </>
  );
}

export function App(): JSX.Element {
  const [view, setView] = useState<AppView>('runtime');
  const [hasKey, setHasKey] = useState(false);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // M07-IRL #7: seed `hasKey` from the OS keychain so a key entered in
    // a prior session survives an app restart. The root cause of the
    // finding was the absent startup read — `hasKey` was hardcoded false
    // and only flipped inside handleSetKey.
    void invokeHasApiKey()
      .then((present) => setHasKey(present))
      .catch((e) => {
        console.error('has_api_key error:', e);
      });

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

  const lastSessionId =
    typeof localStorage !== 'undefined' ? localStorage.getItem(LAST_SESSION_KEY) : null;

  return (
    <main>
      <BudgetHeaderBar />
      <h1>Agent Runtime — M03 live graph</h1>
      <ViewSwitch value={view} onChange={setView} />
      <SettingsPanel />
      {view === 'runtime' ? (
        <RuntimeLayout
          hasKey={hasKey}
          running={running}
          error={error}
          onSetKey={handleSetKey}
          onSmoke={handleSmoke}
          lastSessionId={lastSessionId}
        />
      ) : (
        <BuilderShell />
      )}
    </main>
  );
}
