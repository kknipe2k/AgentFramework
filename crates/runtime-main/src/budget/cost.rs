//! Token-cost cache used by the budget enforcer to avoid re-querying the
//! provider's `count_tokens` endpoint for messages the SDK has already
//! costed. Per spec §2a Stage F: cost computation uses real
//! `count_tokens` (Stage A2 endpoint) cached per-message with LRU per
//! session.
//!
//! The cache is keyed by a stable hash of the message-content list. Hits
//! return the cached token count; misses cost the caller a single
//! provider round-trip. Eviction is LRU — `capacity` is the max number
//! of distinct message-content hashes retained per session.

use std::collections::HashMap;
use std::collections::VecDeque;

use crate::providers::Message;

/// LRU key — stable hash of a [`Message`] slice. Lifetime-free so it can
/// be stored in the cache's `VecDeque<CostKey>` for eviction order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CostKey(u64);

impl CostKey {
    /// Compute the cache key for a message slice. Uses
    /// `serde_json::to_string` so role + content variants are reflected
    /// — different content blocks for the same role hash differently.
    #[must_use]
    pub fn from_messages(messages: &[Message]) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        // serde-json round-trip gives a stable canonical form for the
        // `ContentBlock` enum across runs in this process (the serde
        // derives are deterministic for the same input). For
        // proptest-style stability across process boundaries the hash
        // here is enough — the cache is per-session, in-memory only.
        for msg in messages {
            serde_json::to_string(msg)
                .unwrap_or_default()
                .hash(&mut hasher);
        }
        Self(hasher.finish())
    }

    /// Construct from a raw hash. Test affordance.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

/// LRU-evicting cache mapping a [`CostKey`] to a cached input-token count.
#[derive(Debug)]
pub struct CostCache {
    capacity: usize,
    /// Eviction order — back = most recent, front = oldest.
    order: VecDeque<CostKey>,
    map: HashMap<CostKey, u64>,
}

impl CostCache {
    /// New cache with the given capacity. Capacity 0 disables caching;
    /// every `get` returns `None`.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            order: VecDeque::with_capacity(capacity),
            map: HashMap::with_capacity(capacity),
        }
    }

    /// Look up a cached count. Bumps the entry to the back of the LRU
    /// order on hit.
    pub fn get(&mut self, key: &CostKey) -> Option<u64> {
        let value = *self.map.get(key)?;
        // Bump to most-recent.
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
        self.order.push_back(*key);
        Some(value)
    }

    /// Insert or update. Evicts the oldest entry if over capacity.
    pub fn insert(&mut self, key: CostKey, value: u64) {
        if self.capacity == 0 {
            return;
        }
        if self.map.insert(key, value).is_none() {
            // New entry — track in order list.
            self.order.push_back(key);
            while self.order.len() > self.capacity {
                if let Some(oldest) = self.order.pop_front() {
                    self.map.remove(&oldest);
                }
            }
        } else {
            // Existing entry — bump to most-recent.
            if let Some(pos) = self.order.iter().position(|k| k == &key) {
                self.order.remove(pos);
            }
            self.order.push_back(key);
        }
    }

    /// Number of entries currently cached.
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// `true` when the cache holds no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Test affordance — expose the eviction order for assertions.
    #[cfg(test)]
    #[must_use]
    pub fn order_snapshot(&self) -> Vec<CostKey> {
        self.order.iter().copied().collect()
    }
}

impl Default for CostCache {
    fn default() -> Self {
        // 256 messages × ~hundreds of bytes per entry = trivial memory.
        // Larger than the typical multi-turn agent loop's message count.
        Self::new(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{ContentBlock, Message, MessageRole};

    fn msg(text: &str) -> Message {
        Message {
            role: MessageRole::User,
            content: vec![ContentBlock::Text {
                text: text.to_string(),
            }],
        }
    }

    #[test]
    fn empty_cache_returns_none() {
        let mut c = CostCache::new(4);
        let key = CostKey::from_messages(&[msg("hi")]);
        assert_eq!(c.get(&key), None);
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);
    }

    #[test]
    fn insert_then_get_returns_value() {
        let mut c = CostCache::new(4);
        let key = CostKey::from_messages(&[msg("hi")]);
        c.insert(key, 42);
        assert_eq!(c.get(&key), Some(42));
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn lru_evicts_oldest_when_over_capacity() {
        let mut c = CostCache::new(2);
        let k1 = CostKey::from_raw(1);
        let k2 = CostKey::from_raw(2);
        let k3 = CostKey::from_raw(3);
        c.insert(k1, 1);
        c.insert(k2, 2);
        c.insert(k3, 3); // evicts k1.
        assert_eq!(c.get(&k1), None);
        assert_eq!(c.get(&k2), Some(2));
        assert_eq!(c.get(&k3), Some(3));
    }

    #[test]
    fn get_bumps_to_most_recent() {
        let mut c = CostCache::new(2);
        let k1 = CostKey::from_raw(1);
        let k2 = CostKey::from_raw(2);
        let k3 = CostKey::from_raw(3);
        c.insert(k1, 1);
        c.insert(k2, 2);
        let _ = c.get(&k1); // k1 now most-recent; k2 oldest.
        c.insert(k3, 3); // evicts k2 (oldest).
        assert_eq!(c.get(&k1), Some(1));
        assert_eq!(c.get(&k2), None);
        assert_eq!(c.get(&k3), Some(3));
    }

    #[test]
    fn capacity_zero_disables_caching() {
        let mut c = CostCache::new(0);
        let k = CostKey::from_raw(1);
        c.insert(k, 42);
        assert_eq!(c.get(&k), None);
        assert!(c.is_empty());
    }

    #[test]
    fn update_existing_does_not_evict() {
        let mut c = CostCache::new(2);
        let k1 = CostKey::from_raw(1);
        let k2 = CostKey::from_raw(2);
        c.insert(k1, 1);
        c.insert(k2, 2);
        // Update k1; cache still has both.
        c.insert(k1, 11);
        assert_eq!(c.get(&k1), Some(11));
        assert_eq!(c.get(&k2), Some(2));
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn update_bumps_to_most_recent() {
        let mut c = CostCache::new(2);
        let k1 = CostKey::from_raw(1);
        let k2 = CostKey::from_raw(2);
        let k3 = CostKey::from_raw(3);
        c.insert(k1, 1);
        c.insert(k2, 2);
        c.insert(k1, 11); // bumps k1 to most-recent.
        c.insert(k3, 3); // evicts k2 (oldest).
        assert_eq!(c.get(&k1), Some(11));
        assert_eq!(c.get(&k2), None);
        assert_eq!(c.get(&k3), Some(3));
    }

    #[test]
    fn cost_key_from_messages_is_deterministic() {
        // Same input → same key across calls.
        let a = CostKey::from_messages(&[msg("hi")]);
        let b = CostKey::from_messages(&[msg("hi")]);
        assert_eq!(a, b);
    }

    #[test]
    fn cost_key_distinguishes_messages() {
        let a = CostKey::from_messages(&[msg("hi")]);
        let b = CostKey::from_messages(&[msg("bye")]);
        assert_ne!(a, b);
    }

    #[test]
    fn order_snapshot_reflects_insert_order() {
        let mut c = CostCache::new(3);
        let k1 = CostKey::from_raw(1);
        let k2 = CostKey::from_raw(2);
        c.insert(k1, 1);
        c.insert(k2, 2);
        assert_eq!(c.order_snapshot(), vec![k1, k2]);
    }

    #[test]
    fn default_constructs_with_reasonable_capacity() {
        let c = CostCache::default();
        // Don't pin to an exact number — just sanity-check non-zero.
        assert!(c.capacity > 0);
    }
}
