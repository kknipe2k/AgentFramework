import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

const requestTierTransition = vi.fn();
const invokeSetGlobalBudget = vi.fn();
vi.mock('../../../src/lib/ipc', () => ({
  requestTierTransition: (...a: unknown[]) => requestTierTransition(...a),
  invokeSetGlobalBudget: (...a: unknown[]) => invokeSetGlobalBudget(...a),
  unwrapCmdError: (e: unknown) => `unwrapped:${String(e)}`,
}));

import { SettingsPanel } from '../../../src/components/SettingsPanel';
import { useGraphStore } from '../../../src/lib/graphStore';

describe('SettingsPanel (M08.G)', () => {
  beforeEach(() => {
    // currentTier + globalBudgetCap both persist across clear() — reset
    // per the v1.6 <test_isolation_audit> discipline.
    useGraphStore.setState({ currentTier: 'novice', globalBudgetCap: 0 });
    requestTierTransition.mockReset().mockResolvedValue(undefined);
    invokeSetGlobalBudget.mockReset().mockResolvedValue(undefined);
  });
  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it('renders_settings_panel_with_tier_and_budget_sections', () => {
    render(<SettingsPanel />);
    expect(screen.getByTestId('settings-panel')).toBeInTheDocument();
    expect(screen.getByTestId('tier-control')).toBeInTheDocument();
    expect(screen.getByTestId('budget-control')).toBeInTheDocument();
  });

  it('tier_control_displays_current_tier_from_store', () => {
    useGraphStore.setState({ currentTier: 'novice' });
    render(<SettingsPanel />);
    expect(screen.getByTestId('tier-current')).toHaveTextContent(/novice/i);
  });

  it('tier_control_button_label_is_promote_when_novice', () => {
    // #20 / principle 8 (labels-true): the redundant "Promote to Promoted"
    // is replaced by a truthful "Promote". Asserted on exact textContent —
    // a substring `toHaveTextContent('Promote')` would pass against the old
    // "Promote to Promoted" and never go red.
    useGraphStore.setState({ currentTier: 'novice' });
    render(<SettingsPanel />);
    expect(screen.getByTestId('tier-transition-button').textContent).toBe('Promote');
  });

  it('tier_control_button_label_is_demote_when_promoted', () => {
    useGraphStore.setState({ currentTier: 'promoted' });
    render(<SettingsPanel />);
    expect(screen.getByTestId('tier-transition-button').textContent).toBe('Demote');
  });

  it('clicking_promote_calls_requestTierTransition_with_promoted', async () => {
    const user = userEvent.setup();
    useGraphStore.setState({ currentTier: 'novice' });
    render(<SettingsPanel />);
    await user.click(screen.getByTestId('tier-transition-button'));
    await waitFor(() => expect(requestTierTransition).toHaveBeenCalledTimes(1));
    expect(requestTierTransition).toHaveBeenCalledWith('promoted', expect.any(String));
  });

  it('clicking_demote_calls_requestTierTransition_with_novice', async () => {
    const user = userEvent.setup();
    useGraphStore.setState({ currentTier: 'promoted' });
    render(<SettingsPanel />);
    await user.click(screen.getByTestId('tier-transition-button'));
    await waitFor(() => expect(requestTierTransition).toHaveBeenCalledTimes(1));
    expect(requestTierTransition).toHaveBeenCalledWith('novice', expect.any(String));
  });

  it('tier_control_does_not_optimistically_set_currentTier', async () => {
    const user = userEvent.setup();
    useGraphStore.setState({ currentTier: 'novice' });
    render(<SettingsPanel />);
    await user.click(screen.getByTestId('tier-transition-button'));
    await waitFor(() => expect(requestTierTransition).toHaveBeenCalled());
    // The backend's tier_transition event is the single writer — the
    // component must NOT flip currentTier itself.
    expect(useGraphStore.getState().currentTier).toBe('novice');
  });

  it('tier_control_never_renders_an_operator_option', () => {
    useGraphStore.setState({ currentTier: 'novice' });
    render(<SettingsPanel />);
    // §0d locks v0.1 to Novice + Promoted; Operator is v1.0.
    expect(screen.getByTestId('settings-panel')).not.toHaveTextContent(/operator/i);
  });

  it('tier_transition_error_surfaces_via_unwrapCmdError', async () => {
    const user = userEvent.setup();
    requestTierTransition.mockRejectedValueOnce({ type: 'internal', message: 'tier write failed' });
    useGraphStore.setState({ currentTier: 'novice' });
    render(<SettingsPanel />);
    await user.click(screen.getByTestId('tier-transition-button'));
    const err = await screen.findByTestId('tier-error');
    expect(err).toHaveTextContent(/unwrapped:/);
  });

  it('budget_control_input_reflects_globalBudgetCap_slot', () => {
    useGraphStore.setState({ globalBudgetCap: 25 });
    render(<SettingsPanel />);
    expect(screen.getByTestId('budget-cap-input')).toHaveValue(25);
  });

  it('clicking_save_cap_calls_invokeSetGlobalBudget_with_parsed_value', async () => {
    const user = userEvent.setup();
    render(<SettingsPanel />);
    fireEvent.change(screen.getByTestId('budget-cap-input'), { target: { value: '40' } });
    await user.click(screen.getByTestId('budget-save-button'));
    await waitFor(() => expect(invokeSetGlobalBudget).toHaveBeenCalledWith(40));
  });

  it('saving_cap_updates_globalBudgetCap_slot_so_input_reflects_it', async () => {
    const user = userEvent.setup();
    render(<SettingsPanel />);
    fireEvent.change(screen.getByTestId('budget-cap-input'), { target: { value: '40' } });
    await user.click(screen.getByTestId('budget-save-button'));
    // The 🟡-4 contract: the configured cap persists into the slot so a
    // re-opened panel reflects it.
    await waitFor(() => expect(useGraphStore.getState().globalBudgetCap).toBe(40));
    expect(screen.getByTestId('budget-cap-input')).toHaveValue(40);
  });

  it('budget_control_rejects_negative_input_without_calling_command', async () => {
    const user = userEvent.setup();
    render(<SettingsPanel />);
    fireEvent.change(screen.getByTestId('budget-cap-input'), { target: { value: '-5' } });
    await user.click(screen.getByTestId('budget-save-button'));
    expect(invokeSetGlobalBudget).not.toHaveBeenCalled();
    expect(screen.getByTestId('budget-error')).toBeInTheDocument();
  });

  it('budget_save_command_error_surfaces_via_unwrapCmdError', async () => {
    const user = userEvent.setup();
    invokeSetGlobalBudget.mockRejectedValueOnce({ type: 'internal', message: 'cap write failed' });
    render(<SettingsPanel />);
    fireEvent.change(screen.getByTestId('budget-cap-input'), { target: { value: '40' } });
    await user.click(screen.getByTestId('budget-save-button'));
    const err = await screen.findByTestId('budget-error');
    expect(err).toHaveTextContent(/unwrapped:/);
  });

  it('every_settings_panel_class_has_a_corresponding_css_rule', () => {
    // gotcha #67 — a rendered className with no CSS rule renders unstyled.
    const css = readFileSync(resolve(__dirname, '../../../src/styles.css'), 'utf8');
    const selectors = [
      '.settings-panel',
      '.settings-panel__header',
      '.settings-panel__title',
      '.settings-panel__section',
      '.settings-panel__section--tier',
      '.settings-panel__section--budget',
      '.settings-panel__section-title',
      '.settings-panel__tier-current',
      '.settings-panel__tier-value',
      '.settings-panel__tier-value--novice',
      '.settings-panel__tier-value--promoted',
      '.settings-panel__tier-explainer',
      '.settings-panel__tier-button',
      '.settings-panel__error',
      '.settings-panel__budget-label',
      '.settings-panel__budget-input',
      '.settings-panel__budget-button',
      '.settings-panel__budget-hint',
    ];
    for (const sel of selectors) {
      expect(css, `missing CSS rule for ${sel}`).toContain(sel);
    }
  });
});
