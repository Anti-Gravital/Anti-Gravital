//! Tipo de error del crate `ag-domains`.
//!
//! Las conversiones `From` para errores de upstreams concretos
//! (`reqwest`, `instant_acme`, `hickory_resolver`) se añaden cuando esos
//! upstreams se introducen en las Etapas 2-3 y 2-4.

use thiserror::Error;

/// Errores producidos por las operaciones de `ag-domains`.
#[derive(Debug, Error)]
pub enum AgDomainsError {
    /// La zona DNS solicitada no existe en el proveedor.
    #[error("zona DNS no encontrada: {0}")]
    ZoneNotFound(String),

    /// El registro DNS solicitado no existe.
    #[error("registro DNS no encontrado: {0}")]
    RecordNotFound(String),

    /// Error de autenticacion contra el proveedor DNS.
    #[error("autenticacion fallida con el proveedor ({provider}): {message}")]
    Auth {
        /// Nombre del proveedor reportando el fallo.
        provider: &'static str,
        /// Mensaje propagado.
        message: String,
    },

    /// Error de propagacion: los registros aun no se ven en los resolvers.
    #[error("propagacion DNS pendiente: {0}")]
    PropagationPending(String),

    /// Error de ACME (emision o renovacion de certificado).
    #[error("error ACME: {0}")]
    Acme(String),

    /// Error generico del proveedor DNS.
    #[error("error del proveedor DNS ({provider}): {message}")]
    Provider {
        /// Nombre del proveedor reportando el fallo.
        provider: &'static str,
        /// Mensaje propagado.
        message: String,
    },

    /// Error generico de configuracion.
    #[error("configuracion invalida: {0}")]
    Config(String),
}
