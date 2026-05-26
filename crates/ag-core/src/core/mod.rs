//! Core layer: Axum router, typed extractors and shared state.
//!
//! This layer receives requests that already passed the Shield pipeline and
//! dispatches them to business-logic handlers. It re-exports the Axum and
//! Shield extractors so handlers do not need to import directly from those
//! modules.
//!
//! # Available extractors
//!
//! | Extractor | Source | What it extracts |
//! | --- | --- | --- |
//! | [`State<T>`] | axum | Shared router state |
//! | [`Path<T>`] | axum | Typed route segments |
//! | [`Query<T>`] | axum | Query-string parameters |
//! | [`ValidatedBody<T>`] | shield | Deserialized and validated JSON body |
//! | [`Claims<T>`] | shield | Verified JWT claims |
//!
//! # Response system
//!
//! The [`response`] module re-exports the Axum response types and adds
//! [`response::PlainText`] for plain-text responses.
//!
//! # Application state
//!
//! [`AppState`] is the base shared-state type. In real projects the
//! developer defines their own state type with the database, cache and
//! external-service clients:
//!
//! ```rust
//! use ag_core::core::AppState;
//!
//! #[derive(Clone)]
//! struct MiEstado {
//!     nombre: String,
//! }
//! ```
//!
//! That type is passed to [`axum::Router::with_state`] and extracted with
//! [`State<T>`].

// Re-exports of Axum extractors for the Core handlers.
pub use axum::extract::{Path, Query, State};

// Re-exports of the validating body extractor from Shield.
#[cfg(feature = "validation")]
pub use crate::shield::validation::ValidatedJson as ValidatedBody;

// Re-export of the JWT claims extractor from Shield.
#[cfg(feature = "auth-jwt")]
pub use crate::shield::Claims;

/// Core response system.
///
/// Re-exports the Axum response types and adds convenient variants for
/// plain text. Handlers that return JSON use [`response::Json`] directly;
/// those that return plain text use [`response::PlainText`].
pub mod response {
    pub use axum::http::StatusCode;
    pub use axum::response::{IntoResponse, Response};
    pub use axum::Json;

    /// Plain-text response with `Content-Type: text/plain; charset=utf-8`.
    ///
    /// ```rust
    /// use ag_core::core::response::{IntoResponse, PlainText};
    ///
    /// async fn handler() -> impl IntoResponse {
    ///     PlainText("pong".into())
    /// }
    /// ```
    pub struct PlainText(pub String);

    impl IntoResponse for PlainText {
        fn into_response(self) -> Response {
            use axum::http::header;
            (
                [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                self.0,
            )
                .into_response()
        }
    }

    /// Open byte stream.
    ///
    /// Useful for chunked responses or basic SSE. For full SSE with
    /// reconnection and event IDs use `ag-realtime` (Phase 4).
    pub use axum::body::Body as BodyStream;
}

/// Default shared state.
///
/// Placeholder type for projects that do not need global state. Real
/// projects define their own state with the database, cache and
/// external-service clients, then register it with
/// [`axum::Router::with_state`].
#[derive(Debug, Default, Clone)]
pub struct AppState;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_state_default_is_constructible() {
        let _state = AppState;
    }

    #[test]
    fn path_query_state_re_exported() {
        // Verify that the types are available in the module.
        // Compilation already validates the correct re-export.
        type _P = Path<String>;
        type _Q = Query<std::collections::HashMap<String, String>>;
        type _S = State<()>;
    }

    #[test]
    fn plain_text_into_response() {
        use response::{IntoResponse, PlainText};
        let resp = PlainText("hola".into()).into_response();
        let ct = resp
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(ct.contains("text/plain"));
    }
}
