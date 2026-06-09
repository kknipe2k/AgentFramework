import { beforeEach, describe, expect, it } from 'vitest';
import { createGraphStore, useGraphStore } from '../../src/lib/graphStore';
import type { AgentEvent } from '../../src/types/agent_event';

// M08.8.A — the live-graph execution view (TD-034). The Output/Inspector
// rail renders from store state; these tests pin the store-level data the
// rail reads: the `stream_text` output buffer (no longer a no-op) and the
// tool nodes' retained {input, output} payload. The scoped Tester store
// (createGraphStore) reuses the SAME reducer, so the parity case proves
// "works over BOTH stores" at the store layer.

const spawn: AgentEvent = {
  type: 'agent_spawned',
  agent_id: 'a1',
  agent_name: 'reader',
  parent_id: null,
  session_id: 's1',
};

const streamHello: AgentEvent = { type: 'stream_text', agent_id: 'a1', text: 'hello ' };
const streamWorld: AgentEvent = { type: 'stream_text', agent_id: 'a1', text: 'world' };

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
  duration_ms: 12,
};

function reset(): void {
  useGraphStore.getState().clear();
}

describe('graphStore — execution-view surfacing (M08.8.A)', () => {
  beforeEach(reset);

  it('stream_text_appends_event_text_to_the_output_buffer', () => {
    const store = useGraphStore.getState();
    store.applyEvent(streamHello);
    store.applyEvent(streamWorld);
    // The agent's streamed reply is no longer dropped — it accumulates
    // into outputBuffer so the Output rail can render it (DESIGN principle
    // 1: visible output; mono register). Pre-A this was a graphStore no-op.
    expect(useGraphStore.getState().outputBuffer).toBe('hello world');
  });

  it('clear_resets_the_output_buffer', () => {
    const store = useGraphStore.getState();
    store.applyEvent(streamHello);
    expect(useGraphStore.getState().outputBuffer).not.toBe('');
    store.clear();
    expect(useGraphStore.getState().outputBuffer).toBe('');
  });

  it('tool_invoked_retains_the_input_payload_on_the_tool_node', () => {
    const store = useGraphStore.getState();
    store.applyEvent(spawn);
    store.applyEvent(readInvoked);
    const tool = useGraphStore.getState().nodes.find((n) => n.id === 'tool:a1:Read');
    expect(tool).toBeDefined();
    expect(tool!.data.input).toEqual({ path: 'Cargo.toml' });
  });

  it('tool_result_retains_the_output_payload_on_the_tool_node', () => {
    const store = useGraphStore.getState();
    store.applyEvent(spawn);
    store.applyEvent(readInvoked);
    store.applyEvent(readResult);
    const tool = useGraphStore.getState().nodes.find((n) => n.id === 'tool:a1:Read');
    expect(tool).toBeDefined();
    expect(tool!.data.output).toBe('[package]\nname = "agent-runtime"');
    // The status axis still flips to complete (the existing behavior must
    // survive the payload addition).
    expect(tool!.data.status).toBe('complete');
  });

  it('scoped_store_surfaces_output_and_tool_payloads_identically_to_the_live_store', () => {
    // The Tester's scoped store (createGraphStore) reuses the SAME reducer
    // — "works over BOTH stores" must hold at the store layer.
    const scoped = createGraphStore();
    scoped.getState().applyEvent(spawn);
    scoped.getState().applyEvent(streamHello);
    scoped.getState().applyEvent(readInvoked);
    scoped.getState().applyEvent(readResult);
    expect(scoped.getState().outputBuffer).toBe('hello ');
    const tool = scoped.getState().nodes.find((n) => n.id === 'tool:a1:Read');
    expect(tool!.data.input).toEqual({ path: 'Cargo.toml' });
    expect(tool!.data.output).toBe('[package]\nname = "agent-runtime"');
  });
});
