//! Job dispatch: the closed loop that executes a leased job and classifies its result.
//!
//! This is where the poison-job circuit breaker lives. Because the attempt counter is
//! incremented at lease time (see the backend), a job that aborts the process on every
//! execution accumulates attempts purely from re-leases. When that count exceeds
//! [`RuntimeConfig::poison_guard_attempts`], the job is routed straight to the
//! dead-letter queue *before* executing — converting an infinite crash loop (under the
//! workspace `panic = "abort"` profile) into a bounded, observable DLQ entry.

use std::time::Instant;

use chrono::Utc;

use crate::config::RuntimeConfig;
use crate::context::JobContext;
use crate::error::WorkerErrorKind;
use crate::job::JobEnvelope;
use crate::outcome::{DeadLetterReason, JobOutcome, RetryDelay};
use crate::queue::{QueueBackend, QueueError, RetryDecision};
use crate::registry::WorkerRegistry;
use crate::shutdown::Shutdown;
use crate::telemetry;

/// What the dispatch loop did with a job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecResult {
    /// The handler completed; the job was acknowledged.
    Completed,
    /// The handler discarded the job intentionally (recorded no-op); acknowledged.
    Discarded,
    /// The job was scheduled for another attempt.
    Retried,
    /// The job was released for re-lease (cancellation/shutdown).
    Released,
    /// The job was dead-lettered for the given reason.
    DeadLettered(DeadLetterReason),
}

/// Executes one leased job and applies its outcome to the backend.
///
/// `env` must be a job that the backend has already leased (status `leased`, attempt
/// count incremented). Returns the [`ExecResult`] or a [`QueueError`] if a backend
/// operation failed.
pub async fn execute_leased_job(
    backend: &dyn QueueBackend,
    registry: &WorkerRegistry,
    config: &RuntimeConfig,
    shutdown: &Shutdown,
    env: JobEnvelope,
) -> Result<ExecResult, QueueError> {
    let queue = env.queue.as_str().to_string();
    let kind = env.kind.as_str().to_string();

    // Pre-execution guards (no handler runs).
    if config.retry_policy.is_expired(env.created_at, Utc::now()) {
        return dead_letter(backend, &env, DeadLetterReason::Expired, &queue, &kind).await;
    }
    if env.attempts > config.poison_guard_attempts {
        telemetry::record_poison_guard(&queue, &kind);
        return dead_letter(backend, &env, DeadLetterReason::PoisonGuard, &queue, &kind).await;
    }

    // Build the execution context: a child cancellation token (tripped on shutdown) and
    // a deadline derived from the per-job timeout.
    let token = shutdown.token().child_token();
    let deadline = Instant::now() + env.timeout;
    let ctx = JobContext::new(
        env.id,
        env.kind.clone(),
        env.queue.clone(),
        env.attempts,
        env.max_attempts,
        deadline,
        env.dedup_key.clone(),
        env.tenant_id.clone(),
        env.trace_id.clone(),
        token,
    );

    let started = Instant::now();
    let dispatch = registry.dispatch(ctx, &env);
    let result = match tokio::time::timeout(env.timeout, dispatch).await {
        Ok(r) => r,
        Err(_elapsed) => Err(crate::WorkerError::timed_out("job exceeded its timeout")),
    };
    telemetry::record_duration(&queue, &kind, started.elapsed().as_secs_f64());

    match result {
        Ok(JobOutcome::Complete) => {
            backend.ack(env.id).await?;
            telemetry::record_job(&queue, &kind, "succeeded");
            Ok(ExecResult::Completed)
        }
        Ok(JobOutcome::Discard { reason: _ }) => {
            backend.ack(env.id).await?;
            telemetry::record_job(&queue, &kind, "discarded");
            Ok(ExecResult::Discarded)
        }
        Ok(JobOutcome::DeadLetter { reason }) => {
            dead_letter(
                backend,
                &env,
                DeadLetterReason::Requested(reason),
                &queue,
                &kind,
            )
            .await
        }
        Ok(JobOutcome::Retry { delay }) => {
            schedule_retry(backend, config, &env, delay, None, &queue, &kind).await
        }
        Err(err) => match err.kind() {
            WorkerErrorKind::InvalidPayload => {
                dead_letter(
                    backend,
                    &env,
                    DeadLetterReason::InvalidPayload(err.message().to_string()),
                    &queue,
                    &kind,
                )
                .await
            }
            WorkerErrorKind::Permanent => {
                dead_letter(
                    backend,
                    &env,
                    DeadLetterReason::Permanent(err.message().to_string()),
                    &queue,
                    &kind,
                )
                .await
            }
            WorkerErrorKind::Cancelled => {
                // Release immediately for another worker / after restart.
                backend
                    .retry(
                        env.id,
                        RetryDecision {
                            available_at: Utc::now(),
                            last_error: Some(err.message().to_string()),
                        },
                    )
                    .await?;
                Ok(ExecResult::Released)
            }
            WorkerErrorKind::Retriable
            | WorkerErrorKind::TimedOut
            | WorkerErrorKind::DependencyUnavailable => {
                schedule_retry(
                    backend,
                    config,
                    &env,
                    RetryDelay::Policy,
                    Some(err.message().to_string()),
                    &queue,
                    &kind,
                )
                .await
            }
        },
    }
}

/// Schedules a retry, or dead-letters as `AttemptsExhausted` when the cap is reached.
async fn schedule_retry(
    backend: &dyn QueueBackend,
    config: &RuntimeConfig,
    env: &JobEnvelope,
    delay: RetryDelay,
    last_error: Option<String>,
    queue: &str,
    kind: &str,
) -> Result<ExecResult, QueueError> {
    // The effective attempt cap is the lower of the poison-guard threshold and the
    // retry policy's max_attempts. `env.attempts` already counts the current attempt.
    let cap = config
        .poison_guard_attempts
        .min(config.retry_policy.max_attempts);
    if env.attempts >= cap {
        return dead_letter(
            backend,
            env,
            DeadLetterReason::AttemptsExhausted,
            queue,
            kind,
        )
        .await;
    }
    let available_at = match delay {
        RetryDelay::Policy => config
            .retry_policy
            .next_available_at(Utc::now(), env.attempts.saturating_sub(1)),
        RetryDelay::After(d) => {
            Utc::now() + chrono::Duration::from_std(d).unwrap_or_else(|_| chrono::Duration::zero())
        }
    };
    backend
        .retry(
            env.id,
            RetryDecision {
                available_at,
                last_error,
            },
        )
        .await?;
    telemetry::record_retry(queue, kind);
    Ok(ExecResult::Retried)
}

async fn dead_letter(
    backend: &dyn QueueBackend,
    env: &JobEnvelope,
    reason: DeadLetterReason,
    queue: &str,
    kind: &str,
) -> Result<ExecResult, QueueError> {
    telemetry::record_dead_letter(queue, kind, reason.as_str());
    telemetry::record_job(queue, kind, "dead_lettered");
    backend.dead_letter(env.id, reason.clone()).await?;
    Ok(ExecResult::DeadLettered(reason))
}
