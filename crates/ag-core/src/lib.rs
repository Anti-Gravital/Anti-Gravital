//! Anti-Gravital core: HTTP runtime, Shield pipeline and typed extractors.
//!
//! `ag-core` is the mandatory piece of the Anti-Gravital ecosystem. It
//! provides the **Shield** security pipeline on top of [Tower] plus the
//! base of the **Core** router on top of [Axum] + [Tokio]. The two layers
//! coexist in a single Rust process, communicate by function call (no IPC
//! or FFI) and compose into a single end-to-end HTTP request.
//!
//! [Tower]: https://docs.rs/tower
//! [Axum]: https://docs.rs/axum
//! [Tokio]: https://docs.rs/tokio
//!
//! # Shield pipeline
//!
//! The pipeline is built from [`ShieldConfig`] and is applied to any
//! [`axum::Router`] with [`Shield::apply`]. Each layer is enabled with
//! its Cargo feature and the corresponding section of the configuration
//! TOML:
//!
//! | Layer           | Feature        | TOML section       | What it provides                                  |
//! | --- | --- | --- | --- |
//! | Logging         | `logging`      | (always active)    | Structured tracing, per-request latency.          |
//! | Rate limit      | `rate-limit`   | `[rate_limit]`     | Per-IP token bucket with `governor`.              |
//! | CORS            | `cors`         | `[cors]`           | Secure defaults on top of `tower-http`.           |
//! | Auth JWT        | `auth-jwt`     | `[auth]`           | Ed25519 `Authorization: Bearer` verification.     |
//! | CSRF            | `csrf`         | `[csrf]`           | Stateless double-submit cookie.                   |
//! | Validation      | `validation`   | (per-handler)      | The [`ValidatedJson<T>`] extractor.               |
//! | TLS 1.3         | `tls`          | `[tls]`            | Termination with `rustls` via [`Shield::serve`].  |
//!
//! [`Shield::apply`]: crate::shield::Shield::apply
//! [`Shield::serve`]: crate::shield::Shield::serve
//! [`ValidatedJson<T>`]: crate::shield::validation::ValidatedJson
//!
//! # Minimal example
//!
//! ```no_run
//! use ag_core::{Shield, ShieldConfig};
//! use axum::routing::get;
//! use axum::Router;
//!
//! # async fn run() -> Result<(), ag_core::AgError> {
//! let shield = Shield::try_new(ShieldConfig::default())?;
//! let app = shield.apply(
//!     Router::new().route("/", get(|| async { "hello, shield" })),
//! );
//!
//! let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
//! shield.serve(listener, app).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Loading configuration from TOML
//!
//! The operator provides a TOML file with all the relevant sections.
//! See `crates/ag-core/config.example.toml` and the
//! [manual chapter](https://github.com/anti-gravital/anti-gravital/blob/main/docs/manual/01-shield-as-library.md)
//! for a complete reference.
//!
//! ```no_run
//! use ag_core::{Shield, ShieldConfig};
//!
//! # async fn run() -> Result<(), ag_core::AgError> {
//! let config = ShieldConfig::from_path("config.toml")?;
//! let shield = Shield::try_new(config)?;
//! # let _ = shield;
//! # Ok(())
//! # }
//! ```
//!
//! # Secrets and declared environment variables
//!
//! `ag-core` also provides the two building blocks of RFC-0020 stage B.
//! [`Secret<T>`] redacts `Debug`/`Display`, has no `Serialize` implementation,
//! zeroes its bytes on drop and is only readable through `expose()`.
//! [`env::EnvLoader`] resolves declared keys from the process environment
//! and a declared default, validates their types and returns every problem
//! at once so the composition root can fail before opening a listener:
//!
//! ```no_run
//! use ag_core::{EnvDeclaration, EnvLoader, EnvType, Secret};
//!
//! # fn run() -> Result<(), ag_core::EnvValidationError> {
//! let loader = EnvLoader::new(vec![
//!     EnvDeclaration::new("DATABASE_URL", EnvType::Url).secret(true),
//!     EnvDeclaration::new("PORT", EnvType::Int).with_default("8080"),
//! ]);
//! let env = loader.load_process()?;
//! let url: &Secret<String> = env.secret("DATABASE_URL").unwrap();
//! let _ = (url.expose(), env.int("PORT"));
//! # Ok(())
//! # }
//! ```
//!
//! This API is additive: existing crates keep their direct `std::env::var`
//! calls and migrate one crate per pull request. The encrypted secrets file
//! and the `ag env` command are later stages of RFC-0020.
//!
//! # Key public types
//!
//! - [`Shield`]: pipeline and service helper.
//! - [`ShieldConfig`]: complete configuration, deserializable from
//!   TOML.
//! - [`Secret<T>`]: confidential value with redacted formatting and
//!   zeroization on drop (RFC-0020).
//! - [`EnvLoader`] and [`EnvDeclaration`]: declared, fail-fast environment
//!   loading (RFC-0020).
//! - [`AgError`] and [`AgResult`]: error types with automatic mapping to
//!   HTTP responses via `axum::response::IntoResponse`.
//! - [`shield::Claims<T>`]: extractor for typed JWT claims.
//! - [`shield::validation::ValidatedJson<T>`]: JSON extractor with
//!   declarative validation via the [`shield::validation::Validate`] trait.
//!
//! # Crate status
//!
//! Phase 1 (Shield MVP) closed in terms of repository content. The hard
//! phase-closure metrics (throughput, p99 latency, idle memory, startup
//! time) require measurement on reference hardware and are recorded in
//! `docs/benchmarks/` following the `measurement-template.md` template.
//! See `docs/roadmap/fase-01-shield-mvp.md` for the complete criteria and
//! `docs/roadmap/STATUS.md` for the live status.
//!
//! # Applicable rules
//!
//! - No `unsafe` in our own code. The workspace lint has
//!   `unsafe_code = "deny"`.
//! - Secure defaults: layers that mutate state (CORS, CSRF, rate-limit,
//!   JWT, TLS) are disabled until explicitly declared in the
//!   configuration.
//! - Unknown keys in TOML are rejected with [`AgError::Config`].

#![deny(missing_docs)]

pub mod config;
pub mod core;
pub mod env;
pub mod error;
pub mod runtime;
pub mod secret;
pub mod shield;

pub use crate::config::ShieldConfig;
pub use crate::core::AppState;
pub use crate::env::{EnvDeclaration, EnvLoader, EnvType, EnvValidationError, LoadedEnv};
pub use crate::error::{AgError, AgResult};
pub use crate::secret::Secret;
pub use crate::shield::Shield;
