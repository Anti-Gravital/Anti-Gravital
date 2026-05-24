//! Trait `MailSender` y adapters de proveedor.
//!
//! `MailSender` es el puerto (interfaz) de salida de `ag-mail`. Los adapters
//! (`SmtpSender`, `ResendSender`, etc.) son los implementations concretas
//! detrás de features de Cargo.

use async_trait::async_trait;

use crate::{error::AgMailError, message::Email};

#[cfg(feature = "smtp")]
pub mod smtp;

#[cfg(feature = "resend")]
pub mod resend;

#[cfg(feature = "ses")]
pub mod ses;

#[cfg(feature = "postmark")]
pub mod postmark;

/// Resultado de un envio exitoso.
#[derive(Debug, Clone)]
pub struct SendResult {
    /// Identificador del mensaje asignado por el proveedor (si lo devuelve).
    pub message_id: Option<String>,
    /// Nombre del proveedor que realizo el envio.
    pub provider: &'static str,
}

/// Abstraccion de envio de correo.
///
/// Implementado por `SmtpSender` (default) y los adapters de proveedor.
#[async_trait]
pub trait MailSender: Send + Sync {
    /// Nombre del proveedor para logs y trazas.
    fn name(&self) -> &'static str;

    /// Envia un correo electronico.
    ///
    /// Retorna `SendResult` con el id del mensaje si el proveedor lo expone,
    /// o `AgMailError` si el envio fallo.
    async fn send(&self, email: &Email) -> Result<SendResult, AgMailError>;
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils {
    //! Helpers para tests: `NullSender` que captura los correos enviados.

    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use super::*;

    /// Sender falso que almacena los correos en memoria para verificar en tests.
    #[derive(Clone, Default)]
    pub struct NullSender {
        /// Correos capturados por el sender.
        pub sent: Arc<Mutex<Vec<Email>>>,
    }

    #[async_trait]
    impl MailSender for NullSender {
        fn name(&self) -> &'static str {
            "null"
        }

        async fn send(&self, email: &Email) -> Result<SendResult, AgMailError> {
            self.sent.lock().unwrap().push(email.clone());
            Ok(SendResult {
                message_id: Some(format!("null-{}", self.sent.lock().unwrap().len())),
                provider: "null",
            })
        }
    }

    impl NullSender {
        /// Crea un sender vacio.
        pub fn new() -> Self {
            Self::default()
        }

        /// Numero de correos capturados hasta ahora.
        pub fn emails_sent(&self) -> usize {
            self.sent.lock().unwrap().len()
        }

        /// Ultimo correo capturado, si existe.
        pub fn last_email(&self) -> Option<Email> {
            self.sent.lock().unwrap().last().cloned()
        }
    }
}
