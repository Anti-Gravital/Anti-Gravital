//! Runnable edge HTTP(S) listener.
//!
//! Wires the pure edge logic into a real server: it serves ACME HTTP-01
//! challenges, applies canonical/redirect policy, and resolves the request
//! `Host` to a route. Unknown custom hostnames fail closed with 404.
//!
//! Upstream proxying to a service is the deployment's responsibility; on a
//! successful custom-domain match the edge returns the resolved target in
//! `x-ag-*` headers so the surrounding system (or a test) can act on it. This
//! keeps the edge crate free of an opinion about transport to upstreams while
//! remaining fully exercisable.

use std::sync::{Arc, RwLock};

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderValue, Request, StatusCode};
use axum::response::Response;
use axum::Router;

use crate::challenge::{acme_challenge_token, Http01ChallengeStore};
use crate::redirect::{CanonicalPolicy, RedirectDecision};
use crate::router::{resolve_hostname, BindingCache, RouteResolution};

type LegacyResolver = Arc<dyn Fn(&str) -> Option<String> + Send + Sync>;

/// Shared state for the edge server.
#[derive(Clone)]
pub struct EdgeState {
    bindings: Arc<RwLock<BindingCache>>,
    challenges: Http01ChallengeStore,
    redirects: Arc<RwLock<Vec<CanonicalPolicy>>>,
    legacy: LegacyResolver,
}

impl EdgeState {
    /// Creates edge state from a binding cache and a challenge store.
    pub fn new(bindings: BindingCache, challenges: Http01ChallengeStore) -> Self {
        Self {
            bindings: Arc::new(RwLock::new(bindings)),
            challenges,
            redirects: Arc::new(RwLock::new(Vec::new())),
            legacy: Arc::new(|_| None),
        }
    }

    /// Sets the canonical/redirect policies.
    pub fn with_redirects(self, policies: Vec<CanonicalPolicy>) -> Self {
        *self.redirects.write().expect("redirects lock poisoned") = policies;
        self
    }

    /// Sets a legacy default-route resolver (backward compatibility).
    pub fn with_legacy_resolver(
        mut self,
        f: impl Fn(&str) -> Option<String> + Send + Sync + 'static,
    ) -> Self {
        self.legacy = Arc::new(f);
        self
    }

    /// Shared handle to the challenge store.
    pub fn challenges(&self) -> &Http01ChallengeStore {
        &self.challenges
    }

    /// Replaces the binding cache contents.
    pub fn set_bindings(&self, bindings: BindingCache) {
        *self.bindings.write().expect("bindings lock poisoned") = bindings;
    }
}

/// Builds the edge `axum` router.
pub fn build_router(state: EdgeState) -> Router {
    Router::new().fallback(edge_handler).with_state(state)
}

fn host_of(req: &Request<Body>) -> Option<String> {
    // Prefer the HTTP/2 :authority (uri authority), fall back to Host header.
    if let Some(auth) = req.uri().authority() {
        return Some(auth.host().to_ascii_lowercase());
    }
    req.headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(|h| h.split(':').next().unwrap_or(h).to_ascii_lowercase())
}

async fn edge_handler(State(state): State<EdgeState>, req: Request<Body>) -> Response {
    let path = req.uri().path().to_owned();
    let query = req.uri().query().map(|q| q.to_owned());

    // 1. ACME HTTP-01 challenge.
    if let Some(token) = acme_challenge_token(&path) {
        if let Some(key_auth) = state.challenges.get(token) {
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from(key_auth))
                .expect("valid response");
        }
        return not_found("unknown acme challenge token");
    }

    let host = match host_of(&req) {
        Some(h) => h,
        None => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("missing host"))
                .expect("valid response");
        }
    };

    // 2. Canonical/redirect policy.
    {
        let redirects = state.redirects.read().expect("redirects lock poisoned");
        for policy in redirects.iter() {
            if let RedirectDecision::Redirect { location, status } =
                policy.evaluate(&host, &path, query.as_deref())
            {
                let mut resp = Response::builder()
                    .status(StatusCode::from_u16(status).unwrap_or(StatusCode::PERMANENT_REDIRECT));
                if let Ok(value) = HeaderValue::from_str(&location) {
                    resp = resp.header(header::LOCATION, value);
                }
                return resp.body(Body::empty()).expect("valid response");
            }
        }
    }

    // 3. Hostname resolution.
    let resolution = {
        let bindings = state.bindings.read().expect("bindings lock poisoned");
        let legacy = state.legacy.clone();
        resolve_hostname(&bindings, &host, move |h| legacy(h))
    };

    match resolution {
        RouteResolution::Custom(binding) => Response::builder()
            .status(StatusCode::OK)
            .header("x-ag-project", binding.project)
            .header("x-ag-environment", binding.environment)
            .header("x-ag-target", binding.target_ref)
            .body(Body::from("routed"))
            .expect("valid response"),
        RouteResolution::Existing(handle) => Response::builder()
            .status(StatusCode::OK)
            .header("x-ag-legacy", handle)
            .body(Body::from("legacy"))
            .expect("valid response"),
        RouteResolution::ExistingFallback | RouteResolution::Unknown => {
            not_found("no route for host")
        }
    }
}

fn not_found(msg: &'static str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from(msg))
        .expect("valid response")
}

/// Serves the edge over plain HTTP on the given listener until it errors.
pub async fn serve_http(
    listener: tokio::net::TcpListener,
    state: EdgeState,
) -> std::io::Result<()> {
    let app = build_router(state);
    axum::serve(listener, app).await
}

#[cfg(feature = "tls")]
mod tls_serve {
    use super::*;
    use std::sync::Arc;

    use hyper_util::rt::{TokioExecutor, TokioIo};
    use hyper_util::server::conn::auto::Builder as AutoBuilder;
    use hyper_util::service::TowerToHyperService;
    use rustls::ServerConfig;

    /// Serves the edge over HTTPS, selecting certificates by SNI from the
    /// `rustls` config produced by [`crate::cert::server_config`].
    pub async fn serve_https(
        listener: tokio::net::TcpListener,
        state: EdgeState,
        tls_config: Arc<ServerConfig>,
    ) -> std::io::Result<()> {
        let acceptor = tokio_rustls::TlsAcceptor::from(tls_config);
        let app = build_router(state);

        loop {
            let (stream, _peer) = listener.accept().await?;
            let acceptor = acceptor.clone();
            let app = app.clone();
            tokio::spawn(async move {
                let tls_stream = match acceptor.accept(stream).await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::debug!(error = %e, "TLS handshake failed");
                        return;
                    }
                };
                let io = TokioIo::new(tls_stream);
                let service = TowerToHyperService::new(app);
                if let Err(e) = AutoBuilder::new(TokioExecutor::new())
                    .serve_connection(io, service)
                    .await
                {
                    tracing::debug!(error = %e, "edge HTTPS connection error");
                }
            });
        }
    }
}

#[cfg(feature = "tls")]
pub use tls_serve::serve_https;
