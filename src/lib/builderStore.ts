import type { Edge, Node } from '@xyflow/react';
import { create } from 'zustand';
import type { Agent, Framework } from '../types/framework';
import { createGraphStore } from './graphStore';
import { unwrapCmdError, validateFramework, type FrameworkValidationReport } from './ipc';
import { layoutGraph } from './layout';

// M08.C/D1/D2 — the Builder store (ADR-0020). builderStore holds the
// in-progress framework.json as the single source of truth; the canvas
// is a projection derived from `framework`, and canvas edits mutate
// `framework`. It is a SEPARATE Zustand store from `graphStore` (the
// live-execution store) — the two have disjoint lifecycles (build-time
// vs run-time) and conflating them is the dual-purpose-store
// anti-pattern. C shipped the store shape + replaceFramework /
// setDiskFramework / selectNode / setValidation; D1 implemented addNode
// / updateNode / moveNode + the framework→canvas node projection; D2
// implements connectEdge (the four edge types) + the canvasEdges
// projection + the debounced continuous-validation trigger; E adds
// testerOpen + openTester / closeTester (the Inspector Test button —
// F2 renders the Tester modal on `testerOpen`). removeNode stays a
// typed no-op stub — node deletion is not in D2's scope.

/** One Palette item dragged onto the canvas. */
export type BuilderNodeKind = 'agent' | 'tool' | 'skill' | 'hitl' | 'hook';

/** A canvas coordinate. */
interface Position {
  x: number;
  y: number;
}

/** The default model id a freshly dropped Agent node carries — kept
 *  aligned with `emptyFramework()`'s top-level model. */
const DEFAULT_MODEL_ID = 'claude-sonnet-4-6';

/** Runtime built-in tools — a built-in Tool drop records `source:
 *  'builtin'`; any other Tool is an imported `'external'` artifact. */
const BUILTIN_TOOL_NAMES = ['Read', 'Write', 'Bash'];

/** The quiet interval (ms) the debounced continuous-validation trigger
 *  (D2.3.4) waits after the last `framework` mutation before firing one
 *  `validate_framework` call — a burst of canvas edits coalesces into a
 *  single backend pass rather than one per keystroke. */
export const VALIDATION_DEBOUNCE_MS = 250;

/** The Builder store contract — SEPARATE from `graphStore` (ADR-0020). */
export interface BuilderState {
  /** THE source of truth — the in-progress framework.json. */
  framework: Framework;
  /** Last saved/loaded snapshot — Stage E diffs `framework` against it. */
  diskFramework: Framework | null;
  /** Selected canvas node id — drives D1's inline config + E's Inspector. */
  selectedNodeId: string | null;
  /** Latest validate_framework report (D2 continuous pass populates it;
   *  null until the first pass completes). */
  validation: FrameworkValidationReport | null;
  /** Whether the Tester is open — the Inspector's Test button (E) sets
   *  it; Stage F2's modal renders on it. INERT-but-wired at Stage E. */
  testerOpen: boolean;

  /** Replace the whole document (E's JSON-tab edit). NOT the
   *  load-from-disk path — Stage M08.6.D split that out to
   *  `applyLoadedFramework` so a JSON-tab edit preserves the user's
   *  manual `nodePositions`. */
  replaceFramework: (fw: Framework) => void;
  /** M08.6.D: swap the document AND seed `nodePositions` via the
   *  existing `layoutGraph` dagre top-down layout — the canvas
   *  auto-lays-out on a framework LOAD instead of stacking every node
   *  at {0,0}. Used by the Inspector's Load button (ADR-0022's loader
   *  resolution returns inline agents that the canvas projection now
   *  paints with real positions). Distinct from `replaceFramework` so
   *  auto-layout fires on LOAD only, not on every framework mutation
   *  — ADR-0020 keeps `nodePositions` as editor-local view state. */
  applyLoadedFramework: (fw: Framework) => void;
  /** Record the on-disk snapshot after a save/load (E's diff baseline). */
  setDiskFramework: (fw: Framework | null) => void;
  /** Select / clear the active canvas node. */
  selectNode: (id: string | null) => void;
  /** D1: instantiate a node from a dropped Palette item. */
  addNode: (kind: BuilderNodeKind, ref: string, position: Position) => void;
  /** D1: inline-config edit (role / model / allowed_*). */
  updateNode: (nodeId: string, patch: Record<string, unknown>) => void;
  /** D2: map a connection to one of the four spec edge types — the
   *  right `framework` mutation — and reject every other node-pair. */
  connectEdge: (sourceId: string, targetId: string) => void;
  /** Drop a node — a typed no-op stub; node deletion is not in D2 scope. */
  removeNode: (nodeId: string) => void;
  /** Store a fresh validation report (D2 continuous + E explicit). */
  setValidation: (report: FrameworkValidationReport) => void;
  /** E: open the Tester (the Inspector Test button). F2 renders the
   *  modal on `testerOpen`; INERT-but-wired at Stage E. */
  openTester: () => void;
  /** E: close the Tester (F2's modal close control). */
  closeTester: () => void;

  /** D1: user-placed canvas coordinates, keyed by `${kind}:${ref}` node
   *  id — editor-local layout view state (ADR-0020: NOT part of the
   *  framework document, not persisted by save_framework). */
  nodePositions: Record<string, Position>;
  /** D1: derive the React-Flow node array from `framework` +
   *  `nodePositions` — the ADR-0020 framework→canvas projection. */
  canvasNodes: () => Node[];
  /** D2: derive the React-Flow edge array from `framework` — the four
   *  edge types projected back as wires (a pure function of the
   *  document, ADR-0020). */
  canvasEdges: () => Edge[];
  /** D1: update one node's canvas position (React Flow v12 controlled
   *  drag). Touches `nodePositions` only — never `framework`. */
  moveNode: (nodeId: string, position: Position) => void;
}

/**
 * The cold-start framework.json a fresh Builder session opens with.
 *
 * Deliberately pre-valid: schemas/framework.v1.json requires `agents` to
 * be non-empty (`minItems: 1`) and `session_root_agent` to name a real
 * agent. A brand-new framework has neither until the user drags the
 * first Agent node onto the canvas (Stage D1). The generated `Framework`
 * type models the *valid* shape — its `agents` field is a non-empty
 * tuple — so the empty cold-start needs one boundary cast here; D2's
 * continuous validation surfaces the still-missing pieces as the user
 * builds.
 */
export function emptyFramework(): Framework {
  return {
    name: 'untitled',
    version: '0.1.0',
    description: 'New framework.',
    model: { provider: 'anthropic', id: 'claude-sonnet-4-6' },
    tools: [],
    skills: [],
    agents: [],
    session_root_agent: '',
  } as unknown as Framework;
}

/**
 * A builder-authored agent entry. The Builder writes a *minimal* agent
 * into `framework.agents` as the user composes it on the canvas; the
 * generated `Agent` type models the complete valid shape (capabilities,
 * spawn_constraints, …) the framework only reaches once fully built —
 * D2's continuous validation surfaces the gap. The one boundary cast
 * here is the same pre-valid stance `emptyFramework()` documents.
 */
function builderAgent(id: string): Agent {
  return {
    id,
    role: '',
    model: { provider: 'anthropic', id: DEFAULT_MODEL_ID },
    allowed_tools: [],
    allowed_skills: [],
    spawns: [],
  } as unknown as Agent;
}

/** True when an `agents[]` entry is an inline `Agent` (D1 only creates
 *  inline agents; a `{ id, path }` $ref entry arrives only via a
 *  loaded framework — E's concern). */
function isInlineAgent(entry: Framework['agents'][number]): entry is Agent {
  return 'role' in entry;
}

/**
 * Split a canvas node id `${kind}:${ref}` into its kind prefix and ref.
 * The canvas node-id scheme (D1's `projectCanvasNodes`) prefixes every
 * id with its kind; `connectEdge` reads the kind off the prefix to map
 * a connection to one of the four spec edge types. Exported so Stage E
 * / M09 reuse one node-id parser rather than re-splitting ad hoc.
 */
export function parseNodeId(id: string): { kind: string; ref: string } {
  const sep = id.indexOf(':');
  if (sep === -1) {
    return { kind: id, ref: '' };
  }
  return { kind: id.slice(0, sep), ref: id.slice(sep + 1) };
}

/** Apply a Palette-item drop to the framework document. Each kind lands
 *  in its schema home so the projection can derive the node and Stage E
 *  can save/load it; a D2 edge later wires Tool/Skill into an agent's
 *  `allowed_*`. The `nodeId in nodePositions` guard in `addNode`
 *  already makes this idempotent — a key is never clobbered here. */
function applyDrop(framework: Framework, kind: BuilderNodeKind, ref: string): Framework {
  switch (kind) {
    case 'agent':
      return {
        ...framework,
        agents: [...framework.agents, builderAgent(ref)] as Framework['agents'],
      };
    case 'tool':
      return {
        ...framework,
        tools: [
          ...framework.tools,
          { name: ref, source: BUILTIN_TOOL_NAMES.includes(ref) ? 'builtin' : 'external' },
        ],
      };
    case 'skill':
      return {
        ...framework,
        skills: [...framework.skills, { name: ref, source: 'external' }],
      };
    case 'hitl':
      return {
        ...framework,
        hitl_policy: { ...framework.hitl_policy, [ref]: { enabled: true } },
      };
    case 'hook':
      return {
        ...framework,
        hooks: { ...framework.hooks, [ref]: [] },
      };
  }
}

/** The framework → React-Flow node projection (ADR-0020). Node
 *  existence is derived from `framework` (so Stage E's load
 *  reconstructs the canvas); positions come from the editor-local
 *  `nodePositions`. Every node carries `nodePath` — the key the
 *  validate_framework report attributes a `NodeError` to (a bare agent
 *  id for agents; the artifact name for tools / skills) — so D2's red
 *  badge can attribute an error to its node. */
function projectCanvasNodes(framework: Framework, nodePositions: Record<string, Position>): Node[] {
  const at = (id: string): Position => nodePositions[id] ?? { x: 0, y: 0 };
  const nodes: Node[] = [];
  for (const entry of framework.agents) {
    const id = `agent:${entry.id}`;
    const agent = isInlineAgent(entry) ? entry : undefined;
    nodes.push({
      id,
      type: 'agent',
      position: at(id),
      data: {
        agentId: entry.id,
        nodePath: entry.id,
        role: agent?.role ?? '',
        model: agent?.model.id ?? '',
        allowedTools: agent?.allowed_tools ?? [],
        allowedSkills: agent?.allowed_skills ?? [],
      },
    });
  }
  for (const tool of framework.tools) {
    const id = `tool:${tool.name}`;
    nodes.push({
      id,
      type: 'tool',
      position: at(id),
      data: { name: tool.name, nodePath: tool.name },
    });
  }
  for (const skill of framework.skills) {
    const id = `skill:${skill.name}`;
    nodes.push({
      id,
      type: 'skill',
      position: at(id),
      data: { name: skill.name, nodePath: skill.name },
    });
  }
  for (const trigger of Object.keys(framework.hitl_policy ?? {})) {
    const id = `hitl:${trigger}`;
    nodes.push({ id, type: 'hitl', position: at(id), data: { trigger, nodePath: trigger } });
  }
  for (const point of Object.keys(framework.hooks ?? {})) {
    const id = `hook:${point}`;
    nodes.push({ id, type: 'hook', position: at(id), data: { point, nodePath: point } });
  }
  return nodes;
}

// canvasNodes() runs inside a React render selector — useSyncExternalStore
// invokes the selector repeatedly per render for its consistency check,
// so it MUST return a referentially stable array when its inputs are
// unchanged or React infinite-loops ("Maximum update depth exceeded";
// gotcha #75). useShallow alone cannot break the loop because the
// projected element objects are fresh each call. Memoize on the
// `framework` + `nodePositions` identities — both change only via set(),
// so reference equality is a sound cache key.
let projectionCache: {
  framework: Framework;
  nodePositions: Record<string, Position>;
  nodes: Node[];
} | null = null;

function memoizedCanvasNodes(
  framework: Framework,
  nodePositions: Record<string, Position>,
): Node[] {
  if (
    projectionCache !== null &&
    projectionCache.framework === framework &&
    projectionCache.nodePositions === nodePositions
  ) {
    return projectionCache.nodes;
  }
  const nodes = projectCanvasNodes(framework, nodePositions);
  projectionCache = { framework, nodePositions, nodes };
  return nodes;
}

/** The framework → React-Flow edge projection (ADR-0020). The four edge
 *  types live as `allowed_skills` / `allowed_tools` / `spawns` arrays on
 *  the framework's agents; each becomes a wire. An edge is projected
 *  only when BOTH endpoints exist as canvas nodes — an `allowed_*` entry
 *  set via the inline config with no node on the canvas would otherwise
 *  point a React Flow edge at nothing. */
function projectCanvasEdges(framework: Framework, nodeIds: Set<string>): Edge[] {
  const edges: Edge[] = [];
  const add = (source: string, target: string): void => {
    if (nodeIds.has(source) && nodeIds.has(target)) {
      edges.push({ id: `${source}->${target}`, source, target, className: 'builder-edge' });
    }
  };
  for (const entry of framework.agents) {
    if (!isInlineAgent(entry)) {
      continue;
    }
    const source = `agent:${entry.id}`;
    for (const skill of entry.allowed_skills) {
      add(source, `skill:${skill}`);
    }
    for (const tool of entry.allowed_tools) {
      add(source, `tool:${tool}`);
    }
    for (const child of entry.spawns) {
      add(source, `agent:${child}`);
    }
  }
  return edges;
}

// The edge projection is a pure function of `framework` (the node-id set
// it filters against is itself derived from `framework`), so it memoizes
// on the `framework` identity alone — returning a referentially stable
// array so the canvas's useShallow selector does not re-render per
// commit (gotcha #75, the canvasNodes precedent).
let edgeProjectionCache: { framework: Framework; edges: Edge[] } | null = null;

function memoizedCanvasEdges(framework: Framework, nodeIds: Set<string>): Edge[] {
  if (edgeProjectionCache !== null && edgeProjectionCache.framework === framework) {
    return edgeProjectionCache.edges;
  }
  const edges = projectCanvasEdges(framework, nodeIds);
  edgeProjectionCache = { framework, edges };
  return edges;
}

/** Push `value` onto one inline agent's `allowed_skills` / `allowed_tools`
 *  / `spawns` list. Idempotent — `connectEdge` re-connecting an
 *  already-recorded edge does not append a duplicate. Returns the state
 *  unchanged when the agent is absent or the value is already present. */
function pushAgentList(
  state: BuilderState,
  agentId: string,
  field: 'allowed_skills' | 'allowed_tools' | 'spawns',
  value: string,
): BuilderState {
  let mutated = false;
  const agents = state.framework.agents.map((entry) => {
    if (!isInlineAgent(entry) || entry.id !== agentId) {
      return entry;
    }
    if (entry[field].includes(value)) {
      return entry; // idempotent — no duplicate edge
    }
    mutated = true;
    return { ...entry, [field]: [...entry[field], value] };
  }) as Framework['agents'];
  if (!mutated) {
    return state;
  }
  return { ...state, framework: { ...state.framework, agents } };
}

/** Push a hook reference onto `task_defaults.post_hooks` (the Hook→Task
 *  edge). The schema's `{ $ref }` post-hook variant carries the hook
 *  node's ref. Idempotent. `task_defaults` is created if absent. */
function pushPostHook(state: BuilderState, hookRef: string): BuilderState {
  const existing = state.framework.task_defaults?.post_hooks ?? [];
  if (existing.some((hook) => '$ref' in hook && hook.$ref === hookRef)) {
    return state; // idempotent — no duplicate post-hook
  }
  return {
    ...state,
    framework: {
      ...state.framework,
      task_defaults: {
        ...state.framework.task_defaults,
        post_hooks: [...existing, { $ref: hookRef }],
      },
    },
  };
}

/**
 * Map a connection `(sourceId, targetId)` to one of the four spec
 * Phase 9 edge types and apply the matching `framework` mutation, or
 * reject every other node-pair. The kind of each endpoint is read off
 * its node-id prefix (`parseNodeId`):
 *
 * - Agent→Skill = an `allowed_skills` entry on the source agent
 * - Agent→Tool  = an `allowed_tools` entry on the source agent
 * - Agent→Agent = a `spawns` entry on the source (parent) agent
 * - Hook→Task   = a `{ $ref }` entry on `task_defaults.post_hooks`
 *
 * A rejected pair returns the state unchanged: no `framework` mutation,
 * so the canvas edge projection — a pure function of `framework` — paints
 * no wire. The Agent→Agent capability narrowing is NOT computed here;
 * drawing the edge re-runs `validate_framework`, whose Rust report
 * carries the intersection (spec §9 — never re-implemented in TS).
 */
function connectEdgeReducer(state: BuilderState, sourceId: string, targetId: string): BuilderState {
  const source = parseNodeId(sourceId);
  const target = parseNodeId(targetId);
  switch (`${source.kind}->${target.kind}`) {
    case 'agent->skill':
      return pushAgentList(state, source.ref, 'allowed_skills', target.ref);
    case 'agent->tool':
      return pushAgentList(state, source.ref, 'allowed_tools', target.ref);
    case 'agent->agent':
      return pushAgentList(state, source.ref, 'spawns', target.ref);
    case 'hook->task':
      return pushPostHook(state, source.ref);
    default:
      // Not one of the four spec edge types — reject. No framework
      // mutation; the projection paints no edge.
      return state;
  }
}

// Module-scoped debounce handle — one in-flight scheduled validation per
// store, coalescing a burst of framework mutations into one call.
let validateTimer: ReturnType<typeof setTimeout> | null = null;

/**
 * The Tester's SCOPED graph store (M08.F2 — spec Phase 9; ADR-0019). A
 * SECOND, independent graph-store instance, distinct from the live
 * `useGraphStore` module singleton. F2's Tester modal reduces the test
 * session's `AgentEvent` trace into THIS store, so a test run renders in
 * the modal's smaller graph pane WITHOUT ever mutating the runtime
 * graph. `closeTester` clears it — discard-on-close.
 */
export const useTestGraphStore = createGraphStore();

export const useBuilderStore = create<BuilderState>((set, get) => {
  /**
   * The debounced continuous-validation trigger (D2.3.4). Every
   * framework mutation schedules one `validate_framework` call after a
   * quiet interval; a burst of edits coalesces to a single backend
   * pass. The Validate button (Stage E) calls the SAME validator with
   * no debounce — one Rust validator, two triggers (spec §9).
   */
  function scheduleValidation(): void {
    if (validateTimer !== null) {
      clearTimeout(validateTimer);
    }
    validateTimer = setTimeout(() => {
      validateTimer = null;
      void validateFramework(get().framework)
        .then((report) => set({ validation: report }))
        .catch((error: unknown) => {
          // A failed pass leaves the prior report in place — no flicker
          // to "no errors". gotcha #30 — log the structured error
          // rather than String()-ing it; Stage E's Inspector surfaces
          // validation failures to the user.
          console.error('[builder] validate_framework failed:', unwrapCmdError(error));
        });
    }, VALIDATION_DEBOUNCE_MS);
  }

  return {
    framework: emptyFramework(),
    diskFramework: null,
    selectedNodeId: null,
    validation: null,
    testerOpen: false,
    nodePositions: {},
    replaceFramework: (fw) => set({ framework: fw }),
    // M08.6.D: swap the document AND seed `nodePositions` with the
    // dagre top-down layout (`layoutGraph` from src/lib/layout.ts —
    // the same engine the live `GraphCanvas` uses). The canvas
    // projection (`projectCanvasNodes`/`projectCanvasEdges`) is the
    // input to the layout: project once with an empty `nodePositions`
    // map (the {0,0} fallback), feed dagre, then write the resulting
    // positions into `nodePositions`. The next `canvasNodes()` read
    // re-projects against the seeded map and the React Flow canvas
    // renders the laid-out graph. Auto-layout fires here only —
    // `replaceFramework` (the JSON-tab edit path) deliberately does
    // not lay out, so a user's manual drags (the editor-local view
    // state per ADR-0020) survive a JSON tweak.
    applyLoadedFramework: (fw) =>
      set(() => {
        const nodes = projectCanvasNodes(fw, {});
        const nodeIds = new Set(nodes.map((node) => node.id));
        const edges = projectCanvasEdges(fw, nodeIds);
        const laidOut = layoutGraph(nodes, edges);
        const nodePositions: Record<string, Position> = {};
        for (const node of laidOut) {
          nodePositions[node.id] = node.position;
        }
        return { framework: fw, nodePositions };
      }),
    setDiskFramework: (fw) => set({ diskFramework: fw }),
    selectNode: (id) => set({ selectedNodeId: id }),
    addNode: (kind, ref, position) => {
      set((s) => {
        const nodeId = `${kind}:${ref}`;
        if (nodeId in s.nodePositions) {
          return s; // idempotent on a re-drop of the same Palette item
        }
        return {
          framework: applyDrop(s.framework, kind, ref),
          nodePositions: { ...s.nodePositions, [nodeId]: position },
        };
      });
      scheduleValidation();
    },
    updateNode: (nodeId, patch) => {
      set((s) => {
        const agentId = nodeId.replace(/^agent:/, '');
        const agents = s.framework.agents.map((entry) =>
          entry.id === agentId ? { ...entry, ...patch } : entry,
        ) as Framework['agents'];
        return { framework: { ...s.framework, agents } };
      });
      scheduleValidation();
    },
    connectEdge: (sourceId, targetId) => {
      set((s) => connectEdgeReducer(s, sourceId, targetId));
      scheduleValidation();
    },
    // removeNode stays a typed no-op stub — node deletion is not in D2's
    // scope (no D2 deliverable, no D2 test); a later stage fills it.
    removeNode: () => set((s) => s),
    setValidation: (report) => set({ validation: report }),
    // E ships openTester / closeTester INERT-but-wired — the Inspector's
    // Test button calls openTester; Stage F2's Tester modal renders on
    // `testerOpen`. The M07.E "wired-but-pending" incremental-
    // construction precedent, not dead code.
    openTester: () => set({ testerOpen: true }),
    closeTester: () => {
      // Discard-on-close (spec Phase 9; ADR-0019): drop the scoped
      // test-session graph so a re-open starts fresh. F1's backend
      // already deleted the throwaway test DB; the modal drops its
      // TestOutcome in its own close handler.
      useTestGraphStore.getState().clear();
      set({ testerOpen: false });
    },
    canvasNodes: () => memoizedCanvasNodes(get().framework, get().nodePositions),
    canvasEdges: () => {
      const framework = get().framework;
      const nodeIds = new Set(
        memoizedCanvasNodes(framework, get().nodePositions).map((node) => node.id),
      );
      return memoizedCanvasEdges(framework, nodeIds);
    },
    moveNode: (nodeId, position) =>
      set((s) => ({ nodePositions: { ...s.nodePositions, [nodeId]: position } })),
  };
});
