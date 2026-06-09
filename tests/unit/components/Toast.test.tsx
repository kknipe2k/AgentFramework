import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { act, cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ToastProvider, useToast } from '../../../src/components/Toast';
import { useToastStore } from '../../../src/lib/toastStore';

function reset(): void {
  useToastStore.setState({ toasts: [] });
}

describe('Toast — reusable notification primitive (M08.8.B)', () => {
  beforeEach(() => {
    reset();
    vi.useFakeTimers();
  });
  afterEach(() => {
    // useRealTimers() discards any pending fake-timer dismissals; the
    // store reset clears toasts. (One test opts into real timers itself,
    // so do not call a fake-timer-only API here.)
    vi.useRealTimers();
    cleanup();
    reset();
  });

  it('the_stack_is_an_aria_live_polite_status_region', () => {
    render(
      <ToastProvider>
        <div />
      </ToastProvider>,
    );
    const region = screen.getByTestId('toast-stack');
    // role=status + aria-live=polite so screen readers announce the
    // confirmation without interrupting (Soueidan aria-live part 1; MDN
    // status role). DESIGN.md principle 1 — every action gives feedback.
    expect(region).toHaveAttribute('role', 'status');
    expect(region).toHaveAttribute('aria-live', 'polite');
  });

  it('renders_a_pushed_toast_title_and_message', () => {
    render(
      <ToastProvider>
        <div />
      </ToastProvider>,
    );
    act(() => {
      useToastStore.getState().push({ kind: 'ok', title: 'Budget cap saved', message: '$5/day' });
    });
    expect(screen.getByText('Budget cap saved')).toBeInTheDocument();
    expect(screen.getByText('$5/day')).toBeInTheDocument();
  });

  it('auto_dismisses_after_the_default_4_2s_ttl', () => {
    render(
      <ToastProvider>
        <div />
      </ToastProvider>,
    );
    act(() => {
      useToastStore.getState().push({ kind: 'ok', title: 'Saved' });
    });
    expect(screen.getByText('Saved')).toBeInTheDocument();
    act(() => {
      vi.advanceTimersByTime(4200);
    });
    expect(screen.queryByText('Saved')).not.toBeInTheDocument();
  });

  it('a_long_message_extends_the_ttl_to_6s', () => {
    render(
      <ToastProvider>
        <div />
      </ToastProvider>,
    );
    const long = 'x'.repeat(120);
    act(() => {
      useToastStore.getState().push({ kind: 'info', title: 'Heads up', message: long });
    });
    // Still up past the short ttl — a long message gets longer to read.
    act(() => {
      vi.advanceTimersByTime(4200);
    });
    expect(screen.getByText('Heads up')).toBeInTheDocument();
    act(() => {
      vi.advanceTimersByTime(1800);
    });
    expect(screen.queryByText('Heads up')).not.toBeInTheDocument();
  });

  it('has_no_focusable_descendants_so_the_live_region_never_traps_focus', () => {
    render(
      <ToastProvider>
        <div />
      </ToastProvider>,
    );
    act(() => {
      useToastStore.getState().push({ kind: 'error', title: 'Failed', message: 'bad input' });
    });
    const region = screen.getByTestId('toast-stack');
    // A role=status live region must not contain interactive controls
    // (the stage's explicit a11y contract) — auto-dismiss only.
    expect(region.querySelectorAll('button, a, input, select, textarea, [tabindex]')).toHaveLength(
      0,
    );
  });

  it('useToast_push_surfaces_a_toast_from_a_consumer', async () => {
    function Harness(): JSX.Element {
      const { push } = useToast();
      return <button onClick={() => push({ kind: 'ok', title: 'Hi there' })}>go</button>;
    }
    // userEvent + fake timers do not interleave cleanly; this assertion is
    // about the push linkage, not the dismissal timer.
    vi.useRealTimers();
    const user = userEvent.setup();
    render(
      <ToastProvider>
        <Harness />
      </ToastProvider>,
    );
    await user.click(screen.getByText('go'));
    expect(screen.getByText('Hi there')).toBeInTheDocument();
  });
});
