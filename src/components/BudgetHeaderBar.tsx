import { useState } from 'react';
import { invokeSetGlobalBudget, unwrapCmdError } from '../lib/ipc';
import { useGraphStore, type BudgetState, type BudgetStatus } from '../lib/graphStore';

/**
 * Color gradient per spec §2a Graph integration:
 * `< warn_at_percent` → green, `>= warn` → amber, `>= downshift` → orange,
 * `>= hitl` → red, `>= hard_stop` → red + exceeded badge.
 *
 * The status field on `BudgetState` already encodes the bucket so the
 * renderer is agnostic to the specific thresholds (those live in the
 * enforcer + framework JSON).
 */
function colorClass(status: BudgetStatus): string {
  switch (status) {
    case 'ok':
      return 'budget-bar__bar--ok';
    case 'warn':
      return 'budget-bar__bar--warn';
    case 'downshift':
      return 'budget-bar__bar--downshift';
    case 'suspended':
      return 'budget-bar__bar--suspended';
    case 'exceeded':
      return 'budget-bar__bar--exceeded';
  }
}

function statusLabel(status: BudgetStatus): string {
  switch (status) {
    case 'ok':
      return 'OK';
    case 'warn':
      return 'Warning';
    case 'downshift':
      return 'Downshifted';
    case 'suspended':
      return 'Suspended';
    case 'exceeded':
      return 'Exceeded';
  }
}

function formatUsd(n: number): string {
  return `$${n.toFixed(2)}`;
}

/**
 * BudgetHeaderBar — spec §2a (M04 Stage F).
 *
 * Sticky top-of-screen bar. Shows current spend / cap / percent with
 * color gradient. Tooltip on hover shows scope breakdown. Click reveals
 * a settings panel for the global per-day cap (M10 wires persistence;
 * Stage F exposes the seam via `set_global_budget`).
 *
 * Renders only when a budget event has landed (state.budget !== null).
 * Idle sessions show no bar — first budget event lights it up.
 */
export function BudgetHeaderBar(): JSX.Element | null {
  const budget = useGraphStore((s) => s.budget);
  const [showSettings, setShowSettings] = useState(false);
  const [globalCap, setGlobalCap] = useState('');
  const [settingsError, setSettingsError] = useState<string | null>(null);
  const [settingsSaved, setSettingsSaved] = useState(false);

  if (budget === null) {
    return null;
  }

  async function saveGlobal(currentInput: string): Promise<void> {
    const parsed = parseFloat(currentInput);
    if (Number.isNaN(parsed) || parsed < 0) {
      setSettingsError('Enter a non-negative number (USD).');
      setSettingsSaved(false);
      return;
    }
    try {
      await invokeSetGlobalBudget(parsed);
      setSettingsError(null);
      setSettingsSaved(true);
    } catch (e) {
      console.error('set_global_budget error:', e);
      setSettingsError(unwrapCmdError(e));
      setSettingsSaved(false);
    }
  }

  return (
    <div
      className="budget-bar"
      role="region"
      aria-label="Session budget"
      data-testid="budget-header-bar"
      data-status={budget.status}
    >
      <button
        type="button"
        className={`budget-bar__bar ${colorClass(budget.status)}`}
        data-testid="budget-bar-button"
        aria-label={`Budget ${statusLabel(budget.status)}: ${formatUsd(budget.spentUsd)} of ${formatUsd(budget.capUsd)} (${budget.percent}%)`}
        title={tooltipText(budget)}
        onClick={() => setShowSettings((v) => !v)}
      >
        <span className="budget-bar__spent" data-testid="budget-bar-spent">
          {formatUsd(budget.spentUsd)}
        </span>
        <span className="budget-bar__separator"> / </span>
        <span className="budget-bar__cap" data-testid="budget-bar-cap">
          {formatUsd(budget.capUsd)}
        </span>
        <span className="budget-bar__percent" data-testid="budget-bar-percent">
          {' '}
          ({budget.percent}%)
        </span>
        {budget.status === 'exceeded' && (
          <span
            className="budget-bar__badge budget-bar__badge--exceeded"
            data-testid="budget-bar-exceeded-banner"
          >
            Session terminated — budget exceeded
          </span>
        )}
        {budget.status === 'suspended' && (
          <span
            className="budget-bar__badge budget-bar__badge--suspended"
            data-testid="budget-bar-suspended-badge"
          >
            Suspended — awaiting approval
          </span>
        )}
        {budget.status === 'downshift' && (
          <span
            className="budget-bar__badge budget-bar__badge--downshift"
            data-testid="budget-bar-downshift-badge"
          >
            Downshifted
          </span>
        )}
      </button>
      {showSettings && (
        <form
          className="budget-bar__settings"
          data-testid="budget-bar-settings"
          onSubmit={(e) => {
            e.preventDefault();
            void saveGlobal(globalCap);
          }}
        >
          <label className="budget-bar__settings-label">
            Global per-day cap (USD):
            <input
              type="number"
              min="0"
              step="0.01"
              value={globalCap}
              data-testid="budget-bar-global-cap-input"
              onChange={(e) => {
                setGlobalCap(e.target.value);
                setSettingsSaved(false);
              }}
              aria-label="Global per-day budget cap in USD"
            />
          </label>
          <button type="submit" data-testid="budget-bar-save-global">
            Save
          </button>
          {settingsError !== null && (
            <span className="budget-bar__settings-error" data-testid="budget-bar-settings-error">
              {settingsError}
            </span>
          )}
          {settingsSaved && (
            <span className="budget-bar__settings-ok" data-testid="budget-bar-settings-ok">
              Saved
            </span>
          )}
        </form>
      )}
    </div>
  );
}

function tooltipText(b: BudgetState): string {
  return `${statusLabel(b.status)} — ${formatUsd(b.spentUsd)} of ${formatUsd(b.capUsd)} spent (${b.percent}%)`;
}

/**
 * Test-only re-export of internal helpers (M04.E pattern). Allows
 * vitest to exercise the pure helpers (`colorClass`, `statusLabel`,
 * `tooltipText`) without going through the React render path.
 */
export const _testing = { colorClass, statusLabel, tooltipText, formatUsd };
