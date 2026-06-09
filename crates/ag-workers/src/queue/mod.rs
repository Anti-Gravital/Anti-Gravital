//! Queue backend abstraction and the in-memory backend.
//!
//! A [`QueueBackend`] persists jobs and leases them to workers. The trait is the
//! contract every backend (in-memory now; PostgreSQL later) must satisfy. The methods
//! defined here are the ones the memory runtime needs; durability-only operations
//! (heartbeat, reaper, transactional enqueue, schedule claim) are added with the
//! PostgreSQL backend.

pub mod dlq;
pub mod memory;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::ids::{JobId, QueueName, WorkerId};
use crate::job::{JobEnvelope, NewJob};
use crate::outcome::DeadLetterReason;

/// Errors produced by a queue backend.
#[derive(Debug, Error)]
pub enum QueueError {
    /// The queue is at capacity; the enqueue was rejected (backpressure).
    #[error("queue '{0}' is full")]
    Full(QueueName),

    /// The referenced job does not exist (or is no longer leasable by this caller).
    #[error("job not found: {0}")]
    NotFound(JobId),

    /// The payload exceeds the configured maximum size.
    #[error("payload too large: {actual} bytes exceeds limit of {limit} bytes")]
    PayloadTooLarge {
        /// Rejected size.
        actual: usize,
        /// Configured maximum.
        limit: usize,
    },

    /// A backend-specific failure (I/O, database, etc.).
    #[error("queue backend error: {0}")]
    Backend(String),
}

/// A snapshot of how many jobs are in each phase for a queue.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct QueueDepth {
    /// Jobs ready to be leased now.
    pub ready: u64,
    /// Jobs scheduled for the future (delayed or retry-scheduled).
    pub scheduled: u64,
    /// Jobs currently leased/running.
    pub leased: u64,
}

impl QueueDepth {
    /// Total number of non-terminal jobs in the queue.
    pub fn total(&self) -> u64 {
        self.ready + self.scheduled + self.leased
    }
}

/// Instruction for re-queuing a job after a retriable failure.
#[derive(Debug, Clone)]
pub struct RetryDecision {
    /// When the job becomes eligible again.
    pub available_at: DateTime<Utc>,
    /// Redactable summary of the failure, stored on the job.
    pub last_error: Option<String>,
}

/// A queue backend: persists jobs and leases them to workers.
#[async_trait]
pub trait QueueBackend: Send + Sync + 'static {
    /// Enqueues a new job, returning its id. May reject with [`QueueError::Full`] when
    /// the queue is at capacity (admission control / backpressure).
    async fn enqueue(&self, job: NewJob) -> Result<JobId, QueueError>;

    /// Leases up to `limit` eligible jobs from `queue` for `worker`, marking them
    /// `leased` and incrementing their attempt count (attempt increment at lease time
    /// is what makes the poison guard correct under `panic = "abort"`).
    async fn lease(
        &self,
        queue: &QueueName,
        worker: &WorkerId,
        limit: usize,
    ) -> Result<Vec<JobEnvelope>, QueueError>;

    /// Acknowledges a job as completed (terminal `succeeded`).
    async fn ack(&self, job_id: JobId) -> Result<(), QueueError>;

    /// Returns a leased job to the queue for a future attempt.
    async fn retry(&self, job_id: JobId, decision: RetryDecision) -> Result<(), QueueError>;

    /// Moves a job to the dead-letter queue, preserving it.
    async fn dead_letter(&self, job_id: JobId, reason: DeadLetterReason) -> Result<(), QueueError>;

    /// Returns the current depth of `queue`.
    async fn depth(&self, queue: &QueueName) -> Result<QueueDepth, QueueError>;
}
