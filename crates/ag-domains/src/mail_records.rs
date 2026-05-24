//! Generacion y aplicacion de registros DNS para la entrega de correo.
//!
//! `ag-mail` declara sus requisitos a traves de `MailDnsRequirements`
//! y este modulo los materializa como `DnsRecordSpec` y los aplica via
//! `DnsProvider`. Sin ciclo de dependencia: `ag-mail` y `ag-domains` se
//! conocen solo por estos tipos intermedios.
//!
//! La funcion principal del flujo de cooperacion es `apply_mail_records`:
//! genera los registros necesarios y hace upsert contra la zona DNS,
//! creando los que no existen y actualizando los que difieren en contenido.

use crate::{
    error::AgDomainsError,
    metrics,
    provider::DnsProvider,
    record::{DnsRecordSpec, RecordType},
};

/// Requisitos DNS para la entrega de correo de un dominio.
///
/// El caller (tipicamente `ag-mail`) rellena esta estructura y la pasa a
/// `generate_mail_records`. `ag-domains` la materializa en registros DNS.
#[derive(Debug, Clone)]
pub struct MailDnsRequirements {
    /// Dominio raiz (e.g., `"ejemplo.com"`).
    pub domain: String,

    /// Selectores DKIM activos. Cada selector produce un registro TXT
    /// `{selector}._domainkey.{domain}`.
    pub dkim_selectors: Vec<DkimSelector>,

    /// Incluidos SPF adicionales (e.g., `"include:sendgrid.net"`).
    pub spf_includes: Vec<String>,

    /// IP addresses autorizadas en SPF (mecanismo `ip4:`/`ip6:`).
    pub spf_ips: Vec<String>,

    /// Politica DMARC: `"none"`, `"quarantine"` o `"reject"`.
    pub dmarc_policy: DmarcPolicy,

    /// Email al que enviar los reportes DMARC agregados.
    pub dmarc_rua: Option<String>,
}

/// Selector DKIM con su clave publica.
#[derive(Debug, Clone)]
pub struct DkimSelector {
    /// Nombre del selector (e.g., `"s1"`, `"mail"`).
    pub name: String,
    /// Valor del registro TXT DKIM (contenido completo, sin el prefijo `v=DKIM1`).
    pub public_key_record: String,
}

/// Politica DMARC (RFC 7489).
#[derive(Debug, Clone, Default)]
pub enum DmarcPolicy {
    /// Solo recopilar reportes, no afectar entrega.
    #[default]
    None,
    /// Marcar como spam los mensajes que fallen.
    Quarantine,
    /// Rechazar los mensajes que fallen.
    Reject,
}

impl DmarcPolicy {
    fn as_str(&self) -> &'static str {
        match self {
            DmarcPolicy::None => "none",
            DmarcPolicy::Quarantine => "quarantine",
            DmarcPolicy::Reject => "reject",
        }
    }
}

/// Genera los registros DNS necesarios para la entrega de correo.
///
/// Retorna una lista de `DnsRecordSpec` listos para aplicar via
/// `DnsProvider::create_record`. El caller debe verificar si ya existen
/// y hacer upsert si procede.
pub fn generate_mail_records(req: &MailDnsRequirements) -> Vec<DnsRecordSpec> {
    let mut records = Vec::new();

    // SPF
    records.push(spf_record(req));

    // DKIM por selector
    for selector in &req.dkim_selectors {
        records.push(dkim_record(&req.domain, selector));
    }

    // DMARC
    records.push(dmarc_record(req));

    records
}

fn spf_record(req: &MailDnsRequirements) -> DnsRecordSpec {
    let includes = req
        .spf_includes
        .iter()
        .map(|i| format!("include:{i}"))
        .collect::<Vec<_>>();

    let ips = req.spf_ips.iter().map(|ip| {
        if ip.contains(':') {
            format!("ip6:{ip}")
        } else {
            format!("ip4:{ip}")
        }
    });

    let mechanisms = std::iter::once("mx".to_owned())
        .chain(includes)
        .chain(ips)
        .collect::<Vec<_>>()
        .join(" ");

    DnsRecordSpec {
        name: req.domain.clone(),
        record_type: RecordType::Txt,
        content: format!("v=spf1 {mechanisms} ~all"),
        ttl: 300,
        proxied: false,
    }
}

fn dkim_record(domain: &str, selector: &DkimSelector) -> DnsRecordSpec {
    DnsRecordSpec {
        name: format!("{}._domainkey.{}", selector.name, domain),
        record_type: RecordType::Txt,
        content: selector.public_key_record.clone(),
        ttl: 300,
        proxied: false,
    }
}

fn dmarc_record(req: &MailDnsRequirements) -> DnsRecordSpec {
    let rua = req
        .dmarc_rua
        .as_deref()
        .map(|email| format!("; rua=mailto:{email}"))
        .unwrap_or_default();

    DnsRecordSpec {
        name: format!("_dmarc.{}", req.domain),
        record_type: RecordType::Txt,
        content: format!("v=DMARC1; p={}{}", req.dmarc_policy.as_str(), rua),
        ttl: 300,
        proxied: false,
    }
}

/// Resultado del upsert de registros DNS de correo.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MailRecordsResult {
    /// Registros creados porque no existian.
    pub created: usize,
    /// Registros actualizados porque el contenido diferia.
    pub updated: usize,
    /// Registros sin cambios (ya existian con el contenido correcto).
    pub unchanged: usize,
}

/// Aplica los registros DNS necesarios para la entrega de correo.
///
/// Para cada registro que `generate_mail_records` produce:
/// - Si no existe en la zona: se crea con `DnsProvider::create_record`.
/// - Si existe pero el contenido es diferente: se actualiza.
/// - Si existe con el mismo contenido: se omite (idempotente).
///
/// La funcion es segura de llamar repetidamente; las re-ejecuciones
/// devuelven `unchanged` para todos los registros ya correctos.
pub async fn apply_mail_records<P: DnsProvider>(
    req: &MailDnsRequirements,
    provider: &P,
    zone_id: &str,
) -> Result<MailRecordsResult, AgDomainsError> {
    let desired = generate_mail_records(req);
    let existing = provider.list_records(zone_id).await?;

    let mut result = MailRecordsResult::default();

    for spec in &desired {
        // Busca un registro existente con el mismo nombre y tipo.
        let existing_match = existing
            .iter()
            .find(|r| r.name == spec.name && r.record_type == spec.record_type);

        match existing_match {
            None => {
                provider.create_record(zone_id, spec).await?;
                metrics::record_dns_upsert(
                    provider.name(),
                    &format!("{:?}", spec.record_type),
                    true,
                );
                result.created += 1;
            }
            Some(existing_rec) if existing_rec.content != spec.content => {
                provider
                    .update_record(zone_id, &existing_rec.id, spec)
                    .await?;
                metrics::record_dns_upsert(
                    provider.name(),
                    &format!("{:?}", spec.record_type),
                    true,
                );
                result.updated += 1;
            }
            Some(_) => {
                result.unchanged += 1;
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use super::*;
    use crate::record::DnsRecord;

    // --- InMemoryProvider para tests de apply_mail_records -------------------

    #[derive(Default)]
    struct InMemoryProvider {
        records: Arc<Mutex<Vec<DnsRecord>>>,
        next_id: Arc<Mutex<u64>>,
    }

    impl InMemoryProvider {
        fn all_records(&self) -> Vec<DnsRecord> {
            self.records.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl DnsProvider for InMemoryProvider {
        fn name(&self) -> &'static str {
            "memory"
        }

        async fn zone_id(&self, _domain: &str) -> Result<String, AgDomainsError> {
            Ok("zone-1".to_owned())
        }

        async fn list_records(&self, _zone_id: &str) -> Result<Vec<DnsRecord>, AgDomainsError> {
            Ok(self.records.lock().unwrap().clone())
        }

        async fn create_record(
            &self,
            zone_id: &str,
            spec: &DnsRecordSpec,
        ) -> Result<DnsRecord, AgDomainsError> {
            let mut id_guard = self.next_id.lock().unwrap();
            *id_guard += 1;
            let record = DnsRecord {
                id: id_guard.to_string(),
                zone_id: zone_id.to_owned(),
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
            let rec = records
                .iter_mut()
                .find(|r| r.id == record_id)
                .ok_or_else(|| AgDomainsError::RecordNotFound(record_id.to_owned()))?;
            rec.content = spec.content.clone();
            rec.ttl = spec.ttl;
            Ok(rec.clone())
        }

        async fn delete_record(
            &self,
            _zone_id: &str,
            record_id: &str,
        ) -> Result<(), AgDomainsError> {
            let mut records = self.records.lock().unwrap();
            records.retain(|r| r.id != record_id);
            Ok(())
        }
    }

    fn base_req() -> MailDnsRequirements {
        MailDnsRequirements {
            domain: "ejemplo.com".to_owned(),
            dkim_selectors: vec![DkimSelector {
                name: "s1".to_owned(),
                public_key_record: "v=DKIM1; k=rsa; p=MIIB...".to_owned(),
            }],
            spf_includes: vec!["sendgrid.net".to_owned()],
            spf_ips: vec!["192.0.2.1".to_owned()],
            dmarc_policy: DmarcPolicy::Quarantine,
            dmarc_rua: Some("admin@ejemplo.com".to_owned()),
        }
    }

    #[test]
    fn generates_three_records() {
        let records = generate_mail_records(&base_req());
        // SPF + 1 DKIM + DMARC
        assert_eq!(records.len(), 3);
    }

    #[test]
    fn spf_content() {
        let records = generate_mail_records(&base_req());
        let spf = records.iter().find(|r| r.name == "ejemplo.com").unwrap();
        assert!(
            spf.content.starts_with("v=spf1"),
            "SPF debe empezar con v=spf1"
        );
        assert!(spf.content.contains("include:sendgrid.net"));
        assert!(spf.content.contains("ip4:192.0.2.1"));
    }

    #[test]
    fn dkim_name_format() {
        let records = generate_mail_records(&base_req());
        let dkim = records
            .iter()
            .find(|r| r.name == "s1._domainkey.ejemplo.com")
            .unwrap();
        assert_eq!(dkim.record_type, RecordType::Txt);
    }

    #[test]
    fn dmarc_policy_and_rua() {
        let records = generate_mail_records(&base_req());
        let dmarc = records
            .iter()
            .find(|r| r.name == "_dmarc.ejemplo.com")
            .unwrap();
        assert!(dmarc.content.contains("p=quarantine"));
        assert!(dmarc.content.contains("rua=mailto:admin@ejemplo.com"));
    }

    #[test]
    fn dmarc_reject_no_rua() {
        let req = MailDnsRequirements {
            dmarc_policy: DmarcPolicy::Reject,
            dmarc_rua: None,
            ..base_req()
        };
        let records = generate_mail_records(&req);
        let dmarc = records
            .iter()
            .find(|r| r.name == "_dmarc.ejemplo.com")
            .unwrap();
        assert!(dmarc.content.contains("p=reject"));
        assert!(!dmarc.content.contains("rua="));
    }

    #[test]
    fn ipv6_spf_prefix() {
        let req = MailDnsRequirements {
            spf_ips: vec!["2001:db8::1".to_owned()],
            ..base_req()
        };
        let records = generate_mail_records(&req);
        let spf = records.iter().find(|r| r.name == "ejemplo.com").unwrap();
        assert!(spf.content.contains("ip6:2001:db8::1"));
    }

    // --- Tests de apply_mail_records -----------------------------------------

    #[tokio::test]
    async fn apply_creates_all_records_on_empty_zone() {
        let provider = InMemoryProvider::default();
        let result = apply_mail_records(&base_req(), &provider, "zone-1")
            .await
            .unwrap();
        assert_eq!(result.created, 3); // SPF + DKIM + DMARC
        assert_eq!(result.updated, 0);
        assert_eq!(result.unchanged, 0);
        assert_eq!(provider.all_records().len(), 3);
    }

    #[tokio::test]
    async fn apply_is_idempotent_when_content_matches() {
        let provider = InMemoryProvider::default();
        // Primera pasada: crea todo
        apply_mail_records(&base_req(), &provider, "zone-1")
            .await
            .unwrap();
        // Segunda pasada: nada debe cambiar
        let result = apply_mail_records(&base_req(), &provider, "zone-1")
            .await
            .unwrap();
        assert_eq!(result.created, 0);
        assert_eq!(result.updated, 0);
        assert_eq!(result.unchanged, 3);
    }

    #[tokio::test]
    async fn apply_updates_record_when_content_changes() {
        let provider = InMemoryProvider::default();
        // Crea los registros con la configuracion base.
        apply_mail_records(&base_req(), &provider, "zone-1")
            .await
            .unwrap();

        // Cambia la politica DMARC — el registro _dmarc debe actualizarse.
        let req_v2 = MailDnsRequirements {
            dmarc_policy: DmarcPolicy::Reject,
            ..base_req()
        };
        let result = apply_mail_records(&req_v2, &provider, "zone-1")
            .await
            .unwrap();
        assert_eq!(result.updated, 1); // solo DMARC
        assert_eq!(result.unchanged, 2); // SPF y DKIM sin cambio

        let dmarc = provider
            .all_records()
            .into_iter()
            .find(|r| r.name == "_dmarc.ejemplo.com")
            .unwrap();
        assert!(dmarc.content.contains("p=reject"));
    }

    #[tokio::test]
    async fn apply_adds_new_dkim_selector() {
        let provider = InMemoryProvider::default();
        apply_mail_records(&base_req(), &provider, "zone-1")
            .await
            .unwrap();

        // Añade un segundo selector DKIM.
        let req_v2 = MailDnsRequirements {
            dkim_selectors: vec![
                DkimSelector {
                    name: "s1".to_owned(),
                    public_key_record: "v=DKIM1; k=rsa; p=MIIB...".to_owned(),
                },
                DkimSelector {
                    name: "s2".to_owned(),
                    public_key_record: "v=DKIM1; k=rsa; p=MIIC...".to_owned(),
                },
            ],
            ..base_req()
        };
        let result = apply_mail_records(&req_v2, &provider, "zone-1")
            .await
            .unwrap();
        assert_eq!(result.created, 1); // s2 es nuevo
        assert_eq!(result.unchanged, 3); // SPF, s1, DMARC sin cambio
        assert_eq!(provider.all_records().len(), 4);
    }
}
