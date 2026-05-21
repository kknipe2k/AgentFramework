interface NarrowingNoticeProps {
  /** Spawn-edge id `agent:<parent>-><child>` (D2.3.6's id scheme). */
  spawnEdgeId: string;
}

// M08.D2 red-phase stub — the real component (D2.3.6) surfaces Stage B's
// per-spawn-edge narrowing decision from the validate_framework report's
// capability_summary.spawn_edges[]. Implemented in the green phase.
export function NarrowingNotice(_props: NarrowingNoticeProps): JSX.Element | null {
  return null;
}
