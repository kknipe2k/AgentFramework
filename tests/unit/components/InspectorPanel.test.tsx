import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { InspectorPanel } from '../../../src/components/InspectorPanel';
import { useGraphStore } from '../../../src/lib/graphStore';
import type { AgentEvent } from '../../../src/types/agent_event';

const spawnA: AgentEvent = {
  type: 'agent_spawned',
  agent_id: 'a1',
  agent_name: 'smoke',
  parent_id: null,
  session_id: 's1',
};

function reset(): void {
  useGraphStore.getState().clear();
}

describe('InspectorPanel', () => {
  beforeEach(reset);
  afterEach(reset);

  it('returns_null_when_no_selectedNodeId', () => {
    const { container } = render(<InspectorPanel />);
    expect(container).toBeEmptyDOMElement();
    expect(screen.queryByTestId('inspector-panel')).toBeNull();
  });

  it('renders_panel_when_selectedNodeId_is_set', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().selectNode('agent:a1');
    render(<InspectorPanel />);
    expect(screen.getByTestId('inspector-panel')).toBeInTheDocument();
  });

  it('renders_selected_nodes_data_as_json', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().selectNode('agent:a1');
    render(<InspectorPanel />);
    const pre = screen.getByTestId('inspector-panel').querySelector('pre');
    expect(pre).not.toBeNull();
    expect(pre!.textContent).toContain('"agentId"');
    expect(pre!.textContent).toContain('"a1"');
    expect(pre!.textContent).toContain('"agentName"');
    expect(pre!.textContent).toContain('"smoke"');
  });

  it('escape_keydown_clears_selectedNodeId', async () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().selectNode('agent:a1');
    render(<InspectorPanel />);
    expect(screen.getByTestId('inspector-panel')).toBeInTheDocument();
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(useGraphStore.getState().selectedNodeId).toBeNull();
  });

  it('close_button_clears_selectedNodeId', async () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().selectNode('agent:a1');
    const user = userEvent.setup();
    render(<InspectorPanel />);
    await user.click(screen.getByRole('button', { name: /close inspector/i }));
    expect(useGraphStore.getState().selectedNodeId).toBeNull();
  });

  it('exposes_aria_dialog_attributes_with_aria_modal_false', () => {
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().selectNode('agent:a1');
    render(<InspectorPanel />);
    const panel = screen.getByTestId('inspector-panel');
    expect(panel).toHaveAttribute('role', 'dialog');
    expect(panel).toHaveAttribute('aria-label', expect.stringMatching(/inspector/i));
    // Non-modal: graph behind the panel stays interactable.
    expect(panel).toHaveAttribute('aria-modal', 'false');
  });
});
