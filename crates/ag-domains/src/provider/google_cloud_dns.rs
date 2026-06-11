//! Google Cloud DNS adapter for `DnsProvider` (feature `google-cloud-dns`).
//!
//! Authenticates with a service account: a short-lived RS256 JWT is exchanged at
//! the OAuth2 token endpoint for an access token, then the Cloud DNS JSON API is
//! called with a bearer token. Like Route 53, Cloud DNS has no per-record id
//! (a record is a resource record set keyed by `(name, type)`), so this adapter
//! synthesizes a stable `TYPE:name` id and mutates through the `changes` API
//! (`additions`/`deletions`), reading the live set back for update/delete.
//!
//! # Configuration
//!
//! ```rust,ignore
//! use ag_domains::provider::google_cloud_dns::GoogleCloudDnsProvider;
//!
//! let provider = GoogleCloudDnsProvider::from_service_account_json(json, None)?;
//! let zone = provider.zone_id("example.com").await?;
//! ```

use async_trait::async_trait;
use chrono::Utc;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::{
    error::AgDomainsError,
    provider::DnsProvider,
    record::{DnsRecord, DnsRecordSpec, RecordType},
};

const DNS_BASE: &str = "https://dns.googleapis.com/dns/v1";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const SCOPE: &str = "https://www.googleapis.com/auth/ndev.clouddns.readwrite";
const PROVIDER: &str = "google-cloud-dns";

/// Adapter for the Google Cloud DNS JSON API.
pub struct GoogleCloudDnsProvider {
    client: reqwest::Client,
    service_account_email: String,
    signing_key: EncodingKey,
    project: String,
    base_url: String,
    token_url: String,
}

/// Minimal view of a service account key file.
#[derive(Deserialize)]
struct ServiceAccountKey {
    client_email: String,
    private_key: String,
    #[serde(default)]
    project_id: Option<String>,
}

impl GoogleCloudDnsProvider {
    /// Builds the adapter from a service account JSON key. `project` overrides the
    /// key's `project_id` when given.
    pub fn from_service_account_json(
        json: &str,
        project: Option<&str>,
    ) -> Result<Self, AgDomainsError> {
        let key: ServiceAccountKey = serde_json::from_str(json)
            .map_err(|e| provider_err(format!("service account: {e}")))?;
        let project = project
            .map(|p| p.to_owned())
            .or(key.project_id)
            .ok_or_else(|| provider_err("missing project id".to_owned()))?;
        Self::with_endpoints(
            key.client_email,
            key.private_key.as_bytes(),
            project,
            DNS_BASE,
            TOKEN_URL,
        )
    }

    /// Builds the adapter with explicit endpoints (custom or wiremock in tests).
    pub fn with_endpoints(
        service_account_email: impl Into<String>,
        private_key_pem: &[u8],
        project: impl Into<String>,
        base_url: impl Into<String>,
        token_url: impl Into<String>,
    ) -> Result<Self, AgDomainsError> {
        let signing_key = EncodingKey::from_rsa_pem(private_key_pem)
            .map_err(|e| provider_err(format!("private key: {e}")))?;
        let client = reqwest::Client::builder()
            .user_agent(concat!("ag-domains/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest client builds with default TLS");
        Ok(Self {
            client,
            service_account_email: service_account_email.into(),
            signing_key,
            project: project.into(),
            base_url: base_url.into(),
            token_url: token_url.into(),
        })
    }

    /// Mints a service-account JWT and exchanges it for an OAuth2 access token.
    async fn access_token(&self) -> Result<String, AgDomainsError> {
        let now = Utc::now().timestamp();
        let claims = JwtClaims {
            iss: &self.service_account_email,
            scope: SCOPE,
            aud: &self.token_url,
            iat: now,
            exp: now + 3600,
        };
        let jwt = jsonwebtoken::encode(&Header::new(Algorithm::RS256), &claims, &self.signing_key)
            .map_err(|e| provider_err(format!("jwt: {e}")))?;
        let resp = self
            .client
            .post(&self.token_url)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await
            .map_err(|e| provider_err(e.to_string()))?;
        let status = resp.status();
        let body = resp.text().await.map_err(|e| provider_err(e.to_string()))?;
        if !status.is_success() {
            return Err(provider_err(format!(
                "token endpoint HTTP {status}: {body}"
            )));
        }
        let token: TokenResponse =
            serde_json::from_str(&body).map_err(|e| provider_err(e.to_string()))?;
        Ok(token.access_token)
    }

    fn zones_url(&self) -> String {
        format!("{}/projects/{}/managedZones", self.base_url, self.project)
    }

    async fn get_json(&self, url: &str) -> Result<String, AgDomainsError> {
        let token = self.access_token().await?;
        let resp = self
            .client
            .get(url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| provider_err(e.to_string()))?;
        let status = resp.status();
        let body = resp.text().await.map_err(|e| provider_err(e.to_string()))?;
        if !status.is_success() {
            return Err(provider_err(format!("HTTP {status}: {body}")));
        }
        Ok(body)
    }

    /// Raw resource record sets of a zone (full `rrdatas`, used by update/delete).
    async fn fetch_rrsets(&self, zone_id: &str) -> Result<Vec<Rrset>, AgDomainsError> {
        let url = format!(
            "{}/projects/{}/managedZones/{zone_id}/rrsets",
            self.base_url, self.project
        );
        let body = self.get_json(&url).await?;
        let parsed: RrsetsResponse =
            serde_json::from_str(&body).map_err(|e| provider_err(e.to_string()))?;
        Ok(parsed.rrsets)
    }

    /// Applies a `changes` mutation (additions/deletions) to a zone.
    async fn apply_change(
        &self,
        zone_id: &str,
        change: &ChangesBody,
    ) -> Result<(), AgDomainsError> {
        let url = format!(
            "{}/projects/{}/managedZones/{zone_id}/changes",
            self.base_url, self.project
        );
        let token = self.access_token().await?;
        let resp = self
            .client
            .post(&url)
            .bearer_auth(token)
            .json(change)
            .send()
            .await
            .map_err(|e| provider_err(e.to_string()))?;
        let status = resp.status();
        let body = resp.text().await.map_err(|e| provider_err(e.to_string()))?;
        if !status.is_success() {
            return Err(provider_err(format!("HTTP {status}: {body}")));
        }
        Ok(())
    }
}

fn provider_err(message: String) -> AgDomainsError {
    AgDomainsError::Provider {
        provider: PROVIDER,
        message,
    }
}

// ---- API types --------------------------------------------------------------

#[derive(Serialize)]
struct JwtClaims<'a> {
    iss: &'a str,
    scope: &'a str,
    aud: &'a str,
    iat: i64,
    exp: i64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize, Default)]
struct ManagedZonesResponse {
    #[serde(rename = "managedZones", default)]
    managed_zones: Vec<ManagedZone>,
}

#[derive(Deserialize)]
struct ManagedZone {
    name: String,
    #[serde(rename = "dnsName")]
    dns_name: String,
}

#[derive(Deserialize, Default)]
struct RrsetsResponse {
    #[serde(default)]
    rrsets: Vec<Rrset>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Rrset {
    name: String,
    #[serde(rename = "type")]
    record_type: String,
    ttl: u32,
    #[serde(default)]
    rrdatas: Vec<String>,
}

#[derive(Serialize)]
struct ChangesBody {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    additions: Vec<Rrset>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    deletions: Vec<Rrset>,
}

fn parse_record_type(t: &str) -> Option<RecordType> {
    match t {
        "A" => Some(RecordType::A),
        "AAAA" => Some(RecordType::Aaaa),
        "CNAME" => Some(RecordType::Cname),
        "TXT" => Some(RecordType::Txt),
        "MX" => Some(RecordType::Mx),
        _ => None,
    }
}

fn normalize_name(name: &str) -> String {
    name.trim_end_matches('.').to_ascii_lowercase()
}

/// FQDN form Cloud DNS expects (trailing dot).
fn fqdn(name: &str) -> String {
    format!("{}.", normalize_name(name))
}

fn record_id(record_type: &RecordType, name: &str) -> String {
    format!("{}:{}", record_type, normalize_name(name))
}

fn unquote_txt(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .unwrap_or(value)
        .to_owned()
}

/// Builds the `Rrset` Cloud DNS expects from a spec (TXT values are quoted).
fn rrset_from_spec(spec: &DnsRecordSpec) -> Rrset {
    let value = if spec.record_type == RecordType::Txt && !spec.content.starts_with('"') {
        format!("\"{}\"", spec.content)
    } else {
        spec.content.clone()
    };
    Rrset {
        name: fqdn(&spec.name),
        record_type: spec.record_type.to_string(),
        ttl: spec.ttl,
        rrdatas: vec![value],
    }
}

// ---- DnsProvider impl -------------------------------------------------------

#[async_trait]
impl DnsProvider for GoogleCloudDnsProvider {
    fn name(&self) -> &'static str {
        PROVIDER
    }

    async fn zone_id(&self, domain: &str) -> Result<String, AgDomainsError> {
        let body = self.get_json(&self.zones_url()).await?;
        let parsed: ManagedZonesResponse =
            serde_json::from_str(&body).map_err(|e| provider_err(e.to_string()))?;
        let target = normalize_name(domain);
        parsed
            .managed_zones
            .into_iter()
            .find(|z| normalize_name(&z.dns_name) == target)
            .map(|z| z.name)
            .ok_or_else(|| AgDomainsError::ZoneNotFound(domain.to_owned()))
    }

    async fn list_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>, AgDomainsError> {
        let mut out = Vec::new();
        for set in self.fetch_rrsets(zone_id).await? {
            let Some(record_type) = parse_record_type(&set.record_type) else {
                continue;
            };
            let content = match set.rrdatas.first() {
                Some(v) if record_type == RecordType::Txt => unquote_txt(v),
                Some(v) => v.clone(),
                None => continue,
            };
            out.push(DnsRecord {
                id: record_id(&record_type, &set.name),
                zone_id: zone_id.to_owned(),
                name: normalize_name(&set.name),
                record_type,
                content,
                ttl: set.ttl,
                proxied: false,
            });
        }
        Ok(out)
    }

    async fn create_record(
        &self,
        zone_id: &str,
        spec: &DnsRecordSpec,
    ) -> Result<DnsRecord, AgDomainsError> {
        self.apply_change(
            zone_id,
            &ChangesBody {
                additions: vec![rrset_from_spec(spec)],
                deletions: vec![],
            },
        )
        .await?;
        Ok(DnsRecord {
            id: record_id(&spec.record_type, &spec.name),
            zone_id: zone_id.to_owned(),
            name: normalize_name(&spec.name),
            record_type: spec.record_type.clone(),
            content: spec.content.clone(),
            ttl: spec.ttl,
            proxied: false,
        })
    }

    async fn update_record(
        &self,
        zone_id: &str,
        record_id_arg: &str,
        spec: &DnsRecordSpec,
    ) -> Result<DnsRecord, AgDomainsError> {
        // Cloud DNS has no in-place update: delete the old set, add the new one.
        let old = self.find_rrset(zone_id, record_id_arg).await?;
        self.apply_change(
            zone_id,
            &ChangesBody {
                additions: vec![rrset_from_spec(spec)],
                deletions: vec![old],
            },
        )
        .await?;
        Ok(DnsRecord {
            id: record_id(&spec.record_type, &spec.name),
            zone_id: zone_id.to_owned(),
            name: normalize_name(&spec.name),
            record_type: spec.record_type.clone(),
            content: spec.content.clone(),
            ttl: spec.ttl,
            proxied: false,
        })
    }

    async fn delete_record(
        &self,
        zone_id: &str,
        record_id_arg: &str,
    ) -> Result<(), AgDomainsError> {
        let old = self.find_rrset(zone_id, record_id_arg).await?;
        self.apply_change(
            zone_id,
            &ChangesBody {
                additions: vec![],
                deletions: vec![old],
            },
        )
        .await
    }
}

impl GoogleCloudDnsProvider {
    /// Finds the raw resource record set for a synthetic `TYPE:name` id.
    async fn find_rrset(&self, zone_id: &str, record_id: &str) -> Result<Rrset, AgDomainsError> {
        let (rtype_str, name) = record_id
            .split_once(':')
            .ok_or_else(|| provider_err(format!("invalid record id: {record_id}")))?;
        let target = normalize_name(name);
        self.fetch_rrsets(zone_id)
            .await?
            .into_iter()
            .find(|s| s.record_type == rtype_str && normalize_name(&s.name) == target)
            .ok_or_else(|| AgDomainsError::RecordNotFound(record_id.to_owned()))
    }
}

// ---- Tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey};
    use rsa::RsaPrivateKey;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::*;
    use crate::provider::contract;

    /// Generates a throwaway RSA key PEM so tests never embed a real key.
    fn test_key_pem() -> String {
        let mut rng = rand::thread_rng();
        let key = RsaPrivateKey::new(&mut rng, 2048).expect("rsa keygen");
        key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .expect("pkcs8 pem")
            .to_string()
    }

    async fn provider_for(mock: &MockServer) -> GoogleCloudDnsProvider {
        GoogleCloudDnsProvider::with_endpoints(
            "svc@test.iam.gserviceaccount.com",
            test_key_pem().as_bytes(),
            "test-project",
            mock.uri(),
            format!("{}/token", mock.uri()),
        )
        .unwrap()
    }

    async fn mount_token(mock: &MockServer) {
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "test-access-token",
                "token_type": "Bearer",
                "expires_in": 3600
            })))
            .mount(mock)
            .await;
    }

    #[test]
    fn jwt_signs_and_verifies_rs256() {
        // Verifies the JWT crypto path end to end with a generated key pair.
        let pem = test_key_pem();
        let key = EncodingKey::from_rsa_pem(pem.as_bytes()).unwrap();
        let claims = JwtClaims {
            iss: "svc@test",
            scope: SCOPE,
            aud: "https://oauth2.googleapis.com/token",
            iat: 1_700_000_000,
            exp: 1_700_003_600,
        };
        let jwt = jsonwebtoken::encode(&Header::new(Algorithm::RS256), &claims, &key).unwrap();
        let public = RsaPrivateKey::from_pkcs8_pem(&pem).unwrap().to_public_key();
        let public_pem = public
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap();
        let mut validation = jsonwebtoken::Validation::new(Algorithm::RS256);
        validation.set_audience(&["https://oauth2.googleapis.com/token"]);
        // This test asserts the RS256 signature verifies; the fixed claims are in
        // the past, so do not enforce expiry here.
        validation.validate_exp = false;
        let decoded = jsonwebtoken::decode::<serde_json::Value>(
            &jwt,
            &jsonwebtoken::DecodingKey::from_rsa_pem(public_pem.as_bytes()).unwrap(),
            &validation,
        )
        .unwrap();
        assert_eq!(decoded.claims["iss"], "svc@test");
    }

    #[tokio::test]
    async fn zone_id_ok() {
        let mock = MockServer::start().await;
        mount_token(&mock).await;
        Mock::given(method("GET"))
            .and(path("/projects/test-project/managedZones"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "managedZones": [{ "name": "zone-1", "dnsName": "ejemplo.com." }]
            })))
            .mount(&mock)
            .await;
        let provider = provider_for(&mock).await;
        assert_eq!(provider.zone_id("ejemplo.com").await.unwrap(), "zone-1");
    }

    #[tokio::test]
    async fn zone_id_not_found() {
        let mock = MockServer::start().await;
        mount_token(&mock).await;
        Mock::given(method("GET"))
            .and(path("/projects/test-project/managedZones"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "managedZones": [] })),
            )
            .mount(&mock)
            .await;
        let provider = provider_for(&mock).await;
        assert!(matches!(
            provider.zone_id("nope.com").await.unwrap_err(),
            AgDomainsError::ZoneNotFound(_)
        ));
    }

    async fn full_mock_server() -> (MockServer, GoogleCloudDnsProvider) {
        let mock = MockServer::start().await;
        mount_token(&mock).await;
        Mock::given(method("GET"))
            .and(path("/projects/test-project/managedZones/zone-1/rrsets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "rrsets": [{
                    "name": "_ag-contract-test.ejemplo.com.",
                    "type": "TXT",
                    "ttl": 60,
                    "rrdatas": ["\"ag-domains-contract-test-value\""]
                }]
            })))
            .mount(&mock)
            .await;
        Mock::given(method("POST"))
            .and(path("/projects/test-project/managedZones/zone-1/changes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "kind": "dns#change", "status": "pending"
            })))
            .mount(&mock)
            .await;
        let provider = provider_for(&mock).await;
        (mock, provider)
    }

    #[tokio::test]
    async fn list_records_unquotes_txt() {
        let (_mock, provider) = full_mock_server().await;
        let records = provider.list_records("zone-1").await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].content, "ag-domains-contract-test-value");
        assert_eq!(records[0].id, "TXT:_ag-contract-test.ejemplo.com");
    }

    #[tokio::test]
    async fn contract_create_list_delete() {
        let (_mock, provider) = full_mock_server().await;
        contract::test_create_list_delete(&provider, "zone-1").await;
    }
}
