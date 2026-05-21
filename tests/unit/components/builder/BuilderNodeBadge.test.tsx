import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider, type NodeProps } from '@xyflow/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { BuilderAgentNode } from '../../../../src/components/builder/nodes/BuilderAgentNode';
import { BuilderToolNode } from '../../../../src/components/builder/nodes/BuilderToolNode';
import { useBuilderStore } from '../../../../src/lib/builderStore';
import type { FrameworkValidationReport, NodeError } from '../../../../src/lib/ipc';

// M08.D2 — the red validation badge (spec Phase 9 "errors surfaced as
// red badges"). Each builder node component reads builderStore.validation
// and renders a red badge when the report keys a schema_errors /
// capability_errors entry to that node's path. The badge logic is
// per-node — a node not keyed by any error stays clean even when the
// report carries errors for other nodes.

/** Build a validation report carrying the given keyed errors. */
function reportWith(schema: NodeError[], capability: NodeError[]): FrameworkValidationReport {
  return {
    schema_errors: schema,
    capability_errors: capability,
    ok: schema.length === 0 && capability.length === 0,
    capability_summary: null,
  };
}

/** Render one builder node component with the full NodeProps surface. */
function renderNode(
  Component: (props: NodeProps) => JSX.Element,
  type: string,
  data: Record<string, unknown>,
): void {
  render(
    <ReactFlowProvider>
      <Component
        id={`${type}:x`}
        type={type}
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

/** An agent node whose validation path is `nodePath`. */
function renderAgent(nodePath: string): void {
  renderNode(BuilderAgentNode, 'agent', {
    agentId: nodePath,
    nodePath,
    role: 'Lead',
    model: 'claude-sonnet-4-6',
    allowedTools: [],
    allowedSkills: [],
  });
}

describe('Builder node validation badge', () => {
  beforeEach(() => {
    useBuilderStore.setState({ validation: null });
  });

  it('node_with_no_validation_errors_renders_no_badge', () => {
    useBuilderStore.setState({ validation: reportWith([], []) });
    renderAgent('planner');
    expect(screen.queryByTestId('builder-node-badge')).not.toBeInTheDocument();
  });

  it('node_with_a_schema_error_keyed_to_its_path_renders_the_red_badge', () => {
    useBuilderStore.setState({
      validation: reportWith([{ node_path: 'planner', message: 'missing required field' }], []),
    });
    renderAgent('planner');
    const badge = screen.getByTestId('builder-node-badge');
    expect(badge).toBeInTheDocument();
    // The node root carries the --invalid modifier (red border, D2.3.7).
    expect(screen.getByTestId('builder-agent-node-planner')).toHaveClass('builder-node--invalid');
  });

  it('node_with_a_capability_error_keyed_to_its_path_renders_the_red_badge', () => {
    useBuilderStore.setState({
      validation: reportWith([], [{ node_path: 'planner', message: 'unresolved tool reference' }]),
    });
    renderAgent('planner');
    expect(screen.getByTestId('builder-node-badge')).toBeInTheDocument();
  });

  it('badge_count_reflects_the_number_of_errors_for_that_node', () => {
    useBuilderStore.setState({
      validation: reportWith(
        [{ node_path: 'planner', message: 'schema problem' }],
        [{ node_path: 'planner', message: 'capability problem' }],
      ),
    });
    renderAgent('planner');
    // One schema + one capability error keyed to this node — the badge
    // is the at-a-glance count.
    expect(screen.getByTestId('builder-node-badge')).toHaveTextContent('2');
  });

  it('a_node_not_keyed_by_any_error_renders_no_badge_even_when_the_report_has_errors', () => {
    useBuilderStore.setState({
      validation: reportWith([{ node_path: 'other-agent', message: 'schema problem' }], []),
    });
    renderAgent('planner');
    // The report is non-empty, but no entry keys THIS node — no badge.
    expect(screen.queryByTestId('builder-node-badge')).not.toBeInTheDocument();
  });

  it('the_badge_is_generic_across_node_kinds_a_tool_node_badges_too', () => {
    useBuilderStore.setState({
      validation: reportWith([{ node_path: 'Read', message: 'schema problem' }], []),
    });
    renderNode(BuilderToolNode, 'tool', { name: 'Read', nodePath: 'Read' });
    expect(screen.getByTestId('builder-node-badge')).toBeInTheDocument();
    expect(screen.getByTestId('builder-tool-node-Read')).toHaveClass('builder-node--invalid');
  });
});

// gotcha #67 — a className rendered in the DOM with no styles.css rule
// renders unstyled and the user sees nothing. Every D2 class gets a rule.
describe('Builder D2 styles (gotcha #67)', () => {
  const css = readFileSync(resolve(__dirname, '../../../../src/styles.css'), 'utf8');
  const D2_CLASSES = ['builder-node--invalid', 'builder-node__badge', 'builder-edge'] as const;

  it.each(D2_CLASSES)('styles.css defines a rule for .%s', (cls) => {
    expect(css).toMatch(new RegExp(`\\.${cls}[\\s,{:]`));
  });

  it('the invalid-node + badge styles use theme variables, not literal colors', () => {
    // The red token is --node-error (M07-IRL #3 — no literal hex).
    expect(css).toMatch(/\.builder-node--invalid[\s\S]*?var\(--node-error\)/);
  });
});
