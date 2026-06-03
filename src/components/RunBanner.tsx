export type RunStatus = 'idle' | 'running' | 'suspended' | 'done';

/**
 * The run banner over the canvas (M08.8.B; DESIGN.md "Live-graph
 * execution view"). A state pill that reflects the run — running,
 * suspended (capability gap), or complete. Collapses when idle. Pure
 * presentation: it shows the state C–F's flips drive; it owns no run
 * behavior.
 */
export function RunBanner({ status }: { status: RunStatus }): JSX.Element | null {
  if (status === 'idle') {
    return null;
  }
  return (
    <div
      className={`run-banner run-banner--${status}`}
      role="status"
      aria-live="polite"
      data-testid="run-banner"
    >
      <span className={`run-banner__pulse run-banner__pulse--${status}`} aria-hidden="true" />
      <span className="run-banner__label">
        {status === 'running' && 'Framework executing — events streaming live into the graph.'}
        {status === 'suspended' && 'Session suspended — a required capability is missing.'}
        {status === 'done' && 'Run complete.'}
      </span>
    </div>
  );
}
