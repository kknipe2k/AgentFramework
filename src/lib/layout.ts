import dagre from '@dagrejs/dagre';
import type { Edge, Node } from '@xyflow/react';

// Node-shape bounds dagre uses to compute the rank-and-rank gap. The
// dagre coords are CENTER-based; React Flow expects TOP-LEFT, so we
// translate by half the width/height at the end.
const NODE_WIDTH = 180;
const NODE_HEIGHT = 60;

/**
 * Run a top-down dagre layout over a snapshot of nodes + edges and
 * return a new node array with computed positions. Pure function — no
 * side effects, deterministic for a given input. Empty graph returns []
 * to avoid touching dagre on the cold-start render.
 *
 * Per React Flow v12's layouting guide
 * (https://reactflow.dev/learn/layouting/layouting), layout is a
 * visualization concern rather than state. Callers thread this from a
 * `useMemo` so the recomputation cost stays bounded by node-count
 * change, not every render.
 *
 * Generic over `N extends Node, E extends Edge` so the caller's narrow
 * node-union flows through unchanged. Both the live-graph store's
 * `GraphNode` discriminated union and the Builder canvas's narrower
 * agent / tool / skill / hitl / hook `Node[]` satisfy this — the
 * function reads only `id` and writes only `position`, both common to
 * every React Flow node shape. M08.6.D adds the Builder-side caller
 * (canvas auto-layout on framework load) sharing the same engine the
 * live `GraphCanvas` uses.
 */
export function layoutGraph<N extends Node, E extends Edge>(nodes: N[], edges: E[]): N[] {
  if (nodes.length === 0) {
    return nodes;
  }

  const g = new dagre.graphlib.Graph<Record<string, unknown>>();
  g.setDefaultEdgeLabel(() => ({}));
  g.setGraph({ rankdir: 'TB', nodesep: 50, ranksep: 80 });

  for (const node of nodes) {
    g.setNode(node.id, { width: NODE_WIDTH, height: NODE_HEIGHT });
  }
  for (const edge of edges) {
    if (g.node(edge.source) && g.node(edge.target)) {
      g.setEdge(edge.source, edge.target);
    }
  }

  dagre.layout(g);

  // Every node was added via `setNode` above, so dagre populates x/y on
  // each. Translate from dagre's center-based coords to React Flow's
  // top-left coords.
  return nodes.map((node) => {
    const laid = g.node(node.id) as { x: number; y: number };
    return {
      ...node,
      position: { x: laid.x - NODE_WIDTH / 2, y: laid.y - NODE_HEIGHT / 2 },
    };
  });
}
