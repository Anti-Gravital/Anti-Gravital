//! Binario de ejemplo para medir el pipeline Shield bajo carga.
//!
//! Arranca un Shield con configuracion minima (solo logging activo)
//! y sirve `GET /` con cuerpo `hello, world`. Pensado para
//! `cargo run --release -p ag-core --example hello_world` seguido de
//! medicion con `oha`, `wrk` o `bombardier` desde un cliente externo.
//!
//! Si se desea evaluar el pipeline con capas adicionales (CORS, CSRF,
//! rate-limit, etc.), configurelo via TOML y use
//! `ShieldConfig::from_path` en lugar del default.

use ag_core::{Shield, ShieldConfig};
use axum::routing::get;
use axum::Router;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = ShieldConfig::default();
    let bind = config.bind;
    let shield = Shield::try_new(config)?;
    let app = shield.apply(Router::new().route("/", get(|| async { "hello, world" })));

    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(addr = %bind, "ag-core hello_world example listening");

    shield.serve(listener, app).await?;
    Ok(())
}
