//! Middleware de autenticacion Bearer token para el servidor HTTP.
//!
//! Sin feature `auth`: compara el token contra `STORE_TOKEN` (string estatico).
//! Con feature `auth`: valida el Bearer como JWT Ed25519 via `ag-auth` (TECH-DEBT).
//!
//! Si el token configurado esta vacio, el servidor acepta todas las peticiones
//! sin autenticacion (modo desarrollo).

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Estado del middleware: el token Bearer estatico configurado.
///
/// Si el string es vacio, el servidor funciona sin autenticacion (modo dev).
pub type AuthToken = Arc<String>;

/// Middleware Axum que exige `Authorization: Bearer <token>`.
///
/// Se aplica a nivel de `Router`, no por ruta individual, para que cualquier
/// ruta nueva herede la proteccion automaticamente.
pub async fn bearer_auth_middleware(
    State(token): State<AuthToken>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if token.is_empty() {
        // Modo desarrollo: token vacio = sin autenticacion requerida.
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
