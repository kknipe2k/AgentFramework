import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

const mcpRemoveServer = vi.fn();
const mcpTestConnection = vi.fn();
const mcpAddServer = vi.fn();
vi.mock('../../../src/lib/ipc', () => ({
  mcpRemoveServer: (...a: unknown[]) => mcpRemoveServer(...a),
  mcpTestConnection: (...a: unknown[]) => mcpTestConnection(...a),
  mcpAddServer: (...a: unknown[]) => mcpAddServer(...a),
  unwrapCmdError: (e: unknown) => String(e),
}));

import { MCPServerSettings } from '../../../src/components/MCPServerSettings';
import { useGraphStore, type McpServerStatusRecord } from '../../../src/lib/graphStore';

function setServers(records: McpServerStatusRecord[]): void {
  const map: Record<string, McpServerStatusRecord> = {};
  for (const r of records) map[r.name] = r;
  useGraphStore.setState({ currentMcpServers: map });
}

describe('MCPServerSettings (M06.E)', () => {
  beforeEach(() => {
    // currentMcpServers persists across clear() — reset per the v1.6
    // <test_isolation_audit> discipline.
    useGraphStore.setState({ currentMcpServers: {}, activeMcpCalls: {}, currentTier: 'novice' });
    mcpRemoveServer.mockReset().mockResolvedValue(undefined);
    mcpTestConnection.mockReset().mockResolvedValue([]);
    mcpAddServer.mockReset().mockResolvedValue(undefined);
  });
  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it('renders_empty_state_when_no_servers_installed', () => {
    render(<MCPServerSettings />);
    expect(screen.getByTestId('mcp-server-settings-empty')).toBeInTheDocument();
    expect(screen.getByText(/no mcp servers/i)).toBeInTheDocument();
  });

  it('renders_row_per_installed_server', () => {
    setServers([
      { name: 'filesystem', transportKind: 'stdio', hasAuth: false, status: 'connected' },
      { name: 'remote-api', transportKind: 'http', hasAuth: true, status: 'error' },
    ]);
    render(<MCPServerSettings />);
    expect(screen.getByTestId('mcp-server-row-filesystem')).toBeInTheDocument();
    expect(screen.getByTestId('mcp-server-row-remote-api')).toBeInTheDocument();
    expect(screen.getByText('filesystem')).toBeInTheDocument();
    expect(screen.getByText('remote-api')).toBeInTheDocument();
  });

  it('status_indicator_class_and_text_match_server_status', () => {
    // gotcha #66 contract — assert BOTH the modifier class AND the
    // user-visible status text, not just the className string.
    setServers([
      { name: 'filesystem', transportKind: 'stdio', hasAuth: false, status: 'health_pending' },
    ]);
    render(<MCPServerSettings />);
    const row = screen.getByTestId('mcp-server-row-filesystem');
    expect(row.className).toContain('mcp-server-row--health_pending');
    expect(row).toHaveTextContent(/health_pending/i);
  });

  it('clicking_add_opens_modal', async () => {
    const user = userEvent.setup();
    render(<MCPServerSettings />);
    expect(screen.queryByTestId('mcp-server-add-modal')).not.toBeInTheDocument();
    await user.click(screen.getByTestId('mcp-add-server-button'));
    // Re-query after the await (gotcha #27).
    expect(screen.getByTestId('mcp-server-add-modal')).toBeInTheDocument();
  });

  it('clicking_remove_shows_confirmation_then_calls_mcpRemoveServer', async () => {
    const user = userEvent.setup();
    setServers([
      { name: 'filesystem', transportKind: 'stdio', hasAuth: false, status: 'connected' },
    ]);
    render(<MCPServerSettings />);
    await user.click(screen.getByTestId('mcp-remove-filesystem'));
    // Confirmation surfaces; the IPC is NOT called until confirmed.
    expect(mcpRemoveServer).not.toHaveBeenCalled();
    const confirm = screen.getByTestId('mcp-remove-confirm-filesystem');
    await user.click(confirm);
    await waitFor(() => expect(mcpRemoveServer).toHaveBeenCalledWith('filesystem'));
  });

  it('survives_repeated_renders_with_currentMcpServers_mutation', () => {
    // gotcha #66 / #75 contract — repeated equivalent store writes must
    // not infinite-loop the useShallow-derived server list.
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    setServers([
      { name: 'filesystem', transportKind: 'stdio', hasAuth: false, status: 'connected' },
    ]);
    render(<MCPServerSettings />);
    for (let i = 0; i < 5; i += 1) {
      setServers([
        { name: 'filesystem', transportKind: 'stdio', hasAuth: false, status: 'connected' },
      ]);
    }
    expect(screen.getByTestId('mcp-server-row-filesystem')).toBeInTheDocument();
    expect(
      errorSpy.mock.calls.find((c) => String(c[0]).includes('Maximum update depth')),
    ).toBeUndefined();
  });

  it('every_mcp_server_class_has_a_corresponding_CSS_rule_in_styles_css', () => {
    const css = readFileSync(resolve(__dirname, '../../../src/styles.css'), 'utf8');
    const selectors = [
      '.mcp-server-settings',
      '.mcp-server-list',
      '.mcp-server-row',
      '.mcp-server-row--connected',
      '.mcp-server-row--disconnected',
      '.mcp-server-row--health_pending',
      '.mcp-server-row--error',
      '.mcp-server-row__name',
      '.mcp-server-row__status',
    ];
    for (const sel of selectors) {
      expect(css, `missing CSS rule for ${sel}`).toContain(sel);
    }
  });
});
