//! Dead-letter queue records.
//!
//! A dead-lettered job is preserved, never silently dropped: the original envelope,
//! the reason and the timestamp are retained for inspection and re-drive.

use chrono::{DateTime, Utc};

use crate::job::JobEnvelope;
use crate::outcome::DeadLetterReason;

/// A preserved dead-letter entry.
#[derive(Debug, Clone)]
pub struct DeadLetter {
    /// The job as it was when dead-lettered (payload and metadata preserved).
    pub envelope: JobEnvelope,
    /// Why the job was dead-lettered.
    pub reason: DeadLetterReason,
    /// When it was moved to the dead-letter queue.
    pub dead_lettered_at: DateTime<Utc>,
}

impl DeadLetter {
    /// Builds a dead-letter entry stamped at `now`.
    pub fn new(envelope: JobEnvelope, reason: DeadLetterReason, now: DateTime<Utc>) -> Self {
        Self {
            envelope,
            reason,
            dead_lettered_at: now,
        }
    }
}
