import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const invokeMock = vi.fn(async (..._args: unknown[]) => undefined);

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { HITLToast, _testing } from '../../../src/components/HITLToast';
import { useGraphStore } from '../../../src/lib/graphStore';
import type { AgentEvent } from '../../../src/types/agent_event';

function dispatch(events: AgentEvent[]): void {
  act(() => {
    for (const e of events) {
      useGraphStore.getState().applyEvent(e);
    }
  });
}

const toastPrompt: AgentEvent = {
  type: 'hitl_requested',
  prompt_id: 't-1',
  trigger: 'per_task',
  agent_id: null,
  question: 'Approve task t-1?',
  options: ['ok', 'skip'],
  ui_variant: 'toast',
  timeout_at_unix_ms: 1_000_000,
};

describe('HITLToast', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
    useGraphStore.getState().clear();
  });

  afterEach(() => {
    vi.useRealTimers();
    useGraphStore.getState().clear();
  });

  it('returns_null_when_no_pending_toast', () => {
    const { container } = render(<HITLToast />);
    expect(container).toBeEmptyDOMElement();
  });

  it('renders_summary_button_when_collapsed_with_role_status', () => {
    dispatch([toastPrompt]);
    render(<HITLToast />);
    const toast = screen.getByTestId('hitl-toast');
    expect(toast).toHaveAttribute('role', 'status');
    expect(toast).toHaveAttribute('aria-live', 'polite');
    expect(screen.getByTestId('hitl-toast-summary')).toBeInTheDocument();
    expect(screen.queryByTestId('hitl-toast-question')).toBeNull();
  });

  it('clicking_summary_expands_to_show_question_and_options', () => {
    dispatch([toastPrompt]);
    render(<HITLToast />);
    act(() => {
      fireEvent.click(screen.getByTestId('hitl-toast-summary'));
    });
    expect(screen.getByTestId('hitl-toast-question')).toHaveTextContent('Approve task t-1?');
    expect(screen.getByTestId('hitl-toast-option-ok')).toBeInTheDocument();
    expect(screen.getByTestId('hitl-toast-option-skip')).toBeInTheDocument();
  });

  it('clicking_option_dispatches_respond_hitl', async () => {
    dispatch([toastPrompt]);
    render(<HITLToast />);
    act(() => {
      fireEvent.click(screen.getByTestId('hitl-toast-summary'));
    });
    act(() => {
      fireEvent.click(screen.getByTestId('hitl-toast-option-ok'));
    });
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('respond_hitl', {
        promptId: 't-1',
        choice: 'ok',
      });
    });
  });

  it('auto_dismisses_after_30_seconds_without_interaction', () => {
    vi.useFakeTimers();
    dispatch([toastPrompt]);
    render(<HITLToast />);
    expect(screen.getByTestId('hitl-toast')).toBeInTheDocument();
    act(() => {
      vi.advanceTimersByTime(_testing.TOAST_AUTO_DISMISS_MS + 1);
    });
    expect(screen.queryByTestId('hitl-toast')).toBeNull();
    // Auto-dismiss is renderer-local; no invoke fired.
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it('returns_null_when_only_panel_prompts_pending', () => {
    dispatch([
      {
        ...toastPrompt,
        ui_variant: 'panel',
      },
    ]);
    render(<HITLToast />);
    expect(screen.queryByTestId('hitl-toast')).toBeNull();
  });

  it('renders_error_text_when_invoke_rejects', async () => {
    invokeMock.mockRejectedValueOnce({ type: 'internal', message: 'seam vanished' });
    dispatch([toastPrompt]);
    render(<HITLToast />);
    act(() => {
      fireEvent.click(screen.getByTestId('hitl-toast-summary'));
    });
    act(() => {
      fireEvent.click(screen.getByTestId('hitl-toast-option-skip'));
    });
    await waitFor(() => {
      expect(screen.getByText(/internal: seam vanished/i)).toBeInTheDocument();
    });
  });

  it('firstToastPrompt_returns_null_when_no_toast_variant', () => {
    expect(_testing.firstToastPrompt({})).toBeNull();
    expect(
      _testing.firstToastPrompt({
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

  it('firstToastPrompt_returns_first_toast_prompt', () => {
    expect(
      _testing.firstToastPrompt({
        a: {
          promptId: 'a',
          trigger: 'per_task',
          agentId: null,
          question: '?',
          options: [],
          uiVariant: 'toast',
          timeoutAtUnixMs: 1,
        },
      }),
    ).toMatchObject({ promptId: 'a' });
  });
});
