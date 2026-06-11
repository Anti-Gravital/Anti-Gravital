//! Azure DNS adapter for `DnsProvider` (feature `azure-dns`).
//!
//! Authenticates with the OAuth2 client-credentials flow (no request signing: a
//! form POST to the tenant token endpoint returns a bearer token), then calls the
//! Azure Resource Manager DNS API. Azure addresses a record set by its relative
//! name and type within a zone; this adapter synthesizes a stable `TYPE:fqdn` id,
//! upserts with `PUT` and deletes with `DELETE`.
//!
//! # Configuration
//!
//! ```rust,ignore
//! use ag_domains::provider::azure_dns::AzureDnsProvider;
//!
//! let provider = AzureDnsProvider::new(tenant, client_id, secret, subscription, rg);
//! let zone = provider.zone_id("example.com").await?;
//! ```

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

use crate::{
    error::AgDomainsError,
    provider::DnsProvider,
    record::{DnsRecord, DnsRecordSpec, RecordType},
};

const MANAGEMENT_BASE: &str = "https://management.azure.com";
const LOGIN_BASE: &str = "https://login.microsoftonline.com";
const API_VERSION: &str = "2018-05-01";
const PROVIDER: &str = "azure-dns";

/// Adapter for the Azure DNS (Resource Manager) API.
pub struct AzureDnsProvider {
    client: reqwest::Client,
    tenant_id: String,
    client_id: String,
    client_secret: String,
    subscription_id: String,
    resource_group: String,
    base_url: String,
    login_url: String,
}

impl AzureDnsProvider {
    /// Creates an adapter against the public Azure endpoints.
    pub fn new(
        tenant_id: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        subscription_id: impl Into<String>,
        resource_group: impl Into<String>,
    ) -> Self {
        Self::with_endpoints(
            tenant_id,
            client_id,
            client_secret,
            subscription_id,
            resource_group,
            MANAGEMENT_BASE,
            LOGIN_BASE,
        )
    }

    /// Creates an adapter with explicit endpoints (custom or wiremock in tests).
    #[allow(clippy::too_many_arguments)]
    pub fn with_endpoints(
        tenant_id: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        subscription_id: impl Into<String>,
        resource_group: impl Into<String>,
        base_url: impl Into<String>,
        login_url: impl Into<String>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(concat!("ag-domains/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest client builds with default TLS");
        Self {
            client,
            tenant_id: tenant_id.into(),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            subscription_id: subscription_id.into(),
            resource_group: resource_group.into(),
            base_url: base_url.into(),
            login_url: login_url.into(),
        }
    }

    /// Acquires a bearer token via the client-credentials grant.
    async fn access_token(&self) -> Result<String, AgDomainsError> {
        let url = format!("{}/{}/oauth2/v2.0/token", self.login_url, self.tenant_id);
        let resp = self
            .client
            .post(&url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
                ("scope", "https://management.azure.com/.default"),
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

    fn zone_base(&self, zone: &str) -> String {
        format!(
            "{}/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/dnsZones/{zone}",
            self.base_url, self.subscription_id, self.resource_group
        )
    }

    async fn send_json(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<String, AgDomainsError> {
        let token = self.access_token().await?;
        let mut req = self.client.request(method, url).bearer_auth(token);
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req.send().await.map_err(|e| provider_err(e.to_string()))?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| provider_err(e.to_string()))?;
        if !status.is_success() {
            return Err(provider_err(format!("HTTP {status}: {text}")));
        }
        Ok(text)
    }
}

fn provider_err(message: String) -> AgDomainsError {
    AgDomainsError::Provider {
        provider: PROVIDER,
        message,
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct ZoneListResponse {
    #[serde(default)]
    value: Vec<ZoneEntry>,
}

#[derive(Deserialize)]
struct ZoneEntry {
    name: String,
}

#[derive(Deserialize)]
struct RecordSetListResponse {
    #[serde(default)]
    value: Vec<serde_json::Value>,
}

fn record_type_path(rt: &RecordType) -> &'static str {
    match rt {
        RecordType::A => "A",
        RecordType::Aaaa => "AAAA",
        RecordType::Cname => "CNAME",
        RecordType::Txt => "TXT",
        RecordType::Mx => "MX",
    }
}

fn parse_record_type(azure_type: &str) -> Option<RecordType> {
    match azure_type.rsplit('/').next()? {
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

fn record_id(record_type: &RecordType, fqdn: &str) -> String {
    format!("{}:{}", record_type, normalize_name(fqdn))
}

/// Azure addresses record sets by name relative to the zone (`@` for the apex).
fn relative_name(fqdn: &str, zone: &str) -> String {
    let fqdn = normalize_name(fqdn);
    let zone = normalize_name(zone);
    if fqdn == zone {
        "@".to_owned()
    } else if let Some(prefix) = fqdn.strip_suffix(&format!(".{zone}")) {
        prefix.to_owned()
    } else {
        fqdn
    }
}

/// Rebuilds the FQDN from an Azure relative record-set name and the zone.
fn fqdn_from_relative(relative: &str, zone: &str) -> String {
    let zone = normalize_name(zone);
    if relative == "@" {
        zone
    } else {
        format!("{}.{zone}", normalize_name(relative))
    }
}

/// Builds the Azure `properties` object for a spec (per-type record arrays).
fn properties_for(spec: &DnsRecordSpec) -> serde_json::Value {
    let ttl = spec.ttl;
    match spec.record_type {
        RecordType::A => json!({ "TTL": ttl, "ARecords": [{ "ipv4Address": spec.content }] }),
        RecordType::Aaaa => {
            json!({ "TTL": ttl, "AAAARecords": [{ "ipv6Address": spec.content }] })
        }
        RecordType::Cname => json!({ "TTL": ttl, "CNAMERecord": { "cname": spec.content } }),
        RecordType::Txt => json!({ "TTL": ttl, "TXTRecords": [{ "value": [spec.content] }] }),
        RecordType::Mx => {
            let (preference, exchange) = parse_mx(&spec.content);
            json!({ "TTL": ttl, "MXRecords": [{ "preference": preference, "exchange": exchange }] })
        }
    }
}

/// Parses MX content of the form `"<preference> <exchange>"`; defaults preference 10.
fn parse_mx(content: &str) -> (u32, String) {
    let mut parts = content.split_whitespace();
    match (parts.next(), parts.next()) {
        (Some(p), Some(ex)) if p.parse::<u32>().is_ok() => (p.parse().unwrap(), ex.to_owned()),
        _ => (10, content.trim().to_owned()),
    }
}

/// Extracts the display content of a record set from its `properties`.
fn content_from_properties(rt: &RecordType, props: &serde_json::Value) -> Option<String> {
    match rt {
        RecordType::A => props["ARecords"][0]["ipv4Address"]
            .as_str()
            .map(str::to_owned),
        RecordType::Aaaa => props["AAAARecords"][0]["ipv6Address"]
            .as_str()
            .map(str::to_owned),
        RecordType::Cname => props["CNAMERecord"]["cname"].as_str().map(str::to_owned),
        RecordType::Txt => props["TXTRecords"][0]["value"].as_array().map(|vals| {
            vals.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join("")
        }),
        RecordType::Mx => {
            let mx = &props["MXRecords"][0];
            let exchange = mx["exchange"].as_str()?;
            let preference = mx["preference"].as_u64().unwrap_or(10);
            Some(format!("{preference} {exchange}"))
        }
    }
}

// ---- DnsProvider impl -------------------------------------------------------

#[async_trait]
impl DnsProvider for AzureDnsProvider {
    fn name(&self) -> &'static str {
        PROVIDER
    }

    async fn zone_id(&self, domain: &str) -> Result<String, AgDomainsError> {
        let url = format!(
            "{}/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/dnsZones?api-version={API_VERSION}",
            self.base_url, self.subscription_id, self.resource_group
        );
        let body = self.send_json(reqwest::Method::GET, &url, None).await?;
        let parsed: ZoneListResponse =
            serde_json::from_str(&body).map_err(|e| provider_err(e.to_string()))?;
        let target = normalize_name(domain);
        parsed
            .value
            .into_iter()
            .find(|z| normalize_name(&z.name) == target)
            .map(|z| z.name)
            .ok_or_else(|| AgDomainsError::ZoneNotFound(domain.to_owned()))
    }

    async fn list_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>, AgDomainsError> {
        let url = format!(
            "{}/recordsets?api-version={API_VERSION}",
            self.zone_base(zone_id)
        );
        let body = self.send_json(reqwest::Method::GET, &url, None).await?;
        let parsed: RecordSetListResponse =
            serde_json::from_str(&body).map_err(|e| provider_err(e.to_string()))?;
        let mut out = Vec::new();
        for set in parsed.value {
            let Some(azure_type) = set["type"].as_str() else {
                continue;
            };
            let Some(record_type) = parse_record_type(azure_type) else {
                continue;
            };
            let relative = set["name"].as_str().unwrap_or("@");
            let Some(content) = content_from_properties(&record_type, &set["properties"]) else {
                continue;
            };
            let ttl = set["properties"]["TTL"].as_u64().unwrap_or(3600) as u32;
            let fqdn = fqdn_from_relative(relative, zone_id);
            out.push(DnsRecord {
                id: record_id(&record_type, &fqdn),
                zone_id: zone_id.to_owned(),
                name: fqdn,
                record_type,
                content,
                ttl,
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
        let relative = relative_name(&spec.name, zone_id);
        let url = format!(
            "{}/{}/{relative}?api-version={API_VERSION}",
            self.zone_base(zone_id),
            record_type_path(&spec.record_type),
        );
        let body = json!({ "properties": properties_for(spec) });
        self.send_json(reqwest::Method::PUT, &url, Some(body))
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
        _record_id: &str,
        spec: &DnsRecordSpec,
    ) -> Result<DnsRecord, AgDomainsError> {
        // PUT is an upsert addressed by (relative name, type), implied by the spec.
        self.create_record(zone_id, spec).await
    }

    async fn delete_record(&self, zone_id: &str, record_id: &str) -> Result<(), AgDomainsError> {
        let (rtype_str, fqdn) = record_id
            .split_once(':')
            .ok_or_else(|| provider_err(format!("invalid record id: {record_id}")))?;
        let record_type = parse_record_type(rtype_str)
            .ok_or_else(|| provider_err(format!("unsupported record type: {rtype_str}")))?;
        let relative = relative_name(fqdn, zone_id);
        let url = format!(
            "{}/{}/{relative}?api-version={API_VERSION}",
            self.zone_base(zone_id),
            record_type_path(&record_type),
        );
        self.send_json(reqwest::Method::DELETE, &url, None).await?;
        Ok(())
    }
}

// ---- Tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use wiremock::{
        matchers::{method, path, path_regex},
        Mock, MockServer, ResponseTemplate,
    };

    use super::*;
    use crate::provider::contract;

    async fn mount_token(mock: &MockServer) {
        Mock::given(method("POST"))
            .and(path("/tenant-1/oauth2/v2.0/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "test-token", "token_type": "Bearer", "expires_in": 3600
            })))
            .mount(mock)
            .await;
    }

    fn provider_for(mock: &MockServer) -> AzureDnsProvider {
        AzureDnsProvider::with_endpoints(
            "tenant-1",
            "client-1",
            "secret",
            "sub-1",
            "rg-1",
            mock.uri(),
            mock.uri(),
        )
    }

    fn zone_base_path() -> String {
        "/subscriptions/sub-1/resourceGroups/rg-1/providers/Microsoft.Network/dnsZones".to_owned()
    }

    #[tokio::test]
    async fn zone_id_ok() {
        let mock = MockServer::start().await;
        mount_token(&mock).await;
        Mock::given(method("GET"))
            .and(path(zone_base_path()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "value": [{ "name": "ejemplo.com" }]
            })))
            .mount(&mock)
            .await;
        let provider = provider_for(&mock);
        assert_eq!(
            provider.zone_id("ejemplo.com").await.unwrap(),
            "ejemplo.com"
        );
    }

    #[tokio::test]
    async fn zone_id_not_found() {
        let mock = MockServer::start().await;
        mount_token(&mock).await;
        Mock::given(method("GET"))
            .and(path(zone_base_path()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "value": [] })))
            .mount(&mock)
            .await;
        let provider = provider_for(&mock);
        assert!(matches!(
            provider.zone_id("nope.com").await.unwrap_err(),
            AgDomainsError::ZoneNotFound(_)
        ));
    }

    fn recordsets_body() -> serde_json::Value {
        json!({
            "value": [{
                "name": "_ag-contract-test",
                "type": "Microsoft.Network/dnszones/TXT",
                "properties": {
                    "TTL": 60,
                    "TXTRecords": [{ "value": ["ag-domains-contract-test-value"] }]
                }
            }]
        })
    }

    async fn full_mock_server() -> (MockServer, AzureDnsProvider) {
        let mock = MockServer::start().await;
        mount_token(&mock).await;
        Mock::given(method("GET"))
            .and(path(format!("{}/ejemplo.com/recordsets", zone_base_path())))
            .respond_with(ResponseTemplate::new(200).set_body_json(recordsets_body()))
            .mount(&mock)
            .await;
        // PUT upsert and DELETE for any record-set path under the zone.
        Mock::given(method("PUT"))
            .and(path_regex(r".*/dnsZones/ejemplo\.com/.+"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": "rs" })))
            .mount(&mock)
            .await;
        Mock::given(method("DELETE"))
            .and(path_regex(r".*/dnsZones/ejemplo\.com/.+"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock)
            .await;
        let provider = provider_for(&mock);
        (mock, provider)
    }

    #[tokio::test]
    async fn list_records_parses_txt() {
        let (_mock, provider) = full_mock_server().await;
        let records = provider.list_records("ejemplo.com").await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].content, "ag-domains-contract-test-value");
        assert_eq!(records[0].id, "TXT:_ag-contract-test.ejemplo.com");
        assert_eq!(records[0].name, "_ag-contract-test.ejemplo.com");
    }

    #[tokio::test]
    async fn relative_name_handles_apex_and_sub() {
        assert_eq!(relative_name("ejemplo.com", "ejemplo.com"), "@");
        assert_eq!(relative_name("api.ejemplo.com", "ejemplo.com"), "api");
        assert_eq!(relative_name("a.b.ejemplo.com", "ejemplo.com"), "a.b");
    }

    #[tokio::test]
    async fn contract_create_list_delete() {
        let (_mock, provider) = full_mock_server().await;
        contract::test_create_list_delete(&provider, "ejemplo.com").await;
    }
}
