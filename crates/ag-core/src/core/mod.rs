//! Capa Core: router Axum, extractores tipados y estado compartido.
//!
//! Esta capa recibe los requests que ya pasaron la pipeline Shield y los
//! despacha a los handlers de logica de negocio. Reexporta los extractores
//! de Axum y de la Shield para que los handlers no necesiten importar
//! directamente desde esos modulos.
//!
//! # Extractores disponibles
//!
//! | Extractor | Origen | Que extrae |
//! | --- | --- | --- |
//! | [`State<T>`] | axum | Estado compartido del router |
//! | [`Path<T>`] | axum | Segmentos de ruta tipados |
//! | [`Query<T>`] | axum | Parametros de query string |
//! | [`ValidatedBody<T>`] | shield | Body JSON deserializado y validado |
//! | [`Claims<T>`] | shield | Claims JWT verificados |
//!
//! # Sistema de respuestas
//!
//! El modulo [`response`] reexporta los tipos de respuesta de Axum y
//! agrega [`response::PlainText`] para respuestas de texto plano.
//!
//! # Estado de la aplicacion
//!
//! [`AppState`] es el tipo base de estado compartido. En proyectos reales
//! el desarrollador define su propio tipo de estado con los clientes de
//! base de datos, cache y servicios externos:
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
//! Ese tipo se pasa a [`axum::Router::with_state`] y se extrae con [`State<T>`].

// Re-exports de extractores de Axum para los handlers del Core.
pub use axum::extract::{Path, Query, State};

// Re-exports del extractor de body con validacion desde Shield.
#[cfg(feature = "validation")]
pub use crate::shield::validation::ValidatedJson as ValidatedBody;

// Re-export del extractor de claims JWT desde Shield.
#[cfg(feature = "auth-jwt")]
pub use crate::shield::Claims;

/// Sistema de respuestas del Core.
///
/// Reexporta los tipos de respuesta de Axum y agrega variantes
/// convenientes para texto plano. Los handlers que devuelven JSON
/// usan [`response::Json`] directamente; los que devuelven texto plano
/// usan [`response::PlainText`].
pub mod response {
    pub use axum::http::StatusCode;
    pub use axum::response::{IntoResponse, Response};
    pub use axum::Json;

    /// Respuesta de texto plano con `Content-Type: text/plain; charset=utf-8`.
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

    /// Stream de bytes abierto.
    ///
    /// Util para respuestas chunked o para SSE basico. Para SSE completo
    /// con reconexion y event IDs usar `ag-realtime` (Fase 4).
    pub use axum::body::Body as BodyStream;
}

/// Estado compartido por defecto.
///
/// Tipo placeholder para proyectos que no necesitan estado global.
/// Los proyectos reales definen su propio estado con los clientes
/// de base de datos, cache y servicios externos, luego lo registran
/// con [`axum::Router::with_state`].
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
        // Verifica que los tipos esten disponibles en el modulo.
        // La compilacion ya valida la re-exportacion correcta.
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
