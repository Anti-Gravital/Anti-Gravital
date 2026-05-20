//! Nucleo de Anti-Gravital: runtime HTTP, pipeline Shield y extractores tipados.
//!
//! `ag-core` es la pieza obligatoria del ecosistema Anti-Gravital. Provee
//! la pipeline de seguridad **Shield** sobre [Tower] mas la base del
//! router **Core** sobre [Axum] + [Tokio]. Las dos capas conviven en un
//! mismo proceso Rust, se comunican por llamada de funcion (sin IPC ni
//! FFI) y se componen en una unica peticion HTTP de extremo a extremo.
//!
//! [Tower]: https://docs.rs/tower
//! [Axum]: https://docs.rs/axum
//! [Tokio]: https://docs.rs/tokio
//!
//! # Pipeline Shield
//!
//! La pipeline se construye a partir de [`ShieldConfig`] y se aplica
//! a cualquier [`axum::Router`] con [`Shield::apply`]. Cada capa se
//! activa con su feature Cargo y con la seccion correspondiente del
//! TOML de configuracion:
//!
//! | Capa            | Feature        | Seccion TOML       | Que aporta                                        |
//! | --- | --- | --- | --- |
//! | Logging         | `logging`      | (siempre activa)   | Tracing estructurado, latencia por request.       |
//! | Rate limit      | `rate-limit`   | `[rate_limit]`     | Token bucket por IP con `governor`.               |
//! | CORS            | `cors`         | `[cors]`           | Defaults seguros sobre `tower-http`.              |
//! | Auth JWT        | `auth-jwt`     | `[auth]`           | Verificacion `Authorization: Bearer` Ed25519.     |
//! | CSRF            | `csrf`         | `[csrf]`           | Double-submit cookie apatrida.                    |
//! | Validation      | `validation`   | (per-handler)      | Extractor [`ValidatedJson<T>`].                   |
//! | TLS 1.3         | `tls`          | `[tls]`            | Terminacion con `rustls` via [`Shield::serve`].   |
//!
//! [`Shield::apply`]: crate::shield::Shield::apply
//! [`Shield::serve`]: crate::shield::Shield::serve
//! [`ValidatedJson<T>`]: crate::shield::validation::ValidatedJson
//!
//! # Ejemplo minimo
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
//! # Cargar configuracion desde TOML
//!
//! El operador entrega un archivo TOML con todas las secciones
//! relevantes. Vease `crates/ag-core/config.example.toml` y el
//! [capitulo del manual](https://github.com/anti-gravital/anti-gravital/blob/main/docs/manual/01-shield-as-library.md)
//! para una referencia completa.
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
//! # Tipos publicos clave
//!
//! - [`Shield`]: pipeline y helper de servicio.
//! - [`ShieldConfig`]: configuracion completa, deserializable desde
//!   TOML.
//! - [`AgError`] y [`AgResult`]: tipos de error con mapeo automatico a
//!   respuestas HTTP via `axum::response::IntoResponse`.
//! - [`shield::Claims<T>`]: extractor de claims JWT tipados.
//! - [`shield::validation::ValidatedJson<T>`]: extractor JSON con
//!   validacion declarativa via el trait [`shield::validation::Validate`].
//!
//! # Estado del crate
//!
//! Fase 1 (Shield MVP) cerrada en cuanto a contenido en repositorio.
//! Las metricas duras de cierre de fase (throughput, latencia p99,
//! memoria idle, tiempo de arranque) requieren medicion sobre hardware
//! de referencia y se registran en `docs/benchmarks/` siguiendo la
//! plantilla `measurement-template.md`. Vease
//! `docs/roadmap/fase-01-shield-mvp.md` para los criterios completos
//! y `docs/roadmap/STATUS.md` para el estado vivo.
//!
//! # Reglas aplicables
//!
//! - Sin `unsafe` en codigo propio. El lint del workspace tiene
//!   `unsafe_code = "deny"`.
//! - Defaults seguros: las capas que mutan estado (CORS, CSRF,
//!   rate-limit, JWT, TLS) estan deshabilitadas hasta declaracion
//!   explicita en la configuracion.
//! - Claves desconocidas en TOML se rechazan con [`AgError::Config`].

#![deny(missing_docs)]

pub mod config;
pub mod core;
pub mod error;
pub mod runtime;
pub mod shield;

pub use crate::config::ShieldConfig;
pub use crate::core::AppState;
pub use crate::error::{AgError, AgResult};
pub use crate::shield::Shield;
