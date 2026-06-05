//! Integration tests for the control-plane REST API over real HTTP.
#![cfg(feature = "api")]

use std::sync::Arc;

use ag_domains::api::{build_router, ApiState};
use ag_domains::instructions::EdgeTargets;
use ag_domains::store::{AttachmentStore, InMemoryStore};
use serde_json::Value;

async fn start() -> String {
    let store: Arc<dyn AttachmentStore> = Arc::new(InMemoryStore::new());
    let edge = EdgeTargets::new("edge.example-cloud.net");
    let state = ApiState::new(store, edge);
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    format!("http://{addr}")
}

#[tokio::test]
async fn full_attachment_lifecycle_over_http() {
    let base = start().await;
    let client = reqwest::Client::new();

    // Create.
    let resp = client
        .post(format!("{base}/v1/domains/attachments"))
        .json(&serde_json::json!({
            "hostname": "api.example.com",
            "project_id": "site",
            "target_ref": "svc_api"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    let id = body["id"].as_str().unwrap().to_owned();
    assert_eq!(body["hostname_ascii"], "api.example.com");
    assert_eq!(body["lifecycle"], "pending_ownership");
    assert_eq!(body["ownership"]["name"], "_ag-domain.example.com");

    // Get by id.
    let resp = client
        .get(format!("{base}/v1/domains/attachments/{id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // List.
    let resp = client
        .get(format!("{base}/v1/domains/attachments"))
        .send()
        .await
        .unwrap();
    let list: Value = resp.json().await.unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Instructions.
    let resp = client
        .get(format!("{base}/v1/domains/attachments/{id}/instructions"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ins: Value = resp.json().await.unwrap();
    let records = ins["records"].as_array().unwrap();
    assert!(records.iter().any(|r| r["record_type"] == "CNAME"));
    assert!(records.iter().any(|r| r["is_ownership"] == true));

    // Detach.
    let resp = client
        .post(format!("{base}/v1/domains/attachments/{id}/detach"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
}

#[tokio::test]
async fn duplicate_hostname_conflicts() {
    let base = start().await;
    let client = reqwest::Client::new();
    let payload = serde_json::json!({"hostname": "dup.example.com", "project_id": "site"});

    let first = client
        .post(format!("{base}/v1/domains/attachments"))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(first.status(), 201);

    let second = client
        .post(format!("{base}/v1/domains/attachments"))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(second.status(), 409);
}

#[tokio::test]
async fn invalid_hostname_is_400() {
    let base = start().await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base}/v1/domains/attachments"))
        .json(&serde_json::json!({"hostname": "https://x/y", "project_id": "site"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn unknown_id_is_404() {
    let base = start().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base}/v1/domains/attachments/dom_missing"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}
