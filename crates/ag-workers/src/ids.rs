//! Identity and value types for the job model.
//!
//! These are small, explicit newtypes rather than bare strings/UUIDs so the rest of
//! the engine cannot accidentally mix, say, a queue name with a job kind. They are
//! cheap to clone and serialize.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stable, unique identifier of a job.
///
/// Backed by a v4 UUID. Generated at enqueue time and preserved across retries,
/// re-leases and dead-lettering so a job can be traced end to end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(Uuid);

impl JobId {
    /// Generates a fresh random job id.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Builds a job id from an existing UUID (e.g. read back from the database).
    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    /// Returns the underlying UUID.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifier of a worker process/instance that leases and executes jobs.
///
/// Combines a stable random component so two workers (even on the same host) never
/// collide. Used as the `locked_by` value when a durable job is leased.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkerId(String);

impl WorkerId {
    /// Generates a fresh worker id.
    pub fn generate() -> Self {
        Self(format!("worker-{}", Uuid::new_v4()))
    }

    /// Builds a worker id from an explicit string (e.g. a configured name).
    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the worker id as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorkerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Name of a queue a job belongs to.
///
/// Queues are logical lanes (for example `mail`, `webhooks`, `maintenance`). Workers
/// lease from one or more named queues.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct QueueName(String);

impl QueueName {
    /// Builds a queue name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Returns the queue name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for QueueName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for QueueName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// The kind of a job: the stable string that resolves a job to its handler.
///
/// The set of kinds is closed (the registry rejects unknown kinds at dispatch time).
/// A handler exposes its kind as a `&'static str`; the envelope stores it as a
/// [`JobKind`] because it may be read back dynamically from a durable backend.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobKind(String);

impl JobKind {
    /// Builds a job kind.
    pub fn new(kind: impl Into<String>) -> Self {
        Self(kind.into())
    }

    /// Returns the kind as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for JobKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for JobKind {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl PartialEq<str> for JobKind {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

/// Optional tenant identifier carried by a job for multi-tenant applications.
///
/// Propagated into [`crate::JobContext`] when present. It is never used as a metric
/// label (cardinality), per the observability rules.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(String);

impl TenantId {
    /// Builds a tenant id.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the tenant id as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_ids_are_unique() {
        assert_ne!(JobId::new(), JobId::new());
    }

    #[test]
    fn job_id_roundtrips_through_uuid() {
        let id = JobId::new();
        assert_eq!(id, JobId::from_uuid(id.as_uuid()));
    }

    #[test]
    fn worker_ids_are_unique() {
        assert_ne!(WorkerId::generate(), WorkerId::generate());
    }

    #[test]
    fn job_kind_compares_against_str() {
        let kind = JobKind::new("send_email");
        assert_eq!(kind, *"send_email");
        assert_eq!(kind.as_str(), "send_email");
    }

    #[test]
    fn queue_name_from_str() {
        let q: QueueName = "mail".into();
        assert_eq!(q.as_str(), "mail");
    }
}
