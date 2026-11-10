//! `ag mail` commands (ag-mail).

#[allow(clippy::too_many_arguments)]
pub(crate) async fn cmd_mail_test(
    to: &str,
    from: &str,
    subject: &str,
    smtp_host: &str,
    smtp_port: u16,
    smtp_user: Option<String>,
    smtp_pass: Option<String>,
) -> Result<(), String> {
    use ag_mail::{
        message::{Address, EmailBuilder},
        sender::{
            smtp::{SmtpConfig, SmtpSender},
            MailSender,
        },
    };

    println!("Sending test email to '{to}'...");
    println!("  SMTP: {smtp_host}:{smtp_port}");

    let config = match (smtp_user, smtp_pass) {
        (Some(user), Some(pass)) => SmtpConfig::new(smtp_host, smtp_port, user, pass),
        _ => SmtpConfig::new_unauthenticated(smtp_host, smtp_port),
    };

    let sender = SmtpSender::new(config).map_err(|e| format!("invalid SMTP configuration: {e}"))?;

    let email = EmailBuilder::new()
        .from(Address::new(from))
        .to(Address::new(to))
        .subject(subject)
        .html_body(format!(
            "<p>Test email sent by <code>ag mail test</code>.</p>\
             <p>If you can read this message, the SMTP configuration is correct.</p>\
             <p>Host: <strong>{smtp_host}:{smtp_port}</strong></p>"
        ))
        .text_body(format!(
            "Test email from ag mail test.\nHost: {smtp_host}:{smtp_port}"
        ))
        .build()
        .map_err(|e| format!("invalid email address: {e}"))?;

    let result = sender
        .send(&email)
        .await
        .map_err(|e| format!("send failed: {e}"))?;

    match result.message_id {
        Some(id) => println!("OK: email sent. Message-ID: {id}"),
        None => println!("OK: email sent."),
    }

    Ok(())
}
