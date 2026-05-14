import { useShallow } from 'zustand/react/shallow';
import { useGraphStore, type GapReactFlowNode } from '../lib/graphStore';

/**
 * Right-rail list of unresolved gaps — spec §4b (M05 Stage F).
 *
 * Subscribes to the graphStore's GapNodes (mounted by Stage A's four
 * `*_missing` event branches; dismissed by `gap_resolved`). Pure-render —
 * no event handlers of its own. Returns null when no gaps so the rail
 * doesn't float empty over the canvas.
 *
 * `useShallow` keeps the filter-derived array's identity stable across
 * unrelated store updates (Zustand v5 forbids returning fresh arrays from
 * a plain selector — `getSnapshot` would loop). Per gotcha #68: reads
 * `kind`, `missingName`, `severity`, `suggestedAction`, `agentId` — all
 * five fields the projection writes. Per gotcha #67: every
 * `.gap-panel__item--<severity>` class set here has a matching rule in
 * styles.css.
 */
export function GapPanel(): JSX.Element | null {
  const gaps = useGraphStore(
    useShallow((s) => s.nodes.filter((n): n is GapReactFlowNode => n.type === 'gap')),
  );
  if (gaps.length === 0) {
    return null;
  }
  return (
    <aside
      className="gap-panel"
      data-testid="gap-panel"
      role="region"
      aria-label="Unresolved capability gaps"
    >
      <h2 className="gap-panel__title" data-testid="gap-panel-title">
        Unresolved Gaps ({gaps.length})
      </h2>
      <ul className="gap-panel__list">
        {gaps.map((node) => {
          const { gapId, kind, missingName, severity, suggestedAction, agentId } = node.data;
          return (
            <li
              key={gapId}
              className={`gap-panel__item gap-panel__item--${severity}`}
              data-testid={`gap-item-${gapId}`}
              data-severity={severity}
            >
              <span className="gap-panel__kind">{kind}</span>
              <span className="gap-panel__name">{missingName}</span>
              <span className="gap-panel__suggested-action" title={suggestedAction}>
                {suggestedAction}
              </span>
              <span className="gap-panel__agent">agent: {agentId.slice(0, 8)}</span>
            </li>
          );
        })}
      </ul>
    </aside>
  );
}
