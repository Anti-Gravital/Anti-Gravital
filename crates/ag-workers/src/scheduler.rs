//! Delayed and fixed-interval scheduling with distributed-singleton semantics.
//!
//! Delayed jobs are just an `available_at` in the future (handled by the backends).
//! Fixed-interval jobs are recurring definitions: a single tick is claimed atomically so
//! that, across N worker processes, an interval job fires once — not N times. With the
//! memory store (single process) this is trivially satisfied; the PostgreSQL store uses
//! `FOR UPDATE SKIP LOCKED` on `ag_worker_schedules`.

use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::ids::{JobKind, QueueName};
use crate::job::NewJob;
use crate::payload::PayloadBytes;
use crate::queue::{QueueBackend, QueueError};
use crate::shutdown::Shutdown;

/// What to do when interval ticks were missed (e.g. the fleet was down).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MisfirePolicy {
    /// Drop missed ticks; resume at the next future tick. The safe default.
    #[default]
    Skip,
    /// Run at most one catch-up tick.
    RunOnce,
}

/// A recurring interval schedule definition.
#[derive(Debug, Clone)]
pub struct Schedule {
    /// The job kind to enqueue on each tick.
    pub kind: JobKind,
    /// The queue to enqueue into.
    pub queue: QueueName,
    /// The payload to enqueue (same on every tick).
    pub payload: PayloadBytes,
    /// Payload schema version.
    pub payload_version: u16,
    /// Interval between ticks.
    pub interval: Duration,
    /// When the next tick is due.
    pub next_run_at: DateTime<Utc>,
    /// Misfire handling.
    pub misfire: MisfirePolicy,
}

impl Schedule {
    /// Builds a schedule whose first tick is one `interval` from `now`.
    pub fn every(
        kind: impl Into<JobKind>,
        queue: impl Into<QueueName>,
        payload: PayloadBytes,
        payload_version: u16,
        interval: Duration,
        now: DateTime<Utc>,
    ) -> Self {
        let step =
            chrono::Duration::from_std(interval).unwrap_or_else(|_| chrono::Duration::zero());
        Self {
            kind: kind.into(),
            queue: queue.into(),
            payload,
            payload_version,
            interval,
            next_run_at: now + step,
            misfire: MisfirePolicy::Skip,
        }
    }

    /// Advances `next_run_at` past `now` according to the misfire policy, returning the
    /// number of ticks to emit now (0 or 1 in v1).
    fn advance(&mut self, now: DateTime<Utc>) -> u32 {
        if self.next_run_at > now {
            return 0;
        }
        let step = chrono::Duration::from_std(self.interval)
            .unwrap_or_else(|_| chrono::Duration::seconds(1));
        let step = step.max(chrono::Duration::milliseconds(1));
        match self.misfire {
            MisfirePolicy::Skip => {
                // Jump to the next future tick, dropping any missed ones.
                while self.next_run_at <= now {
                    self.next_run_at += step;
                }
            }
            MisfirePolicy::RunOnce => {
                self.next_run_at += step;
                if self.next_run_at <= now {
                    self.next_run_at = now + step;
                }
            }
        }
        1
    }
}

/// A claimed schedule tick: the job to enqueue.
#[derive(Debug, Clone)]
pub struct ScheduleTick {
    /// The job kind.
    pub kind: JobKind,
    /// The queue.
    pub queue: QueueName,
    /// The payload bytes.
    pub payload: PayloadBytes,
    /// Payload schema version.
    pub payload_version: u16,
}

impl ScheduleTick {
    /// Builds the enqueue request for this tick.
    pub fn into_new_job(self) -> NewJob {
        NewJob::new(self.kind, self.queue, self.payload, self.payload_version)
    }
}

/// A store of recurring schedules that hands out due ticks as a singleton.
#[async_trait]
pub trait ScheduleStore: Send + Sync {
    /// Atomically claims all schedules due at `now`, advancing each so the same tick is
    /// never handed out twice (the singleton property), and returns their ticks.
    async fn claim_due(&self, now: DateTime<Utc>) -> Result<Vec<ScheduleTick>, QueueError>;
}

/// In-memory schedule store (single process: trivially singleton).
#[derive(Default)]
pub struct MemoryScheduleStore {
    schedules: Mutex<Vec<Schedule>>,
}

impl MemoryScheduleStore {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a recurring schedule.
    pub fn register(&self, schedule: Schedule) {
        self.schedules
            .lock()
            .expect("schedule mutex poisoned")
            .push(schedule);
    }
}

#[async_trait]
impl ScheduleStore for MemoryScheduleStore {
    async fn claim_due(&self, now: DateTime<Utc>) -> Result<Vec<ScheduleTick>, QueueError> {
        let mut schedules = self.schedules.lock().expect("schedule mutex poisoned");
        let mut ticks = Vec::new();
        for s in schedules.iter_mut() {
            if s.advance(now) > 0 {
                ticks.push(ScheduleTick {
                    kind: s.kind.clone(),
                    queue: s.queue.clone(),
                    payload: s.payload.clone(),
                    payload_version: s.payload_version,
                });
            }
        }
        Ok(ticks)
    }
}

/// Runs the scheduler loop: claim due ticks, enqueue them, sleep, until shutdown.
pub async fn run_scheduler<S>(
    store: S,
    backend: std::sync::Arc<dyn QueueBackend>,
    poll_interval: Duration,
    shutdown: Shutdown,
) where
    S: ScheduleStore,
{
    while !shutdown.is_triggered() {
        match store.claim_due(Utc::now()).await {
            Ok(ticks) => {
                for tick in ticks {
                    if let Err(e) = backend.enqueue(tick.into_new_job()).await {
                        tracing::error!(error = %e, "scheduler enqueue failed");
                    }
                }
            }
            Err(e) => tracing::error!(error = %e, "scheduler claim failed"),
        }
        tokio::select! {
            biased;
            _ = shutdown.wait() => break,
            _ = tokio::time::sleep(poll_interval) => {}
        }
    }
}

/// PostgreSQL schedule store: a single due tick is claimed with `FOR UPDATE SKIP
/// LOCKED` so an interval job fires once across N processes (the v1 distributed
/// singleton). Available with the `postgres` feature.
#[cfg(feature = "postgres")]
pub struct PostgresScheduleStore {
    pool: ag_data::DbPool,
}

#[cfg(feature = "postgres")]
impl PostgresScheduleStore {
    /// Builds the store over an existing `ag-data` pool.
    pub fn new(pool: ag_data::DbPool) -> Self {
        Self { pool }
    }

    /// Registers a recurring schedule.
    pub async fn register(&self, schedule: &Schedule) -> Result<(), QueueError> {
        sqlx::query(
            "INSERT INTO ag_worker_schedules \
                 (id, kind, queue, payload, payload_version, interval_ms, next_run_at, misfire) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(schedule.kind.as_str())
        .bind(schedule.queue.as_str())
        .bind(schedule.payload.as_slice())
        .bind(schedule.payload_version as i32)
        .bind(schedule.interval.as_millis() as i64)
        .bind(schedule.next_run_at)
        .bind(match schedule.misfire {
            MisfirePolicy::Skip => "skip",
            MisfirePolicy::RunOnce => "run_once",
        })
        .execute(&self.pool)
        .await
        .map_err(|e| QueueError::Backend(e.to_string()))?;
        Ok(())
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl ScheduleStore for PostgresScheduleStore {
    async fn claim_due(&self, now: DateTime<Utc>) -> Result<Vec<ScheduleTick>, QueueError> {
        use sqlx::Row;
        // Claim every due schedule and advance it to the next future tick (skip
        // semantics), atomically, so concurrent processes never double-fire.
        let rows = sqlx::query(
            "WITH due AS ( \
                 SELECT id FROM ag_worker_schedules \
                 WHERE next_run_at <= $1 \
                 FOR UPDATE SKIP LOCKED \
             ) \
             UPDATE ag_worker_schedules s \
             SET next_run_at = now() + (s.interval_ms * interval '1 millisecond'), \
                 updated_at = now() \
             FROM due WHERE s.id = due.id \
             RETURNING s.kind, s.queue, s.payload, s.payload_version",
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| QueueError::Backend(e.to_string()))?;

        rows.iter()
            .map(|row| {
                Ok(ScheduleTick {
                    kind: JobKind::new(
                        row.try_get::<String, _>("kind")
                            .map_err(|e| QueueError::Backend(e.to_string()))?,
                    ),
                    queue: QueueName::new(
                        row.try_get::<String, _>("queue")
                            .map_err(|e| QueueError::Backend(e.to_string()))?,
                    ),
                    payload: PayloadBytes::from_vec(
                        row.try_get::<Vec<u8>, _>("payload")
                            .map_err(|e| QueueError::Backend(e.to_string()))?,
                    ),
                    payload_version: row
                        .try_get::<i32, _>("payload_version")
                        .map_err(|e| QueueError::Backend(e.to_string()))?
                        as u16,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schedule(now: DateTime<Utc>) -> Schedule {
        Schedule::every(
            "cleanup",
            "maintenance",
            PayloadBytes::from_vec(vec![]),
            1,
            Duration::from_millis(10),
            now,
        )
    }

    #[tokio::test]
    async fn claims_due_tick_once() {
        let now = Utc::now();
        let store = MemoryScheduleStore::new();
        store.register(schedule(now));

        // Not due yet (first tick is now+10ms).
        assert!(store.claim_due(now).await.unwrap().is_empty());

        // Due after the interval: exactly one tick, and not handed out again.
        let later = now + chrono::Duration::milliseconds(15);
        let ticks = store.claim_due(later).await.unwrap();
        assert_eq!(ticks.len(), 1);
        assert_eq!(ticks[0].kind.as_str(), "cleanup");
        assert!(
            store.claim_due(later).await.unwrap().is_empty(),
            "the same tick is never claimed twice (singleton)"
        );
    }

    #[test]
    fn skip_policy_drops_missed_ticks() {
        let now = Utc::now();
        let mut s = schedule(now);
        s.misfire = MisfirePolicy::Skip;
        // Far in the future relative to a 10ms interval: many ticks missed.
        let far = now + chrono::Duration::seconds(5);
        assert_eq!(s.advance(far), 1);
        assert!(s.next_run_at > far, "skipped to the next future tick");
    }
}
