//! Native SMTP sender based on `lettre` with TLS via rustls.
//!
//! Supports authenticated SMTP (LOGIN/PLAIN) and STARTTLS. For servers
//! without authentication (dev/test) use `SmtpConfig::new_unauthenticated`.
//!
//! # Example
//!
//! ```rust,no_run
//! use ag_mail::sender::smtp::{SmtpConfig, SmtpSender};
//! use ag_mail::sender::MailSender;
//! use ag_mail::message::{Address, EmailBuilder};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = SmtpConfig::new("smtp.ejemplo.com", 587, "user", "pass");
//!     let sender = SmtpSender::new(config).expect("SMTP sender construido");
//!
//!     let email = EmailBuilder::new()
//!         .from(Address::new("noreply@ejemplo.com"))
//!         .to(Address::new("usuario@ejemplo.com"))
//!         .subject("Bienvenido")
//!         .html_body("<h1>Hola</h1>")
//!         .build()
//!         .unwrap();
//!
//!     sender.send(&email).await.expect("enviado");
//! }
//! ```

use async_trait::async_trait;
use lettre::{
    message::{
        header::{ContentType, HeaderName, HeaderValue},
        Mailbox, MessageBuilder, MultiPart, SinglePart,
    },
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};

use crate::{
    error::AgMailError,
    message::{Address, Email},
    sender::{MailSender, SendResult},
};

/// SMTP sender configuration.
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    /// SMTP server hostname.
    pub host: String,
    /// Port (usually 587 for STARTTLS, 465 for SSL).
    pub port: u16,
    /// Authentication username.
    pub username: Option<String>,
    /// Authentication password.
    pub password: Option<String>,
}

impl SmtpConfig {
    /// Configuration with authentication.
    pub fn new(
        host: impl Into<String>,
        port: u16,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            username: Some(username.into()),
            password: Some(password.into()),
        }
    }

    /// Configuration without authentication (useful in development environments).
    pub fn new_unauthenticated(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            username: None,
            password: None,
        }
    }
}

/// Asynchronous SMTP sender.
pub struct SmtpSender {
    transport: AsyncSmtpTransport<Tokio1Executor>,
}

impl SmtpSender {
    /// Builds the sender from the configuration.
    pub fn new(config: SmtpConfig) -> Result<Self, AgMailError> {
        let builder = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)
            .map_err(|e| AgMailError::Config(e.to_string()))?
            .port(config.port);

        let transport = match (config.username, config.password) {
            (Some(u), Some(p)) => builder.credentials(Credentials::new(u, p)).build(),
            _ => builder.build(),
        };

        Ok(Self { transport })
    }
}

#[async_trait]
impl MailSender for SmtpSender {
    fn name(&self) -> &'static str {
        "smtp"
    }

    async fn send(&self, email: &Email) -> Result<SendResult, AgMailError> {
        let lettre_msg = build_lettre_message(email)?;

        self.transport
            .send(lettre_msg)
            .await
            .map_err(|e| AgMailError::Provider {
                provider: "smtp",
                message: e.to_string(),
            })?;

        Ok(SendResult {
            message_id: None,
            provider: "smtp",
        })
    }
}

// ---- Conversion ag-mail::Email -> lettre::Message --------------------------

fn build_lettre_message(email: &Email) -> Result<lettre::Message, AgMailError> {
    let mut builder = MessageBuilder::new()
        .from(to_mailbox(&email.from)?)
        .subject(&email.subject);

    for addr in &email.to {
        builder = builder.to(to_mailbox(addr)?);
    }
    for addr in &email.cc {
        builder = builder.cc(to_mailbox(addr)?);
    }
    for addr in &email.bcc {
        builder = builder.bcc(to_mailbox(addr)?);
    }
    if let Some(reply_to) = &email.reply_to {
        builder = builder.reply_to(to_mailbox(reply_to)?);
    }
    for (name, value) in &email.headers {
        if let Ok(header_name) = HeaderName::new_from_ascii(name.clone()) {
            builder = builder.raw_header(HeaderValue::new(header_name, value.clone()));
        } else {
            tracing::warn!(name = %name, "skipping custom SMTP header with non-ASCII name");
        }
    }

    let msg = match (&email.html_body, &email.text_body) {
        (Some(html), Some(text)) => builder
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text.clone()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html.clone()),
                    ),
            )
            .map_err(|e| AgMailError::Provider {
                provider: "smtp",
                message: e.to_string(),
            })?,
        (Some(html), None) => builder
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(html.clone()),
            )
            .map_err(|e| AgMailError::Provider {
                provider: "smtp",
                message: e.to_string(),
            })?,
        (None, Some(text)) => builder
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_PLAIN)
                    .body(text.clone()),
            )
            .map_err(|e| AgMailError::Provider {
                provider: "smtp",
                message: e.to_string(),
            })?,
        (None, None) => {
            return Err(AgMailError::Config(
                "el correo requiere html_body o text_body".to_owned(),
            ))
        }
    };

    Ok(msg)
}

fn to_mailbox(addr: &Address) -> Result<Mailbox, AgMailError> {
    let mailbox = match &addr.name {
        Some(name) => format!("{name} <{}>", addr.email)
            .parse::<Mailbox>()
            .map_err(|e| AgMailError::InvalidAddress(e.to_string()))?,
        None => addr
            .email
            .parse::<Mailbox>()
            .map_err(|e| AgMailError::InvalidAddress(e.to_string()))?,
    };
    Ok(mailbox)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_mailbox_plain_address() {
        let addr = Address::new("test@ejemplo.com");
        let mb = to_mailbox(&addr).unwrap();
        assert_eq!(mb.email.to_string(), "test@ejemplo.com");
    }

    #[test]
    fn to_mailbox_with_name() {
        let addr = Address::with_name("Angel", "angel@ejemplo.com");
        let mb = to_mailbox(&addr).unwrap();
        assert_eq!(mb.email.to_string(), "angel@ejemplo.com");
        assert!(mb.name.is_some());
    }

    #[test]
    fn to_mailbox_invalid_rejects() {
        let addr = Address::new("no-es-un-email");
        assert!(to_mailbox(&addr).is_err());
    }

    #[test]
    fn build_lettre_html_only() {
        use crate::message::EmailBuilder;
        let email = EmailBuilder::new()
            .from(Address::new("f@f.com"))
            .to(Address::new("t@t.com"))
            .subject("Test")
            .html_body("<p>hola</p>")
            .build()
            .unwrap();
        assert!(build_lettre_message(&email).is_ok());
    }

    #[test]
    fn build_lettre_text_only() {
        use crate::message::EmailBuilder;
        let email = EmailBuilder::new()
            .from(Address::new("f@f.com"))
            .to(Address::new("t@t.com"))
            .subject("Test")
            .text_body("hola")
            .build()
            .unwrap();
        assert!(build_lettre_message(&email).is_ok());
    }

    #[test]
    fn build_lettre_multipart() {
        use crate::message::EmailBuilder;
        let email = EmailBuilder::new()
            .from(Address::new("f@f.com"))
            .to(Address::new("t@t.com"))
            .subject("Test")
            .html_body("<p>hola</p>")
            .text_body("hola")
            .build()
            .unwrap();
        assert!(build_lettre_message(&email).is_ok());
    }

    #[test]
    fn custom_headers_are_applied() {
        use crate::message::EmailBuilder;
        let email = EmailBuilder::new()
            .from(Address::new("a@x.com"))
            .to(Address::new("b@y.com"))
            .subject("s")
            .text_body("t")
            .header("X-Campaign", "spring")
            .build()
            .unwrap();
        let message = build_lettre_message(&email).unwrap();
        let formatted = String::from_utf8(message.formatted()).unwrap();
        assert!(
            formatted.contains("X-Campaign"),
            "custom header must appear in formatted message"
        );
    }
}
