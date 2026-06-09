//! The handler contract.
//!
//! A [`JobHandler`] binds a job kind to typed work. The associated [`JobHandler::Payload`]
//! is decoded from the stored bytes before `handle` runs; a decode/version failure is
//! turned into an `InvalidPayload` dead-letter by the dispatch path, never a panic.

use async_trait::async_trait;

use crate::context::JobContext;
use crate::error::WorkerError;
use crate::outcome::JobOutcome;
use crate::payload::JobPayload;

/// Typed handler for a single job kind.
///
/// Implementations must be cheap to share (`Send + Sync`) because one handler instance
/// serves all jobs of its kind across the worker pool.
///
/// Handler rules: no `unwrap`/`expect` outside tests; errors are explicit
/// [`WorkerError`] values classified retriable vs permanent; cancellation is observed
/// cooperatively via [`JobContext`]; business dependencies are injected, never global.
#[async_trait]
pub trait JobHandler: Send + Sync + 'static {
    /// The typed payload this handler consumes.
    type Payload: JobPayload + Send + Sync + 'static;

    /// Stable, unique identifier for this job kind. Part of the closed registry.
    fn kind(&self) -> &'static str;

    /// Payload schema version this handler produces/expects.
    ///
    /// Defaults to the payload type's [`JobPayload::PAYLOAD_VERSION`]; override only if
    /// a handler intentionally pins a different version.
    fn payload_version(&self) -> u16 {
        Self::Payload::PAYLOAD_VERSION
    }

    /// Executes the job. Returns a [`JobOutcome`] on success or a [`WorkerError`] on
    /// failure (which the runtime classifies for retry/dead-letter).
    async fn handle(
        &self,
        ctx: JobContext,
        payload: Self::Payload,
    ) -> Result<JobOutcome, WorkerError>;
}
