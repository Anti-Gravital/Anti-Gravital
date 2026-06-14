//! Native outbound MTA engine (Phases 4.6-A and 4.6-B).
//!
//! `MtaSender` is a new, opt-in [`MailSender`](crate::sender::MailSender) that
//! delivers mail **directly** to the recipients' mail servers instead of
//! relaying through an SMTP host or a provider HTTP API. It resolves the
//! destination MX, optionally selects an egress source (source IP + EHLO) from
//! a weighted pool, opens an ESMTP session with opportunistic STARTTLS, signs
//! the message with DKIM (Ed25519 or RSA), and submits it.
//!
//! This is additive: it does not replace the default `SmtpSender` relay, and it
//! lives behind the `mta` Cargo feature. Implemented: MX resolution
//! ([`resolve`]), egress sources/pools ([`egress`]), DKIM signing ([`dkim`]),
//! bounce classification ([`bounce`]), per-`site_name` traffic shaping
//! ([`shaping`]), a two-tier scheduled/ready delivery queue with retry/backoff
//! and automatic suppression ([`queue`], [`suppress`]), and `ag-observe`
//! metrics. `MtaSender` is the direct sender and also a [`queue::DeliveryBackend`].
//! A durable queue spool (JetStream / PostgreSQL) and asynchronous DSN/FBL
//! intake are later, optional parts of Phase 4.6-B (`RFC-0009`, `docs/DEBT.md`).
//!
//! Governing decision: `ADR-0010`; technical plan: `RFC-0009`.

pub mod bounce;
pub mod dkim;
pub mod dsn;
pub mod egress;
pub mod queue;
pub mod resolve;
pub mod shaping;
pub mod suppress;

use std::collections::BTreeMap;
use std::sync::Arc;

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
pub use dsn::{process_arf, process_dsn, DsnAction, DsnRecord};
pub use egress::{EgressPool, EgressSource};
pub use queue::{
    CycleReport, DeliveryBackend, DeliveryJob, DeliveryOutcome, MtaQueue, MtaRetryPolicy,
    QueueConfig,
};
pub use resolve::ResolverConfigMta;
pub use shaping::{Shaper, ShapingConfig, ShapingLimits};
pub use suppress::{SuppressionList, SuppressionReason};

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
    /// Optional egress pool. When set, each delivery picks a source (source IP
    /// and EHLO name) by weighted round-robin; when `None`, the OS default
    /// route and `helo_host` are used.
    pub egress: Option<Arc<EgressPool>>,
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
            egress: None,
        }
    }

    /// Sets the DKIM signing configuration.
    pub fn with_dkim(mut self, dkim: DkimConfig) -> Self {
        self.dkim = Some(dkim);
        self
    }

    /// Sets the egress pool used to pick the source IP and EHLO per delivery.
    pub fn with_egress(mut self, pool: Arc<EgressPool>) -> Self {
        self.egress = Some(pool);
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

    /// Renders `email` and splits it into one [`queue::DeliveryJob`] per
    /// recipient domain, ready to be enqueued on a [`queue::MtaQueue`]. The
    /// shaping `site_name` is set to the recipient domain (the true MX rollup is
    /// resolved at delivery time). `now` is the logical enqueue time.
    pub fn build_jobs(
        &self,
        email: &Email,
        tenant: impl Into<String>,
        campaign: impl Into<String>,
        now: std::time::Instant,
    ) -> Result<Vec<queue::DeliveryJob>, AgMailError> {
        let content = self.render(email)?;
        let groups = group_recipients_by_domain(email)?;
        let tenant = tenant.into();
        let campaign = campaign.into();
        Ok(groups
            .into_iter()
            .map(|(domain, recipients)| {
                queue::DeliveryJob::new(
                    tenant.clone(),
                    campaign.clone(),
                    domain.clone(),
                    domain,
                    email.from.email.clone(),
                    recipients,
                    content.clone(),
                    now,
                )
            })
            .collect())
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
        egress: Option<&EgressSource>,
    ) -> Result<(), AgMailError> {
        let hosts = resolve::resolve_mx(domain, &self.config.resolver).await?;
        if hosts.is_empty() {
            return Err(AgMailError::NoMailHost(domain.to_owned()));
        }

        let rcpts: Vec<&str> = recipients.iter().map(String::as_str).collect();
        let mut last_err: Option<SubmitError> = None;

        for host in &hosts {
            match self
                .submit(&host.exchange, from, &rcpts, content, egress)
                .await
            {
                Ok(()) => return Ok(()),
                Err(err) => {
                    crate::metrics::record_retry("mta");
                    tracing::warn!(
                        mx = %host.exchange,
                        domain = %domain,
                        error = %err.message,
                        "MX delivery attempt failed, trying next host"
                    );
                    last_err = Some(err);
                }
            }
        }

        Err(last_err
            .map(|e| AgMailError::Provider {
                provider: "mta",
                message: e.message,
            })
            .unwrap_or_else(|| AgMailError::NoMailHost(domain.to_owned())))
    }

    /// Opens an ESMTP session to a single MX host and submits the message,
    /// classifying any failure as transient or permanent.
    async fn submit(
        &self,
        mx_host: &str,
        from: &str,
        rcpts: &[&str],
        content: &[u8],
        egress: Option<&EgressSource>,
    ) -> Result<(), SubmitError> {
        // The egress source, when present, overrides the EHLO name and binds
        // the outgoing socket to a specific source IP.
        let helo = egress.map_or(self.config.helo_host.as_str(), |s| s.ehlo_domain.as_str());
        let mut builder = SmtpClientBuilder::new(mx_host, self.config.port)
            .map_err(|e| SubmitError::transient(format!("smtp client: {e}")))?
            .implicit_tls(false)
            .helo_host(helo.to_owned());
        if let Some(ip) = egress.and_then(|s| s.source_ip) {
            builder = builder.local_ip(ip);
        }
        if self.config.opportunistic_tls {
            builder = builder.allow_invalid_certs();
        }

        // DKIM-sign last, right before submission, so the signature covers the
        // exact bytes that go over the wire. A signing failure is our own fault,
        // not the recipient's, so it is transient (retryable), never permanent.
        let body = match &self.config.dkim {
            Some(dkim) => dkim
                .sign(content)
                .map_err(|e| SubmitError::transient(format!("dkim: {e}")))?,
            None => content.to_vec(),
        };

        let mut client = builder.connect().await.map_err(classify_mail_send)?;
        let envelope = Envelope::new(from, rcpts.iter().copied(), body.as_slice());
        client.send(envelope).await.map_err(classify_mail_send)?;
        Ok(())
    }
}

#[async_trait]
impl queue::DeliveryBackend for MtaSender {
    async fn deliver(&self, job: &queue::DeliveryJob) -> queue::DeliveryOutcome {
        use queue::DeliveryOutcome;

        let egress = self.config.egress.as_ref().map(|pool| pool.select());
        let rcpts: Vec<&str> = job.recipients.iter().map(String::as_str).collect();

        let hosts = match resolve::resolve_mx(&job.domain, &self.config.resolver).await {
            Ok(hosts) if !hosts.is_empty() => hosts,
            Ok(_) => return DeliveryOutcome::Transient(format!("no MX for {}", job.domain)),
            Err(e) => return DeliveryOutcome::Transient(format!("MX resolution: {e}")),
        };

        let mut last = String::from("delivery failed");
        for host in &hosts {
            match self
                .submit(
                    &host.exchange,
                    &job.from,
                    &rcpts,
                    &job.content,
                    egress.as_ref(),
                )
                .await
            {
                Ok(()) => return DeliveryOutcome::Delivered,
                // A permanent (5xx) rejection will repeat on every MX, so stop
                // and let the queue suppress the recipients.
                Err(SubmitError {
                    kind: SubmitKind::Permanent,
                    message,
                }) => return DeliveryOutcome::Permanent(message),
                Err(SubmitError { message, .. }) => {
                    crate::metrics::record_retry("mta");
                    last = message;
                }
            }
        }
        DeliveryOutcome::Transient(last)
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
            // Pick one egress identity for the whole message so its EHLO and
            // source IP stay consistent across domains and MX retries.
            let egress = self.config.egress.as_ref().map(|pool| pool.select());
            for (domain, recipients) in groups {
                self.deliver_domain(
                    &domain,
                    &recipients,
                    &content,
                    &email.from.email,
                    egress.as_ref(),
                )
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

/// Whether a submit failure is worth retrying or is a permanent rejection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubmitKind {
    /// Temporary failure (4xx, connection, TLS, local signing): retry later.
    Transient,
    /// Permanent destination rejection (5xx): suppress the recipients.
    Permanent,
}

/// A classified delivery-attempt failure.
#[derive(Debug)]
struct SubmitError {
    kind: SubmitKind,
    message: String,
}

impl SubmitError {
    fn transient(message: impl Into<String>) -> Self {
        Self {
            kind: SubmitKind::Transient,
            message: message.into(),
        }
    }

    fn permanent(message: impl Into<String>) -> Self {
        Self {
            kind: SubmitKind::Permanent,
            message: message.into(),
        }
    }
}

/// Classifies a `mail-send` SMTP error. Only a permanent (5xx) reply from the
/// destination is permanent; everything else (4xx, connection, TLS, timeout) is
/// transient and retryable.
fn classify_mail_send(error: mail_send::Error) -> SubmitError {
    match error {
        mail_send::Error::UnexpectedReply(reply)
        | mail_send::Error::AuthenticationFailed(reply) => {
            let message = format!("{} {}", reply.code, reply.message);
            match bounce::classify_code(reply.code) {
                Some(bounce::BounceClass::Permanent) => SubmitError::permanent(message),
                _ => SubmitError::transient(message),
            }
        }
        other => SubmitError::transient(other.to_string()),
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
        assert!(cfg.egress.is_none());
        assert_eq!(MtaSender::new(cfg).name(), "mta");
    }

    #[test]
    fn config_with_egress_pool() {
        let pool = EgressPool::new(
            "tenant-a",
            vec![
                EgressSource::new("ip1", "mx1.send.example").with_weight(3),
                EgressSource::new("ip2", "mx2.send.example"),
            ],
        )
        .unwrap();
        let cfg = MtaConfig::new("mail.send.example").with_egress(Arc::new(pool));
        let egress = cfg.egress.as_ref().expect("egress set");
        assert_eq!(egress.len(), 2);
        // Selection cycles through the pool deterministically.
        let first = egress.select();
        assert!(["ip1", "ip2"].contains(&first.name.as_str()));
    }

    #[test]
    fn name_is_mta() {
        let sender = MtaSender::new(MtaConfig::new("mail.send.example"));
        assert_eq!(sender.name(), "mta");
    }

    #[test]
    fn build_jobs_one_per_domain() {
        let sender = MtaSender::new(MtaConfig::new("mail.send.example"));
        let now = std::time::Instant::now();
        let jobs = sender
            .build_jobs(&email(), "tenant-a", "welcome", now)
            .unwrap();
        // gmail.com (to+cc) and outlook.com (bcc) => two jobs.
        assert_eq!(jobs.len(), 2);
        let gmail = jobs.iter().find(|j| j.domain == "gmail.com").unwrap();
        assert_eq!(gmail.tenant, "tenant-a");
        assert_eq!(gmail.campaign, "welcome");
        assert_eq!(gmail.site_name, "gmail.com");
        assert_eq!(gmail.recipients.len(), 2);
        assert!(!gmail.content.is_empty());
        assert_eq!(gmail.scheduled_key(), "tenant-a:welcome:gmail.com");
    }

    // ---- ESMTP + STARTTLS protocol path against a local in-process sink -----
    //
    // The direct-MX submit path used to be exercised only by `#[ignore]` tests
    // needing real DNS/port-25 egress. These tests stand up a minimal SMTP sink
    // on loopback that advertises STARTTLS and upgrades with a self-signed
    // certificate, then drive `MtaSender::submit` against it. So the EHLO ->
    // STARTTLS -> EHLO -> MAIL -> RCPT -> DATA dialogue (and the DKIM-signed body
    // on the wire) runs in CI with no external services. Real external-MX
    // delivery stays a manual gate (the `#[ignore]` live resolver test), since
    // port 25 egress is blocked here.

    /// What the sink observed during one SMTP session (post-STARTTLS).
    #[derive(Default)]
    struct SinkCapture {
        mail_from: String,
        rcpts: Vec<String>,
        data: Vec<u8>,
    }

    /// Builds a `tokio-rustls` acceptor with a throwaway self-signed cert for
    /// `127.0.0.1`. The client accepts it via opportunistic TLS
    /// (`allow_invalid_certs`).
    fn test_tls_acceptor() -> tokio_rustls::TlsAcceptor {
        use rustls::ServerConfig;
        use rustls_pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
        use std::sync::Arc;

        let ck = rcgen::generate_simple_self_signed(vec!["127.0.0.1".to_owned()]).unwrap();
        let cert = ck.cert.der().clone();
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(ck.key_pair.serialize_der()));
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .unwrap();
        tokio_rustls::TlsAcceptor::from(Arc::new(config))
    }

    /// Reads one CRLF-terminated line without buffering past it, so the raw
    /// stream is left intact for the TLS handshake after `STARTTLS`.
    async fn read_line<S: tokio::io::AsyncRead + Unpin>(stream: &mut S) -> Option<String> {
        use tokio::io::AsyncReadExt;
        let mut buf = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            match stream.read(&mut byte).await {
                Ok(0) => {
                    return (!buf.is_empty()).then(|| String::from_utf8_lossy(&buf).into_owned());
                }
                Ok(_) => {
                    buf.push(byte[0]);
                    if byte[0] == b'\n' {
                        return Some(String::from_utf8_lossy(&buf).into_owned());
                    }
                }
                Err(_) => return None,
            }
        }
    }

    /// Drives the post-STARTTLS SMTP dialogue over the (now TLS) stream.
    async fn serve_smtp<S>(mut stream: S, capture: &mut SinkCapture)
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        use tokio::io::AsyncWriteExt;
        while let Some(line) = read_line(&mut stream).await {
            let cmd = line.trim_end();
            let upper = cmd.to_ascii_uppercase();
            if upper.starts_with("EHLO") || upper.starts_with("HELO") {
                stream.write_all(b"250-sink\r\n250 OK\r\n").await.unwrap();
            } else if upper.starts_with("MAIL FROM") {
                capture.mail_from = cmd.to_owned();
                stream.write_all(b"250 OK\r\n").await.unwrap();
            } else if upper.starts_with("RCPT TO") {
                capture.rcpts.push(cmd.to_owned());
                stream.write_all(b"250 OK\r\n").await.unwrap();
            } else if upper.starts_with("DATA") {
                stream
                    .write_all(b"354 End data with <CR><LF>.<CR><LF>\r\n")
                    .await
                    .unwrap();
                while let Some(dl) = read_line(&mut stream).await {
                    if dl == ".\r\n" {
                        break;
                    }
                    capture.data.extend_from_slice(dl.as_bytes());
                }
                stream.write_all(b"250 OK queued\r\n").await.unwrap();
            } else if upper.starts_with("QUIT") {
                stream.write_all(b"221 Bye\r\n").await.unwrap();
                break;
            } else {
                stream.write_all(b"250 OK\r\n").await.unwrap();
            }
        }
    }

    /// One-session SMTP sink: greets, advertises STARTTLS, upgrades to TLS, then
    /// accepts one message over the encrypted channel.
    async fn run_smtp_sink(
        listener: tokio::net::TcpListener,
        acceptor: tokio_rustls::TlsAcceptor,
    ) -> SinkCapture {
        use tokio::io::AsyncWriteExt;

        let (mut stream, _) = listener.accept().await.expect("sink accepts a connection");
        let mut capture = SinkCapture::default();

        stream.write_all(b"220 sink ESMTP\r\n").await.unwrap();
        // Plaintext phase: greet EHLO, advertise STARTTLS, then upgrade.
        loop {
            let Some(line) = read_line(&mut stream).await else {
                return capture;
            };
            let upper = line.trim_end().to_ascii_uppercase();
            if upper.starts_with("EHLO") || upper.starts_with("HELO") {
                stream
                    .write_all(b"250-sink\r\n250-STARTTLS\r\n250 OK\r\n")
                    .await
                    .unwrap();
            } else if upper.starts_with("STARTTLS") {
                stream.write_all(b"220 Go ahead\r\n").await.unwrap();
                break;
            } else if upper.starts_with("QUIT") {
                stream.write_all(b"221 Bye\r\n").await.unwrap();
                return capture;
            } else {
                stream.write_all(b"250 OK\r\n").await.unwrap();
            }
        }

        let tls = acceptor.accept(stream).await.expect("TLS handshake");
        serve_smtp(tls, &mut capture).await;
        capture
    }

    #[tokio::test]
    async fn submit_speaks_esmtp_over_starttls_to_local_sink() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let sink = tokio::spawn(run_smtp_sink(listener, test_tls_acceptor()));

        // Default config: opportunistic STARTTLS, accepting the self-signed cert.
        let sender = MtaSender::new(MtaConfig::new("test.local").with_port(port));

        let content = b"From: from@send.example\r\nSubject: hi\r\n\r\nbody\r\n";
        let result = sender
            .submit(
                "127.0.0.1",
                "from@send.example",
                &["rcpt@example.com"],
                content,
                None,
            )
            .await;
        assert!(
            result.is_ok(),
            "submit should succeed against the sink: {result:?}"
        );

        let cap = sink.await.unwrap();
        assert!(
            cap.mail_from.contains("from@send.example"),
            "MAIL FROM seen: {}",
            cap.mail_from
        );
        assert!(cap.rcpts.iter().any(|r| r.contains("rcpt@example.com")));
        assert!(String::from_utf8_lossy(&cap.data).contains("Subject: hi"));
    }

    #[tokio::test]
    async fn submit_signs_with_dkim_on_the_wire() {
        use mail_auth::common::crypto::Ed25519Key;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let sink = tokio::spawn(run_smtp_sink(listener, test_tls_acceptor()));

        let pkcs8 = Ed25519Key::generate_pkcs8().expect("key generation succeeds");
        let dkim = DkimConfig::ed25519("send.example", "sel", pkcs8);
        let sender = MtaSender::new(MtaConfig::new("test.local").with_port(port).with_dkim(dkim));

        let content = b"From: from@send.example\r\nSubject: hi\r\n\r\nbody\r\n";
        let result = sender
            .submit(
                "127.0.0.1",
                "from@send.example",
                &["rcpt@example.com"],
                content,
                None,
            )
            .await;
        assert!(result.is_ok(), "DKIM submit should succeed: {result:?}");

        let cap = sink.await.unwrap();
        let wire = String::from_utf8_lossy(&cap.data);
        assert!(
            wire.contains("DKIM-Signature:"),
            "the message on the wire must carry a DKIM signature"
        );
    }
}
