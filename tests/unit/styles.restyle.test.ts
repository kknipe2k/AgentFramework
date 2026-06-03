import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

// M08.8.B.fix — the restyle smoke. Stage B ported the tokens + shell but
// the M03–M08 component CSS was RECOLORED, not restyled to the mockup's
// workbench.css specs. This pins the structural restyle markers: the
// segmented register, the mockup node chip/dot dimensions, the broad
// mono/tabular instrument register, hairline-bordered panels at restrained
// radii, and the locally-bundled IBM Plex (the @import @12 is
// WebView2-blockable). Keeps the app's class names (rename-free) — asserts
// the adopted SPEC, not new names.
const css = readFileSync(resolve(__dirname, '../../src/styles.css'), 'utf8');

describe('styles.css — Light Instrument restyle (M08.8.B.fix)', () => {
  it('bundles_IBM_Plex_via_a_local_font_face_not_only_the_blockable_import', () => {
    expect(css).toMatch(/@font-face/);
    expect(css).toContain("'IBM Plex Sans'");
    expect(css).toContain("'IBM Plex Mono'");
    expect(css).toMatch(/url\(['"]?\.\/assets\/fonts\/[^)]*\.woff2/);
  });

  it('the_view_switch_is_a_segmented_control_grey_well_raised_selected', () => {
    // segmented: a grey well (surface-2) with the selected tab lifted onto
    // surface-0 + e1 + accent text — not the M03 filled-active button.
    expect(css).toMatch(/\.view-switch\s*\{[^}]*--surface-2/);
    expect(css).toMatch(/\.view-switch__option--active\s*\{[^}]*--surface-0/);
    expect(css).toMatch(/\.view-switch__option--active\s*\{[^}]*--e1/);
  });

  it('nodes_carry_the_mockup_glyph_chip_22px_and_status_dot_9px', () => {
    expect(css).toMatch(/\.node-glyph\s*\{[^}]*width:\s*22px/);
    expect(css).toMatch(/\.node-status-dot\s*\{[^}]*width:\s*9px/);
  });

  it('the_instrument_mono_register_is_applied_broadly', () => {
    // B set var(--font-mono) on ~8 selectors; the mockup sets mono on every
    // machine value (IDs, counts, budgets, durations, kv values, JSON,
    // source chips). The restyle lifts the register well past that.
    const count = (css.match(/var\(--font-mono\)/g) ?? []).length;
    expect(count).toBeGreaterThanOrEqual(18);
  });

  it('panels_use_hairline_borders_and_restrained_radii_not_M03_4_6px', () => {
    // ApprovalPanel + GapPanel lifted to the .panel spec (r-md = 8px,
    // 1px hairline border), not the M03 4/6px radii.
    expect(css).toMatch(/\.approval-panel\s*\{[^}]*var\(--r-md\)/);
    expect(css).toMatch(/\.gap-panel\s*\{[^}]*var\(--r-md\)/);
  });

  it('machine_counts_read_tabular_in_the_transport_clock', () => {
    expect(css).toMatch(/\.transport__clock[\s\S]*?tabular-nums/);
  });
});
