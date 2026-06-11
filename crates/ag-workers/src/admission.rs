//! Admission control outcomes.
//!
//! Every enqueue path classifies its result so callers (HTTP handlers, the CLI) can
//! return structured errors instead of blocking forever when capacity is exhausted.

use crate::queue::QueueError;

/// The outcome of an admission decision on an enqueue path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionOutcome {
    /// The job was accepted.
    Accepted,
    /// Rejected because the queue is at its depth limit.
    RejectedQueueFull,
    /// Rejected because the payload exceeds the size limit.
    RejectedPayloadTooLarge,
    /// Rejected by a per-queue admission rate limiter.
    ///
    /// RESERVED (RFC-0012 section 18, issue #113): this is RFC-sanctioned
    /// admission vocabulary and part of the `ag_workers_backpressure_total{reason}`
    /// label space, but it is intentionally **not produced by any admission path
    /// today**. The two implemented rejections are [`Self::RejectedQueueFull`]
    /// (depth) and [`Self::RejectedPayloadTooLarge`] (size). A per-queue rate
    /// limiter that yields this outcome changes the admission contract and is
    /// therefore gated behind a future RFC/ADR; until then the variant is kept
    /// reserved (not deleted) so the metric label space and the RFC vocabulary
    /// stay stable. Its `as_str` mapping is exercised by a unit test.
    RejectedRateLimited,
    /// Rejected because the payload was invalid.
    RejectedInvalidPayload,
}

impl AdmissionOutcome {
    /// Returns the stable string used for metrics labels and logs.
    pub fn as_str(self) -> &'static str {
        match self {
            AdmissionOutcome::Accepted => "accepted",
            AdmissionOutcome::RejectedQueueFull => "rejected_queue_full",
            AdmissionOutcome::RejectedPayloadTooLarge => "rejected_payload_too_large",
            AdmissionOutcome::RejectedRateLimited => "rejected_rate_limited",
            AdmissionOutcome::RejectedInvalidPayload => "rejected_invalid_payload",
        }
    }

    /// Returns `true` if the job was accepted.
    pub fn is_accepted(self) -> bool {
        matches!(self, AdmissionOutcome::Accepted)
    }

    /// Classifies a queue enqueue result into an admission outcome.
    pub fn from_result<T>(result: &Result<T, QueueError>) -> Self {
        match result {
            Ok(_) => AdmissionOutcome::Accepted,
            Err(QueueError::Full(_)) => AdmissionOutcome::RejectedQueueFull,
            Err(QueueError::PayloadTooLarge { .. }) => AdmissionOutcome::RejectedPayloadTooLarge,
            Err(QueueError::NotFound(_)) | Err(QueueError::Backend(_)) => {
                AdmissionOutcome::RejectedInvalidPayload
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::JobId;

    #[test]
    fn classifies_outcomes() {
        let ok: Result<(), QueueError> = Ok(());
        assert_eq!(
            AdmissionOutcome::from_result(&ok),
            AdmissionOutcome::Accepted
        );
        let full: Result<(), QueueError> = Err(QueueError::Full("q".into()));
        assert_eq!(
            AdmissionOutcome::from_result(&full),
            AdmissionOutcome::RejectedQueueFull
        );
        let big: Result<(), QueueError> = Err(QueueError::PayloadTooLarge {
            actual: 10,
            limit: 2,
        });
        assert_eq!(
            AdmissionOutcome::from_result(&big),
            AdmissionOutcome::RejectedPayloadTooLarge
        );
        let nf: Result<(), QueueError> = Err(QueueError::NotFound(JobId::new()));
        assert!(!AdmissionOutcome::from_result(&nf).is_accepted());
    }

    #[test]
    fn outcome_strings_are_stable() {
        assert_eq!(AdmissionOutcome::Accepted.as_str(), "accepted");
        assert_eq!(
            AdmissionOutcome::RejectedQueueFull.as_str(),
            "rejected_queue_full"
        );
        assert_eq!(
            AdmissionOutcome::RejectedPayloadTooLarge.as_str(),
            "rejected_payload_too_large"
        );
        assert_eq!(
            AdmissionOutcome::RejectedRateLimited.as_str(),
            "rejected_rate_limited"
        );
        assert_eq!(
            AdmissionOutcome::RejectedInvalidPayload.as_str(),
            "rejected_invalid_payload"
        );
        assert!(AdmissionOutcome::Accepted.is_accepted());
        assert!(!AdmissionOutcome::RejectedRateLimited.is_accepted());
    }

    /// RESERVED contract (issue #113): no `from_result` mapping yields
    /// `RejectedRateLimited`. The variant exists only as RFC-0012 section 18
    /// vocabulary and metric label space until a per-queue rate limiter is added
    /// behind a future RFC. This test fails loudly if a mapping starts producing
    /// it without the contract being revisited.
    #[test]
    fn rate_limited_is_reserved_and_never_produced_by_from_result() {
        let cases: [Result<(), QueueError>; 4] = [
            Ok(()),
            Err(QueueError::Full("q".into())),
            Err(QueueError::PayloadTooLarge {
                actual: 10,
                limit: 2,
            }),
            Err(QueueError::NotFound(JobId::new())),
        ];
        for case in &cases {
            assert_ne!(
                AdmissionOutcome::from_result(case),
                AdmissionOutcome::RejectedRateLimited,
                "RejectedRateLimited must stay reserved (not produced)"
            );
        }
    }
}
