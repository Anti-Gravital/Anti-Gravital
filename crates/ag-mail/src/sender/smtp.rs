//! Sender SMTP nativo basado en `lettre` con TLS via rustls.
//!
//! Soporta SMTP autenticado (LOGIN/PLAIN) y STARTTLS. Para servidores
//! sin autenticacion (dev/test) usa `SmtpConfig::new_unauthenticated`.
//!
//! # Ejemplo
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
    message::{header::ContentType, Mailbox, MessageBuilder, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
// TECH-DEBT: headers personalizadas (Email::headers) no se propagan al mensaje
// lettre porque la API de lettre 0.11 requiere tipos Header estaticos, no strings.
// motivo: complejidad del API de lettre para headers arbitrarias
// impacto: los campos X-Custom del Email se ignoran en el transporte SMTP
// eliminacion esperada: cuando se actualice a lettre 0.12+

use crate::{
    error::AgMailError,
    message::{Address, Email},
    sender::{MailSender, SendResult},
};

/// Configuracion del sender SMTP.
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    /// Hostname del servidor SMTP.
    pub host: String,
    /// Puerto (habitualmente 587 para STARTTLS, 465 para SSL).
    pub port: u16,
    /// Usuario de autenticacion.
    pub username: Option<String>,
    /// Contrasena de autenticacion.
    pub password: Option<String>,
}

impl SmtpConfig {
    /// Configuracion con autenticacion.
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

    /// Configuracion sin autenticacion (util en entornos de desarrollo).
    pub fn new_unauthenticated(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            username: None,
            password: None,
        }
    }
}

/// Sender SMTP asincrono.
pub struct SmtpSender {
    transport: AsyncSmtpTransport<Tokio1Executor>,
}

impl SmtpSender {
    /// Construye el sender a partir de la configuracion.
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
    // headers personalizadas: ver TECH-DEBT arriba

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
}
