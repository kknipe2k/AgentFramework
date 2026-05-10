import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const invokeMock = vi.fn(async (..._args: unknown[]) => undefined);

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

import { act, fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ApprovalPanel } from '../../../src/components/ApprovalPanel';
import { useGraphStore } from '../../../src/lib/graphStore';
import type { AgentEvent } from '../../../src/types/agent_event';

function dispatch(events: AgentEvent[]): void {
  act(() => {
    for (const e of events) {
      useGraphStore.getState().applyEvent(e);
    }
  });
}

const planCreated: AgentEvent = {
  type: 'plan_created',
  plan_id: 'p1',
  title: 'Refactor auth flow',
  task_count: 3,
  approval_required: true,
};
const approvalRequested: AgentEvent = { type: 'plan_approval_requested', plan_id: 'p1' };

describe('ApprovalPanel', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
    useGraphStore.getState().clear();
  });

  afterEach(() => {
    useGraphStore.getState().clear();
  });

  it('returns_null_when_no_plan_is_awaiting_approval', () => {
    const { container } = render(<ApprovalPanel />);
    expect(container).toBeEmptyDOMElement();
    expect(screen.queryByTestId('approval-panel')).toBeNull();
  });

  it('returns_null_when_plan_is_pending_approval_but_not_yet_requested', () => {
    // plan_created with approval_required=true sets status='pending_approval'
    // but the SDK has not yet emitted plan_approval_requested. The panel
    // surfaces only on awaiting_approval (the explicit "user, please act"
    // signal), not on the intermediate pending_approval state.
    dispatch([planCreated]);
    render(<ApprovalPanel />);
    expect(screen.queryByTestId('approval-panel')).toBeNull();
  });

  it('renders_panel_when_a_plan_transitions_to_awaiting_approval', () => {
    dispatch([planCreated, approvalRequested]);
    render(<ApprovalPanel />);
    const panel = screen.getByTestId('approval-panel');
    expect(panel).toBeInTheDocument();
    expect(panel.textContent).toContain('Refactor auth flow');
    expect(panel.textContent).toMatch(/3 tasks?/);
  });

  it('exposes_aria_region_attributes_with_aria_modal_false', () => {
    dispatch([planCreated, approvalRequested]);
    render(<ApprovalPanel />);
    const panel = screen.getByTestId('approval-panel');
    expect(panel).toHaveAttribute('role', 'region');
    expect(panel).toHaveAttribute('aria-label', expect.stringMatching(/plan approval/i));
    expect(panel).toHaveAttribute('aria-modal', 'false');
  });

  it('approve_button_dispatches_invokeApprovePlan_with_plan_id', async () => {
    dispatch([planCreated, approvalRequested]);
    const user = userEvent.setup();
    render(<ApprovalPanel />);
    await user.click(screen.getByRole('button', { name: /^approve$/i }));
    expect(invokeMock).toHaveBeenCalledWith('approve_plan', { planId: 'p1' });
  });

  it('revise_button_opens_textarea_then_submit_dispatches_revisions', async () => {
    dispatch([planCreated, approvalRequested]);
    const user = userEvent.setup();
    render(<ApprovalPanel />);
    await user.click(screen.getByRole('button', { name: /^revise$/i }));
    const textarea = await screen.findByLabelText(/revisions/i);
    await user.type(textarea, 'add risk callouts');
    await user.click(screen.getByRole('button', { name: /submit revisions/i }));
    expect(invokeMock).toHaveBeenCalledWith('revise_plan', {
      planId: 'p1',
      revisions: 'add risk callouts',
    });
  });

  it('abort_button_opens_textarea_then_submit_dispatches_reason', async () => {
    dispatch([planCreated, approvalRequested]);
    const user = userEvent.setup();
    render(<ApprovalPanel />);
    await user.click(screen.getByRole('button', { name: /^cancel plan$/i }));
    const textarea = await screen.findByLabelText(/reason/i);
    await user.type(textarea, 'wrong scope');
    await user.click(screen.getByRole('button', { name: /confirm cancel/i }));
    expect(invokeMock).toHaveBeenCalledWith('abort_plan', {
      planId: 'p1',
      reason: 'wrong scope',
    });
  });

  it('escape_key_dismisses_without_aborting', () => {
    dispatch([planCreated, approvalRequested]);
    render(<ApprovalPanel />);
    expect(screen.getByTestId('approval-panel')).toBeInTheDocument();
    fireEvent.keyDown(window, { key: 'Escape' });
    // ESC dismisses the panel locally; it does NOT call abort_plan, the
    // SDK keeps awaiting per spec §3a (user can return to the plan).
    expect(screen.queryByTestId('approval-panel')).toBeNull();
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it('panel_dismisses_when_plan_status_transitions_to_in_progress', () => {
    dispatch([planCreated, approvalRequested]);
    const { rerender } = render(<ApprovalPanel />);
    expect(screen.getByTestId('approval-panel')).toBeInTheDocument();
    dispatch([{ type: 'plan_approved', plan_id: 'p1', approved_by: 'user' }]);
    rerender(<ApprovalPanel />);
    expect(screen.queryByTestId('approval-panel')).toBeNull();
  });

  it('renders_only_the_first_awaiting_approval_plan_when_multiple_pending', () => {
    // v0.1 single-session per spec §0d; multiple concurrent approval gates
    // are not in scope but the panel must not render multiple instances
    // even if synthetic state reaches that condition.
    const second: AgentEvent = {
      type: 'plan_created',
      plan_id: 'p2',
      title: 'Second plan',
      task_count: 1,
      approval_required: true,
    };
    dispatch([
      planCreated,
      approvalRequested,
      second,
      { type: 'plan_approval_requested', plan_id: 'p2' },
    ]);
    render(<ApprovalPanel />);
    expect(screen.getAllByTestId('approval-panel')).toHaveLength(1);
  });
});
