import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { NarrowingNotice } from '../../../../src/components/builder/NarrowingNotice';
import { useBuilderStore } from '../../../../src/lib/builderStore';
import type { FrameworkValidationReport, SpawnEdgeNarrowing } from '../../../../src/lib/ipc';
import type { CapabilityDeclaration } from '../../../../src/types/capability';

// M08.D2 — NarrowingNotice SURFACES the Agent→Agent (spawns) edge's
// §8.security L2a narrowing decision from Stage B's validate_framework
// report. The intersection itself is Rust (capability/narrowing.rs);
// spec §9 forbids a TS re-implementation — these tests pin that the
// component renders the backend `narrowed_caps` arm verbatim and
// computes nothing.

/** A minimal capability declaration; `capLabel` renders `kind:resource`. */
function cap(kind: CapabilityDeclaration['kind'], resource: string): CapabilityDeclaration {
  return { kind, resource, scope: { glob: resource }, side_effect_class: 'pure' };
}

/** Wrap one spawn-edge narrowing record in a full validation report. */
function reportWith(spawnEdges: SpawnEdgeNarrowing[]): FrameworkValidationReport {
  return {
    schema_errors: [],
    capability_errors: [],
    ok: spawnEdges.every((e) => 'Ok' in e.narrowed_caps),
    capability_summary: {
      files_read: [],
      files_written: [],
      network_hosts: [],
      any_shell: false,
      spawn_edges: spawnEdges,
    },
  };
}

describe('NarrowingNotice', () => {
  beforeEach(() => {
    useBuilderStore.setState({ validation: null });
  });

  it('renders_nothing_when_no_spawn_edge_in_capability_summary_matches_the_id', () => {
    useBuilderStore.setState({ validation: reportWith([]) });
    const { container } = render(<NarrowingNotice spawnEdgeId="agent:planner->worker" />);
    expect(container).toBeEmptyDOMElement();
  });

  it('renders_child_declared_and_surviving_capability_lists_for_an_ok_narrowing', () => {
    const declared = [cap('read', 'src/**'), cap('network', 'api.example.com')];
    useBuilderStore.setState({
      validation: reportWith([
        {
          parent_id: 'planner',
          child_id: 'worker',
          parent_caps: declared,
          child_declared_caps: declared,
          narrowed_caps: { Ok: declared },
        },
      ]),
    });
    render(<NarrowingNotice spawnEdgeId="agent:planner->worker" />);
    expect(screen.getByText(/narrowing applied/i)).toBeInTheDocument();
    // Child-declared + surviving lists both render the kind:resource labels.
    expect(screen.getByText(/read:src\/\*\*/)).toBeInTheDocument();
    expect(screen.getByText(/network:api\.example\.com/)).toBeInTheDocument();
  });

  it('renders_the_rejection_message_when_the_child_exceeds_the_parent', () => {
    useBuilderStore.setState({
      validation: reportWith([
        {
          parent_id: 'planner',
          child_id: 'worker',
          parent_caps: [cap('read', 'src/**')],
          child_declared_caps: [cap('network', 'net.fetch')],
          narrowed_caps: { Err: 'child declares network:net.fetch which the parent does not hold' },
        },
      ]),
    });
    render(<NarrowingNotice spawnEdgeId="agent:planner->worker" />);
    expect(screen.getByText(/narrowing rejected/i)).toBeInTheDocument();
    const rejected = document.querySelector('.narrowing-notice__rejected');
    expect(rejected?.textContent ?? '').toContain('net.fetch');
  });

  it('surfaces_the_backend_narrowed_caps_verbatim_no_TS_intersection', () => {
    // Contract test (gotcha #66 + spec §9). The backend `narrowed_caps`
    // Ok set carries one declaration the empty `parent_caps` could
    // never yield under a naive TS `child ∩ parent` — so a renderer
    // that recomputed the intersection would show "(none)". Asserting
    // the surviving label IS the backend value proves the renderer
    // surfaces the Rust decision and computes no intersection itself.
    const survivor = cap('exec', 'Bash');
    useBuilderStore.setState({
      validation: reportWith([
        {
          parent_id: 'planner',
          child_id: 'worker',
          parent_caps: [],
          child_declared_caps: [survivor],
          narrowed_caps: { Ok: [survivor] },
        },
      ]),
    });
    render(<NarrowingNotice spawnEdgeId="agent:planner->worker" />);
    const survives = screen.getByTestId('narrowing-notice-survives');
    expect(survives.textContent ?? '').toContain('exec:Bash');
    expect(survives.textContent ?? '').not.toContain('(none)');
  });
});

// gotcha #67 — every D2 narrowing-notice class gets a styles.css rule.
describe('NarrowingNotice styles (gotcha #67)', () => {
  const css = readFileSync(resolve(__dirname, '../../../../src/styles.css'), 'utf8');

  it.each(['narrowing-notice', 'narrowing-notice__rejected'] as const)(
    'styles.css defines a rule for .%s',
    (cls) => {
      expect(css).toMatch(new RegExp(`\\.${cls}[\\s,{:]`));
    },
  );

  it('the narrowing-notice styles use theme variables, not literal colors', () => {
    expect(css).toMatch(/\.narrowing-notice[\s\S]*?var\(--node-/);
  });
});
