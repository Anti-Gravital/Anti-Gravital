//! Successful-dispatch outcomes.
//!
//! A handler that returns `Ok(..)` still chooses what should happen to the job. This
//! is distinct from [`crate::WorkerError`], which is the failure path.

use std::time::Duration;

/// How long to wait before a job becomes eligible for its next attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryDelay {
    /// Use the queue/handler retry policy to compute the delay.
    Policy,
    /// Override the policy with an explicit fixed delay.
    After(Duration),
}

/// Reason a job is moved to the dead-letter queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeadLetterReason {
    /// All retry attempts were exhausted.
    AttemptsExhausted,
    /// The handler classified the failure as permanent.
    Permanent(String),
    /// The payload could not be decoded or its version was unknown.
    InvalidPayload(String),
    /// The job exceeded its maximum age before succeeding.
    Expired,
    /// The poison guard tripped before execution (crash-loop protection).
    PoisonGuard,
    /// The handler explicitly requested dead-lettering.
    Requested(String),
}

impl DeadLetterReason {
    /// Returns the stable string used for metrics labels and logs.
    pub fn as_str(&self) -> &'static str {
        match self {
            DeadLetterReason::AttemptsExhausted => "attempts_exhausted",
            DeadLetterReason::Permanent(_) => "permanent",
            DeadLetterReason::InvalidPayload(_) => "invalid_payload",
            DeadLetterReason::Expired => "expired",
            DeadLetterReason::PoisonGuard => "poison_guard",
            DeadLetterReason::Requested(_) => "requested",
        }
    }
}

/// The result a handler chooses for a job it processed without erroring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobOutcome {
    /// The job is done; acknowledge it.
    Complete,
    /// Retry the job, optionally overriding the policy delay.
    Retry {
        /// When the job should become eligible again.
        delay: RetryDelay,
    },
    /// Intentionally do nothing further; recorded, not an error.
    Discard {
        /// Why the job was discarded.
        reason: String,
    },
    /// Skip remaining attempts and preserve the job in the dead-letter queue.
    DeadLetter {
        /// Why the job was dead-lettered.
        reason: String,
    },
}

impl JobOutcome {
    /// Convenience constructor for a retry that uses the configured policy.
    pub fn retry() -> Self {
        JobOutcome::Retry {
            delay: RetryDelay::Policy,
        }
    }

    /// Convenience constructor for a retry after an explicit delay.
    pub fn retry_after(delay: Duration) -> Self {
        JobOutcome::Retry {
            delay: RetryDelay::After(delay),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_helpers() {
        assert_eq!(
            JobOutcome::retry(),
            JobOutcome::Retry {
                delay: RetryDelay::Policy
            }
        );
        assert_eq!(
            JobOutcome::retry_after(Duration::from_secs(5)),
            JobOutcome::Retry {
                delay: RetryDelay::After(Duration::from_secs(5))
            }
        );
    }

    #[test]
    fn dead_letter_reason_labels_are_stable() {
        assert_eq!(DeadLetterReason::PoisonGuard.as_str(), "poison_guard");
        assert_eq!(
            DeadLetterReason::Permanent("x".into()).as_str(),
            "permanent"
        );
    }
}
