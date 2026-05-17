import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

const mcpAddServer = vi.fn();
const mcpTestConnection = vi.fn();
vi.mock('../../../src/lib/ipc', () => ({
  mcpAddServer: (...a: unknown[]) => mcpAddServer(...a),
  mcpTestConnection: (...a: unknown[]) => mcpTestConnection(...a),
  unwrapCmdError: (e: unknown) => String(e),
}));

import { MCPServerAddModal } from '../../../src/components/MCPServerAddModal';
import { useGraphStore } from '../../../src/lib/graphStore';

function renderModal(): { onClose: ReturnType<typeof vi.fn> } {
  const onClose = vi.fn();
  render(<MCPServerAddModal onClose={onClose} />);
  return { onClose };
}

describe('MCPServerAddModal (M06.E)', () => {
  beforeEach(() => {
    useGraphStore.setState({ currentTier: 'novice' });
    mcpAddServer.mockReset().mockResolvedValue(undefined);
    mcpTestConnection.mockReset().mockResolvedValue([]);
  });
  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it('validates_name_against_regex_disabling_submit_on_invalid', async () => {
    const user = userEvent.setup();
    renderModal();
    const name = screen.getByTestId('mcp-add-name');
    await user.type(name, 'Invalid Name!');
    expect(screen.getByTestId('mcp-add-submit')).toBeDisabled();
    await user.clear(name);
    await user.type(name, 'valid-name');
    expect(screen.getByTestId('mcp-add-submit')).toBeEnabled();
  });

  it('submit_with_stdio_transport_calls_mcpAddServer_with_correct_config', async () => {
    const user = userEvent.setup();
    renderModal();
    await user.type(screen.getByTestId('mcp-add-name'), 'filesystem');
    await user.type(screen.getByTestId('mcp-add-command'), 'npx');
    await user.click(screen.getByTestId('mcp-add-submit'));
    await waitFor(() => expect(mcpAddServer).toHaveBeenCalledTimes(1));
    const [config, auth] = mcpAddServer.mock.calls[0] as [Record<string, unknown>, unknown];
    expect(config.name).toBe('filesystem');
    expect(config.transport).toMatchObject({ type: 'stdio', command: 'npx' });
    expect(auth).toBeNull();
  });

  it('submit_with_http_transport_calls_mcpAddServer_with_url', async () => {
    const user = userEvent.setup();
    renderModal();
    await user.type(screen.getByTestId('mcp-add-name'), 'remote-api');
    await user.selectOptions(screen.getByTestId('mcp-add-transport'), 'http');
    await user.type(screen.getByTestId('mcp-add-url'), 'https://mcp.example.com/v1');
    await user.click(screen.getByTestId('mcp-add-submit'));
    await waitFor(() => expect(mcpAddServer).toHaveBeenCalledTimes(1));
    const [config] = mcpAddServer.mock.calls[0] as [Record<string, unknown>];
    expect(config.transport).toMatchObject({
      type: 'http',
      url: 'https://mcp.example.com/v1',
    });
  });

  it('parses_args_csv_into_array', async () => {
    const user = userEvent.setup();
    renderModal();
    await user.type(screen.getByTestId('mcp-add-name'), 'filesystem');
    await user.type(screen.getByTestId('mcp-add-command'), 'npx');
    await user.type(screen.getByTestId('mcp-add-args'), '-y, @scope/server ,/tmp');
    await user.click(screen.getByTestId('mcp-add-submit'));
    await waitFor(() => expect(mcpAddServer).toHaveBeenCalledTimes(1));
    const [config] = mcpAddServer.mock.calls[0] as [{ transport: { args: string[] } }];
    expect(config.transport.args).toEqual(['-y', '@scope/server', '/tmp']);
  });

  it('parses_env_lines_into_record', async () => {
    const user = userEvent.setup();
    renderModal();
    await user.type(screen.getByTestId('mcp-add-name'), 'filesystem');
    await user.type(screen.getByTestId('mcp-add-command'), 'npx');
    await user.type(screen.getByTestId('mcp-add-env'), 'API_KEY=abc123{enter}LOG=debug');
    await user.click(screen.getByTestId('mcp-add-submit'));
    await waitFor(() => expect(mcpAddServer).toHaveBeenCalledTimes(1));
    const [config] = mcpAddServer.mock.calls[0] as [{ transport: { env: Record<string, string> } }];
    expect(config.transport.env).toEqual({ API_KEY: 'abc123', LOG: 'debug' });
  });

  it('clicking_test_calls_mcpTestConnection_and_displays_tool_list', async () => {
    const user = userEvent.setup();
    mcpTestConnection.mockResolvedValueOnce([
      { name: 'read_file', input_schema: {} },
      { name: 'write_file', input_schema: {} },
    ]);
    renderModal();
    await user.type(screen.getByTestId('mcp-add-name'), 'filesystem');
    await user.type(screen.getByTestId('mcp-add-command'), 'npx');
    await user.click(screen.getByTestId('mcp-add-test'));
    await waitFor(() => expect(mcpTestConnection).toHaveBeenCalledTimes(1));
    // Re-query after the await (gotcha #27).
    const list = await screen.findByTestId('mcp-add-tool-list');
    expect(list).toHaveTextContent('read_file');
    expect(list).toHaveTextContent('write_file');
  });

  it('displays_tier_eval_outcome_novice_requires_promoted', () => {
    useGraphStore.setState({ currentTier: 'novice' });
    renderModal();
    expect(screen.getByTestId('mcp-add-tier-eval')).toHaveTextContent(/requires promoted/i);
  });

  it('displays_tier_eval_outcome_promoted_auto_accepts', () => {
    useGraphStore.setState({ currentTier: 'promoted' });
    renderModal();
    expect(screen.getByTestId('mcp-add-tier-eval')).toHaveTextContent(/auto-accept/i);
  });
});
