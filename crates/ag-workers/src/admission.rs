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
    /// Rejected by a rate limiter.
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
}
