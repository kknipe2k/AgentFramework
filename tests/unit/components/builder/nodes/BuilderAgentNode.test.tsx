import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider } from '@xyflow/react';
import { BuilderAgentNode } from '../../../../../src/components/builder/nodes/BuilderAgentNode';

// M08.D1 — the interactive Agent node. Rendered directly inside a
// ReactFlowProvider (the MCPNode unit-test precedent) so the
// React-Flow Handle elements resolve their context; the full canvas
// integration is the Playwright spec's job.

interface AgentNodeData extends Record<string, unknown> {
  agentId: string;
  role: string;
  model: string;
  allowedTools: string[];
  allowedSkills: string[];
}

function renderAgentNode(data: AgentNodeData): ReturnType<typeof render> {
  return render(
    <ReactFlowProvider>
      <BuilderAgentNode
        id={`agent:${data.agentId}`}
        type="agent"
        data={data}
        dragging={false}
        zIndex={0}
        selectable
        deletable
        selected={false}
        draggable
        isConnectable
        positionAbsoluteX={0}
        positionAbsoluteY={0}
      />
    </ReactFlowProvider>,
  );
}

describe('BuilderAgentNode', () => {
  it('renders_the_agent_id_and_role', () => {
    renderAgentNode({
      agentId: 'planner',
      role: 'Lead planner',
      model: 'claude-sonnet-4-6',
      allowedTools: [],
      allowedSkills: [],
    });
    expect(screen.getByText('planner')).toBeInTheDocument();
    expect(screen.getByText('Lead planner')).toBeInTheDocument();
  });

  it('renders_the_plain_english_capability_disclosure_from_allowed_star', () => {
    renderAgentNode({
      agentId: 'planner',
      role: 'Lead',
      model: 'claude-sonnet-4-6',
      allowedTools: ['Read', 'Write'],
      allowedSkills: ['planning'],
    });
    const disclosure = screen.getByTestId('builder-node-disclosure-planner');
    // Derived live from allowed_tools / allowed_skills.
    expect(disclosure).toHaveTextContent('Read');
    expect(disclosure).toHaveTextContent('Write');
    expect(disclosure).toHaveTextContent('planning');
    // Plain English — full sentences, not a bare identifier dump.
    expect(disclosure.textContent ?? '').toMatch(/tool/i);
    expect(disclosure.textContent ?? '').toMatch(/skill/i);
    // One disclosure line per declared capability.
    expect(disclosure.querySelectorAll('li')).toHaveLength(3);
  });

  it('renders_a_placeholder_when_no_role_is_set', () => {
    renderAgentNode({
      agentId: 'planner',
      role: '',
      model: 'claude-sonnet-4-6',
      allowedTools: [],
      allowedSkills: [],
    });
    expect(screen.getByText(/no role set/i)).toBeInTheDocument();
  });

  it('renders_an_empty_disclosure_message_when_no_tools_or_skills', () => {
    renderAgentNode({
      agentId: 'planner',
      role: 'Lead',
      model: 'claude-sonnet-4-6',
      allowedTools: [],
      allowedSkills: [],
    });
    const disclosure = screen.getByTestId('builder-node-disclosure-planner');
    expect(disclosure.querySelectorAll('li')).toHaveLength(0);
    expect(disclosure.textContent ?? '').toMatch(/no tools or skills/i);
  });
});
