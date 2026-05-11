import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const invokeMock = vi.fn(async (..._args: unknown[]) => undefined);

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

import { act, fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { HITLModal, _testing } from '../../../src/components/HITLModal';
import { useGraphStore } from '../../../src/lib/graphStore';
import type { AgentEvent } from '../../../src/types/agent_event';

function dispatch(events: AgentEvent[]): void {
  act(() => {
    for (const e of events) {
      useGraphStore.getState().applyEvent(e);
    }
  });
}

const modalPrompt: AgentEvent = {
  type: 'hitl_requested',
  prompt_id: 'm-1',
  trigger: 'on_risky_tool',
  agent_id: 'a1',
  question: 'Run Bash:rm -rf /tmp/foo?',
  options: ['allow', 'block'],
  ui_variant: 'modal',
  timeout_at_unix_ms: 9_999,
};

describe('HITLModal', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
    useGraphStore.getState().clear();
  });

  afterEach(() => {
    useGraphStore.getState().clear();
  });

  it('returns_null_when_no_pending_hitl', () => {
    const { container } = render(<HITLModal />);
    expect(container).toBeEmptyDOMElement();
  });

  it('renders_modal_dialog_with_aria_modal_true', () => {
    dispatch([modalPrompt]);
    render(<HITLModal />);
    const dialog = screen.getByTestId('hitl-modal');
    expect(dialog).toHaveAttribute('role', 'dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    expect(dialog).toHaveAttribute('aria-labelledby', 'hitl-modal-title');
    expect(dialog).toHaveAttribute('aria-describedby', 'hitl-modal-question');
  });

  it('renders_question_and_trigger_label', () => {
    dispatch([modalPrompt]);
    render(<HITLModal />);
    expect(screen.getByTestId('hitl-modal-question')).toHaveTextContent('Bash:rm');
    expect(screen.getByTestId('hitl-modal-trigger')).toHaveTextContent('on_risky_tool');
  });

  it('clicking_allow_dispatches_respond_hitl_with_choice_allow', async () => {
    dispatch([modalPrompt]);
    const user = userEvent.setup();
    render(<HITLModal />);
    await user.click(screen.getByTestId('hitl-modal-option-allow'));
    expect(invokeMock).toHaveBeenCalledWith('respond_hitl', { promptId: 'm-1', choice: 'allow' });
  });

  it('escape_key_dismisses_modal_without_dispatching_invoke', () => {
    dispatch([modalPrompt]);
    render(<HITLModal />);
    expect(screen.getByTestId('hitl-modal')).toBeInTheDocument();
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(screen.queryByTestId('hitl-modal')).toBeNull();
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it('returns_null_when_only_panel_prompts_pending', () => {
    dispatch([
      {
        ...modalPrompt,
        ui_variant: 'panel',
      },
    ]);
    render(<HITLModal />);
    expect(screen.queryByTestId('hitl-modal')).toBeNull();
  });

  it('renders_error_text_when_invoke_rejects', async () => {
    invokeMock.mockRejectedValueOnce({ type: 'internal', message: 'seam vanished' });
    dispatch([modalPrompt]);
    const user = userEvent.setup();
    render(<HITLModal />);
    await user.click(screen.getByTestId('hitl-modal-option-block'));
    expect(await screen.findByText(/internal: seam vanished/i)).toBeInTheDocument();
  });

  it('firstModalPrompt_returns_null_when_no_modal_variant', () => {
    expect(_testing.firstModalPrompt({})).toBeNull();
    expect(
      _testing.firstModalPrompt({
        p: {
          promptId: 'p',
          trigger: 'on_gap',
          agentId: null,
          question: '?',
          options: [],
          uiVariant: 'panel',
          timeoutAtUnixMs: 1,
        },
      }),
    ).toBeNull();
  });

  it('firstModalPrompt_returns_first_modal_prompt', () => {
    expect(
      _testing.firstModalPrompt({
        a: {
          promptId: 'a',
          trigger: 'on_risky_tool',
          agentId: null,
          question: '?',
          options: [],
          uiVariant: 'modal',
          timeoutAtUnixMs: 1,
        },
      }),
    ).toMatchObject({ promptId: 'a' });
  });
});
