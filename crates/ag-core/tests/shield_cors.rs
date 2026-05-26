//! E2E tests for the CORS layer.

#![cfg(feature = "cors")]

use ag_core::config::CorsConfig;
use ag_core::{Shield, ShieldConfig};
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;

async fn start_server(cors: CorsConfig) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let config = ShieldConfig {
        cors,
        ..ShieldConfig::default()
    };
    let shield = Shield::try_new(config).unwrap();
    let app = shield.apply(Router::new().route("/ping", get(|| async { "pong" })));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });
    (addr, handle)
}

#[tokio::test]
async fn cors_disabled_omits_headers() {
    let cors = CorsConfig::default();
    let (addr, handle) = start_server(cors).await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/ping"))
        .header("origin", "https://other.example")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert!(resp.headers().get("access-control-allow-origin").is_none());
    handle.abort();
}

#[tokio::test]
async fn cors_enabled_allows_listed_origin() {
    let cors = CorsConfig {
        enabled: true,
        allow_origins: vec!["https://app.example.com".into()],
        allow_methods: vec!["GET".into()],
        allow_headers: vec!["content-type".into()],
        allow_credentials: false,
    };
    let (addr, handle) = start_server(cors).await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/ping"))
        .header("origin", "https://app.example.com")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let allow_origin = resp.headers().get("access-control-allow-origin").unwrap();
    assert_eq!(allow_origin, "https://app.example.com");
    handle.abort();
}

#[tokio::test]
async fn cors_enabled_rejects_unlisted_origin() {
    let cors = CorsConfig {
        enabled: true,
        allow_origins: vec!["https://app.example.com".into()],
        allow_methods: vec!["GET".into()],
        allow_headers: vec!["content-type".into()],
        allow_credentials: false,
    };
    let (addr, handle) = start_server(cors).await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/ping"))
        .header("origin", "https://attacker.example")
        .send()
        .await
        .unwrap();
    // The request passes (200 OK), but the browser rejects it due to the
    // absence of the `access-control-allow-origin` header for this origin.
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert!(resp.headers().get("access-control-allow-origin").is_none());
    handle.abort();
}

#[tokio::test]
async fn cors_preflight_options_returns_204_with_allow_methods() {
    let cors = CorsConfig {
        enabled: true,
        allow_origins: vec!["https://app.example.com".into()],
        allow_methods: vec!["GET".into(), "POST".into()],
        allow_headers: vec!["content-type".into()],
        allow_credentials: false,
    };
    let (addr, handle) = start_server(cors).await;
    let client = reqwest::Client::new();
    let resp = client
        .request(reqwest::Method::OPTIONS, format!("http://{addr}/ping"))
        .header("origin", "https://app.example.com")
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", "content-type")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let allow_methods = resp.headers().get("access-control-allow-methods").unwrap();
    let methods_str = allow_methods.to_str().unwrap();
    assert!(methods_str.contains("POST"));
    handle.abort();
}
