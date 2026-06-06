//! End-to-end HTTPS test: SNI certificate selection + routing over real TLS.
#![cfg(feature = "tls")]

use std::sync::Arc;

use ag_edge::cert::{server_config, CertStore};
use ag_edge::challenge::Http01ChallengeStore;
use ag_edge::router::{BindingCache, RouteBinding};
use ag_edge::server::{serve_https, EdgeState};

#[tokio::test]
async fn serves_https_with_sni_selected_cert() {
    // Self-signed cert for the test hostname.
    let ck = rcgen::generate_simple_self_signed(vec!["test.local".to_string()]).unwrap();
    let cert_pem = ck.cert.pem();
    let key_pem = ck.key_pair.serialize_pem();

    let mut store = CertStore::new();
    store.insert_pem("test.local", &cert_pem, &key_pem).unwrap();
    let tls = server_config(Arc::new(store)).unwrap();

    // Bind the route.
    let mut cache = BindingCache::new();
    cache.insert(RouteBinding {
        hostname_ascii: "test.local".to_owned(),
        project: "site".to_owned(),
        environment: "production".to_owned(),
        target_ref: "svc_tls".to_owned(),
    });
    let state = EdgeState::new(cache, Http01ChallengeStore::new());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = serve_https(listener, state, tls).await;
    });

    // Client trusts any cert (self-signed) and resolves test.local -> addr so
    // the TLS SNI is "test.local", exercising the SNI cert resolver.
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .resolve("test.local", addr)
        .build()
        .unwrap();

    let resp = client.get("https://test.local/").send().await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers().get("x-ag-target").unwrap().to_str().unwrap(),
        "svc_tls"
    );
}
