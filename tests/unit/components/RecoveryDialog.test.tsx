import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const invokeMock = vi.fn(async (..._args: unknown[]): Promise<unknown> => undefined);

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { RecoveryDialog } from '../../../src/components/RecoveryDialog';
import { useGraphStore } from '../../../src/lib/graphStore';

describe('RecoveryDialog', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue({
      snapshot_id: 'snap-1',
      plans: [],
      tasks: [],
      uncertain_tool_invocations: [],
      has_state: true,
    });
    localStorage.clear();
    useGraphStore.getState().clear();
  });

  afterEach(() => {
    localStorage.clear();
    useGraphStore.getState().clear();
  });

  it('returns_null_when_no_prior_session_in_localStorage', () => {
    const { container } = render(<RecoveryDialog />);
    expect(container).toBeEmptyDOMElement();
  });

  it('renders_dialog_when_prior_session_id_present', () => {
    localStorage.setItem('lastSessionId', 's1');
    render(<RecoveryDialog />);
    expect(screen.getByTestId('recovery-dialog')).toBeInTheDocument();
    expect(screen.getByTestId('recovery-dialog-session-id')).toHaveTextContent('s1');
  });

  it('aria_dialog_attributes_present', () => {
    localStorage.setItem('lastSessionId', 's1');
    render(<RecoveryDialog />);
    const dialog = screen.getByTestId('recovery-dialog');
    expect(dialog).toHaveAttribute('role', 'dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    expect(dialog).toHaveAttribute('aria-labelledby', 'recovery-dialog-title');
  });

  it('resume_dispatches_request_resume_and_dismisses', async () => {
    localStorage.setItem('lastSessionId', 's1');
    render(<RecoveryDialog />);
    fireEvent.click(screen.getByTestId('recovery-dialog-resume'));
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith('request_resume', { sessionId: 's1' }),
    );
    await waitFor(() => expect(screen.queryByTestId('recovery-dialog')).toBeNull());
  });

  it('resume_populates_uncertain_invocations_list', async () => {
    localStorage.setItem('lastSessionId', 's1');
    invokeMock.mockResolvedValueOnce({
      snapshot_id: 'snap-1',
      plans: [],
      tasks: [],
      uncertain_tool_invocations: ['sig-tool-1', 'sig-tool-2'],
      has_state: true,
    });
    render(<RecoveryDialog />);
    fireEvent.click(screen.getByTestId('recovery-dialog-resume'));
    await waitFor(() => {
      expect(useGraphStore.getState().uncertainInvocations).toHaveLength(2);
    });
    const invocationIds = useGraphStore.getState().uncertainInvocations.map((u) => u.invocationId);
    expect(invocationIds).toEqual(['sig-tool-1', 'sig-tool-2']);
  });

  it('discard_clears_localStorage_and_dismisses_without_resume_call', () => {
    localStorage.setItem('lastSessionId', 's1');
    render(<RecoveryDialog />);
    fireEvent.click(screen.getByTestId('recovery-dialog-discard'));
    expect(localStorage.getItem('lastSessionId')).toBeNull();
    expect(screen.queryByTestId('recovery-dialog')).toBeNull();
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it('resume_surfaces_invoke_error_inline_without_dismissing', async () => {
    localStorage.setItem('lastSessionId', 's1');
    invokeMock.mockReset();
    invokeMock.mockRejectedValueOnce({ type: 'drone', message: 'transport down' });
    render(<RecoveryDialog />);
    fireEvent.click(screen.getByTestId('recovery-dialog-resume'));
    await waitFor(() =>
      expect(screen.getByTestId('recovery-dialog-error')).toHaveTextContent('transport down'),
    );
    // Dialog stays mounted so the user can retry.
    expect(screen.getByTestId('recovery-dialog')).toBeInTheDocument();
  });

  it('onClose_callback_fires_on_resume', async () => {
    localStorage.setItem('lastSessionId', 's1');
    const onClose = vi.fn();
    render(<RecoveryDialog onClose={onClose} />);
    fireEvent.click(screen.getByTestId('recovery-dialog-resume'));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
  });

  it('onClose_callback_fires_on_discard', () => {
    localStorage.setItem('lastSessionId', 's1');
    const onClose = vi.fn();
    render(<RecoveryDialog onClose={onClose} />);
    fireEvent.click(screen.getByTestId('recovery-dialog-discard'));
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
