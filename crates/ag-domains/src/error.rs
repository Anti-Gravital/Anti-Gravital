//! Error type for the `ag-domains` crate.
//!
//! The `From` conversions for errors from concrete upstreams
//! (`reqwest`, `instant_acme`, `hickory_resolver`) are added when those
//! upstreams are introduced in Stages 2-3 and 2-4.

use thiserror::Error;

/// Errors produced by `ag-domains` operations.
#[derive(Debug, Error)]
pub enum AgDomainsError {
    /// The requested DNS zone does not exist in the provider.
    #[error("zona DNS no encontrada: {0}")]
    ZoneNotFound(String),

    /// The requested DNS record does not exist.
    #[error("registro DNS no encontrado: {0}")]
    RecordNotFound(String),

    /// Authentication error against the DNS provider.
    #[error("autenticacion fallida con el proveedor ({provider}): {message}")]
    Auth {
        /// Name of the provider reporting the failure.
        provider: &'static str,
        /// Propagated message.
        message: String,
    },

    /// Propagation error: records are not yet visible on the resolvers.
    #[error("propagacion DNS pendiente: {0}")]
    PropagationPending(String),

    /// ACME error (certificate issuance or renewal).
    #[error("error ACME: {0}")]
    Acme(String),

    /// Generic DNS provider error.
    #[error("error del proveedor DNS ({provider}): {message}")]
    Provider {
        /// Name of the provider reporting the failure.
        provider: &'static str,
        /// Propagated message.
        message: String,
    },

    /// Generic configuration error.
    #[error("configuracion invalida: {0}")]
    Config(String),
}
