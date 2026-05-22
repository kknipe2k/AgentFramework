import { useState } from 'react';
import { BuilderCanvas } from './BuilderCanvas';
import { Inspector } from './Inspector';
import { JsonView } from './JsonView';
import { NodeConfigPanel } from './NodeConfigPanel';
import { Palette } from './Palette';
import { TesterModal } from './TesterModal';

/** The center-region editor tab — the visual canvas or the raw JSON. */
type CenterTab = 'canvas' | 'json';

const CENTER_TABS: readonly { id: CenterTab; label: string }[] = [
  { id: 'canvas', label: 'Canvas' },
  { id: 'json', label: 'JSON' },
];

/**
 * The three-panel Builder shell (M08.C/D/E — spec Phase 9): Palette
 * left, the canvas / JSON editor center, Inspector right, laid out as a
 * CSS grid.
 *
 * M08.E fills the two regions Stage C stubbed: the center region gains
 * a Canvas | JSON tab toggle — the JSON tab is just another editor over
 * `builderStore.framework`, exactly as the canvas is (ADR-0020) — and
 * the Inspector region mounts the live `Inspector`.
 *
 * M08.F2 mounts the `TesterModal` — a non-blocking modal that renders
 * over the shell on `builderStore.testerOpen` (the Inspector's Test
 * button); it returns `null` while closed.
 */
export function BuilderShell(): JSX.Element {
  const [centerTab, setCenterTab] = useState<CenterTab>('canvas');
  return (
    <div className="builder-shell" data-testid="builder-shell">
      <aside className="builder-shell__palette" data-testid="builder-palette-region">
        <Palette />
      </aside>
      <section className="builder-shell__canvas" data-testid="builder-canvas-region">
        <div className="builder-tabs" role="tablist" aria-label="Canvas or JSON editor">
          {CENTER_TABS.map((tab) => (
            <button
              key={tab.id}
              type="button"
              role="tab"
              aria-selected={tab.id === centerTab}
              className={`builder-tab${tab.id === centerTab ? ' builder-tab--active' : ''}`}
              data-testid={`builder-tab-${tab.id}`}
              onClick={() => setCenterTab(tab.id)}
            >
              {tab.label}
            </button>
          ))}
        </div>
        {centerTab === 'canvas' ? (
          <>
            <BuilderCanvas />
            <NodeConfigPanel />
          </>
        ) : (
          <JsonView />
        )}
      </section>
      <aside className="builder-shell__inspector" data-testid="builder-inspector-region">
        <Inspector />
      </aside>
      {/* The Tester modal — renders over the shell on testerOpen (F2). */}
      <TesterModal />
    </div>
  );
}
