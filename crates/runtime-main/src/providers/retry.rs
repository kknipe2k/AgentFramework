//! Bounded pre-stream request retry with exponential backoff + full
//! jitter (TD-054, review C5).
//!
//! Anthropic guidance: exponential backoff with jitter on 429/5xx/529,
//! honoring `retry-after`. The retry wraps *send request → check status*
//! only — it exits the moment a 2xx response exists, BEFORE the SSE layer
//! wraps the body, so a stream that has yielded events is past the retry
//! boundary by construction (replay is impossible, not merely forbidden).
//!
//! This module is the COVERED home for the retry logic (the §6
//! `runtime-main` coverage gate); `providers/anthropic.rs` — the excluded
//! thin HTTP shell — only delegates here.

use std::future::Future;
use std::time::Duration;

use super::ProviderError;

/// Total attempts per request (not "retries"): the first try plus two
/// retries, then the typed error surfaces (TD-054).
pub const RETRY_MAX_ATTEMPTS: u32 = 3;

/// Exponential-backoff base: attempt `n` sleeps within
/// `[0, min(base · 2^n, cap)]` (full jitter). Parity with the official
/// Anthropic SDK defaults (initial 0.5s).
pub const BACKOFF_BASE: Duration = Duration::from_millis(500);

/// Backoff ceiling. Parity with the official Anthropic SDK defaults (8s).
pub const BACKOFF_CAP: Duration = Duration::from_secs(8);

/// Retry policy: attempt bound, backoff envelope, and the jitter seed.
///
/// Production callers use [`RetryPolicy::default`] (time-seeded jitter);
/// tests pin determinism via [`RetryPolicy::with_seed`] under
/// `tokio::time::pause()` (CLAUDE.md §5 reproducibility).
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Total attempts before the typed error surfaces.
    pub max_attempts: u32,
    /// Exponential-backoff base.
    pub base: Duration,
    /// Backoff ceiling.
    pub cap: Duration,
    seed: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: RETRY_MAX_ATTEMPTS,
            base: BACKOFF_BASE,
            cap: BACKOFF_CAP,
            seed: seed_from_time(),
        }
    }
}

impl RetryPolicy {
    /// Deterministic-jitter policy for tests (fixed xorshift seed).
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self {
            seed,
            ..Self::default()
        }
    }
}

/// Production jitter seed — `SystemTime` nanos. No `rand` dependency
/// (M09.5.D scope lock: std-only jitter); xorshift64* below provides the
/// uniform spread.
fn seed_from_time() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0x9E37_79B9_7F4A_7C15, |d| {
            #[allow(
                clippy::cast_possible_truncation,
                reason = "nanos truncation is exactly the entropy mix wanted for a jitter seed"
            )]
            let nanos = d.as_nanos() as u64;
            nanos | 1
        })
}

/// xorshift64* — a tiny std-only PRNG; statistical quality is ample for
/// backoff jitter (this is NOT a cryptographic source and must never
/// become one).
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

/// Uniform draw in `[0, max]` (full jitter per the AWS backoff archetype).
fn jitter(rng: &mut XorShift64, max: Duration) -> Duration {
    if max.is_zero() {
        return Duration::ZERO;
    }
    #[allow(
        clippy::cast_possible_truncation,
        reason = "max is bounded by BACKOFF_CAP (8s) — nanos fit u64 by orders of magnitude"
    )]
    let nanos = max.as_nanos() as u64;
    Duration::from_nanos(rng.next() % (nanos + 1))
}

/// Verdict of [`classify`]: whether — and how — an error is retried.
enum RetryClass {
    /// Not retryable; surface immediately.
    Never,
    /// Retryable with jittered backoff only.
    Backoff,
    /// Retryable; the provider's `retry-after` seconds floor the sleep.
    FlooredBackoff(u64),
}

/// The retry matrix's RETRY column.
///
/// 529 reaches here as [`ProviderError::Overloaded`] (mapped by
/// [`map_http_error`]), so the `Api` arm carries 500/502/503 only.
const fn classify(err: &ProviderError) -> RetryClass {
    match err {
        ProviderError::RateLimit { retry_after_secs }
        | ProviderError::Overloaded {
            retry_after_secs: Some(retry_after_secs),
        } => RetryClass::FlooredBackoff(*retry_after_secs),
        ProviderError::Overloaded {
            retry_after_secs: None,
        }
        | ProviderError::Api {
            status: 500 | 502 | 503,
            ..
        } => RetryClass::Backoff,
        _ => RetryClass::Never,
    }
}

/// Run `op` under the bounded retry policy.
///
/// Retryable failures sleep `max(full-jitter backoff, retry-after)` and
/// re-attempt; the `max_attempts`-th failure (or any non-retryable
/// failure) surfaces the typed error unchanged. Sleeps ride `tokio::time`
/// (pause-testable).
///
/// # Errors
///
/// The final attempt's [`ProviderError`], or the first non-retryable one.
pub async fn execute<T, F, Fut>(policy: &RetryPolicy, mut op: F) -> Result<T, ProviderError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, ProviderError>>,
{
    let mut rng = XorShift64::new(policy.seed);
    let mut attempt: u32 = 0;
    loop {
        match op().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                attempt += 1;
                let floor = match classify(&err) {
                    RetryClass::Never => return Err(err),
                    RetryClass::Backoff => Duration::ZERO,
                    RetryClass::FlooredBackoff(secs) => Duration::from_secs(secs),
                };
                if attempt >= policy.max_attempts {
                    return Err(err);
                }
                let exp = policy
                    .base
                    .saturating_mul(2_u32.saturating_pow(attempt - 1))
                    .min(policy.cap);
                let delay = jitter(&mut rng, exp).max(floor);
                tracing::warn!(
                    attempt,
                    max_attempts = policy.max_attempts,
                    error = %err,
                    delay_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX),
                    "provider request failed; retrying after backoff (TD-054)"
                );
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Single mapping from a non-success HTTP response to the typed
/// [`ProviderError`].
///
/// Both request paths (`/v1/messages` streaming POST and `count_tokens`)
/// delegate here, replacing the previously duplicated in-wrapper blocks.
/// Existing semantics preserved byte-for-byte: 429 → `RateLimit` (header
/// default 60), 401/403 → `Auth`, other non-2xx → `Api`. New (TD-054):
/// HTTP 529 — or an `error.type == "overloaded_error"` body on any
/// status — maps to the typed `Overloaded`.
#[must_use]
pub fn map_http_error(status: u16, retry_after_secs: Option<u64>, body: &str) -> ProviderError {
    if status == 529 || body_is_overloaded(body) {
        return ProviderError::Overloaded { retry_after_secs };
    }
    match status {
        429 => ProviderError::RateLimit {
            retry_after_secs: retry_after_secs.unwrap_or(60),
        },
        401 | 403 => ProviderError::Auth,
        _ => ProviderError::Api {
            status,
            body: body.to_string(),
        },
    }
}

fn body_is_overloaded(body: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(body)
        .is_ok_and(|v| v["error"]["type"] == "overloaded_error")
}
