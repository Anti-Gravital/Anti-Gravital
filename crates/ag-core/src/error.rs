//! Core error system.
//!
//! `AgError` enumerates the error classes the Shield pipeline can
//! produce. Each variant maps to a stable HTTP response with a
//! `snake_case` code in the JSON body. The conversion to
//! `axum::response::Response` is automatic via `IntoResponse`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

/// Standard core result.
pub type AgResult<T> = Result<T, AgError>;

/// Errors produced by the core and by the Shield pipeline.
#[derive(Debug, Error)]
pub enum AgError {
    /// Configuration load or validation error.
    #[error("config error: {0}")]
    Config(String),

    /// TLS layer error.
    #[error("tls error: {0}")]
    Tls(String),

    /// Authentication error.
    #[error("auth error: {0}")]
    Auth(String),

    /// Rate limit exceeded.
    #[error("rate limit exceeded")]
    RateLimit,

    /// Payload validation error.
    #[error("validation error: {0}")]
    Validation(String),

    /// CORS error.
    #[error("cors error: {0}")]
    Cors(String),

    /// CSRF error.
    #[error("csrf error: {0}")]
    Csrf(String),

    /// Resource not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Bad request (invalid parameters, incorrect semantics).
    #[error("bad request: {0}")]
    BadRequest(String),

    /// State conflict (duplicate, uniqueness constraint, etc.).
    #[error("conflict: {0}")]
    Conflict(String),

    /// Data layer error (database, migrations).
    #[error("database error: {0}")]
    Database(String),

    /// I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Any other unclassified error.
    #[error("internal error: {0}")]
    Other(String),
}

impl AgError {
    /// Stable `snake_case` code for public serialization.
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
            Self::NotFound(_) => "not_found",
            Self::BadRequest(_) => "bad_request",
            Self::Conflict(_) => "conflict",
            Self::Database(_) => "database_error",
            Self::Io(_) => "io_error",
            Self::Other(_) => "internal_error",
        }
    }

    /// HTTP code corresponding to the variant.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        match self {
            Self::Config(_) | Self::Tls(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Auth(_) => StatusCode::UNAUTHORIZED,
            Self::RateLimit => StatusCode::TOO_MANY_REQUESTS,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Cors(_) | Self::Csrf(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Database(_) | Self::Io(_) | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

    #[test]
    fn not_found_maps_to_404() {
        let err = AgError::NotFound("todo 42".into());
        assert_eq!(err.status(), StatusCode::NOT_FOUND);
        assert_eq!(err.code(), "not_found");
    }

    #[test]
    fn bad_request_maps_to_400() {
        let err = AgError::BadRequest("invalid id".into());
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
        assert_eq!(err.code(), "bad_request");
    }

    #[test]
    fn conflict_maps_to_409() {
        let err = AgError::Conflict("email already exists".into());
        assert_eq!(err.status(), StatusCode::CONFLICT);
        assert_eq!(err.code(), "conflict");
    }

    #[test]
    fn database_maps_to_500() {
        let err = AgError::Database("connection refused".into());
        assert_eq!(err.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code(), "database_error");
    }
}
