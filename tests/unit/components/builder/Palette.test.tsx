import { beforeEach, describe, expect, it, vi } from 'vitest';

// list_installed_artifacts crosses the invoke boundary — mock it.
const invokeMock = vi.fn(async (..._args: unknown[]) => undefined as unknown);
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Palette } from '../../../../src/components/builder/Palette';
import { emptyFramework, useBuilderStore } from '../../../../src/lib/builderStore';
import type { InstalledArtifact } from '../../../../src/lib/ipc';
import type { Agent, Framework } from '../../../../src/types/framework';

// M08.C — the five-tab Palette (Tools / Skills / Agents / HITL / Hooks).
// Tools/Skills/Agents list built-ins + whatever list_installed_artifacts
// returns; HITL lists the §6a trigger types; Hooks lists the §4a firing
// points. Every item is a native-HTML drag source carrying the
// application/x-builder-node payload D1's drop handler reads.

const installedTool: InstalledArtifact = {
  key: 'fs-tool@1.0.0',
  kind: 'tool',
  source: { type: 'url', url: 'https://example.com/fs.json' },
  installed_at: '2026-05-21T00:00:00Z',
};

describe('Palette', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue([]);
  });

  it('renders_the_active_tab_items_only', async () => {
    render(<Palette />);
    // The default tab is Tools — the built-in tools render; the Hooks
    // tab's firing points do not.
    expect(await screen.findByTestId('palette-item-Read')).toBeInTheDocument();
    expect(screen.queryByTestId('palette-item-pre_task')).not.toBeInTheDocument();
  });

  it('switching_tabs_changes_the_listed_items', async () => {
    render(<Palette />);
    await screen.findByTestId('palette-item-Read');

    await userEvent.click(screen.getByTestId('palette-tab-hooks'));
    expect(screen.getByTestId('palette-item-pre_task')).toBeInTheDocument();
    expect(screen.queryByTestId('palette-item-Read')).not.toBeInTheDocument();

    await userEvent.click(screen.getByTestId('palette-tab-hitl'));
    expect(screen.getByTestId('palette-item-on_gap')).toBeInTheDocument();
    expect(screen.queryByTestId('palette-item-pre_task')).not.toBeInTheDocument();
  });

  it('the_filter_input_narrows_the_list_case_insensitively', async () => {
    render(<Palette />);
    await screen.findByTestId('palette-item-Read');
    await userEvent.type(screen.getByTestId('palette-filter'), 'rEaD');
    expect(screen.getByTestId('palette-item-Read')).toBeInTheDocument();
    expect(screen.queryByTestId('palette-item-Write')).not.toBeInTheDocument();
  });

  it('installed_artifacts_from_list_installed_appear_in_the_tools_tab', async () => {
    invokeMock.mockResolvedValue([installedTool]);
    render(<Palette />);
    // list_installed_artifacts takes zero JS args — the Tauri shell
    // resolves the skills.lock path internally (wire pinned to Stage B).
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith('list_installed_artifacts', undefined),
    );
    expect(await screen.findByTestId('palette-item-fs-tool@1.0.0')).toBeInTheDocument();
  });

  it('dragStart_sets_the_application_x_builder_node_payload', async () => {
    render(<Palette />);
    const item = await screen.findByTestId('palette-item-Read');
    expect(item).toHaveAttribute('draggable', 'true');

    const setData = vi.fn();
    fireEvent.dragStart(item, { dataTransfer: { setData, effectAllowed: '' } });
    // The C->D1 contract: D1's onDrop reads this MIME type + JSON payload.
    expect(setData).toHaveBeenCalledWith(
      'application/x-builder-node',
      JSON.stringify({ kind: 'tool', ref: 'Read' }),
    );
  });
});

// M08.6.E — the Palette draws a THIRD source from the loaded framework
// (builderStore.framework) so a framework's defined agents / tools /
// skills surface as reusable drag sources in the matching tab. The
// existing two sources (built-ins + listInstalledArtifacts) remain;
// the union is de-duplicated by (kind, ref) so an artifact in both an
// installed lock and the loaded framework appears once. Framework-
// sourced items carry data-source="framework" so the user can tell
// where an item comes from; the drag payload is identical to a
// built-in / installed item — uniform drop contract per phase doc E.3.
describe('Palette — loaded-framework source (M08.6.E)', () => {
  /** Build a minimal inline Agent — the resolved shape Stage B's
   *  loader returns and Stage E surfaces. */
  function inlineAgent(id: string): Agent {
    return {
      id,
      role: 'role-for-test',
      model: { provider: 'anthropic', id: 'claude-sonnet-4-6' },
      allowed_tools: [],
      allowed_skills: [],
      spawns: [],
    } as unknown as Agent;
  }

  /** A framework with the test's overrides spliced into the cold-start
   *  empty document. Pre-valid (matches `emptyFramework`'s stance). */
  function frameworkWith(overrides: Partial<Framework>): Framework {
    return { ...emptyFramework(), ...overrides };
  }

  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue([]);
    // Reset builderStore — the Palette reads `framework` from the
    // store; cross-test bleed would mask a missing implementation.
    useBuilderStore.setState(useBuilderStore.getInitialState(), true);
  });

  it('loaded_framework_agents_appear_in_the_agents_tab', async () => {
    useBuilderStore.setState({
      framework: frameworkWith({
        agents: [inlineAgent('orchestrator'), inlineAgent('planner')] as Framework['agents'],
      }),
    });
    render(<Palette />);
    await userEvent.click(screen.getByTestId('palette-tab-agents'));
    // Pre-E the Agents tab carries only listInstalledArtifacts results
    // (empty here); the framework-source addition makes ARIA-like
    // agents surface as drag sources.
    expect(screen.getByTestId('palette-item-orchestrator')).toBeInTheDocument();
    expect(screen.getByTestId('palette-item-planner')).toBeInTheDocument();
  });

  it('loaded_framework_tools_appear_in_the_tools_tab', async () => {
    useBuilderStore.setState({
      framework: frameworkWith({
        tools: [{ name: 'git_checkpoint', source: 'generated', path: 'tools/git_checkpoint.md' }],
      }),
    });
    render(<Palette />);
    // The default tab is Tools — a framework-sourced (non-built-in)
    // tool appears alongside Read / Write / Bash.
    expect(await screen.findByTestId('palette-item-git_checkpoint')).toBeInTheDocument();
  });

  it('loaded_framework_skills_appear_in_the_skills_tab', async () => {
    useBuilderStore.setState({
      framework: frameworkWith({
        skills: [{ name: 'planning', path: 'skills/planning.md', source: 'local' }],
      }),
    });
    render(<Palette />);
    await userEvent.click(screen.getByTestId('palette-tab-skills'));
    expect(screen.getByTestId('palette-item-planning')).toBeInTheDocument();
  });

  it('palette_de_duplicates_a_kind_ref_present_in_both_installed_and_framework', async () => {
    // The installed lock declares a tool whose key matches a
    // framework tool's name; the dedupe key (kind, ref) collapses
    // the two sources into one Palette item — no duplicates on the
    // canvas, no duplicate drop on user click.
    const sharedInstalled: InstalledArtifact = {
      key: 'shared-tool',
      kind: 'tool',
      source: { type: 'url', url: 'https://example.com/shared.json' },
      installed_at: '2026-05-26T00:00:00Z',
    };
    invokeMock.mockResolvedValue([sharedInstalled]);
    useBuilderStore.setState({
      framework: frameworkWith({
        tools: [{ name: 'shared-tool', source: 'external' }],
      }),
    });
    render(<Palette />);
    // Wait for listInstalledArtifacts to resolve so both sources are
    // present in the unfiltered list before the dedupe runs.
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith('list_installed_artifacts', undefined),
    );
    const items = await screen.findAllByTestId('palette-item-shared-tool');
    expect(items.length).toBe(1);
    // Dedupe precedence: built-ins → installed → framework. An item
    // in both `installed` and `framework` keeps the installed entry
    // (the lock carries version + install timestamp); the surviving
    // item carries data-source='installed'. The data-source assertion
    // is the discriminator that fails right-reason on `main` pre-E —
    // pre-E there is no data-source attribute at all.
    expect(items[0]).toHaveAttribute('data-source', 'installed');
  });

  it('framework_sourced_items_carry_data_source_framework_and_drag_identically', async () => {
    useBuilderStore.setState({
      framework: frameworkWith({
        agents: [inlineAgent('orchestrator')] as Framework['agents'],
      }),
    });
    render(<Palette />);
    await userEvent.click(screen.getByTestId('palette-tab-agents'));
    const item = await screen.findByTestId('palette-item-orchestrator');
    // Visible-and-testable distinguisher per phase doc E.3 — a
    // data-source attribute marks the artifact as framework-sourced.
    expect(item).toHaveAttribute('data-source', 'framework');
    // Drag payload contract is uniform — identical to built-ins /
    // installed items per Key constraints (uniform drop contract).
    const setData = vi.fn();
    fireEvent.dragStart(item, { dataTransfer: { setData, effectAllowed: '' } });
    expect(setData).toHaveBeenCalledWith(
      'application/x-builder-node',
      JSON.stringify({ kind: 'agent', ref: 'orchestrator' }),
    );
  });

  it('built_in_tools_carry_data_source_builtin', async () => {
    render(<Palette />);
    const readItem = await screen.findByTestId('palette-item-Read');
    expect(readItem).toHaveAttribute('data-source', 'builtin');
  });

  it('installed_artifacts_carry_data_source_installed', async () => {
    const installedTool2: InstalledArtifact = {
      key: 'installed-only-tool',
      kind: 'tool',
      source: { type: 'url', url: 'https://example.com/x.json' },
      installed_at: '2026-05-26T00:00:00Z',
    };
    invokeMock.mockResolvedValue([installedTool2]);
    render(<Palette />);
    const item = await screen.findByTestId('palette-item-installed-only-tool');
    expect(item).toHaveAttribute('data-source', 'installed');
  });
});

// M09.A — the "+ New agent" affordance. Pre-M09 the Agents tab listed
// only installed + loaded-framework agents (Palette.tsx:173-184), so a
// fresh project (emptyFramework, agents: []) rendered the empty state and
// nothing could be authored on the canvas. M09.A prepends a blank-create
// item carrying a fresh nextAgentRef id through the same drag contract;
// the existing addNode path mints the agent on drop (no drop-handler /
// store-core change). Repeated creates yield distinct ids because the
// Palette reads `framework` and re-derives nextAgentRef each render.
describe('Palette — New agent affordance (M09.A)', () => {
  /** A minimal inline Agent, the shape addNode mints + a loaded framework
   *  surfaces (matches builderStore.builderAgent's pre-capabilities form). */
  function inlineAgent(id: string): Agent {
    return {
      id,
      role: 'role-for-test',
      model: { provider: 'anthropic', id: 'claude-sonnet-4-6' },
      allowed_tools: [],
      allowed_skills: [],
      spawns: [],
    } as unknown as Agent;
  }

  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue([]);
    // The Palette reads `framework` from the store; reset it so a stale
    // agents[] from a prior test cannot mask a missing implementation.
    useBuilderStore.setState(useBuilderStore.getInitialState(), true);
  });

  it('a_fresh_agents_tab_shows_a_new_agent_item', async () => {
    render(<Palette />);
    await userEvent.click(screen.getByTestId('palette-tab-agents'));
    // The blank-create affordance — its testid suffix is the fresh ref
    // (agent-1 on an empty project), its label the "+ New agent" cue.
    expect(screen.getByText('+ New agent')).toBeInTheDocument();
    expect(screen.getByTestId('palette-item-agent-1')).toBeInTheDocument();
  });

  it('the_new_agent_item_drags_a_fresh_agent_payload', async () => {
    render(<Palette />);
    await userEvent.click(screen.getByTestId('palette-tab-agents'));
    const item = screen.getByTestId('palette-item-agent-1');
    expect(item).toHaveAttribute('draggable', 'true');
    // The uniform application/x-builder-node contract BuilderCanvas.onDrop
    // reads — addNode('agent', 'agent-1', position) mints builderAgent.
    const setData = vi.fn();
    fireEvent.dragStart(item, { dataTransfer: { setData, effectAllowed: '' } });
    expect(setData).toHaveBeenCalledWith(
      'application/x-builder-node',
      JSON.stringify({ kind: 'agent', ref: 'agent-1' }),
    );
  });

  it('the_new_agent_ref_advances_past_existing_agents', async () => {
    // With agent-1 already in the document, the New-agent item carries
    // agent-2 — so a second create never collides with the first.
    useBuilderStore.setState({
      framework: { ...emptyFramework(), agents: [inlineAgent('agent-1')] as Framework['agents'] },
    });
    render(<Palette />);
    await userEvent.click(screen.getByTestId('palette-tab-agents'));
    const item = screen.getByTestId('palette-item-agent-2');
    expect(item).toHaveTextContent('+ New agent');
    const setData = vi.fn();
    fireEvent.dragStart(item, { dataTransfer: { setData, effectAllowed: '' } });
    expect(setData).toHaveBeenCalledWith(
      'application/x-builder-node',
      JSON.stringify({ kind: 'agent', ref: 'agent-2' }),
    );
  });
});

// M09.C — an installed MCP server's tools become draggable Palette items.
// Pre-M09.C the Tools tab listed only built-ins + installed-artifacts +
// loaded-framework tools (Palette.tsx:143-160); a connected MCP server's
// tools — the data-bearing tools an agent actually needs — were never
// reachable on the canvas. M09.C adds the `mcp_list_server_tools(name)`
// command (Vec<McpTool>) and the Palette fetches `mcp_list_servers()` on
// mount, then each server's tools, and surfaces them as `source:'mcp'`
// items labelled `<server> · <tool>` with the canonical `<server>__<tool>`
// drag ref (the §5a namespace form the dispatcher resolves). The drag
// contract stays the uniform application/x-builder-node payload.
describe('Palette — MCP server tools (M09.C)', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    // Per-command dispatch: list_installed_artifacts → []; mcp_list_servers
    // → one connected stdio server; mcp_list_server_tools → that server's
    // single tool. The Palette reads `framework` from the store; reset it so
    // a stale tools[] cannot mask a missing implementation.
    invokeMock.mockImplementation(async (...args: unknown[]) => {
      const command = args[0] as string;
      if (command === 'mcp_list_servers') {
        return [{ name: 'fs', transport: 'stdio', has_auth: false, status: 'connected' }];
      }
      if (command === 'mcp_list_server_tools') {
        expect(args[1]).toEqual({ name: 'fs' });
        return [{ name: 'read_file', description: 'Read a file', input_schema: {} }];
      }
      return [];
    });
    useBuilderStore.setState(useBuilderStore.getInitialState(), true);
  });

  it('an_installed_servers_tools_appear_in_the_tools_tab_with_source_mcp', async () => {
    render(<Palette />);
    // The default tab is Tools — the MCP server's tool surfaces alongside
    // the built-ins, keyed by the canonical `<server>__<tool>` ref.
    const item = await screen.findByTestId('palette-item-fs__read_file');
    expect(item).toBeInTheDocument();
    // Visible-and-testable origin marker — distinct from builtin/installed/
    // framework so the user sees the tool came from a connected MCP server.
    expect(item).toHaveAttribute('data-source', 'mcp');
    // Labelled `<server> · <tool>` so the same short tool name across two
    // servers is distinguishable in the list.
    expect(item).toHaveTextContent('fs · read_file');
  });

  it('the_mcp_tool_drags_the_canonical_server__tool_payload', async () => {
    render(<Palette />);
    const item = await screen.findByTestId('palette-item-fs__read_file');
    expect(item).toHaveAttribute('draggable', 'true');
    // The uniform application/x-builder-node contract — the ref is the
    // canonical `<server>__<tool>` the §5a resolver accepts unambiguously,
    // so the dropped tool node + Agent→Tool edge record a dispatchable name.
    const setData = vi.fn();
    fireEvent.dragStart(item, { dataTransfer: { setData, effectAllowed: '' } });
    expect(setData).toHaveBeenCalledWith(
      'application/x-builder-node',
      JSON.stringify({ kind: 'tool', ref: 'fs__read_file' }),
    );
  });

  it('an_mcp_list_servers_failure_degrades_to_built_ins_only', async () => {
    // A backend with no McpClient (or a registry error) must not blank the
    // Tools tab — the built-ins still render; the MCP source is simply
    // absent. Mirrors listInstalledArtifacts' catch-and-log resilience.
    invokeMock.mockImplementation(async (...args: unknown[]) => {
      if ((args[0] as string) === 'mcp_list_servers') {
        throw new Error('no McpClient');
      }
      return [];
    });
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => undefined);
    render(<Palette />);
    expect(await screen.findByTestId('palette-item-Read')).toBeInTheDocument();
    expect(screen.queryByTestId('palette-item-fs__read_file')).not.toBeInTheDocument();
    consoleError.mockRestore();
  });
});
