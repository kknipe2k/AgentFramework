//! Tier system — spec §8.security L4 (M05 Stage D).
//!
//! v0.1 ships two tiers per §0d release scope:
//!
//! - `Tier::Novice` — curated allowlist. Read-only filesystem (no
//!   `Write`/`Exec`); HTTPS-only network (no `ProcessSpawn`, no
//!   glob-scoped `Network`). The default-safe first-run posture.
//! - `Tier::Promoted` — full capability surface. L1 still narrows by
//!   declaration; the L4 gate is a pass-through for this tier.
//!
//! The L4 tier evaluator sits BEFORE the L1+L2a enforcer in the
//! dispatch chain: tier check → enforcer check → dispatch. A Promoted
//! user with a `Write` declaration still passes through L1; a Novice
//! user requesting `Write` is rejected at L4 before L1 even runs.
//!
//! Tier transitions are user-initiated. Promotion (Novice → Promoted)
//! routes through a renderer-side confirmation modal (Settings panel)
//! that calls the `request_tier_transition` Tauri command. Demotion
//! (Promoted → Novice) is also user-initiated but skips confirmation —
//! it's always safer.
//!
//! Persisted in `<app_data_dir>/tier.json`. First-run default is
//! `Tier::Novice` — `tier.json` absent → `Tier::default()`.
//!
//! Safety primitive: ≥95% per-module coverage gate per CLAUDE.md §5.

/// Error types — `TierError` (evaluator) + `TierPersistenceError`
/// (load/save).
pub mod error;
/// Stateless L4 evaluator — `TierEvaluator::allows(tier, decl)`.
pub mod evaluator;
/// Data-driven Novice allowlist tables.
pub mod matrix;
/// Read / write the user's current tier from `<dir>/tier.json`.
pub mod persistence;
/// Tier transition primitive (M05 Stage E).
///
/// Emits the audit line + returns the previous/next tier pair so the
/// Tauri layer can update renderer state and persist the new value.
pub mod transition;

pub use error::{TierError, TierPersistenceError};
pub use evaluator::{Tier, TierEvaluator};
pub use persistence::{load_tier, save_tier};
pub use transition::{transition, TierTransitionRecord};
