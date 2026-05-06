import { useEffect, useRef } from 'react';
import { useGraphStore } from '../lib/graphStore';

/**
 * Right-rail node-inspector panel. Subscribes to `selectedNodeId` via
 * Zustand selectors and renders the selected node's data as JSON. Stage
 * E will extend the panel with VDR-correlated decision history.
 *
 * ARIA: `role="dialog"` + `aria-modal="false"` per WAI APG dialog
 * pattern — non-modal so the graph behind the panel stays interactable.
 * ESC + close-button both clear the store's selection (the single
 * source of truth; `<GraphCanvas>`'s `onPaneClick` does the same).
 */
export function InspectorPanel(): JSX.Element | null {
  const selectedNodeId = useGraphStore((s) => s.selectedNodeId);
  const selectedNode = useGraphStore((s) =>
    s.selectedNodeId ? s.nodes.find((n) => n.id === s.selectedNodeId) : null,
  );
  const selectNode = useGraphStore((s) => s.selectNode);
  const panelRef = useRef<HTMLDivElement>(null);

  // Move focus to the panel root on selection so screen readers announce
  // it. Per WAI APG dialog pattern; non-modal so we don't trap focus.
  useEffect(() => {
    if (selectedNodeId && panelRef.current) {
      panelRef.current.focus();
    }
  }, [selectedNodeId]);

  // ESC dismisses per ARIA dialog pattern. Listener attaches only when
  // the panel is open so background ESC presses don't churn the store.
  useEffect(() => {
    if (!selectedNodeId) {
      return undefined;
    }
    const onKey = (e: KeyboardEvent): void => {
      if (e.key === 'Escape') {
        selectNode(null);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [selectedNodeId, selectNode]);

  if (!selectedNodeId || !selectedNode) {
    return null;
  }

  return (
    <aside
      ref={panelRef}
      className="inspector-panel"
      role="dialog"
      aria-modal="false"
      aria-label="node inspector"
      tabIndex={-1}
      data-testid="inspector-panel"
    >
      <header className="inspector-panel__header">
        <h2 className="inspector-panel__title">{selectedNode.type} node</h2>
        <button
          type="button"
          className="inspector-panel__close"
          onClick={() => selectNode(null)}
          aria-label="close inspector"
        >
          ×
        </button>
      </header>
      <pre className="inspector-panel__data">{JSON.stringify(selectedNode.data, null, 2)}</pre>
      {/* Stage E adds VDR-correlated decision history here. */}
    </aside>
  );
}
