import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen } from '@testing-library/react';
import { ReactFlowProvider } from '@xyflow/react';
import { MCPNode } from '../../../src/components/nodes/MCPNode';
import { useGraphStore, type MCPNodeData } from '../../../src/lib/graphStore';

function renderMCP(data: MCPNodeData): ReturnType<typeof render> {
  return render(
    <ReactFlowProvider>
      <MCPNode
        id={`mcp:${data.serverId}`}
        type="mcp"
        data={data}
        dragging={false}
        zIndex={0}
        selectable
        deletable
        selected={false}
        draggable
        isConnectable
        positionAbsoluteX={0}
        positionAbsoluteY={0}
      />
    </ReactFlowProvider>,
  );
}

const baseData: MCPNodeData = {
  serverId: 'github-mcp',
  serverName: 'github-mcp',
  status: 'active',
  discoveredToolCount: null,
};

describe('MCPNode', () => {
  it('renders_server_name', () => {
    renderMCP(baseData);
    expect(screen.getByText('github-mcp')).toBeInTheDocument();
  });

  it('applies_active_complete_error_status_classes', () => {
    for (const status of ['active', 'complete', 'error'] as const) {
      const { unmount } = renderMCP({ ...baseData, status });
      const root = screen.getByTestId('mcp-node-github-mcp');
      expect(root.className).toContain(`mcp-node--${status}`);
      expect(root).toHaveAttribute('data-status', status);
      unmount();
    }
  });

  it('renders_tool_count_when_provided', () => {
    renderMCP({ ...baseData, discoveredToolCount: 7 });
    expect(screen.getByText(/7\s*tools?/i)).toBeInTheDocument();
  });

  it('exposes_accessible_aria_label_with_server_name_and_status', () => {
    renderMCP(baseData);
    const root = screen.getByTestId('mcp-node-github-mcp');
    expect(root).toHaveAttribute('aria-label', expect.stringMatching(/mcp.*github-mcp.*active/i));
  });

  it('renders_source_and_target_handle_elements', () => {
    renderMCP(baseData);
    const root = screen.getByTestId('mcp-node-github-mcp');
    const handles = root.querySelectorAll('.react-flow__handle');
    expect(handles.length).toBe(2);
  });
});

// ── M06.E — live wiring from currentMcpServers + activeMcpCalls ──────
//
// MCPNode (M03 stub) gains a live connection-status indicator sourced
// from the store's `currentMcpServers[serverName]` record (McpServerStatus:
// connected | disconnected | health_pending | error) — kept on a SEPARATE
// `mcp-node--conn-<status>` class family so it doesn't collide with the
// existing NodeStatus `mcp-node--<active|complete|error>` classes the 5
// tests above pin. Active-call animation reads the new `activeMcpCalls`
// slot. Per gotcha #75 the derived `currentMcpServers[serverName]` read
// is `useShallow`-wrapped (no infinite loop).
describe('MCPNode live wiring (M06.E)', () => {
  // currentMcpServers + activeMcpCalls persist across clear()
  // (registry-backed / per-session animation) — reset explicitly per
  // the v1.6 <test_isolation_audit> discipline.
  beforeEach(() => {
    useGraphStore.setState({ currentMcpServers: {}, activeMcpCalls: {} });
  });
  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  function setServer(status: string): void {
    useGraphStore.setState({
      currentMcpServers: {
        'github-mcp': { name: 'github-mcp', transportKind: 'stdio', hasAuth: false, status },
      },
    });
  }

  it('renders_live_connection_status_indicator_when_server_in_currentMcpServers', () => {
    setServer('connected');
    renderMCP(baseData);
    const root = screen.getByTestId('mcp-node-github-mcp');
    const indicator = root.querySelector('.mcp-node__status-indicator');
    expect(indicator).not.toBeNull();
    expect(indicator).toHaveAttribute('data-conn-status', 'connected');
    expect(root.className).toContain('mcp-node--conn-connected');
  });

  it('renders_status_indicator_per_connection_status_value', () => {
    for (const status of ['connected', 'disconnected', 'health_pending', 'error'] as const) {
      setServer(status);
      const { unmount } = renderMCP(baseData);
      const root = screen.getByTestId('mcp-node-github-mcp');
      expect(root.className).toContain(`mcp-node--conn-${status}`);
      const indicator = root.querySelector('.mcp-node__status-indicator');
      expect(indicator).toHaveAttribute('data-conn-status', status);
      unmount();
    }
  });

  it('falls_back_to_disconnected_conn_status_when_no_store_record', () => {
    renderMCP(baseData); // currentMcpServers empty
    const root = screen.getByTestId('mcp-node-github-mcp');
    expect(root.className).toContain('mcp-node--conn-disconnected');
    expect(root.querySelector('.mcp-node__status-indicator')).toHaveAttribute(
      'data-conn-status',
      'disconnected',
    );
  });

  it('renders_active_call_class_when_activeMcpCalls_has_serverName', () => {
    setServer('connected');
    useGraphStore.setState({ activeMcpCalls: { 'github-mcp': 'tool:a1:list_repos' } });
    renderMCP(baseData);
    const root = screen.getByTestId('mcp-node-github-mcp');
    expect(root.className).toContain('mcp-node--call-active');
  });

  it('no_active_call_class_when_activeMcpCalls_lacks_serverName', () => {
    setServer('connected');
    renderMCP(baseData); // activeMcpCalls empty
    const root = screen.getByTestId('mcp-node-github-mcp');
    expect(root.className).not.toContain('mcp-node--call-active');
  });

  it('uses_useShallow_for_derived_selector_no_infinite_loop', () => {
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    setServer('connected');
    renderMCP(baseData);
    // Re-set an equivalent-content record several times. With a naive
    // (non-useShallow) selector Zustand v5 returns a fresh reference each
    // store update → "Maximum update depth exceeded" (gotcha #75).
    for (let i = 0; i < 5; i += 1) {
      useGraphStore.setState({
        currentMcpServers: {
          'github-mcp': {
            name: 'github-mcp',
            transportKind: 'stdio',
            hasAuth: false,
            status: 'connected',
          },
        },
      });
    }
    expect(screen.getByTestId('mcp-node-github-mcp')).toBeInTheDocument();
    const loopErr = errorSpy.mock.calls.find((c) =>
      String(c[0]).includes('Maximum update depth'),
    );
    expect(loopErr).toBeUndefined();
  });

  it('every_mcp_node_live_class_has_a_corresponding_CSS_rule_in_styles_css', () => {
    // gotcha #67 — component renders className but styles.css may have no
    // rule. Static-assert each new class has a defined rule.
    const css = readFileSync(resolve(__dirname, '../../../src/styles.css'), 'utf8');
    const selectors = [
      '.mcp-node__status-indicator',
      '.mcp-node--conn-connected',
      '.mcp-node--conn-disconnected',
      '.mcp-node--conn-health_pending',
      '.mcp-node--conn-error',
      '.mcp-node--call-active',
    ];
    for (const sel of selectors) {
      expect(css, `missing CSS rule for ${sel}`).toContain(sel);
    }
  });
});
