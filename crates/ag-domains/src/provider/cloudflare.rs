//! Cloudflare adapter for `DnsProvider`.
//!
//! Requires an API token with `Zone:Read` and `DNS:Edit` permissions
//! over the zones to be managed.
//!
//! # Configuration
//!
//! ```rust,ignore
//! use ag_domains::provider::cloudflare::CloudflareProvider;
//!
//! let provider = CloudflareProvider::new("cf-api-token-aqui");
//! let zone_id = provider.zone_id("ejemplo.com").await?;
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    error::AgDomainsError,
    provider::DnsProvider,
    record::{DnsRecord, DnsRecordSpec, RecordType},
};

const CF_BASE: &str = "https://api.cloudflare.com/client/v4";

/// Adapter for the Cloudflare REST API.
pub struct CloudflareProvider {
    client: reqwest::Client,
    token: String,
    /// Configurable base URL to facilitate tests with wiremock.
    base_url: String,
}

impl CloudflareProvider {
    /// Creates an adapter pointing at the Cloudflare production API.
    pub fn new(token: impl Into<String>) -> Self {
        Self::with_base_url(token, CF_BASE)
    }

    /// Creates an adapter pointing at a custom base URL.
    ///
    /// Useful in tests with wiremock: `CloudflareProvider::with_base_url(token, mock.uri())`.
    pub fn with_base_url(token: impl Into<String>, base_url: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(concat!("ag-domains/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("construccion del cliente reqwest siempre tiene exito con defaults TLS");
        Self {
            client,
            token: token.into(),
            base_url: base_url.into(),
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

// ---- Cloudflare API types --------------------------------------------------

#[derive(Deserialize)]
struct CfResponse<T> {
    result: Option<T>,
    success: bool,
    errors: Vec<CfError>,
}

#[derive(Deserialize)]
struct CfError {
    message: String,
}

#[derive(Deserialize)]
struct CfZone {
    id: String,
}

#[derive(Serialize, Deserialize)]
struct CfRecord {
    id: String,
    zone_id: String,
    name: String,
    #[serde(rename = "type")]
    record_type: RecordType,
    content: String,
    ttl: u32,
    #[serde(default)]
    proxied: bool,
}

#[derive(Serialize)]
struct CfRecordBody<'a> {
    name: &'a str,
    #[serde(rename = "type")]
    record_type: &'a RecordType,
    content: &'a str,
    ttl: u32,
    proxied: bool,
}

impl CfRecord {
    fn into_domain(self) -> DnsRecord {
        DnsRecord {
            id: self.id,
            zone_id: self.zone_id,
            name: self.name,
            record_type: self.record_type,
            content: self.content,
            ttl: self.ttl,
            proxied: self.proxied,
        }
    }
}

impl<'a> From<&'a DnsRecordSpec> for CfRecordBody<'a> {
    fn from(s: &'a DnsRecordSpec) -> Self {
        CfRecordBody {
            name: &s.name,
            record_type: &s.record_type,
            content: &s.content,
            ttl: s.ttl,
            proxied: s.proxied,
        }
    }
}

fn cf_result<T>(resp: CfResponse<T>, op: &str) -> Result<T, AgDomainsError> {
    if !resp.success {
        let msg = resp
            .errors
            .iter()
            .map(|e| e.message.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(AgDomainsError::Provider {
            provider: "cloudflare",
            message: format!("{op}: {msg}"),
        });
    }
    resp.result.ok_or_else(|| AgDomainsError::Provider {
        provider: "cloudflare",
        message: format!("{op}: respuesta exitosa sin campo `result`"),
    })
}

// ---- DnsProvider impl -------------------------------------------------------

#[async_trait]
impl DnsProvider for CloudflareProvider {
    fn name(&self) -> &'static str {
        "cloudflare"
    }

    async fn zone_id(&self, domain: &str) -> Result<String, AgDomainsError> {
        let url = self.url(&format!("/zones?name={domain}"));
        let resp: CfResponse<Vec<CfZone>> = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| AgDomainsError::Provider {
                provider: "cloudflare",
                message: e.to_string(),
            })?
            .json()
            .await
            .map_err(|e| AgDomainsError::Provider {
                provider: "cloudflare",
                message: e.to_string(),
            })?;

        let zones = cf_result(resp, "zone_id")?;
        zones
            .into_iter()
            .next()
            .map(|z| z.id)
            .ok_or_else(|| AgDomainsError::ZoneNotFound(domain.to_owned()))
    }

    async fn list_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>, AgDomainsError> {
        let url = self.url(&format!("/zones/{zone_id}/dns_records"));
        let resp: CfResponse<Vec<CfRecord>> = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| AgDomainsError::Provider {
                provider: "cloudflare",
                message: e.to_string(),
            })?
            .json()
            .await
            .map_err(|e| AgDomainsError::Provider {
                provider: "cloudflare",
                message: e.to_string(),
            })?;

        Ok(cf_result(resp, "list_records")?
            .into_iter()
            .map(CfRecord::into_domain)
            .collect())
    }

    async fn create_record(
        &self,
        zone_id: &str,
        spec: &DnsRecordSpec,
    ) -> Result<DnsRecord, AgDomainsError> {
        let url = self.url(&format!("/zones/{zone_id}/dns_records"));
        let body = CfRecordBody::from(spec);
        let resp: CfResponse<CfRecord> = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await
            .map_err(|e| AgDomainsError::Provider {
                provider: "cloudflare",
                message: e.to_string(),
            })?
            .json()
            .await
            .map_err(|e| AgDomainsError::Provider {
                provider: "cloudflare",
                message: e.to_string(),
            })?;

        cf_result(resp, "create_record").map(CfRecord::into_domain)
    }

    async fn update_record(
        &self,
        zone_id: &str,
        record_id: &str,
        spec: &DnsRecordSpec,
    ) -> Result<DnsRecord, AgDomainsError> {
        let url = self.url(&format!("/zones/{zone_id}/dns_records/{record_id}"));
        let body = CfRecordBody::from(spec);
        let resp: CfResponse<CfRecord> = self
            .client
            .put(&url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await
            .map_err(|e| AgDomainsError::Provider {
                provider: "cloudflare",
                message: e.to_string(),
            })?
            .json()
            .await
            .map_err(|e| AgDomainsError::Provider {
                provider: "cloudflare",
                message: e.to_string(),
            })?;

        cf_result(resp, "update_record").map(CfRecord::into_domain)
    }

    async fn delete_record(&self, zone_id: &str, record_id: &str) -> Result<(), AgDomainsError> {
        let url = self.url(&format!("/zones/{zone_id}/dns_records/{record_id}"));
        // The API returns {"id":"..."} — we do not need the field, only success.
        let resp: CfResponse<serde_json::Value> = self
            .client
            .delete(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| AgDomainsError::Provider {
                provider: "cloudflare",
                message: e.to_string(),
            })?
            .json()
            .await
            .map_err(|e| AgDomainsError::Provider {
                provider: "cloudflare",
                message: e.to_string(),
            })?;

        cf_result(resp, "delete_record").map(|_| ())
    }
}

// ---- Tests with wiremock ----------------------------------------------------

#[cfg(test)]
mod tests {
    use wiremock::{
        matchers::{header, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    use super::*;
    use crate::{provider::contract, record::RecordType};

    fn mock_cf_record(id: &str, zone_id: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "zone_id": zone_id,
            "name": "_ag-contract-test.ejemplo.com",
            "type": "TXT",
            "content": "ag-domains-contract-test-value",
            "ttl": 60,
            "proxied": false
        })
    }

    fn cf_ok<T: serde::Serialize>(result: T) -> serde_json::Value {
        serde_json::json!({ "success": true, "errors": [], "result": result })
    }

    fn cf_err(message: &str) -> serde_json::Value {
        serde_json::json!({
            "success": false,
            "errors": [{ "code": 9999, "message": message }],
            "result": null
        })
    }

    // ---- zone_id -----------------------------------------------------------

    #[tokio::test]
    async fn zone_id_ok() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/zones"))
            .and(query_param("name", "ejemplo.com"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(cf_ok(serde_json::json!([{ "id": "zone-abc123" }]))),
            )
            .mount(&mock)
            .await;

        let provider = CloudflareProvider::with_base_url("test-token", mock.uri());
        let id = provider.zone_id("ejemplo.com").await.unwrap();
        assert_eq!(id, "zone-abc123");
    }

    #[tokio::test]
    async fn zone_id_not_found() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/zones"))
            .respond_with(ResponseTemplate::new(200).set_body_json(cf_ok(serde_json::json!([]))))
            .mount(&mock)
            .await;

        let provider = CloudflareProvider::with_base_url("t", mock.uri());
        let err = provider.zone_id("noexiste.com").await.unwrap_err();
        assert!(matches!(err, AgDomainsError::ZoneNotFound(_)));
    }

    #[tokio::test]
    async fn zone_id_api_error() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/zones"))
            .respond_with(ResponseTemplate::new(200).set_body_json(cf_err("token invalido")))
            .mount(&mock)
            .await;

        let provider = CloudflareProvider::with_base_url("bad-token", mock.uri());
        let err = provider.zone_id("x.com").await.unwrap_err();
        assert!(matches!(err, AgDomainsError::Provider { .. }));
    }

    // ---- list_records ------------------------------------------------------

    #[tokio::test]
    async fn list_records_ok() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/zones/zone1/dns_records"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(cf_ok(serde_json::json!([
                    mock_cf_record("rec1", "zone1"),
                    mock_cf_record("rec2", "zone1"),
                ]))),
            )
            .mount(&mock)
            .await;

        let provider = CloudflareProvider::with_base_url("t", mock.uri());
        let records = provider.list_records("zone1").await.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, "rec1");
        assert_eq!(records[1].id, "rec2");
    }

    // ---- create_record -----------------------------------------------------

    #[tokio::test]
    async fn create_record_ok() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/zones/zone1/dns_records"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(cf_ok(mock_cf_record("new-rec", "zone1"))),
            )
            .mount(&mock)
            .await;

        let provider = CloudflareProvider::with_base_url("t", mock.uri());
        let spec = DnsRecordSpec {
            name: "_ag-contract-test.ejemplo.com".to_owned(),
            record_type: RecordType::Txt,
            content: "ag-domains-contract-test-value".to_owned(),
            ttl: 60,
            proxied: false,
        };
        let record = provider.create_record("zone1", &spec).await.unwrap();
        assert_eq!(record.id, "new-rec");
        assert_eq!(record.record_type, RecordType::Txt);
    }

    // ---- update_record -----------------------------------------------------

    #[tokio::test]
    async fn update_record_ok() {
        let mock = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/zones/zone1/dns_records/rec1"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(cf_ok(serde_json::json!({
                    "id": "rec1",
                    "zone_id": "zone1",
                    "name": "_ag.ejemplo.com",
                    "type": "TXT",
                    "content": "actualizado",
                    "ttl": 60,
                    "proxied": false
                }))),
            )
            .mount(&mock)
            .await;

        let provider = CloudflareProvider::with_base_url("t", mock.uri());
        let spec = DnsRecordSpec {
            name: "_ag.ejemplo.com".to_owned(),
            record_type: RecordType::Txt,
            content: "actualizado".to_owned(),
            ttl: 60,
            proxied: false,
        };
        let record = provider
            .update_record("zone1", "rec1", &spec)
            .await
            .unwrap();
        assert_eq!(record.content, "actualizado");
    }

    // ---- delete_record -----------------------------------------------------

    #[tokio::test]
    async fn delete_record_ok() {
        let mock = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/zones/zone1/dns_records/rec1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(cf_ok(serde_json::json!({ "id": "rec1" }))),
            )
            .mount(&mock)
            .await;

        let provider = CloudflareProvider::with_base_url("t", mock.uri());
        provider.delete_record("zone1", "rec1").await.unwrap();
    }

    // ---- contract tests with full mocks ------------------------------------

    async fn full_mock_server() -> (MockServer, CloudflareProvider) {
        let mock = MockServer::start().await;

        // zone_id
        Mock::given(method("GET"))
            .and(path("/zones"))
            .and(query_param("name", "ejemplo.com"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(cf_ok(serde_json::json!([{ "id": "zone-test" }]))),
            )
            .mount(&mock)
            .await;

        // create
        Mock::given(method("POST"))
            .and(path("/zones/zone-test/dns_records"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(cf_ok(mock_cf_record("r1", "zone-test"))),
            )
            .mount(&mock)
            .await;

        // list
        Mock::given(method("GET"))
            .and(path("/zones/zone-test/dns_records"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(cf_ok(serde_json::json!([
                    mock_cf_record("r1", "zone-test"),
                ]))),
            )
            .mount(&mock)
            .await;

        // update
        Mock::given(method("PUT"))
            .and(path("/zones/zone-test/dns_records/r1"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(cf_ok(serde_json::json!({
                    "id": "r1",
                    "zone_id": "zone-test",
                    "name": "_ag-contract-update.ejemplo.com",
                    "type": "TXT",
                    "content": "actualizado",
                    "ttl": 60,
                    "proxied": false
                }))),
            )
            .mount(&mock)
            .await;

        // delete
        Mock::given(method("DELETE"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(cf_ok(serde_json::json!({ "id": "r1" }))),
            )
            .mount(&mock)
            .await;

        let provider = CloudflareProvider::with_base_url("test-token", mock.uri());
        (mock, provider)
    }

    #[tokio::test]
    async fn contract_create_list_delete() {
        let (_mock, provider) = full_mock_server().await;
        contract::test_create_list_delete(&provider, "zone-test").await;
    }

    #[tokio::test]
    async fn contract_update() {
        let (_mock, provider) = full_mock_server().await;
        contract::test_update(&provider, "zone-test").await;
    }
}
