//! E2E tests for the payload validation layer.

#![cfg(feature = "validation")]

use ag_core::shield::{Validate, ValidatedJson, ValidationErrors};
use ag_core::{Shield, ShieldConfig};
use axum::routing::post;
use axum::Router;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
struct NewProject {
    name: String,
    slug: String,
}

impl Validate for NewProject {
    fn validate(&self, errors: &mut ValidationErrors) {
        if self.name.is_empty() {
            errors.add("name", "must not be empty");
        }
        if self.slug.is_empty() {
            errors.add("slug", "must not be empty");
        }
        if !self
            .slug
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            errors.add("slug", "only ascii alphanumeric and dashes allowed");
        }
    }
}

async fn handler(ValidatedJson(project): ValidatedJson<NewProject>) -> String {
    format!("created {} ({})", project.name, project.slug)
}

async fn start_server() -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let shield = Shield::try_new(ShieldConfig::default()).unwrap();
    let app = shield.apply(Router::new().route("/projects", post(handler)));

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
async fn valid_payload_returns_200() {
    let (addr, handle) = start_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/projects"))
        .json(&serde_json::json!({ "name": "anti", "slug": "anti-gravital" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body = resp.text().await.unwrap();
    assert_eq!(body, "created anti (anti-gravital)");
    handle.abort();
}

#[tokio::test]
async fn invalid_payload_returns_422_with_field_details() {
    let (addr, handle) = start_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/projects"))
        .json(&serde_json::json!({ "name": "", "slug": "bad slug!" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "validation_error");
    let message = body["message"].as_str().unwrap();
    assert!(message.contains("name"));
    assert!(message.contains("slug"));
    handle.abort();
}

#[tokio::test]
async fn malformed_json_returns_422() {
    let (addr, handle) = start_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/projects"))
        .header("content-type", "application/json")
        .body("{ not json")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "validation_error");
    handle.abort();
}
