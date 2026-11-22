//! Adapter from `ag-edge`'s dependency-free producer port to `ag-workers`.
//!
//! `ag-edge` owns the port and has no dependency on `ag-workers`. This adapter
//! lives in the consuming example, where both crates are available. It performs
//! only enqueue: no leasing loop, scheduler or worker spawn is started here.

use std::sync::Arc;

use ag_edge::producer::{DeferredJobProducer, EnqueueRequest, ProducerError};
use ag_workers::{JobKind, NewJob, PayloadBytes, QueueBackend, QueueError, QueueName};
use async_trait::async_trait;

/// Enqueues edge requests into a shared `ag-workers` queue backend.
pub struct WorkersProducer {
    backend: Arc<dyn QueueBackend>,
}

impl WorkersProducer {
    /// Wraps a queue backend as a producer-only sink.
    pub fn new(backend: Arc<dyn QueueBackend>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl DeferredJobProducer for WorkersProducer {
    async fn enqueue(&self, request: EnqueueRequest) -> Result<String, ProducerError> {
        let job = NewJob::new(
            JobKind::new(request.kind),
            QueueName::new(request.queue),
            PayloadBytes::from_vec(request.payload),
            request.payload_version,
        );
        self.backend
            .enqueue(job)
            .await
            .map(|id| id.to_string())
            .map_err(|err| match err {
                QueueError::Full(queue) => ProducerError::Backpressure {
                    queue: queue.to_string(),
                },
                other => ProducerError::Backend(other.to_string()),
            })
    }
}

#[cfg(test)]
mod tests {
    use ag_edge::producer::emit;
    use ag_workers::{MemoryQueue, QueueName, WorkerId};

    use super::*;

    #[tokio::test]
    async fn port_enqueues_to_memory_backend_and_is_consumed_elsewhere() {
        let backend: Arc<dyn QueueBackend> = Arc::new(MemoryQueue::new());
        let producer = WorkersProducer::new(backend.clone());

        // Edge side: emit an opaque request through the port. No worker loop here.
        let payload = vec![0x81, 0xa5, 0x68, 0x65, 0x6c, 0x6c, 0x6f];
        let id = emit(
            &producer,
            EnqueueRequest::new("deliver_webhook", "webhooks", payload.clone(), 1),
        )
        .await
        .expect("emit");
        assert!(!id.is_empty());

        let queue = QueueName::new("webhooks");
        assert_eq!(backend.depth(&queue).await.expect("depth").total(), 1);

        // Separate consumer side: lease and acknowledge without touching the port.
        let leased = backend
            .lease(&queue, &WorkerId::from_string("worker-1"), 1)
            .await
            .expect("lease");
        assert_eq!(leased.len(), 1);
        assert_eq!(leased[0].payload_version, 1);
        assert_eq!(leased[0].payload.as_slice(), payload.as_slice());
        backend.ack(leased[0].id).await.expect("ack");
        assert_eq!(backend.depth(&queue).await.expect("depth").total(), 0);
    }
}
