//! Tests E2E de la capa de autenticacion JWT Ed25519.

#![cfg(feature = "auth-jwt")]

use ag_core::config::AuthConfig;
use ag_core::shield::Claims;
use ag_core::{Shield, ShieldConfig};
use axum::routing::get;
use axum::Router;
use ed25519_dalek::pkcs8::spki::der::pem::LineEnding;
use ed25519_dalek::pkcs8::EncodePrivateKey;
use ed25519_dalek::pkcs8::EncodePublicKey;
use ed25519_dalek::SigningKey;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppClaims {
    sub: String,
    exp: u64,
    iss: Option<String>,
}

struct TestKeys {
    public_pem: String,
    signing_pem: String,
}

fn generate_keys() -> TestKeys {
    use rand_core::{OsRng, RngCore};
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);
    let signing_key = SigningKey::from_bytes(&secret);
    let verifying_key = signing_key.verifying_key();
    let public_pem = verifying_key.to_public_key_pem(LineEnding::LF).unwrap();
    let signing_pem = signing_key
        .to_pkcs8_pem(LineEnding::LF)
        .unwrap()
        .to_string();
    TestKeys {
        public_pem,
        signing_pem,
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn sign(claims: &AppClaims, signing_pem: &str) -> String {
    let header = Header::new(Algorithm::EdDSA);
    let encoding_key = EncodingKey::from_ed_pem(signing_pem.as_bytes()).unwrap();
    jsonwebtoken::encode(&header, claims, &encoding_key).unwrap()
}

async fn protected_handler(Claims(c): Claims<AppClaims>) -> String {
    format!("hello {}", c.sub)
}

async fn start_server(auth: AuthConfig) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let config = ShieldConfig {
        auth,
        ..ShieldConfig::default()
    };
    let shield = Shield::try_new(config).unwrap();
    let app = shield.apply(Router::new().route("/me", get(protected_handler)));

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
async fn missing_authorization_returns_401() {
    let keys = generate_keys();
    let cfg = AuthConfig {
        enabled: true,
        public_key_pem: Some(keys.public_pem.clone()),
        ..AuthConfig::default()
    };
    let (addr, handle) = start_server(cfg).await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/me"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "auth_error");
    handle.abort();
}

#[tokio::test]
async fn valid_token_passes_and_handler_sees_claims() {
    let keys = generate_keys();
    let token = sign(
        &AppClaims {
            sub: "angel".into(),
            exp: now() + 60,
            iss: None,
        },
        &keys.signing_pem,
    );

    let cfg = AuthConfig {
        enabled: true,
        public_key_pem: Some(keys.public_pem.clone()),
        ..AuthConfig::default()
    };
    let (addr, handle) = start_server(cfg).await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/me"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert_eq!(resp.text().await.unwrap(), "hello angel");
    handle.abort();
}

#[tokio::test]
async fn expired_token_is_rejected() {
    let keys = generate_keys();
    let token = sign(
        &AppClaims {
            sub: "angel".into(),
            exp: now() - 120,
            iss: None,
        },
        &keys.signing_pem,
    );

    let cfg = AuthConfig {
        enabled: true,
        public_key_pem: Some(keys.public_pem.clone()),
        ..AuthConfig::default()
    };
    let (addr, handle) = start_server(cfg).await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/me"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
    handle.abort();
}

#[tokio::test]
async fn token_signed_with_other_key_is_rejected() {
    let keys_a = generate_keys();
    let keys_b = generate_keys();
    let token = sign(
        &AppClaims {
            sub: "intruder".into(),
            exp: now() + 60,
            iss: None,
        },
        &keys_b.signing_pem,
    );

    let cfg = AuthConfig {
        enabled: true,
        public_key_pem: Some(keys_a.public_pem.clone()),
        ..AuthConfig::default()
    };
    let (addr, handle) = start_server(cfg).await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/me"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
    handle.abort();
}

#[tokio::test]
async fn wrong_issuer_is_rejected() {
    let keys = generate_keys();
    let token = sign(
        &AppClaims {
            sub: "angel".into(),
            exp: now() + 60,
            iss: Some("https://other.example/".into()),
        },
        &keys.signing_pem,
    );

    let cfg = AuthConfig {
        enabled: true,
        public_key_pem: Some(keys.public_pem.clone()),
        expected_issuer: Some("https://expected.example/".into()),
        ..AuthConfig::default()
    };
    let (addr, handle) = start_server(cfg).await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/me"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
    handle.abort();
}

#[tokio::test]
async fn auth_disabled_lets_request_through_but_extractor_fails() {
    let cfg = AuthConfig::default();
    let (addr, handle) = start_server(cfg).await;
    let client = reqwest::Client::new();
    // El handler usa Claims<T> que falla porque no hay AuthContext.
    let resp = client
        .get(format!("http://{addr}/me"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
    handle.abort();
}
