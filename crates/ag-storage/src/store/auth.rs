//! Bearer token authentication middleware for the HTTP server.
//!
//! Without feature `auth`: compares the token against `STORE_TOKEN` (static string).
//! With feature `auth`: validates the Bearer as JWT Ed25519 via `ag-auth` (pending, issue #148).
//!
//! If the configured token is empty, the server accepts all requests
//! without authentication (development mode).

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Middleware state: the configured static Bearer token.
///
/// If the string is empty, the server operates without authentication (dev mode).
pub type AuthToken = Arc<String>;

/// Axum middleware that requires `Authorization: Bearer <token>`.
///
/// Applied at `Router` level, not per individual route, so that any
/// new route inherits the protection automatically.
pub async fn bearer_auth_middleware(
    State(token): State<AuthToken>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if token.is_empty() {
        // Dev mode: empty token = no authentication required.
        return next.run(request).await;
    }

    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(h) if h.starts_with("Bearer ") && h[7..] == **token => next.run(request).await,
        _ => StatusCode::UNAUTHORIZED.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn ok_handler() -> StatusCode {
        StatusCode::OK
    }

    fn app_with_token(token: &str) -> Router {
        let auth_token: AuthToken = Arc::new(token.to_string());
        Router::new()
            .route("/test", get(ok_handler))
            .layer(middleware::from_fn_with_state(
                auth_token,
                bearer_auth_middleware,
            ))
    }

    #[tokio::test]
    async fn empty_token_allows_all_requests() {
        let app = app_with_token("");
        let req = Request::builder().uri("/test").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn valid_token_allows_request() {
        let app = app_with_token("secret123");
        let req = Request::builder()
            .uri("/test")
            .header("Authorization", "Bearer secret123")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_token_returns_401() {
        let app = app_with_token("secret123");
        let req = Request::builder()
            .uri("/test")
            .header("Authorization", "Bearer wrongtoken")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn missing_auth_header_returns_401() {
        let app = app_with_token("secret123");
        let req = Request::builder().uri("/test").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn malformed_bearer_returns_401() {
        let app = app_with_token("secret123");
        let req = Request::builder()
            .uri("/test")
            .header("Authorization", "Basic secret123")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
