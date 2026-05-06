import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const invokeMock = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SqlInspector } from '../../../src/components/SqlInspector';

describe('SqlInspector', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('renders_default_sql_textarea_and_execute_button', () => {
    render(<SqlInspector />);
    const textarea = screen.getByLabelText(/sql query/i);
    expect(textarea).toBeInTheDocument();
    expect(textarea).toHaveValue('SELECT * FROM signals LIMIT 10;');
    expect(screen.getByRole('button', { name: /execute/i })).toBeInTheDocument();
  });

  it('user_typing_then_execute_invokes_query_session_db_with_sql', async () => {
    invokeMock.mockResolvedValueOnce([{ id: 1 }]);
    const user = userEvent.setup();
    render(<SqlInspector />);
    const textarea = screen.getByLabelText(/sql query/i);
    await user.clear(textarea);
    await user.type(textarea, 'SELECT id FROM signals');
    await user.click(screen.getByRole('button', { name: /execute/i }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('query_session_db', {
        sql: 'SELECT id FROM signals',
      });
    });
  });

  it('renders_results_table_on_success', async () => {
    invokeMock.mockResolvedValueOnce([
      { id: 'sig-1', type: 'tool' },
      { id: 'sig-2', type: 'decision' },
    ]);
    const user = userEvent.setup();
    render(<SqlInspector />);
    await user.click(screen.getByRole('button', { name: /execute/i }));
    await screen.findByText('sig-1');
    expect(screen.getByText('sig-2')).toBeInTheDocument();
    expect(screen.getByText('tool')).toBeInTheDocument();
    expect(screen.getByText('decision')).toBeInTheDocument();
  });

  it('renders_error_paragraph_on_rejection', async () => {
    invokeMock.mockRejectedValueOnce({
      type: 'internal',
      message: 'only SELECT statements permitted',
    });
    const user = userEvent.setup();
    render(<SqlInspector />);
    await user.click(screen.getByRole('button', { name: /execute/i }));
    const err = await screen.findByRole('alert');
    expect(err.textContent).toMatch(/only SELECT/i);
  });

  it('disables_execute_while_query_is_in_flight', async () => {
    let resolve!: (value: unknown[]) => void;
    invokeMock.mockReturnValueOnce(
      new Promise<unknown[]>((r) => {
        resolve = r;
      }),
    );
    const user = userEvent.setup();
    render(<SqlInspector />);
    const button = screen.getByRole('button', { name: /execute/i });
    await user.click(button);
    // While the promise is pending, the button must be disabled (debounce
    // discipline — rapid Execute clicks fire only one IPC call).
    await waitFor(() => expect(button).toBeDisabled());
    await user.click(button);
    await user.click(button);
    expect(invokeMock).toHaveBeenCalledTimes(1);
    resolve([]);
    await waitFor(() => expect(button).toBeEnabled());
  });
});
