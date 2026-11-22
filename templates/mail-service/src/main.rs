//! Transactional mail service scaffold built on `ag-mail`.
//!
//! The default sender is the native SMTP relay. When `AG_SMTP_HOST` is not
//! configured the service runs in dry-run mode: messages are validated and
//! logged but never leave the process. This keeps the template buildable and
//! runnable offline; no mail server is required to compile or start it.

use std::env;
use std::sync::Arc;

use ag_mail::message::{Address, Email, EmailBuilder};
use ag_mail::sender::smtp::{SmtpConfig, SmtpSender};
use ag_mail::sender::{MailSender, SendResult};
use ag_mail::AgMailError;
use async_trait::async_trait;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
struct SendRequest {
    to: String,
    subject: String,
    body: String,
}

/// DKIM configuration stub. The fields are read and logged at startup so the
/// deployment can confirm its settings. Actual signing is provided by the
/// opt-in `mta` feature of `ag-mail` with a deployment-supplied private key.
struct DkimStub {
    selector: String,
    domain: String,
    key_path: Option<String>,
}

impl DkimStub {
    fn from_env() -> Self {
        Self {
            selector: env::var("AG_DKIM_SELECTOR").unwrap_or_default(),
            domain: env::var("AG_DKIM_DOMAIN").unwrap_or_default(),
            key_path: env::var("AG_DKIM_KEY_PATH").ok(),
        }
    }
}

/// Offline sender used when no SMTP relay is configured.
struct DryRunSender;

#[async_trait]
impl MailSender for DryRunSender {
    fn name(&self) -> &'static str {
        "dry-run"
    }

    async fn send(&self, email: &Email) -> Result<SendResult, AgMailError> {
        tracing::warn!(
            to = ?email.to,
            subject = %email.subject,
            "dry-run: email accepted but not delivered"
        );
        Ok(SendResult {
            message_id: Some("dry-run".into()),
            provider: "dry-run",
        })
    }
}

fn from_address() -> Address {
    Address::new(
        env::var("AG_MAIL_FROM").unwrap_or_else(|_| "no-reply@localhost".into()),
    )
}

async fn health() -> &'static str {
    "ok"
}

async fn send_mail(
    State(mailer): State<Arc<dyn MailSender>>,
    Json(request): Json<SendRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let email = EmailBuilder::new()
        .from(from_address())
        .to(Address::new(&request.to))
        .subject(&request.subject)
        .text_body(&request.body)
        .build()
        .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))?;

    let result = mailer
        .send(&email)
        .await
        .map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))?;

    Ok(Json(json!({
        "message_id": result.message_id,
        "provider": result.provider,
    })))
}

fn build_mailer() -> Result<Arc<dyn MailSender>, AgMailError> {
    let Ok(host) = env::var("AG_SMTP_HOST") else {
        tracing::warn!("AG_SMTP_HOST is not set; using the dry-run sender");
        return Ok(Arc::new(DryRunSender));
    };

    let port = env::var("AG_SMTP_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(587);

    let config = match (env::var("AG_SMTP_USERNAME"), env::var("AG_SMTP_PASSWORD")) {
        (Ok(username), Ok(password)) => SmtpConfig::new(host, port, username, password),
        _ => SmtpConfig::new_unauthenticated(host, port),
    };

    Ok(Arc::new(SmtpSender::new(config)?))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let dkim = DkimStub::from_env();
    tracing::info!(
        dkim_selector = %dkim.selector,
        dkim_domain = %dkim.domain,
        dkim_key_configured = dkim.key_path.is_some(),
        "DKIM configuration stub loaded; signing requires ag-mail mta feature"
    );

    let mailer = build_mailer()?;
    let app = Router::new()
        .route("/health", get(health))
        .route("/send", post(send_mail))
        .with_state(mailer);

    let bind = env::var("BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "{{name}} listening");
    axum::serve(listener, app).await?;
    Ok(())
}
