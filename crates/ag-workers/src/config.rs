//! Runtime configuration.

use std::time::Duration;

use crate::backoff::RetryPolicy;

/// Default poison-guard threshold: a job leased more than this many times without an
/// outcome is dead-lettered before execution (crash-loop protection).
pub const DEFAULT_POISON_GUARD_ATTEMPTS: u32 = 3;

/// Configuration shared by the dispatch loop and worker pools.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// A job whose attempt count exceeds this is dead-lettered before executing. Keep
    /// this at or below the retry policy's `max_attempts`; it is the closed loop that
    /// turns an infinite crash loop into a bounded dead-letter entry under
    /// `panic = "abort"`.
    pub poison_guard_attempts: u32,
    /// How long a worker waits before polling again when a queue is empty.
    pub poll_interval: Duration,
    /// Maximum number of jobs a worker leases per poll.
    pub lease_batch: usize,
    /// Retry/backoff policy applied to retriable failures.
    pub retry_policy: RetryPolicy,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            poison_guard_attempts: DEFAULT_POISON_GUARD_ATTEMPTS,
            poll_interval: Duration::from_millis(100),
            lease_batch: 10,
            retry_policy: RetryPolicy::default(),
        }
    }
}
