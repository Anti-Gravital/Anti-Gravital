//! Generacion de registros DNS requeridos para la entrega de correo.
//!
//! `ag-mail` declara sus requisitos a traves de `MailDnsRequirements`
//! y este modulo los materializa como `DnsRecordSpec` para que el caller
//! los aplique via `DnsProvider`. Sin ciclo de dependencia: `ag-mail` y
//! `ag-domains` se conocen solo por estos tipos intermedios.

use crate::record::{DnsRecordSpec, RecordType};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
