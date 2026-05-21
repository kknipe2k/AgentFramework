import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ViewSwitch } from '../../../../src/components/builder/ViewSwitch';

// M08.C — the Runtime <-> Builder top-level view toggle (App chrome).
// ViewSwitch is a controlled component: it renders the current `value`
// and reports a chosen view via `onChange`; App.tsx owns the state.

describe('ViewSwitch', () => {
  it('renders_runtime_and_builder_options', () => {
    render(<ViewSwitch value="runtime" onChange={() => undefined} />);
    expect(screen.getByTestId('view-switch-runtime')).toBeInTheDocument();
    expect(screen.getByTestId('view-switch-builder')).toBeInTheDocument();
  });

  it('clicking_builder_calls_onChange_with_builder', async () => {
    const onChange = vi.fn();
    render(<ViewSwitch value="runtime" onChange={onChange} />);
    await userEvent.click(screen.getByTestId('view-switch-builder'));
    expect(onChange).toHaveBeenCalledWith('builder');
  });

  it('clicking_runtime_calls_onChange_with_runtime', async () => {
    const onChange = vi.fn();
    render(<ViewSwitch value="builder" onChange={onChange} />);
    await userEvent.click(screen.getByTestId('view-switch-runtime'));
    expect(onChange).toHaveBeenCalledWith('runtime');
  });
});
