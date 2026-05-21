export interface CapabilityDisclosureProps {
  /** Plain-English capability lines to disclose. */
  capabilities: string[];
  /** Rendered as a single line when `capabilities` is empty. */
  emptyMessage: string;
  /** Test/landmark id placed on the rendered list or empty paragraph. */
  'data-testid'?: string;
}

/**
 * The shared plain-English capability-disclosure surface — the M05
 * §8.security L1 disclosure. Lifted out of `ImportPanel`'s import-review
 * modal at M08.D1 as a behavior-preserving extraction so the Builder
 * Canvas nodes render the same surface (its third reuse: M05 → M07.E →
 * here). Renders a declared-capability list, or one empty-state line
 * when there is nothing to disclose.
 *
 * @example
 * ```tsx
 * <CapabilityDisclosure
 *   capabilities={['Can use the Read tool']}
 *   emptyMessage="No tools or skills assigned yet."
 * />
 * ```
 */
export function CapabilityDisclosure({
  capabilities,
  emptyMessage,
  'data-testid': testId,
}: CapabilityDisclosureProps): JSX.Element {
  if (capabilities.length === 0) {
    return (
      <p
        className="import-capability-disclosure import-capability-disclosure--empty"
        data-testid={testId}
      >
        {emptyMessage}
      </p>
    );
  }
  return (
    <ul className="import-capability-disclosure" data-testid={testId}>
      {capabilities.map((c) => (
        <li key={c} className="import-capability-disclosure__item">
          {c}
        </li>
      ))}
    </ul>
  );
}
