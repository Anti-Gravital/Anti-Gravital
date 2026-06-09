//! Shared test handlers and payloads for the ag-workers integration tests.
// Each integration-test binary uses only a subset of these helpers.
#![allow(dead_code)]

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use ag_workers::{JobContext, JobHandler, JobOutcome, JobPayload, NewJob, WorkerError};
use serde::{Deserialize, Serialize};

/// A trivial payload used across the integration tests.
#[derive(Debug, Serialize, Deserialize)]
pub struct Unit {}

impl JobPayload for Unit {
    const PAYLOAD_VERSION: u16 = 1;
}

/// Builds an enqueue request of kind `kind` on queue `queue` with an empty payload.
pub fn unit_job(kind: &str, queue: &str) -> NewJob {
    NewJob::encode::<Unit>(kind, queue, &Unit {}).expect("encode unit payload")
}

/// A handler that counts executions and always completes.
pub struct CountingHandler {
    /// Number of times `handle` ran.
    pub calls: Arc<AtomicU32>,
    kind: &'static str,
}

impl CountingHandler {
    /// Builds a counting handler for `kind`.
    pub fn new(kind: &'static str) -> Self {
        Self {
            calls: Arc::new(AtomicU32::new(0)),
            kind,
        }
    }

    /// Returns the shared call counter.
    pub fn counter(&self) -> Arc<AtomicU32> {
        self.calls.clone()
    }
}

#[async_trait::async_trait]
impl JobHandler for CountingHandler {
    type Payload = Unit;
    fn kind(&self) -> &'static str {
        self.kind
    }
    async fn handle(&self, _ctx: JobContext, _payload: Unit) -> Result<JobOutcome, WorkerError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(JobOutcome::Complete)
    }
}

/// A handler that counts executions and always fails retriably.
pub struct AlwaysRetriable {
    /// Number of times `handle` ran.
    pub calls: Arc<AtomicU32>,
    kind: &'static str,
}

impl AlwaysRetriable {
    /// Builds an always-retriable handler for `kind`.
    pub fn new(kind: &'static str) -> Self {
        Self {
            calls: Arc::new(AtomicU32::new(0)),
            kind,
        }
    }

    /// Returns the shared call counter.
    pub fn counter(&self) -> Arc<AtomicU32> {
        self.calls.clone()
    }
}

#[async_trait::async_trait]
impl JobHandler for AlwaysRetriable {
    type Payload = Unit;
    fn kind(&self) -> &'static str {
        self.kind
    }
    async fn handle(&self, _ctx: JobContext, _payload: Unit) -> Result<JobOutcome, WorkerError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(WorkerError::retriable("transient failure"))
    }
}
