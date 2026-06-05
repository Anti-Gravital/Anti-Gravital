//! Traffic shaping for the native MTA (Phase 4.6-B).
//!
//! Two independent limits are applied per destination `site_name` (the MX
//! rollup from [`super::resolve`]): a **message rate** (token bucket) and a
//! **maximum number of simultaneous connections** (semaphore). Limits are a
//! default plus optional per-site overrides, so a provider that throttles
//! aggressively can be given a slower rate without touching the rest.
//!
//! The rate limiter is driven by an explicit clock (`Instant`) so it is
//! deterministic and fully unit-tested; the connection limiter uses a Tokio
//! semaphore so the worker can await a free slot.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// Shaping limits for one destination.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapingLimits {
    /// Maximum messages per second. `0.0` means unlimited.
    pub max_messages_per_second: f64,
    /// Maximum simultaneous connections. `0` means unlimited.
    pub max_connections: u32,
}

impl ShapingLimits {
    /// Unlimited on both axes.
    pub const UNLIMITED: ShapingLimits = ShapingLimits {
        max_messages_per_second: 0.0,
        max_connections: 0,
    };

    /// Limits with the given message rate and connection cap.
    pub fn new(max_messages_per_second: f64, max_connections: u32) -> Self {
        Self {
            max_messages_per_second,
            max_connections,
        }
    }
}

impl Default for ShapingLimits {
    fn default() -> Self {
        ShapingLimits::UNLIMITED
    }
}

/// A default plus per-`site_name` shaping overrides.
#[derive(Debug, Clone, Default)]
pub struct ShapingConfig {
    default: ShapingLimits,
    per_site: HashMap<String, ShapingLimits>,
}

impl ShapingConfig {
    /// Creates a config with the given default limits and no overrides.
    pub fn new(default: ShapingLimits) -> Self {
        Self {
            default,
            per_site: HashMap::new(),
        }
    }

    /// Adds or replaces the limits for a specific `site_name`.
    pub fn with_site(mut self, site_name: impl Into<String>, limits: ShapingLimits) -> Self {
        self.per_site.insert(site_name.into(), limits);
        self
    }

    /// Returns the effective limits for a `site_name` (override or default).
    pub fn limits_for(&self, site_name: &str) -> ShapingLimits {
        self.per_site
            .get(site_name)
            .copied()
            .unwrap_or(self.default)
    }
}

/// A token bucket: refills at `rate` tokens/second up to `capacity`, consuming
/// one token per message. Deterministic given the clock passed to `acquire`.
#[derive(Debug)]
struct TokenBucket {
    capacity: f64,
    tokens: f64,
    rate: f64,
    last: Instant,
}

impl TokenBucket {
    fn new(rate: f64, now: Instant) -> Self {
        // Allow a one-second burst, but at least a single message so a slow
        // rate (< 1/s) can still send one message immediately.
        let capacity = rate.max(1.0);
        Self {
            capacity,
            tokens: capacity,
            rate,
            last: now,
        }
    }

    /// Tries to consume one token. Returns `Duration::ZERO` if a message may be
    /// sent now (a token was consumed); otherwise the time to wait until one
    /// token is available (nothing is consumed).
    fn acquire(&mut self, now: Instant) -> Duration {
        let elapsed = now.saturating_duration_since(self.last).as_secs_f64();
        self.last = now;
        self.tokens = (self.tokens + elapsed * self.rate).min(self.capacity);

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            Duration::ZERO
        } else {
            let missing = 1.0 - self.tokens;
            Duration::from_secs_f64(missing / self.rate)
        }
    }
}

/// Applies per-site rate and connection limits.
#[derive(Debug)]
pub struct Shaper {
    config: ShapingConfig,
    buckets: Mutex<HashMap<String, TokenBucket>>,
    semaphores: Mutex<HashMap<String, Arc<Semaphore>>>,
}

impl Shaper {
    /// Builds a shaper from its configuration.
    pub fn new(config: ShapingConfig) -> Self {
        Self {
            config,
            buckets: Mutex::new(HashMap::new()),
            semaphores: Mutex::new(HashMap::new()),
        }
    }

    /// Effective limits for a destination.
    pub fn limits_for(&self, site_name: &str) -> ShapingLimits {
        self.config.limits_for(site_name)
    }

    /// Returns how long the caller must wait before sending one message to
    /// `site_name`. `Duration::ZERO` means "send now"; the rate token is
    /// consumed only when the return value is zero.
    pub fn acquire_rate(&self, site_name: &str, now: Instant) -> Duration {
        let rate = self.config.limits_for(site_name).max_messages_per_second;
        if rate <= 0.0 {
            return Duration::ZERO;
        }
        let mut buckets = self.buckets.lock().expect("shaper bucket mutex poisoned");
        let bucket = buckets
            .entry(site_name.to_owned())
            .or_insert_with(|| TokenBucket::new(rate, now));
        bucket.acquire(now)
    }

    /// Acquires a connection permit for `site_name`, awaiting a free slot when
    /// the cap is reached. Returns `None` when the destination is unlimited
    /// (no permit needed). Drop the permit to release the slot.
    pub async fn acquire_connection(&self, site_name: &str) -> Option<OwnedSemaphorePermit> {
        let max = self.config.limits_for(site_name).max_connections;
        if max == 0 {
            return None;
        }
        let semaphore = {
            let mut sems = self
                .semaphores
                .lock()
                .expect("shaper semaphore mutex poisoned");
            Arc::clone(
                sems.entry(site_name.to_owned())
                    .or_insert_with(|| Arc::new(Semaphore::new(max as usize))),
            )
        };
        // The semaphore is never closed, so acquire cannot fail.
        Some(
            semaphore
                .acquire_owned()
                .await
                .expect("shaper semaphore closed"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_and_override() {
        let cfg = ShapingConfig::new(ShapingLimits::new(10.0, 4))
            .with_site("aspmx.l.google.com", ShapingLimits::new(2.0, 1));
        assert_eq!(
            cfg.limits_for("unknown.example").max_messages_per_second,
            10.0
        );
        assert_eq!(cfg.limits_for("aspmx.l.google.com").max_connections, 1);
    }

    #[test]
    fn unlimited_rate_never_waits() {
        let shaper = Shaper::new(ShapingConfig::new(ShapingLimits::UNLIMITED));
        let t0 = Instant::now();
        for _ in 0..1000 {
            assert_eq!(shaper.acquire_rate("site", t0), Duration::ZERO);
        }
    }

    #[test]
    fn rate_limit_consumes_burst_then_waits() {
        // 2 msg/s => capacity 2. Two immediate, third must wait ~0.5s.
        let shaper = Shaper::new(ShapingConfig::new(ShapingLimits::new(2.0, 0)));
        let t0 = Instant::now();
        assert_eq!(shaper.acquire_rate("site", t0), Duration::ZERO);
        assert_eq!(shaper.acquire_rate("site", t0), Duration::ZERO);
        let wait = shaper.acquire_rate("site", t0);
        assert!(wait > Duration::ZERO);
        // ~0.5s to refill one token at 2/s.
        assert!((wait.as_secs_f64() - 0.5).abs() < 1e-6, "wait = {wait:?}");
    }

    #[test]
    fn rate_limit_refills_over_time() {
        let shaper = Shaper::new(ShapingConfig::new(ShapingLimits::new(1.0, 0)));
        let t0 = Instant::now();
        assert_eq!(shaper.acquire_rate("site", t0), Duration::ZERO); // capacity 1 consumed
        assert!(shaper.acquire_rate("site", t0) > Duration::ZERO); // empty
                                                                   // After 1 second exactly one token refilled.
        assert_eq!(
            shaper.acquire_rate("site", t0 + Duration::from_secs(1)),
            Duration::ZERO
        );
    }

    #[test]
    fn slow_rate_allows_one_immediately() {
        // 0.5 msg/s (one every 2s): capacity clamped to 1 so the first sends now.
        let shaper = Shaper::new(ShapingConfig::new(ShapingLimits::new(0.5, 0)));
        let t0 = Instant::now();
        assert_eq!(shaper.acquire_rate("site", t0), Duration::ZERO);
        let wait = shaper.acquire_rate("site", t0);
        assert!((wait.as_secs_f64() - 2.0).abs() < 1e-6, "wait = {wait:?}");
    }

    #[test]
    fn per_site_buckets_are_independent() {
        let shaper = Shaper::new(ShapingConfig::new(ShapingLimits::new(1.0, 0)));
        let t0 = Instant::now();
        assert_eq!(shaper.acquire_rate("a", t0), Duration::ZERO);
        // A different site has its own full bucket.
        assert_eq!(shaper.acquire_rate("b", t0), Duration::ZERO);
    }

    #[tokio::test]
    async fn connection_limit_unlimited_returns_no_permit() {
        let shaper = Shaper::new(ShapingConfig::new(ShapingLimits::UNLIMITED));
        assert!(shaper.acquire_connection("site").await.is_none());
    }

    #[tokio::test]
    async fn connection_limit_caps_concurrency() {
        let shaper = Shaper::new(ShapingConfig::new(ShapingLimits::new(0.0, 1)));
        let permit = shaper.acquire_connection("site").await;
        assert!(permit.is_some());
        // The cap is one; a second acquire would block, so a timeout elapses.
        let blocked =
            tokio::time::timeout(Duration::from_millis(50), shaper.acquire_connection("site"))
                .await;
        assert!(blocked.is_err(), "second permit must block at the cap");
        // Releasing the first frees the slot.
        drop(permit);
        let next =
            tokio::time::timeout(Duration::from_millis(50), shaper.acquire_connection("site"))
                .await;
        assert!(next.is_ok() && next.unwrap().is_some());
    }
}
