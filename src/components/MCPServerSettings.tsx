import { useState } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { MCPServerAddModal } from './MCPServerAddModal';
import { mcpRemoveServer, unwrapCmdError } from '../lib/ipc';
import { useGraphStore } from '../lib/graphStore';

/**
 * Settings → MCP Servers panel (M06.E). Lists installed servers from the
 * store's `currentMcpServers` (driven by the M06.C lifecycle events),
 * with per-row status + a Remove affordance and an Add-server modal.
 *
 * No Settings-tab infrastructure exists in v0.1 — App.tsx mounts panels
 * directly into `.graph-layout`, so this renders as a sibling panel like
 * GapPanel (reconciled against actual DOM; the E.4.4 phase-doc
 * `[data-test=open-settings]` tab pseudocode drifted).
 *
 * Per-row Test is intentionally absent: `mcp_test_connection` (Stage C)
 * takes a full `McpServerConfig`, but neither `currentMcpServers` nor
 * `McpServerSummary` carries the transport command/url — Test lives in
 * the Add modal where the form supplies the config. Surfacing a
 * registry-config-returning command is a Stage C retroactive add, not a
 * Stage E introduction (per the execution_warnings).
 *
 * Per gotcha #75: the derived `Object.values` selector is
 * `useShallow`-wrapped.
 */
export function MCPServerSettings(): JSX.Element {
  const servers = useGraphStore(useShallow((s) => Object.values(s.currentMcpServers)));
  const [showAdd, setShowAdd] = useState(false);
  const [confirmRemove, setConfirmRemove] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function handleRemove(name: string): Promise<void> {
    try {
      await mcpRemoveServer(name);
      setConfirmRemove(null);
    } catch (e) {
      console.error('mcp_remove_server error:', e);
      setError(unwrapCmdError(e));
    }
  }

  return (
    <section className="mcp-server-settings" data-testid="mcp-server-settings">
      <header className="mcp-server-settings__header">
        <h2 className="mcp-server-settings__title">MCP Servers</h2>
        <button data-testid="mcp-add-server-button" onClick={() => setShowAdd(true)}>
          Add Server
        </button>
      </header>
      {error !== null && <p className="mcp-server-settings__error">{error}</p>}
      {servers.length === 0 ? (
        <p className="mcp-server-settings__empty" data-testid="mcp-server-settings-empty">
          No MCP servers installed.
        </p>
      ) : (
        <ul className="mcp-server-list">
          {servers.map((s) => (
            <li
              key={s.name}
              className={`mcp-server-row mcp-server-row--${s.status}`}
              data-testid={`mcp-server-row-${s.name}`}
            >
              <span className="mcp-server-row__name">{s.name}</span>
              <span className="mcp-server-row__transport">{s.transportKind ?? 'unknown'}</span>
              <span className="mcp-server-row__status">{s.status}</span>
              {confirmRemove === s.name ? (
                <button
                  className="mcp-server-row__confirm"
                  data-testid={`mcp-remove-confirm-${s.name}`}
                  onClick={() => void handleRemove(s.name)}
                >
                  Confirm remove
                </button>
              ) : (
                <button
                  className="mcp-server-row__remove"
                  data-testid={`mcp-remove-${s.name}`}
                  onClick={() => setConfirmRemove(s.name)}
                >
                  Remove
                </button>
              )}
            </li>
          ))}
        </ul>
      )}
      {showAdd && <MCPServerAddModal onClose={() => setShowAdd(false)} />}
    </section>
  );
}
