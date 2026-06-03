import type { ReactNode } from 'react';
import { ViewSwitch, type AppView } from './builder/ViewSwitch';

export interface AppShellProps {
  view: AppView;
  onViewChange: (view: AppView) => void;
  hasKey: boolean;
  /** The current runtime tier (`novice` / `promoted`) — shown in a chip. */
  tier: string;
  /** Full-width chrome under the topbar, present in BOTH views (the
   *  dormant budget bar + the collapsible Settings panel). */
  subchrome?: ReactNode;
  left?: ReactNode;
  center: ReactNode;
  right?: ReactNode;
}

function titleCase(s: string): string {
  return s.length === 0 ? s : s.charAt(0).toUpperCase() + s.slice(1);
}

/**
 * The Light Instrument shell (M08.8.B; DESIGN.md "Layout & spacing"): a
 * 52px top chrome (brand · tab nav · tier + key state) over a three-pane
 * workspace — a 232px left rail (palette / task+results), a fluid center
 * (canvas + transport), and a 360px right rail (Inspector / Output).
 *
 * Replaces the bare `<h1>Agent Runtime — M03 live graph</h1>` dev label
 * (IRL #20 / TD-009); the brand stays a heading so the surface is still
 * announced as "Agent Runtime". The side rails are omitted in single-pane
 * mode (the Builder owns its own internal layout).
 */
export function AppShell({
  view,
  onViewChange,
  hasKey,
  tier,
  subchrome,
  left,
  center,
  right,
}: AppShellProps): JSX.Element {
  const triple = left !== undefined && right !== undefined;
  return (
    <div className="app-shell" data-testid="app-shell">
      <header className="topbar" data-testid="topbar">
        <div className="brand">
          <span className="brand__mark" aria-hidden="true" />
          <h1 className="brand__name">
            Agent Runtime <span className="brand__ver mono-micro">workbench · v0.1</span>
          </h1>
        </div>
        <ViewSwitch value={view} onChange={onViewChange} />
        <span className="topbar__spacer" />
        <span className="chip chip--tier" data-testid="topbar-tier-chip">
          <span className="chip__swatch chip__swatch--tier" aria-hidden="true" />
          {titleCase(tier)}
        </span>
        <span className="chip chip--key mono-micro" data-testid="topbar-key-chip">
          <span
            className={`chip__swatch ${hasKey ? 'chip__swatch--ok' : 'chip__swatch--off'}`}
            aria-hidden="true"
          />
          {hasKey ? 'key active' : 'no key'}
        </span>
      </header>

      {subchrome !== undefined && <div className="subchrome">{subchrome}</div>}

      <div className={`workspace ${triple ? 'workspace--triple' : 'workspace--single'}`}>
        {left !== undefined && (
          <aside className="rail rail--left" data-testid="rail-left">
            {left}
          </aside>
        )}
        <section className="pane pane--center" data-testid="pane-center">
          {center}
        </section>
        {right !== undefined && (
          <aside className="rail rail--right" data-testid="rail-right">
            {right}
          </aside>
        )}
      </div>
    </div>
  );
}
