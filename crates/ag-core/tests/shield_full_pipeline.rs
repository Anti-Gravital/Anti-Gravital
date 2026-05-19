//! Test E2E del pipeline Shield completo.
//!
//! Arranca un servidor con TODAS las capas activas (TLS, auth-jwt,
//! csrf, cors, rate-limit, validation, logging) y verifica que una
//! peticion legitima atraviesa el pipeline mientras que peticiones
//! invalidas se rechazan en la capa correcta.

#![cfg(all(
    feature = "tls",
    feature = "auth-jwt",
    feature = "csrf",
    feature = "cors",
    feature = "rate-limit",
    feature = "validation"
))]

use ag_core::config::{AuthConfig, CorsConfig, CsrfConfig, RateLimitConfig, TlsConfig};
use ag_core::shield::{Claims, Validate, ValidatedJson, ValidationErrors};
use ag_core::{Shield, ShieldConfig};
use axum::routing::{get, post};
use axum::Router;
use ed25519_dalek::pkcs8::spki::der::pem::LineEnding;
use ed25519_dalek::pkcs8::{EncodePrivateKey, EncodePublicKey};
use ed25519_dalek::SigningKey;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const ALLOWED_ORIGIN: &str = "https://app.example.com";
const CSRF_TOKEN: &str = "csrf-token-abc";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppClaims {
    sub: String,
    exp: u64,
}

#[derive(Debug, Deserialize)]
struct CreatePost {
    title: String,
}

impl Validate for CreatePost {
    fn validate(&self, errors: &mut ValidationErrors) {
        if self.title.is_empty() {
            errors.add("title", "must not be empty");
        }
    }
}

async fn me_handler(Claims(c): Claims<AppClaims>) -> String {
    format!("hello {}", c.sub)
}

async fn create_handler(
    Claims(c): Claims<AppClaims>,
    ValidatedJson(post): ValidatedJson<CreatePost>,
) -> String {
    format!("created \"{}\" by {}", post.title, c.sub)
}

struct TestFixture {
    addr: SocketAddr,
    signing_pem: String,
    handle: tokio::task::JoinHandle<()>,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn generate_ed25519_keypair() -> (String, String) {
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
    (public_pem, signing_pem)
}

fn generate_tls_cert() -> (PathBuf, PathBuf) {
    let cert =
        rcgen::generate_simple_self_signed(vec!["localhost".to_owned(), "127.0.0.1".to_owned()])
            .unwrap();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir();
    let cert_path = dir.join(format!("ag-fp-cert-{nanos}.pem"));
    let key_path = dir.join(format!("ag-fp-key-{nanos}.pem"));
    std::fs::File::create(&cert_path)
        .unwrap()
        .write_all(cert.cert.pem().as_bytes())
        .unwrap();
    std::fs::File::create(&key_path)
        .unwrap()
        .write_all(cert.key_pair.serialize_pem().as_bytes())
        .unwrap();
    (cert_path, key_path)
}

fn sign_jwt(claims: &AppClaims, signing_pem: &str) -> String {
    let header = Header::new(Algorithm::EdDSA);
    let encoding_key = EncodingKey::from_ed_pem(signing_pem.as_bytes()).unwrap();
    jsonwebtoken::encode(&header, claims, &encoding_key).unwrap()
}

async fn start_full_pipeline() -> TestFixture {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let (public_pem, signing_pem) = generate_ed25519_keypair();
    let (cert_path, key_path) = generate_tls_cert();

    let config = ShieldConfig {
        tls: TlsConfig {
            enabled: true,
            cert_path: Some(cert_path),
            key_path: Some(key_path),
        },
        auth: AuthConfig {
            enabled: true,
            public_key_pem: Some(public_pem),
            ..AuthConfig::default()
        },
        csrf: CsrfConfig {
            enabled: true,
            ..CsrfConfig::default()
        },
        cors: CorsConfig {
            enabled: true,
            allow_origins: vec![ALLOWED_ORIGIN.to_owned()],
            allow_methods: vec!["GET".into(), "POST".into()],
            allow_headers: vec!["content-type".into(), "authorization".into()],
            allow_credentials: false,
        },
        rate_limit: RateLimitConfig {
            enabled: true,
            per_ip_rps: 1_000,
            burst: 2_000,
        },
        ..ShieldConfig::default()
    };

    let shield = Shield::try_new(config).unwrap();
    let app = shield.apply(
        Router::new()
            .route("/me", get(me_handler))
            .route("/posts", post(create_handler)),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let shield_clone = shield.clone();
    let handle = tokio::spawn(async move {
        let _ = shield_clone.serve(listener, app).await;
    });

    TestFixture {
        addr,
        signing_pem,
        handle,
    }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
}

#[tokio::test]
async fn valid_get_with_jwt_and_origin_passes_through_all_layers() {
    let fixture = start_full_pipeline().await;
    let token = sign_jwt(
        &AppClaims {
            sub: "angel".into(),
            exp: now() + 60,
        },
        &fixture.signing_pem,
    );
    let resp = client()
        .get(format!("https://{}/me", fixture.addr))
        .header("origin", ALLOWED_ORIGIN)
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body = resp.text().await.unwrap();
    assert_eq!(body, "hello angel");
    fixture.handle.abort();
}

#[tokio::test]
async fn valid_post_with_csrf_jwt_origin_validation_passes() {
    let fixture = start_full_pipeline().await;
    let token = sign_jwt(
        &AppClaims {
            sub: "angel".into(),
            exp: now() + 60,
        },
        &fixture.signing_pem,
    );
    let resp = client()
        .post(format!("https://{}/posts", fixture.addr))
        .header("origin", ALLOWED_ORIGIN)
        .header("x-csrf-token", CSRF_TOKEN)
        .header("cookie", format!("ag_csrf={CSRF_TOKEN}"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "title": "RFC-0002 PR 10" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body = resp.text().await.unwrap();
    assert_eq!(body, "created \"RFC-0002 PR 10\" by angel");
    fixture.handle.abort();
}

#[tokio::test]
async fn invalid_jwt_is_blocked_at_auth_layer_with_401() {
    let fixture = start_full_pipeline().await;
    let resp = client()
        .get(format!("https://{}/me", fixture.addr))
        .header("origin", ALLOWED_ORIGIN)
        .header("authorization", "Bearer not-a-jwt")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "auth_error");
    fixture.handle.abort();
}

#[tokio::test]
async fn missing_csrf_on_post_is_blocked_with_403() {
    let fixture = start_full_pipeline().await;
    let token = sign_jwt(
        &AppClaims {
            sub: "angel".into(),
            exp: now() + 60,
        },
        &fixture.signing_pem,
    );
    let resp = client()
        .post(format!("https://{}/posts", fixture.addr))
        .header("origin", ALLOWED_ORIGIN)
        .bearer_auth(&token)
        .json(&serde_json::json!({ "title": "ok" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::FORBIDDEN);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "csrf_error");
    fixture.handle.abort();
}

#[tokio::test]
async fn invalid_payload_is_blocked_with_422() {
    let fixture = start_full_pipeline().await;
    let token = sign_jwt(
        &AppClaims {
            sub: "angel".into(),
            exp: now() + 60,
        },
        &fixture.signing_pem,
    );
    let resp = client()
        .post(format!("https://{}/posts", fixture.addr))
        .header("origin", ALLOWED_ORIGIN)
        .header("x-csrf-token", CSRF_TOKEN)
        .header("cookie", format!("ag_csrf={CSRF_TOKEN}"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "title": "" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "validation_error");
    fixture.handle.abort();
}

#[tokio::test]
async fn unlisted_origin_does_not_receive_allow_origin_header() {
    let fixture = start_full_pipeline().await;
    let token = sign_jwt(
        &AppClaims {
            sub: "angel".into(),
            exp: now() + 60,
        },
        &fixture.signing_pem,
    );
    let resp = client()
        .get(format!("https://{}/me", fixture.addr))
        .header("origin", "https://attacker.example")
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    // El request pasa la capa CORS (no es preflight), pero el navegador
    // bloqueara la respuesta porque no hay header allow-origin para
    // este origen.
    assert!(resp.headers().get("access-control-allow-origin").is_none());
    fixture.handle.abort();
}
