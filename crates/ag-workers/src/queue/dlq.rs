//! Dead-letter queue records.
//!
//! A dead-lettered job is preserved, never silently dropped: the original envelope,
//! the reason and the timestamp are retained for inspection and re-drive.

use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::ids::{JobId, JobKind, QueueName};
use crate::job::JobEnvelope;
use crate::outcome::DeadLetterReason;

/// Filter for bulk dead-letter operations (RFC-0017).
///
/// `queue` is required so a bulk action cannot span every queue by accident;
/// `kind` narrows further; `older_than` (purge only) composes with the filter;
/// `limit` bounds how many entries a single invocation affects, so one command
/// cannot re-drive or purge an unbounded flood.
#[derive(Debug, Clone)]
pub struct DlqFilter {
    /// Queue name the entries must belong to (required).
    pub queue: String,
    /// Optional job kind to narrow the selection.
    pub kind: Option<String>,
    /// Optional age threshold (purge): only entries older than this match.
    pub older_than: Option<Duration>,
    /// Maximum number of entries one call may affect.
    pub limit: i64,
}

impl DlqFilter {
    /// Builds a filter for `queue` with the given `limit` and no further narrowing.
    pub fn new(queue: impl Into<String>, limit: i64) -> Self {
        Self {
            queue: queue.into(),
            kind: None,
            older_than: None,
            limit,
        }
    }

    /// Narrows the filter to a single job kind.
    pub fn with_kind(mut self, kind: Option<String>) -> Self {
        self.kind = kind;
        self
    }

    /// Adds an age threshold (used by purge).
    pub fn with_older_than(mut self, older_than: Option<Duration>) -> Self {
        self.older_than = older_than;
        self
    }
}

/// Outcome of a bulk dead-letter operation, or its dry-run preview.
///
/// For a dry run, `affected` is the number of entries that *would* be touched and
/// no change is made; for a real run it is the number actually re-driven or purged.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BulkDlqOutcome {
    /// Entries affected (or that would be affected, for a dry run).
    pub affected: u64,
    /// A bounded sample of affected job ids, for operator confirmation.
    pub sample: Vec<JobId>,
}

/// Maximum number of ids returned in a [`BulkDlqOutcome::sample`].
pub const BULK_DLQ_SAMPLE_CAP: usize = 10;

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
