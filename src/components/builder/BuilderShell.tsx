import { Palette } from './Palette';

/**
 * The three-panel Builder shell (M08.C — spec Phase 9): Palette left,
 * Canvas center, Inspector right, laid out as a CSS grid.
 *
 * The Palette ships fully working (C.3.5). The Canvas region ships
 * empty — a valid React-Flow drop target Stage D1 fills — and the
 * Inspector region ships empty — a stub Stage E fills. Both empty
 * regions are real DOM landmarks (`data-testid`), not dead code: D1/E
 * mount their components into them (incremental construction).
 */
export function BuilderShell(): JSX.Element {
  return (
    <div className="builder-shell" data-testid="builder-shell">
      <aside className="builder-shell__palette" data-testid="builder-palette-region">
        <Palette />
      </aside>
      <section className="builder-shell__canvas" data-testid="builder-canvas-region">
        {/* D1 mounts <BuilderCanvas/> here; until then an empty drop target. */}
      </section>
      <aside className="builder-shell__inspector" data-testid="builder-inspector-region">
        {/* E mounts the Inspector here. */}
      </aside>
    </div>
  );
}
