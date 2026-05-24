//! Tests E2E de Fase 4.5: ag-mail, ag-domains y la integracion ag-auth+ag-mail.
//!
//! Estos tests verifican que los modulos cooperan correctamente sin
//! necesitar servicios externos (SMTP real, Cloudflare API, DNS real).
//! Se usan NullSender e InMemoryProvider para controlar el entorno.

use std::sync::Arc;

use ag_auth::{mailer::AuthMailer, AgAuth, AuthConfig};
use ag_domains::{
    mail_records::{
        apply_mail_records, DkimSelector, DmarcPolicy, MailDnsRequirements, MailRecordsResult,
    },
    provider::DnsProvider,
    record::{DnsRecord, DnsRecordSpec},
    AgDomainsError,
};
use ag_mail::{
    message::{Address, EmailBuilder},
    queue::{InMemoryQueue, MailQueue, RetryPolicy},
    sender::{test_utils::NullSender, MailSender},
    template::{MailTemplate, StringTemplate},
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::time::Duration;

// ---- Helpers ----------------------------------------------------------------

fn make_auth_config() -> AuthConfig {
    AuthConfig {
        jwt_private_key_pem: "demo-priv".to_owned(),
        jwt_public_key_pem: "demo-pub".to_owned(),
        webauthn_rp_id: String::new(),
        webauthn_origin: String::new(),
        oauth_google_client_id: None,
        oauth_google_client_secret: None,
        oauth_github_client_id: None,
        oauth_github_client_secret: None,
    }
}

/// Proveedor DNS en memoria para tests.
#[derive(Default)]
struct InMemoryDnsProvider {
    records: Mutex<Vec<DnsRecord>>,
}

#[async_trait]
impl DnsProvider for InMemoryDnsProvider {
    fn name(&self) -> &'static str {
        "memory"
    }

    async fn zone_id(&self, domain: &str) -> Result<String, AgDomainsError> {
        Ok(format!("zone-{domain}"))
    }

    async fn list_records(&self, _zone_id: &str) -> Result<Vec<DnsRecord>, AgDomainsError> {
        Ok(self.records.lock().unwrap().clone())
    }

    async fn create_record(
        &self,
        _zone_id: &str,
        spec: &DnsRecordSpec,
    ) -> Result<DnsRecord, AgDomainsError> {
        let id = format!("rec-{}", self.records.lock().unwrap().len());
        let record = DnsRecord {
            id: id.clone(),
            zone_id: _zone_id.to_owned(),
            name: spec.name.clone(),
            record_type: spec.record_type.clone(),
            content: spec.content.clone(),
            ttl: spec.ttl,
            proxied: spec.proxied,
        };
        self.records.lock().unwrap().push(record.clone());
        Ok(record)
    }

    async fn update_record(
        &self,
        _zone_id: &str,
        record_id: &str,
        spec: &DnsRecordSpec,
    ) -> Result<DnsRecord, AgDomainsError> {
        let mut records = self.records.lock().unwrap();
        if let Some(existing) = records.iter_mut().find(|r| r.id == record_id) {
            existing.content = spec.content.clone();
            return Ok(existing.clone());
        }
        Err(AgDomainsError::RecordNotFound(record_id.to_owned()))
    }

    async fn delete_record(&self, _zone_id: &str, _record_id: &str) -> Result<(), AgDomainsError> {
        Ok(())
    }
}

fn mail_requirements(domain: &str, dkim_selectors: Vec<&str>) -> MailDnsRequirements {
    MailDnsRequirements {
        domain: domain.to_owned(),
        dkim_selectors: dkim_selectors
            .into_iter()
            .map(|s| DkimSelector {
                name: s.to_owned(),
                public_key_record: format!("v=DKIM1; k=rsa; p=KEY_{s}"),
            })
            .collect(),
        spf_includes: vec!["include:spf.example.com".to_owned()],
        spf_ips: Vec::new(),
        dmarc_policy: DmarcPolicy::None,
        dmarc_rua: Some("mailto:dmarc@example.com".to_owned()),
    }
}

// ---- ag-mail: envio basico --------------------------------------------------

#[tokio::test]
async fn test_mail_null_sender_captures_email() {
    let sender = Arc::new(NullSender::new());
    let email = EmailBuilder::new()
        .from(Address::new("from@example.com"))
        .to(Address::new("to@example.com"))
        .subject("Test E2E")
        .text_body("Hola desde E2E")
        .build()
        .expect("email valido");

    sender
        .send(&email)
        .await
        .expect("NullSender no debe fallar");
    assert_eq!(sender.emails_sent(), 1);
    let last = sender.last_email().unwrap();
    assert_eq!(last.subject, "Test E2E");
}

// ---- ag-mail: template StringTemplate --------------------------------------

#[test]
fn test_mail_template_renders_vars() {
    let tpl = StringTemplate::html_and_text(
        "Bienvenido {{nombre}}",
        "<p>Hola {{nombre}}, tu enlace es {{link}}</p>",
        "Hola {{nombre}}, tu enlace es {{link}}",
    );

    let vars: HashMap<String, String> = [
        ("nombre".to_owned(), "Angel".to_owned()),
        ("link".to_owned(), "https://x.com".to_owned()),
    ]
    .into();

    let subject = tpl.render_subject(&vars).expect("subject ok");
    let html = tpl.render_html(&vars).expect("html ok").unwrap();
    let text = tpl.render_text(&vars).expect("text ok").unwrap();

    assert!(subject.contains("Angel"));
    assert!(html.contains("https://x.com"));
    assert!(text.contains("Angel"));
}

// ---- ag-mail: cola con reintentos ------------------------------------------

#[tokio::test]
async fn test_mail_queue_enqueue_multiple() {
    let policy = RetryPolicy {
        max_retries: 2,
        base_delay: Duration::from_millis(10),
        backoff_factor: 2,
    };
    let sender = Arc::new(NullSender::new());
    let queue = InMemoryQueue::new(sender.clone(), policy, 64);

    for i in 0..3u32 {
        let email = EmailBuilder::new()
            .from(Address::new("from@example.com"))
            .to(Address::new(format!("user{i}@example.com")))
            .subject(format!("Correo {i}"))
            .text_body("cuerpo")
            .build()
            .expect("email valido");
        queue.enqueue(email).await.expect("enqueue no debe fallar");
    }
    // La cola procesa async; damos tiempo al worker.
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(sender.emails_sent(), 3);
}

// ---- ag-domains: apply_mail_records ----------------------------------------

#[tokio::test]
async fn test_domains_apply_creates_spf_dkim_dmarc() {
    let provider = InMemoryDnsProvider::default();
    let req = mail_requirements("example.com", vec!["mail"]);

    let result: MailRecordsResult = apply_mail_records(&req, &provider, "zone-1")
        .await
        .expect("apply debe tener exito");

    // SPF + DKIM (1 selector) + DMARC = 3 registros
    assert_eq!(result.created, 3, "deben crearse 3 registros");
    assert_eq!(result.updated, 0);
    assert_eq!(result.unchanged, 0);
}

#[tokio::test]
async fn test_domains_apply_is_idempotent() {
    let provider = InMemoryDnsProvider::default();
    let req = mail_requirements("example.com", vec!["mail"]);

    apply_mail_records(&req, &provider, "zone-1")
        .await
        .expect("primera aplicacion");

    let second: MailRecordsResult = apply_mail_records(&req, &provider, "zone-1")
        .await
        .expect("segunda aplicacion");

    assert_eq!(
        second.created, 0,
        "segunda aplicacion no debe crear registros"
    );
    assert_eq!(
        second.unchanged, 3,
        "todos los registros deben estar sin cambios"
    );
}

// ---- ag-auth + ag-mail: integracion cross-module ---------------------------

#[tokio::test]
async fn test_auth_mail_verification_flow() {
    let sender = Arc::new(NullSender::new());
    let sender_dyn: Arc<dyn MailSender> = sender.clone();

    let mailer = Arc::new(AuthMailer::new(
        sender_dyn,
        "no-reply@example.com",
        "TestApp",
    ));

    let auth = AgAuth::new(make_auth_config(), reqwest::Client::new())
        .expect("auth config valida")
        .with_mail(Arc::clone(&mailer));

    assert!(
        auth.mailer.is_some(),
        "AgAuth debe tener el mailer configurado"
    );

    let (token, _hash) = auth.create_api_key("vrf");
    mailer
        .send_verification("nuevo@usuario.com", &token, "https://app.example.com")
        .await
        .expect("send_verification no debe fallar");

    assert_eq!(sender.emails_sent(), 1);
    let email = sender.last_email().unwrap();
    assert!(
        email.subject.contains("Verifica"),
        "asunto debe contener 'Verifica'"
    );
    assert!(
        email
            .html_body
            .as_deref()
            .unwrap_or("")
            .contains("verify?token="),
        "html debe contener el enlace de verificacion"
    );
}

#[tokio::test]
async fn test_auth_mail_all_three_flows() {
    let sender = Arc::new(NullSender::new());
    let sender_dyn: Arc<dyn MailSender> = sender.clone();
    let mailer = Arc::new(AuthMailer::new(sender_dyn, "noreply@x.com", "X"));

    mailer
        .send_verification("a@b.com", "t1", "https://x.com")
        .await
        .unwrap();
    mailer
        .send_password_reset("a@b.com", "t2", "https://x.com")
        .await
        .unwrap();
    mailer
        .send_magic_link("a@b.com", "t3", "https://x.com")
        .await
        .unwrap();

    assert_eq!(sender.emails_sent(), 3, "deben haberse enviado 3 correos");
}
