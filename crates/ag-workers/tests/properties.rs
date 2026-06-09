//! Property-based invariants for the retry policy.

use std::time::Duration;

use ag_workers::RetryPolicy;
use proptest::prelude::*;

proptest! {
    // The computed delay never exceeds max_delay, for any attempt/base/factor.
    #[test]
    fn delay_never_exceeds_max_delay(
        base_ms in 1u64..5_000,
        factor in 1u32..10,
        max_ms in 1u64..60_000,
        attempt in 0u32..64,
    ) {
        let policy = RetryPolicy {
            max_attempts: 5,
            base_delay: Duration::from_millis(base_ms),
            backoff_factor: factor,
            max_delay: Duration::from_millis(max_ms),
            jitter: false,
            max_age: None,
        };
        let delay = policy.delay_for(attempt);
        prop_assert!(delay <= Duration::from_millis(max_ms));
    }

    // Jittered delay also respects the ceiling, for any random fraction.
    #[test]
    fn jittered_delay_within_ceiling(
        base_ms in 1u64..5_000,
        factor in 1u32..10,
        max_ms in 1u64..60_000,
        attempt in 0u32..64,
        rand01 in 0.0f64..1.0,
    ) {
        let policy = RetryPolicy {
            max_attempts: 5,
            base_delay: Duration::from_millis(base_ms),
            backoff_factor: factor,
            max_delay: Duration::from_millis(max_ms),
            jitter: true,
            max_age: None,
        };
        let delay = policy.delay_for_jittered(attempt, rand01);
        prop_assert!(delay <= Duration::from_millis(max_ms));
    }
}
