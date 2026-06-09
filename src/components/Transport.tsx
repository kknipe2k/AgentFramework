export interface TransportProps {
  /** Whether a prior session is available to re-drive (the existing replay). */
  replayAvailable: boolean;
  /** Re-drive the EXISTING `agent_event` replay (invokeReplaySession) — no
   *  new persistence/replay channel (M08.8.B scope lock). */
  onReplay: () => void;
}

/**
 * The transport over the canvas (M08.8.B; DESIGN.md "Live-graph execution
 * view"). Ports `livegraph.jsx`'s transport chrome — play / restart, a
 * timeline scrubber, speed. In B it re-drives the EXISTING replay
 * (`invokeReplaySession`); the scrubber position is presentational (a
 * scrub-to-seek backend is out of B's renderer-only scope). Collapses
 * when no session is replayable.
 */
export function Transport({ replayAvailable, onReplay }: TransportProps): JSX.Element | null {
  if (!replayAvailable) {
    return null;
  }
  return (
    <div className="transport" data-testid="transport">
      <button
        type="button"
        className="transport__play"
        aria-label="Replay session"
        data-testid="transport-play"
        onClick={onReplay}
      >
        ▶
      </button>
      <button
        type="button"
        className="transport__restart"
        aria-label="Restart replay"
        data-testid="transport-restart"
        onClick={onReplay}
      >
        ↻
      </button>
      <div className="transport__scrub" aria-hidden="true">
        <div className="transport__track">
          <div className="transport__fill" />
          <div className="transport__head" />
        </div>
      </div>
      {/* Elapsed clock — mono tabular so the digits don't jitter (the
          instrument register). Presentational placeholder in B/B.fix; a
          live elapsed wires with the scrub-to-seek backend later. */}
      <span className="transport__clock" aria-hidden="true">
        00:00
      </span>
      <div className="transport__speed" aria-hidden="true">
        <button type="button" className="transport__speed-opt transport__speed-opt--on">
          1×
        </button>
      </div>
    </div>
  );
}
