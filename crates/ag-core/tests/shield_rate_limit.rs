//! E2E tests for the rate-limit layer (per-IP token bucket).

#![cfg(feature = "rate-limit")]

use ag_core::config::RateLimitConfig;
use ag_core::{Shield, ShieldConfig};
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;

async fn start_server(rl: RateLimitConfig) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let config = ShieldConfig {
        rate_limit: rl,
        ..ShieldConfig::default()
    };
    let shield = Shield::try_new(config).unwrap();
    let app = shield.apply(Router::new().route("/ping", get(|| async { "pong" })));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    (addr, handle)
}

#[tokio::test]
async fn rate_limit_disabled_lets_all_requests_through() {
    let rl = RateLimitConfig::default();
    let (addr, handle) = start_server(rl).await;
    let client = reqwest::Client::new();
    for _ in 0..10 {
        let resp = client
            .get(format!("http://{addr}/ping"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), reqwest::StatusCode::OK);
    }
    handle.abort();
}

#[tokio::test]
async fn rate_limit_enabled_blocks_after_burst() {
    let rl = RateLimitConfig {
        enabled: true,
        per_ip_rps: 1,
        burst: 2,
    };
    let (addr, handle) = start_server(rl).await;
    let client = reqwest::Client::builder().build().unwrap();

    // Burst of 2: the first two pass immediately.
    for _ in 0..2 {
        let resp = client
            .get(format!("http://{addr}/ping"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), reqwest::StatusCode::OK);
    }

    // The next one, without waiting for the bucket to refill, must be rejected.
    let resp = client
        .get(format!("http://{addr}/ping"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::TOO_MANY_REQUESTS);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "rate_limit_exceeded");

    handle.abort();
}

#[tokio::test]
async fn invalid_rate_limit_config_fails_construction() {
    let config = ShieldConfig {
        rate_limit: RateLimitConfig {
            enabled: true,
            per_ip_rps: 0,
            burst: 10,
        },
        ..ShieldConfig::default()
    };
    let err = Shield::try_new(config).unwrap_err();
    assert_eq!(err.code(), "config_error");
}
