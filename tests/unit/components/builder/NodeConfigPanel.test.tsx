import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { NodeConfigPanel } from '../../../../src/components/builder/NodeConfigPanel';
import { emptyFramework, useBuilderStore } from '../../../../src/lib/builderStore';
import type { Framework } from '../../../../src/types/framework';

// M08.D1 — the inline node-configuration surface (spec Phase 9
// "right-click for properties"). For an Agent: role / model / the
// allowed_tools + allowed_skills editable lists; every edit flows
// through builderStore.updateNode -> framework mutation.

function frameworkWithAgent(): Framework {
  return {
    ...emptyFramework(),
    agents: [
      {
        id: 'planner',
        role: 'Lead',
        model: { provider: 'anthropic', id: 'claude-sonnet-4-6' },
        allowed_tools: ['Read'],
        allowed_skills: [],
        spawns: [],
        // capabilities is REQUIRED by agent.v1.json:9; M09.B's NodeConfigPanel
        // renders the File-access editor off capabilities.file_access, so the
        // fixture must carry the full minimal-valid Capabilities (mirrors
        // builderAgent — builderStore.ts).
        capabilities: {
          tools_called: [],
          skills_loaded: [],
          file_access: { read: [], write: [] },
          network: [],
          shell: false,
          spawn_agents: [],
        },
      },
    ],
  } as unknown as Framework;
}

describe('NodeConfigPanel', () => {
  beforeEach(() => {
    useBuilderStore.setState(useBuilderStore.getInitialState(), true);
  });

  it('renders_nothing_when_no_node_is_selected', () => {
    useBuilderStore.setState({ framework: frameworkWithAgent(), selectedNodeId: null });
    render(<NodeConfigPanel />);
    expect(screen.queryByTestId('builder-node-config')).not.toBeInTheDocument();
  });

  it('renders_role_model_and_allowed_lists_for_a_selected_agent', () => {
    useBuilderStore.setState({ framework: frameworkWithAgent(), selectedNodeId: 'agent:planner' });
    render(<NodeConfigPanel />);
    expect(screen.getByTestId('builder-node-config')).toBeInTheDocument();
    expect(screen.getByTestId('node-config-role')).toHaveValue('Lead');
    expect(screen.getByTestId('node-config-model')).toHaveValue('claude-sonnet-4-6');
    // The allowed-tools list reflects the agent's current allowed_tools.
    expect(screen.getByTestId('node-config-tools')).toHaveTextContent('Read');
    expect(screen.getByTestId('node-config-skills')).toBeInTheDocument();
  });

  it('editing_the_role_field_calls_updateNode', () => {
    const updateNode = vi.fn();
    useBuilderStore.setState({
      framework: frameworkWithAgent(),
      selectedNodeId: 'agent:planner',
      updateNode,
    });
    render(<NodeConfigPanel />);
    fireEvent.change(screen.getByTestId('node-config-role'), { target: { value: 'Architect' } });
    expect(updateNode).toHaveBeenCalledWith('agent:planner', { role: 'Architect' });
  });

  it('selecting_a_model_calls_updateNode_with_the_model_patch', () => {
    const updateNode = vi.fn();
    useBuilderStore.setState({
      framework: frameworkWithAgent(),
      selectedNodeId: 'agent:planner',
      updateNode,
    });
    render(<NodeConfigPanel />);
    fireEvent.change(screen.getByTestId('node-config-model'), {
      target: { value: 'claude-opus-4-7' },
    });
    // model is the schema ModelRef shape — id swapped, provider held.
    expect(updateNode).toHaveBeenCalledWith('agent:planner', {
      model: { provider: 'anthropic', id: 'claude-opus-4-7' },
    });
  });

  it('adding_a_tool_to_the_allowed_list_calls_updateNode', async () => {
    const updateNode = vi.fn();
    useBuilderStore.setState({
      framework: frameworkWithAgent(),
      selectedNodeId: 'agent:planner',
      updateNode,
    });
    render(<NodeConfigPanel />);
    await userEvent.type(screen.getByTestId('node-config-add-tool-input'), 'Bash');
    await userEvent.click(screen.getByTestId('node-config-add-tool'));
    expect(updateNode).toHaveBeenCalledWith('agent:planner', { allowed_tools: ['Read', 'Bash'] });
  });

  it('removing_a_tool_from_the_allowed_list_calls_updateNode', async () => {
    const updateNode = vi.fn();
    useBuilderStore.setState({
      framework: frameworkWithAgent(),
      selectedNodeId: 'agent:planner',
      updateNode,
    });
    render(<NodeConfigPanel />);
    await userEvent.click(screen.getByTestId('node-config-tool-remove-Read'));
    expect(updateNode).toHaveBeenCalledWith('agent:planner', { allowed_tools: [] });
  });
});
