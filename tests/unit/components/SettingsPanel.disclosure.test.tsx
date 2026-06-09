import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

// M09.D.fix — the bundled DESIGN.md disclosure pass. The M09.D IRL surfaced
// that the Settings panel is a flat always-on stack of sections (IRL #18);
// DESIGN.md principle 3 (progressive disclosure for dense surfaces) + the
// Panels component rule ("dense config panels default collapsed") require the
// sections to collapse/expand behind a disclosure control. This pins the
// budget section — a dense config surface — as collapsible and
// default-collapsed, revealed on its disclosure toggle.

const requestTierTransition = vi.fn();
const invokeSetGlobalBudget = vi.fn();
vi.mock('../../../src/lib/ipc', () => ({
  requestTierTransition: (...a: unknown[]) => requestTierTransition(...a),
  invokeSetGlobalBudget: (...a: unknown[]) => invokeSetGlobalBudget(...a),
  unwrapCmdError: (e: unknown) => `unwrapped:${String(e)}`,
}));

import { SettingsPanel } from '../../../src/components/SettingsPanel';
import { useGraphStore } from '../../../src/lib/graphStore';

describe('SettingsPanel — progressive disclosure (M09.D.fix / DESIGN.md principle 3)', () => {
  beforeEach(() => {
    useGraphStore.setState({ currentTier: 'novice', globalBudgetCap: 0 });
    requestTierTransition.mockReset().mockResolvedValue(undefined);
    invokeSetGlobalBudget.mockReset().mockResolvedValue(undefined);
  });
  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it('the_budget_section_is_collapsible_behind_a_disclosure_toggle', () => {
    render(<SettingsPanel />);
    // The disclosure control exists (the flat always-on section is gone).
    expect(screen.getByTestId('settings-section-toggle-budget')).toBeInTheDocument();
  });

  it('the_dense_budget_section_defaults_collapsed_and_reveals_on_toggle', async () => {
    render(<SettingsPanel />);
    // DESIGN.md Panels rule — a dense config panel defaults collapsed, so its
    // body (the cap input) is NOT mounted until disclosed.
    expect(screen.queryByTestId('budget-cap-input')).not.toBeInTheDocument();
    await userEvent.click(screen.getByTestId('settings-section-toggle-budget'));
    expect(screen.getByTestId('budget-cap-input')).toBeInTheDocument();
  });
});
