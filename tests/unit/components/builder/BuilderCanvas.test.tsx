import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// M08.D1/D2 — the interactive Builder Canvas. @xyflow/react's <ReactFlow>
// is replaced with a deterministic test double: React Flow's real
// rendering needs a measured pane (ResizeObserver, DOMMatrix) that
// happy-dom does not provide, and the canvas unit test's job is to
// verify BuilderCanvas's WIRING — the framework->projection feed, the
// drop handler, the selection handlers, and (D2) the onConnect edge
// route + the edge projection — not React Flow's own rendering. The
// real <ReactFlow> integration (a node renders, an edge drag works) is
// covered by the Playwright spec against a real browser.
interface MockFlowNode {
  id: string;
  type: string;
  position: { x: number; y: number };
  data: Record<string, unknown>;
}
interface MockFlowEdge {
  id: string;
  source: string;
  target: string;
}
interface MockConnection {
  source: string | null;
  target: string | null;
  sourceHandle: string | null;
  targetHandle: string | null;
}
interface MockReactFlowProps {
  nodes: MockFlowNode[];
  edges?: MockFlowEdge[];
  onNodeClick?: (e: unknown, n: MockFlowNode) => void;
  onPaneClick?: (e?: unknown) => void;
  onConnect?: (c: MockConnection) => void;
}

vi.mock('@xyflow/react', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@xyflow/react')>();
  return {
    ...actual,
    // screenToFlowPosition is identity here — the drop handler's coord
    // conversion is React Flow's concern; the test asserts addNode is
    // fed the converted point, whatever it is.
    useReactFlow: () => ({ screenToFlowPosition: (p: { x: number; y: number }) => p }),
    ReactFlow: ({ nodes, edges, onNodeClick, onPaneClick, onConnect }: MockReactFlowProps) => (
      <div data-testid="rf-mock">
        <button type="button" data-testid="rf-pane" onClick={() => onPaneClick?.()} />
        {/* D2 — fires a handle-to-handle connection the way React Flow's
            real onConnect would; BuilderCanvas routes it to connectEdge. */}
        <button
          type="button"
          data-testid="rf-connect"
          onClick={() =>
            onConnect?.({
              source: 'agent:planner',
              target: 'skill:research',
              sourceHandle: null,
              targetHandle: null,
            })
          }
        />
        {nodes.map((n) => (
          <button
            type="button"
            key={n.id}
            data-testid={`rf-node-${n.id}`}
            data-node-type={n.type}
            onClick={() => onNodeClick?.(undefined, n)}
          >
            {n.id}
          </button>
        ))}
        {(edges ?? []).map((e) => (
          <div
            key={e.id}
            data-testid={`rf-edge-${e.id}`}
            data-source={e.source}
            data-target={e.target}
          />
        ))}
      </div>
    ),
  };
});

// builderStore imports validateFramework from ipc — D2's addNode /
// connectEdge schedule a debounced validate_framework call. Mock it
// (partial — unwrapCmdError + the other ipc exports stay real) so the
// canvas wiring tests never reach the real Tauri bridge.
const { validateFrameworkMock } = vi.hoisted(() => ({ validateFrameworkMock: vi.fn() }));
vi.mock('../../../../src/lib/ipc', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../../../src/lib/ipc')>();
  return { ...actual, validateFramework: validateFrameworkMock };
});

import { createEvent, fireEvent, render, screen } from '@testing-library/react';
import {
  BuilderCanvas,
  applyPositionChanges,
  builderNodeTypes,
} from '../../../../src/components/builder/BuilderCanvas';
import { useBuilderStore } from '../../../../src/lib/builderStore';

describe('BuilderCanvas', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    validateFrameworkMock.mockReset().mockResolvedValue({
      schema_errors: [],
      capability_errors: [],
      ok: true,
      capability_summary: null,
    });
    // Full reset — tests below swap addNode / selectNode / connectEdge
    // for spies, so the real actions must be restored between cases.
    useBuilderStore.setState(useBuilderStore.getInitialState(), true);
  });

  afterEach(() => {
    vi.runOnlyPendingTimers();
    vi.useRealTimers();
  });

  it('builderNodeTypes_is_module_level_with_an_entry_per_kind', () => {
    // Module-level per the GraphCanvas trap — redefining nodeTypes per
    // render re-mounts every node. One entry per BuilderNodeKind.
    expect(Object.keys(builderNodeTypes).sort()).toEqual([
      'agent',
      'hitl',
      'hook',
      'skill',
      'tool',
    ]);
  });

  it('renders_a_node_per_canvasNodes_projection_entry', () => {
    const store = useBuilderStore.getState();
    store.addNode('agent', 'planner', { x: 0, y: 0 });
    store.addNode('agent', 'researcher', { x: 0, y: 0 });
    render(<BuilderCanvas />);
    expect(screen.getByTestId('rf-node-agent:planner')).toBeInTheDocument();
    expect(screen.getByTestId('rf-node-agent:researcher')).toBeInTheDocument();
  });

  it('onDrop_parses_the_palette_payload_and_calls_addNode', () => {
    const addNode = vi.fn();
    useBuilderStore.setState({ addNode });
    render(<BuilderCanvas />);
    // happy-dom's fireEvent.drop init does not propagate clientX/clientY
    // onto the synthetic event — construct the drop event and pin the
    // cursor coords explicitly so the screenToFlowPosition conversion is
    // genuinely exercised.
    const canvas = screen.getByTestId('builder-canvas');
    const drop = createEvent.drop(canvas, {
      dataTransfer: { getData: () => JSON.stringify({ kind: 'tool', ref: 'Read' }) },
    });
    Object.defineProperty(drop, 'clientX', { value: 50 });
    Object.defineProperty(drop, 'clientY', { value: 60 });
    fireEvent(canvas, drop);
    expect(addNode).toHaveBeenCalledWith('tool', 'Read', { x: 50, y: 60 });
  });

  it('onDrop_ignores_a_drag_without_the_builder_node_mime_type', () => {
    const addNode = vi.fn();
    useBuilderStore.setState({ addNode });
    render(<BuilderCanvas />);
    fireEvent.drop(screen.getByTestId('builder-canvas'), {
      dataTransfer: { getData: () => '' },
      clientX: 10,
      clientY: 10,
    });
    expect(addNode).not.toHaveBeenCalled();
  });

  it('clicking_a_node_calls_selectNode', () => {
    const selectNode = vi.fn();
    useBuilderStore.getState().addNode('agent', 'planner', { x: 0, y: 0 });
    useBuilderStore.setState({ selectNode });
    render(<BuilderCanvas />);
    fireEvent.click(screen.getByTestId('rf-node-agent:planner'));
    expect(selectNode).toHaveBeenCalledWith('agent:planner');
  });

  it('clicking_the_pane_clears_the_selection', () => {
    const selectNode = vi.fn();
    useBuilderStore.setState({ selectNode });
    render(<BuilderCanvas />);
    fireEvent.click(screen.getByTestId('rf-pane'));
    expect(selectNode).toHaveBeenCalledWith(null);
  });

  it('onNodesChange_with_a_position_change_calls_moveNode', () => {
    // React Flow v12 is fully controlled — a user node drag arrives as
    // an onNodesChange position change; BuilderCanvas routes it here.
    const moveNode = vi.fn();
    applyPositionChanges(
      [{ id: 'agent:planner', type: 'position', position: { x: 7, y: 8 }, dragging: true }],
      moveNode,
    );
    expect(moveNode).toHaveBeenCalledWith('agent:planner', { x: 7, y: 8 });
  });

  it('onNodesChange_ignores_a_non_position_change', () => {
    const moveNode = vi.fn();
    applyPositionChanges([{ id: 'agent:planner', type: 'select', selected: true }], moveNode);
    expect(moveNode).not.toHaveBeenCalled();
  });

  // ── D2 — onConnect edge route + the edge projection ──────────────
  it('onConnect_routes_a_connection_to_connectEdge', () => {
    // React Flow v12's onConnect fires on a handle-to-handle drag;
    // BuilderCanvas fills the D1-deferred slot and routes the
    // connection's (source, target) to builderStore.connectEdge.
    const connectEdge = vi.fn();
    useBuilderStore.setState({ connectEdge });
    render(<BuilderCanvas />);
    fireEvent.click(screen.getByTestId('rf-connect'));
    expect(connectEdge).toHaveBeenCalledWith('agent:planner', 'skill:research');
  });

  it('canvas_renders_an_edge_per_canvasEdges_projection_entry', () => {
    const store = useBuilderStore.getState();
    store.addNode('agent', 'planner', { x: 0, y: 0 });
    store.addNode('skill', 'research', { x: 0, y: 0 });
    store.connectEdge('agent:planner', 'skill:research');
    render(<BuilderCanvas />);
    // The edge projection feeds <ReactFlow edges> — a connectEdge
    // mutation is the only way a wire appears (ADR-0020).
    expect(screen.getByTestId('rf-edge-agent:planner->skill:research')).toBeInTheDocument();
  });
});

// gotcha #67 — a className with no styles.css rule renders unstyled and
// the user sees nothing. Every Builder class introduced this stage must
// have a corresponding rule, and use --node-* theme tokens (M07-IRL #3).
describe('Builder canvas styles (gotcha #67)', () => {
  const css = readFileSync(resolve(__dirname, '../../../../src/styles.css'), 'utf8');
  const D1_CLASSES = [
    'builder-canvas',
    'builder-agent-node',
    'builder-tool-node',
    'builder-skill-node',
    'builder-hitl-node',
    'builder-hook-node',
    'builder-node-config',
    'builder-node-config__title',
    'builder-node-config__field',
    'builder-node-config__list',
    'builder-node-config__list-item',
    'builder-node-config__add',
    'import-capability-disclosure--empty',
  ] as const;

  it.each(D1_CLASSES)('styles.css defines a rule for .%s', (cls) => {
    expect(css).toMatch(new RegExp(`\\.${cls}[\\s,{]`));
  });

  it('builder canvas styles use theme variables, not literal colors (M07-IRL #3)', () => {
    expect(css).toMatch(/\.builder-canvas[\s\S]*?var\(--node-/);
  });
});
