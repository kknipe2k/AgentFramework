/**
 * Render a tool payload for inspection (the M08.8.A Output-rail surface):
 * strings pass through verbatim (a file body reads as-is in the mono
 * register), everything else is pretty-printed JSON. `undefined` shows an
 * explicit em-dash placeholder.
 *
 * Lifted out of `InspectorPanel` at M08.9.B as a behavior-preserving
 * extraction so the Tester's run drill-down (`TraceDrilldown`) renders
 * `outcome.trace` tool input/result through the SAME formatter the live
 * Inspector uses — one source of truth for payload formatting.
 */
export function formatPayload(value: unknown): string {
  if (value === undefined) {
    return '—';
  }
  if (typeof value === 'string') {
    return value;
  }
  return JSON.stringify(value, null, 2);
}
