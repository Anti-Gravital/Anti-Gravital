//! Example binary to measure the Shield pipeline under load.
//!
//! Starts a Shield with minimal configuration (only logging active) and
//! serves `GET /` with the body `hello, world`. Intended for
//! `cargo run --release -p ag-core --example hello_world` followed by
//! measurement with `oha`, `wrk` or `bombardier` from an external
//! client.
//!
//! To evaluate the pipeline with additional layers (CORS, CSRF,
//! rate-limit, etc.), configure it via TOML and use
//! `ShieldConfig::from_path` instead of the default.

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
