import { BuilderCanvas } from './BuilderCanvas';
import { NodeConfigPanel } from './NodeConfigPanel';
import { Palette } from './Palette';

/**
 * The three-panel Builder shell (M08.C — spec Phase 9): Palette left,
 * Canvas center, Inspector right, laid out as a CSS grid.
 *
 * The Palette ships fully working (C.3.5). M08.D1 mounts the
 * interactive `BuilderCanvas` + its inline `NodeConfigPanel` overlay
 * into the Canvas region. The Inspector region ships empty — a stub
 * Stage E fills — a real DOM landmark, not dead code.
 */
export function BuilderShell(): JSX.Element {
  return (
    <div className="builder-shell" data-testid="builder-shell">
      <aside className="builder-shell__palette" data-testid="builder-palette-region">
        <Palette />
      </aside>
      <section className="builder-shell__canvas" data-testid="builder-canvas-region">
        <BuilderCanvas />
        <NodeConfigPanel />
      </section>
      <aside className="builder-shell__inspector" data-testid="builder-inspector-region">
        {/* E mounts the Inspector here. */}
      </aside>
    </div>
  );
}
