import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { act, render, screen } from '@testing-library/react';
import { CapabilityBadge } from '../../../src/components/nodes/CapabilityBadge';
import { useGraphStore } from '../../../src/lib/graphStore';
import type { AgentEvent } from '../../../src/types/agent_event';

// Spec §8.security L4 (tier) + L2a (capability grants). CapabilityBadge is
// a per-AgentNode pill showing the current user tier + a count of grants
// already issued to this agent. Pure-render — reads `currentTier` and
// `capabilityGrants` via selectors. Per gotcha #67 (every class set must
// have a CSS rule) and gotcha #68 (read the right projection field).

function dispatch(events: AgentEvent[]): void {
  act(() => {
    for (const e of events) {
      useGraphStore.getState().applyEvent(e);
    }
  });
}

describe('CapabilityBadge', () => {
  beforeEach(() => {
    // `clear()` preserves `currentTier` by design (per-installation
    // preference per spec §8.security L4). Tests that mutate the tier
    // must reset it explicitly so the next case starts from the default.
    useGraphStore.getState().clear();
    useGraphStore.setState({ currentTier: 'novice' });
  });

  afterEach(() => {
    useGraphStore.getState().clear();
    useGraphStore.setState({ currentTier: 'novice' });
  });

  it('renders_N_letter_for_novice_tier', () => {
    // Default tier is 'novice' (matches the runtime's Tier::default()).
    render(<CapabilityBadge agentId="a1" />);
    const badge = screen.getByTestId('capability-badge-a1');
    expect(badge.textContent).toContain('N');
    expect(badge.className).toContain('capability-badge--novice');
  });

  it('renders_P_letter_after_tier_transition_to_promoted', () => {
    dispatch([
      {
        type: 'tier_transition',
        previous: 'novice',
        current: 'promoted',
        reason: 'user_promoted',
      },
    ]);
    render(<CapabilityBadge agentId="a1" />);
    const badge = screen.getByTestId('capability-badge-a1');
    expect(badge.textContent).toContain('P');
    expect(badge.className).toContain('capability-badge--promoted');
  });

  it('hides_grant_count_when_zero', () => {
    render(<CapabilityBadge agentId="a1" />);
    const badge = screen.getByTestId('capability-badge-a1');
    expect(badge.querySelector('.capability-badge__count')).toBeNull();
  });

  it('shows_grant_count_when_at_least_one_grant_to_this_agent', () => {
    dispatch([
      {
        type: 'capability_grant',
        granted_to: 'a1',
        capability_kind: 'read',
        resource: '/tmp/foo',
      },
      {
        type: 'capability_grant',
        granted_to: 'a1',
        capability_kind: 'network',
        resource: 'api.example.com',
      },
    ]);
    render(<CapabilityBadge agentId="a1" />);
    const count = screen
      .getByTestId('capability-badge-a1')
      .querySelector('.capability-badge__count');
    expect(count).not.toBeNull();
    expect(count?.textContent).toBe('2');
  });

  it('only_counts_grants_for_this_agent_per_gotcha_68', () => {
    // Gotcha #68: read the right projection field. The selector filters
    // by `grantedTo === agentId`; a regression dropping the filter would
    // surface every grant in the session under every agent's badge.
    dispatch([
      {
        type: 'capability_grant',
        granted_to: 'a1',
        capability_kind: 'read',
        resource: '/tmp/foo',
      },
      {
        type: 'capability_grant',
        granted_to: 'a2',
        capability_kind: 'network',
        resource: 'api.example.com',
      },
      {
        type: 'capability_grant',
        granted_to: 'a2',
        capability_kind: 'write',
        resource: '/var/log',
      },
    ]);
    render(<CapabilityBadge agentId="a1" />);
    const a1Count = screen
      .getByTestId('capability-badge-a1')
      .querySelector('.capability-badge__count');
    expect(a1Count?.textContent).toBe('1');
  });

  it('every_tier_class_has_a_corresponding_CSS_rule_in_styles_css', () => {
    // Gotcha #67 (PR #64 pattern). Component writes
    // `.capability-badge--novice` / `.capability-badge--promoted` from the
    // tier projection; styles.css MUST contain matching selectors or the
    // pill renders unstyled even with the correct className.
    const cssPath = resolve(__dirname, '../../../src/styles.css');
    const css = readFileSync(cssPath, 'utf8');
    for (const tier of ['novice', 'promoted'] as const) {
      const selector = `.capability-badge--${tier}`;
      expect(css, `missing CSS rule for ${selector}`).toContain(selector);
    }
    expect(css, 'missing CSS rule for .capability-badge').toContain('.capability-badge');
    expect(css, 'missing CSS rule for .capability-badge__count').toContain(
      '.capability-badge__count',
    );
  });

  it('exposes_title_attribute_for_keyboard_hover_disclosure', () => {
    dispatch([
      {
        type: 'capability_grant',
        granted_to: 'a1',
        capability_kind: 'read',
        resource: '/tmp/foo',
      },
    ]);
    render(<CapabilityBadge agentId="a1" />);
    const badge = screen.getByTestId('capability-badge-a1');
    const title = badge.getAttribute('title') ?? '';
    expect(title.toLowerCase()).toContain('novice');
    expect(title).toContain('1');
  });
});
