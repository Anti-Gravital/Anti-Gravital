//! Tests E2E de la capa CSRF (double-submit cookie).

#![cfg(feature = "csrf")]

use ag_core::config::CsrfConfig;
use ag_core::{Shield, ShieldConfig};
use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;

async fn start_server(csrf: CsrfConfig) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let config = ShieldConfig {
        csrf,
        ..ShieldConfig::default()
    };
    let shield = Shield::try_new(config).unwrap();
    let app = shield.apply(
        Router::new()
            .route("/get", get(|| async { "ok-get" }))
            .route("/post", post(|| async { "ok-post" })),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });
    (addr, handle)
}

fn enabled_config() -> CsrfConfig {
    CsrfConfig {
        enabled: true,
        ..CsrfConfig::default()
    }
}

#[tokio::test]
async fn get_request_skips_csrf_check() {
    let (addr, handle) = start_server(enabled_config()).await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/get"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert_eq!(resp.text().await.unwrap(), "ok-get");
    handle.abort();
}

#[tokio::test]
async fn post_without_token_returns_403() {
    let (addr, handle) = start_server(enabled_config()).await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/post"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::FORBIDDEN);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "csrf_error");
    handle.abort();
}

#[tokio::test]
async fn post_with_matching_header_and_cookie_passes() {
    let (addr, handle) = start_server(enabled_config()).await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/post"))
        .header("x-csrf-token", "abc123")
        .header("cookie", "ag_csrf=abc123")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert_eq!(resp.text().await.unwrap(), "ok-post");
    handle.abort();
}

#[tokio::test]
async fn post_with_mismatched_token_returns_403() {
    let (addr, handle) = start_server(enabled_config()).await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/post"))
        .header("x-csrf-token", "header-value")
        .header("cookie", "ag_csrf=cookie-value")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::FORBIDDEN);
    handle.abort();
}

#[tokio::test]
async fn post_with_only_header_returns_403() {
    let (addr, handle) = start_server(enabled_config()).await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/post"))
        .header("x-csrf-token", "abc")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::FORBIDDEN);
    handle.abort();
}

#[tokio::test]
async fn csrf_disabled_lets_post_through() {
    let cfg = CsrfConfig {
        enabled: false,
        ..CsrfConfig::default()
    };
    let (addr, handle) = start_server(cfg).await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/post"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    handle.abort();
}
