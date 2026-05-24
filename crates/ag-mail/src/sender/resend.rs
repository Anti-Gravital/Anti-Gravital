//! Adapter Resend (<https://resend.com>).
//!
//! Implementa `MailSender` via la API REST de Resend. Solo requiere
//! un token de API; no tiene dependencias nativas de correo.
//!
//! # Ejemplo
//!
//! ```rust,no_run
//! use ag_mail::sender::resend::{ResendConfig, ResendSender};
//! use ag_mail::sender::MailSender;
//! use ag_mail::message::{Address, EmailBuilder};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = ResendConfig::new("re_api_key_aqui");
//!     let sender = ResendSender::new(config);
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
use serde::{Deserialize, Serialize};

use crate::{
    error::AgMailError,
    message::{Address, Email},
    sender::{MailSender, SendResult},
};

const RESEND_BASE: &str = "https://api.resend.com";

/// Configuracion del adapter Resend.
#[derive(Debug, Clone)]
pub struct ResendConfig {
    /// Token de API de Resend (`re_...`).
    pub api_key: String,
    /// URL base de la API (sobreescribible en tests).
    pub base_url: String,
}

impl ResendConfig {
    /// Configuracion con la URL de produccion de Resend.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: RESEND_BASE.to_owned(),
        }
    }

    /// Configuracion con URL base personalizada (util para tests con wiremock).
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.into(),
        }
    }
}

/// Adapter Resend asincrono.
pub struct ResendSender {
    client: reqwest::Client,
    config: ResendConfig,
}

impl ResendSender {
    /// Construye el sender a partir de la configuracion.
    pub fn new(config: ResendConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }
}

#[async_trait]
impl MailSender for ResendSender {
    fn name(&self) -> &'static str {
        "resend"
    }

    async fn send(&self, email: &Email) -> Result<SendResult, AgMailError> {
        let body = build_resend_body(email);
        let url = format!("{}/emails", self.config.base_url);

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.config.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AgMailError::Provider {
                provider: "resend",
                message: e.to_string(),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AgMailError::Provider {
                provider: "resend",
                message: format!("HTTP {status}: {text}"),
            });
        }

        let result: ResendResponse = resp.json().await.map_err(|e| AgMailError::Provider {
            provider: "resend",
            message: format!("respuesta inesperada: {e}"),
        })?;

        Ok(SendResult {
            message_id: Some(result.id),
            provider: "resend",
        })
    }
}

// ---- Tipos internos --------------------------------------------------------

#[derive(Debug, Serialize)]
struct ResendBody<'a> {
    from: String,
    to: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cc: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    bcc: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to: Option<String>,
    subject: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    html: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct ResendResponse {
    id: String,
}

fn addr_display(addr: &Address) -> String {
    addr.display()
}

fn build_resend_body(email: &Email) -> ResendBody<'_> {
    ResendBody {
        from: addr_display(&email.from),
        to: email.to.iter().map(addr_display).collect(),
        cc: email.cc.iter().map(addr_display).collect(),
        bcc: email.bcc.iter().map(addr_display).collect(),
        reply_to: email.reply_to.as_ref().map(addr_display),
        subject: &email.subject,
        html: email.html_body.as_deref(),
        text: email.text_body.as_deref(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{Address, EmailBuilder};
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn minimal_email() -> Email {
        EmailBuilder::new()
            .from(Address::new("from@ejemplo.com"))
            .to(Address::new("to@ejemplo.com"))
            .subject("Asunto")
            .html_body("<p>Hola</p>")
            .build()
            .unwrap()
    }

    #[test]
    fn build_body_minimal() {
        let email = minimal_email();
        let body = build_resend_body(&email);
        assert_eq!(body.from, "from@ejemplo.com");
        assert_eq!(body.to, vec!["to@ejemplo.com"]);
        assert_eq!(body.subject, "Asunto");
        assert_eq!(body.html, Some("<p>Hola</p>"));
        assert!(body.text.is_none());
        assert!(body.cc.is_empty());
        assert!(body.bcc.is_empty());
    }

    #[test]
    fn build_body_with_name() {
        let email = EmailBuilder::new()
            .from(Address::with_name("Angel", "a@b.com"))
            .to(Address::new("t@t.com"))
            .subject("s")
            .text_body("t")
            .build()
            .unwrap();
        let body = build_resend_body(&email);
        assert_eq!(body.from, "Angel <a@b.com>");
    }

    #[test]
    fn build_body_cc_bcc_reply_to() {
        let email = EmailBuilder::new()
            .from(Address::new("f@f.com"))
            .to(Address::new("t@t.com"))
            .cc(Address::new("cc@cc.com"))
            .bcc(Address::new("bcc@bcc.com"))
            .reply_to(Address::new("reply@reply.com"))
            .subject("s")
            .text_body("t")
            .build()
            .unwrap();
        let body = build_resend_body(&email);
        assert_eq!(body.cc, vec!["cc@cc.com"]);
        assert_eq!(body.bcc, vec!["bcc@bcc.com"]);
        assert_eq!(body.reply_to, Some("reply@reply.com".to_owned()));
    }

    #[tokio::test]
    async fn send_ok() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/emails"))
            .and(header("Authorization", "Bearer test-key"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "msg-123"})),
            )
            .mount(&server)
            .await;

        let config = ResendConfig::with_base_url("test-key", server.uri());
        let sender = ResendSender::new(config);
        let result = sender.send(&minimal_email()).await.unwrap();
        assert_eq!(result.message_id.as_deref(), Some("msg-123"));
        assert_eq!(result.provider, "resend");
    }

    #[tokio::test]
    async fn send_api_error_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/emails"))
            .respond_with(
                ResponseTemplate::new(422)
                    .set_body_string(r#"{"name":"validation_error","message":"invalid from"}"#),
            )
            .mount(&server)
            .await;

        let config = ResendConfig::with_base_url("bad-key", server.uri());
        let sender = ResendSender::new(config);
        let err = sender.send(&minimal_email()).await.unwrap_err();
        assert!(matches!(
            err,
            AgMailError::Provider {
                provider: "resend",
                ..
            }
        ));
    }

    #[tokio::test]
    async fn send_unauthorized_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/emails"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&server)
            .await;

        let config = ResendConfig::with_base_url("wrong-key", server.uri());
        let sender = ResendSender::new(config);
        let err = sender.send(&minimal_email()).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("401"), "debe mencionar el codigo HTTP");
    }
}
