//! M09.5.D red — unit targets for the two NEW covered modules
//! (`providers::retry`, `providers::stream_guard`) under
//! `tokio::time::pause()` (CLAUDE.md §5 reproducibility — every backoff
//! assertion runs on the virtual clock).
//!
//! Red expectation: this file FAILS TO COMPILE (unresolved imports — the
//! modules do not exist yet; the §5-endorsed hard-fail red). Each test
//! crate under `tests/` compiles independently, so this compile-red does
//! not mask `provider_resilience.rs`'s behavioral red.
//!
//! These are the in-package mutation-killer targets the blocking gate
//! relies on (riders 1 and 4 + the C-stage lesson: pin values
//! in-package — cargo-mutants only counts the mutated package's tests).

use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::{Stream, StreamExt};
use runtime_main::providers::retry::{
    execute, map_http_error, RetryPolicy, BACKOFF_BASE, BACKOFF_CAP, RETRY_MAX_ATTEMPTS,
};
use runtime_main::providers::stream_guard::{with_idle_timeout, IDLE_TIMEOUT};
use runtime_main::providers::{ProviderError, ProviderEvent};

// ── value pins (in-package mutation killers on the constants) ──

#[test]
fn retry_constants_are_pinned() {
    assert_eq!(RETRY_MAX_ATTEMPTS, 3, "three attempts total (TD-054)");
    assert_eq!(BACKOFF_BASE, Duration::from_millis(500));
    assert_eq!(BACKOFF_CAP, Duration::from_secs(8));
}

#[test]
fn idle_timeout_is_pinned_at_90s() {
    assert_eq!(IDLE_TIMEOUT, Duration::from_secs(90), "TD-054 idle window");
}

// ── retry::execute ──

fn counting_op<T: Clone + Send + 'static>(
    calls: &Arc<AtomicUsize>,
    results: Vec<Result<T, ProviderError>>,
) -> impl FnMut() -> Pin<Box<dyn std::future::Future<Output = Result<T, ProviderError>> + Send>> {
    let calls = Arc::clone(calls);
    let mut queue = results.into_iter();
    move || {
        calls.fetch_add(1, Ordering::SeqCst);
        let next = queue.next().expect("op called more times than scripted");
        Box::pin(async move { next })
    }
}

fn overloaded(retry_after_secs: Option<u64>) -> ProviderError {
    ProviderError::Overloaded { retry_after_secs }
}

fn api(status: u16) -> ProviderError {
    ProviderError::Api {
        status,
        body: String::new(),
    }
}

/// D.4 scenario 3 + rider 4: `retry-after` paces the sleeps, and the
/// exhausted 429 surfaces as the TYPED RateLimit. With retry-after 7s
/// (> any jittered backoff at base 500ms / cap 8s for attempts 0..2),
/// both inter-attempt sleeps are exactly 7s → 14s total virtual time.
#[tokio::test(start_paused = true)]
async fn retry_after_paces_sleeps_and_exhaustion_is_typed_rate_limit() {
    let calls = Arc::new(AtomicUsize::new(0));
    let op = counting_op::<()>(
        &calls,
        vec![
            Err(ProviderError::RateLimit {
                retry_after_secs: 7,
            }),
            Err(ProviderError::RateLimit {
                retry_after_secs: 7,
            }),
            Err(ProviderError::RateLimit {
                retry_after_secs: 7,
            }),
        ],
    );
    let start = tokio::time::Instant::now();
    let result = execute(&RetryPolicy::with_seed(42), op).await;
    let elapsed = start.elapsed();

    assert_eq!(calls.load(Ordering::SeqCst), 3, "three attempts total");
    assert!(
        matches!(
            result,
            Err(ProviderError::RateLimit {
                retry_after_secs: 7
            })
        ),
        "exhaustion must surface the typed RateLimit (rider 4)"
    );
    assert!(
        elapsed >= Duration::from_secs(14),
        "two retry-after-honoring sleeps of 7s expected, virtual elapsed {elapsed:?}"
    );
    assert!(
        elapsed < Duration::from_secs(16),
        "retry-after must not be double-applied, virtual elapsed {elapsed:?}"
    );
}

/// Overloaded with a retry-after floor: each sleep is max(jitter, 1s).
#[tokio::test(start_paused = true)]
async fn overloaded_retry_after_is_honored_as_floor() {
    let calls = Arc::new(AtomicUsize::new(0));
    let op = counting_op::<()>(
        &calls,
        vec![
            Err(overloaded(Some(1))),
            Err(overloaded(Some(1))),
            Err(overloaded(Some(1))),
        ],
    );
    let start = tokio::time::Instant::now();
    let result = execute(&RetryPolicy::with_seed(7), op).await;
    let elapsed = start.elapsed();

    assert_eq!(calls.load(Ordering::SeqCst), 3);
    assert!(
        matches!(
            result,
            Err(ProviderError::Overloaded {
                retry_after_secs: Some(1)
            })
        ),
        "exhaustion must surface the typed Overloaded"
    );
    assert!(
        elapsed >= Duration::from_secs(2),
        "retry-after floor (1s × 2 sleeps) not honored: {elapsed:?}"
    );
}

/// Full jitter stays inside the exponential envelope: with no retry-after,
/// sleep n is uniform in [0, min(base·2^n, cap)] — for attempts 0 and 1
/// that bounds total virtual time at 0.5s + 1.0s.
#[tokio::test(start_paused = true)]
async fn full_jitter_stays_within_exponential_bounds() {
    let calls = Arc::new(AtomicUsize::new(0));
    let op = counting_op::<()>(&calls, vec![Err(api(500)), Err(api(500)), Err(api(500))]);
    let start = tokio::time::Instant::now();
    let result = execute(&RetryPolicy::with_seed(1234), op).await;
    let elapsed = start.elapsed();

    assert_eq!(calls.load(Ordering::SeqCst), 3);
    assert!(matches!(
        result,
        Err(ProviderError::Api { status: 500, .. })
    ));
    assert!(
        elapsed <= Duration::from_millis(1500),
        "full-jitter sleeps must stay within base·2^n (≤0.5s + ≤1.0s), got {elapsed:?}"
    );
}

/// A transient failure followed by success returns Ok — and stops retrying.
#[tokio::test(start_paused = true)]
async fn transient_failure_then_success_returns_ok() {
    let calls = Arc::new(AtomicUsize::new(0));
    let op = counting_op::<u8>(&calls, vec![Err(api(503)), Ok(7)]);
    let result = execute(&RetryPolicy::with_seed(9), op).await;
    assert_eq!(result.expect("second attempt succeeds"), 7);
    assert_eq!(calls.load(Ordering::SeqCst), 2, "no retry after success");
}

/// The retry matrix's NEVER column: Auth and non-429 4xx fail fast.
#[tokio::test(start_paused = true)]
async fn non_retryable_errors_fail_fast() {
    for err in [ProviderError::Auth, api(400), api(404)] {
        let calls = Arc::new(AtomicUsize::new(0));
        let label = err.to_string();
        let op = counting_op::<()>(&calls, vec![Err(err)]);
        let result = execute(&RetryPolicy::with_seed(3), op).await;
        assert!(result.is_err());
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "{label}: must not be retried"
        );
    }
}

/// The retry matrix's RETRY column: 429, 500, 502, 503, 529 all get the
/// full three attempts.
#[tokio::test(start_paused = true)]
async fn retryable_statuses_get_three_attempts() {
    let retryables: Vec<fn() -> ProviderError> = vec![
        || ProviderError::RateLimit {
            retry_after_secs: 0,
        },
        || api(500),
        || api(502),
        || api(503),
        || overloaded(None),
    ];
    for make in retryables {
        let calls = Arc::new(AtomicUsize::new(0));
        let label = make().to_string();
        let op = counting_op::<()>(&calls, vec![Err(make()), Err(make()), Err(make())]);
        let result = execute(&RetryPolicy::with_seed(5), op).await;
        assert!(result.is_err());
        assert_eq!(
            calls.load(Ordering::SeqCst),
            3,
            "{label}: expected the full bounded retry"
        );
    }
}

// ── map_http_error (the single mapping both request paths delegate to) ──

#[test]
fn map_http_error_preserves_existing_mappings() {
    assert!(matches!(
        map_http_error(429, Some(30), ""),
        ProviderError::RateLimit {
            retry_after_secs: 30
        }
    ));
    assert!(matches!(
        map_http_error(429, None, ""),
        ProviderError::RateLimit {
            retry_after_secs: 60
        }
    ));
    assert!(matches!(map_http_error(401, None, ""), ProviderError::Auth));
    assert!(matches!(map_http_error(403, None, ""), ProviderError::Auth));
    assert!(matches!(
        map_http_error(500, None, "boom"),
        ProviderError::Api { status: 500, .. }
    ));
    assert!(matches!(
        map_http_error(400, None, ""),
        ProviderError::Api { status: 400, .. }
    ));
}

#[test]
fn map_http_error_maps_529_and_overloaded_body_to_overloaded() {
    assert!(matches!(
        map_http_error(529, Some(5), ""),
        ProviderError::Overloaded {
            retry_after_secs: Some(5)
        }
    ));
    assert!(matches!(
        map_http_error(529, None, ""),
        ProviderError::Overloaded {
            retry_after_secs: None
        }
    ));
    // `error.type == "overloaded_error"` in the body wins even when the
    // status is not 529.
    let body = r#"{"type":"error","error":{"type":"overloaded_error","message":"x"}}"#;
    assert!(matches!(
        map_http_error(503, None, body),
        ProviderError::Overloaded { .. }
    ));
}

#[test]
fn overloaded_display_is_plain_language() {
    let display = overloaded(Some(5)).to_string().to_lowercase();
    assert!(display.contains("overloaded"), "got: {display}");
}

// ── stream_guard ──

fn text(n: u32) -> ProviderEvent {
    ProviderEvent::TextDelta {
        text: n.to_string(),
    }
}

/// Rider 1(a): the idle timer RESETS on every yielded event — a stream
/// pacing events at 60s (each under the 90s window) yields all of them;
/// the timeout fires 90s after the LAST event, not 90s into the stream.
#[tokio::test(start_paused = true)]
async fn idle_timer_resets_on_each_event() {
    let inner = futures::stream::unfold(0u32, |n| async move {
        if n < 3 {
            tokio::time::sleep(Duration::from_secs(60)).await;
            Some((text(n), n + 1))
        } else {
            std::future::pending::<()>().await;
            unreachable!()
        }
    });
    let start = tokio::time::Instant::now();
    let guarded = with_idle_timeout(Box::pin(inner), IDLE_TIMEOUT);
    let events: Vec<ProviderEvent> = guarded.collect().await;
    let elapsed = start.elapsed();

    assert_eq!(events.len(), 4, "three deltas then the timeout event");
    for (i, event) in events.iter().take(3).enumerate() {
        assert!(
            matches!(event, ProviderEvent::TextDelta { .. }),
            "event {i} should be a TextDelta (60s < 90s window), got {event:?}"
        );
    }
    assert!(
        matches!(&events[3], ProviderEvent::Error { code, .. } if code == "provider_idle_timeout"),
        "final event must be the typed timeout, got {:?}",
        events[3]
    );
    // 3 × 60s of healthy pacing + 90s of silence — a non-resetting
    // (total-deadline) implementation would have died at 90s with one event.
    assert!(
        elapsed >= Duration::from_secs(270) && elapsed < Duration::from_secs(271),
        "expected ~270s virtual (timer resets per event), got {elapsed:?}"
    );
}

/// Counting stream that never yields: pins that the guard does not poll
/// the inner stream again after the timeout fired (rider 1(b)).
struct CountingPending {
    polls: Arc<AtomicUsize>,
}

impl Stream for CountingPending {
    type Item = ProviderEvent;
    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<ProviderEvent>> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        Poll::Pending
    }
}

#[tokio::test(start_paused = true)]
async fn guard_is_fused_after_timeout() {
    let polls = Arc::new(AtomicUsize::new(0));
    let inner = CountingPending {
        polls: Arc::clone(&polls),
    };
    let mut guarded = Box::pin(with_idle_timeout(inner, IDLE_TIMEOUT));

    let first = guarded.next().await;
    assert!(
        matches!(
            &first,
            Some(ProviderEvent::Error { code, message })
                if code == "provider_idle_timeout" && message.contains("provider idle timeout")
        ),
        "expected the typed timeout event, got {first:?}"
    );
    let polls_at_timeout = polls.load(Ordering::SeqCst);
    assert!(polls_at_timeout >= 1, "inner must have been polled");

    assert!(guarded.next().await.is_none(), "fused: ends after timeout");
    assert!(guarded.next().await.is_none(), "fused: stays ended");
    assert_eq!(
        polls.load(Ordering::SeqCst),
        polls_at_timeout,
        "inner stream must never be polled again after the timeout (rider 1b)"
    );
}
