//! Two-tier delivery queue for the native MTA (Phase 4.6-B).
//!
//! The queue separates *scheduled* jobs (waiting for their next attempt time,
//! ordered by a min-heap) from the *ready* batch (up to `max_ready` jobs whose
//! time has come, pulled per cycle). A cycle pulls due jobs, drops suppressed
//! recipients, applies per-`site_name` rate shaping (rescheduling instead of
//! blocking), takes a connection permit, delivers through a
//! [`DeliveryBackend`], and on failure either reschedules with exponential
//! backoff or gives up and suppresses.
//!
//! Delivery is abstracted behind [`DeliveryBackend`] so the scheduling logic is
//! deterministic and unit-tested with a mock; [`super::MtaSender`] implements
//! the trait for real delivery. This is the native, in-memory tier; a durable
//! spool (JetStream / PostgreSQL) is a later, optional backing (see
//! `docs/DEBT.md`).

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use async_trait::async_trait;

use super::shaping::Shaper;
use super::suppress::{SuppressionList, SuppressionReason};

/// A unit of work: one rendered message destined for one recipient domain.
#[derive(Debug, Clone)]
pub struct DeliveryJob {
    /// Tenant that owns the message (part of the scheduled-queue key).
    pub tenant: String,
    /// Campaign / stream (part of the scheduled-queue key).
    pub campaign: String,
    /// Recipient domain (part of the scheduled-queue key).
    pub domain: String,
    /// Destination `site_name` (MX rollup) used for shaping.
    pub site_name: String,
    /// Envelope recipients at this domain.
    pub recipients: Vec<String>,
    /// Envelope sender.
    pub from: String,
    /// Rendered RFC 5322 message bytes.
    pub content: Vec<u8>,
    /// Attempts made so far.
    pub attempts: u32,
    /// When the job first entered the queue (for the max-age cutoff).
    pub enqueued_at: Instant,
    /// Earliest time the job may be attempted.
    pub next_attempt_at: Instant,
}

impl DeliveryJob {
    /// Builds a job ready to attempt at `now`. The routing key, addressing and
    /// payload are all distinct required fields, so the constructor takes them
    /// directly; all fields are public for struct-literal construction too.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant: impl Into<String>,
        campaign: impl Into<String>,
        domain: impl Into<String>,
        site_name: impl Into<String>,
        from: impl Into<String>,
        recipients: Vec<String>,
        content: Vec<u8>,
        now: Instant,
    ) -> Self {
        Self {
            tenant: tenant.into(),
            campaign: campaign.into(),
            domain: domain.into(),
            site_name: site_name.into(),
            recipients,
            from: from.into(),
            content,
            attempts: 0,
            enqueued_at: now,
            next_attempt_at: now,
        }
    }

    /// The scheduled-queue key `tenant:campaign:domain`.
    pub fn scheduled_key(&self) -> String {
        format!("{}:{}:{}", self.tenant, self.campaign, self.domain)
    }
}

/// The result of one delivery attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryOutcome {
    /// The message was accepted by the destination.
    Delivered,
    /// A temporary failure (4xx); retry later.
    Transient(String),
    /// A permanent failure (5xx); suppress the recipients.
    Permanent(String),
}

/// Performs a single delivery attempt for a job.
#[async_trait]
pub trait DeliveryBackend: Send + Sync {
    /// Attempts to deliver `job` once and classifies the result.
    async fn deliver(&self, job: &DeliveryJob) -> DeliveryOutcome;
}

/// Retry/backoff policy for the scheduled queue.
#[derive(Debug, Clone)]
pub struct MtaRetryPolicy {
    /// Maximum attempts before giving up.
    pub max_attempts: u32,
    /// Backoff for the first retry; doubles each attempt up to `max_backoff`.
    pub base_backoff: Duration,
    /// Upper bound on the backoff between attempts.
    pub max_backoff: Duration,
    /// Maximum age a job may reach before it is dropped.
    pub max_age: Duration,
}

impl Default for MtaRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 10,
            base_backoff: Duration::from_secs(60),
            max_backoff: Duration::from_secs(2 * 60 * 60),
            max_age: Duration::from_secs(72 * 60 * 60),
        }
    }
}

impl MtaRetryPolicy {
    /// Exponential backoff for the given attempt number (1-based), capped at
    /// `max_backoff`.
    pub fn backoff(&self, attempt: u32) -> Duration {
        let factor = 2u64.saturating_pow(attempt.saturating_sub(1));
        let secs = self.base_backoff.as_secs().saturating_mul(factor);
        Duration::from_secs(secs).min(self.max_backoff)
    }
}

/// Queue configuration.
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Maximum jobs moved from scheduled to ready per cycle.
    pub max_ready: usize,
    /// Retry policy.
    pub retry: MtaRetryPolicy,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_ready: 1024,
            retry: MtaRetryPolicy::default(),
        }
    }
}

/// Counts produced by one [`MtaQueue::process_due`] cycle.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CycleReport {
    /// Jobs delivered successfully.
    pub delivered: u64,
    /// Jobs rescheduled after a transient failure.
    pub retried: u64,
    /// Jobs dropped after a permanent failure (recipients suppressed).
    pub suppressed: u64,
    /// Jobs dropped after exhausting attempts or max age.
    pub expired: u64,
    /// Jobs rescheduled because the rate limiter asked them to wait.
    pub throttled: u64,
    /// Jobs dropped because every recipient was already suppressed.
    pub skipped_suppressed: u64,
}

// Heap entry: ordered by `next_attempt_at` then insertion order, so a
// `BinaryHeap` of `Reverse<_>` yields the earliest-due job first.
struct Scheduled {
    at: Instant,
    seq: u64,
    job: DeliveryJob,
}

impl PartialEq for Scheduled {
    fn eq(&self, other: &Self) -> bool {
        self.at == other.at && self.seq == other.seq
    }
}
impl Eq for Scheduled {}
impl Ord for Scheduled {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse so the max-heap pops the earliest time first.
        other
            .at
            .cmp(&self.at)
            .then_with(|| other.seq.cmp(&self.seq))
    }
}
impl PartialOrd for Scheduled {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Native in-memory two-tier delivery queue.
pub struct MtaQueue {
    scheduled: Mutex<BinaryHeap<Scheduled>>,
    seq: AtomicU64,
    config: QueueConfig,
    shaper: Shaper,
    suppress: SuppressionList,
}

impl MtaQueue {
    /// Builds a queue with the given configuration and shaper.
    pub fn new(config: QueueConfig, shaper: Shaper) -> Self {
        Self {
            scheduled: Mutex::new(BinaryHeap::new()),
            seq: AtomicU64::new(0),
            config,
            shaper,
            suppress: SuppressionList::new(),
        }
    }

    /// Access to the suppression list (shared by inbound DSN/FBL processing).
    pub fn suppressions(&self) -> &SuppressionList {
        &self.suppress
    }

    /// Number of jobs currently scheduled.
    pub fn len(&self) -> usize {
        self.scheduled.lock().expect("queue lock poisoned").len()
    }

    /// Whether the scheduled queue is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Enqueues a job. It becomes eligible at its `next_attempt_at`.
    pub fn enqueue(&self, job: DeliveryJob) {
        crate::metrics::queue_depth_inc();
        self.push(job);
    }

    fn push(&self, job: DeliveryJob) {
        let seq = self.seq.fetch_add(1, AtomicOrdering::Relaxed);
        let entry = Scheduled {
            at: job.next_attempt_at,
            seq,
            job,
        };
        self.scheduled
            .lock()
            .expect("queue lock poisoned")
            .push(entry);
    }

    /// Instant at which the next job becomes due, if any.
    pub fn next_due(&self) -> Option<Instant> {
        self.scheduled
            .lock()
            .expect("queue lock poisoned")
            .peek()
            .map(|s| s.at)
    }

    /// Pulls up to `max_ready` jobs that are due at `now`.
    fn drain_due(&self, now: Instant) -> Vec<DeliveryJob> {
        let mut heap = self.scheduled.lock().expect("queue lock poisoned");
        let mut batch = Vec::new();
        while batch.len() < self.config.max_ready {
            match heap.peek() {
                Some(top) if top.at <= now => {
                    batch.push(heap.pop().expect("peeked entry").job);
                }
                _ => break,
            }
        }
        batch
    }

    /// Runs one delivery cycle at logical time `now`: pulls the ready batch and
    /// attempts each job through `backend`, applying suppression, shaping,
    /// retry/backoff and the max-age cutoff. Returns the per-cycle counts.
    pub async fn process_due<B: DeliveryBackend>(&self, now: Instant, backend: &B) -> CycleReport {
        let mut report = CycleReport::default();

        for mut job in self.drain_due(now) {
            // Drop recipients already on the suppression list.
            job.recipients.retain(|r| !self.suppress.is_suppressed(r));
            if job.recipients.is_empty() {
                report.skipped_suppressed += 1;
                crate::metrics::queue_depth_dec();
                continue;
            }

            // Rate shaping: if the destination is over its rate, reschedule.
            let wait = self.shaper.acquire_rate(&job.site_name, now);
            if wait > Duration::ZERO {
                job.next_attempt_at = now + wait;
                self.push(job);
                report.throttled += 1;
                continue;
            }

            // Hold a connection slot for the destination during the attempt.
            let _permit = self.shaper.acquire_connection(&job.site_name).await;

            match backend.deliver(&job).await {
                DeliveryOutcome::Delivered => {
                    report.delivered += 1;
                    crate::metrics::queue_depth_dec();
                }
                DeliveryOutcome::Permanent(_) => {
                    for r in &job.recipients {
                        self.suppress.suppress(r, SuppressionReason::HardBounce);
                    }
                    report.suppressed += 1;
                    crate::metrics::queue_depth_dec();
                }
                DeliveryOutcome::Transient(_) => {
                    job.attempts += 1;
                    let age = now.saturating_duration_since(job.enqueued_at);
                    if job.attempts >= self.config.retry.max_attempts
                        || age >= self.config.retry.max_age
                    {
                        for r in &job.recipients {
                            self.suppress
                                .suppress(r, SuppressionReason::SoftBounceLimit);
                        }
                        report.expired += 1;
                        crate::metrics::queue_depth_dec();
                    } else {
                        job.next_attempt_at = now + self.config.retry.backoff(job.attempts);
                        self.push(job);
                        report.retried += 1;
                    }
                }
            }
        }

        report
    }

    /// Runs the queue as a daemon: processes due jobs, then sleeps until the
    /// next job is due (bounded by `poll`). Intended to be `tokio::spawn`ed.
    /// Loops forever; drop the task or the queue to stop it.
    pub async fn run<B: DeliveryBackend>(&self, backend: &B, poll: Duration) {
        loop {
            self.process_due(Instant::now(), backend).await;
            let sleep = match self.next_due() {
                Some(at) => at.saturating_duration_since(Instant::now()).min(poll),
                None => poll,
            };
            tokio::time::sleep(sleep).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex as StdMutex;

    use super::super::shaping::{ShapingConfig, ShapingLimits};

    /// Backend that returns a scripted sequence of outcomes, recording calls.
    struct ScriptBackend {
        outcomes: StdMutex<std::collections::VecDeque<DeliveryOutcome>>,
        default: DeliveryOutcome,
        calls: StdMutex<u32>,
    }
    impl ScriptBackend {
        fn new(outcomes: Vec<DeliveryOutcome>, default: DeliveryOutcome) -> Self {
            Self {
                outcomes: StdMutex::new(outcomes.into()),
                default,
                calls: StdMutex::new(0),
            }
        }
        fn calls(&self) -> u32 {
            *self.calls.lock().unwrap()
        }
    }
    #[async_trait]
    impl DeliveryBackend for ScriptBackend {
        async fn deliver(&self, _job: &DeliveryJob) -> DeliveryOutcome {
            *self.calls.lock().unwrap() += 1;
            self.outcomes
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| self.default.clone())
        }
    }

    fn shaper_unlimited() -> Shaper {
        Shaper::new(ShapingConfig::new(ShapingLimits::UNLIMITED))
    }

    fn job(now: Instant) -> DeliveryJob {
        DeliveryJob::new(
            "t1",
            "default",
            "example.com",
            "mx.example.com",
            "from@send.example",
            vec!["a@example.com".to_owned()],
            b"From: from@send.example\r\nSubject: hi\r\n\r\nbody\r\n".to_vec(),
            now,
        )
    }

    #[test]
    fn backoff_is_exponential_and_capped() {
        let p = MtaRetryPolicy {
            max_attempts: 10,
            base_backoff: Duration::from_secs(60),
            max_backoff: Duration::from_secs(600),
            max_age: Duration::from_secs(100_000),
        };
        assert_eq!(p.backoff(1), Duration::from_secs(60));
        assert_eq!(p.backoff(2), Duration::from_secs(120));
        assert_eq!(p.backoff(3), Duration::from_secs(240));
        // Capped at max_backoff.
        assert_eq!(p.backoff(20), Duration::from_secs(600));
    }

    #[test]
    fn scheduled_key_format() {
        let now = Instant::now();
        assert_eq!(job(now).scheduled_key(), "t1:default:example.com");
    }

    #[tokio::test]
    async fn delivered_job_leaves_the_queue() {
        let q = MtaQueue::new(QueueConfig::default(), shaper_unlimited());
        let now = Instant::now();
        q.enqueue(job(now));
        assert_eq!(q.len(), 1);
        let backend =
            ScriptBackend::new(vec![DeliveryOutcome::Delivered], DeliveryOutcome::Delivered);
        let report = q.process_due(now, &backend).await;
        assert_eq!(report.delivered, 1);
        assert!(q.is_empty());
        assert_eq!(backend.calls(), 1);
    }

    #[tokio::test]
    async fn permanent_failure_suppresses_recipients() {
        let q = MtaQueue::new(QueueConfig::default(), shaper_unlimited());
        let now = Instant::now();
        q.enqueue(job(now));
        let backend = ScriptBackend::new(
            vec![DeliveryOutcome::Permanent("550 5.1.1".into())],
            DeliveryOutcome::Delivered,
        );
        let report = q.process_due(now, &backend).await;
        assert_eq!(report.suppressed, 1);
        assert!(q.is_empty());
        assert!(q.suppressions().is_suppressed("a@example.com"));
        assert_eq!(
            q.suppressions().reason("a@example.com"),
            Some(SuppressionReason::HardBounce)
        );
    }

    #[tokio::test]
    async fn transient_failure_reschedules_with_backoff() {
        let q = MtaQueue::new(QueueConfig::default(), shaper_unlimited());
        let now = Instant::now();
        q.enqueue(job(now));
        let backend = ScriptBackend::new(
            vec![DeliveryOutcome::Transient("451 4.7.0".into())],
            DeliveryOutcome::Delivered,
        );
        let report = q.process_due(now, &backend).await;
        assert_eq!(report.retried, 1);
        assert_eq!(q.len(), 1);
        // Not due now; due after the first backoff (60s default).
        let not_due = q.process_due(now, &backend).await;
        assert_eq!(not_due, CycleReport::default());
        let due = q.next_due().unwrap();
        assert!(due >= now + Duration::from_secs(60));
        // It delivers once its time arrives.
        let later = now + Duration::from_secs(60);
        let report = q.process_due(later, &backend).await;
        assert_eq!(report.delivered, 1);
        assert!(q.is_empty());
    }

    #[tokio::test]
    async fn gives_up_after_max_attempts() {
        let config = QueueConfig {
            max_ready: 16,
            retry: MtaRetryPolicy {
                max_attempts: 3,
                base_backoff: Duration::from_secs(1),
                max_backoff: Duration::from_secs(10),
                max_age: Duration::from_secs(100_000),
            },
        };
        let q = MtaQueue::new(config, shaper_unlimited());
        let mut now = Instant::now();
        q.enqueue(job(now));
        let backend = ScriptBackend::new(vec![], DeliveryOutcome::Transient("451".into()));
        // attempt 1 -> retry, attempt 2 -> retry, attempt 3 -> expired
        let r1 = q.process_due(now, &backend).await;
        assert_eq!(r1.retried, 1);
        now += Duration::from_secs(10);
        let r2 = q.process_due(now, &backend).await;
        assert_eq!(r2.retried, 1);
        now += Duration::from_secs(10);
        let r3 = q.process_due(now, &backend).await;
        assert_eq!(r3.expired, 1);
        assert!(q.is_empty());
        // After giving up, the recipient is soft-suppressed.
        assert!(q.suppressions().is_suppressed("a@example.com"));
    }

    #[tokio::test]
    async fn suppressed_recipient_is_skipped() {
        let q = MtaQueue::new(QueueConfig::default(), shaper_unlimited());
        let now = Instant::now();
        q.suppressions()
            .suppress("a@example.com", SuppressionReason::Unsubscribe);
        q.enqueue(job(now));
        let backend = ScriptBackend::new(vec![], DeliveryOutcome::Delivered);
        let report = q.process_due(now, &backend).await;
        assert_eq!(report.skipped_suppressed, 1);
        assert_eq!(backend.calls(), 0);
        assert!(q.is_empty());
    }

    #[tokio::test]
    async fn rate_limit_reschedules_without_delivering() {
        // 1 msg/s, capacity 1: first job consumes the token, second is throttled.
        let shaper = Shaper::new(ShapingConfig::new(ShapingLimits::new(1.0, 0)));
        let q = MtaQueue::new(QueueConfig::default(), shaper);
        let now = Instant::now();
        q.enqueue(job(now));
        q.enqueue(job(now));
        let backend = ScriptBackend::new(vec![], DeliveryOutcome::Delivered);
        let report = q.process_due(now, &backend).await;
        assert_eq!(report.delivered, 1);
        assert_eq!(report.throttled, 1);
        // The throttled job is still queued, due a bit later.
        assert_eq!(q.len(), 1);
        assert!(q.next_due().unwrap() > now);
    }

    #[tokio::test]
    async fn max_ready_caps_the_batch() {
        let config = QueueConfig {
            max_ready: 2,
            retry: MtaRetryPolicy::default(),
        };
        let q = MtaQueue::new(config, shaper_unlimited());
        let now = Instant::now();
        for _ in 0..5 {
            q.enqueue(job(now));
        }
        let backend = ScriptBackend::new(vec![], DeliveryOutcome::Delivered);
        let report = q.process_due(now, &backend).await;
        assert_eq!(report.delivered, 2);
        assert_eq!(q.len(), 3);
    }
}
