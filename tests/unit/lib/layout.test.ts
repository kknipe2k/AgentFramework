import { describe, expect, it } from 'vitest';
import type { GraphEdge, GraphNode } from '../../../src/lib/graphStore';
import { layoutGraph } from '../../../src/lib/layout';

// Minimal AgentNode-shape factory so tests stay local to layout concerns
// (no Zustand, no React Flow runtime). The renderer types accept any
// `Node<Data, Type>` with `position`/`data`; we only inspect `position`
// here so the data shape is intentionally minimal.
function agent(id: string): GraphNode {
  return {
    id,
    type: 'agent',
    position: { x: 0, y: 0 },
    data: {
      agentId: id,
      agentName: id,
      status: 'active',
      parentAgentId: null,
      tokensIn: 0,
      tokensOut: 0,
    },
  } as GraphNode;
}

function edge(source: string, target: string): GraphEdge {
  return {
    id: `${source}->${target}`,
    source,
    target,
    data: { kind: 'agent-spawn' },
  };
}

describe('layoutGraph', () => {
  it('empty_graph_returns_empty', () => {
    expect(layoutGraph([], [])).toEqual([]);
  });

  it('single_node_graph_returns_one_positioned_node_with_finite_coords', () => {
    const out = layoutGraph([agent('a1')], []);
    expect(out).toHaveLength(1);
    const { x, y } = out[0]!.position;
    expect(Number.isFinite(x)).toBe(true);
    expect(Number.isFinite(y)).toBe(true);
  });

  it('parent_child_edge_produces_top_down_layout', () => {
    const nodes = [agent('parent'), agent('child')];
    const edges = [edge('parent', 'child')];
    const out = layoutGraph(nodes, edges);
    const parent = out.find((n) => n.id === 'parent')!;
    const child = out.find((n) => n.id === 'child')!;
    // dagre TB rankdir + parent→child edge ⇒ child below parent.
    expect(child.position.y).toBeGreaterThan(parent.position.y);
  });

  it('deterministic_for_same_input', () => {
    const nodes = [agent('a1'), agent('a2'), agent('a3')];
    const edges = [edge('a1', 'a2'), edge('a2', 'a3')];
    const first = layoutGraph(nodes, edges);
    const second = layoutGraph(nodes, edges);
    expect(first.map((n) => n.position)).toEqual(second.map((n) => n.position));
  });
});
