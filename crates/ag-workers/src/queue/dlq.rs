//! Dead-letter queue records.
//!
//! A dead-lettered job is preserved, never silently dropped: the original envelope,
//! the reason and the timestamp are retained for inspection and re-drive.

use chrono::{DateTime, Utc};

use crate::ids::{JobId, JobKind, QueueName};
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

/// A row from the dead-letter table, as returned by inspection queries.
///
/// Unlike [`DeadLetter`], this mirrors the persisted columns (the DLQ table does not
/// keep the full runtime envelope — no lease/status fields), which is what operational
/// tooling (`ag workers dlq ...`) lists and inspects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadLetterRecord {
    /// Stable job id (shared with the original job).
    pub id: JobId,
    /// Job kind.
    pub kind: JobKind,
    /// Queue the job belonged to.
    pub queue: QueueName,
    /// Attempts made before dead-lettering.
    pub attempts: u32,
    /// Configured maximum attempts.
    pub max_attempts: u32,
    /// Why the job was dead-lettered (reason text).
    pub reason: String,
    /// Last error summary, if any (redactable; never raw secrets).
    pub last_error: Option<String>,
    /// When the job was originally created.
    pub created_at: DateTime<Utc>,
    /// When it was moved to the dead-letter queue.
    pub dead_lettered_at: DateTime<Utc>,
}
