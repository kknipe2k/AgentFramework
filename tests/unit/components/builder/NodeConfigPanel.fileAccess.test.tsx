import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { NodeConfigPanel } from '../../../../src/components/builder/NodeConfigPanel';
import { emptyFramework, useBuilderStore } from '../../../../src/lib/builderStore';
import type { Framework } from '../../../../src/types/framework';

// M09.B — the File-access editor on the inline node-config surface. The
// enforced read/write scope lives at capabilities.file_access.{read,write}
// (glob lists — common.v1.json:56-64); the L2 enforcer denies a Write whose
// target is outside file_access.write (E-02 capability_live_tool.rs). Pre-M09
// NodeConfigPanel edited only role/model/allowed_* (NodeConfigPanel.tsx:105-146)
// — no capability surface — so a canvas agent's Write could never be granted.
// M09.B adds two glob lists (Read, Write) reusing the AllowedList component;
// each edit recomputes capabilities immutably and flows through updateNode as a
// { capabilities: nextCaps } patch (updateNode merges {...entry,...patch} —
// builderStore.ts:522). Declaration-only — the enforced write lands at M09.D.

function frameworkWithCapableAgent(read: string[], write: string[]): Framework {
  return {
    ...emptyFramework(),
    agents: [
      {
        id: 'planner',
        role: 'Lead',
        model: { provider: 'anthropic', id: 'claude-sonnet-4-6' },
        allowed_tools: [],
        allowed_skills: [],
        spawns: [],
        capabilities: {
          tools_called: [],
          skills_loaded: [],
          file_access: { read, write },
          network: [],
          shell: false,
          spawn_agents: [],
        },
      },
    ],
  } as unknown as Framework;
}

/** The full Capabilities the editor must emit for the given file_access — the
 *  other required fields are carried through untouched (declaration-only). */
function capsWith(read: string[], write: string[]): unknown {
  return {
    tools_called: [],
    skills_loaded: [],
    file_access: { read, write },
    network: [],
    shell: false,
    spawn_agents: [],
  };
}

describe('NodeConfigPanel — File access editor (M09.B)', () => {
  beforeEach(() => {
    useBuilderStore.setState(useBuilderStore.getInitialState(), true);
  });

  it('renders_read_and_write_glob_lists_reflecting_file_access', () => {
    useBuilderStore.setState({
      framework: frameworkWithCapableAgent(['data/**'], ['out/**']),
      selectedNodeId: 'agent:planner',
    });
    render(<NodeConfigPanel />);
    expect(screen.getByTestId('node-config-fa-read')).toHaveTextContent('data/**');
    expect(screen.getByTestId('node-config-fa-write')).toHaveTextContent('out/**');
  });

  it('adding_a_write_glob_calls_updateNode_with_a_capabilities_patch', async () => {
    const updateNode = vi.fn();
    useBuilderStore.setState({
      framework: frameworkWithCapableAgent([], []),
      selectedNodeId: 'agent:planner',
      updateNode,
    });
    render(<NodeConfigPanel />);
    await userEvent.type(screen.getByTestId('node-config-add-fa-write-input'), 'out/**');
    await userEvent.click(screen.getByTestId('node-config-add-fa-write'));
    // The new write glob lands; read stays []; the rest of Capabilities is
    // carried through — the granted scope the L2 enforcer consumes at run time.
    expect(updateNode).toHaveBeenCalledWith('agent:planner', {
      capabilities: capsWith([], ['out/**']),
    });
  });

  it('adding_a_read_glob_preserves_the_existing_write_scope', async () => {
    const updateNode = vi.fn();
    useBuilderStore.setState({
      framework: frameworkWithCapableAgent([], ['out/**']),
      selectedNodeId: 'agent:planner',
      updateNode,
    });
    render(<NodeConfigPanel />);
    await userEvent.type(screen.getByTestId('node-config-add-fa-read-input'), 'data/**');
    await userEvent.click(screen.getByTestId('node-config-add-fa-read'));
    expect(updateNode).toHaveBeenCalledWith('agent:planner', {
      capabilities: capsWith(['data/**'], ['out/**']),
    });
  });

  it('removing_a_write_glob_calls_updateNode_with_the_pruned_scope', async () => {
    const updateNode = vi.fn();
    useBuilderStore.setState({
      framework: frameworkWithCapableAgent([], ['out/**']),
      selectedNodeId: 'agent:planner',
      updateNode,
    });
    render(<NodeConfigPanel />);
    await userEvent.click(screen.getByTestId('node-config-fa-write-remove-out/**'));
    expect(updateNode).toHaveBeenCalledWith('agent:planner', { capabilities: capsWith([], []) });
  });
});
