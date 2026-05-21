// M08.D1 stub — red phase. The interactive React-Flow Builder Canvas
// is implemented in the impl commit.
import type { NodeChange, NodeTypes } from '@xyflow/react';

export const builderNodeTypes: NodeTypes = {};

export function applyPositionChanges(
  _changes: NodeChange[],
  _moveNode: (nodeId: string, position: { x: number; y: number }) => void,
): void {
  // impl commit
}

export function BuilderCanvas(): JSX.Element {
  return <div />;
}
