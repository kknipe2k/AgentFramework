import { useEffect, useState } from 'react';
import { ApprovalPanel } from './components/ApprovalPanel';
import { AppShell } from './components/AppShell';
import { BudgetHeaderBar } from './components/BudgetHeaderBar';
import { BuilderShell } from './components/builder/BuilderShell';
import { type AppView } from './components/builder/ViewSwitch';
import { GapPanel } from './components/GapPanel';
import { GraphCanvas } from './components/GraphCanvas';
import { HITLModal } from './components/HITLModal';
import { HITLPanel } from './components/HITLPanel';
import { HITLToast } from './components/HITLToast';
import { ImportPanel } from './components/ImportPanel';
import { InspectorPanel } from './components/InspectorPanel';
import { MCPServerSettings } from './components/MCPServerSettings';
import { RecoveryDialog } from './components/RecoveryDialog';
import { RunBanner, type RunStatus } from './components/RunBanner';
import { SettingsPanel } from './components/SettingsPanel';
import { SetupPanel } from './components/SetupPanel';
import { SmokeButton } from './components/SmokeButton';
import { SqlInspector } from './components/SqlInspector';
import { ToastProvider } from './components/Toast';
import { Transport } from './components/Transport';
import { UncertaintyPrompt } from './components/UncertaintyPrompt';
import {
  invokeHasApiKey,
  invokeReplaySession,
  invokeRunSmokeSession,
  invokeSetApiKey,
  subscribeAgentEvents,
  unwrapCmdError,
} from './lib/ipc';
import { useBuilderStore, useTestGraphStore } from './lib/builderStore';
import { useGraphStore } from './lib/graphStore';
import { useToastStore } from './lib/toastStore';
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
    // M08.6.D — same expose pattern for the Builder store so
    // tests/e2e-tauri/builder_load_aria.e2e.ts can drive the
    // load-applying store action directly (the OS file dialog
    // Inspector.onLoad opens is non-driveable by tauri-driver, so the
    // test invokes the underlying load_framework Tauri command + the
    // applyLoadedFramework action via executeScript — the exact code
    // path Inspector.onLoad runs minus the dialog click).
    __builderStore?: typeof useBuilderStore;
    // M08.8.A — the Tester's scoped graph store, exposed so
    // tests/e2e-tauri/execution_view.e2e.ts can inject a run trace into
    // the scoped store the Tester modal renders (the real `agent_event`
    // path replays into this store; injecting mirrors it without a key).
    __testGraphStore?: typeof useTestGraphStore;
    // M08.8.B — the reusable Toast store, exposed so
    // tests/e2e-tauri/visual_foundation.e2e.ts can push a toast and watch
    // it appear bottom-right + auto-dismiss (same affordance pattern as
    // __graphStore; the store carries no secrets).
    __toastStore?: typeof useToastStore;
  }
}
if (typeof window !== 'undefined') {
  window.__graphStore = useGraphStore;
  window.__builderStore = useBuilderStore;
  window.__testGraphStore = useTestGraphStore;
  window.__toastStore = useToastStore;
}

export function App(): JSX.Element {
  const [view, setView] = useState<AppView>('runtime');
  const [hasKey, setHasKey] = useState(false);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const tier = useGraphStore((s) => s.currentTier);

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
  const replayAvailable = lastSessionId !== null && lastSessionId.length > 0;
  const handleReplay = (): void => {
    if (lastSessionId !== null && lastSessionId.length > 0) {
      void invokeReplaySession(lastSessionId).catch((e) => {
        console.error('Replay session error:', e);
      });
    }
  };
  const runStatus: RunStatus = running ? 'running' : 'idle';

  // BudgetHeaderBar (dormant until a budget event) + the collapsible
  // SettingsPanel mount OUTSIDE the view conditional — both stay reachable
  // in Runtime and Builder (settings_tier_promotion.spec relies on this).
  const subchrome = (
    <>
      <BudgetHeaderBar />
      <SettingsPanel />
    </>
  );

  return (
    <ToastProvider>
      {view === 'runtime' ? (
        <>
          <AppShell
            view={view}
            onViewChange={setView}
            hasKey={hasKey}
            tier={tier}
            subchrome={subchrome}
            left={
              <>
                <SetupPanel onSave={handleSetKey} />
                <SmokeButton disabled={!hasKey || running} onClick={handleSmoke} />
                {error && <p className="error">{error}</p>}
                <ApprovalPanel />
                <HITLPanel />
                <GapPanel />
                <MCPServerSettings />
                <ImportPanel />
              </>
            }
            center={
              <>
                <RunBanner status={runStatus} />
                <GraphCanvas />
                <Transport replayAvailable={replayAvailable} onReplay={handleReplay} />
              </>
            }
            right={<InspectorPanel />}
          />
          <HITLModal />
          <HITLToast />
          <RecoveryDialog />
          <UncertaintyPrompt sessionId={lastSessionId ?? ''} />
          <SqlInspector />
        </>
      ) : (
        <AppShell
          view={view}
          onViewChange={setView}
          hasKey={hasKey}
          tier={tier}
          subchrome={subchrome}
          center={<BuilderShell />}
        />
      )}
    </ToastProvider>
  );
}
