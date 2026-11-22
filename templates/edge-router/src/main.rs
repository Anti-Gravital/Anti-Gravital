//! Hostname-based edge routing with `ag-edge`.
//!
//! Loads bindings and canonical redirect policies from `config.toml`, builds an
//! `ag_edge` data plane and serves HTTP. The resolver applies exact bindings,
//! then wildcard bindings, then the legacy/fallback chain; unknown custom
//! hostnames fail closed with 404. ACME HTTP-01 challenge paths are served from
//! an empty challenge store (populate it from the control plane in production).

use std::path::Path;

use ag_edge::challenge::Http01ChallengeStore;
use ag_edge::redirect::CanonicalPolicy;
use ag_edge::router::{BindingCache, RouteBinding};
use ag_edge::server::{serve_http, EdgeState};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    server: ServerConfig,
    #[serde(default)]
    bindings: Vec<BindingDef>,
    #[serde(default)]
    redirects: Vec<CanonicalPolicy>,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    bind: String,
}

#[derive(Debug, Deserialize)]
struct BindingDef {
    hostname: String,
    project: String,
    environment: String,
    target: String,
}

fn load_config(path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&text)?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config_path = std::env::var("CONFIG").unwrap_or_else(|_| "config.toml".into());
    let cfg = load_config(Path::new(&config_path))?;

    let mut cache = BindingCache::new();
    for binding in &cfg.bindings {
        cache.insert(RouteBinding {
            hostname_ascii: binding.hostname.to_ascii_lowercase(),
            project: binding.project.clone(),
            environment: binding.environment.clone(),
            target_ref: binding.target.clone(),
        });
    }

    let state = EdgeState::new(cache, Http01ChallengeStore::new())
        .with_redirects(cfg.redirects);

    let listener = tokio::net::TcpListener::bind(&cfg.server.bind).await?;
    tracing::info!(bind = %cfg.server.bind, "edge-router listening");
    serve_http(listener, state).await?;
    Ok(())
}
