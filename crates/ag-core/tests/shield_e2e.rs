//! End-to-end tests for the Shield bootstrap.
//!
//! They start a real server on `127.0.0.1:0` (ephemeral port) and verify
//! that the pipeline accepts HTTP/1.1 requests and produces the expected
//! responses.

use ag_core::{Shield, ShieldConfig};
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;

async fn start_server() -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let shield = Shield::try_new(ShieldConfig::default()).unwrap();
    let app = shield.apply(Router::new().route("/hello", get(|| async { "hello, shield" })));

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
async fn hello_endpoint_responds_200_with_body() {
    let (addr, handle) = start_server().await;
    let url = format!("http://{addr}/hello");

    let client = reqwest::Client::builder().build().unwrap();
    let resp = client.get(&url).send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body = resp.text().await.unwrap();
    assert_eq!(body, "hello, shield");

    handle.abort();
}

#[tokio::test]
async fn missing_route_responds_404() {
    let (addr, handle) = start_server().await;
    let url = format!("http://{addr}/no-such-route");

    let client = reqwest::Client::builder().build().unwrap();
    let resp = client.get(&url).send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);

    handle.abort();
}
