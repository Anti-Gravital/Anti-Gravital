//! Tipo de error del crate `ag-mail`.
//!
//! Conversiones desde errores de proveedores (SMTP, HTTP de adapters) se
//! añaden cuando se implementan los modulos correspondientes. Por ahora
//! el enum cubre las variantes mas comunes para que el resto del skeleton
//! pueda referenciarlas.

use thiserror::Error;

/// Errores producidos por las operaciones de `ag-mail`.
#[derive(Debug, Error)]
pub enum AgMailError {
    /// El correo no se pudo enviar despues de agotar los reintentos.
    #[error("envio de correo agotado tras reintentos: {0}")]
    SendExhausted(String),

    /// La direccion de origen o destino no es valida.
    #[error("direccion de correo invalida: {0}")]
    InvalidAddress(String),

    /// El template no existe o no se pudo renderizar.
    #[error("error de template: {0}")]
    Template(String),

    /// Las variables declaradas en el bloque `mail` del DSL no coinciden con
    /// las del template HTML.
    #[error("incoherencia entre variables declaradas y template: {0}")]
    VarMismatch(String),

    /// Error generico de configuracion (falta variable de entorno, credencial
    /// invalida, etc).
    #[error("configuracion invalida: {0}")]
    Config(String),

    /// Error generico de proveedor (SMTP nativo o adapter).
    #[error("error del proveedor de correo ({provider}): {message}")]
    Provider {
        /// Nombre del proveedor reportando el fallo.
        provider: &'static str,
        /// Mensaje propagado por el proveedor.
        message: String,
    },
}
