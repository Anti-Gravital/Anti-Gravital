//! Amazon Route 53 adapter for `DnsProvider` (feature `route53`).
//!
//! Route 53 is a global service signed with AWS Signature Version 4 (region
//! `us-east-1`, service `route53`). Unlike Cloudflare, Route 53 has no opaque
//! per-record id: a record is identified by its `(name, type)` resource record
//! set. This adapter therefore synthesizes a stable id as `TYPE:name` and, on
//! delete, reads the live values back to build the exact `DELETE` change Route 53
//! requires.
//!
//! Authentication uses an access key id + secret access key. The SigV4 signer is
//! verified against AWS's published `get-vanilla` test vector (see tests); the
//! live API path is covered by an `#[ignore]` credentialed test.
//!
//! # Configuration
//!
//! ```rust,ignore
//! use ag_domains::provider::route53::Route53Provider;
//!
//! let provider = Route53Provider::new("AKID...", "secret...");
//! let zone_id = provider.zone_id("example.com").await?;
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{
    error::AgDomainsError,
    provider::DnsProvider,
    record::{DnsRecord, DnsRecordSpec, RecordType},
};

const ROUTE53_BASE: &str = "https://route53.amazonaws.com";
const API_VERSION: &str = "2013-04-01";
const REGION: &str = "us-east-1";
const SERVICE: &str = "route53";
const PROVIDER: &str = "route53";

/// Adapter for the Amazon Route 53 REST API.
pub struct Route53Provider {
    client: reqwest::Client,
    access_key: String,
    secret_key: String,
    base_url: String,
}

impl Route53Provider {
    /// Creates an adapter pointing at the Route 53 production endpoint.
    pub fn new(access_key: impl Into<String>, secret_key: impl Into<String>) -> Self {
        Self::with_base_url(access_key, secret_key, ROUTE53_BASE)
    }

    /// Creates an adapter pointing at a custom base URL (wiremock in tests).
    pub fn with_base_url(
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(concat!("ag-domains/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest client builds with default TLS");
        Self {
            client,
            access_key: access_key.into(),
            secret_key: secret_key.into(),
            base_url: base_url.into(),
        }
    }

    fn host(&self) -> String {
        self.base_url
            .split("://")
            .nth(1)
            .unwrap_or(&self.base_url)
            .split('/')
            .next()
            .unwrap_or_default()
            .to_owned()
    }

    /// Performs a signed request and returns the response body text, mapping
    /// transport and non-2xx responses to provider errors.
    async fn send(
        &self,
        method: &str,
        path: &str,
        query: &str,
        body: &str,
    ) -> Result<String, AgDomainsError> {
        let now = Utc::now();
        let host = self.host();
        let auth = sigv4::authorization_header(SigV4Request {
            method,
            host: &host,
            canonical_uri: path,
            canonical_query: query,
            payload: body,
            access_key: &self.access_key,
            secret_key: &self.secret_key,
            region: REGION,
            service: SERVICE,
            now,
        });
        let url = if query.is_empty() {
            format!("{}{path}", self.base_url)
        } else {
            format!("{}{path}?{query}", self.base_url)
        };
        let req = self
            .client
            .request(method.parse().expect("static method is valid"), &url)
            .header("Host", &host)
            .header("X-Amz-Date", sigv4::amz_date(now))
            .header("Authorization", auth);
        let req = if body.is_empty() {
            req
        } else {
            req.header("Content-Type", "text/xml").body(body.to_owned())
        };
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

// ---- Route 53 XML response types -------------------------------------------

#[derive(Deserialize)]
struct ListHostedZonesByNameResponse {
    #[serde(rename = "HostedZones", default)]
    hosted_zones: HostedZones,
}

#[derive(Deserialize, Default)]
struct HostedZones {
    #[serde(rename = "HostedZone", default)]
    zones: Vec<HostedZoneXml>,
}

#[derive(Deserialize)]
struct HostedZoneXml {
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "Name")]
    name: String,
}

#[derive(Deserialize)]
struct ListResourceRecordSetsResponse {
    #[serde(rename = "ResourceRecordSets", default)]
    record_sets: ResourceRecordSets,
}

#[derive(Deserialize, Default)]
struct ResourceRecordSets {
    #[serde(rename = "ResourceRecordSet", default)]
    sets: Vec<ResourceRecordSetXml>,
}

#[derive(Deserialize)]
struct ResourceRecordSetXml {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Type")]
    record_type: String,
    #[serde(rename = "TTL", default)]
    ttl: u32,
    #[serde(rename = "ResourceRecords", default)]
    resource_records: ResourceRecords,
}

#[derive(Deserialize, Default)]
struct ResourceRecords {
    #[serde(rename = "ResourceRecord", default)]
    records: Vec<ResourceRecordXml>,
}

#[derive(Deserialize)]
struct ResourceRecordXml {
    #[serde(rename = "Value")]
    value: String,
}

/// Parses a Route 53 record type string into the supported subset.
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

/// Normalizes a DNS name for stable identity: lowercase, no trailing dot.
fn normalize_name(name: &str) -> String {
    name.trim_end_matches('.').to_ascii_lowercase()
}

/// Stable synthetic id for a Route 53 record: `TYPE:name`.
fn record_id(record_type: &RecordType, name: &str) -> String {
    format!("{}:{}", record_type, normalize_name(name))
}

/// TXT values are quoted on the wire; strip a single surrounding pair for display.
fn unquote_txt(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .unwrap_or(value)
        .to_owned()
}

/// Wire value for a spec: TXT content is wrapped in quotes, others verbatim.
fn wire_value(spec: &DnsRecordSpec) -> String {
    if spec.record_type == RecordType::Txt && !spec.content.starts_with('"') {
        format!("\"{}\"", spec.content)
    } else {
        spec.content.clone()
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn change_batch_xml(action: &str, name: &str, rtype: &RecordType, ttl: u32, value: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<ChangeResourceRecordSetsRequest xmlns=\"https://route53.amazonaws.com/doc/2013-04-01/\">\
<ChangeBatch><Changes><Change>\
<Action>{action}</Action>\
<ResourceRecordSet><Name>{}</Name><Type>{rtype}</Type><TTL>{ttl}</TTL>\
<ResourceRecords><ResourceRecord><Value>{}</Value></ResourceRecord></ResourceRecords>\
</ResourceRecordSet></Change></Changes></ChangeBatch>\
</ChangeResourceRecordSetsRequest>",
        xml_escape(name),
        xml_escape(value),
    )
}

// ---- DnsProvider impl -------------------------------------------------------

#[async_trait]
impl DnsProvider for Route53Provider {
    fn name(&self) -> &'static str {
        PROVIDER
    }

    async fn zone_id(&self, domain: &str) -> Result<String, AgDomainsError> {
        let path = format!("/{API_VERSION}/hostedzonesbyname");
        let query = format!("dnsname={domain}&maxitems=10");
        let body = self.send("GET", &path, &query, "").await?;
        let parsed: ListHostedZonesByNameResponse =
            quick_xml::de::from_str(&body).map_err(|e| provider_err(e.to_string()))?;
        let target = normalize_name(domain);
        parsed
            .hosted_zones
            .zones
            .into_iter()
            .find(|z| normalize_name(&z.name) == target)
            .map(|z| z.id.trim_start_matches("/hostedzone/").to_owned())
            .ok_or_else(|| AgDomainsError::ZoneNotFound(domain.to_owned()))
    }

    async fn list_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>, AgDomainsError> {
        let path = format!("/{API_VERSION}/hostedzone/{zone_id}/rrset");
        let body = self.send("GET", &path, "", "").await?;
        let parsed: ListResourceRecordSetsResponse =
            quick_xml::de::from_str(&body).map_err(|e| provider_err(e.to_string()))?;
        let mut out = Vec::new();
        for set in parsed.record_sets.sets {
            let Some(record_type) = parse_record_type(&set.record_type) else {
                continue; // Skip record types ag-domains does not model.
            };
            let content = match set.resource_records.records.first() {
                Some(r) if record_type == RecordType::Txt => unquote_txt(&r.value),
                Some(r) => r.value.clone(),
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
        // UPSERT is idempotent and serves both create and update on Route 53.
        let xml = change_batch_xml(
            "UPSERT",
            &spec.name,
            &spec.record_type,
            spec.ttl,
            &wire_value(spec),
        );
        let path = format!("/{API_VERSION}/hostedzone/{zone_id}/rrset/");
        self.send("POST", &path, "", &xml).await?;
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
        // UPSERT replaces by (name, type); the record id is implied by the spec.
        self.create_record(zone_id, spec).await
    }

    async fn delete_record(&self, zone_id: &str, record_id: &str) -> Result<(), AgDomainsError> {
        // Route 53 DELETE needs the exact current values; read them back first.
        let (rtype_str, target_name) = record_id
            .split_once(':')
            .ok_or_else(|| provider_err(format!("invalid record id: {record_id}")))?;
        let target = normalize_name(target_name);
        let existing = self
            .list_records(zone_id)
            .await?
            .into_iter()
            .find(|r| r.record_type.to_string() == rtype_str && r.name == target)
            .ok_or_else(|| AgDomainsError::RecordNotFound(record_id.to_owned()))?;
        let value = if existing.record_type == RecordType::Txt {
            format!("\"{}\"", existing.content)
        } else {
            existing.content.clone()
        };
        let xml = change_batch_xml(
            "DELETE",
            &existing.name,
            &existing.record_type,
            existing.ttl,
            &value,
        );
        let path = format!("/{API_VERSION}/hostedzone/{zone_id}/rrset/");
        self.send("POST", &path, "", &xml).await?;
        Ok(())
    }
}

// ---- AWS Signature Version 4 ------------------------------------------------

/// Inputs to the SigV4 `Authorization` header computation.
struct SigV4Request<'a> {
    method: &'a str,
    host: &'a str,
    canonical_uri: &'a str,
    canonical_query: &'a str,
    payload: &'a str,
    access_key: &'a str,
    secret_key: &'a str,
    region: &'a str,
    service: &'a str,
    now: DateTime<Utc>,
}

mod sigv4 {
    use super::*;

    type HmacSha256 = Hmac<Sha256>;

    pub(super) fn amz_date(now: DateTime<Utc>) -> String {
        now.format("%Y%m%dT%H%M%SZ").to_string()
    }

    fn date_stamp(now: DateTime<Utc>) -> String {
        now.format("%Y%m%d").to_string()
    }

    fn sha256_hex(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn hmac(key: &[u8], data: &str) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
        mac.update(data.as_bytes());
        mac.finalize().into_bytes().to_vec()
    }

    /// Derives the SigV4 signing key (kDate -> kRegion -> kService -> kSigning).
    fn signing_key(secret: &str, date: &str, region: &str, service: &str) -> Vec<u8> {
        let k_date = hmac(format!("AWS4{secret}").as_bytes(), date);
        let k_region = hmac(&k_date, region);
        let k_service = hmac(&k_region, service);
        hmac(&k_service, "aws4_request")
    }

    /// Builds the full `Authorization` header value for a request.
    pub(super) fn authorization_header(req: SigV4Request<'_>) -> String {
        let amz_date = amz_date(req.now);
        let date_stamp = date_stamp(req.now);
        let payload_hash = sha256_hex(req.payload);

        let canonical_headers = format!("host:{}\nx-amz-date:{}\n", req.host, amz_date);
        let signed_headers = "host;x-amz-date";
        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            req.method,
            req.canonical_uri,
            req.canonical_query,
            canonical_headers,
            signed_headers,
            payload_hash,
        );

        let scope = format!("{date_stamp}/{}/{}/aws4_request", req.region, req.service);
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{}",
            sha256_hex(&canonical_request),
        );

        let key = signing_key(req.secret_key, &date_stamp, req.region, req.service);
        let signature = hex::encode(hmac(&key, &string_to_sign));

        format!(
            "AWS4-HMAC-SHA256 Credential={}/{scope}, SignedHeaders={signed_headers}, Signature={signature}",
            req.access_key,
        )
    }
}

// ---- Tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::*;
    use crate::provider::contract;

    /// AWS's published `get-vanilla` SigV4 test vector. Verifies the whole signer
    /// (canonical request, string-to-sign, key derivation) against a known result.
    #[test]
    fn sigv4_matches_aws_get_vanilla_vector() {
        let now = Utc.with_ymd_and_hms(2015, 8, 30, 12, 36, 0).unwrap();
        let header = sigv4::authorization_header(SigV4Request {
            method: "GET",
            host: "example.amazonaws.com",
            canonical_uri: "/",
            canonical_query: "",
            payload: "",
            access_key: "AKIDEXAMPLE",
            secret_key: "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
            region: "us-east-1",
            service: "service",
            now,
        });
        assert_eq!(
            header,
            "AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20150830/us-east-1/service/aws4_request, \
             SignedHeaders=host;x-amz-date, \
             Signature=5fa00fa31553b73ebf1942676e86291e8372ff2a2260956d9b8aae1d763fbf31"
        );
    }

    fn list_zones_xml() -> &'static str {
        "<?xml version=\"1.0\"?><ListHostedZonesByNameResponse>\
<HostedZones><HostedZone><Id>/hostedzone/Z-TEST</Id><Name>ejemplo.com.</Name></HostedZone>\
</HostedZones></ListHostedZonesByNameResponse>"
    }

    fn rrset_xml(content: &str) -> String {
        format!(
            "<?xml version=\"1.0\"?><ListResourceRecordSetsResponse><ResourceRecordSets>\
<ResourceRecordSet><Name>_ag-contract-test.ejemplo.com.</Name><Type>TXT</Type><TTL>60</TTL>\
<ResourceRecords><ResourceRecord><Value>\"{content}\"</Value></ResourceRecord></ResourceRecords>\
</ResourceRecordSet></ResourceRecordSets></ListResourceRecordSetsResponse>"
        )
    }

    #[tokio::test]
    async fn zone_id_ok() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/2013-04-01/hostedzonesbyname"))
            .respond_with(ResponseTemplate::new(200).set_body_string(list_zones_xml()))
            .mount(&mock)
            .await;
        let provider = Route53Provider::with_base_url("AKID", "secret", mock.uri());
        let id = provider.zone_id("ejemplo.com").await.unwrap();
        assert_eq!(id, "Z-TEST");
    }

    #[tokio::test]
    async fn zone_id_not_found() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/2013-04-01/hostedzonesbyname"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "<?xml version=\"1.0\"?><ListHostedZonesByNameResponse><HostedZones></HostedZones></ListHostedZonesByNameResponse>",
            ))
            .mount(&mock)
            .await;
        let provider = Route53Provider::with_base_url("AKID", "secret", mock.uri());
        let err = provider.zone_id("nope.com").await.unwrap_err();
        assert!(matches!(err, AgDomainsError::ZoneNotFound(_)));
    }

    #[tokio::test]
    async fn list_records_parses_and_unquotes_txt() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/2013-04-01/hostedzone/Z-TEST/rrset"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(rrset_xml("ag-domains-contract-test-value")),
            )
            .mount(&mock)
            .await;
        let provider = Route53Provider::with_base_url("AKID", "secret", mock.uri());
        let records = provider.list_records("Z-TEST").await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].content, "ag-domains-contract-test-value");
        assert_eq!(records[0].record_type, RecordType::Txt);
        assert_eq!(records[0].id, "TXT:_ag-contract-test.ejemplo.com");
    }

    async fn full_mock_server() -> (MockServer, Route53Provider) {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/2013-04-01/hostedzonesbyname"))
            .respond_with(ResponseTemplate::new(200).set_body_string(list_zones_xml()))
            .mount(&mock)
            .await;
        Mock::given(method("GET"))
            .and(path("/2013-04-01/hostedzone/Z-TEST/rrset"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(rrset_xml("ag-domains-contract-test-value")),
            )
            .mount(&mock)
            .await;
        Mock::given(method("POST"))
            .and(path("/2013-04-01/hostedzone/Z-TEST/rrset/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "<?xml version=\"1.0\"?><ChangeResourceRecordSetsResponse><ChangeInfo><Id>/change/C1</Id><Status>PENDING</Status></ChangeInfo></ChangeResourceRecordSetsResponse>",
            ))
            .mount(&mock)
            .await;
        let provider = Route53Provider::with_base_url("AKID", "secret", mock.uri());
        (mock, provider)
    }

    #[tokio::test]
    async fn contract_create_list_delete() {
        let (_mock, provider) = full_mock_server().await;
        contract::test_create_list_delete(&provider, "Z-TEST").await;
    }
}
