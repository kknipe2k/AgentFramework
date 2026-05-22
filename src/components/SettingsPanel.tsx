import { useState } from 'react';
import { useGraphStore } from '../lib/graphStore';
import { requestTierTransition, invokeSetGlobalBudget, unwrapCmdError } from '../lib/ipc';
import type { TierRef } from '../types/agent_event';

/**
 * Settings panel (M08 Stage G). A focused settings surface hosting the
 * Novice↔Promoted tier control + the global-budget-cap control. Closes
 * M07-IRL #5 (no tier-promotion UI → the Promoted tier was unreachable)
 * and M06.5 IRL 🟡-4 (budget settings not state-wired).
 *
 * NOT a catch-all: the Anthropic API key stays in SetupPanel. Operator
 * tier is NOT surfaced (v1.0 — §0d locks v0.1 to Novice + Promoted).
 *
 * Mounted at App.tsx top level as cross-mode chrome — outside the
 * Runtime↔Builder view conditional (C.3.2), so the tier control is
 * reachable in both modes. v0.1 has no Settings-tab infrastructure
 * (the M06.E no-routing rule).
 */
export function SettingsPanel(): JSX.Element {
  return (
    <section className="settings-panel" data-testid="settings-panel">
      <header className="settings-panel__header">
        <h2 className="settings-panel__title">Settings</h2>
      </header>
      <TierControl />
      <BudgetControl />
    </section>
  );
}

/**
 * Current-tier display + the Novice↔Promoted transition control. Reads
 * `currentTier` from the store (the EXISTING slot, reduced by the
 * EXISTING `tier_transition` branch — graphStore.ts:513/:1549). The
 * control calls the EXISTING `request_tier_transition` backend command
 * via `requestTierTransition`; Stage G does NOT reimplement tier logic
 * (Hard Rule 8). The displayed tier updates when the backend's
 * `tier_transition` event flows through the existing reducer.
 */
function TierControl(): JSX.Element {
  const tier = useGraphStore((s) => s.currentTier);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Novice → Promoted is the promotion; Promoted → Novice the demotion.
  // Operator is NOT a target — TierRef has only 'novice' | 'promoted'.
  const target: TierRef = tier === 'novice' ? 'promoted' : 'novice';
  const actionLabel = tier === 'novice' ? 'Promote to Promoted' : 'Demote to Novice';

  async function handleTransition(): Promise<void> {
    setPending(true);
    setError(null);
    try {
      // The `reason` param is for the audit/event record — a fixed
      // string, not user-entered here.
      await requestTierTransition(target, `user requested ${target} via Settings`);
      // NB: do NOT setState currentTier here — the backend's
      // tier_transition event updates it through the existing reducer.
    } catch (e) {
      console.error('request_tier_transition error:', e);
      setError(unwrapCmdError(e));
    } finally {
      setPending(false);
    }
  }

  return (
    <div
      className="settings-panel__section settings-panel__section--tier"
      data-testid="tier-control"
    >
      <h3 className="settings-panel__section-title">Capability tier</h3>
      <p className="settings-panel__tier-current" data-testid="tier-current">
        Current tier:{' '}
        <span className={`settings-panel__tier-value settings-panel__tier-value--${tier}`}>
          {tier}
        </span>
      </p>
      <p className="settings-panel__tier-explainer">
        {tier === 'novice'
          ? 'Novice restricts capabilities to safe defaults. Promote to enable MCP-server management and broader tool access.'
          : 'Promoted enables MCP-server management and broader tool access. Demote to return to Novice safe defaults.'}
      </p>
      <button
        type="button"
        className="settings-panel__tier-button"
        data-testid="tier-transition-button"
        disabled={pending}
        onClick={() => void handleTransition()}
      >
        {pending ? 'Applying…' : actionLabel}
      </button>
      {error !== null && (
        <p className="settings-panel__error" data-testid="tier-error">
          {error}
        </p>
      )}
    </div>
  );
}

/**
 * Global per-day budget-cap control (M06.5 IRL 🟡-4 state-wiring). Reads
 * the configured cap from the store's `globalBudgetCap` slot and
 * persists changes via the EXISTING `invokeSetGlobalBudget` command
 * (ipc.ts). The input REFLECTS the live slot value — the 🟡-4 complaint
 * was that it did not. `0` disables the cap.
 *
 * Distinct from `graphStore.budget` (the per-session SPEND snapshot from
 * budget_* events) — this is the user-CONFIGURED cap. The budget
 * PRIMITIVE shipped at M04; G wires only the settings-surface input.
 */
function BudgetControl(): JSX.Element {
  const cap = useGraphStore((s) => s.globalBudgetCap);
  const setCap = useGraphStore((s) => s.setGlobalBudgetCap);
  const [draft, setDraft] = useState<string>(String(cap));
  const [error, setError] = useState<string | null>(null);

  async function handleSave(): Promise<void> {
    const parsed = Number(draft);
    if (!Number.isFinite(parsed) || parsed < 0) {
      setError('Enter a non-negative dollar amount (0 disables the cap).');
      return;
    }
    setError(null);
    try {
      await invokeSetGlobalBudget(parsed);
      // Mirror into the store so the input reflects the committed value.
      setCap(parsed);
    } catch (e) {
      console.error('set_global_budget error:', e);
      setError(unwrapCmdError(e));
    }
  }

  return (
    <div
      className="settings-panel__section settings-panel__section--budget"
      data-testid="budget-control"
    >
      <h3 className="settings-panel__section-title">Daily budget cap (USD)</h3>
      <label className="settings-panel__budget-label">
        Cap:{' '}
        <input
          className="settings-panel__budget-input"
          data-testid="budget-cap-input"
          type="number"
          min={0}
          step="0.01"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
        />
      </label>{' '}
      <button
        type="button"
        className="settings-panel__budget-button"
        data-testid="budget-save-button"
        onClick={() => void handleSave()}
      >
        Save cap
      </button>
      <p className="settings-panel__budget-hint">Set 0 to disable the cap.</p>
      {error !== null && (
        <p className="settings-panel__error" data-testid="budget-error">
          {error}
        </p>
      )}
    </div>
  );
}
