import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// M08.E — the Builder Inspector (spec Phase 9 right sidebar): a live
// framework.json preview, a disk diff, the whole-framework capability
// summary (read from the validate_framework report's capability_summary
// FIELD — not a separate command), and Validate / Test / Save / Load.
//
// validate_framework / save_framework / load_framework are Stage B Rust
// commands; the @tauri-apps/plugin-dialog `open` picker is Stage C.
// Mock all four at the module boundary so the Inspector wiring tests
// never reach the real Tauri bridge. The ipc mock is partial
// (importOriginal spread) so unwrapCmdError + diffFramework's siblings
// stay real.
const { validateFrameworkMock, saveFrameworkMock, loadFrameworkMock, openMock } = vi.hoisted(
  () => ({
    validateFrameworkMock: vi.fn(),
    saveFrameworkMock: vi.fn(),
    loadFrameworkMock: vi.fn(),
    openMock: vi.fn(),
  }),
);
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: (...args: unknown[]) => openMock(...args),
}));
vi.mock('../../../../src/lib/ipc', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../../../src/lib/ipc')>();
  return {
    ...actual,
    validateFramework: validateFrameworkMock,
    saveFramework: saveFrameworkMock,
    loadFramework: loadFrameworkMock,
  };
});

import { act, render, screen, waitFor } from '@testing-library/react';
import { Inspector } from '../../../../src/components/builder/Inspector';
import { emptyFramework, useBuilderStore } from '../../../../src/lib/builderStore';
import type { FrameworkValidationReport } from '../../../../src/lib/ipc';
import type { Framework } from '../../../../src/types/framework';

function namedFramework(name: string): Framework {
  return { ...emptyFramework(), name };
}

/** A report carrying a populated whole-framework capability summary. */
function reportWithSummary(): FrameworkValidationReport {
  return {
    schema_errors: [],
    capability_errors: [],
    ok: true,
    capability_summary: {
      files_read: ['src/**'],
      files_written: [],
      network_hosts: ['api.example.com'],
      any_shell: true,
      spawn_edges: [],
    },
  };
}

describe('Inspector', () => {
  beforeEach(() => {
    validateFrameworkMock.mockReset().mockResolvedValue({
      schema_errors: [],
      capability_errors: [],
      ok: true,
      capability_summary: null,
    });
    saveFrameworkMock.mockReset().mockResolvedValue(undefined);
    loadFrameworkMock.mockReset();
    openMock.mockReset();
    useBuilderStore.setState(useBuilderStore.getInitialState(), true);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('renders_the_live_framework_json_preview', () => {
    render(<Inspector />);
    // The preview is the serialized source-of-truth document.
    expect(screen.getByTestId('inspector-preview')).toHaveTextContent('"untitled"');
  });

  it('the_preview_updates_when_the_framework_changes', () => {
    render(<Inspector />);
    act(() => {
      useBuilderStore.getState().replaceFramework(namedFramework('preview-renamed'));
    });
    expect(screen.getByTestId('inspector-preview')).toHaveTextContent('preview-renamed');
  });

  it('renders_the_disk_diff_only_when_diskFramework_is_set', () => {
    render(<Inspector />);
    // No disk origin yet — the diff section is absent.
    expect(screen.queryByTestId('inspector-diff')).not.toBeInTheDocument();
    act(() => {
      useBuilderStore.getState().setDiskFramework(emptyFramework());
    });
    expect(screen.getByTestId('inspector-diff')).toBeInTheDocument();
  });

  it('renders_the_capability_summary_from_the_validate_framework_report', () => {
    // The summary is the capability_summary FIELD on the report (Stage
    // B B.3.4) — no separate framework_capability_summary command.
    act(() => {
      useBuilderStore.getState().setValidation(reportWithSummary());
    });
    render(<Inspector />);
    const caps = screen.getByTestId('inspector-capabilities');
    expect(caps).toHaveTextContent('src/**');
    expect(caps).toHaveTextContent('api.example.com');
  });

  it('clicking_validate_calls_validateFramework_and_surfaces_the_full_report', async () => {
    validateFrameworkMock.mockResolvedValue({
      schema_errors: [{ node_path: 'planner', message: 'session_root_agent is empty' }],
      capability_errors: [],
      ok: false,
      capability_summary: null,
    });
    render(<Inspector />);
    screen.getByRole('button', { name: 'Validate' }).click();
    // The Validate button surfaces the per-node messages D2's badges
    // only counted.
    expect(await screen.findByText(/session_root_agent is empty/)).toBeInTheDocument();
    expect(validateFrameworkMock).toHaveBeenCalledWith(useBuilderStore.getState().framework);
  });

  it('clicking_test_calls_builderStore_openTester', () => {
    // The Test button is INERT-but-wired — E ships openTester; Stage F2
    // renders the modal on it. The test asserts the call, not a modal.
    const openTester = vi.fn();
    useBuilderStore.setState({ openTester });
    render(<Inspector />);
    screen.getByRole('button', { name: 'Test' }).click();
    expect(openTester).toHaveBeenCalledTimes(1);
  });

  it('the_validate_button_uses_the_same_validate_framework_the_continuous_pass_uses', async () => {
    // Contract test (gotcha #66 / spec §9 — one validator, two
    // triggers): the backend report claims a verdict no TS-side
    // validator would invent; the Inspector must surface it verbatim,
    // proving it renders the Rust report rather than recomputing.
    validateFrameworkMock.mockResolvedValue({
      schema_errors: [{ node_path: '(root)', message: 'BACKEND-ONLY VERDICT' }],
      capability_errors: [],
      ok: false,
      capability_summary: null,
    });
    render(<Inspector />);
    screen.getByRole('button', { name: 'Validate' }).click();
    expect(await screen.findByText(/BACKEND-ONLY VERDICT/)).toBeInTheDocument();
  });

  it('clicking_save_picks_a_directory_and_calls_saveFramework', async () => {
    openMock.mockResolvedValue('C:/picked-dir');
    render(<Inspector />);
    screen.getByRole('button', { name: 'Save' }).click();
    await waitFor(() => expect(saveFrameworkMock).toHaveBeenCalled());
    expect(openMock).toHaveBeenCalledWith({ directory: true });
    expect(saveFrameworkMock).toHaveBeenCalledWith(
      'C:/picked-dir',
      useBuilderStore.getState().framework,
    );
  });

  it('cancelling_the_save_picker_does_not_call_saveFramework', async () => {
    // A cancelled picker resolves null — a normal user action, not an
    // error; the Inspector short-circuits.
    openMock.mockResolvedValue(null);
    render(<Inspector />);
    screen.getByRole('button', { name: 'Save' }).click();
    await waitFor(() => expect(openMock).toHaveBeenCalled());
    expect(saveFrameworkMock).not.toHaveBeenCalled();
  });

  it('clicking_load_picks_a_directory_loads_and_replaces_the_framework', async () => {
    openMock.mockResolvedValue('C:/load-dir');
    loadFrameworkMock.mockResolvedValue({
      framework: namedFramework('loaded-from-disk'),
      companions: [],
    });
    const replaceFramework = vi.fn();
    useBuilderStore.setState({ replaceFramework });
    render(<Inspector />);
    screen.getByRole('button', { name: 'Load' }).click();
    await waitFor(() => expect(loadFrameworkMock).toHaveBeenCalledWith('C:/load-dir'));
    // load → replaceFramework swaps the source of truth; the canvas
    // re-derives (ADR-0020).
    expect(replaceFramework).toHaveBeenCalledWith(
      expect.objectContaining({ name: 'loaded-from-disk' }),
    );
  });

  it('cancelling_the_load_picker_does_not_call_loadFramework', async () => {
    openMock.mockResolvedValue(null);
    render(<Inspector />);
    screen.getByRole('button', { name: 'Load' }).click();
    await waitFor(() => expect(openMock).toHaveBeenCalled());
    expect(loadFrameworkMock).not.toHaveBeenCalled();
  });
});

// gotcha #67 — a className with no styles.css rule renders unstyled and
// the user sees nothing. Every Builder class M08.E introduces must have
// a corresponding rule, and use --node-* theme tokens (M07-IRL #3).
describe('M08.E Inspector + Canvas|JSON styles (gotcha #67)', () => {
  const css = readFileSync(resolve(__dirname, '../../../../src/styles.css'), 'utf8');
  const E_CLASSES = [
    'builder-tabs',
    'builder-tab',
    'builder-tab--active',
    'builder-inspector',
    'inspector-section',
    'inspector-section--preview',
    'inspector-section--capabilities',
    'inspector__diff',
    'inspector__diff-add',
    'inspector__diff-remove',
    'inspector__actions',
    'inspector__errors',
    'json-view',
    'json-view__error',
  ] as const;

  it.each(E_CLASSES)('styles.css defines a rule for .%s', (cls) => {
    expect(css).toMatch(new RegExp(`\\.${cls}[\\s,{]`));
  });

  it('M08.E styles use theme variables, not literal colors (M07-IRL #3)', () => {
    expect(css).toMatch(/\.builder-inspector[\s\S]*?var\(--node-/);
  });
});
