//! Automatic TLS certificate issuance and renewal via ACME.
//!
//! The complete flow is:
//! 1. Create or restore an ACME account on Let's Encrypt.
//! 2. Create an order for the domain.
//! 3. Resolve the DNS-01 challenge via `DnsProvider`.
//! 4. Wait for validation with exponential backoff.
//! 5. Generate a CSR with `rcgen` and finalize the order.
//! 6. Download the PEM chain + private key.
//!
//! The renewal task (`spawn_renewal_task`) restarts the process
//! automatically when the certificate is N days from expiring.

use std::time::Duration;

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use instant_acme::{
    Account, AccountCredentials, AuthorizationStatus, ChallengeType, Identifier, LetsEncrypt,
    NewAccount, NewOrder, OrderStatus,
};
use rcgen::{CertificateParams, DistinguishedName, KeyPair};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::{
    acme::ari,
    acme::challenge::{remove_dns01_challenge, set_dns01_challenge},
    error::AgDomainsError,
    metrics::set_cert_days_until_expiry,
    provider::DnsProvider,
};

/// Issued certificate: PEM chain + PEM private key.
#[derive(Debug, Clone)]
pub struct IssuedCert {
    /// Certificate chain in PEM format (leaf + intermediates).
    pub cert_chain_pem: String,
    /// ECDSA P-256 private key in PEM format.
    pub private_key_pem: String,
}

/// Configuration for issuing/renewing a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertConfig {
    /// Domain for which the certificate is issued.
    pub domain: String,
    /// Zone ID in the DNS provider (can be obtained with `DnsProvider::zone_id`).
    pub zone_id: String,
    /// Contact email for the ACME account.
    pub contact_email: String,
    /// Use Let's Encrypt staging (for testing).
    #[serde(default)]
    pub staging: bool,
}

/// Issues a certificate using ACME DNS-01 with the given `DnsProvider`.
///
/// Creates a new ACME account on each call. To reuse accounts,
/// use `issue_with_credentials`.
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

/// Issues a certificate reusing previous ACME credentials.
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

/// Spawns a Tokio task that renews the certificate before it expires.
///
/// After each successful issuance the task parses `notAfter` from the PEM
/// chain, computes how long to sleep with [`seconds_until_renewal`], and
/// wakes up exactly when the threshold is crossed. On error it retries
/// after `check_interval_secs` (24 h by default).
///
/// The task lives until the `JoinHandle` is aborted.
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
    // No ARI source: schedule purely from `notAfter`.
    spawn_renewal_task_with_ari(
        config,
        credentials,
        provider,
        renew_before_days,
        |_| None,
        on_renewed,
    )
}

/// Like [`spawn_renewal_task`], but ARI-aware (RFC 9773): after each issuance,
/// `renewal_info_for` may return the CA's [`ari::RenewalInfo`] for the new
/// certificate, and the next renewal is scheduled inside that window instead of
/// from `notAfter`. Returning `None` falls back to the `notAfter` schedule.
///
/// `renewal_info_for` is the ARI fetch boundary: the caller performs the HTTP
/// GET of the `RenewalInfo` resource (needing the live CA, hence not exercised
/// in unit tests) and parses it with [`ari::parse_renewal_info`]. The scheduling
/// decision itself is [`ari::next_renewal_sleep`], which is unit-tested.
pub fn spawn_renewal_task_with_ari<P>(
    config: CertConfig,
    credentials: AccountCredentials,
    provider: P,
    renew_before_days: u64,
    renewal_info_for: impl Fn(&IssuedCert) -> Option<ari::RenewalInfo> + Send + 'static,
    on_renewed: impl Fn(IssuedCert) + Send + 'static,
) -> tokio::task::JoinHandle<()>
where
    P: DnsProvider + 'static,
{
    // AccountCredentials does not implement Clone; serialize once and
    // deserialize on each iteration for a fresh copy.
    let credentials_json =
        serde_json::to_vec(&credentials).expect("AccountCredentials is always JSON-serializable");

    const CHECK_INTERVAL_SECS: u64 = 86_400; // 24 h fallback on error

    tokio::spawn(async move {
        let mut not_after: Option<DateTime<Utc>> = None;
        let mut ari_info: Option<ari::RenewalInfo> = None;

        loop {
            // Sleep until the renewal window. Prefer the CA's ARI window; fall
            // back to `notAfter` minus the threshold. Run immediately on first
            // pass (no certificate yet).
            let sleep_secs = match not_after {
                Some(exp) => {
                    ari::next_renewal_sleep(ari_info.as_ref(), exp, renew_before_days, Utc::now())
                }
                None => 0,
            };

            if sleep_secs > 0 {
                info!(
                    domain = config.domain,
                    sleep_secs, "TLS certificate valid; sleeping until renewal window"
                );
                tokio::time::sleep(Duration::from_secs(sleep_secs)).await;
            }

            info!(domain = config.domain, "renewing TLS certificate via ACME");

            let creds: AccountCredentials = serde_json::from_slice(&credentials_json)
                .expect("previously serialized AccountCredentials is always deserializable");

            match issue_with_credentials(&config, creds, &provider).await {
                Ok(cert) => {
                    not_after = parse_not_after(&cert.cert_chain_pem).ok();
                    // Ask the caller for the CA's ARI window for this new cert;
                    // `None` keeps the `notAfter` schedule.
                    ari_info = renewal_info_for(&cert);
                    if let Some(exp) = not_after {
                        let days = (exp - Utc::now()).num_days().max(0);
                        set_cert_days_until_expiry(&config.domain, days);
                        info!(
                            domain = config.domain,
                            days_until_expiry = days,
                            "TLS certificate renewed"
                        );
                    } else {
                        warn!(
                            domain = config.domain,
                            "renewed but could not parse notAfter; will retry after check interval"
                        );
                        not_after = None;
                    }
                    on_renewed(cert);
                }
                Err(e) => {
                    error!(domain = config.domain, error = %e, "TLS certificate renewal failed; retrying after check interval");
                    tokio::time::sleep(Duration::from_secs(CHECK_INTERVAL_SECS)).await;
                }
            }
        }
    })
}

// ---- internal helpers -------------------------------------------------------

/// Parses the `notAfter` field from the first PEM certificate in `cert_chain_pem`.
pub fn parse_not_after(cert_chain_pem: &str) -> Result<DateTime<Utc>, AgDomainsError> {
    let (_, pem) = x509_parser::pem::parse_x509_pem(cert_chain_pem.as_bytes())
        .map_err(|e| AgDomainsError::Acme(format!("PEM parse error: {e}")))?;
    let cert = pem
        .parse_x509()
        .map_err(|e| AgDomainsError::Acme(format!("X509 parse error: {e}")))?;
    let ts = cert.validity().not_after.timestamp();
    DateTime::from_timestamp(ts, 0)
        .ok_or_else(|| AgDomainsError::Acme("notAfter timestamp out of range".to_owned()))
}

/// Returns how many seconds to sleep before the next renewal attempt.
///
/// If the time remaining before `not_after` exceeds `renew_before_days`,
/// returns the gap minus the threshold. Returns 0 when already inside the
/// renewal window or when the certificate is expired.
pub fn seconds_until_renewal(
    not_after: DateTime<Utc>,
    renew_before_days: u64,
    now: DateTime<Utc>,
) -> u64 {
    let threshold = ChronoDuration::days(renew_before_days as i64);
    let remaining = not_after.signed_duration_since(now);
    if remaining <= threshold {
        return 0;
    }
    let sleep = remaining - threshold;
    sleep.num_seconds().max(0) as u64
}

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

    // Collect the pending DNS-01 challenges.
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

    // Notify the server that the challenges are ready.
    for (_, challenge_url, _) in &challenge_records {
        order
            .set_challenge_ready(challenge_url)
            .await
            .map_err(|e| AgDomainsError::Acme(e.to_string()))?;
    }

    // Wait with exponential backoff until the order is ready.
    let result = wait_for_order_ready(&mut order).await;

    // Clean up TXT records regardless of the result.
    for (_, _, record_id) in &challenge_records {
        if let Err(e) = remove_dns01_challenge(provider, &config.zone_id, record_id).await {
            warn!(record_id, error = %e, "no se pudo eliminar el registro challenge DNS-01");
        }
    }

    result?; // propagate error if the order failed

    // Generate CSR and finalize.
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

    // Download the certificate with simple polling.
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
    fn parse_not_after_reads_certificate_validity() {
        use chrono::Datelike;
        use rcgen::{CertificateParams, KeyPair};

        let mut params = CertificateParams::new(vec!["example.com".to_string()]).unwrap();
        let target_year = Utc::now().year() + 1;
        params.not_after = rcgen::date_time_ymd(target_year, 1, 1);
        let key = KeyPair::generate().unwrap();
        let cert = params.self_signed(&key).unwrap();
        let pem = cert.pem();

        let not_after = parse_not_after(&pem).expect("must parse notAfter");
        assert_eq!(not_after.year(), target_year, "parsed year must match cert");
    }

    #[test]
    fn parse_not_after_rejects_garbage() {
        assert!(parse_not_after("not a pem").is_err());
    }

    #[test]
    fn seconds_until_renewal_respects_threshold() {
        let now = Utc::now();
        let not_after = now + ChronoDuration::days(30);
        let renew_before_days = 10u64;

        // 30 days left, threshold is 10 days → should sleep 20 days
        let secs = seconds_until_renewal(not_after, renew_before_days, now);
        let expected = 20u64 * 86_400;
        assert_eq!(secs, expected);
    }

    #[test]
    fn seconds_until_renewal_past_threshold_returns_zero() {
        let now = Utc::now();
        // cert expires in 5 days, threshold is 10 → already past threshold
        let not_after = now + ChronoDuration::days(5);
        let secs = seconds_until_renewal(not_after, 10, now);
        assert_eq!(secs, 0);
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
