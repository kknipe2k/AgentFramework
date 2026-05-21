import { beforeEach, describe, expect, it, vi } from 'vitest';

// list_installed_artifacts crosses the invoke boundary — mock it.
const invokeMock = vi.fn(async (..._args: unknown[]) => undefined as unknown);
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Palette } from '../../../../src/components/builder/Palette';
import type { InstalledArtifact } from '../../../../src/lib/ipc';

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
