import { useShallow } from 'zustand/react/shallow';
import { useBuilderStore } from '../../../lib/builderStore';
import type { FrameworkValidationReport, NodeError } from '../../../lib/ipc';

/**
 * The validation errors keyed to one node's path — schema + capability,
 * the offending-node filter D2's red badge derives from. Pure: a
 * `validate_framework` report keys each `NodeError` to a `node_path`
 * (`(root)` or a bare agent id); this returns the entries that key the
 * given node.
 */
export function nodeErrorsFor(
  report: FrameworkValidationReport | null,
  nodePath: string,
): NodeError[] {
  if (report === null) {
    return [];
  }
  return [...report.schema_errors, ...report.capability_errors].filter(
    (error) => error.node_path === nodePath,
  );
}

/**
 * Selector hook — the validation errors keyed to `nodePath`. Wrapped in
 * `useShallow` so a node re-renders only when ITS error slice changes,
 * not on every unrelated `validation` commit (gotcha #75).
 */
export function useNodeErrors(nodePath: string): NodeError[] {
  return useBuilderStore(useShallow((s) => nodeErrorsFor(s.validation, nodePath)));
}

interface NodeValidationBadgeProps {
  /** The node's validation errors (from {@link useNodeErrors}). */
  errors: NodeError[];
}

/**
 * The red validation badge a builder node renders when the continuous
 * `validate_framework` pass keys an error to it (M08.D2 — spec Phase 9
 * "errors surfaced as red badges"). Shows the at-a-glance error count;
 * the full messages are the `title` tooltip (and Stage E's Inspector).
 * Renders `null` when the node has no errors.
 */
export function NodeValidationBadge({ errors }: NodeValidationBadgeProps): JSX.Element | null {
  if (errors.length === 0) {
    return null;
  }
  return (
    <span
      className="builder-node__badge"
      role="alert"
      data-testid="builder-node-badge"
      title={errors.map((error) => error.message).join('\n')}
    >
      {errors.length}
    </span>
  );
}
