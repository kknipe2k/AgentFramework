import { useState } from 'react';
import { createPortal } from 'react-dom';
import { mcpAddServer, mcpTestConnection, unwrapCmdError, type McpTool } from '../lib/ipc';
import { useGraphStore } from '../lib/graphStore';
import type { McpServerConfig, McpTransport } from '../types/mcp';

interface MCPServerAddModalProps {
  onClose: () => void;
}

const NAME_RE = /^[a-z0-9][a-z0-9-]*$/;

/**
 * Add-server form (M06.E). A plain renderer modal — NOT an HITL seam
 * (ADR-0007): the Stage C `mcp_add_server` Tauri command does the
 * lifecycle heavy-lift; this component only collects + shapes the
 * {@link McpServerConfig}.
 *
 * Tier-gate display (spec §8.security L4): MCP tool calls run with the
 * Exec capability, which the Novice tier forbids (M06.D §5a capability
 * shape). The outcome is computed renderer-side from the store's
 * `currentTier` — same reuse pattern as M05.F CapabilityBadge; no MCP
 * tier Tauri command exists and none is added at Stage E.
 */
export function MCPServerAddModal({ onClose }: MCPServerAddModalProps): JSX.Element {
  const tier = useGraphStore((s) => s.currentTier);
  const [name, setName] = useState('');
  const [transport, setTransport] = useState<'stdio' | 'http'>('stdio');
  const [command, setCommand] = useState('');
  const [url, setUrl] = useState('');
  const [argsCsv, setArgsCsv] = useState('');
  const [envText, setEnvText] = useState('');
  const [auth, setAuth] = useState('');
  const [tools, setTools] = useState<McpTool[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  const nameValid = NAME_RE.test(name);

  function buildConfig(): McpServerConfig {
    let t: McpTransport;
    if (transport === 'http') {
      t = { type: 'http', url };
    } else {
      const args = argsCsv
        .split(',')
        .map((a) => a.trim())
        .filter((a) => a.length > 0);
      const env: Record<string, string> = {};
      for (const line of envText.split('\n')) {
        const trimmed = line.trim();
        if (trimmed.length === 0) continue;
        const eq = trimmed.indexOf('=');
        if (eq <= 0) continue;
        env[trimmed.slice(0, eq).trim()] = trimmed.slice(eq + 1).trim();
      }
      t = {
        type: 'stdio',
        command,
        ...(args.length > 0 ? { args } : {}),
        ...(Object.keys(env).length > 0 ? { env } : {}),
      };
    }
    return { name, transport: t };
  }

  async function handleSubmit(): Promise<void> {
    try {
      await mcpAddServer(buildConfig(), auth.trim().length > 0 ? auth : null);
      onClose();
    } catch (e) {
      console.error('mcp_add_server error:', e);
      setError(unwrapCmdError(e));
    }
  }

  async function handleTest(): Promise<void> {
    try {
      setTools(await mcpTestConnection(buildConfig()));
    } catch (e) {
      console.error('mcp_test_connection error:', e);
      setError(unwrapCmdError(e));
    }
  }

  const tierEval =
    tier === 'promoted'
      ? 'Promoted tier — MCP Exec tools auto-accept on install.'
      : 'MCP tools run with the Exec capability — install requires Promoted tier (Novice forbids Exec at §8.security L4).';

  // Render through a portal to `document.body` so the modal escapes
  // any ancestor stacking context — the parent path is
  // App → <main> → .graph-layout → MCPServerSettings → … and the
  // modal must overlay the entire viewport irrespective of how those
  // ancestors lay out (M08.5 🔴-3: the modal mounted inside the
  // .graph-layout flex tree had its buttons non-responsive in the
  // real Tauri/WebView2 app; the portal + the z-index + the
  // max-height/overflow hardening in styles.css are the defensive
  // triple that matches .import-review-modal's robust pattern). The
  // existing 8 MCPServerAddModal.test.tsx Vitest tests stay green —
  // RTL's `screen` queries from `document.body`, which IS the portal
  // target.
  const modal = (
    <div className="mcp-server-add-modal-backdrop" data-testid="mcp-server-add-modal-backdrop">
      <div
        className="mcp-server-add-modal"
        role="dialog"
        aria-modal="true"
        aria-label="Add MCP server"
        data-testid="mcp-server-add-modal"
      >
        <h2 className="mcp-server-add-modal__title">Add MCP Server</h2>
        <p
          className={`mcp-server-add-modal__tier-eval mcp-server-add-modal__tier-eval--${tier}`}
          data-testid="mcp-add-tier-eval"
        >
          {tierEval}
        </p>
        {error !== null && <p className="mcp-server-add-modal__error">{error}</p>}
        <form
          className="mcp-server-add-modal__form"
          onSubmit={(e) => {
            e.preventDefault();
            void handleSubmit();
          }}
        >
          <label className="mcp-server-add-modal__label">
            <span>Name</span>
            <input
              data-testid="mcp-add-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </label>
          <label className="mcp-server-add-modal__label">
            <span>Transport</span>
            <select
              data-testid="mcp-add-transport"
              value={transport}
              onChange={(e) => setTransport(e.target.value as 'stdio' | 'http')}
            >
              <option value="stdio">stdio</option>
              <option value="http">http</option>
            </select>
          </label>
          {transport === 'stdio' ? (
            <>
              <label className="mcp-server-add-modal__label">
                <span>Command</span>
                <input
                  data-testid="mcp-add-command"
                  value={command}
                  onChange={(e) => setCommand(e.target.value)}
                />
              </label>
              <label className="mcp-server-add-modal__label">
                <span>Args (comma-separated)</span>
                <input
                  data-testid="mcp-add-args"
                  value={argsCsv}
                  onChange={(e) => setArgsCsv(e.target.value)}
                />
              </label>
              <label className="mcp-server-add-modal__label">
                <span>Env (KEY=value per line)</span>
                <textarea
                  data-testid="mcp-add-env"
                  value={envText}
                  rows={3}
                  onChange={(e) => setEnvText(e.target.value)}
                />
              </label>
            </>
          ) : (
            <label className="mcp-server-add-modal__label">
              <span>URL</span>
              <input
                data-testid="mcp-add-url"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
              />
            </label>
          )}
          <label className="mcp-server-add-modal__label">
            <span>Auth secret (optional)</span>
            <textarea
              data-testid="mcp-add-auth"
              value={auth}
              rows={2}
              onChange={(e) => setAuth(e.target.value)}
            />
          </label>
          {tools !== null && (
            <ul className="mcp-tool-list" data-testid="mcp-add-tool-list">
              {tools.map((t) => (
                <li key={t.name} className="mcp-tool-list__item">
                  {t.name}
                </li>
              ))}
            </ul>
          )}
          <div className="mcp-server-add-modal__actions">
            <button type="button" data-testid="mcp-add-test" onClick={() => void handleTest()}>
              Test
            </button>
            <button type="submit" data-testid="mcp-add-submit" disabled={!nameValid}>
              Add
            </button>
            <button type="button" data-testid="mcp-add-cancel" onClick={onClose}>
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  );

  return createPortal(modal, document.body);
}
