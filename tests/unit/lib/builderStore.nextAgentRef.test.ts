import { describe, expect, it } from 'vitest';
import { emptyFramework, nextAgentRef } from '../../../src/lib/builderStore';
import type { Framework } from '../../../src/types/framework';

// M09.A — the pure id helper behind the "+ New agent" Palette affordance.
// A fresh project opens with emptyFramework() (agents: []); the store
// already mints an agent on a Palette drop, but the Palette never offered
// a blank item. nextAgentRef generates the fresh `agent-N` id that item
// carries — the first `agent-N` not already in framework.agents, so a
// re-create never collides with addNode's `${kind}:${ref}` idempotence
// guard (builderStore.ts). The id must satisfy agent.v1.json's pattern
// `^[a-z][a-z0-9-]*$` so a created agent is id-valid the moment it lands.

/** A pre-valid framework carrying agents with the given ids (the rest of
 *  the minimal Agent shape mirrors builderAgent — see builderStore.ts). */
function frameworkWithAgentIds(ids: string[]): Framework {
  return {
    ...emptyFramework(),
    agents: ids.map((id) => ({
      id,
      role: 'r',
      model: { provider: 'anthropic', id: 'claude-sonnet-4-6' },
      allowed_tools: [],
      allowed_skills: [],
      spawns: [],
    })) as unknown as Framework['agents'],
  };
}

describe('nextAgentRef', () => {
  it('returns_agent_1_for_an_empty_framework', () => {
    // A brand-new project — the very first blank-created agent.
    expect(nextAgentRef(emptyFramework())).toBe('agent-1');
  });

  it('skips_an_existing_agent_1', () => {
    // After one create, the next New-agent item must carry a distinct id
    // so the second drop is not swallowed by addNode's idempotence guard.
    expect(nextAgentRef(frameworkWithAgentIds(['agent-1']))).toBe('agent-2');
  });

  it('fills_the_first_free_gap', () => {
    // Holes (e.g. agent-2 deleted, agent-1 + agent-3 remain) are reused
    // before extending past the highest id.
    expect(nextAgentRef(frameworkWithAgentIds(['agent-1', 'agent-3']))).toBe('agent-2');
  });

  it('ignores_non_agent_N_ids', () => {
    // A loaded framework's named agents (orchestrator, planner) never
    // shadow the agent-N sequence — the first New agent is still agent-1.
    expect(nextAgentRef(frameworkWithAgentIds(['planner', 'researcher']))).toBe('agent-1');
  });

  it('matches_the_agent_v1_json_id_pattern', () => {
    const ref = nextAgentRef(frameworkWithAgentIds(['agent-1', 'agent-2']));
    expect(ref).toBe('agent-3');
    // agent.v1.json:14 — the id pattern the schema enforces.
    expect(ref).toMatch(/^[a-z][a-z0-9-]*$/);
  });
});
