//! Emision y renovacion automatica de certificados TLS via ACME.
//!
//! El flujo completo es:
//! 1. Crear o restaurar cuenta ACME en Let's Encrypt.
//! 2. Crear orden para el dominio.
//! 3. Resolver challenge DNS-01 via `DnsProvider`.
//! 4. Esperar validacion con backoff exponencial.
//! 5. Generar CSR con `rcgen` y finalizar la orden.
//! 6. Descargar el chain PEM + clave privada.
//!
//! La tarea de renovacion (`spawn_renewal_task`) relanza el proceso
//! automaticamente cuando el certificado esta a N dias de vencer.

use std::time::Duration;

use instant_acme::{
    Account, AccountCredentials, AuthorizationStatus, ChallengeType, Identifier, LetsEncrypt,
    NewAccount, NewOrder, OrderStatus,
};
use rcgen::{CertificateParams, DistinguishedName, KeyPair};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::{
    acme::challenge::{remove_dns01_challenge, set_dns01_challenge},
    error::AgDomainsError,
    provider::DnsProvider,
};

/// Certificado emitido: chain PEM + clave privada PEM.
#[derive(Debug, Clone)]
pub struct IssuedCert {
    /// Cadena de certificados en formato PEM (leaf + intermedios).
    pub cert_chain_pem: String,
    /// Clave privada ECDSA P-256 en formato PEM.
    pub private_key_pem: String,
}

/// Configuracion para la emision/renovacion de un certificado.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertConfig {
    /// Dominio para el que se emite el certificado.
    pub domain: String,
    /// Zone ID en el proveedor DNS (se puede obtener con `DnsProvider::zone_id`).
    pub zone_id: String,
    /// Email de contacto para la cuenta ACME.
    pub contact_email: String,
    /// Usar staging de Let's Encrypt (para pruebas).
    #[serde(default)]
    pub staging: bool,
}

/// Emite un certificado usando ACME DNS-01 con el `DnsProvider` dado.
///
/// Crea una cuenta ACME nueva en cada llamada. Para reutilizar cuentas,
/// usa `issue_with_credentials`.
pub async fn issue<P: DnsProvider>(
    config: &CertConfig,
    provider: &P,
) -> Result<(IssuedCert, AccountCredentials), AgDomainsError> {
    let server_url = acme_url(config.staging);
    let contact = format!("mailto:{}", config.contact_email);

    let (account, credentials) = Account::create(
        &NewAccount {
            contact: &[contact.as_str()],
            terms_of_service_agreed: true,
            only_return_existing: false,
        },
        server_url,
        None,
    )
    .await
    .map_err(|e| AgDomainsError::Acme(e.to_string()))?;

    let cert = issue_order(&account, config, provider).await?;
    Ok((cert, credentials))
}

/// Emite un certificado reutilizando credenciales ACME previas.
pub async fn issue_with_credentials<P: DnsProvider>(
    config: &CertConfig,
    credentials: AccountCredentials,
    provider: &P,
) -> Result<IssuedCert, AgDomainsError> {
    let account = Account::from_credentials(credentials)
        .await
        .map_err(|e| AgDomainsError::Acme(e.to_string()))?;

    issue_order(&account, config, provider).await
}

/// Lanza una tarea Tokio que renueva el certificado periodicamente.
///
/// Comprueba cada dia si faltan menos de `renew_before_days` dias para el
/// vencimiento. Mientras el parseo del `notAfter` no este implementado, renueva
/// en cada ciclo y usa `renew_before_days` como periodo de sleep (cota conservadora).
///
/// La tarea vive mientras el `JoinHandle` no sea abortado. En caso de
/// error de renovacion, lo registra y reintenta al ciclo siguiente.
pub fn spawn_renewal_task<P>(
    config: CertConfig,
    credentials: AccountCredentials,
    provider: P,
    renew_before_days: u64,
    on_renewed: impl Fn(IssuedCert) + Send + 'static,
) -> tokio::task::JoinHandle<()>
where
    P: DnsProvider + 'static,
{
    // AccountCredentials no implementa Clone; serializamos a JSON una vez y
    // deserializamos en cada iteracion del loop para obtener una copia fresca.
    let credentials_json = serde_json::to_vec(&credentials)
        .expect("AccountCredentials siempre es serializable a JSON");

    // Cuantos segundos dormir entre ciclos de renovacion.
    // TECH-DEBT: cuando se implemente el parseo de `notAfter`, usar `check_interval`
    // (24 h) y comparar la fecha de expiracion con `renew_before_days`.
    // motivo: simplificacion de la primera iteracion
    // impacto: renueva cada `renew_before_days` dias en lugar de exactamente al umbral
    // eliminacion esperada: Etapa 3 de ag-domains
    let sleep_secs = renew_before_days.max(1) * 86_400;

    tokio::spawn(async move {
        loop {
            info!(domain = config.domain, "renovando certificado TLS via ACME");

            let creds: AccountCredentials = serde_json::from_slice(&credentials_json)
                .expect("AccountCredentials previamente serializado es siempre deserializable");

            match issue_with_credentials(&config, creds, &provider).await {
                Ok(cert) => {
                    info!(domain = config.domain, "certificado renovado correctamente");
                    on_renewed(cert);
                }
                Err(e) => {
                    error!(domain = config.domain, error = %e, "fallo al renovar certificado");
                }
            }

            tokio::time::sleep(Duration::from_secs(sleep_secs)).await;
        }
    })
}

// ---- helpers internos -------------------------------------------------------

async fn issue_order<P: DnsProvider>(
    account: &Account,
    config: &CertConfig,
    provider: &P,
) -> Result<IssuedCert, AgDomainsError> {
    let identifier = Identifier::Dns(config.domain.clone());
    let mut order = account
        .new_order(&NewOrder {
            identifiers: &[identifier],
        })
        .await
        .map_err(|e| AgDomainsError::Acme(e.to_string()))?;

    // Recopilar los challenges DNS-01 pendientes.
    let authorizations = order
        .authorizations()
        .await
        .map_err(|e| AgDomainsError::Acme(e.to_string()))?;

    let mut challenge_records: Vec<(
        String, /* domain */
        String, /* url */
        String, /* record_id */
    )> = Vec::new();

    for authz in &authorizations {
        if !matches!(authz.status, AuthorizationStatus::Pending) {
            continue;
        }

        let challenge = authz
            .challenges
            .iter()
            .find(|c| c.r#type == ChallengeType::Dns01)
            .ok_or_else(|| AgDomainsError::Acme("no se encontro challenge DNS-01".to_owned()))?;

        let Identifier::Dns(domain) = &authz.identifier;
        let dns_value = order.key_authorization(challenge).dns_value();

        let record_id = set_dns01_challenge(provider, &config.zone_id, domain, &dns_value).await?;

        challenge_records.push((domain.clone(), challenge.url.clone(), record_id));
    }

    // Notificar al servidor que los challenges estan listos.
    for (_, challenge_url, _) in &challenge_records {
        order
            .set_challenge_ready(challenge_url)
            .await
            .map_err(|e| AgDomainsError::Acme(e.to_string()))?;
    }

    // Esperar con backoff exponencial hasta que la orden este lista.
    let result = wait_for_order_ready(&mut order).await;

    // Limpiar registros TXT independientemente del resultado.
    for (_, _, record_id) in &challenge_records {
        if let Err(e) = remove_dns01_challenge(provider, &config.zone_id, record_id).await {
            warn!(record_id, error = %e, "no se pudo eliminar el registro challenge DNS-01");
        }
    }

    result?; // propagar error si la orden fallo

    // Generar CSR y finalizar.
    let names = challenge_records
        .iter()
        .map(|(d, _, _)| d.clone())
        .collect::<Vec<_>>();

    let mut params =
        CertificateParams::new(names).map_err(|e| AgDomainsError::Acme(e.to_string()))?;
    params.distinguished_name = DistinguishedName::new();
    let private_key = KeyPair::generate().map_err(|e| AgDomainsError::Acme(e.to_string()))?;
    let csr = params
        .serialize_request(&private_key)
        .map_err(|e| AgDomainsError::Acme(e.to_string()))?;

    order
        .finalize(csr.der())
        .await
        .map_err(|e| AgDomainsError::Acme(e.to_string()))?;

    // Descargar el certificado con polling simple.
    let cert_chain_pem = loop {
        match order
            .certificate()
            .await
            .map_err(|e| AgDomainsError::Acme(e.to_string()))?
        {
            Some(chain) => break chain,
            None => tokio::time::sleep(Duration::from_secs(2)).await,
        }
    };

    Ok(IssuedCert {
        cert_chain_pem,
        private_key_pem: private_key.serialize_pem(),
    })
}

async fn wait_for_order_ready(order: &mut instant_acme::Order) -> Result<(), AgDomainsError> {
    let mut delay = Duration::from_millis(500);
    let max_tries = 10u8;

    for attempt in 1..=max_tries {
        tokio::time::sleep(delay).await;
        let state = order
            .refresh()
            .await
            .map_err(|e| AgDomainsError::Acme(e.to_string()))?;

        match state.status {
            OrderStatus::Ready => return Ok(()),
            OrderStatus::Invalid => {
                return Err(AgDomainsError::Acme(
                    "la orden ACME quedo en estado invalido".to_owned(),
                ))
            }
            _ => {
                info!(
                    attempt,
                    delay_ms = delay.as_millis(),
                    "esperando orden ACME..."
                );
                delay = (delay * 2).min(Duration::from_secs(30));
            }
        }
    }

    Err(AgDomainsError::Acme(format!(
        "la orden ACME no llego a estado Ready tras {max_tries} intentos"
    )))
}

fn acme_url(staging: bool) -> &'static str {
    if staging {
        LetsEncrypt::Staging.url()
    } else {
        LetsEncrypt::Production.url()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acme_url_staging() {
        let url = acme_url(true);
        assert!(
            url.contains("staging"),
            "staging URL debe contener 'staging'"
        );
    }

    #[test]
    fn acme_url_production() {
        let url = acme_url(false);
        assert!(
            !url.contains("staging"),
            "production URL no debe contener 'staging'"
        );
    }

    #[test]
    fn cert_config_staging_flag() {
        let cfg = CertConfig {
            domain: "ejemplo.com".to_owned(),
            zone_id: "z123".to_owned(),
            contact_email: "admin@ejemplo.com".to_owned(),
            staging: true,
        };
        assert!(cfg.staging);
        assert_eq!(cfg.domain, "ejemplo.com");
    }

    #[test]
    fn cert_config_staging_default_is_false() {
        let json = r#"{"domain":"a.com","zone_id":"z","contact_email":"e@e.com"}"#;
        let cfg: CertConfig = serde_json::from_str(json).unwrap();
        assert!(!cfg.staging, "staging debe ser false por defecto");
    }

    #[test]
    fn cert_config_roundtrip_json() {
        let cfg = CertConfig {
            domain: "test.com".to_owned(),
            zone_id: "zid".to_owned(),
            contact_email: "x@x.com".to_owned(),
            staging: false,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let cfg2: CertConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.domain, cfg2.domain);
        assert_eq!(cfg.zone_id, cfg2.zone_id);
        assert_eq!(cfg.contact_email, cfg2.contact_email);
        assert_eq!(cfg.staging, cfg2.staging);
    }

    #[test]
    fn issued_cert_fields_accessible() {
        let cert = IssuedCert {
            cert_chain_pem: "-----BEGIN CERTIFICATE-----".to_owned(),
            private_key_pem: "-----BEGIN PRIVATE KEY-----".to_owned(),
        };
        assert!(cert.cert_chain_pem.contains("CERTIFICATE"));
        assert!(cert.private_key_pem.contains("PRIVATE KEY"));
    }
}
