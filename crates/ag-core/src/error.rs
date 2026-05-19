//! Sistema de errores del nucleo.
//!
//! `AgError` enumera las clases de error que la pipeline Shield puede
//! producir. Cada variante mapea a una respuesta HTTP estable con un
//! codigo `snake_case` en el cuerpo JSON. La conversion a
//! `axum::response::Response` es automatica via `IntoResponse`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

/// Resultado estandar del nucleo.
pub type AgResult<T> = Result<T, AgError>;

/// Errores producidos por el nucleo y por la pipeline Shield.
#[derive(Debug, Error)]
pub enum AgError {
    /// Error de carga o validacion de la configuracion.
    #[error("config error: {0}")]
    Config(String),

    /// Error de capa TLS.
    #[error("tls error: {0}")]
    Tls(String),

    /// Error de autenticacion.
    #[error("auth error: {0}")]
    Auth(String),

    /// Limite de tasa excedido.
    #[error("rate limit exceeded")]
    RateLimit,

    /// Error de validacion de payload.
    #[error("validation error: {0}")]
    Validation(String),

    /// Error de CORS.
    #[error("cors error: {0}")]
    Cors(String),

    /// Error de CSRF.
    #[error("csrf error: {0}")]
    Csrf(String),

    /// Error de I/O.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Cualquier otro error no clasificado.
    #[error("internal error: {0}")]
    Other(String),
}

impl AgError {
    /// Codigo estable `snake_case` para serializacion publica.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Config(_) => "config_error",
            Self::Tls(_) => "tls_error",
            Self::Auth(_) => "auth_error",
            Self::RateLimit => "rate_limit_exceeded",
            Self::Validation(_) => "validation_error",
            Self::Cors(_) => "cors_error",
            Self::Csrf(_) => "csrf_error",
            Self::Io(_) => "io_error",
            Self::Other(_) => "internal_error",
        }
    }

    /// Codigo HTTP correspondiente a la variante.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        match self {
            Self::Config(_) | Self::Tls(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Auth(_) => StatusCode::UNAUTHORIZED,
            Self::RateLimit => StatusCode::TOO_MANY_REQUESTS,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Cors(_) | Self::Csrf(_) => StatusCode::FORBIDDEN,
            Self::Io(_) | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    code: &'a str,
    message: String,
}

impl IntoResponse for AgError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            code: self.code(),
            message: self.to_string(),
        };
        (self.status(), Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limit_maps_to_429() {
        let err = AgError::RateLimit;
        assert_eq!(err.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(err.code(), "rate_limit_exceeded");
    }

    #[test]
    fn validation_maps_to_422() {
        let err = AgError::Validation("bad field".into());
        assert_eq!(err.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(err.code(), "validation_error");
    }

    #[test]
    fn auth_maps_to_401() {
        let err = AgError::Auth("missing token".into());
        assert_eq!(err.status(), StatusCode::UNAUTHORIZED);
    }
}
