import { Background, Controls, ReactFlow } from '@xyflow/react';
import { useMemo } from 'react';
import { useTestGraphStore } from '../../lib/builderStore';
import { layoutGraph } from '../../lib/layout';
import { nodeTypes } from '../GraphCanvas';

/**
 * The Tester's smaller graph pane (spec Phase 9 — "watch graph render in
 * a smaller pane"; M08.F2).
 *
 * Renders the TEST session's nodes/edges from `useTestGraphStore` — the
 * SCOPED graph store, NOT the live `useGraphStore` module singleton.
 * Reducing a test run into the live store would corrupt the runtime
 * graph; the scoped store keeps build-time and run-time disjoint.
 *
 * Reuses the live-graph rendering verbatim: the module-level 11-entry
 * `nodeTypes` map (imported from `GraphCanvas`, never redefined — the
 * @xyflow/react v12 stable-reference trap) and the pure `layoutGraph`
 * dagre pass.
 */
export function TesterGraphPane(): JSX.Element {
  const nodes = useTestGraphStore((s) => s.nodes);
  const edges = useTestGraphStore((s) => s.edges);
  const selectNode = useTestGraphStore((s) => s.selectNode);
  // `layoutGraph` is pure; memoize on the store's array identities so the
  // dagre pass re-runs only when the scoped graph actually changes (the
  // arrays are referentially stable between reducer no-ops).
  const laidNodes = useMemo(() => layoutGraph(nodes, edges), [nodes, edges]);
  return (
    <div className="tester-graph-pane" data-testid="tester-graph-pane">
      <ReactFlow
        nodes={laidNodes}
        edges={edges}
        nodeTypes={nodeTypes}
        fitView
        // M08.8.A: clicking a node selects it in the SCOPED store so the
        // Tester's Inspector rail surfaces its payload (the live
        // GraphCanvas wires the same onNodeClick → selectNode).
        onNodeClick={(_, node) => selectNode(node.id)}
        onPaneClick={() => selectNode(null)}
      >
        <Background />
        <Controls />
      </ReactFlow>
    </div>
  );
}
