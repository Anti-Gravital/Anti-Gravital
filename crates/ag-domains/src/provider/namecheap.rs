//! Namecheap adapter for `DnsProvider` (feature `namecheap`).
//!
//! Namecheap's host-records API is replace-all: `getHosts` returns every record
//! and `setHosts` rewrites the entire set. This adapter therefore reads the live
//! records, applies a single create/update/delete in memory, and writes the whole
//! set back. That read-modify-write is racy if two writers act concurrently — a
//! documented limitation of the provider, not of this adapter. Records have no
//! stable provider id across a `setHosts`, so a synthetic `TYPE:fqdn` id is used.
//!
//! Authentication is by `ApiUser`/`ApiKey`/`UserName`/`ClientIp` query parameters
//! (no request signing).
//!
//! # Configuration
//!
//! ```rust,ignore
//! use ag_domains::provider::namecheap::NamecheapProvider;
//!
//! let provider = NamecheapProvider::new("user", "api-key", "user", "203.0.113.7");
//! let zone = provider.zone_id("example.com").await?;
//! ```

use async_trait::async_trait;
use serde::Deserialize;

use crate::{
    error::AgDomainsError,
    provider::DnsProvider,
    record::{DnsRecord, DnsRecordSpec, RecordType},
};

const NAMECHEAP_BASE: &str = "https://api.namecheap.com/xml.response";
const GET_HOSTS: &str = "namecheap.domains.dns.getHosts";
const SET_HOSTS: &str = "namecheap.domains.dns.setHosts";
const PROVIDER: &str = "namecheap";

/// Adapter for the Namecheap host-records API.
pub struct NamecheapProvider {
    client: reqwest::Client,
    api_user: String,
    api_key: String,
    username: String,
    client_ip: String,
    base_url: String,
}

impl NamecheapProvider {
    /// Creates an adapter against the production Namecheap endpoint.
    pub fn new(
        api_user: impl Into<String>,
        api_key: impl Into<String>,
        username: impl Into<String>,
        client_ip: impl Into<String>,
    ) -> Self {
        Self::with_base_url(api_user, api_key, username, client_ip, NAMECHEAP_BASE)
    }

    /// Creates an adapter against a custom endpoint (sandbox or wiremock in tests).
    pub fn with_base_url(
        api_user: impl Into<String>,
        api_key: impl Into<String>,
        username: impl Into<String>,
        client_ip: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(concat!("ag-domains/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest client builds with default TLS");
        Self {
            client,
            api_user: api_user.into(),
            api_key: api_key.into(),
            username: username.into(),
            client_ip: client_ip.into(),
            base_url: base_url.into(),
        }
    }

    /// Common query parameters carrying credentials and the command.
    fn base_query(&self, command: &str, sld: &str, tld: &str) -> Vec<(String, String)> {
        vec![
            ("ApiUser".into(), self.api_user.clone()),
            ("ApiKey".into(), self.api_key.clone()),
            ("UserName".into(), self.username.clone()),
            ("ClientIp".into(), self.client_ip.clone()),
            ("Command".into(), command.into()),
            ("SLD".into(), sld.into()),
            ("TLD".into(), tld.into()),
        ]
    }

    /// Reads every host record of a domain.
    async fn get_hosts(&self, sld: &str, tld: &str) -> Result<Vec<HostXml>, AgDomainsError> {
        let query = self.base_query(GET_HOSTS, sld, tld);
        let resp = self
            .client
            .get(&self.base_url)
            .query(&query)
            .send()
            .await
            .map_err(|e| provider_err(e.to_string()))?;
        let text = resp.text().await.map_err(|e| provider_err(e.to_string()))?;
        let parsed: ApiResponse =
            quick_xml::de::from_str(&text).map_err(|e| provider_err(e.to_string()))?;
        parsed.check_ok()?;
        Ok(parsed
            .command_response
            .and_then(|c| c.get_hosts)
            .map(|g| g.hosts)
            .unwrap_or_default())
    }

    /// Rewrites the full host set (Namecheap is replace-all).
    async fn set_hosts(
        &self,
        sld: &str,
        tld: &str,
        hosts: &[HostXml],
    ) -> Result<(), AgDomainsError> {
        let query = self.base_query(SET_HOSTS, sld, tld);
        let mut form: Vec<(String, String)> = Vec::new();
        for (i, h) in hosts.iter().enumerate() {
            let n = i + 1;
            form.push((format!("HostName{n}"), h.name.clone()));
            form.push((format!("RecordType{n}"), h.record_type.clone()));
            form.push((format!("Address{n}"), h.address.clone()));
            form.push((format!("TTL{n}"), h.ttl.to_string()));
        }
        let resp = self
            .client
            .post(&self.base_url)
            .query(&query)
            .form(&form)
            .send()
            .await
            .map_err(|e| provider_err(e.to_string()))?;
        let text = resp.text().await.map_err(|e| provider_err(e.to_string()))?;
        let parsed: ApiResponse =
            quick_xml::de::from_str(&text).map_err(|e| provider_err(e.to_string()))?;
        parsed.check_ok()
    }
}

fn provider_err(message: String) -> AgDomainsError {
    AgDomainsError::Provider {
        provider: PROVIDER,
        message,
    }
}

// ---- Namecheap XML types ----------------------------------------------------

#[derive(Deserialize)]
struct ApiResponse {
    #[serde(rename = "@Status", default)]
    status: String,
    #[serde(rename = "Errors", default)]
    errors: Errors,
    #[serde(rename = "CommandResponse", default)]
    command_response: Option<CommandResponse>,
}

impl ApiResponse {
    fn check_ok(&self) -> Result<(), AgDomainsError> {
        if self.status.eq_ignore_ascii_case("OK") {
            Ok(())
        } else {
            let msg = self
                .errors
                .errors
                .iter()
                .map(|e| e.message.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            Err(provider_err(if msg.is_empty() {
                "API returned ERROR".to_owned()
            } else {
                msg
            }))
        }
    }
}

#[derive(Deserialize, Default)]
struct Errors {
    #[serde(rename = "Error", default)]
    errors: Vec<ErrorEntry>,
}

#[derive(Deserialize)]
struct ErrorEntry {
    #[serde(rename = "$text", default)]
    message: String,
}

#[derive(Deserialize)]
struct CommandResponse {
    #[serde(rename = "DomainDNSGetHostsResult", default)]
    get_hosts: Option<GetHostsResult>,
}

#[derive(Deserialize)]
struct GetHostsResult {
    #[serde(rename = "host", default)]
    hosts: Vec<HostXml>,
}

#[derive(Deserialize, Clone)]
struct HostXml {
    #[serde(rename = "@Name")]
    name: String,
    #[serde(rename = "@Type")]
    record_type: String,
    #[serde(rename = "@Address")]
    address: String,
    #[serde(rename = "@TTL", default = "default_ttl")]
    ttl: u32,
}

fn default_ttl() -> u32 {
    1800
}

fn parse_record_type(t: &str) -> Option<RecordType> {
    match t.to_ascii_uppercase().as_str() {
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

/// Splits a domain into Namecheap's SLD + TLD (TLD is everything after the first label).
fn split_sld_tld(domain: &str) -> Result<(String, String), AgDomainsError> {
    let domain = normalize_name(domain);
    domain
        .split_once('.')
        .map(|(sld, tld)| (sld.to_owned(), tld.to_owned()))
        .ok_or_else(|| provider_err(format!("domain has no TLD: {domain}")))
}

/// Namecheap host name relative to the domain (`@` for the apex).
fn relative_name(fqdn: &str, domain: &str) -> String {
    let fqdn = normalize_name(fqdn);
    let domain = normalize_name(domain);
    if fqdn == domain {
        "@".to_owned()
    } else if let Some(prefix) = fqdn.strip_suffix(&format!(".{domain}")) {
        prefix.to_owned()
    } else {
        fqdn
    }
}

fn fqdn_from_host(name: &str, domain: &str) -> String {
    let domain = normalize_name(domain);
    if name == "@" || name.is_empty() {
        domain
    } else {
        format!("{}.{domain}", normalize_name(name))
    }
}

// ---- DnsProvider impl -------------------------------------------------------

#[async_trait]
impl DnsProvider for NamecheapProvider {
    fn name(&self) -> &'static str {
        PROVIDER
    }

    async fn zone_id(&self, domain: &str) -> Result<String, AgDomainsError> {
        // Namecheap has no zone id; the domain is the handle. Validate it exists
        // by reading its hosts (an unknown/foreign domain returns an API error).
        let (sld, tld) = split_sld_tld(domain)?;
        self.get_hosts(&sld, &tld)
            .await
            .map_err(|_| AgDomainsError::ZoneNotFound(domain.to_owned()))?;
        Ok(normalize_name(domain))
    }

    async fn list_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>, AgDomainsError> {
        let (sld, tld) = split_sld_tld(zone_id)?;
        let hosts = self.get_hosts(&sld, &tld).await?;
        let mut out = Vec::new();
        for h in hosts {
            let Some(record_type) = parse_record_type(&h.record_type) else {
                continue;
            };
            let fqdn = fqdn_from_host(&h.name, zone_id);
            out.push(DnsRecord {
                id: record_id(&record_type, &fqdn),
                zone_id: zone_id.to_owned(),
                name: fqdn,
                record_type,
                content: h.address,
                ttl: h.ttl,
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
        let (sld, tld) = split_sld_tld(zone_id)?;
        let mut hosts = self.get_hosts(&sld, &tld).await?;
        let new_host = HostXml {
            name: relative_name(&spec.name, zone_id),
            record_type: spec.record_type.to_string(),
            address: spec.content.clone(),
            ttl: spec.ttl,
        };
        // Replace any record with the same (name, type), then append the new one.
        hosts.retain(|h| {
            !(h.name == new_host.name && h.record_type.eq_ignore_ascii_case(&new_host.record_type))
        });
        hosts.push(new_host);
        self.set_hosts(&sld, &tld, &hosts).await?;
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
        // create_record already replaces a same-(name,type) record, so it updates.
        self.create_record(zone_id, spec).await
    }

    async fn delete_record(&self, zone_id: &str, record_id: &str) -> Result<(), AgDomainsError> {
        let (rtype_str, fqdn) = record_id
            .split_once(':')
            .ok_or_else(|| provider_err(format!("invalid record id: {record_id}")))?;
        let (sld, tld) = split_sld_tld(zone_id)?;
        let target_name = relative_name(fqdn, zone_id);
        let mut hosts = self.get_hosts(&sld, &tld).await?;
        let before = hosts.len();
        hosts.retain(|h| !(h.name == target_name && h.record_type.eq_ignore_ascii_case(rtype_str)));
        if hosts.len() == before {
            return Err(AgDomainsError::RecordNotFound(record_id.to_owned()));
        }
        self.set_hosts(&sld, &tld, &hosts).await
    }
}

// ---- Tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use wiremock::{
        matchers::{method, query_param},
        Mock, MockServer, Request, Respond, ResponseTemplate,
    };

    use super::*;
    use crate::provider::contract;

    fn ok_set_hosts() -> String {
        "<?xml version=\"1.0\"?><ApiResponse Status=\"OK\"><CommandResponse>\
<DomainDNSSetHostsResult Domain=\"ejemplo.com\" IsSuccess=\"true\"/></CommandResponse></ApiResponse>"
            .to_owned()
    }

    fn get_hosts_xml(hosts: &[(&str, &str, &str)]) -> String {
        let mut body = String::from(
            "<?xml version=\"1.0\"?><ApiResponse Status=\"OK\"><CommandResponse>\
<DomainDNSGetHostsResult Domain=\"ejemplo.com\" IsUsingOurDNS=\"true\">",
        );
        for (i, (name, rtype, addr)) in hosts.iter().enumerate() {
            body.push_str(&format!(
                "<host HostId=\"{i}\" Name=\"{name}\" Type=\"{rtype}\" Address=\"{addr}\" TTL=\"1800\"/>"
            ));
        }
        body.push_str("</DomainDNSGetHostsResult></CommandResponse></ApiResponse>");
        body
    }

    #[tokio::test]
    async fn zone_id_ok_and_not_found() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(query_param("Command", GET_HOSTS))
            .respond_with(ResponseTemplate::new(200).set_body_string(get_hosts_xml(&[])))
            .mount(&mock)
            .await;
        let provider = NamecheapProvider::with_base_url("u", "k", "u", "1.2.3.4", mock.uri());
        assert_eq!(
            provider.zone_id("ejemplo.com").await.unwrap(),
            "ejemplo.com"
        );

        let mock_err = MockServer::start().await;
        Mock::given(method("GET"))
            .and(query_param("Command", GET_HOSTS))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "<?xml version=\"1.0\"?><ApiResponse Status=\"ERROR\"><Errors><Error Number=\"1\">no such domain</Error></Errors></ApiResponse>",
            ))
            .mount(&mock_err)
            .await;
        let provider_err =
            NamecheapProvider::with_base_url("u", "k", "u", "1.2.3.4", mock_err.uri());
        assert!(matches!(
            provider_err.zone_id("nope.com").await.unwrap_err(),
            AgDomainsError::ZoneNotFound(_)
        ));
    }

    #[tokio::test]
    async fn list_records_maps_hosts() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(query_param("Command", GET_HOSTS))
            .respond_with(ResponseTemplate::new(200).set_body_string(get_hosts_xml(&[
                ("@", "A", "192.0.2.1"),
                ("_ag-contract-test", "TXT", "ag-domains-contract-test-value"),
            ])))
            .mount(&mock)
            .await;
        let provider = NamecheapProvider::with_base_url("u", "k", "u", "1.2.3.4", mock.uri());
        let records = provider.list_records("ejemplo.com").await.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, "A:ejemplo.com");
        assert_eq!(records[0].name, "ejemplo.com");
        assert_eq!(records[1].id, "TXT:_ag-contract-test.ejemplo.com");
        assert_eq!(records[1].content, "ag-domains-contract-test-value");
    }

    /// Stateful responder: serves the current host set on getHosts and accepts
    /// setHosts, so a create/list/delete round-trip is observable end to end.
    struct Stateful {
        hosts: Arc<Mutex<Vec<(String, String, String)>>>,
    }

    impl Respond for Stateful {
        fn respond(&self, req: &Request) -> ResponseTemplate {
            let url = req.url.as_str();
            if url.contains(SET_HOSTS) {
                let body = String::from_utf8_lossy(&req.body);
                let mut hosts = Vec::new();
                let mut i = 1;
                loop {
                    let name = form_value(&body, &format!("HostName{i}"));
                    let rtype = form_value(&body, &format!("RecordType{i}"));
                    let addr = form_value(&body, &format!("Address{i}"));
                    match (name, rtype, addr) {
                        (Some(n), Some(t), Some(a)) => hosts.push((n, t, a)),
                        _ => break,
                    }
                    i += 1;
                }
                *self.hosts.lock().unwrap() = hosts;
                ResponseTemplate::new(200).set_body_string(ok_set_hosts())
            } else {
                let hosts = self.hosts.lock().unwrap();
                let refs: Vec<(&str, &str, &str)> = hosts
                    .iter()
                    .map(|(n, t, a)| (n.as_str(), t.as_str(), a.as_str()))
                    .collect();
                ResponseTemplate::new(200).set_body_string(get_hosts_xml(&refs))
            }
        }
    }

    fn form_value(body: &str, key: &str) -> Option<String> {
        body.split('&')
            .find_map(|kv| kv.strip_prefix(&format!("{key}=")))
            .map(|v| v.replace('+', " "))
            .map(|v| urldecode(&v))
    }

    fn urldecode(s: &str) -> String {
        let mut out = String::new();
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                if let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    out.push(b as char);
                    i += 3;
                    continue;
                }
            }
            out.push(bytes[i] as char);
            i += 1;
        }
        out
    }

    #[tokio::test]
    async fn contract_create_list_delete() {
        let mock = MockServer::start().await;
        let state = Stateful {
            hosts: Arc::new(Mutex::new(Vec::new())),
        };
        Mock::given(method("GET"))
            .and(query_param("Command", GET_HOSTS))
            .respond_with(Stateful {
                hosts: state.hosts.clone(),
            })
            .mount(&mock)
            .await;
        Mock::given(method("POST"))
            .and(query_param("Command", SET_HOSTS))
            .respond_with(state)
            .mount(&mock)
            .await;
        let provider = NamecheapProvider::with_base_url("u", "k", "u", "1.2.3.4", mock.uri());
        contract::test_create_list_delete(&provider, "ejemplo.com").await;
    }
}
