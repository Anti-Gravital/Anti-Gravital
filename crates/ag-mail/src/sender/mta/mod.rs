//! Native outbound MTA engine (Phase 4.6-A).
//!
//! `MtaSender` is a new, opt-in [`MailSender`](crate::sender::MailSender) that
//! delivers mail **directly** to the recipients' mail servers instead of
//! relaying through an SMTP host or a provider HTTP API. It resolves the
//! destination MX, opens an ESMTP session with opportunistic STARTTLS, signs
//! the message with DKIM, and submits it.
//!
//! This is additive: it does not replace the default `SmtpSender` relay or any
//! provider adapter, and it lives behind the `mta` Cargo feature. Durable
//! scheduled/ready queues, per-`site_name` traffic shaping, egress IP pools,
//! and asynchronous DSN/FBL processing are later phases (`RFC-0009`); this
//! phase is the synchronous direct-delivery core plus bounce classification.
//!
//! Governing decision: `ADR-0010`; technical plan: `RFC-0009`.

pub mod bounce;
pub mod dkim;
pub mod resolve;

use std::collections::BTreeMap;

use async_trait::async_trait;
use mail_send::{
    mail_builder::{headers::address::Address as MbAddress, MessageBuilder},
    smtp::message::Message as Envelope,
    SmtpClientBuilder,
};

use crate::{
    error::AgMailError,
    message::{Address, Email},
    sender::{MailSender, SendResult},
};

pub use dkim::DkimConfig;
pub use resolve::ResolverConfigMta;

/// Configuration for the native MTA sender.
#[derive(Debug, Clone)]
pub struct MtaConfig {
    /// EHLO/HELO hostname this sender announces. Should be a real FQDN with
    /// valid forward and reverse (PTR) DNS for deliverability.
    pub helo_host: String,
    /// Destination port for the recipient MX (25 for inter-server delivery).
    pub port: u16,
    /// Use opportunistic STARTTLS and accept the MX certificate even when it
    /// does not validate. This is standard for inter-MTA delivery, where the
    /// alternative is sending in the clear.
    pub opportunistic_tls: bool,
    /// DNS resolver configuration for MX lookups.
    pub resolver: ResolverConfigMta,
    /// Optional DKIM signing configuration. When `None`, messages are sent
    /// unsigned (not recommended for production).
    pub dkim: Option<DkimConfig>,
}

impl MtaConfig {
    /// Creates a configuration with sensible defaults for the given EHLO host.
    pub fn new(helo_host: impl Into<String>) -> Self {
        Self {
            helo_host: helo_host.into(),
            port: 25,
            opportunistic_tls: true,
            resolver: ResolverConfigMta::default(),
            dkim: None,
        }
    }

    /// Sets the DKIM signing configuration.
    pub fn with_dkim(mut self, dkim: DkimConfig) -> Self {
        self.dkim = Some(dkim);
        self
    }

    /// Overrides the destination port (useful for tests against a local sink).
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Overrides the resolver configuration.
    pub fn with_resolver(mut self, resolver: ResolverConfigMta) -> Self {
        self.resolver = resolver;
        self
    }
}

/// Native outbound MTA sender.
#[derive(Debug, Clone)]
pub struct MtaSender {
    config: MtaConfig,
}

impl MtaSender {
    /// Builds the sender from its configuration.
    pub fn new(config: MtaConfig) -> Self {
        Self { config }
    }

    /// Renders the RFC 5322 message bytes once; the same content is submitted
    /// to every destination MX (the envelope differs per domain).
    fn render(&self, email: &Email) -> Result<Vec<u8>, AgMailError> {
        let mut builder = MessageBuilder::new()
            .from(mb_address(&email.from))
            .subject(email.subject.clone());

        if !email.to.is_empty() {
            builder = builder.to(mb_list(&email.to));
        }
        if !email.cc.is_empty() {
            builder = builder.cc(mb_list(&email.cc));
        }
        if let Some(reply_to) = &email.reply_to {
            builder = builder.reply_to(mb_address(reply_to));
        }
        if let Some(html) = &email.html_body {
            builder = builder.html_body(html.clone());
        }
        if let Some(text) = &email.text_body {
            builder = builder.text_body(text.clone());
        }

        builder.write_to_vec().map_err(|e| AgMailError::Provider {
            provider: "mta",
            message: format!("MIME build failed: {e}"),
        })
    }

    /// Delivers the rendered content to one recipient domain, trying each MX
    /// host in preference order until one accepts the message.
    async fn deliver_domain(
        &self,
        domain: &str,
        recipients: &[String],
        content: &[u8],
        from: &str,
    ) -> Result<(), AgMailError> {
        let hosts = resolve::resolve_mx(domain, &self.config.resolver).await?;
        if hosts.is_empty() {
            return Err(AgMailError::NoMailHost(domain.to_owned()));
        }

        let rcpts: Vec<&str> = recipients.iter().map(String::as_str).collect();
        let mut last_err: Option<AgMailError> = None;

        for host in &hosts {
            match self.submit(&host.exchange, from, &rcpts, content).await {
                Ok(()) => return Ok(()),
                Err(err) => {
                    crate::metrics::record_retry("mta");
                    tracing::warn!(
                        mx = %host.exchange,
                        domain = %domain,
                        error = %err,
                        "MX delivery attempt failed, trying next host"
                    );
                    last_err = Some(err);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| AgMailError::NoMailHost(domain.to_owned())))
    }

    /// Opens an ESMTP session to a single MX host and submits the message.
    async fn submit(
        &self,
        mx_host: &str,
        from: &str,
        rcpts: &[&str],
        content: &[u8],
    ) -> Result<(), AgMailError> {
        let mut builder = SmtpClientBuilder::new(mx_host, self.config.port)
            .map_err(|e| AgMailError::Config(e.to_string()))?
            .implicit_tls(false)
            .helo_host(self.config.helo_host.clone());
        if self.config.opportunistic_tls {
            builder = builder.allow_invalid_certs();
        }

        // DKIM-sign last, right before submission, so the signature covers the
        // exact bytes that go over the wire.
        let body = match &self.config.dkim {
            Some(dkim) => dkim.sign(content)?,
            None => content.to_vec(),
        };

        let mut client = builder.connect().await.map_err(provider_err)?;
        let envelope = Envelope::new(from, rcpts.iter().copied(), body.as_slice());
        client.send(envelope).await.map_err(provider_err)?;
        Ok(())
    }
}

#[async_trait]
impl MailSender for MtaSender {
    fn name(&self) -> &'static str {
        "mta"
    }

    async fn send(&self, email: &Email) -> Result<SendResult, AgMailError> {
        let start = std::time::Instant::now();
        let outcome = async {
            let content = self.render(email)?;
            let groups = group_recipients_by_domain(email)?;
            for (domain, recipients) in groups {
                self.deliver_domain(&domain, &recipients, &content, &email.from.email)
                    .await?;
            }
            Ok::<(), AgMailError>(())
        }
        .await;

        crate::metrics::record_send("mta", outcome.is_ok());
        if outcome.is_ok() {
            crate::metrics::record_send_latency("mta", start.elapsed().as_secs_f64());
        }

        outcome.map(|()| SendResult {
            message_id: None,
            provider: "mta",
        })
    }
}

// ---- helpers ---------------------------------------------------------------

/// Groups all recipients (To, Cc, Bcc) by their domain so each MX receives
/// only the envelope recipients it is responsible for. Deterministic order via
/// `BTreeMap`.
fn group_recipients_by_domain(email: &Email) -> Result<BTreeMap<String, Vec<String>>, AgMailError> {
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let all = email.to.iter().chain(&email.cc).chain(&email.bcc);
    for addr in all {
        let domain = recipient_domain(&addr.email)?;
        groups.entry(domain).or_default().push(addr.email.clone());
    }
    if groups.is_empty() {
        return Err(AgMailError::Config(
            "el correo debe tener al menos un destinatario".to_owned(),
        ));
    }
    Ok(groups)
}

/// Extracts and normalizes the domain part of an address.
fn recipient_domain(email: &str) -> Result<String, AgMailError> {
    email
        .rsplit_once('@')
        .map(|(_, domain)| domain.trim().to_ascii_lowercase())
        .filter(|d| !d.is_empty())
        .ok_or_else(|| AgMailError::InvalidAddress(email.to_owned()))
}

/// Converts an `ag-mail` address into a `mail-builder` address.
fn mb_address(addr: &Address) -> MbAddress<'static> {
    match &addr.name {
        Some(name) => MbAddress::new_address(Some(name.clone()), addr.email.clone()),
        None => MbAddress::new_address(None::<String>, addr.email.clone()),
    }
}

/// Converts a list of addresses into a single `mail-builder` address list.
fn mb_list(addrs: &[Address]) -> MbAddress<'static> {
    MbAddress::new_list(addrs.iter().map(mb_address).collect())
}

/// Maps a `mail-send` SMTP error to an `ag-mail` provider error.
fn provider_err(error: mail_send::Error) -> AgMailError {
    AgMailError::Provider {
        provider: "mta",
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::EmailBuilder;

    fn email() -> Email {
        EmailBuilder::new()
            .from(Address::with_name("Sender", "noreply@send.example"))
            .to(Address::new("alice@gmail.com"))
            .cc(Address::new("bob@gmail.com"))
            .bcc(Address::new("carol@outlook.com"))
            .subject("Hello")
            .html_body("<p>hi</p>")
            .text_body("hi")
            .build()
            .unwrap()
    }

    #[test]
    fn groups_recipients_by_domain() {
        let groups = group_recipients_by_domain(&email()).unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups["gmail.com"].len(), 2);
        assert_eq!(groups["outlook.com"], vec!["carol@outlook.com"]);
    }

    #[test]
    fn recipient_domain_lowercased() {
        assert_eq!(recipient_domain("X@Gmail.COM").unwrap(), "gmail.com");
        assert!(recipient_domain("no-domain").is_err());
    }

    #[test]
    fn renders_mime_content() {
        let sender = MtaSender::new(MtaConfig::new("mail.send.example"));
        let bytes = sender.render(&email()).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("Subject: Hello"));
        assert!(text.contains("alice@gmail.com"));
    }

    #[test]
    fn config_defaults_to_mta_port_and_tls() {
        let cfg = MtaConfig::new("mail.send.example");
        assert_eq!(cfg.port, 25);
        assert!(cfg.opportunistic_tls);
        assert!(cfg.dkim.is_none());
        assert_eq!(MtaSender::new(cfg).name(), "mta");
    }

    #[test]
    fn name_is_mta() {
        let sender = MtaSender::new(MtaConfig::new("mail.send.example"));
        assert_eq!(sender.name(), "mta");
    }
}
