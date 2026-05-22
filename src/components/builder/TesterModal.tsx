import type { WireDuration } from '../../lib/ipc';

// M08.F2 — the Builder Tester modal. RED-PHASE STUB; the green phase
// implements the modal, the scoped graph pane, and the result surfaces.

/** Fold serde's Duration shape ({ secs, nanos }) to a millisecond label. */
function formatTiming(_duration: WireDuration): string {
  return '';
}

export function TesterModal(): JSX.Element | null {
  return null;
}

export const _testing = { formatTiming };
