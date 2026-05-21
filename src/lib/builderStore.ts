import { create } from 'zustand';
import type { Framework } from '../types/framework';
import type { FrameworkValidationReport } from './ipc';

// M08.C — the Builder store (ADR-0020). builderStore holds the
// in-progress framework.json as the single source of truth; the canvas
// (D1/D2) is a projection derived from `framework`, and canvas edits
// mutate `framework`. It is a SEPARATE Zustand store from `graphStore`
// (the live-execution store) — the two have disjoint lifecycles
// (build-time vs run-time) and conflating them is the dual-purpose-store
// anti-pattern. C ships the store shape + replaceFramework /
// setDiskFramework / selectNode / setValidation; the canvas-mutation
// actions (addNode / updateNode / connectEdge / removeNode) ship as
// typed no-op stubs that D1/D2 fill — shipping them keeps the store
// interface final at C so no later stage re-shapes a useBuilderStore
// selector.

/** One Palette item dragged onto the canvas (D1 consumes these). */
export type BuilderNodeKind = 'agent' | 'tool' | 'skill' | 'hitl' | 'hook';

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
  addNode: (kind: BuilderNodeKind, ref: string, position: { x: number; y: number }) => void;
  /** D1: inline-config edit (role / model / allowed_*). */
  updateNode: (nodeId: string, patch: Record<string, unknown>) => void;
  /** D2: the four edge types -> the right `framework` field. */
  connectEdge: (sourceId: string, targetId: string) => void;
  /** D1/D2: drop a node + its edges. */
  removeNode: (nodeId: string) => void;
  /** Store a fresh validation report (D2 continuous + E explicit). */
  setValidation: (report: FrameworkValidationReport) => void;
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

export const useBuilderStore = create<BuilderState>((set) => ({
  framework: emptyFramework(),
  diskFramework: null,
  selectedNodeId: null,
  validation: null,
  replaceFramework: (fw) => set({ framework: fw }),
  setDiskFramework: (fw) => set({ diskFramework: fw }),
  selectNode: (id) => set({ selectedNodeId: id }),
  // D1/D2 replace these no-op bodies; shipping them typed keeps the
  // store shape final at C.
  addNode: () => set((s) => s),
  updateNode: () => set((s) => s),
  connectEdge: () => set((s) => s),
  removeNode: () => set((s) => s),
  setValidation: (report) => set({ validation: report }),
}));
