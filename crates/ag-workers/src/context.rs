//! Execution context handed to a job handler.
//!
//! [`JobContext`] exposes everything a handler legitimately needs (identity, attempt
//! metadata, deadline, cooperative cancellation, optional tenant/trace context) and
//! nothing it should not. Business dependencies are injected through the registry, not
//! through globals.

use std::time::Instant;

use tokio_util::sync::CancellationToken;

use crate::ids::{JobId, JobKind, QueueName, TenantId};

/// Context passed to [`crate::JobHandler::handle`].
#[derive(Debug, Clone)]
pub struct JobContext {
    /// Stable id of the job being executed.
    pub job_id: JobId,
    /// The job kind.
    pub kind: JobKind,
    /// The queue the job came from.
    pub queue: QueueName,
    /// 1-based attempt number for this execution.
    pub attempt: u32,
    /// Maximum number of attempts configured for the job.
    pub max_attempts: u32,
    /// Wall-clock deadline derived from the per-job timeout.
    pub deadline: Instant,
    /// Optional deduplication key, for idempotent side effects.
    pub dedup_key: Option<String>,
    /// Optional tenant context.
    pub tenant_id: Option<TenantId>,
    /// Optional trace/correlation id propagated from the enqueuing request.
    pub trace_id: Option<String>,
    // Cooperative cancellation; observed by the handler and tripped on shutdown or
    // when the per-job timeout elapses.
    cancel: CancellationToken,
}

impl JobContext {
    /// Builds a job context.
    ///
    /// `deadline` is the instant the per-job timeout elapses; `cancel` is the token the
    /// runtime trips on shutdown or timeout.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        job_id: JobId,
        kind: JobKind,
        queue: QueueName,
        attempt: u32,
        max_attempts: u32,
        deadline: Instant,
        dedup_key: Option<String>,
        tenant_id: Option<TenantId>,
        trace_id: Option<String>,
        cancel: CancellationToken,
    ) -> Self {
        Self {
            job_id,
            kind,
            queue,
            attempt,
            max_attempts,
            deadline,
            dedup_key,
            tenant_id,
            trace_id,
            cancel,
        }
    }

    /// Returns `true` if cancellation has been requested.
    ///
    /// A cooperative handler polls this on long-running work and returns promptly.
    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }

    /// Resolves when cancellation is requested.
    ///
    /// Use inside `tokio::select!` to abandon work cooperatively on shutdown/timeout.
    pub async fn cancelled(&self) {
        self.cancel.cancelled().await
    }

    /// Returns the underlying cancellation token (for `select!` composition).
    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(token: CancellationToken) -> JobContext {
        JobContext::new(
            JobId::new(),
            JobKind::new("k"),
            QueueName::new("q"),
            1,
            5,
            Instant::now(),
            None,
            None,
            None,
            token,
        )
    }

    #[tokio::test]
    async fn cancellation_is_observed() {
        let token = CancellationToken::new();
        let c = ctx(token.clone());
        assert!(!c.is_cancelled());
        token.cancel();
        assert!(c.is_cancelled());
        // `cancelled()` resolves immediately once cancelled.
        c.cancelled().await;
    }
}
