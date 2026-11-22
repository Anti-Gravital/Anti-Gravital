//! Canonical data types and deterministic helpers for `ag dns`.
//!
//! No I/O happens here: this module only declares the manifest schema, the
//! resolved binding, the plan shape and the hashing/field helpers used to give
//! every plan a stable identity.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use ag_domains::record::RecordType;
use serde::{Deserialize, Serialize};

/// Canonical manifest path (RFC-0019 `ag.toml`), relative to the project root.
pub(crate) const DEFAULT_MANIFEST: &str = "ag.toml";
/// Local resolved-configuration path, relative to the project root.
pub(crate) const DEFAULT_STATE: &str = ".ag/dns/state.json";

pub(crate) const STATE_VERSION: u32 = 1;
pub(crate) const DEFAULT_TTL: u32 = 300;
pub(crate) const MIN_TTL: u32 = 60;
pub(crate) const MAX_TTL: u32 = 86_400;

/// Canonical environment ids of RFC-0033 section 4.8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Environment {
    Dev,
    Staging,
    Production,
    /// Loopback only: a valid target with an empty domain plan.
    Local,
}

impl Environment {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Environment::Dev => "dev",
            Environment::Staging => "staging",
            Environment::Production => "production",
            Environment::Local => "local",
        }
    }

    /// `prod` is normalized to `production`; other aliases are accepted at
    /// the CLI boundary for compatibility with wave-8 adapters.
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "dev" | "development" | "develop" => Ok(Environment::Dev),
            "staging" | "stage" => Ok(Environment::Staging),
            "prod" | "production" => Ok(Environment::Production),
            "local" => Ok(Environment::Local),
            other => Err(format!(
                "unknown environment '{other}'; expected dev, staging, production or local"
            )),
        }
    }
}

/// Exposure of a domain configuration. Private is the fail-closed default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Visibility {
    Private,
    Public,
}

impl Visibility {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Visibility::Private => "private",
            Visibility::Public => "public",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "private" => Ok(Visibility::Private),
            "public" => Ok(Visibility::Public),
            other => Err(format!(
                "unknown visibility '{other}'; expected public or private"
            )),
        }
    }
}

/// TLS mode reused from the ag-domains ACME contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TlsMode {
    Disabled,
    ManagedHttp01,
    ManagedDns01,
}

impl TlsMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            TlsMode::Disabled => "disabled",
            TlsMode::ManagedHttp01 => "managed_http01",
            TlsMode::ManagedDns01 => "managed_dns01",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "disabled" => Ok(TlsMode::Disabled),
            "managed_http01" => Ok(TlsMode::ManagedHttp01),
            "managed_dns01" => Ok(TlsMode::ManagedDns01),
            other => Err(format!(
                "unknown tls mode '{other}'; expected disabled, managed_http01 or managed_dns01"
            )),
        }
    }
}

/// Raw manifest shape (RFC-0033 section 4.1).
#[derive(Debug, Deserialize)]
pub(crate) struct AgManifest {
    #[serde(default)]
    pub(crate) project: Option<RawProject>,
    pub(crate) domain: Option<RawDomain>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawProject {
    #[serde(default)]
    pub(crate) name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawDomain {
    pub(crate) domain: String,
    #[serde(default)]
    pub(crate) zone: Option<String>,
    #[serde(default = "default_environment")]
    pub(crate) environment: String,
    #[serde(default = "default_visibility")]
    pub(crate) visibility: String,
    #[serde(default)]
    pub(crate) target: RawTarget,
    #[serde(default = "default_ttl")]
    pub(crate) ttl: u32,
    #[serde(default = "default_tls")]
    pub(crate) tls: String,
    #[serde(default)]
    pub(crate) allow_wildcard: bool,
    #[serde(default)]
    pub(crate) dns: BTreeMap<String, Vec<RawExtraRecord>>,
    #[serde(default)]
    pub(crate) environments: BTreeMap<String, RawDomainOverride>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RawDomainOverride {
    #[serde(default)]
    pub(crate) domain: Option<String>,
    #[serde(default)]
    pub(crate) zone: Option<String>,
    #[serde(default)]
    pub(crate) visibility: Option<String>,
    #[serde(default)]
    pub(crate) target: Option<RawTarget>,
    #[serde(default)]
    pub(crate) ttl: Option<u32>,
    #[serde(default)]
    pub(crate) tls: Option<String>,
    #[serde(default)]
    pub(crate) allow_wildcard: Option<bool>,
    #[serde(default)]
    pub(crate) dns: Option<BTreeMap<String, Vec<RawExtraRecord>>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawTarget {
    #[serde(default = "default_target_kind")]
    pub(crate) kind: String,
    #[serde(rename = "ref", default = "default_target_ref")]
    pub(crate) ref_value: String,
}

impl Default for RawTarget {
    fn default() -> Self {
        Self {
            kind: default_target_kind(),
            ref_value: default_target_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct RawExtraRecord {
    pub(crate) name: String,
    pub(crate) value: String,
}

pub(crate) fn default_environment() -> String {
    "production".to_owned()
}

pub(crate) fn default_visibility() -> String {
    "private".to_owned()
}

pub(crate) fn default_ttl() -> u32 {
    DEFAULT_TTL
}

pub(crate) fn default_tls() -> String {
    "managed_http01".to_owned()
}

pub(crate) fn default_target_kind() -> String {
    "service".to_owned()
}

pub(crate) fn default_target_ref() -> String {
    "default".to_owned()
}

/// Resolved domain binding: the only input to plan generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Binding {
    pub(crate) host: String,
    pub(crate) zone: String,
    pub(crate) environment: Environment,
    pub(crate) visibility: Visibility,
    pub(crate) target_ref: String,
    pub(crate) ttl: u32,
    pub(crate) tls: TlsMode,
    pub(crate) allow_wildcard: bool,
    pub(crate) dns: BTreeMap<String, Vec<RawExtraRecord>>,
}

/// A validated DNS record in a plan. Ordered and serialized deterministically.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PlannedRecord {
    pub(crate) record_type: RecordType,
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) ttl: u32,
    pub(crate) is_ownership: bool,
}

/// The complete, network-free result of `ag dns plan`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DnsPlan {
    pub(crate) plan_id: String,
    pub(crate) domain: String,
    pub(crate) zone: String,
    pub(crate) environment: Environment,
    pub(crate) visibility: Visibility,
    pub(crate) project: String,
    pub(crate) service: String,
    pub(crate) target_kind: String,
    pub(crate) tls: TlsMode,
    pub(crate) records: Vec<PlannedRecord>,
    pub(crate) notes: Vec<String>,
}

impl DnsPlan {
    /// Canonical stream for the identity-only plan id. It has no credential,
    /// zone id from a provider, path or timestamp, so the same binding always
    /// produces the same id on any machine.
    pub(crate) fn canonical_stream(&self) -> String {
        let mut out = String::from("ag-dns-plan-v1\n");
        push_field(&mut out, "domain", &self.domain);
        push_field(&mut out, "zone", &self.zone);
        push_field(&mut out, "environment", self.environment.as_str());
        push_field(&mut out, "visibility", self.visibility.as_str());
        push_field(&mut out, "project", &self.project);
        push_field(&mut out, "service", &self.service);
        push_field(&mut out, "target_kind", &self.target_kind);
        push_field(&mut out, "tls", self.tls.as_str());
        for record in &self.records {
            let row = format!(
                "{}|{}|{}|{}|{}",
                record.record_type, record.name, record.value, record.ttl, record.is_ownership
            );
            push_field(&mut out, "record", &row);
        }
        out
    }
}

pub(crate) fn push_field(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "{key}:{}:{value}", value.len());
}

pub(crate) fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}
