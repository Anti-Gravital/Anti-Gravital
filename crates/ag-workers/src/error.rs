//! Handler error taxonomy.
//!
//! Handlers return `Result<JobOutcome, WorkerError>`. The [`WorkerErrorKind`] drives
//! retry classification: transient failures obey the backoff policy, permanent ones go
//! straight to the dead-letter queue. Errors are explicit values, never panics — a
//! panic is a programmer bug and, under the workspace `panic = "abort"` profile, aborts
//! the process (see the poison guard in the runtime).

use std::fmt;

use crate::payload::PayloadError;

/// Classification of a handler failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorkerErrorKind {
    /// Transient failure; obey the backoff policy and retry.
    Retriable,
    /// Permanent failure; do not retry, dead-letter the job.
    Permanent,
    /// Shutdown/cancellation observed; release the job for re-lease.
    Cancelled,
    /// The per-job timeout elapsed; counts as an attempt.
    TimedOut,
    /// The payload could not be decoded or its version is unknown; dead-letter.
    InvalidPayload,
    /// A dependency was unavailable; retriable, and may inform queue-level hints.
    DependencyUnavailable,
}

impl WorkerErrorKind {
    /// Returns `true` if a job with this error should be retried (subject to limits).
    pub fn is_retriable(self) -> bool {
        matches!(
            self,
            WorkerErrorKind::Retriable
                | WorkerErrorKind::TimedOut
                | WorkerErrorKind::DependencyUnavailable
        )
    }

    /// Returns the stable string used for metrics labels and logs.
    pub fn as_str(self) -> &'static str {
        match self {
            WorkerErrorKind::Retriable => "retriable",
            WorkerErrorKind::Permanent => "permanent",
            WorkerErrorKind::Cancelled => "cancelled",
            WorkerErrorKind::TimedOut => "timed_out",
            WorkerErrorKind::InvalidPayload => "invalid_payload",
            WorkerErrorKind::DependencyUnavailable => "dependency_unavailable",
        }
    }
}

/// An error returned by a job handler, classified for retry handling.
#[derive(Debug)]
pub struct WorkerError {
    kind: WorkerErrorKind,
    message: String,
}

impl WorkerError {
    /// Builds an error of an explicit kind with a message.
    pub fn new(kind: WorkerErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// A transient failure that should be retried with backoff.
    pub fn retriable(message: impl Into<String>) -> Self {
        Self::new(WorkerErrorKind::Retriable, message)
    }

    /// A permanent failure that should be dead-lettered without retrying.
    pub fn permanent(message: impl Into<String>) -> Self {
        Self::new(WorkerErrorKind::Permanent, message)
    }

    /// A cancellation/shutdown signal; the job is released for re-lease.
    pub fn cancelled(message: impl Into<String>) -> Self {
        Self::new(WorkerErrorKind::Cancelled, message)
    }

    /// The per-job timeout elapsed; counts as an attempt.
    pub fn timed_out(message: impl Into<String>) -> Self {
        Self::new(WorkerErrorKind::TimedOut, message)
    }

    /// The payload was undecodable or its version unknown; dead-letter.
    pub fn invalid_payload(message: impl Into<String>) -> Self {
        Self::new(WorkerErrorKind::InvalidPayload, message)
    }

    /// A dependency was unavailable; retriable.
    pub fn dependency_unavailable(message: impl Into<String>) -> Self {
        Self::new(WorkerErrorKind::DependencyUnavailable, message)
    }

    /// Returns the classification of this error.
    pub fn kind(&self) -> WorkerErrorKind {
        self.kind
    }

    /// Returns the human-readable message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns `true` if the job should be retried (subject to attempt limits).
    pub fn is_retriable(&self) -> bool {
        self.kind.is_retriable()
    }
}

impl fmt::Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} error: {}", self.kind.as_str(), self.message)
    }
}

impl std::error::Error for WorkerError {}

impl From<PayloadError> for WorkerError {
    fn from(err: PayloadError) -> Self {
        // A payload that cannot be decoded or migrated is never retriable.
        WorkerError::invalid_payload(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retriable_kinds() {
        assert!(WorkerErrorKind::Retriable.is_retriable());
        assert!(WorkerErrorKind::TimedOut.is_retriable());
        assert!(WorkerErrorKind::DependencyUnavailable.is_retriable());
        assert!(!WorkerErrorKind::Permanent.is_retriable());
        assert!(!WorkerErrorKind::InvalidPayload.is_retriable());
        assert!(!WorkerErrorKind::Cancelled.is_retriable());
    }

    #[test]
    fn payload_error_maps_to_invalid_payload() {
        let err: WorkerError = PayloadError::Decode("bad".into()).into();
        assert_eq!(err.kind(), WorkerErrorKind::InvalidPayload);
        assert!(!err.is_retriable());
    }

    #[test]
    fn display_includes_kind_and_message() {
        let err = WorkerError::permanent("boom");
        assert_eq!(err.to_string(), "permanent error: boom");
    }

    #[test]
    fn kind_strings_are_stable() {
        assert_eq!(WorkerErrorKind::Retriable.as_str(), "retriable");
        assert_eq!(WorkerErrorKind::Permanent.as_str(), "permanent");
        assert_eq!(WorkerErrorKind::Cancelled.as_str(), "cancelled");
        assert_eq!(WorkerErrorKind::TimedOut.as_str(), "timed_out");
        assert_eq!(WorkerErrorKind::InvalidPayload.as_str(), "invalid_payload");
        assert_eq!(
            WorkerErrorKind::DependencyUnavailable.as_str(),
            "dependency_unavailable"
        );
    }

    #[test]
    fn constructors_set_kind_and_message() {
        let cases = [
            (WorkerError::retriable("a"), WorkerErrorKind::Retriable),
            (WorkerError::permanent("b"), WorkerErrorKind::Permanent),
            (WorkerError::cancelled("c"), WorkerErrorKind::Cancelled),
            (WorkerError::timed_out("d"), WorkerErrorKind::TimedOut),
            (
                WorkerError::invalid_payload("e"),
                WorkerErrorKind::InvalidPayload,
            ),
            (
                WorkerError::dependency_unavailable("f"),
                WorkerErrorKind::DependencyUnavailable,
            ),
        ];
        for (err, kind) in cases {
            assert_eq!(err.kind(), kind);
            assert_eq!(err.kind().is_retriable(), err.is_retriable());
            assert!(!err.message().is_empty());
        }
    }
}
