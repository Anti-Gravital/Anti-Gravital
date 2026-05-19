//! Nucleo de Anti-Gravital: Shield, Core, runtime Tokio y extractores tipados.
//!
//! Fase 1 (en curso): bootstrap del Shield. La pipeline Tower del Shield
//! se construye capa por capa en PRs incrementales descritos en
//! `docs/rfc/RFC-0002-diseno-shield-mvp.md`. Este crate ya compila y
//! expone una API publica minima que sirve HTTP/1.1 y HTTP/2 sobre
//! Axum + Tokio sin TLS, suficiente para el ejemplo Hello World.
//!
//! # Ejemplo
//!
//! ```no_run
//! use ag_core::{Shield, ShieldConfig};
//!
//! # async fn run() -> Result<(), ag_core::AgError> {
//! let config = ShieldConfig::default();
//! let shield = Shield::try_new(config)?;
//! let app = axum::Router::new()
//!     .route("/", axum::routing::get(|| async { "hello, shield" }));
//! let app = shield.apply(app);
//!
//! let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
//! axum::serve(listener, app).await?;
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]

pub mod config;
pub mod core;
pub mod error;
pub mod runtime;
pub mod shield;

pub use crate::config::ShieldConfig;
pub use crate::error::{AgError, AgResult};
pub use crate::shield::Shield;
