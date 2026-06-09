import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Modal } from '../../../src/components/Modal';

afterEach(cleanup);

function open(props: Record<string, unknown> = {}): { onClose: ReturnType<typeof vi.fn> } {
  const onClose = vi.fn();
  render(
    <Modal open onClose={onClose} title="Add MCP Server" {...props}>
      <p>body content</p>
    </Modal>,
  );
  return { onClose };
}

describe('Modal — reusable overlay primitive (M08.8.B; closes #24)', () => {
  it('renders_a_labelled_modal_dialog_with_its_title_and_children', () => {
    open();
    const dialog = screen.getByRole('dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    expect(screen.getByText('Add MCP Server')).toBeInTheDocument();
    expect(screen.getByText('body content')).toBeInTheDocument();
  });

  it('renders_nothing_when_closed', () => {
    const onClose = vi.fn();
    render(
      <Modal open={false} onClose={onClose} title="Hidden">
        <p>secret</p>
      </Modal>,
    );
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(screen.queryByText('secret')).toBeNull();
  });

  it('escape_closes', async () => {
    const user = userEvent.setup();
    const { onClose } = open();
    await user.keyboard('{Escape}');
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('clicking_the_scrim_closes_but_clicking_the_panel_does_not', async () => {
    const user = userEvent.setup();
    const { onClose } = open();
    await user.click(screen.getByText('body content'));
    expect(onClose).not.toHaveBeenCalled();
    await user.click(screen.getByTestId('modal-scrim'));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('the_close_button_closes', async () => {
    const user = userEvent.setup();
    const { onClose } = open();
    await user.click(screen.getByRole('button', { name: /close/i }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('the_scroll_body_is_bounded_to_86vh', () => {
    open();
    // Read the inline style directly — happy-dom's computed-style does not
    // resolve viewport units, but the bounded-scroll contract is the
    // inline max-height the component sets.
    expect(screen.getByTestId('modal-body').style.maxHeight).toBe('86vh');
  });

  it('moves_focus_into_the_dialog_on_open_and_restores_it_on_close', () => {
    const trigger = document.createElement('button');
    trigger.textContent = 'opener';
    document.body.appendChild(trigger);
    trigger.focus();
    expect(document.activeElement).toBe(trigger);

    const onClose = vi.fn();
    const { rerender } = render(
      <Modal open onClose={onClose} title="T">
        <input data-testid="field" />
      </Modal>,
    );
    expect(screen.getByRole('dialog').contains(document.activeElement)).toBe(true);

    rerender(
      <Modal open={false} onClose={onClose} title="T">
        <input data-testid="field" />
      </Modal>,
    );
    // Focus returns to the control that opened the modal (WAI APG).
    expect(document.activeElement).toBe(trigger);
    document.body.removeChild(trigger);
  });

  it('renders_untruncated_action_labels_in_the_footer', () => {
    const onClose = vi.fn();
    render(
      <Modal open onClose={onClose} title="T" footer={<button>Cancel</button>}>
        <p>x</p>
      </Modal>,
    );
    // #24 was a "Canc" truncation; the Modal must render the full label.
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeInTheDocument();
  });
});
