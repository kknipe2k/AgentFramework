import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

// CSS-content assertion (the M04-IRL precedent: a component can carry the
// right class names while styles.css never defines the rule). M08.8.B
// ports docs/design/workbench-mockup/tokens.css into styles.css:9,
// replacing the dark M03 :root. Verify the VALUES landed and the dark
// baseline is gone — the structural red→green signal for the repaint.
const css = readFileSync(resolve(__dirname, '../../src/styles.css'), 'utf8');

describe('styles.css — Light Instrument token port (M08.8.B)', () => {
  it('defines_the_light_surface_and_canvas_tokens', () => {
    expect(css).toContain('--app-bg: #e9eef6');
    expect(css).toContain('--canvas-bg: #f1f4fa');
    expect(css).toContain('--surface-0: #ffffff');
    expect(css).toContain('--surface-2: #eef2f8');
  });

  it('defines_the_full_five_node_kind_family_for_the_light_canvas', () => {
    expect(css).toContain('--kind-agent: #2563eb');
    expect(css).toContain('--kind-tool: #0d9488');
    expect(css).toContain('--kind-skill: #7c3aed');
    expect(css).toContain('--kind-hook: #db2777');
    expect(css).toContain('--kind-hitl: #d97706');
  });

  it('defines_the_orthogonal_status_family_as_semantic_aliases', () => {
    // The ported three-tier system aliases status onto the semantic
    // primitives (tokens.css) rather than re-stating hex — the rename-free
    // port. The literals live on --ok / --error.
    expect(css).toContain('--ok: #16a34a');
    expect(css).toContain('--error: #dc2626');
    expect(css).toContain('--st-idle: #98a3b4');
    expect(css).toContain('--st-complete: var(--ok)');
    expect(css).toContain('--st-error: var(--error)');
  });

  it('retires_the_dark_m03_root_so_no_surface_paints_dark', () => {
    expect(css).not.toContain('#15192a'); // the dark --node-bg
    expect(css).not.toContain('#0e1014'); // the dark .graph-canvas field
  });

  it('exposes_the_theming_hooks_on_the_document_root', () => {
    expect(css).toContain("[data-accent='cyan']");
    expect(css).toContain("[data-density='compact']");
    expect(css).toContain("[data-direction='expressive']");
  });
});
