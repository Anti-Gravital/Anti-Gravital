//! Error type for the `ag-mail` crate.
//!
//! Conversions from provider errors (SMTP, adapter HTTP) are added when
//! the corresponding modules are implemented. For now the enum covers the
//! most common variants so the rest of the skeleton can reference them.

use thiserror::Error;

/// Errors produced by `ag-mail` operations.
#[derive(Debug, Error)]
pub enum AgMailError {
    /// The email could not be sent after exhausting the retries.
    #[error("envio de correo agotado tras reintentos: {0}")]
    SendExhausted(String),

    /// The source or destination address is not valid.
    #[error("direccion de correo invalida: {0}")]
    InvalidAddress(String),

    /// The template does not exist or could not be rendered.
    #[error("error de template: {0}")]
    Template(String),

    /// The variables declared in the DSL `mail` block do not match the ones
    /// in the template HTML.
    #[error("incoherencia entre variables declaradas y template: {0}")]
    VarMismatch(String),

    /// Generic configuration error (missing environment variable, invalid
    /// credential, etc).
    #[error("configuracion invalida: {0}")]
    Config(String),

    /// Generic provider error (native SMTP or adapter).
    #[error("error del proveedor de correo ({provider}): {message}")]
    Provider {
        /// Name of the provider reporting the failure.
        provider: &'static str,
        /// Message propagated by the provider.
        message: String,
    },
}
