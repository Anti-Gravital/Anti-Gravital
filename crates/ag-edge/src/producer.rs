//! Dependency-free producer port for deferred work emitted at the edge.
//!
//! Some edge/serverless deployments need to hand work off to a background queue
//! without hosting a worker loop. RFC-0012 section 5 (S6) assigns that
//! producer-only wiring to `ag-edge`. This module defines only the contract: an
//! [`EnqueueRequest`] carrying opaque, already-encoded bytes, and a
//! [`DeferredJobProducer`] trait that a host supplies an adapter for.
//!
//! # Dependency direction
//!
//! `ag-edge` does not depend on `ag-workers`, and `ag-workers` must never depend
//! on `ag-edge` (RFC-0012 section 4). The adapter that translates this port into a
//! concrete queue backend belongs to the consuming crate, exactly as a
//! `DnsProvider` adapter belongs to its consumer. The only dependency added by
//! this feature is the optional `async-trait` crate already present in the
//! workspace; no external service or stack change is involved.
//!
//! # Contract and limitations
//!
//! - The port is behavioural, not a link-time guarantee: a producer emits
//!   enqueue requests and hosts no leasing loop, scheduler or spawn. Whether the
//!   request is durable, deduplicated or admitted under backpressure is decided
//!   by the backend adapter, not by this crate.
//! - [`ProducerError::Backpressure`] lets an adapter report admission rejection
//!   without leaking backend-specific types into the edge.
//! - This API is v1 and unstable; implementors should expect additive evolution.

use std::fmt;

/// An opaque deferred-work request produced at the edge.
///
/// The payload is already encoded by the caller (for example, MessagePack for an
/// `ag-workers` typed job), so this crate stays independent of any payload
/// schema or queue backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnqueueRequest {
    /// Stable handler kind the backend must resolve.
    pub kind: String,
    /// Destination queue name.
    pub queue: String,
    /// Already-encoded payload bytes.
    pub payload: Vec<u8>,
    /// Schema version of the encoded payload.
    pub payload_version: u16,
}

impl EnqueueRequest {
    /// Builds an enqueue request from owned, already-encoded bytes.
    pub fn new(
        kind: impl Into<String>,
        queue: impl Into<String>,
        payload: Vec<u8>,
        payload_version: u16,
    ) -> Self {
        Self {
            kind: kind.into(),
            queue: queue.into(),
            payload,
            payload_version,
        }
    }
}

/// Errors reported by a [`DeferredJobProducer`] adapter.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProducerError {
    /// The destination queue rejected the request under admission control.
    Backpressure {
        /// Queue that was at capacity.
        queue: String,
    },
    /// A backend-specific failure, reduced to a redactable message.
    Backend(String),
}

impl fmt::Display for ProducerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backpressure { queue } => {
                write!(f, "producer queue '{queue}' is at capacity")
            }
            Self::Backend(message) => write!(f, "producer backend error: {message}"),
        }
    }
}

impl std::error::Error for ProducerError {}

/// A caller-supplied sink for deferred work emitted at the edge.
///
/// Implementations must only enqueue: they must not lease, dispatch or spawn
/// workers. A producer-only process writes requests to a shared backend and
/// leaves execution to a separate fleet.
#[async_trait::async_trait]
pub trait DeferredJobProducer: Send + Sync + 'static {
    /// Enqueues `request` and returns the backend's opaque job identifier.
    async fn enqueue(&self, request: EnqueueRequest) -> Result<String, ProducerError>;
}

/// Emits a deferred-work request through `producer`.
///
/// This is the edge-owned call shape: the host provides the adapter, while the
/// act of emitting stays in this crate and remains free of backend types.
pub async fn emit<P: DeferredJobProducer>(
    producer: &P,
    request: EnqueueRequest,
) -> Result<String, ProducerError> {
    producer.enqueue(request).await
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    struct RecordingProducer {
        seen: Mutex<Vec<EnqueueRequest>>,
    }

    #[async_trait::async_trait]
    impl DeferredJobProducer for RecordingProducer {
        async fn enqueue(&self, request: EnqueueRequest) -> Result<String, ProducerError> {
            let id = format!("job-{}", self.seen.lock().unwrap().len() + 1);
            self.seen.lock().unwrap().push(request);
            Ok(id)
        }
    }

    #[tokio::test]
    async fn emit_forwards_request_to_adapter() {
        let producer = RecordingProducer {
            seen: Mutex::new(Vec::new()),
        };
        let request = EnqueueRequest::new("deliver_webhook", "webhooks", vec![1, 2, 3], 1);
        let id = emit(&producer, request.clone()).await.expect("emit");
        assert_eq!(id, "job-1");
        assert_eq!(producer.seen.lock().unwrap().as_slice(), &[request]);
    }
}
