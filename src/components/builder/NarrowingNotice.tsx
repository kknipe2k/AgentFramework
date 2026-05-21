import { useShallow } from 'zustand/react/shallow';
import { useBuilderStore } from '../../lib/builderStore';
import type { CapabilityDeclaration } from '../../types/capability';

interface NarrowingNoticeProps {
  /** Spawn-edge id `agent:<parent>-><child>` (D2.3.6's id scheme). */
  spawnEdgeId: string;
}

/** `kind:resource` — a stable display label for one capability. */
function capLabel(c: CapabilityDeclaration): string {
  return `${c.kind}:${c.resource}`;
}

/** A capability list as a comma-joined label string, or `(none)`. */
function capList(caps: CapabilityDeclaration[]): string {
  return caps.map(capLabel).join(', ') || '(none)';
}

/**
 * Surfaces one Agent→Agent (`spawns`) edge's §8.security L2a narrowing
 * decision (M08.D2 — spec Phase 9 / MVP §M8 criterion 3). The decision
 * is computed by `capability/narrowing.rs::narrow()` in the Rust main
 * process (M05.B); this component reads the `SpawnEdgeNarrowing` triple
 * off the `validate_framework` report's
 * `capability_summary.spawn_edges[]` and renders the backend's
 * `narrowed_caps` arm verbatim — spec §9 forbids a TS re-implementation
 * of the intersection, so it computes nothing. Renders `null` when no
 * spawn edge matches `spawnEdgeId`.
 */
export function NarrowingNotice({ spawnEdgeId }: NarrowingNoticeProps): JSX.Element | null {
  // The narrowing decisions ride on the validate_framework report's
  // capability_summary.spawn_edges[]; find this edge by its
  // `agent:<parent>-><child>` id. useShallow so the notice re-renders
  // only on its own slice (gotcha #75).
  const edge = useBuilderStore(
    useShallow(
      (s) =>
        s.validation?.capability_summary?.spawn_edges.find(
          (e) => `agent:${e.parent_id}->${e.child_id}` === spawnEdgeId,
        ) ?? null,
    ),
  );
  if (edge === null) {
    return null;
  }
  const declared = capList(edge.child_declared_caps);

  // narrowed_caps is the serde-tagged Result `narrow` returned. An
  // `Err` means the child declared a capability the parent does not
  // hold — L2a all-or-nothing rejects the whole edge (Stage B folds the
  // Err into capability_errors → the child node's red badge).
  if ('Err' in edge.narrowed_caps) {
    return (
      <aside className="narrowing-notice" role="note">
        <h4>Capability narrowing rejected</h4>
        <p data-testid="narrowing-notice-declared">Child declared: {declared}</p>
        <p className="narrowing-notice__rejected">{edge.narrowed_caps.Err}</p>
      </aside>
    );
  }
  // `Ok` — every child capability is subsumed by the parent. All-or-
  // nothing narrowing carries `proposed` verbatim, so the surviving set
  // is rendered straight from the backend value — no TS intersection.
  return (
    <aside className="narrowing-notice" role="note">
      <h4>Capability narrowing applied</h4>
      <p data-testid="narrowing-notice-declared">Child declared: {declared}</p>
      <p data-testid="narrowing-notice-survives">
        Survives intersection with the parent: {capList(edge.narrowed_caps.Ok)}
      </p>
    </aside>
  );
}
