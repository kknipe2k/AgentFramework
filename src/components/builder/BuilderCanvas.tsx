import {
  Background,
  Controls,
  ReactFlow,
  ReactFlowProvider,
  useReactFlow,
  type Connection,
  type NodeChange,
  type NodeTypes,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import type React from 'react';
import { useCallback } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useBuilderStore, type BuilderNodeKind } from '../../lib/builderStore';
import { BuilderAgentNode } from './nodes/BuilderAgentNode';
import { BuilderHitlNode } from './nodes/BuilderHitlNode';
import { BuilderHookNode } from './nodes/BuilderHookNode';
import { BuilderSkillNode } from './nodes/BuilderSkillNode';
import { BuilderToolNode } from './nodes/BuilderToolNode';

// Defined OUTSIDE the component per @xyflow/react v12 docs + the
// GraphCanvas.tsx trap: nodeTypes is a stable-reference map; redefining
// it on each render forces React Flow to re-mount every node. One entry
// per BuilderNodeKind — keep it module-level.
export const builderNodeTypes: NodeTypes = {
  agent: BuilderAgentNode as NodeTypes[string],
  tool: BuilderToolNode as NodeTypes[string],
  skill: BuilderSkillNode as NodeTypes[string],
  hitl: BuilderHitlNode as NodeTypes[string],
  hook: BuilderHookNode as NodeTypes[string],
};

/** The drag MIME the Stage C Palette sets on every item — the C↔D1
 *  contract the drop handler reads. */
const DND_MIME = 'application/x-builder-node';

/**
 * Route React Flow v12 controlled-drag changes to `moveNode`. v12 is
 * fully controlled — a dropped node will not reposition on a user drag
 * unless the position change flows back into the store.
 */
export function applyPositionChanges(
  changes: NodeChange[],
  moveNode: (nodeId: string, position: { x: number; y: number }) => void,
): void {
  for (const change of changes) {
    if (change.type === 'position' && change.position !== undefined) {
      moveNode(change.id, change.position);
    }
  }
}

function BuilderCanvasInner(): JSX.Element {
  // Derived projection selectors — useShallow per gotcha #75 + the
  // <zustand_selector_audit>; canvasNodes() is itself memoized so the
  // selector returns a referentially stable array. canvasEdges is empty
  // until D2.
  const nodes = useBuilderStore(useShallow((s) => s.canvasNodes()));
  const edges = useBuilderStore(useShallow((s) => s.canvasEdges()));
  const addNode = useBuilderStore((s) => s.addNode);
  const moveNode = useBuilderStore((s) => s.moveNode);
  const selectNode = useBuilderStore((s) => s.selectNode);
  const connectEdge = useBuilderStore((s) => s.connectEdge);
  const { screenToFlowPosition } = useReactFlow();

  // React Flow v12 onConnect — fires on a handle-to-handle drag (D2,
  // filling the slot D1 left unset). connectEdge maps the (source,
  // target) pair to one of the four spec edge types and rejects every
  // other pair internally — a rejected pair mutates no `framework` so
  // the pure edge projection paints no wire.
  const onConnect = useCallback(
    (c: Connection) => {
      if (c.source !== null && c.target !== null) {
        connectEdge(c.source, c.target);
      }
    },
    [connectEdge],
  );

  function onDragOver(e: React.DragEvent): void {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
  }

  function onDrop(e: React.DragEvent): void {
    e.preventDefault();
    const raw = e.dataTransfer.getData(DND_MIME);
    if (raw.length === 0) {
      return; // not a Palette drag
    }
    const payload = JSON.parse(raw) as { kind: BuilderNodeKind; ref: string };
    // screenToFlowPosition converts the cursor point to canvas
    // coordinates so the node lands where the user dropped it.
    const position = screenToFlowPosition({ x: e.clientX, y: e.clientY });
    addNode(payload.kind, payload.ref, position);
  }

  return (
    <div
      className="builder-canvas"
      data-testid="builder-canvas"
      onDrop={onDrop}
      onDragOver={onDragOver}
    >
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={builderNodeTypes}
        onNodesChange={(changes) => applyPositionChanges(changes, moveNode)}
        onNodeClick={(_, node) => selectNode(node.id)}
        onPaneClick={() => selectNode(null)}
        onConnect={onConnect}
        fitView
      >
        <Background />
        <Controls />
      </ReactFlow>
    </div>
  );
}

/**
 * The interactive Builder Canvas (M08.D1/D2 — spec Phase 9) — a NEW
 * React-Flow node editor distinct from the read-only live-graph
 * `GraphCanvas`. Nodes and edges are a projection of
 * `builderStore.framework` (ADR-0020); a Palette drop instantiates a
 * node, a user drag repositions it, and a handle-to-handle connection
 * (`onConnect`, D2) records one of the four spec edge types.
 *
 * Wrapped in `ReactFlowProvider` so the drop handler can call
 * `useReactFlow().screenToFlowPosition`.
 */
export function BuilderCanvas(): JSX.Element {
  return (
    <ReactFlowProvider>
      <BuilderCanvasInner />
    </ReactFlowProvider>
  );
}
