import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReactNode } from 'react';

// M08.E — the BuilderShell Canvas | JSON tab toggle + the mounted
// Inspector. The shell renders the interactive BuilderCanvas (real
// @xyflow/react needs a measured pane happy-dom does not provide — the
// BuilderCanvas.test.tsx precedent), so <ReactFlow> is replaced with a
// deterministic test double; the test asserts the shell's WIRING — the
// tab toggle swaps the canvas / JSON editor, the Inspector mounts.
vi.mock('@xyflow/react', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@xyflow/react')>();
  return {
    ...actual,
    useReactFlow: () => ({ screenToFlowPosition: (p: { x: number; y: number }) => p }),
    ReactFlow: ({ children }: { children?: ReactNode }) => (
      <div data-testid="rf-mock">{children}</div>
    ),
  };
});

// The Palette mounts inside BuilderShell and calls list_installed_artifacts
// on mount — mock the invoke boundary so the shell render is deterministic.
const invokeMock = vi.fn(async (..._args: unknown[]) => [] as unknown);
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

import { fireEvent, render, screen } from '@testing-library/react';
import { BuilderShell } from '../../../../src/components/builder/BuilderShell';

describe('BuilderShell — Canvas | JSON tab toggle + Inspector (M08.E)', () => {
  beforeEach(() => {
    invokeMock.mockResolvedValue([]);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('renders_the_canvas_json_tab_toggle', () => {
    render(<BuilderShell />);
    expect(screen.getByRole('tab', { name: 'Canvas' })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: 'JSON' })).toBeInTheDocument();
  });

  it('the_canvas_tab_is_active_by_default', () => {
    render(<BuilderShell />);
    expect(screen.getByRole('tab', { name: 'Canvas' })).toHaveAttribute('aria-selected', 'true');
    expect(screen.getByTestId('builder-canvas')).toBeInTheDocument();
  });

  it('switching_to_the_json_tab_shows_the_json_view', () => {
    render(<BuilderShell />);
    fireEvent.click(screen.getByRole('tab', { name: 'JSON' }));
    expect(screen.getByTestId('builder-json-view')).toBeInTheDocument();
    // The canvas is swapped out — the JSON tab is the other editor over
    // the same document, not an overlay (ADR-0020).
    expect(screen.queryByTestId('builder-canvas')).not.toBeInTheDocument();
  });

  it('switching_back_to_the_canvas_tab_shows_the_canvas', () => {
    render(<BuilderShell />);
    fireEvent.click(screen.getByRole('tab', { name: 'JSON' }));
    fireEvent.click(screen.getByRole('tab', { name: 'Canvas' }));
    expect(screen.getByTestId('builder-canvas')).toBeInTheDocument();
    expect(screen.queryByTestId('builder-json-view')).not.toBeInTheDocument();
  });

  it('mounts_the_inspector_in_the_inspector_region', () => {
    render(<BuilderShell />);
    expect(screen.getByTestId('builder-inspector')).toBeInTheDocument();
  });
});
