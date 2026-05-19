//! Benchmark Hello World del Shield.
//!
//! Mide la latencia por request a nivel Tower (sin pasar por la pila
//! de red TCP). Compara tres configuraciones:
//!
//! 1. `bare_axum`: router Axum minimo, sin Shield.
//! 2. `shield_default`: Shield con configuracion por defecto (solo
//!    capa de logging estructurado).
//! 3. `shield_full_default`: Shield con CORS, CSRF y rate-limit
//!    activas (TLS y auth requieren material criptografico y se
//!    cubren en tests E2E).
//!
//! Methodologia documentada en `crates/ag-core/benches/README.md`.

// El macro `criterion_group!` genera codigo que no documenta sus
// items. Suprimimos `missing_docs` solo en este archivo para no
// debilitar la regla global del workspace.
#![allow(missing_docs)]

use ag_core::config::{CorsConfig, CsrfConfig, RateLimitConfig};
use ag_core::{Shield, ShieldConfig};
use axum::body::Body;
use axum::http::{Method, Request};
use axum::routing::get;
use axum::Router;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tower::ServiceExt;

fn make_bare_router() -> Router {
    Router::new().route("/", get(|| async { "hello, world" }))
}

fn make_shield_default_router() -> Router {
    let shield = Shield::try_new(ShieldConfig::default()).unwrap();
    shield.apply(Router::new().route("/", get(|| async { "hello, world" })))
}

fn make_shield_full_router() -> Router {
    let config = ShieldConfig {
        cors: CorsConfig {
            enabled: true,
            allow_origins: vec!["https://app.example".into()],
            allow_methods: vec!["GET".into()],
            allow_headers: vec!["content-type".into()],
            allow_credentials: false,
        },
        csrf: CsrfConfig {
            enabled: true,
            ..CsrfConfig::default()
        },
        rate_limit: RateLimitConfig {
            enabled: true,
            per_ip_rps: 100_000,
            burst: 200_000,
        },
        ..ShieldConfig::default()
    };
    let shield = Shield::try_new(config).unwrap();
    shield.apply(Router::new().route("/", get(|| async { "hello, world" })))
}

fn build_get_request() -> Request<Body> {
    Request::builder()
        .method(Method::GET)
        .uri("/")
        .body(Body::empty())
        .unwrap()
}

fn run_through(router: &Router) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let resp = router
            .clone()
            .oneshot(build_get_request())
            .await
            .expect("router service must succeed");
        black_box(resp.status());
    });
}

fn bench_hello_world(c: &mut Criterion) {
    let bare = make_bare_router();
    c.bench_function("bare_axum_hello", |b| {
        b.iter(|| run_through(&bare));
    });

    let default_shield = make_shield_default_router();
    c.bench_function("shield_default_hello", |b| {
        b.iter(|| run_through(&default_shield));
    });

    let full_shield = make_shield_full_router();
    c.bench_function("shield_full_default_hello", |b| {
        b.iter(|| run_through(&full_shield));
    });
}

criterion_group!(benches, bench_hello_world);
criterion_main!(benches);
