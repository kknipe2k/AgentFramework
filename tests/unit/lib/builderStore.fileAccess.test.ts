import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// M09.B — a created agent is granted a valid Capabilities. builderAgent
// (builderStore.ts:145) omitted `capabilities`, which agent.v1.json:9 marks
// REQUIRED (common.v1.json#/$defs/Capabilities) — so every authored agent was
// schema-invalid AND, with no file_access.write, had every Write denied at the
// L2 enforcer (E-02 capability_live_tool.rs). M09.B mints a minimal-valid
// Capabilities on drop (all-empty, file_access {read:[],write:[]}); the
// NodeConfigPanel File-access editor then writes file_access.{read,write}
// through updateNode. Declaration-only — the enforcer (unchanged) consumes the
// grant at run time; the *enforced* write lands at M09.D (CLAUDE.md §4 rule 11).
//
// validate_framework is the Stage B Rust command — mocked here (partial module
// mock) so the debounced scheduleValidation never reaches the real Tauri
// bridge. Fake timers drop the dangling debounce timer the connectEdge suite
// precedent (builderStore.connectEdge.test.ts) established.

const { validateFrameworkMock } = vi.hoisted(() => ({ validateFrameworkMock: vi.fn() }));
vi.mock('../../../src/lib/ipc', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../../src/lib/ipc')>();
  return { ...actual, validateFramework: validateFrameworkMock };
});

import { emptyFramework, useBuilderStore } from '../../../src/lib/builderStore';
import type { FrameworkValidationReport } from '../../../src/lib/ipc';
import type { Agent } from '../../../src/types/framework';

/** A clean, passing validation report — the debounce-trigger default. */
function okReport(): FrameworkValidationReport {
  return { schema_errors: [], capability_errors: [], ok: true, capability_summary: null };
}

/** The sole inline agent these tests author on. */
function soleAgent(): Agent {
  return useBuilderStore.getState().framework.agents[0] as Agent;
}

describe('builderStore — a created agent is granted a valid Capabilities (M09.B)', () => {
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

  it('a_dropped_agent_carries_a_minimal_valid_capabilities', () => {
    useBuilderStore.getState().addNode('agent', 'agent-1', { x: 0, y: 0 });
    // capabilities is REQUIRED by agent.v1.json:9 — builderAgent must mint the
    // full Capabilities shape (common.v1.json#/$defs/Capabilities) so a created
    // agent is schema-valid the moment it has a role.
    expect(soleAgent().capabilities).toEqual({
      tools_called: [],
      skills_loaded: [],
      file_access: { read: [], write: [] },
      network: [],
      shell: false,
      spawn_agents: [],
    });
  });

  it('granting_file_access_preserves_the_other_required_capability_fields', () => {
    const store = useBuilderStore.getState();
    store.addNode('agent', 'agent-1', { x: 0, y: 0 });
    // The File-access editor recomputes capabilities immutably off the agent's
    // current grant: { ...caps, file_access: { ...caps.file_access, write } }.
    // The other required fields (tools_called/skills_loaded/network/shell/
    // spawn_agents) survive ONLY because builderAgent minted a full
    // Capabilities — the M09.B invariant this pins.
    const caps = soleAgent().capabilities;
    store.updateNode('agent:agent-1', {
      capabilities: { ...caps, file_access: { ...caps.file_access, write: ['out/**'] } },
    });
    expect(soleAgent().capabilities).toEqual({
      tools_called: [],
      skills_loaded: [],
      file_access: { read: [], write: ['out/**'] },
      network: [],
      shell: false,
      spawn_agents: [],
    });
  });
});
