//! Sending authentication emails: verification, recovery and magic link.
//!
//! `AuthMailer` wraps any `ag_mail::sender::MailSender` and exposes
//! three high-level operations. The caller injects the concrete sender
//! (SMTP, Resend, NullSender in tests) when constructing `AgAuth`.
//!
//! # Dependency rule
//!
//! `ag-auth` depends on `ag-mail` (feature `mail`). The reverse direction
//! is forbidden by ADR-0007 and the sixth dependency rule of CLAUDE.md.

use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;

use ag_mail::{
    message::{Address, EmailBuilder},
    sender::MailSender,
    AgMailError,
};

/// Error that an authentication email operation can produce.
#[derive(Debug, Error)]
pub enum AuthMailerError {
    /// The underlying sender rejected or failed to send.
    #[error("fallo de envio: {0}")]
    Send(#[from] AgMailError),
}

/// Sends authentication emails using an injected `MailSender`.
///
/// # Example
///
/// ```no_run
/// use std::sync::Arc;
/// use ag_auth::mailer::AuthMailer;
/// use ag_mail::sender::smtp::{SmtpConfig, SmtpSender};
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let config = SmtpConfig::new_unauthenticated("localhost", 1025);
/// let sender = SmtpSender::new(config)?;
/// let mailer = AuthMailer::new(Arc::new(sender), "no-reply@example.com", "MiApp");
///
/// mailer.send_verification("usuario@example.com", "TOKEN123", "https://example.com").await?;
/// # Ok(())
/// # }
/// ```
pub struct AuthMailer {
    sender: Arc<dyn MailSender>,
    from: String,
    project_name: String,
}

impl AuthMailer {
    /// Creates a new `AuthMailer`.
    ///
    /// - `sender`: concrete implementation of the `MailSender` trait.
    /// - `from`: sender address that will appear in all emails.
    /// - `project_name`: project name used in the subjects.
    pub fn new(
        sender: Arc<dyn MailSender>,
        from: impl Into<String>,
        project_name: impl Into<String>,
    ) -> Self {
        Self {
            sender,
            from: from.into(),
            project_name: project_name.into(),
        }
    }

    /// Sends an email verification message.
    ///
    /// The generated link is `{base_url}/verify?token={token}`.
    pub async fn send_verification(
        &self,
        to: &str,
        token: &str,
        base_url: &str,
    ) -> Result<(), AuthMailerError> {
        let link = format!("{base_url}/verify?token={token}");
        let subject = format!("{} — Verifica tu email", self.project_name);
        let html = render_html(
            AUTH_VERIFICATION_HTML,
            &[("link", &link), ("project", &self.project_name)],
        );
        let text = render_text(
            AUTH_VERIFICATION_TEXT,
            &[("link", &link), ("project", &self.project_name)],
        );
        self.send_email(to, &subject, &html, &text).await
    }

    /// Sends a password reset email.
    ///
    /// The generated link is `{base_url}/reset?token={token}`.
    pub async fn send_password_reset(
        &self,
        to: &str,
        token: &str,
        base_url: &str,
    ) -> Result<(), AuthMailerError> {
        let link = format!("{base_url}/reset?token={token}");
        let subject = format!("{} — Restablece tu contrasena", self.project_name);
        let html = render_html(
            AUTH_RESET_HTML,
            &[("link", &link), ("project", &self.project_name)],
        );
        let text = render_text(
            AUTH_RESET_TEXT,
            &[("link", &link), ("project", &self.project_name)],
        );
        self.send_email(to, &subject, &html, &text).await
    }

    /// Sends an email with a magic link for passwordless authentication.
    ///
    /// The generated link is `{base_url}/magic?token={token}`.
    pub async fn send_magic_link(
        &self,
        to: &str,
        token: &str,
        base_url: &str,
    ) -> Result<(), AuthMailerError> {
        let link = format!("{base_url}/magic?token={token}");
        let subject = format!("{} — Accede con tu magic link", self.project_name);
        let html = render_html(
            AUTH_MAGIC_HTML,
            &[("link", &link), ("project", &self.project_name)],
        );
        let text = render_text(
            AUTH_MAGIC_TEXT,
            &[("link", &link), ("project", &self.project_name)],
        );
        self.send_email(to, &subject, &html, &text).await
    }

    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        html: &str,
        text: &str,
    ) -> Result<(), AuthMailerError> {
        let email = EmailBuilder::new()
            .from(Address::new(&self.from))
            .to(Address::new(to))
            .subject(subject)
            .html_body(html)
            .text_body(text)
            .build()
            .map_err(AuthMailerError::Send)?;

        self.sender
            .send(&email)
            .await
            .map_err(AuthMailerError::Send)?;
        Ok(())
    }
}

// ---- Inline templates -------------------------------------------------------

const AUTH_VERIFICATION_HTML: &str = "\
<p>Haz clic en el siguiente enlace para verificar tu email:</p>\
<p><a href=\"{{link}}\">Verificar email</a></p>\
<p>Si no solicitaste esto, ignora este mensaje.</p>\
<p>— {{project}}</p>";

const AUTH_VERIFICATION_TEXT: &str = "\
Verifica tu email abriendo este enlace:\n\
{{link}}\n\
Si no solicitaste esto, ignora este mensaje.\n\
— {{project}}";

const AUTH_RESET_HTML: &str = "\
<p>Haz clic en el siguiente enlace para restablecer tu contrasena:</p>\
<p><a href=\"{{link}}\">Restablecer contrasena</a></p>\
<p>El enlace expira en 30 minutos.</p>\
<p>Si no solicitaste esto, ignora este mensaje.</p>\
<p>— {{project}}</p>";

const AUTH_RESET_TEXT: &str = "\
Restablece tu contrasena abriendo este enlace:\n\
{{link}}\n\
El enlace expira en 30 minutos.\n\
Si no solicitaste esto, ignora este mensaje.\n\
— {{project}}";

const AUTH_MAGIC_HTML: &str = "\
<p>Haz clic en el siguiente enlace para acceder sin contrasena:</p>\
<p><a href=\"{{link}}\">Acceder ahora</a></p>\
<p>El enlace expira en 15 minutos y solo puede usarse una vez.</p>\
<p>— {{project}}</p>";

const AUTH_MAGIC_TEXT: &str = "\
Accede sin contrasena abriendo este enlace:\n\
{{link}}\n\
El enlace expira en 15 minutos y solo puede usarse una vez.\n\
— {{project}}";

// ---- Simple {{var}} rendering -----------------------------------------------

fn render_html(template: &str, vars: &[(&str, &str)]) -> String {
    let map: HashMap<&str, &str> = vars.iter().copied().collect();
    let mut result = template.to_owned();
    for (k, v) in &map {
        result = result.replace(&format!("{{{{{k}}}}}"), v);
    }
    result
}

fn render_text(template: &str, vars: &[(&str, &str)]) -> String {
    render_html(template, vars)
}

// ---- Tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ag_mail::sender::test_utils::NullSender;

    fn null_mailer(from: &str, project: &str) -> (Arc<NullSender>, AuthMailer) {
        let null = Arc::new(NullSender::new());
        let sender: Arc<dyn MailSender> = null.clone();
        let mailer = AuthMailer::new(sender, from, project);
        (null, mailer)
    }

    #[tokio::test]
    async fn send_verification_uses_null_sender() {
        let (null, mailer) = null_mailer("no-reply@x.com", "X");
        mailer
            .send_verification("a@b.com", "t", "https://x.com")
            .await
            .unwrap();
        assert_eq!(null.emails_sent(), 1);
    }

    #[tokio::test]
    async fn send_password_reset_uses_null_sender() {
        let (null, mailer) = null_mailer("no-reply@x.com", "X");
        mailer
            .send_password_reset("a@b.com", "t", "https://x.com")
            .await
            .unwrap();
        assert_eq!(null.emails_sent(), 1);
    }

    #[tokio::test]
    async fn send_magic_link_uses_null_sender() {
        let (null, mailer) = null_mailer("no-reply@x.com", "X");
        mailer
            .send_magic_link("a@b.com", "t", "https://x.com")
            .await
            .unwrap();
        assert_eq!(null.emails_sent(), 1);
    }

    #[tokio::test]
    async fn render_includes_link_and_project() {
        let (null, mailer) = null_mailer("no-reply@x.com", "Proyecto");
        mailer
            .send_verification("u@e.com", "TOKEN", "https://base.com")
            .await
            .unwrap();
        let email = null
            .last_email()
            .expect("debe haber un email en NullSender");
        let html = email.html_body.unwrap_or_default();
        assert!(
            html.contains("https://base.com/verify?token=TOKEN"),
            "html debe contener el enlace completo"
        );
        assert!(
            html.contains("Proyecto"),
            "html debe contener el nombre del proyecto"
        );
    }

    #[test]
    fn render_html_substitutes_vars() {
        let result = render_html(
            "Hola {{name}}, enlace: {{link}}",
            &[("name", "Angel"), ("link", "https://x.com")],
        );
        assert_eq!(result, "Hola Angel, enlace: https://x.com");
    }
}
