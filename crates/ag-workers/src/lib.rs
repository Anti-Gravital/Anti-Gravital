//! Rust-native background execution engine for Anti-Gravital.
//!
//! `ag-workers` moves non-request work out of HTTP handlers: typed background jobs,
//! retries, a dead-letter queue, scheduling and worker pools. The in-memory backend is
//! the default; a durable PostgreSQL backend is opt-in behind the `postgres` feature.
//! No external service is required to use the crate (native-first, ADR-0009).
//!
//! Decision: `RFC-0012` / `ADR-0013`. Module documentation:
//! `docs/modules/ag-workers/README.md`.
//!
//! # Status
//!
//! In progress, delivered in sequenced stages (RFC-0012 §5). This stage (S1) provides
//! the foundations — identity types, the job model and state machine, versioned payload
//! encoding, the handler contract and the closed registry. The runtime (queue backends,
//! dispatch loop, poison guard, scheduling, CLI and DSL) lands in later stages. No
//! production-readiness or GA claim is made until the pre-Phase-5 release gate closes.
//!
//! # Example
//!
//! Defining a handler and resolving it through the registry:
//!
//! ```
//! use ag_workers::{
//!     JobContext, JobHandler, JobOutcome, JobPayload, NewJob, WorkerError, WorkerRegistry,
//! };
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct SendWelcome { user_id: u64 }
//! impl JobPayload for SendWelcome { const PAYLOAD_VERSION: u16 = 1; }
//!
//! struct SendWelcomeHandler;
//!
//! #[async_trait::async_trait]
//! impl JobHandler for SendWelcomeHandler {
//!     type Payload = SendWelcome;
//!     fn kind(&self) -> &'static str { "send_welcome" }
//!     async fn handle(&self, _ctx: JobContext, _payload: SendWelcome)
//!         -> Result<JobOutcome, WorkerError>
//!     {
//!         Ok(JobOutcome::Complete)
//!     }
//! }
//!
//! let registry = WorkerRegistry::builder()
//!     .register(SendWelcomeHandler)
//!     .unwrap()
//!     .build();
//! assert!(registry.contains("send_welcome"));
//!
//! let _job = NewJob::encode::<SendWelcome>("send_welcome", "mail", &SendWelcome { user_id: 7 })
//!     .unwrap();
//! ```

pub mod admission;
pub mod backoff;
pub mod config;
pub mod context;
pub mod error;
pub mod handler;
pub mod ids;
pub mod job;
pub mod outcome;
pub mod payload;
pub mod queue;
pub mod registry;
pub mod runtime;
pub mod shutdown;
pub mod supervisor;
pub mod telemetry;

pub use admission::AdmissionOutcome;
pub use backoff::RetryPolicy;
pub use config::RuntimeConfig;
pub use context::JobContext;
pub use error::{WorkerError, WorkerErrorKind};
pub use handler::JobHandler;
pub use ids::{JobId, JobKind, QueueName, TenantId, WorkerId};
pub use job::{JobEnvelope, JobPriority, JobStatus, NewJob, DEFAULT_MAX_ATTEMPTS, DEFAULT_TIMEOUT};
pub use outcome::{DeadLetterReason, JobOutcome, RetryDelay};
pub use payload::{decode, decode_versioned, encode, JobPayload, PayloadBytes, PayloadError};
pub use queue::dlq::DeadLetter;
pub use queue::memory::MemoryQueue;
#[cfg(feature = "postgres")]
pub use queue::postgres::PostgresQueue;
pub use queue::{QueueBackend, QueueDepth, QueueError, RetryDecision};
pub use registry::{RegistryError, WorkerRegistry, WorkerRegistryBuilder};
pub use runtime::{execute_leased_job, ExecResult};
pub use shutdown::Shutdown;
pub use supervisor::WorkerPool;
