//! `MailSender` trait and senders.
//!
//! `MailSender` is the outbound port (interface) of `ag-mail`. The concrete
//! senders are the native SMTP relay (`SmtpSender`, default) and the native
//! outbound MTA (`mta::MtaSender`, opt-in). To send through an external email
//! provider, point `SmtpSender` at that provider's SMTP endpoint.

use async_trait::async_trait;

use crate::{error::AgMailError, message::Email};

#[cfg(feature = "smtp")]
pub mod smtp;

#[cfg(feature = "mta")]
pub mod mta;

/// Result of a successful send.
#[derive(Debug, Clone)]
pub struct SendResult {
    /// Message identifier assigned by the provider (if it returns one).
    pub message_id: Option<String>,
    /// Name of the provider that performed the send.
    pub provider: &'static str,
}

/// Email sending abstraction.
///
/// Implemented by `SmtpSender` (default) and the provider adapters.
#[async_trait]
pub trait MailSender: Send + Sync {
    /// Provider name for logs and traces.
    fn name(&self) -> &'static str;

    /// Sends an email.
    ///
    /// Returns `SendResult` with the message id if the provider exposes it,
    /// or `AgMailError` if the send failed.
    async fn send(&self, email: &Email) -> Result<SendResult, AgMailError>;
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils {
    //! Test helpers: `NullSender` that captures the sent emails.

    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use super::*;

    /// Fake sender that stores emails in memory to verify in tests.
    #[derive(Clone, Default)]
    pub struct NullSender {
        /// Emails captured by the sender.
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
        /// Creates an empty sender.
        pub fn new() -> Self {
            Self::default()
        }

        /// Number of emails captured so far.
        pub fn emails_sent(&self) -> usize {
            self.sent.lock().unwrap().len()
        }

        /// Last captured email, if any.
        pub fn last_email(&self) -> Option<Email> {
            self.sent.lock().unwrap().last().cloned()
        }
    }
}
