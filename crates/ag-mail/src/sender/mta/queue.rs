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
//! the trait for real delivery. This is the native, in-memory tier; an opt-in
//! durable spool ([`super::spool`], PostgreSQL behind the `queue-postgres`
//! feature) mirrors the scheduled set so jobs survive a restart, while the
//! in-memory tier stays the default (ADR-0009 rule 2).

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;

use super::shaping::Shaper;
use super::spool::{PersistedJob, Spool, SpoolError};
use super::suppress::{SuppressionList, SuppressionReason};

/// Generates a stable, durable-unique job id (128 random bits, hex-encoded).
///
/// Uniqueness must hold across process restarts and across instances writing to
/// a shared spool, so the id is random rather than a per-process counter. On the
/// (essentially impossible) event that the OS RNG fails, it falls back to a
/// wall-clock + counter mix, which is still unique within and across restarts.
fn new_job_id() -> String {
    static FALLBACK_SEQ: AtomicU64 = AtomicU64::new(0);
    let mut bytes = [0u8; 16];
    if getrandom::getrandom(&mut bytes).is_err() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = u128::from(FALLBACK_SEQ.fetch_add(1, AtomicOrdering::Relaxed));
        bytes = (nanos ^ (seq << 64)).to_be_bytes();
    }
    let mut id = String::with_capacity(32);
    for b in bytes {
        id.push_str(&format!("{b:02x}"));
    }
    id
}

/// A unit of work: one rendered message destined for one recipient domain.
#[derive(Debug, Clone)]
pub struct DeliveryJob {
    /// Stable, durable-unique id. Assigned at construction and preserved across
    /// reschedules; the durable spool keys rows on it.
    pub id: String,
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
            id: new_job_id(),
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

/// Native in-memory two-tier delivery queue, with an optional durable spool.
pub struct MtaQueue {
    scheduled: Mutex<BinaryHeap<Scheduled>>,
    seq: AtomicU64,
    config: QueueConfig,
    shaper: Shaper,
    suppress: SuppressionList,
    /// Opt-in durable mirror. `None` keeps the pure in-memory behaviour.
    spool: Option<Arc<dyn Spool>>,
    /// Reference pair to convert this process's monotonic `Instant`s to
    /// wall-clock for persistence (`Instant`s are meaningless across restarts).
    base_instant: Instant,
    base_wall: SystemTime,
}

impl MtaQueue {
    /// Builds a queue with the given configuration and shaper. No durable spool:
    /// the queue is purely in-memory (the native default, ADR-0009 rule 2).
    pub fn new(config: QueueConfig, shaper: Shaper) -> Self {
        Self {
            scheduled: Mutex::new(BinaryHeap::new()),
            seq: AtomicU64::new(0),
            config,
            shaper,
            suppress: SuppressionList::new(),
            spool: None,
            base_instant: Instant::now(),
            base_wall: SystemTime::now(),
        }
    }

    /// Attaches a durable [`Spool`]. The queue then mirrors its scheduled set to
    /// the spool (write-through) so jobs survive a restart; call [`recover`] on
    /// startup to repopulate from it. The in-memory heap stays the runtime
    /// source of truth.
    ///
    /// [`recover`]: MtaQueue::recover
    #[must_use]
    pub fn with_spool(mut self, spool: Arc<dyn Spool>) -> Self {
        self.spool = Some(spool);
        self
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

    /// Enqueues a job and persists it to the durable spool, if one is attached.
    ///
    /// Unlike the best-effort mirroring during a delivery cycle, the initial
    /// persist is strict: a spool error is returned so the caller knows the job
    /// is not yet durable. With no spool attached this behaves like [`enqueue`]
    /// and never errors.
    ///
    /// [`enqueue`]: MtaQueue::enqueue
    pub async fn enqueue_persistent(&self, job: DeliveryJob) -> Result<(), SpoolError> {
        let persisted = self.to_persisted(&job);
        crate::metrics::queue_depth_inc();
        self.push(job);
        if let Some(spool) = &self.spool {
            spool.upsert(&persisted).await?;
        }
        Ok(())
    }

    /// Repopulates the scheduled queue from the durable spool. Call once on
    /// startup, before the delivery loop runs. Returns the number of jobs
    /// recovered (zero when no spool is attached).
    pub async fn recover(&self) -> Result<usize, SpoolError> {
        let Some(spool) = &self.spool else {
            return Ok(0);
        };
        let jobs = spool.load_all().await?;
        let now = Instant::now();
        let now_wall_ms = now_wall_ms();
        let count = jobs.len();
        for persisted in jobs {
            let job = persisted_to_job(persisted, now, now_wall_ms);
            crate::metrics::queue_depth_inc();
            self.push(job);
        }
        Ok(count)
    }

    /// Best-effort spool upsert during a delivery cycle: logs and swallows
    /// errors, since the in-memory queue remains the source of truth.
    async fn mirror_upsert(&self, persisted: &PersistedJob) {
        if let Some(spool) = &self.spool {
            if let Err(e) = spool.upsert(persisted).await {
                tracing::warn!(error = %e, id = %persisted.id, "mta spool upsert failed");
            }
        }
    }

    /// Best-effort spool removal for a job that has left the queue.
    async fn mirror_remove(&self, id: &str) {
        if let Some(spool) = &self.spool {
            if let Err(e) = spool.remove(id).await {
                tracing::warn!(error = %e, id, "mta spool remove failed");
            }
        }
    }

    /// Converts a job's monotonic timing into the wall-clock representation the
    /// spool stores, using this process's reference pair.
    fn to_persisted(&self, job: &DeliveryJob) -> PersistedJob {
        PersistedJob {
            id: job.id.clone(),
            tenant: job.tenant.clone(),
            campaign: job.campaign.clone(),
            domain: job.domain.clone(),
            site_name: job.site_name.clone(),
            recipients: job.recipients.clone(),
            from: job.from.clone(),
            content: job.content.clone(),
            attempts: job.attempts,
            enqueued_at_ms: self.instant_to_wall_ms(job.enqueued_at),
            next_attempt_at_ms: self.instant_to_wall_ms(job.next_attempt_at),
        }
    }

    /// Maps a monotonic `Instant` to wall-clock UNIX millis via the reference
    /// pair captured when the queue was built.
    fn instant_to_wall_ms(&self, i: Instant) -> i64 {
        let wall = if i >= self.base_instant {
            self.base_wall.checked_add(i - self.base_instant)
        } else {
            self.base_wall.checked_sub(self.base_instant - i)
        };
        let ms = wall
            .and_then(|w| w.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_millis());
        i64::try_from(ms).unwrap_or(i64::MAX)
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
                self.mirror_remove(&job.id).await;
                continue;
            }

            // Rate shaping: if the destination is over its rate, reschedule.
            let wait = self.shaper.acquire_rate(&job.site_name, now);
            if wait > Duration::ZERO {
                job.next_attempt_at = now + wait;
                let persisted = self.to_persisted(&job);
                self.push(job);
                self.mirror_upsert(&persisted).await;
                report.throttled += 1;
                continue;
            }

            // Hold a connection slot for the destination during the attempt.
            let _permit = self.shaper.acquire_connection(&job.site_name).await;

            match backend.deliver(&job).await {
                DeliveryOutcome::Delivered => {
                    report.delivered += 1;
                    crate::metrics::queue_depth_dec();
                    self.mirror_remove(&job.id).await;
                }
                DeliveryOutcome::Permanent(_) => {
                    for r in &job.recipients {
                        self.suppress.suppress(r, SuppressionReason::HardBounce);
                    }
                    report.suppressed += 1;
                    crate::metrics::queue_depth_dec();
                    self.mirror_remove(&job.id).await;
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
                        self.mirror_remove(&job.id).await;
                    } else {
                        job.next_attempt_at = now + self.config.retry.backoff(job.attempts);
                        let persisted = self.to_persisted(&job);
                        self.push(job);
                        self.mirror_upsert(&persisted).await;
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

/// Current wall-clock time as UNIX milliseconds.
fn now_wall_ms() -> i64 {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    i64::try_from(ms).unwrap_or(i64::MAX)
}

/// Rebuilds a [`DeliveryJob`] from its persisted form, mapping wall-clock
/// timestamps back to monotonic `Instant`s relative to `now`. A time already in
/// the past becomes due immediately.
fn persisted_to_job(p: PersistedJob, now: Instant, now_wall_ms: i64) -> DeliveryJob {
    DeliveryJob {
        id: p.id,
        tenant: p.tenant,
        campaign: p.campaign,
        domain: p.domain,
        site_name: p.site_name,
        recipients: p.recipients,
        from: p.from,
        content: p.content,
        attempts: p.attempts,
        enqueued_at: wall_ms_to_instant(now, now_wall_ms, p.enqueued_at_ms),
        next_attempt_at: wall_ms_to_instant(now, now_wall_ms, p.next_attempt_at_ms),
    }
}

/// Maps a wall-clock UNIX-millis timestamp to a monotonic `Instant` relative to
/// `now`/`now_wall_ms`. Past timestamps clamp to at-or-before `now` (due now).
fn wall_ms_to_instant(now: Instant, now_wall_ms: i64, ms: i64) -> Instant {
    if ms >= now_wall_ms {
        now + Duration::from_millis(u64::try_from(ms - now_wall_ms).unwrap_or(0))
    } else {
        now.checked_sub(Duration::from_millis(
            u64::try_from(now_wall_ms - ms).unwrap_or(0),
        ))
        .unwrap_or(now)
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

    // ---- durable spool mechanism (backend-agnostic, in-process) -------------

    use super::super::spool::InMemorySpool;

    #[tokio::test]
    async fn scheduled_jobs_survive_restart_via_spool() {
        let spool = Arc::new(InMemorySpool::new());
        let now = Instant::now();

        // First process lifetime: enqueue two jobs due in the future.
        {
            let q =
                MtaQueue::new(QueueConfig::default(), shaper_unlimited()).with_spool(spool.clone());
            let mut j1 = job(now);
            j1.next_attempt_at = now + Duration::from_secs(3600);
            let mut j2 = job(now);
            j2.next_attempt_at = now + Duration::from_secs(7200);
            q.enqueue_persistent(j1).await.unwrap();
            q.enqueue_persistent(j2).await.unwrap();
            assert_eq!(q.len(), 2);
            assert_eq!(spool.len(), 2);
        } // queue dropped == process crash

        // Second process lifetime: a fresh queue recovers from the same spool.
        let q2 =
            MtaQueue::new(QueueConfig::default(), shaper_unlimited()).with_spool(spool.clone());
        assert_eq!(q2.len(), 0);
        let recovered = q2.recover().await.unwrap();
        assert_eq!(recovered, 2);
        assert_eq!(q2.len(), 2);
    }

    #[tokio::test]
    async fn delivered_job_is_removed_from_spool() {
        let spool = Arc::new(InMemorySpool::new());
        let now = Instant::now();
        let q = MtaQueue::new(QueueConfig::default(), shaper_unlimited()).with_spool(spool.clone());
        q.enqueue_persistent(job(now)).await.unwrap();
        assert_eq!(spool.len(), 1);

        let backend =
            ScriptBackend::new(vec![DeliveryOutcome::Delivered], DeliveryOutcome::Delivered);
        q.process_due(now, &backend).await;

        assert!(q.is_empty());
        assert_eq!(spool.len(), 0);
    }

    #[tokio::test]
    async fn retried_job_is_updated_in_spool() {
        let spool = Arc::new(InMemorySpool::new());
        let now = Instant::now();
        let q = MtaQueue::new(QueueConfig::default(), shaper_unlimited()).with_spool(spool.clone());
        q.enqueue_persistent(job(now)).await.unwrap();

        let backend = ScriptBackend::new(
            vec![DeliveryOutcome::Transient("451 4.7.0".into())],
            DeliveryOutcome::Delivered,
        );
        q.process_due(now, &backend).await;

        // Rescheduled (not delivered): still persisted, attempts bumped to 1.
        assert_eq!(spool.len(), 1);
        let jobs = spool.load_all().await.unwrap();
        assert_eq!(jobs[0].attempts, 1);
    }

    #[tokio::test]
    async fn enqueue_persistent_without_spool_is_inmemory() {
        let now = Instant::now();
        let q = MtaQueue::new(QueueConfig::default(), shaper_unlimited());
        q.enqueue_persistent(job(now)).await.unwrap();
        assert_eq!(q.len(), 1);
        assert_eq!(q.recover().await.unwrap(), 0);
    }
}
