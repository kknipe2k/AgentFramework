import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const invokeMock = vi.fn(async (..._args: unknown[]): Promise<unknown> => undefined);

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { UncertaintyPrompt, _testing } from '../../../src/components/UncertaintyPrompt';
import { useGraphStore } from '../../../src/lib/graphStore';

function seedUncertain(
  invocations: { invocationId: string; toolName?: string; agentId?: string }[],
): void {
  act(() => {
    const r = useGraphStore.getState().recordUncertainInvocation;
    for (const i of invocations) {
      r(i);
    }
  });
}

describe('UncertaintyPrompt', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue({
      signal_id: 'sig-1',
      action: 'skip',
      invocation_id: 'sig-tool-1',
    });
    useGraphStore.getState().clear();
  });

  afterEach(() => {
    useGraphStore.getState().clear();
  });

  it('returns_null_when_no_uncertain_invocations', () => {
    const { container } = render(<UncertaintyPrompt sessionId="s1" />);
    expect(container).toBeEmptyDOMElement();
  });

  it('renders_first_invocation_with_aria_dialog', () => {
    seedUncertain([{ invocationId: 'sig-tool-1', toolName: 'Read', agentId: 'a1' }]);
    render(<UncertaintyPrompt sessionId="s1" />);
    const dialog = screen.getByTestId('uncertainty-prompt');
    expect(dialog).toHaveAttribute('role', 'dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    expect(screen.getByTestId('uncertainty-prompt-invocation-id')).toHaveTextContent('sig-tool-1');
  });

  it('renders_all_four_action_buttons', () => {
    seedUncertain([{ invocationId: 'sig-tool-1' }]);
    render(<UncertaintyPrompt sessionId="s1" />);
    for (const a of ['retry', 'skip', 'mark', 'abort']) {
      expect(screen.getByTestId(`uncertainty-action-${a}`)).toBeInTheDocument();
    }
  });

  it('clicking_skip_dispatches_respond_uncertainty_with_skip', async () => {
    seedUncertain([{ invocationId: 'sig-tool-1', agentId: 'a1' }]);
    render(<UncertaintyPrompt sessionId="s1" />);
    fireEvent.click(screen.getByTestId('uncertainty-action-skip'));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('respond_uncertainty', {
        sessionId: 's1',
        invocationId: 'sig-tool-1',
        action: 'skip',
        agentId: 'a1',
      });
    });
  });

  it('each_of_four_action_buttons_dispatches_distinct_token', async () => {
    seedUncertain([{ invocationId: 'sig-1' }]);
    render(<UncertaintyPrompt sessionId="s1" />);
    const cases: { testid: string; expected: string }[] = [
      { testid: 'uncertainty-action-retry', expected: 'retry' },
      { testid: 'uncertainty-action-skip', expected: 'skip' },
      { testid: 'uncertainty-action-mark', expected: 'mark_complete' },
      { testid: 'uncertainty-action-abort', expected: 'abort' },
    ];
    // After each click the invocation is removed; re-seed for the next.
    for (const c of cases) {
      seedUncertain([{ invocationId: 'sig-1' }]);
      fireEvent.click(screen.getByTestId(c.testid));
      await waitFor(() => {
        expect(invokeMock).toHaveBeenCalledWith(
          'respond_uncertainty',
          expect.objectContaining({ action: c.expected }),
        );
      });
      invokeMock.mockClear();
      invokeMock.mockResolvedValue({
        signal_id: 'sig-x',
        action: c.expected,
        invocation_id: 'sig-1',
      });
    }
  });

  it('resolves_invocation_from_store_after_successful_response', async () => {
    seedUncertain([{ invocationId: 'sig-tool-1' }]);
    render(<UncertaintyPrompt sessionId="s1" />);
    fireEvent.click(screen.getByTestId('uncertainty-action-skip'));
    await waitFor(() => {
      expect(useGraphStore.getState().uncertainInvocations).toHaveLength(0);
    });
  });

  it('advances_to_next_invocation_after_first_resolved', async () => {
    seedUncertain([{ invocationId: 'sig-1' }, { invocationId: 'sig-2' }]);
    render(<UncertaintyPrompt sessionId="s1" />);
    expect(screen.getByTestId('uncertainty-prompt-invocation-id')).toHaveTextContent('sig-1');
    expect(screen.getByTestId('uncertainty-prompt-remaining')).toHaveTextContent(
      '1 more invocation pending',
    );
    fireEvent.click(screen.getByTestId('uncertainty-action-skip'));
    await waitFor(() =>
      expect(screen.getByTestId('uncertainty-prompt-invocation-id')).toHaveTextContent('sig-2'),
    );
  });

  it('surfaces_invoke_error_inline_without_removing_invocation', async () => {
    seedUncertain([{ invocationId: 'sig-tool-1' }]);
    invokeMock.mockReset();
    invokeMock.mockRejectedValueOnce({ type: 'drone', message: 'transport down' });
    render(<UncertaintyPrompt sessionId="s1" />);
    fireEvent.click(screen.getByTestId('uncertainty-action-retry'));
    await waitFor(() =>
      expect(screen.getByTestId('uncertainty-prompt-error')).toHaveTextContent('transport down'),
    );
    // Invocation must not be removed on error.
    expect(useGraphStore.getState().uncertainInvocations).toHaveLength(1);
  });

  it('helper_ACTIONS_has_all_four_spec_actions', () => {
    const tokens = _testing.ACTIONS.map((a) => a.value);
    expect(tokens).toEqual(['retry', 'skip', 'mark_complete', 'abort']);
  });
});
