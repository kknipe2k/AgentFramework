import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// Mock @tauri-apps/api modules at the test boundary so the IPC wrapper
// can be exercised in isolation. The wrappers are thin; the value of the
// tests is verifying the call shape (command name + arg shape) and the
// subscriber lifecycle (subscribe → emit → unsubscribe).

const invokeMock = vi.fn();
const listenMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: (...args: unknown[]) => listenMock(...args),
}));

import {
  cancelPendingImport,
  completeImportArtifact,
  getCurrentTier,
  importArtifact,
  invokeQuerySessionDb,
  invokeReplaySession,
  invokeRespondHitl,
  invokeRunSmokeSession,
  invokeSetApiKey,
  mcpAddServer,
  mcpListServers,
  mcpRemoveServer,
  mcpTestConnection,
  subscribeAgentEvents,
  unwrapCmdError,
  type ImportOutcome,
} from '../../src/lib/ipc';
import type { AgentEvent } from '../../src/types/agent_event';
import type { McpServerConfig } from '../../src/types/mcp';

describe('ipc', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('invokeRunSmokeSession_calls_invoke_with_correct_command_name', async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    await invokeRunSmokeSession();
    expect(invokeMock).toHaveBeenCalledTimes(1);
    expect(invokeMock).toHaveBeenCalledWith('run_smoke_session');
  });

  it('invokeSetApiKey_passes_key_arg', async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    await invokeSetApiKey('sk-ant-test');
    expect(invokeMock).toHaveBeenCalledWith('set_api_key', { key: 'sk-ant-test' });
  });

  it('subscribeAgentEvents_returns_unlisten_fn', async () => {
    const unlisten = vi.fn();
    listenMock.mockResolvedValueOnce(unlisten);
    const handler = vi.fn();
    const result = await subscribeAgentEvents(handler);
    expect(result).toBe(unlisten);
    expect(listenMock).toHaveBeenCalledWith('agent_event', expect.any(Function));
  });

  it('subscribeAgentEvents_handler_called_on_emit', async () => {
    const unlisten = vi.fn();
    let registeredHandler: ((e: { payload: AgentEvent }) => void) | undefined;
    listenMock.mockImplementationOnce(
      async (_channel: string, cb: (e: { payload: AgentEvent }) => void): Promise<() => void> => {
        registeredHandler = cb;
        return unlisten;
      },
    );

    const handler = vi.fn();
    await subscribeAgentEvents(handler);
    expect(registeredHandler).toBeDefined();

    const event: AgentEvent = {
      type: 'stream_text',
      agent_id: 'a1',
      text: 'hi',
    };
    registeredHandler!({ payload: event });

    expect(handler).toHaveBeenCalledWith(event);
  });

  it('unsubscribe_stops_handler_calls', async () => {
    const unlisten = vi.fn();
    listenMock.mockResolvedValueOnce(unlisten);
    const handler = vi.fn();
    const stop = await subscribeAgentEvents(handler);
    expect(handler).not.toHaveBeenCalled();
    stop();
    expect(unlisten).toHaveBeenCalledTimes(1);
  });

  it('invokeQuerySessionDb_passes_sql_arg_and_returns_rows', async () => {
    invokeMock.mockResolvedValueOnce([{ id: 1 }, { id: 2 }]);
    const rows = await invokeQuerySessionDb('SELECT id FROM signals');
    expect(invokeMock).toHaveBeenCalledWith('query_session_db', {
      sql: 'SELECT id FROM signals',
    });
    expect(rows).toEqual([{ id: 1 }, { id: 2 }]);
  });

  it('invokeReplaySession_passes_session_id_arg', async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    await invokeReplaySession('s-xyz');
    expect(invokeMock).toHaveBeenCalledWith('replay_session', { sessionId: 's-xyz' });
  });

  // ── M08.8.C.fix (#19 display desync) — getCurrentTier wrapper. ──
  // The App mount seeds `currentTier` from this so the Settings display
  // matches the enforced tier across a restart (the invokeHasApiKey
  // precedent). ZERO JS args; returns the serde-lowercase TierRef
  // direct from `get_current_tier` (commands.rs:633 → Tier), no mapping.

  it('getCurrentTier_invokes_get_current_tier_with_no_args_and_returns_promoted', async () => {
    invokeMock.mockResolvedValueOnce('promoted');
    const tier = await getCurrentTier();
    expect(invokeMock).toHaveBeenCalledWith('get_current_tier');
    const [, arg] = invokeMock.mock.calls[0] as [string, unknown];
    expect(arg).toBeUndefined();
    expect(tier).toBe('promoted');
  });

  it('getCurrentTier_returns_novice_direct_tierref', async () => {
    invokeMock.mockResolvedValueOnce('novice');
    expect(await getCurrentTier()).toBe('novice');
  });

  // ── unwrapCmdError — M04 Stage A2 consumes generated CmdError shape. ──

  it('unwrapCmdError_setup_required_returns_user_facing_message', () => {
    // The unit variant has no `message` field; the helper substitutes a
    // user-actionable instruction so the renderer doesn't surface a bare
    // discriminator string. Wire shape from src/types/error.ts.
    const e = { type: 'setup_required' };
    const out = unwrapCmdError(e);
    expect(out).toContain('API key not set');
    expect(out).toContain('Save key');
  });

  it('unwrapCmdError_provider_renders_type_and_message', () => {
    // Generated CmdError tuple variants serialize as
    // {"type":"provider","message":"..."} per #[serde(content="message")].
    const e = { type: 'provider', message: 'auth failed' };
    expect(unwrapCmdError(e)).toBe('provider: auth failed');
  });

  it('unwrapCmdError_drone_renders_type_and_message', () => {
    const e = { type: 'drone', message: 'subprocess died' };
    expect(unwrapCmdError(e)).toBe('drone: subprocess died');
  });

  it('unwrapCmdError_key_store_renders_type_and_message', () => {
    const e = { type: 'key_store', message: 'keychain locked' };
    expect(unwrapCmdError(e)).toBe('key_store: keychain locked');
  });

  it('unwrapCmdError_internal_renders_type_and_message', () => {
    const e = { type: 'internal', message: 'channel closed' };
    expect(unwrapCmdError(e)).toBe('internal: channel closed');
  });

  it('unwrapCmdError_error_instance_returns_error_message', () => {
    // A raw JS Error (e.g., a network failure inside the @tauri-apps
    // bridge that didn't reach the CmdError path) should still surface
    // its `message` rather than collapsing to "[object Object]".
    const e = new Error('network unavailable');
    expect(unwrapCmdError(e)).toBe('network unavailable');
  });

  it('unwrapCmdError_unknown_object_with_message_returns_message', () => {
    // Compatibility path: arbitrary error-shape objects from elsewhere in
    // the renderer (not generated CmdError but with a `message` field).
    const e = { message: 'something else' };
    expect(unwrapCmdError(e)).toBe('something else');
  });

  it('unwrapCmdError_falls_back_to_string_for_unknown_types', () => {
    // Last-resort fallback preserves M02 Stage E behavior — anything
    // not matching the typed paths goes through String() so the user
    // sees *something* rather than nothing.
    expect(unwrapCmdError(42)).toBe('42');
    expect(unwrapCmdError(null)).toBe('null');
  });

  it('unwrapCmdError_object_without_recognized_type_or_message_falls_through', () => {
    // An object with neither a recognized `type` discriminator nor a
    // `message` field falls through to the last-resort String() branch.
    const e = { foo: 'bar' };
    expect(unwrapCmdError(e)).toBe('[object Object]');
  });

  it('invokeRespondHitl_passes_promptId_and_choice', async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    await invokeRespondHitl('u-1', 'skip');
    expect(invokeMock).toHaveBeenCalledWith('respond_hitl', {
      promptId: 'u-1',
      choice: 'skip',
    });
  });

  // ── M06.E — MCP server lifecycle wrappers (Stage C commands) ──────
  const stdioConfig: McpServerConfig = {
    name: 'filesystem',
    transport: { type: 'stdio', command: 'npx', args: ['-y', 'server'] },
  };

  it('mcpAddServer_passes_config_and_auth', async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    await mcpAddServer(stdioConfig, 'secret-token');
    expect(invokeMock).toHaveBeenCalledWith('mcp_add_server', {
      config: stdioConfig,
      auth: 'secret-token',
    });
  });

  it('mcpAddServer_passes_null_auth_for_unauthenticated_server', async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    await mcpAddServer(stdioConfig, null);
    expect(invokeMock).toHaveBeenCalledWith('mcp_add_server', {
      config: stdioConfig,
      auth: null,
    });
  });

  it('mcpRemoveServer_passes_name_arg', async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    await mcpRemoveServer('filesystem');
    expect(invokeMock).toHaveBeenCalledWith('mcp_remove_server', { name: 'filesystem' });
  });

  it('mcpTestConnection_passes_config_arg_not_name', async () => {
    // Contract guard (gotcha #66): the Stage C command takes `config`,
    // NOT `name` — the E.3.4 phase-doc pseudocode drifted. A regression
    // to `{ name }` would silently fail the Test button.
    invokeMock.mockResolvedValueOnce([]);
    await mcpTestConnection(stdioConfig);
    expect(invokeMock).toHaveBeenCalledWith('mcp_test_connection', { config: stdioConfig });
    const [, arg] = invokeMock.mock.calls[0] as [string, Record<string, unknown>];
    expect(arg).not.toHaveProperty('name');
  });

  it('mcpListServers_calls_invoke_with_correct_command_name', async () => {
    invokeMock.mockResolvedValueOnce([]);
    await mcpListServers();
    expect(invokeMock).toHaveBeenCalledWith('mcp_list_servers');
  });

  // M07.5 / ADR-0017 — the import_artifact / complete_import_artifact /
  // cancel_pending_import wrappers. import_artifact's three flat
  // camelCased args `{ sourceKind, location, artifactKind }` are
  // unchanged from M07.E; only the RETURN type became the discriminated
  // ImportOutcome union (`status: 'pending' | 'installed'`). The
  // complete_/cancel_ params are PINNED to the A.fix-shipped commands.rs
  // signatures — a single `pendingReviewId` camelCased arg (Tauri
  // auto-converts the snake_case Rust `pending_review_id`).
  const pendingOutcome: ImportOutcome = {
    status: 'pending',
    pending_review_id: 'pri-1',
    lock_key: 'fs-test@2.0.0',
    requires_secrets: ['OPENAI_API_KEY'],
    capabilities: ['network: api.example.com', 'shell: true'],
    l3_report: { report_id: 'vr-1', passed: true, reasons: [] },
    share_provenance: { exported_by: 'share-it@0.1.0', rebake_changes: [] },
  };

  const installedOutcome: ImportOutcome = {
    status: 'installed',
    lock_key: 'fs-test@2.0.0',
    requires_secrets: [],
    capabilities: ['network: api.example.com'],
    l3_report: { report_id: 'vr-2', passed: true, reasons: [] },
    share_provenance: null,
  };

  it('importArtifact_passes_pinned_camelCase_args_not_src_kind', async () => {
    invokeMock.mockResolvedValueOnce(pendingOutcome);
    const out = await importArtifact(
      'url',
      'https://raw.githubusercontent.com/o/r/main/fs.json',
      'skill',
    );
    expect(invokeMock).toHaveBeenCalledWith('import_artifact', {
      sourceKind: 'url',
      location: 'https://raw.githubusercontent.com/o/r/main/fs.json',
      artifactKind: 'skill',
    });
    const [, arg] = invokeMock.mock.calls[0] as [string, Record<string, unknown>];
    expect(arg).not.toHaveProperty('src');
    expect(arg).not.toHaveProperty('kind');
    // The discriminated outcome round-trips unchanged.
    expect(out.status).toBe('pending');
    expect(out.capabilities).toEqual(['network: api.example.com', 'shell: true']);
    expect(out.l3_report.passed).toBe(true);
  });

  it('importArtifact_passes_file_source_and_mcp_kind', async () => {
    invokeMock.mockResolvedValueOnce(installedOutcome);
    await importArtifact('file', 'C:/tmp/server.json', 'mcp_server');
    expect(invokeMock).toHaveBeenCalledWith('import_artifact', {
      sourceKind: 'file',
      location: 'C:/tmp/server.json',
      artifactKind: 'mcp_server',
    });
  });

  it('completeImportArtifact_invokes_complete_import_artifact_with_pinned_pendingReviewId', async () => {
    invokeMock.mockResolvedValueOnce(installedOutcome);
    const out = await completeImportArtifact('pri-1');
    expect(invokeMock).toHaveBeenCalledWith('complete_import_artifact', {
      pendingReviewId: 'pri-1',
    });
    expect(out.status).toBe('installed');
  });

  it('cancelPendingImport_invokes_cancel_pending_import_with_pinned_pendingReviewId', async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    await cancelPendingImport('pri-1');
    expect(invokeMock).toHaveBeenCalledWith('cancel_pending_import', {
      pendingReviewId: 'pri-1',
    });
  });
});
