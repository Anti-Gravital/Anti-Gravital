//! Integration tests for the edge HTTP listener over real TCP.
#![cfg(feature = "server")]

use ag_edge::challenge::Http01ChallengeStore;
use ag_edge::redirect::CanonicalPolicy;
use ag_edge::router::{BindingCache, RouteBinding};
use ag_edge::server::{serve_http, serve_http_redirect, EdgeState};

fn binding(host: &str, target: &str) -> RouteBinding {
    RouteBinding {
        hostname_ascii: host.to_owned(),
        project: "site".to_owned(),
        environment: "production".to_owned(),
        target_ref: target.to_owned(),
    }
}

async fn start() -> (String, Http01ChallengeStore) {
    let mut cache = BindingCache::new();
    cache.insert(binding("api.example.com", "svc_api"));

    let challenges = Http01ChallengeStore::new();
    challenges.set("tok-123", "tok-123.keyauthz");

    let redirects = vec![CanonicalPolicy {
        canonical_hostname: "www.example.com".to_owned(),
        redirect_from: vec!["example.com".to_owned()],
        status_code: 308,
        preserve_path: true,
        preserve_query: true,
    }];

    let state = EdgeState::new(cache, challenges.clone()).with_redirects(redirects);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = serve_http(listener, state).await;
    });
    (format!("http://{addr}"), challenges)
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

#[tokio::test]
async fn serves_acme_http01_challenge() {
    let (base, _ch) = start().await;
    let resp = client()
        .get(format!("{base}/.well-known/acme-challenge/tok-123"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap(), "tok-123.keyauthz");
}

#[tokio::test]
async fn unknown_challenge_token_is_404() {
    let (base, _ch) = start().await;
    let resp = client()
        .get(format!("{base}/.well-known/acme-challenge/nope"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn routes_known_custom_host() {
    let (base, _ch) = start().await;
    let resp = client()
        .get(&base)
        .header("host", "api.example.com")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers().get("x-ag-target").unwrap().to_str().unwrap(),
        "svc_api"
    );
}

#[tokio::test]
async fn unknown_host_fails_closed_404() {
    let (base, _ch) = start().await;
    let resp = client()
        .get(&base)
        .header("host", "evil.example.com")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn apex_redirects_to_canonical() {
    let (base, _ch) = start().await;
    let resp = client()
        .get(format!("{base}/products"))
        .header("host", "example.com")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 308);
    assert_eq!(
        resp.headers().get("location").unwrap().to_str().unwrap(),
        "https://www.example.com/products"
    );
}

async fn start_redirect() -> (String, Http01ChallengeStore) {
    let challenges = Http01ChallengeStore::new();
    challenges.set("rtok", "rtok.keyauth");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let ch = challenges.clone();
    tokio::spawn(async move {
        let _ = serve_http_redirect(listener, ch, 308).await;
    });
    (format!("http://{addr}"), challenges)
}

#[tokio::test]
async fn redirect_upgrades_http_to_https_preserving_path_and_query() {
    let (base, _ch) = start_redirect().await;
    let resp = client()
        .get(format!("{base}/products?page=2"))
        .header("host", "example.com")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 308);
    assert_eq!(
        resp.headers().get("location").unwrap().to_str().unwrap(),
        "https://example.com/products?page=2"
    );
}

#[tokio::test]
async fn redirect_serves_acme_challenge_over_http() {
    // ACME HTTP-01 must NOT be redirected, or issuance breaks.
    let (base, _ch) = start_redirect().await;
    let resp = client()
        .get(format!("{base}/.well-known/acme-challenge/rtok"))
        .header("host", "example.com")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap(), "rtok.keyauth");
}
