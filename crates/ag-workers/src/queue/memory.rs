//! In-memory queue backend.
//!
//! The default backend for development, tests, local demos and single-process apps. It
//! enforces a bounded depth (explicit [`QueueError::Full`] backpressure) and a maximum
//! payload size; it makes no durability claim (jobs are lost on restart). Leasing is
//! FIFO within a priority, then by eligibility time.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;

use crate::ids::{JobId, QueueName, WorkerId};
use crate::job::{JobEnvelope, JobStatus, NewJob};
use crate::outcome::DeadLetterReason;
use crate::queue::dlq::DeadLetter;
use crate::queue::{QueueBackend, QueueDepth, QueueError, RetryDecision};

/// Default maximum number of non-terminal jobs per queue before enqueue is rejected.
pub const DEFAULT_MAX_DEPTH: usize = 10_000;

/// Default maximum encoded payload size accepted at enqueue.
pub const DEFAULT_MAX_PAYLOAD_BYTES: usize = 256 * 1024;

/// Default lease duration before an unacked job becomes leasable again.
pub const DEFAULT_LEASE_TIMEOUT: Duration = Duration::from_secs(60);

struct Inner {
    jobs: HashMap<JobId, JobEnvelope>,
    dead: Vec<DeadLetter>,
}

/// In-memory [`QueueBackend`] backed by a mutex-guarded map.
pub struct MemoryQueue {
    inner: Mutex<Inner>,
    max_depth: usize,
    max_payload_bytes: usize,
    lease_timeout: Duration,
}

impl Default for MemoryQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryQueue {
    /// Creates an empty in-memory queue with default limits.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                jobs: HashMap::new(),
                dead: Vec::new(),
            }),
            max_depth: DEFAULT_MAX_DEPTH,
            max_payload_bytes: DEFAULT_MAX_PAYLOAD_BYTES,
            lease_timeout: DEFAULT_LEASE_TIMEOUT,
        }
    }

    /// Sets the maximum number of non-terminal jobs per queue.
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Sets the maximum accepted encoded payload size.
    pub fn with_max_payload_bytes(mut self, max_payload_bytes: usize) -> Self {
        self.max_payload_bytes = max_payload_bytes;
        self
    }

    /// Sets the lease duration.
    pub fn with_lease_timeout(mut self, lease_timeout: Duration) -> Self {
        self.lease_timeout = lease_timeout;
        self
    }

    /// Returns a snapshot of the current dead-letter entries.
    pub fn dead_letters(&self) -> Vec<DeadLetter> {
        self.inner
            .lock()
            .expect("queue mutex poisoned")
            .dead
            .clone()
    }

    /// Returns the number of non-terminal jobs currently held.
    pub fn job_count(&self) -> usize {
        self.inner.lock().expect("queue mutex poisoned").jobs.len()
    }

    fn count_for_queue(inner: &Inner, queue: &QueueName) -> usize {
        inner
            .jobs
            .values()
            .filter(|j| &j.queue == queue && !j.status.is_terminal())
            .count()
    }
}

#[async_trait]
impl QueueBackend for MemoryQueue {
    async fn enqueue(&self, job: NewJob) -> Result<JobId, QueueError> {
        if job.payload.len() > self.max_payload_bytes {
            return Err(QueueError::PayloadTooLarge {
                actual: job.payload.len(),
                limit: self.max_payload_bytes,
            });
        }
        let queue = job.queue.clone();
        let mut inner = self.inner.lock().expect("queue mutex poisoned");
        if Self::count_for_queue(&inner, &queue) >= self.max_depth {
            return Err(QueueError::Full(queue));
        }
        let envelope = job.into_envelope(Utc::now());
        let id = envelope.id;
        inner.jobs.insert(id, envelope);
        Ok(id)
    }

    async fn lease(
        &self,
        queue: &QueueName,
        worker: &WorkerId,
        limit: usize,
    ) -> Result<Vec<JobEnvelope>, QueueError> {
        let now = Utc::now();
        let mut inner = self.inner.lock().expect("queue mutex poisoned");

        // Collect eligible job ids: ready and not currently locked. `Leased` jobs whose
        // lease has expired are reclaimed here (this is the memory backend's equivalent
        // of the durable reaper: a worker that dropped a job without acking gets it
        // re-leased once the lease elapses).
        let mut eligible: Vec<JobId> = inner
            .jobs
            .values()
            .filter(|j| {
                &j.queue == queue
                    && matches!(
                        j.status,
                        JobStatus::Queued
                            | JobStatus::Scheduled
                            | JobStatus::RetryScheduled
                            | JobStatus::Leased
                    )
                    && j.available_at <= now
                    && j.lock_until.map(|u| u < now).unwrap_or(true)
            })
            .map(|j| j.id)
            .collect();

        // Deterministic leasing order: priority desc, then available_at asc, then
        // created_at asc. Sort on a snapshot of the ordering keys.
        eligible.sort_by(|a, b| {
            let ja = &inner.jobs[a];
            let jb = &inner.jobs[b];
            jb.priority
                .cmp(&ja.priority)
                .then(ja.available_at.cmp(&jb.available_at))
                .then(ja.created_at.cmp(&jb.created_at))
        });
        eligible.truncate(limit);

        let lease_until = now
            + chrono::Duration::from_std(self.lease_timeout)
                .unwrap_or_else(|_| chrono::Duration::seconds(60));

        let mut leased = Vec::with_capacity(eligible.len());
        for id in eligible {
            let job = inner.jobs.get_mut(&id).expect("eligible id must exist");
            // Promote scheduled/retry-scheduled/expired-lease to queued, then to leased,
            // honoring the state machine (all three -> Queued are legal transitions).
            if matches!(
                job.status,
                JobStatus::Scheduled | JobStatus::RetryScheduled | JobStatus::Leased
            ) {
                let _ = job.transition_to(JobStatus::Queued);
            }
            if job.transition_to(JobStatus::Leased) {
                // Attempt increment happens at lease time: this is what makes the poison
                // guard correct under panic = "abort".
                job.attempts += 1;
                job.locked_by = Some(worker.clone());
                job.lock_until = Some(lease_until);
                leased.push(job.clone());
            }
        }
        Ok(leased)
    }

    async fn ack(&self, job_id: JobId) -> Result<(), QueueError> {
        let mut inner = self.inner.lock().expect("queue mutex poisoned");
        match inner.jobs.get_mut(&job_id) {
            // The job completed: terminal success. We drop it from the active map.
            Some(_) => {
                inner.jobs.remove(&job_id);
                Ok(())
            }
            None => Err(QueueError::NotFound(job_id)),
        }
    }

    async fn retry(&self, job_id: JobId, decision: RetryDecision) -> Result<(), QueueError> {
        let mut inner = self.inner.lock().expect("queue mutex poisoned");
        match inner.jobs.get_mut(&job_id) {
            Some(job) => {
                job.status = JobStatus::RetryScheduled;
                job.available_at = decision.available_at;
                job.last_error = decision.last_error;
                job.locked_by = None;
                job.lock_until = None;
                job.updated_at = Utc::now();
                Ok(())
            }
            None => Err(QueueError::NotFound(job_id)),
        }
    }

    async fn dead_letter(&self, job_id: JobId, reason: DeadLetterReason) -> Result<(), QueueError> {
        let now = Utc::now();
        let mut inner = self.inner.lock().expect("queue mutex poisoned");
        match inner.jobs.remove(&job_id) {
            Some(mut job) => {
                job.status = JobStatus::DeadLettered;
                job.locked_by = None;
                job.lock_until = None;
                job.updated_at = now;
                inner.dead.push(DeadLetter::new(job, reason, now));
                Ok(())
            }
            None => Err(QueueError::NotFound(job_id)),
        }
    }

    async fn depth(&self, queue: &QueueName) -> Result<QueueDepth, QueueError> {
        let now = Utc::now();
        let inner = self.inner.lock().expect("queue mutex poisoned");
        let mut depth = QueueDepth::default();
        for job in inner.jobs.values().filter(|j| &j.queue == queue) {
            match job.status {
                JobStatus::Leased | JobStatus::Running => depth.leased += 1,
                JobStatus::Queued => depth.ready += 1,
                JobStatus::Scheduled | JobStatus::RetryScheduled => {
                    if job.available_at <= now {
                        depth.ready += 1;
                    } else {
                        depth.scheduled += 1;
                    }
                }
                _ => {}
            }
        }
        Ok(depth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::JobPriority;
    use crate::payload::PayloadBytes;

    fn job(queue: &str) -> NewJob {
        NewJob::new(queue, queue, PayloadBytes::from_vec(vec![1, 2, 3]), 1)
    }

    #[tokio::test]
    async fn enqueue_and_lease_roundtrip() {
        let q = MemoryQueue::new();
        let worker = WorkerId::generate();
        let queue = QueueName::new("mail");
        q.enqueue(job("mail")).await.unwrap();

        let leased = q.lease(&queue, &worker, 10).await.unwrap();
        assert_eq!(leased.len(), 1);
        assert_eq!(leased[0].status, JobStatus::Leased);
        assert_eq!(leased[0].attempts, 1, "attempt increments at lease time");
        assert_eq!(leased[0].locked_by.as_ref(), Some(&worker));

        // A second lease finds nothing (the job is locked).
        assert!(q.lease(&queue, &worker, 10).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn ack_removes_job() {
        let q = MemoryQueue::new();
        let worker = WorkerId::generate();
        let queue = QueueName::new("mail");
        q.enqueue(job("mail")).await.unwrap();
        let leased = q.lease(&queue, &worker, 1).await.unwrap();
        q.ack(leased[0].id).await.unwrap();
        assert_eq!(q.job_count(), 0);
    }

    #[tokio::test]
    async fn dead_letter_preserves_entry() {
        let q = MemoryQueue::new();
        let worker = WorkerId::generate();
        let queue = QueueName::new("mail");
        q.enqueue(job("mail")).await.unwrap();
        let leased = q.lease(&queue, &worker, 1).await.unwrap();
        q.dead_letter(leased[0].id, DeadLetterReason::PoisonGuard)
            .await
            .unwrap();
        let dead = q.dead_letters();
        assert_eq!(dead.len(), 1);
        assert_eq!(dead[0].reason, DeadLetterReason::PoisonGuard);
        assert_eq!(dead[0].envelope.status, JobStatus::DeadLettered);
        assert_eq!(q.job_count(), 0);
    }

    #[tokio::test]
    async fn full_queue_rejects() {
        let q = MemoryQueue::new().with_max_depth(1);
        q.enqueue(job("mail")).await.unwrap();
        let err = q.enqueue(job("mail")).await.unwrap_err();
        assert!(matches!(err, QueueError::Full(_)));
    }

    #[tokio::test]
    async fn oversized_payload_rejected() {
        let q = MemoryQueue::new().with_max_payload_bytes(2);
        let err = q.enqueue(job("mail")).await.unwrap_err();
        assert!(matches!(err, QueueError::PayloadTooLarge { .. }));
    }

    #[tokio::test]
    async fn higher_priority_leased_first() {
        let q = MemoryQueue::new();
        let queue = QueueName::new("mail");
        let worker = WorkerId::generate();
        q.enqueue(job("mail").with_priority(JobPriority::Low))
            .await
            .unwrap();
        let crit = q
            .enqueue(job("mail").with_priority(JobPriority::Critical))
            .await
            .unwrap();
        let leased = q.lease(&queue, &worker, 1).await.unwrap();
        assert_eq!(leased[0].id, crit, "critical job is leased before low");
    }

    #[tokio::test]
    async fn depth_counts_phases() {
        let q = MemoryQueue::new();
        let queue = QueueName::new("mail");
        let worker = WorkerId::generate();
        q.enqueue(job("mail")).await.unwrap();
        q.enqueue(job("mail")).await.unwrap();
        let d0 = q.depth(&queue).await.unwrap();
        assert_eq!(d0.ready, 2);
        q.lease(&queue, &worker, 1).await.unwrap();
        let d1 = q.depth(&queue).await.unwrap();
        assert_eq!(d1.ready, 1);
        assert_eq!(d1.leased, 1);
    }
}
