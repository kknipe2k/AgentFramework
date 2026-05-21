import type { Edge, Node } from '@xyflow/react';
import { create } from 'zustand';
import type { Agent, Framework } from '../types/framework';
import type { FrameworkValidationReport } from './ipc';

// M08.C/D1 — the Builder store (ADR-0020). builderStore holds the
// in-progress framework.json as the single source of truth; the canvas
// is a projection derived from `framework`, and canvas edits mutate
// `framework`. It is a SEPARATE Zustand store from `graphStore` (the
// live-execution store) — the two have disjoint lifecycles (build-time
// vs run-time) and conflating them is the dual-purpose-store
// anti-pattern. C shipped the store shape + replaceFramework /
// setDiskFramework / selectNode / setValidation; D1 implements the
// canvas-mutation actions addNode / updateNode / moveNode + the
// framework→canvas projection (canvasNodes / canvasEdges). connectEdge
// / removeNode stay typed no-op stubs D2 fills.

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

/** The Builder store contract — SEPARATE from `graphStore` (ADR-0020). */
export interface BuilderState {
  /** THE source of truth — the in-progress framework.json. */
  framework: Framework;
  /** Last saved/loaded snapshot — Stage E diffs `framework` against it. */
  diskFramework: Framework | null;
  /** Selected canvas node id — drives D1's inline config + E's Inspector. */
  selectedNodeId: string | null;
  /** Latest validate_framework report (D2 populates; null until first run). */
  validation: FrameworkValidationReport | null;

  /** Replace the whole document (E's JSON-tab edit; load_framework). */
  replaceFramework: (fw: Framework) => void;
  /** Record the on-disk snapshot after a save/load (E's diff baseline). */
  setDiskFramework: (fw: Framework | null) => void;
  /** Select / clear the active canvas node. */
  selectNode: (id: string | null) => void;
  /** D1: instantiate a node from a dropped Palette item. */
  addNode: (kind: BuilderNodeKind, ref: string, position: Position) => void;
  /** D1: inline-config edit (role / model / allowed_*). */
  updateNode: (nodeId: string, patch: Record<string, unknown>) => void;
  /** D2: the four edge types -> the right `framework` field. */
  connectEdge: (sourceId: string, targetId: string) => void;
  /** D1/D2: drop a node + its edges. */
  removeNode: (nodeId: string) => void;
  /** Store a fresh validation report (D2 continuous + E explicit). */
  setValidation: (report: FrameworkValidationReport) => void;

  /** D1: user-placed canvas coordinates, keyed by `${kind}:${ref}` node
   *  id — editor-local layout view state (ADR-0020: NOT part of the
   *  framework document, not persisted by save_framework). */
  nodePositions: Record<string, Position>;
  /** D1: derive the React-Flow node array from `framework` +
   *  `nodePositions` — the ADR-0020 framework→canvas projection. */
  canvasNodes: () => Node[];
  /** D1: derive the React-Flow edge array — empty in D1; D2 fills it. */
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
 *  `nodePositions`. D1 projects all five node kinds; D2 adds the edges
 *  that wire Tool/Skill nodes into an agent's `allowed_*`. */
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
        role: agent?.role ?? '',
        model: agent?.model.id ?? '',
        allowedTools: agent?.allowed_tools ?? [],
        allowedSkills: agent?.allowed_skills ?? [],
      },
    });
  }
  for (const tool of framework.tools) {
    const id = `tool:${tool.name}`;
    nodes.push({ id, type: 'tool', position: at(id), data: { name: tool.name } });
  }
  for (const skill of framework.skills) {
    const id = `skill:${skill.name}`;
    nodes.push({ id, type: 'skill', position: at(id), data: { name: skill.name } });
  }
  for (const trigger of Object.keys(framework.hitl_policy ?? {})) {
    const id = `hitl:${trigger}`;
    nodes.push({ id, type: 'hitl', position: at(id), data: { trigger } });
  }
  for (const point of Object.keys(framework.hooks ?? {})) {
    const id = `hook:${point}`;
    nodes.push({ id, type: 'hook', position: at(id), data: { point } });
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

/** Edges are D2 — D1 ships a stable empty array (a fresh `[]` each call
 *  would defeat the canvas's `useShallow` selector; gotcha #75). */
const EMPTY_EDGES: Edge[] = [];

export const useBuilderStore = create<BuilderState>((set, get) => ({
  framework: emptyFramework(),
  diskFramework: null,
  selectedNodeId: null,
  validation: null,
  nodePositions: {},
  replaceFramework: (fw) => set({ framework: fw }),
  setDiskFramework: (fw) => set({ diskFramework: fw }),
  selectNode: (id) => set({ selectedNodeId: id }),
  addNode: (kind, ref, position) =>
    set((s) => {
      const nodeId = `${kind}:${ref}`;
      if (nodeId in s.nodePositions) {
        return s; // idempotent on a re-drop of the same Palette item
      }
      return {
        framework: applyDrop(s.framework, kind, ref),
        nodePositions: { ...s.nodePositions, [nodeId]: position },
      };
    }),
  updateNode: (nodeId, patch) =>
    set((s) => {
      const agentId = nodeId.replace(/^agent:/, '');
      const agents = s.framework.agents.map((entry) =>
        entry.id === agentId ? { ...entry, ...patch } : entry,
      ) as Framework['agents'];
      return { framework: { ...s.framework, agents } };
    }),
  // D2 replaces these no-op bodies (edges + node deletion).
  connectEdge: () => set((s) => s),
  removeNode: () => set((s) => s),
  setValidation: (report) => set({ validation: report }),
  canvasNodes: () => memoizedCanvasNodes(get().framework, get().nodePositions),
  canvasEdges: () => EMPTY_EDGES,
  moveNode: (nodeId, position) =>
    set((s) => ({ nodePositions: { ...s.nodePositions, [nodeId]: position } })),
}));
