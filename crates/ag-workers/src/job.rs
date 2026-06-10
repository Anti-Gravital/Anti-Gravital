//! The job model: priority, the lifecycle state machine, the envelope and the
//! enqueue request.
//!
//! The developer reasons about exactly one concept: a job — a typed unit of deferred
//! work with an identity, a queue, a payload and a retry policy. [`JobEnvelope`] is the
//! internal persistence/runtime representation; [`NewJob`] is the enqueue request that
//! becomes an envelope.

use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{JobId, JobKind, QueueName, TenantId, WorkerId};
use crate::payload::{encode, JobPayload, PayloadBytes, PayloadError};

/// Default maximum number of attempts when an enqueue does not specify one.
pub const DEFAULT_MAX_ATTEMPTS: u32 = 5;

/// Default per-job execution timeout when an enqueue does not specify one.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Relative importance of a job within its queue.
///
/// A small, totally-ordered enum (not arbitrary integers) keeps the leasing order
/// deterministic and the model legible. `Critical` is leased before `Low`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub enum JobPriority {
    /// Lowest priority.
    Low,
    /// Default priority.
    #[default]
    Normal,
    /// Above normal.
    High,
    /// Highest priority; leased first.
    Critical,
}

impl JobPriority {
    /// Returns the integer used to store and order priorities in the database
    /// (`Critical` is highest, so `ORDER BY priority DESC` leases it first).
    pub fn as_i16(self) -> i16 {
        match self {
            JobPriority::Low => 0,
            JobPriority::Normal => 1,
            JobPriority::High => 2,
            JobPriority::Critical => 3,
        }
    }

    /// Parses a stored priority integer, defaulting to `Normal` for unknown values.
    pub fn from_i16(value: i16) -> Self {
        match value {
            0 => JobPriority::Low,
            2 => JobPriority::High,
            3 => JobPriority::Critical,
            _ => JobPriority::Normal,
        }
    }
}

/// Lifecycle state of a job.
///
/// The allowed transitions are normative: [`JobStatus::can_transition_to`] encodes the
/// table from RFC-0012 and the persistence layer must reject any other move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// `available_at` is in the future; not yet eligible.
    Scheduled,
    /// Ready to be leased.
    Queued,
    /// Claimed by a worker; a lock is held.
    Leased,
    /// The handler is executing.
    Running,
    /// Terminal: completed successfully.
    Succeeded,
    /// Will return to `Queued` at `available_at`.
    RetryScheduled,
    /// Terminal: non-retriable failure, recorded (distinct from dead-lettered).
    Failed,
    /// Terminal: preserved in the dead-letter queue.
    DeadLettered,
    /// Terminal: cancelled before completion.
    Cancelled,
    /// Terminal: exceeded its maximum age before succeeding.
    Expired,
}

impl JobStatus {
    /// Returns the stable string stored in the database column and used in logs.
    pub fn as_str(self) -> &'static str {
        match self {
            JobStatus::Scheduled => "scheduled",
            JobStatus::Queued => "queued",
            JobStatus::Leased => "leased",
            JobStatus::Running => "running",
            JobStatus::Succeeded => "succeeded",
            JobStatus::RetryScheduled => "retry_scheduled",
            JobStatus::Failed => "failed",
            JobStatus::DeadLettered => "dead_lettered",
            JobStatus::Cancelled => "cancelled",
            JobStatus::Expired => "expired",
        }
    }

    /// Parses a status stored in the database column. Returns `None` for unknown text.
    pub fn from_db_str(value: &str) -> Option<Self> {
        Some(match value {
            "scheduled" => JobStatus::Scheduled,
            "queued" => JobStatus::Queued,
            "leased" => JobStatus::Leased,
            "running" => JobStatus::Running,
            "succeeded" => JobStatus::Succeeded,
            "retry_scheduled" => JobStatus::RetryScheduled,
            "failed" => JobStatus::Failed,
            "dead_lettered" => JobStatus::DeadLettered,
            "cancelled" => JobStatus::Cancelled,
            "expired" => JobStatus::Expired,
            _ => return None,
        })
    }

    /// Returns `true` if this is a terminal state (no further transitions allowed).
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            JobStatus::Succeeded
                | JobStatus::Failed
                | JobStatus::DeadLettered
                | JobStatus::Cancelled
                | JobStatus::Expired
        )
    }

    /// Returns `true` if a transition from `self` to `to` is allowed.
    ///
    /// Encodes the normative transition table of RFC-0012 §9.1.
    pub fn can_transition_to(self, to: JobStatus) -> bool {
        use JobStatus::*;
        match self {
            Scheduled => matches!(to, Queued | Cancelled | Expired),
            Queued => matches!(to, Leased | Cancelled | Expired),
            Leased => matches!(to, Running | Queued | Cancelled),
            Running => matches!(
                to,
                Succeeded | RetryScheduled | DeadLettered | Failed | Cancelled | Expired
            ),
            RetryScheduled => matches!(to, Queued | Cancelled | Expired),
            // Terminal states allow no transitions.
            Succeeded | Failed | DeadLettered | Cancelled | Expired => false,
        }
    }
}

/// The internal persistence/runtime representation of a job.
///
/// Typed payloads live only at the API boundary; this envelope carries the opaque
/// [`PayloadBytes`] plus the metadata the runtime needs to lease, time out, retry and
/// trace the job.
#[derive(Debug, Clone)]
pub struct JobEnvelope {
    /// Stable identity, preserved across retries and re-leases.
    pub id: JobId,
    /// The job kind, resolved against the closed registry.
    pub kind: JobKind,
    /// The queue this job belongs to.
    pub queue: QueueName,
    /// Encoded payload bytes.
    pub payload: PayloadBytes,
    /// Schema version of the encoded payload.
    pub payload_version: u16,
    /// Relative priority within the queue.
    pub priority: JobPriority,
    /// Current lifecycle state.
    pub status: JobStatus,
    /// Number of attempts started so far (incremented at lease time).
    pub attempts: u32,
    /// Maximum number of attempts before the job is dead-lettered.
    pub max_attempts: u32,
    /// Per-attempt execution timeout.
    pub timeout: Duration,
    /// When the job was scheduled, if it was a delayed/scheduled enqueue.
    pub scheduled_at: Option<DateTime<Utc>>,
    /// When the job becomes eligible to be leased.
    pub available_at: DateTime<Utc>,
    /// When the job was created.
    pub created_at: DateTime<Utc>,
    /// When the job was last updated.
    pub updated_at: DateTime<Utc>,
    /// Worker currently holding the lease, if leased.
    pub locked_by: Option<WorkerId>,
    /// When the current lease expires, if leased.
    pub lock_until: Option<DateTime<Utc>>,
    /// Optional deduplication key (unique per queue when configured).
    pub dedup_key: Option<String>,
    /// Optional tenant context.
    pub tenant_id: Option<TenantId>,
    /// Optional trace/correlation id propagated from the enqueuing request.
    pub trace_id: Option<String>,
    /// Redactable summary of the last error; never raw secrets.
    pub last_error: Option<String>,
}

impl JobEnvelope {
    /// Attempts to move the job to `to`, returning `false` (and leaving the status
    /// unchanged) if the transition is not allowed by the state machine.
    ///
    /// Also stamps `updated_at` on a successful transition.
    #[must_use]
    pub fn transition_to(&mut self, to: JobStatus) -> bool {
        if self.status.can_transition_to(to) {
            self.status = to;
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Returns `true` if the job has reached or exceeded its attempt limit.
    pub fn attempts_exhausted(&self) -> bool {
        self.attempts >= self.max_attempts
    }
}

/// A request to enqueue a new job.
///
/// Built directly or via [`NewJob::encode`] for a typed payload, then turned into a
/// [`JobEnvelope`] by the backend at enqueue time.
#[derive(Debug, Clone)]
pub struct NewJob {
    /// The job kind (must resolve to a registered handler).
    pub kind: JobKind,
    /// The queue to enqueue into.
    pub queue: QueueName,
    /// Encoded payload bytes.
    pub payload: PayloadBytes,
    /// Schema version of the encoded payload.
    pub payload_version: u16,
    /// Relative priority within the queue.
    pub priority: JobPriority,
    /// Maximum number of attempts.
    pub max_attempts: u32,
    /// Per-attempt execution timeout.
    pub timeout: Duration,
    /// Optional delay before the job becomes eligible.
    pub delay: Option<Duration>,
    /// Marks the enqueue as a scheduled job (records `scheduled_at`).
    pub scheduled: bool,
    /// Optional deduplication key.
    pub dedup_key: Option<String>,
    /// Optional tenant context.
    pub tenant_id: Option<TenantId>,
    /// Optional trace/correlation id.
    pub trace_id: Option<String>,
}

impl NewJob {
    /// Builds an enqueue request from raw, already-encoded payload bytes.
    pub fn new(
        kind: impl Into<JobKind>,
        queue: impl Into<QueueName>,
        payload: PayloadBytes,
        payload_version: u16,
    ) -> Self {
        Self {
            kind: kind.into(),
            queue: queue.into(),
            payload,
            payload_version,
            priority: JobPriority::Normal,
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            timeout: DEFAULT_TIMEOUT,
            delay: None,
            scheduled: false,
            dedup_key: None,
            tenant_id: None,
            trace_id: None,
        }
    }

    /// Builds an enqueue request by encoding a typed payload.
    ///
    /// The payload version is taken from [`JobPayload::PAYLOAD_VERSION`].
    pub fn encode<P: JobPayload>(
        kind: impl Into<JobKind>,
        queue: impl Into<QueueName>,
        payload: &P,
    ) -> Result<Self, PayloadError> {
        let bytes = encode(payload)?;
        Ok(Self::new(kind, queue, bytes, P::PAYLOAD_VERSION))
    }

    /// Sets the priority.
    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the maximum number of attempts.
    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Sets the per-attempt timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Delays eligibility by `delay`.
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }

    /// Marks the job as scheduled (records `scheduled_at`).
    pub fn scheduled(mut self) -> Self {
        self.scheduled = true;
        self
    }

    /// Sets a deduplication key.
    pub fn with_dedup_key(mut self, key: impl Into<String>) -> Self {
        self.dedup_key = Some(key.into());
        self
    }

    /// Sets the tenant context.
    pub fn with_tenant(mut self, tenant: TenantId) -> Self {
        self.tenant_id = Some(tenant);
        self
    }

    /// Sets the trace/correlation id.
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Materializes the request into a [`JobEnvelope`] with timestamps relative to
    /// `now`. The initial status is `Scheduled` for a delayed/scheduled job and
    /// `Queued` otherwise.
    pub fn into_envelope(self, now: DateTime<Utc>) -> JobEnvelope {
        let available_at = match self.delay {
            Some(delay) => {
                now + chrono::Duration::from_std(delay).unwrap_or_else(|_| chrono::Duration::zero())
            }
            None => now,
        };
        let is_delayed = available_at > now || self.scheduled;
        let status = if is_delayed {
            JobStatus::Scheduled
        } else {
            JobStatus::Queued
        };
        JobEnvelope {
            id: JobId::new(),
            kind: self.kind,
            queue: self.queue,
            payload: self.payload,
            payload_version: self.payload_version,
            priority: self.priority,
            status,
            attempts: 0,
            max_attempts: self.max_attempts,
            timeout: self.timeout,
            scheduled_at: self.scheduled.then_some(now),
            available_at,
            created_at: now,
            updated_at: now,
            locked_by: None,
            lock_until: None,
            dedup_key: self.dedup_key,
            tenant_id: self.tenant_id,
            trace_id: self.trace_id,
            last_error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct P {
        x: u32,
    }
    impl JobPayload for P {
        const PAYLOAD_VERSION: u16 = 3;
    }

    #[test]
    fn priority_orders_low_to_critical() {
        assert!(JobPriority::Low < JobPriority::Normal);
        assert!(JobPriority::Normal < JobPriority::High);
        assert!(JobPriority::High < JobPriority::Critical);
        assert_eq!(JobPriority::default(), JobPriority::Normal);
    }

    #[test]
    fn legal_transitions_allowed() {
        assert!(JobStatus::Scheduled.can_transition_to(JobStatus::Queued));
        assert!(JobStatus::Queued.can_transition_to(JobStatus::Leased));
        assert!(JobStatus::Leased.can_transition_to(JobStatus::Running));
        assert!(JobStatus::Leased.can_transition_to(JobStatus::Queued));
        assert!(JobStatus::Running.can_transition_to(JobStatus::Succeeded));
        assert!(JobStatus::Running.can_transition_to(JobStatus::RetryScheduled));
        assert!(JobStatus::Running.can_transition_to(JobStatus::DeadLettered));
        assert!(JobStatus::RetryScheduled.can_transition_to(JobStatus::Queued));
    }

    #[test]
    fn illegal_transitions_rejected() {
        assert!(!JobStatus::Queued.can_transition_to(JobStatus::Running));
        assert!(!JobStatus::Scheduled.can_transition_to(JobStatus::Leased));
        assert!(!JobStatus::Succeeded.can_transition_to(JobStatus::Queued));
        assert!(!JobStatus::DeadLettered.can_transition_to(JobStatus::Queued));
    }

    #[test]
    fn terminal_states_are_terminal() {
        for s in [
            JobStatus::Succeeded,
            JobStatus::Failed,
            JobStatus::DeadLettered,
            JobStatus::Cancelled,
            JobStatus::Expired,
        ] {
            assert!(s.is_terminal());
        }
        assert!(!JobStatus::Queued.is_terminal());
    }

    #[test]
    fn envelope_transition_stamps_and_guards() {
        let mut env = NewJob::encode::<P>("k", "q", &P { x: 1 })
            .unwrap()
            .into_envelope(Utc::now());
        assert_eq!(env.status, JobStatus::Queued);
        assert_eq!(env.payload_version, 3);
        assert!(env.transition_to(JobStatus::Leased));
        assert_eq!(env.status, JobStatus::Leased);
        // Illegal jump is rejected and leaves the status unchanged.
        assert!(!env.transition_to(JobStatus::Succeeded));
        assert_eq!(env.status, JobStatus::Leased);
    }

    #[test]
    fn delayed_enqueue_starts_scheduled() {
        let env = NewJob::new("k", "q", PayloadBytes::from_vec(vec![]), 1)
            .with_delay(Duration::from_secs(60))
            .into_envelope(Utc::now());
        assert_eq!(env.status, JobStatus::Scheduled);
        assert!(env.available_at > env.created_at);
    }

    #[test]
    fn attempts_exhausted_check() {
        let mut env = NewJob::new("k", "q", PayloadBytes::from_vec(vec![]), 1)
            .with_max_attempts(2)
            .into_envelope(Utc::now());
        assert!(!env.attempts_exhausted());
        env.attempts = 2;
        assert!(env.attempts_exhausted());
    }

    #[test]
    fn priority_i16_roundtrips_and_defaults() {
        for p in [
            JobPriority::Low,
            JobPriority::Normal,
            JobPriority::High,
            JobPriority::Critical,
        ] {
            assert_eq!(JobPriority::from_i16(p.as_i16()), p);
        }
        // Unknown stored values fall back to Normal.
        assert_eq!(JobPriority::from_i16(99), JobPriority::Normal);
        assert_eq!(JobPriority::from_i16(-1), JobPriority::Normal);
    }

    #[test]
    fn status_str_roundtrips_and_rejects_unknown() {
        for s in [
            JobStatus::Scheduled,
            JobStatus::Queued,
            JobStatus::Leased,
            JobStatus::Running,
            JobStatus::Succeeded,
            JobStatus::RetryScheduled,
            JobStatus::Failed,
            JobStatus::DeadLettered,
            JobStatus::Cancelled,
            JobStatus::Expired,
        ] {
            assert_eq!(JobStatus::from_db_str(s.as_str()), Some(s));
        }
        assert_eq!(JobStatus::from_db_str("bogus"), None);
    }

    #[test]
    fn newjob_builders_propagate_to_envelope() {
        let env = NewJob::new("k", "q", PayloadBytes::from_vec(vec![]), 1)
            .with_priority(JobPriority::High)
            .with_max_attempts(7)
            .with_timeout(Duration::from_secs(42))
            .with_dedup_key("dk-1")
            .with_tenant(TenantId::new("acme"))
            .with_trace_id("trace-9")
            .into_envelope(Utc::now());
        assert_eq!(env.priority, JobPriority::High);
        assert_eq!(env.max_attempts, 7);
        assert_eq!(env.timeout, Duration::from_secs(42));
        assert_eq!(env.dedup_key.as_deref(), Some("dk-1"));
        assert_eq!(env.tenant_id.as_ref().map(|t| t.as_str()), Some("acme"));
        assert_eq!(env.trace_id.as_deref(), Some("trace-9"));
    }

    #[test]
    fn explicit_scheduled_sets_scheduled_at() {
        let env = NewJob::new("k", "q", PayloadBytes::from_vec(vec![]), 1)
            .scheduled()
            .into_envelope(Utc::now());
        assert_eq!(env.status, JobStatus::Scheduled);
        assert!(env.scheduled_at.is_some());
    }
}
