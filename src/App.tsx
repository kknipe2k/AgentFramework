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
  getCurrentTier,
  invokeReplayLatestSession,
  invokeReplaySession,
  invokeRunSmokeSession,
  invokeSetApiKey,
  isSetupRequired,
  subscribeAgentEvents,
  unwrapCmdError,
} from './lib/ipc';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { useBuilderStore, useTestGraphStore } from './lib/builderStore';
import { useGraphStore } from './lib/graphStore';
import { refreshHasKey } from './lib/keyState';
import { shouldExposeStores } from './lib/testMode';
import { useToastStore } from './lib/toastStore';
import './styles.css';

const LAST_SESSION_KEY = 'lastSessionId';

// Renderer-level test affordance — exposes the Zustand stores on
// `window.__*Store` so the e2e harnesses can drive runtime state without
// an SDK + Anthropic. Module mocking across the @tauri-apps/api ESM
// boundary doesn't work in Playwright (Vitest covers the click→invoke
// linkage); this affordance lets the E2E specs exercise surface-on-state
// flows that only render correctly inside a real browser layout, and the
// 12 e2e-tauri specs drive the BUILT binary through them.
//
// M09.5.A (TD-050 / review C1): the exposure is a typed write path into
// runtime state for any injected script, so it is GATED — exposed only
// under the Vite dev server (`import.meta.env.DEV`) or the shell-resolved
// e2e test mode (`window.__E2E__`, set by main.rs's js_init_script plugin
// only when launched with AGENT_RUNTIME_E2E=1). A production launch
// exposes nothing. See `shouldExposeStores` + src/lib/testMode.ts.
declare global {
  interface Window {
    // Set to `true` by the Tauri shell's e2e-seam plugin when the app is
    // launched with AGENT_RUNTIME_E2E=1; `undefined` in production.
    __E2E__?: boolean;
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
if (typeof window !== 'undefined' && shouldExposeStores(import.meta.env.DEV, window.__E2E__)) {
  window.__graphStore = useGraphStore;
  window.__builderStore = useBuilderStore;
  window.__testGraphStore = useTestGraphStore;
  window.__toastStore = useToastStore;
}

export function App(): JSX.Element {
  const [view, setView] = useState<AppView>('runtime');
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const tier = useGraphStore((s) => s.currentTier);
  // M09.5.F (honest key chip): the chip state lives in graphStore — the
  // currentTier precedent — so the Tester's prop-less catch can flip it
  // too, not just the handlers in this component.
  const hasKey = useGraphStore((s) => s.hasKey);

  useEffect(() => {
    // M07-IRL #7: seed `hasKey` from the OS keychain so a key entered in
    // a prior session survives an app restart. The root cause of the
    // finding was the absent startup read — `hasKey` was hardcoded false
    // and only flipped inside handleSetKey.
    void refreshHasKey();

    // M08.8.C.fix #19: seed `currentTier` from the backend's
    // persisted/enforced tier so the Settings display matches the
    // enforced tier across a restart. The renderer defaulted currentTier
    // to 'novice' and wrote it ONLY from tier_transition events — after a
    // restart with a Promoted backend (tier.json) the display read Novice
    // while the run enforced Promoted. Mirrors the invokeHasApiKey seed
    // above; the seed REFLECTS the enforced tier, it never widens it.
    void getCurrentTier()
      .then((seeded) => {
        // Only seed a valid TierRef — a malformed bridge payload must not
        // corrupt currentTier (it drives the topbar chip's titleCase + the
        // SettingsPanel toggle). Defensive, like listInstalledArtifacts'
        // Array.isArray coercion.
        if (seeded === 'novice' || seeded === 'promoted') {
          useGraphStore.getState().setCurrentTier(seeded);
        }
      })
      .catch((e) => {
        console.error('get_current_tier error:', e);
      });

    // Replay-on-mount reconstructs the prior session's graph. The
    // `agent_event` listener MUST be registered (and its registration
    // awaited) BEFORE any replay fires — the backend re-emits the signal
    // log as soon as the command runs, and an emit that beats the
    // subscription is dropped. The reconstruct-on-mount path is exactly
    // where that race lives (the live smoke path registers long before a
    // user click), so order the subscription first.
    let unlisten: UnlistenFn | undefined;
    let cancelled = false;
    void (async () => {
      unlisten = await subscribeAgentEvents((event) => {
        if (event.type === 'session_start') {
          localStorage.setItem(LAST_SESSION_KEY, event.session_id);
        }
        if (event.type === 'tier_transition') {
          // M08.8.C.fix #20 (DESIGN.md principle 1 — feedback): every tier
          // change confirms with a toast naming the new tier. The reducer
          // updates currentTier; this surfaces the change to the user.
          useToastStore.getState().push({
            kind: 'info',
            title: `Tier changed to ${event.current}`,
          });
        }
        useGraphStore.getState().applyEvent(event);
        if (event.type === 'agent_complete' || event.type === 'agent_error') {
          setRunning(false);
        }
      });
      if (cancelled) {
        unlisten();
        return;
      }
      // `lastSessionId` survives a soft reload but NOT a full app restart
      // (a relaunched WebView comes up on a fresh profile that wipes
      // localStorage — TD-044). The backend owns persistence, so when no
      // id is stashed, ask it to replay the most-recent session WITH
      // signals; graphStore.applyEvent's idempotence guarantees the
      // reconstructed graph matches the original either way.
      const lastSessionId = localStorage.getItem(LAST_SESSION_KEY);
      try {
        if (lastSessionId !== null && lastSessionId.length > 0) {
          await invokeReplaySession(lastSessionId);
        } else {
          await invokeReplayLatestSession();
        }
      } catch (e) {
        console.error('Replay session error:', e);
      }
    })();
    return () => {
      cancelled = true;
      if (unlisten !== undefined) {
        unlisten();
      }
    };
  }, []);

  async function handleSetKey(key: string): Promise<void> {
    try {
      await invokeSetApiKey(key);
      useGraphStore.getState().setHasKey(true);
    } catch (e) {
      console.error('Set API key error:', e);
      setError(unwrapCmdError(e));
    }
  }

  async function handleSmoke(): Promise<void> {
    useGraphStore.getState().clear();
    setRunning(true);
    setError(null);
    // M09.5.F (honest key chip): a run that fails SetupRequired flips the
    // chip false off the run loop's own authoritative read, and that flip
    // is STICKY — the settled-run re-poll is skipped, because a still-true
    // has_api_key probe racing in afterwards would recreate the exact lie
    // this stage kills (the vanish case: probe says present, the run
    // resolved none). Every OTHER settled run re-polls so an out-of-band
    // keychain change surfaces — including a non-key failure, where a
    // provider error must not redden a good key.
    let setupRequiredSeen = false;
    try {
      await invokeRunSmokeSession();
    } catch (e) {
      // Always log structured errors to DevTools console — `error` carries
      // the user-facing string but the full object is needed for
      // diagnostics (per docs/gotchas.md #29 keyring-stub case the renderer
      // alone showed "[object Object]" with no usable signal).
      console.error('Smoke test error:', e);
      if (isSetupRequired(e)) {
        setupRequiredSeen = true;
        useGraphStore.getState().setHasKey(false);
      }
      setError(unwrapCmdError(e));
      setRunning(false);
    } finally {
      if (!setupRequiredSeen) {
        void refreshHasKey();
      }
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
