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
        Some(h) if h.starts_with("Bearer ") && h[7..] == **token => {
            next.run(request).await
        }
        _ => StatusCode::UNAUTHORIZED.into_response(),
    }
}
