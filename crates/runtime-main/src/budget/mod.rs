//! Budget primitive — spec §2a (M04 Stage F).
//!
//! Three budget scopes evaluated tightest-cap-wins:
//! session / framework / per-day global.
//!
//! Four threshold actions: warn (50%), downshift (75%), HITL (90%),
//! hard-stop (100%). Defaults per spec §2a; each is independently
//! configurable per scope and may be disabled by setting to `None`.
//!
//! Cost computation uses the provider's `count_tokens` endpoint
//! (Stage A2 wired the real call) via `cost::CostCache` LRU caching.
//!
//! Safety primitive: ≥95% coverage per CLAUDE.md §5.

#![allow(
    clippy::too_long_first_doc_paragraph,
    reason = "the module-level `//!` doc spans multiple paragraphs but clippy 1.95 reads it as one — splitting further hurts readability"
)]

/// Token-cost cache with LRU eviction.
pub mod cost;
/// Threshold-crossing enforcer that drives the four budget actions.
pub mod enforcer;
/// Downshift hook seam.
pub mod hook;

pub use cost::{CostCache, CostKey};
pub use enforcer::{BudgetEnforcer, BudgetScopeCap, BudgetThreshold, ThresholdAction};
pub use hook::{DefaultLadder, DownshiftHook, RemainingBudget};
