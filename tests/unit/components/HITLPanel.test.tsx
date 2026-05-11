import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const invokeMock = vi.fn(async (..._args: unknown[]) => undefined);

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

import { act, fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { HITLPanel, _testing } from '../../../src/components/HITLPanel';
import { useGraphStore } from '../../../src/lib/graphStore';
import type { AgentEvent } from '../../../src/types/agent_event';

function dispatch(events: AgentEvent[]): void {
  act(() => {
    for (const e of events) {
      useGraphStore.getState().applyEvent(e);
    }
  });
}

const panelPrompt: AgentEvent = {
  type: 'hitl_requested',
  prompt_id: 'u-1',
  trigger: 'on_failure_threshold',
  agent_id: null,
  question: 'Task t-1 exceeded failure budget. Retry, skip, or abort?',
  options: ['retry', 'skip', 'abort'],
  ui_variant: 'panel',
  timeout_at_unix_ms: 1_000_000,
};
const modalPrompt: AgentEvent = {
  type: 'hitl_requested',
  prompt_id: 'u-2',
  trigger: 'on_risky_tool',
  agent_id: 'a1',
  question: 'Run Bash:rm?',
  options: ['allow', 'block'],
  ui_variant: 'modal',
  timeout_at_unix_ms: 1_000_000,
};

describe('HITLPanel', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
    useGraphStore.getState().clear();
  });

  afterEach(() => {
    useGraphStore.getState().clear();
  });

  it('returns_null_when_no_pending_hitl', () => {
    const { container } = render(<HITLPanel />);
    expect(container).toBeEmptyDOMElement();
    expect(screen.queryByTestId('hitl-panel')).toBeNull();
  });

  it('returns_null_when_only_non_panel_prompts_pending', () => {
    dispatch([modalPrompt]);
    render(<HITLPanel />);
    expect(screen.queryByTestId('hitl-panel')).toBeNull();
  });

  it('renders_panel_with_question_and_trigger_label', () => {
    dispatch([panelPrompt]);
    render(<HITLPanel />);
    expect(screen.getByTestId('hitl-panel')).toBeInTheDocument();
    expect(screen.getByTestId('hitl-panel-question')).toHaveTextContent('exceeded failure budget');
    expect(screen.getByTestId('hitl-panel-trigger')).toHaveTextContent('on_failure_threshold');
  });

  it('exposes_aria_region_attributes_with_aria_modal_false', () => {
    dispatch([panelPrompt]);
    render(<HITLPanel />);
    const panel = screen.getByTestId('hitl-panel');
    expect(panel).toHaveAttribute('role', 'region');
    expect(panel).toHaveAttribute('aria-modal', 'false');
    expect(panel).toHaveAttribute('aria-label', expect.stringContaining('on_failure_threshold'));
  });

  it('renders_one_button_per_option_when_options_are_present', () => {
    dispatch([panelPrompt]);
    render(<HITLPanel />);
    expect(screen.getByTestId('hitl-panel-option-retry')).toBeInTheDocument();
    expect(screen.getByTestId('hitl-panel-option-skip')).toBeInTheDocument();
    expect(screen.getByTestId('hitl-panel-option-abort')).toBeInTheDocument();
  });

  it('clicking_skip_dispatches_invokeRespondHitl_with_prompt_id_and_choice', async () => {
    dispatch([panelPrompt]);
    const user = userEvent.setup();
    render(<HITLPanel />);
    await user.click(screen.getByTestId('hitl-panel-option-skip'));
    expect(invokeMock).toHaveBeenCalledWith('respond_hitl', { promptId: 'u-1', choice: 'skip' });
  });

  it('clicking_retry_dispatches_choice_retry', async () => {
    dispatch([panelPrompt]);
    const user = userEvent.setup();
    render(<HITLPanel />);
    await user.click(screen.getByTestId('hitl-panel-option-retry'));
    expect(invokeMock).toHaveBeenCalledWith('respond_hitl', { promptId: 'u-1', choice: 'retry' });
  });

  it('renders_textarea_form_when_options_are_empty', () => {
    dispatch([
      {
        ...panelPrompt,
        options: [],
      },
    ]);
    render(<HITLPanel />);
    expect(screen.getByTestId('hitl-panel-textarea')).toBeInTheDocument();
    expect(screen.getByTestId('hitl-panel-submit')).toBeDisabled();
  });

  it('typing_in_textarea_enables_submit_and_dispatches_with_typed_text', async () => {
    dispatch([
      {
        ...panelPrompt,
        options: [],
      },
    ]);
    const user = userEvent.setup();
    render(<HITLPanel />);
    await user.type(screen.getByTestId('hitl-panel-textarea'), 'free text response');
    const submit = screen.getByTestId('hitl-panel-submit');
    expect(submit).toBeEnabled();
    await user.click(submit);
    expect(invokeMock).toHaveBeenCalledWith('respond_hitl', {
      promptId: 'u-1',
      choice: 'free text response',
    });
  });

  it('escape_key_dismisses_panel_without_dispatching_invoke', () => {
    dispatch([panelPrompt]);
    render(<HITLPanel />);
    expect(screen.getByTestId('hitl-panel')).toBeInTheDocument();
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(screen.queryByTestId('hitl-panel')).toBeNull();
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it('panel_unmounts_when_prompt_resolves', () => {
    dispatch([panelPrompt]);
    const { rerender } = render(<HITLPanel />);
    expect(screen.getByTestId('hitl-panel')).toBeInTheDocument();
    dispatch([{ type: 'hitl_resolved', prompt_id: 'u-1', choice: 'skip', duration_ms: 100 }]);
    rerender(<HITLPanel />);
    expect(screen.queryByTestId('hitl-panel')).toBeNull();
  });

  it('renders_error_text_when_invoke_rejects', async () => {
    invokeMock.mockRejectedValueOnce({ type: 'internal', message: 'seam vanished' });
    dispatch([panelPrompt]);
    const user = userEvent.setup();
    render(<HITLPanel />);
    await user.click(screen.getByTestId('hitl-panel-option-abort'));
    // unwrapCmdError formats internal-type errors as "internal: <msg>"
    expect(await screen.findByText(/internal: seam vanished/i)).toBeInTheDocument();
  });

  it('firstPanelPrompt_returns_null_when_no_panel_variant_pending', () => {
    expect(_testing.firstPanelPrompt({})).toBeNull();
    expect(
      _testing.firstPanelPrompt({
        m: {
          promptId: 'm',
          trigger: 'on_risky_tool',
          agentId: null,
          question: '?',
          options: [],
          uiVariant: 'modal',
          timeoutAtUnixMs: 1,
        },
      }),
    ).toBeNull();
  });

  it('firstPanelPrompt_returns_first_panel_when_multiple_pending', () => {
    expect(
      _testing.firstPanelPrompt({
        a: {
          promptId: 'a',
          trigger: 'on_gap',
          agentId: null,
          question: '?',
          options: [],
          uiVariant: 'panel',
          timeoutAtUnixMs: 1,
        },
      }),
    ).toMatchObject({ promptId: 'a' });
  });
});
