//! Typed contract shared by both services.
//!
//! Keeping the wire types in one module prevents the gateway and the worker
//! from drifting. This module depends only on `serde` and `serde_json`, so it
//! can later be extracted into a standalone crate without pulling in HTTP or
//! runtime code.

use serde::{Deserialize, Serialize};

/// Version of the wire contract. Bump on incompatible changes.
pub const SCHEMA_VERSION: u32 = 1;

/// A job submitted through the gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnqueueRequest {
    /// Discriminator used by the worker to pick a handler.
    pub job_type: String,
    /// Opaque job payload.
    pub payload: serde_json::Value,
}

/// Acknowledgement returned by the gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnqueueResponse {
    /// Identifier assigned at enqueue time.
    pub job_id: String,
    /// Whether the job was accepted.
    pub accepted: bool,
    /// Contract version used for the exchange.
    pub schema_version: u32,
}

/// Common health payload returned by both services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Name of the service answering.
    pub service: &'static str,
    /// Lifecycle status.
    pub status: &'static str,
}

/// Builds a stable job identifier from a per-process sequence.
pub fn new_job_id(sequence: u64) -> String {
    format!("job-{sequence:08}")
}
