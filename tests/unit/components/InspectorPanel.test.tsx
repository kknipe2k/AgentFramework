import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { InspectorPanel } from '../../../src/components/InspectorPanel';
import { createGraphStore, useGraphStore } from '../../../src/lib/graphStore';
import type { AgentEvent } from '../../../src/types/agent_event';

const spawnA: AgentEvent = {
  type: 'agent_spawned',
  agent_id: 'a1',
  agent_name: 'smoke',
  parent_id: null,
  session_id: 's1',
};

const readInvoked: AgentEvent = {
  type: 'tool_invoked',
  agent_id: 'a1',
  tool_name: 'Read',
  source: 'builtin',
  input: { path: 'Cargo.toml' },
};

const readResult: AgentEvent = {
  type: 'tool_result',
  agent_id: 'a1',
  tool_name: 'Read',
  output: '[package]\nname = "agent-runtime"',
  duration_ms: 7,
};

const streamText: AgentEvent = { type: 'stream_text', agent_id: 'a1', text: 'the file says hello' };

function reset(): void {
  useGraphStore.getState().clear();
}

describe('InspectorPanel — the Output/Inspector rail (M08.8.A)', () => {
  beforeEach(reset);
  afterEach(reset);

  it('returns_null_when_no_selection_and_no_output', () => {
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
  });

  it('renders_the_streamed_agent_text_in_the_output_rail_even_with_no_selection', () => {
    // The #1 TD-034 gap: the agent's reply text was trapped in RUST_LOG.
    // stream_text now feeds the Output rail; it renders without a node
    // being selected (the reply is session-level, not node-keyed here).
    useGraphStore.getState().applyEvent(spawnA);
    useGraphStore.getState().applyEvent(streamText);
    render(<InspectorPanel />);
    const output = screen.getByTestId('output-rail');
    expect(output).toBeInTheDocument();
    expect(output.textContent).toContain('the file says hello');
  });

  it('shows_the_selected_tool_nodes_input_and_output_payload', () => {
    // The #2 TD-034 gap: tool nodes were not clickable and exposed no
    // payload. Selecting a Read tool node surfaces its input path + the
    // file contents it returned.
    const store = useGraphStore.getState();
    store.applyEvent(spawnA);
    store.applyEvent(readInvoked);
    store.applyEvent(readResult);
    store.selectNode('tool:a1:Read');
    render(<InspectorPanel />);
    const input = screen.getByTestId('inspector-tool-input');
    const output = screen.getByTestId('inspector-tool-output');
    expect(input.textContent).toContain('Cargo.toml');
    expect(output.textContent).toContain('[package]');
  });

  it('renders_from_a_scoped_store_passed_via_the_store_prop', () => {
    // "Works over BOTH stores": the Tester mounts the same rail bound to
    // the scoped useTestGraphStore. A live-store clear must not blank a
    // scoped-store-bound rail.
    const scoped = createGraphStore();
    scoped.getState().applyEvent(spawnA);
    scoped.getState().applyEvent(streamText);
    render(<InspectorPanel store={scoped} />);
    expect(screen.getByTestId('output-rail').textContent).toContain('the file says hello');
  });

  it('escape_keydown_clears_selectedNodeId', () => {
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
    expect(panel).toHaveAttribute('aria-modal', 'false');
  });
});
