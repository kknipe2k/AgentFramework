import { Handle, Position, type NodeProps } from '@xyflow/react';
import type { HITLNodeData, HITLReactFlowNode } from '../../lib/graphStore';

export function HITLNode({ data }: NodeProps<HITLReactFlowNode>): JSX.Element {
  const { hitlId, prompt, resolved }: HITLNodeData = data;
  // WAI ARIA APG: blocking-on-input affordance uses role=alert +
  // aria-live=assertive so screenreaders announce the prompt the
  // moment the node spawns.
  const modifier = resolved ? 'complete' : 'hitl';
  return (
    <div
      className={`hitl-node hitl-node--${modifier}`}
      data-testid={`hitl-node-${hitlId}`}
      role="alert"
      aria-live="assertive"
      aria-label={`hitl ${resolved ? 'resolved' : 'awaiting'}: ${prompt}`}
    >
      <Handle type="target" position={Position.Top} />
      <div className="hitl-node__prompt">{prompt}</div>
    </div>
  );
}
