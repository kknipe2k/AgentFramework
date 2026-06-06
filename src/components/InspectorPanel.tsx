import { useEffect, useRef } from 'react';
import { useGraphStore } from '../lib/graphStore';

/**
 * Right-rail Output/Inspector panel (DESIGN.md "Inspector / Output" rail;
 * M08.8.A / TD-034). Two stacked sections:
 *
 *  - **Output** — the agent's streamed reply text (`outputBuffer`),
 *    rendered live in the mono "instrument register". Pre-M08.8.A the
 *    reply was a `graphStore` no-op watchable only via `RUST_LOG`.
 *  - **Inspector** — the selected node's payload. For a tool node it
 *    surfaces the retained `{input, output}` (the path a `Read` read +
 *    the contents it returned) — the other half TD-034 closed.
 *
 * Parameterized by `store` so the SAME rail binds to either the live
 * `useGraphStore` (the runtime view) or the Tester's scoped
 * `useTestGraphStore` (works over BOTH stores — the M08.8.A requirement).
 *
 * ARIA: `role="dialog"` + `aria-modal="false"` per WAI APG dialog pattern
 * — non-modal so the graph behind the panel stays interactable. ESC +
 * close-button both clear the store's selection.
 */
export function InspectorPanel({
  store = useGraphStore,
}: {
  store?: typeof useGraphStore;
}): JSX.Element | null {
  const useStore = store;
  const selectedNodeId = useStore((s) => s.selectedNodeId);
  const selectedNode = useStore((s) =>
    s.selectedNodeId ? s.nodes.find((n) => n.id === s.selectedNodeId) : null,
  );
  const outputBuffer = useStore((s) => s.outputBuffer);
  const selectNode = useStore((s) => s.selectNode);
  const panelRef = useRef<HTMLDivElement>(null);

  // Move focus to the panel root on selection so screen readers announce
  // it. Per WAI APG dialog pattern; non-modal so we don't trap focus.
  useEffect(() => {
    if (selectedNodeId && panelRef.current) {
      panelRef.current.focus();
    }
  }, [selectedNodeId]);

  // ESC dismisses per ARIA dialog pattern. Listener attaches only when a
  // node is selected so background ESC presses don't churn the store.
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

  const hasOutput = outputBuffer.length > 0;

  // The rail shows whenever there is streamed output OR a selected node —
  // the Output section is session-level (not node-keyed), so it surfaces
  // the reply even before the user clicks a node.
  if (!selectedNodeId && !hasOutput) {
    return null;
  }

  const isTool = selectedNode?.type === 'tool';
  const toolInput = isTool ? selectedNode.data.input : undefined;
  const toolOutput = isTool ? selectedNode.data.output : undefined;

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
        <h2 className="inspector-panel__title">
          {selectedNode ? `${selectedNode.type} node` : 'Output'}
        </h2>
        {selectedNodeId && (
          <button
            type="button"
            className="inspector-panel__close"
            onClick={() => selectNode(null)}
            aria-label="close inspector"
          >
            ×
          </button>
        )}
      </header>

      {hasOutput && (
        <section className="inspector-panel__output-section" data-testid="output-rail">
          <h3 className="inspector-panel__section-label">Output</h3>
          <pre className="inspector-panel__output">{outputBuffer}</pre>
        </section>
      )}

      {selectedNode && (
        <section className="inspector-panel__node-section">
          {isTool && (toolInput !== undefined || toolOutput !== undefined) && (
            <div className="inspector-panel__tool-io">
              <h3 className="inspector-panel__section-label">Input</h3>
              <pre className="inspector-panel__io" data-testid="inspector-tool-input">
                {formatPayload(toolInput)}
              </pre>
              <h3 className="inspector-panel__section-label">Output</h3>
              <pre className="inspector-panel__io" data-testid="inspector-tool-output">
                {formatPayload(toolOutput)}
              </pre>
            </div>
          )}
          <h3 className="inspector-panel__section-label">Node data</h3>
          <pre className="inspector-panel__data">{JSON.stringify(selectedNode.data, null, 2)}</pre>
        </section>
      )}
    </aside>
  );
}

/**
 * Render a tool payload for the Inspector: strings pass through verbatim
 * (a file body reads as-is in the mono register), everything else is
 * pretty-printed JSON. `undefined` shows an explicit em-dash placeholder.
 */
function formatPayload(value: unknown): string {
  if (value === undefined) {
    return '—';
  }
  if (typeof value === 'string') {
    return value;
  }
  return JSON.stringify(value, null, 2);
}
