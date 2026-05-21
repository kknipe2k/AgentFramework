import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { CapabilityDisclosure } from '../../../src/components/CapabilityDisclosure';

// M08.D1 — the shared plain-English capability-disclosure surface
// (the M05 §8.security L1 disclosure, lifted out of ImportPanel as a
// behavior-preserving extraction so the Builder nodes reuse it — its
// third reuse). Renders a declared-capability list, or an empty-state
// line when there is nothing to disclose.

describe('CapabilityDisclosure', () => {
  it('renders_one_list_item_per_capability_line', () => {
    render(
      <CapabilityDisclosure
        capabilities={['Can use the Read tool', 'Can load the planning skill']}
        emptyMessage="none"
        data-testid="disc"
      />,
    );
    const disc = screen.getByTestId('disc');
    expect(disc.querySelectorAll('li')).toHaveLength(2);
    expect(disc).toHaveTextContent('Can use the Read tool');
    expect(disc).toHaveTextContent('Can load the planning skill');
  });

  it('renders_the_empty_message_when_there_are_no_capabilities', () => {
    render(
      <CapabilityDisclosure
        capabilities={[]}
        emptyMessage="No tools or skills assigned yet."
        data-testid="disc"
      />,
    );
    const disc = screen.getByTestId('disc');
    expect(disc.querySelectorAll('li')).toHaveLength(0);
    expect(disc).toHaveTextContent('No tools or skills assigned yet.');
  });
});
