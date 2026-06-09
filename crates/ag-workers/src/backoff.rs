//! Retry policy and backoff computation.
//!
//! Deterministic, observable, configurable. Exponential backoff with an optional
//! jitter and a hard `max_delay` ceiling; an optional `max_age` bounds how long a job
//! may keep retrying before it transitions to `expired`. Ported and generalized from
//! the `ag-mail` queue's exponential backoff.

use std::time::Duration;

use chrono::{DateTime, Utc};

/// Configuration of retry behavior for a job or queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Maximum number of attempts before the job is dead-lettered.
    pub max_attempts: u32,
    /// Base delay used as the first backoff step.
    pub base_delay: Duration,
    /// Multiplier raised to the power of the attempt number.
    pub backoff_factor: u32,
    /// Hard ceiling on the computed delay.
    pub max_delay: Duration,
    /// Whether to apply jitter to the computed delay.
    pub jitter: bool,
    /// Optional maximum age after which a job is expired instead of retried.
    pub max_age: Option<Duration>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_secs(1),
            backoff_factor: 2,
            max_delay: Duration::from_secs(300),
            jitter: false,
            max_age: Some(Duration::from_secs(24 * 60 * 60)),
        }
    }
}

impl RetryPolicy {
    /// Computes the (un-jittered) delay for a 0-based attempt, capped at `max_delay`.
    pub fn delay_for(&self, attempt: u32) -> Duration {
        let multiplier = self.backoff_factor.saturating_pow(attempt);
        let raw = self.base_delay.saturating_mul(multiplier);
        raw.min(self.max_delay)
    }

    /// Computes the delay applying full jitter, given a `rand01` value in `[0.0, 1.0)`.
    ///
    /// Taking the random source as an argument keeps this function deterministic and
    /// unit-testable. With jitter disabled, returns [`delay_for`](RetryPolicy::delay_for)
    /// unchanged. The result never exceeds `max_delay`.
    pub fn delay_for_jittered(&self, attempt: u32, rand01: f64) -> Duration {
        let capped = self.delay_for(attempt);
        if !self.jitter {
            return capped;
        }
        let frac = rand01.clamp(0.0, 1.0);
        let scaled = capped.as_secs_f64() * frac;
        Duration::from_secs_f64(scaled).min(self.max_delay)
    }

    /// Returns the timestamp at which a job becomes eligible again after `attempt`.
    pub fn next_available_at(&self, now: DateTime<Utc>, attempt: u32) -> DateTime<Utc> {
        let delay = self.delay_for(attempt);
        now + chrono::Duration::from_std(delay).unwrap_or_else(|_| chrono::Duration::zero())
    }

    /// Returns `true` if a job created at `created_at` has exceeded `max_age` by `now`.
    pub fn is_expired(&self, created_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
        match self.max_age {
            Some(max_age) => {
                let age = now.signed_duration_since(created_at);
                match chrono::Duration::from_std(max_age) {
                    Ok(limit) => age > limit,
                    Err(_) => false,
                }
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> RetryPolicy {
        RetryPolicy {
            max_attempts: 5,
            base_delay: Duration::from_millis(100),
            backoff_factor: 2,
            max_delay: Duration::from_secs(1),
            jitter: false,
            max_age: Some(Duration::from_secs(10)),
        }
    }

    #[test]
    fn delay_grows_exponentially_then_caps() {
        let p = policy();
        assert_eq!(p.delay_for(0), Duration::from_millis(100));
        assert_eq!(p.delay_for(1), Duration::from_millis(200));
        assert_eq!(p.delay_for(2), Duration::from_millis(400));
        assert_eq!(p.delay_for(3), Duration::from_millis(800));
        // 1600ms would exceed max_delay (1s); capped.
        assert_eq!(p.delay_for(4), Duration::from_secs(1));
        assert_eq!(p.delay_for(50), Duration::from_secs(1));
    }

    #[test]
    fn jitter_never_exceeds_cap_and_disabled_is_identity() {
        let mut p = policy();
        // Disabled jitter is identity.
        assert_eq!(p.delay_for_jittered(2, 0.99), p.delay_for(2));
        // Enabled jitter stays within [0, capped].
        p.jitter = true;
        for r in [0.0, 0.5, 0.999] {
            let d = p.delay_for_jittered(4, r);
            assert!(d <= Duration::from_secs(1));
        }
        assert_eq!(p.delay_for_jittered(4, 0.0), Duration::ZERO);
    }

    #[test]
    fn expiry_respects_max_age() {
        let p = policy();
        let created = Utc::now();
        assert!(!p.is_expired(created, created));
        assert!(p.is_expired(created, created + chrono::Duration::seconds(11)));
    }

    #[test]
    fn next_available_at_advances() {
        let p = policy();
        let now = Utc::now();
        assert!(p.next_available_at(now, 1) > now);
    }
}
