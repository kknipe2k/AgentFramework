import { useGraphStore } from '../../lib/graphStore';

interface CapabilityBadgeProps {
  agentId: string;
}

/**
 * Per-AgentNode pill — spec §8.security L4 (tier) + L2a (grants).
 *
 * Renders the user's current tier as a single-letter glyph ('N' for
 * Novice, 'P' for Promoted) plus a count of grants already issued to
 * this specific agent. Tier comes from the store's `currentTier` (driven
 * by `tier_transition`); grant count is the filtered length of
 * `capabilityGrants` keyed by `grantedTo === agentId`.
 *
 * Filtering by `agentId` is load-bearing: gotcha #68 — a regression that
 * surfaced every grant under every badge would tell every user their
 * agent was over-privileged. Per gotcha #67 every class set here has a
 * matching rule in styles.css.
 */
export function CapabilityBadge({ agentId }: CapabilityBadgeProps): JSX.Element {
  const tier = useGraphStore((s) => s.currentTier);
  const grantCount = useGraphStore(
    (s) => s.capabilityGrants.filter((g) => g.grantedTo === agentId).length,
  );
  const glyph = tier === 'promoted' ? 'P' : 'N';
  return (
    <span
      className={`capability-badge capability-badge--${tier}`}
      data-testid={`capability-badge-${agentId}`}
      title={`Tier: ${tier} (${grantCount} grants)`}
      aria-label={`tier ${tier}, ${grantCount} grants`}
    >
      <span className="capability-badge__glyph">{glyph}</span>
      {grantCount > 0 && <span className="capability-badge__count">{grantCount}</span>}
    </span>
  );
}
