import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen, within } from '@testing-library/react';
import { AppShell } from '../../../src/components/AppShell';

afterEach(cleanup);

function renderShell(props: Record<string, unknown> = {}): void {
  render(
    <AppShell
      view="runtime"
      onViewChange={vi.fn()}
      hasKey={true}
      tier="promoted"
      left={<div data-testid="left-content">L</div>}
      center={<div data-testid="center-content">C</div>}
      right={<div data-testid="the-inspector">R</div>}
      {...props}
    />,
  );
}

describe('AppShell — the 52/232/360 three-pane Light Instrument shell (M08.8.B)', () => {
  it('renders_the_three_panes', () => {
    renderShell();
    expect(screen.getByTestId('rail-left')).toBeInTheDocument();
    expect(screen.getByTestId('pane-center')).toBeInTheDocument();
    expect(screen.getByTestId('rail-right')).toBeInTheDocument();
  });

  it('mounts_the_inspector_in_the_360px_right_rail', () => {
    renderShell();
    expect(
      within(screen.getByTestId('rail-right')).getByTestId('the-inspector'),
    ).toBeInTheDocument();
  });

  it('the_brand_is_a_heading_and_the_M03_dev_label_is_retired', () => {
    renderShell();
    const heading = screen.getByRole('heading', { name: /Agent Runtime/i });
    expect(heading).toBeInTheDocument();
    expect(heading).not.toHaveTextContent(/M03 live graph/i);
  });

  it('exposes_the_view_switch_as_a_tablist_in_the_topbar', () => {
    renderShell();
    expect(screen.getByRole('tablist')).toBeInTheDocument();
  });

  it('shows_the_current_tier_in_a_chip', () => {
    renderShell();
    expect(screen.getByTestId('topbar-tier-chip')).toHaveTextContent(/promoted/i);
  });

  it('shows_key_state_in_a_chip', () => {
    const { rerender } = render(
      <AppShell
        view="runtime"
        onViewChange={vi.fn()}
        hasKey={true}
        tier="novice"
        center={<div />}
      />,
    );
    expect(screen.getByTestId('topbar-key-chip')).toHaveTextContent(/key active/i);
    rerender(
      <AppShell
        view="runtime"
        onViewChange={vi.fn()}
        hasKey={false}
        tier="novice"
        center={<div />}
      />,
    );
    expect(screen.getByTestId('topbar-key-chip')).toHaveTextContent(/no key/i);
  });

  it('omits_the_side_rails_in_single_pane_mode', () => {
    render(
      <AppShell
        view="builder"
        onViewChange={vi.fn()}
        hasKey={true}
        tier="novice"
        center={<div data-testid="bc">B</div>}
      />,
    );
    expect(screen.queryByTestId('rail-left')).toBeNull();
    expect(screen.queryByTestId('rail-right')).toBeNull();
    expect(screen.getByTestId('pane-center')).toBeInTheDocument();
  });
});
