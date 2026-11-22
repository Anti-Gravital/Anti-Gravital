//! E2E tests for the TLS 1.3 layer.

#![cfg(feature = "tls")]

use ag_core::config::TlsConfig;
use ag_core::{Shield, ShieldConfig};
use axum::routing::get;
use axum::Router;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static TMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn unique_temp(prefix: &str) -> PathBuf {
    let n = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    std::env::temp_dir().join(format!("{prefix}-{pid}-{n}.pem"))
}

fn generate_cert_pair() -> (PathBuf, PathBuf) {
    let subject_alt_names = vec!["localhost".to_owned(), "127.0.0.1".to_owned()];
    let cert = rcgen::generate_simple_self_signed(subject_alt_names).unwrap();
    let cert_pem = cert.cert.pem();
    let key_pem = cert.signing_key.serialize_pem();

    let cert_path = unique_temp("ag-tls-cert");
    let key_path = unique_temp("ag-tls-key");

    let mut cf = std::fs::File::create(&cert_path).unwrap();
    cf.write_all(cert_pem.as_bytes()).unwrap();
    let mut kf = std::fs::File::create(&key_path).unwrap();
    kf.write_all(key_pem.as_bytes()).unwrap();

    (cert_path, key_path)
}

#[tokio::test]
async fn tls_disabled_serves_plain_http() {
    let _ =
        rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider());
    let shield = Shield::try_new(ShieldConfig::default()).unwrap();
    let app = shield.apply(Router::new().route("/ping", get(|| async { "pong" })));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let shield_clone = shield.clone();
    let handle = tokio::spawn(async move {
        let _ = shield_clone.serve(listener, app).await;
    });

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/ping"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    handle.abort();
}

#[tokio::test]
async fn tls_enabled_serves_https() {
    let _ =
        rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider());
    let (cert_path, key_path) = generate_cert_pair();
    let config = ShieldConfig {
        tls: TlsConfig {
            enabled: true,
            cert_path: Some(cert_path),
            key_path: Some(key_path),
        },
        ..ShieldConfig::default()
    };
    let shield = Shield::try_new(config).unwrap();
    let app = shield.apply(Router::new().route("/secure", get(|| async { "tls-ok" })));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let shield_clone = shield.clone();
    let handle = tokio::spawn(async move {
        let _ = shield_clone.serve(listener, app).await;
    });

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let resp = client
        .get(format!("https://{addr}/secure"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body = resp.text().await.unwrap();
    assert_eq!(body, "tls-ok");
    handle.abort();
}

/// A real, verified TLS handshake through `reqwest` against the local rustls
/// server. With `reqwest` built using `rustls-no-provider`, building a client
/// without an installed provider panics, so this test fails in CI on a
/// provider misconfiguration (including under `--all-features`, where other
/// crates may still link `aws-lc-rs`; see ADR-0018).
#[tokio::test]
async fn reqwest_tls_handshake_uses_installed_ring_provider() {
    let _ =
        rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider());
    assert!(
        rustls::crypto::CryptoProvider::get_default().is_some(),
        "ring must be installed before reqwest builds a client"
    );

    let (cert_path, key_path) = generate_cert_pair();
    let cert_pem = std::fs::read(&cert_path).unwrap();
    let config = ShieldConfig {
        tls: TlsConfig {
            enabled: true,
            cert_path: Some(cert_path),
            key_path: Some(key_path),
        },
        ..ShieldConfig::default()
    };
    let shield = Shield::try_new(config).unwrap();
    let app = shield.apply(Router::new().route("/secure", get(|| async { "tls-ok" })));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let shield_clone = shield.clone();
    let handle = tokio::spawn(async move {
        let _ = shield_clone.serve(listener, app).await;
    });

    let cert = reqwest::Certificate::from_pem(&cert_pem).unwrap();
    let client = reqwest::Client::builder()
        .add_root_certificate(cert)
        .build()
        .unwrap();
    let resp = client
        .get(format!("https://localhost:{}/secure", addr.port()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert_eq!(resp.text().await.unwrap(), "tls-ok");
    handle.abort();
}

#[tokio::test]
async fn tls_enabled_without_cert_paths_fails_construction() {
    let _ =
        rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider());
    let config = ShieldConfig {
        tls: TlsConfig {
            enabled: true,
            cert_path: None,
            key_path: None,
        },
        ..ShieldConfig::default()
    };
    let err = Shield::try_new(config).unwrap_err();
    assert_eq!(err.code(), "tls_error");
}
