//! The closed handler registry.
//!
//! A job kind resolves to exactly one handler. The registry is closed: a job whose kind
//! is not registered is dead-lettered as `InvalidPayload` at dispatch time rather than
//! executed. Type erasure lets the runtime dispatch a `JobEnvelope` (opaque bytes +
//! version) to the correct typed handler without the runtime knowing the payload type.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;

use crate::context::JobContext;
use crate::error::WorkerError;
use crate::handler::JobHandler;
use crate::job::JobEnvelope;
use crate::outcome::JobOutcome;
use crate::payload::{decode_versioned, PayloadBytes};

/// Error building a registry.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegistryError {
    /// Two handlers claimed the same kind.
    #[error("duplicate handler kind: {0}")]
    DuplicateKind(&'static str),
}

// Object-safe handler: decodes the concrete payload and invokes `handle`. Kept private
// so the public API never exposes the erased trait object.
#[async_trait]
trait ErasedHandler: Send + Sync {
    fn payload_version(&self) -> u16;
    async fn dispatch(
        &self,
        ctx: JobContext,
        payload: &PayloadBytes,
        stored_version: u16,
    ) -> Result<JobOutcome, WorkerError>;
}

#[async_trait]
impl<H: JobHandler> ErasedHandler for H {
    fn payload_version(&self) -> u16 {
        JobHandler::payload_version(self)
    }

    async fn dispatch(
        &self,
        ctx: JobContext,
        payload: &PayloadBytes,
        stored_version: u16,
    ) -> Result<JobOutcome, WorkerError> {
        // A decode/version failure becomes an `InvalidPayload` WorkerError (never a
        // panic, never a silent drop): see `From<PayloadError> for WorkerError`.
        let decoded = decode_versioned::<H::Payload>(payload, stored_version)?;
        self.handle(ctx, decoded).await
    }
}

/// Incremental builder for a [`WorkerRegistry`].
#[derive(Default)]
pub struct WorkerRegistryBuilder {
    handlers: HashMap<&'static str, Arc<dyn ErasedHandler>>,
}

impl WorkerRegistryBuilder {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a handler, failing if its kind is already taken (closed set).
    pub fn register<H: JobHandler>(mut self, handler: H) -> Result<Self, RegistryError> {
        let kind = JobHandler::kind(&handler);
        if self.handlers.contains_key(kind) {
            return Err(RegistryError::DuplicateKind(kind));
        }
        self.handlers.insert(kind, Arc::new(handler));
        Ok(self)
    }

    /// Finalizes the registry.
    pub fn build(self) -> WorkerRegistry {
        WorkerRegistry {
            handlers: self.handlers,
        }
    }
}

/// An immutable, closed set of handlers keyed by job kind.
pub struct WorkerRegistry {
    handlers: HashMap<&'static str, Arc<dyn ErasedHandler>>,
}

impl WorkerRegistry {
    /// Starts building a registry.
    pub fn builder() -> WorkerRegistryBuilder {
        WorkerRegistryBuilder::new()
    }

    /// Returns `true` if a handler is registered for `kind`.
    pub fn contains(&self, kind: &str) -> bool {
        self.handlers.contains_key(kind)
    }

    /// Returns the number of registered kinds.
    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    /// Returns `true` if no handlers are registered.
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    /// Iterates over the registered kinds.
    pub fn kinds(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.handlers.keys().copied()
    }

    /// Returns the payload version a handler expects, if the kind is registered.
    pub fn payload_version(&self, kind: &str) -> Option<u16> {
        self.handlers.get(kind).map(|h| h.payload_version())
    }

    /// Dispatches a job envelope to its handler.
    ///
    /// Returns `InvalidPayload` if the kind is not registered (closed set) or the
    /// payload cannot be decoded/migrated. The runtime classifies the returned
    /// [`WorkerError`] for retry/dead-letter handling.
    pub async fn dispatch(
        &self,
        ctx: JobContext,
        envelope: &JobEnvelope,
    ) -> Result<JobOutcome, WorkerError> {
        match self.handlers.get(envelope.kind.as_str()) {
            Some(handler) => {
                handler
                    .dispatch(ctx, &envelope.payload, envelope.payload_version)
                    .await
            }
            None => Err(WorkerError::invalid_payload(format!(
                "unknown job kind: {}",
                envelope.kind
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Instant;

    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use tokio_util::sync::CancellationToken;

    use crate::error::WorkerErrorKind;
    use crate::ids::{JobId, JobKind, QueueName};
    use crate::job::NewJob;
    use crate::payload::{JobPayload, PayloadBytes};

    #[derive(Serialize, Deserialize)]
    struct Greeting {
        name: String,
    }
    impl JobPayload for Greeting {
        const PAYLOAD_VERSION: u16 = 1;
    }

    static CALLS: AtomicU32 = AtomicU32::new(0);

    struct GreetHandler;

    #[async_trait]
    impl JobHandler for GreetHandler {
        type Payload = Greeting;
        fn kind(&self) -> &'static str {
            "greet"
        }
        async fn handle(
            &self,
            _ctx: JobContext,
            payload: Greeting,
        ) -> Result<JobOutcome, WorkerError> {
            assert_eq!(payload.name, "ada");
            CALLS.fetch_add(1, Ordering::SeqCst);
            Ok(JobOutcome::Complete)
        }
    }

    fn test_ctx(env: &JobEnvelope) -> JobContext {
        JobContext::new(
            env.id,
            env.kind.clone(),
            env.queue.clone(),
            1,
            env.max_attempts,
            Instant::now(),
            None,
            None,
            None,
            CancellationToken::new(),
        )
    }

    #[test]
    fn duplicate_kind_rejected() {
        // `.err()` avoids requiring the builder (the Ok type) to be Debug.
        let result = WorkerRegistry::builder()
            .register(GreetHandler)
            .unwrap()
            .register(GreetHandler);
        assert_eq!(result.err(), Some(RegistryError::DuplicateKind("greet")));
    }

    #[tokio::test]
    async fn dispatch_decodes_and_calls_handler() {
        let registry = WorkerRegistry::builder()
            .register(GreetHandler)
            .unwrap()
            .build();
        assert!(registry.contains("greet"));
        assert_eq!(registry.payload_version("greet"), Some(1));

        let env = NewJob::encode::<Greeting>("greet", "default", &Greeting { name: "ada".into() })
            .unwrap()
            .into_envelope(chrono::Utc::now());

        let before = CALLS.load(Ordering::SeqCst);
        let outcome = registry.dispatch(test_ctx(&env), &env).await.unwrap();
        assert_eq!(outcome, JobOutcome::Complete);
        assert_eq!(CALLS.load(Ordering::SeqCst), before + 1);
    }

    #[tokio::test]
    async fn unknown_kind_is_invalid_payload() {
        let registry = WorkerRegistry::builder()
            .register(GreetHandler)
            .unwrap()
            .build();
        let mut env =
            NewJob::encode::<Greeting>("greet", "default", &Greeting { name: "ada".into() })
                .unwrap()
                .into_envelope(chrono::Utc::now());
        env.kind = JobKind::new("does_not_exist");

        let err = registry.dispatch(test_ctx(&env), &env).await.unwrap_err();
        assert_eq!(err.kind(), WorkerErrorKind::InvalidPayload);
    }

    #[tokio::test]
    async fn undecodable_payload_is_invalid_payload() {
        let registry = WorkerRegistry::builder()
            .register(GreetHandler)
            .unwrap()
            .build();
        // Build an envelope whose bytes are not a valid Greeting.
        let mut env = NewJob::new(
            "greet",
            "default",
            PayloadBytes::from_vec(vec![0xff, 0x00]),
            1,
        )
        .into_envelope(chrono::Utc::now());
        env.id = JobId::new();
        env.queue = QueueName::new("default");

        let err = registry.dispatch(test_ctx(&env), &env).await.unwrap_err();
        assert_eq!(err.kind(), WorkerErrorKind::InvalidPayload);
    }
}
