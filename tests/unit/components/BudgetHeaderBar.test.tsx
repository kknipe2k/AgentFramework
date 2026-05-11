import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const invokeMock = vi.fn(async (..._args: unknown[]) => undefined);

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { BudgetHeaderBar, _testing } from '../../../src/components/BudgetHeaderBar';
import { useGraphStore } from '../../../src/lib/graphStore';
import type { AgentEvent } from '../../../src/types/agent_event';

function dispatch(events: AgentEvent[]): void {
  act(() => {
    for (const e of events) {
      useGraphStore.getState().applyEvent(e);
    }
  });
}

const warnEvent: AgentEvent = {
  type: 'budget_warn',
  spent_usd: 2.5,
  cap_usd: 5.0,
  percent: 50,
};
const downshiftEvent: AgentEvent = {
  type: 'budget_downshift',
  from_model: 'claude-opus-4-7',
  to_model: 'claude-sonnet-4-6',
  reason: 'budget_threshold',
};
const suspendEvent: AgentEvent = {
  type: 'budget_suspended',
  spent_usd: 4.5,
  cap_usd: 5.0,
};
const exceededEvent: AgentEvent = {
  type: 'budget_exceeded',
  spent_usd: 5.0,
  cap_usd: 5.0,
};

describe('BudgetHeaderBar', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
    useGraphStore.getState().clear();
  });

  afterEach(() => {
    useGraphStore.getState().clear();
  });

  it('returns_null_when_no_budget_state', () => {
    const { container } = render(<BudgetHeaderBar />);
    expect(container).toBeEmptyDOMElement();
    expect(screen.queryByTestId('budget-header-bar')).toBeNull();
  });

  it('renders_warn_after_budget_warn_event', () => {
    dispatch([warnEvent]);
    render(<BudgetHeaderBar />);
    const bar = screen.getByTestId('budget-header-bar');
    expect(bar).toHaveAttribute('data-status', 'warn');
    expect(screen.getByTestId('budget-bar-spent')).toHaveTextContent('$2.50');
    expect(screen.getByTestId('budget-bar-cap')).toHaveTextContent('$5.00');
    expect(screen.getByTestId('budget-bar-percent')).toHaveTextContent('(50%)');
  });

  it('flips_to_downshift_status_after_budget_downshift_event', () => {
    dispatch([warnEvent, downshiftEvent]);
    render(<BudgetHeaderBar />);
    expect(screen.getByTestId('budget-header-bar')).toHaveAttribute('data-status', 'downshift');
    expect(screen.getByTestId('budget-bar-downshift-badge')).toBeInTheDocument();
  });

  it('flips_to_suspended_with_badge_at_90_percent', () => {
    dispatch([suspendEvent]);
    render(<BudgetHeaderBar />);
    expect(screen.getByTestId('budget-header-bar')).toHaveAttribute('data-status', 'suspended');
    expect(screen.getByTestId('budget-bar-suspended-badge')).toBeInTheDocument();
  });

  it('renders_exceeded_terminal_banner_at_100_percent', () => {
    dispatch([exceededEvent]);
    render(<BudgetHeaderBar />);
    const bar = screen.getByTestId('budget-header-bar');
    expect(bar).toHaveAttribute('data-status', 'exceeded');
    expect(screen.getByTestId('budget-bar-exceeded-banner')).toHaveTextContent(
      'Session terminated',
    );
    expect(screen.getByTestId('budget-bar-percent')).toHaveTextContent('(100%)');
  });

  it('aria_label_describes_spend_cap_and_percent', () => {
    dispatch([warnEvent]);
    render(<BudgetHeaderBar />);
    const button = screen.getByTestId('budget-bar-button');
    const label = button.getAttribute('aria-label') ?? '';
    expect(label).toContain('$2.50');
    expect(label).toContain('$5.00');
    expect(label).toContain('50%');
  });

  it('clicking_bar_opens_settings_panel', () => {
    dispatch([warnEvent]);
    render(<BudgetHeaderBar />);
    expect(screen.queryByTestId('budget-bar-settings')).toBeNull();
    fireEvent.click(screen.getByTestId('budget-bar-button'));
    expect(screen.getByTestId('budget-bar-settings')).toBeInTheDocument();
  });

  it('save_global_dispatches_set_global_budget_with_parsed_value', async () => {
    dispatch([warnEvent]);
    render(<BudgetHeaderBar />);
    fireEvent.click(screen.getByTestId('budget-bar-button'));
    fireEvent.change(screen.getByTestId('budget-bar-global-cap-input'), {
      target: { value: '12.50' },
    });
    fireEvent.submit(screen.getByTestId('budget-bar-settings'));
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith('set_global_budget', { usdCap: 12.5 }),
    );
  });

  it('rejects_negative_cap_in_settings_form_with_inline_error', async () => {
    dispatch([warnEvent]);
    render(<BudgetHeaderBar />);
    fireEvent.click(screen.getByTestId('budget-bar-button'));
    fireEvent.change(screen.getByTestId('budget-bar-global-cap-input'), {
      target: { value: '-1' },
    });
    fireEvent.submit(screen.getByTestId('budget-bar-settings'));
    await waitFor(() =>
      expect(screen.getByTestId('budget-bar-settings-error')).toHaveTextContent('non-negative'),
    );
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it('surfaces_invoke_error_in_settings_form', async () => {
    dispatch([warnEvent]);
    invokeMock.mockReset();
    invokeMock.mockRejectedValueOnce({ type: 'internal', message: 'oops' });
    render(<BudgetHeaderBar />);
    fireEvent.click(screen.getByTestId('budget-bar-button'));
    fireEvent.change(screen.getByTestId('budget-bar-global-cap-input'), {
      target: { value: '5' },
    });
    fireEvent.submit(screen.getByTestId('budget-bar-settings'));
    await waitFor(() =>
      expect(screen.getByTestId('budget-bar-settings-error')).toHaveTextContent('oops'),
    );
  });

  it('helper_colorClass_maps_each_status', () => {
    expect(_testing.colorClass('ok')).toContain('--ok');
    expect(_testing.colorClass('warn')).toContain('--warn');
    expect(_testing.colorClass('downshift')).toContain('--downshift');
    expect(_testing.colorClass('suspended')).toContain('--suspended');
    expect(_testing.colorClass('exceeded')).toContain('--exceeded');
  });

  it('helper_statusLabel_maps_each_status', () => {
    expect(_testing.statusLabel('ok')).toBe('OK');
    expect(_testing.statusLabel('warn')).toBe('Warning');
    expect(_testing.statusLabel('downshift')).toBe('Downshifted');
    expect(_testing.statusLabel('suspended')).toBe('Suspended');
    expect(_testing.statusLabel('exceeded')).toBe('Exceeded');
  });

  it('helper_formatUsd_renders_two_decimals', () => {
    expect(_testing.formatUsd(0)).toBe('$0.00');
    expect(_testing.formatUsd(3.456)).toBe('$3.46');
    expect(_testing.formatUsd(100)).toBe('$100.00');
  });

  it('helper_tooltipText_includes_status_spend_cap_percent', () => {
    const text = _testing.tooltipText({
      spentUsd: 2.5,
      capUsd: 5,
      percent: 50,
      status: 'warn',
    });
    expect(text).toContain('Warning');
    expect(text).toContain('$2.50');
    expect(text).toContain('$5.00');
    expect(text).toContain('50%');
  });
});
