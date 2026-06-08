import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// M09.D — a canvas-authored framework is RUNNABLE: creating the first agent
// roots the session on it. This is the composition glue the vertical-slice
// IRL surfaces (docs/build-prompts/M09-workbench-vertical-slice.md Stage D):
// the run path reads `framework.session_root_agent` to pick the dispatch
// agent (agent_sdk.rs:780), but `applyDrop` (builderStore.ts) appended the
// authored agent WITHOUT ever setting `session_root_agent` — it stayed the
// `emptyFramework()` empty string. So a canvas-authored framework could not
// run as a hand-written-JSON framework does (every fixture names
// session_root_agent). M09.D auto-roots the FIRST authored agent so the
// composed authored->serialize->run path matches hand-written JSON.
//
// Declaration-only / store-side: this writes the document's root pointer,
// never the enforcer or the run loop (CLAUDE.md §4 rule 11 — the enforced
// write is the M09.D real-app IRL).
//
// validate_framework is mocked (the connectEdge/fileAccess suite precedent)
// so the debounced scheduleValidation never reaches the real Tauri bridge.

const { validateFrameworkMock } = vi.hoisted(() => ({ validateFrameworkMock: vi.fn() }));
vi.mock('../../../src/lib/ipc', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../../src/lib/ipc')>();
  return { ...actual, validateFramework: validateFrameworkMock };
});

import { emptyFramework, useBuilderStore } from '../../../src/lib/builderStore';
import type { FrameworkValidationReport } from '../../../src/lib/ipc';

/** A clean, passing validation report — the debounce-trigger default. */
function okReport(): FrameworkValidationReport {
  return { schema_errors: [], capability_errors: [], ok: true, capability_summary: null };
}

function currentFramework() {
  return useBuilderStore.getState().framework;
}

describe('builderStore — a canvas-authored framework is runnable (M09.D session root)', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    validateFrameworkMock.mockReset().mockResolvedValue(okReport());
    useBuilderStore.setState(useBuilderStore.getInitialState(), true);
    useBuilderStore.setState({ framework: emptyFramework() });
  });

  afterEach(() => {
    vi.runOnlyPendingTimers();
    vi.useRealTimers();
  });

  it('rooting_the_session_on_the_first_authored_agent', () => {
    // emptyFramework() opens with session_root_agent: '' — unrunnable. The
    // run path picks the dispatch agent off session_root_agent, so the first
    // create must root the session for the authored framework to run.
    expect(currentFramework().session_root_agent).toBe('');
    useBuilderStore.getState().addNode('agent', 'agent-1', { x: 0, y: 0 });
    expect(currentFramework().session_root_agent).toBe('agent-1');
  });

  it('a_second_authored_agent_does_not_re_root_the_session', () => {
    const store = useBuilderStore.getState();
    store.addNode('agent', 'agent-1', { x: 0, y: 0 });
    store.addNode('agent', 'agent-2', { x: 240, y: 0 });
    // The single-agent vertical slice roots on the FIRST agent; a later
    // create never steals the root (the explicit multi-agent root affordance
    // is a later ADR-0032 slice — sub-agents at M11).
    expect(currentFramework().session_root_agent).toBe('agent-1');
    expect(currentFramework().agents.map((a) => ('id' in a ? a.id : ''))).toEqual([
      'agent-1',
      'agent-2',
    ]);
  });

  it('a_loaded_frameworks_existing_root_is_never_clobbered', () => {
    // A loaded framework already names its root; authoring a new agent into
    // it must not hijack the session root.
    useBuilderStore.setState({
      framework: { ...emptyFramework(), session_root_agent: 'orchestrator' },
    });
    useBuilderStore.getState().addNode('agent', 'agent-1', { x: 0, y: 0 });
    expect(currentFramework().session_root_agent).toBe('orchestrator');
  });
});
