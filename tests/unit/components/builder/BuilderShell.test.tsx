import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it, vi } from 'vitest';

// The Palette mounts inside BuilderShell and calls list_installed_artifacts
// on mount — mock the invoke boundary so the shell render is deterministic.
const invokeMock = vi.fn(async (..._args: unknown[]) => undefined as unknown);
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (command: string, args?: unknown) => invokeMock(command, args),
}));

import { render, screen } from '@testing-library/react';
import { BuilderShell } from '../../../../src/components/builder/BuilderShell';

// M08.C — the three-panel Builder shell. The Palette region ships fully
// working; the Canvas region ships empty (a React-Flow drop target D1
// fills) and the Inspector region ships empty (a stub E fills) — both
// are real DOM landmarks, not dead code.

describe('BuilderShell', () => {
  it('renders_palette_canvas_and_inspector_regions', () => {
    invokeMock.mockResolvedValue([]);
    render(<BuilderShell />);
    expect(screen.getByTestId('builder-shell')).toBeInTheDocument();
    expect(screen.getByTestId('builder-palette-region')).toBeInTheDocument();
    // The Canvas region — empty at C; D1 mounts the interactive canvas.
    expect(screen.getByTestId('builder-canvas-region')).toBeInTheDocument();
    // The Inspector region — empty at C; E mounts the Inspector.
    expect(screen.getByTestId('builder-inspector-region')).toBeInTheDocument();
    // The working Palette renders inside the palette region.
    expect(screen.getByTestId('builder-palette')).toBeInTheDocument();
  });
});

// gotcha #67 — a className with no styles.css rule renders unstyled and
// the user sees nothing. Every Builder class introduced this stage must
// have a corresponding rule. The selector regex distinguishes
// `.builder-shell { ... }` from `.builder-shell__palette` (the class is
// followed by whitespace, comma, or the rule brace).
describe('Builder styles (gotcha #67)', () => {
  const css = readFileSync(resolve(__dirname, '../../../../src/styles.css'), 'utf8');
  const BUILDER_CLASSES = [
    'builder-shell',
    'builder-shell__palette',
    'builder-shell__canvas',
    'builder-shell__inspector',
    'builder-palette',
    'builder-palette__tabs',
    'builder-palette__tab',
    'builder-palette__tab--active',
    'builder-palette__filter',
    'builder-palette__list',
    'builder-palette__item',
    'builder-palette__empty',
    'view-switch',
    'view-switch__option',
    'view-switch__option--active',
  ] as const;

  it.each(BUILDER_CLASSES)('styles.css defines a rule for .%s', (cls) => {
    expect(css).toMatch(new RegExp(`\\.${cls}[\\s,{]`));
  });

  it('builder styles use theme variables, not literal colors (M07-IRL #3)', () => {
    // The contrast bug Stage A fixed must not return — the Builder
    // styles reference --node-* theme tokens.
    expect(css).toMatch(/\.builder-shell[\s\S]*?var\(--node-/);
  });
});
