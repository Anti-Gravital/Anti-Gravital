//! `ag dns` commands: local, plan-first domain configuration (RFC-0033).
//!
//! The canonical input is the `\[domain\]` table of the project's `ag.toml`.
//! `plan` is read-only and deterministic; `apply` is gated by the exact plan
//! id and writes only a local resolved configuration. A provider adapter is
//! contacted only when `--provider` is passed explicitly. No credential, zone
//! id from a provider or token is ever read from or written to a file.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::net::IpAddr;
use std::path::Path;

use ag_domains::hostname::{Hostname, HostnameKind};
use ag_domains::provider::DnsProvider;
use ag_domains::record::{DnsRecordSpec, RecordType};
use serde::{Deserialize, Serialize};

/// Canonical manifest path (RFC-0019 `ag.toml`), relative to the project root.
pub(crate) const DEFAULT_MANIFEST: &str = "ag.toml";
/// Local resolved-configuration path, relative to the project root.
pub(crate) const DEFAULT_STATE: &str = ".ag/dns/state.json";

const STATE_VERSION: u32 = 1;
const DEFAULT_TTL: u32 = 300;
const MIN_TTL: u32 = 60;
const MAX_TTL: u32 = 86_400;

// ============================================================
// Environment, visibility, TLS and target parsing
// ============================================================

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

    fn parse(value: &str) -> Result<Self, String> {
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

// ============================================================
// Raw manifest types (RFC-0033 section 4.1)
// ============================================================

#[derive(Debug, Deserialize)]
struct AgManifest {
    #[serde(default)]
    project: Option<RawProject>,
    domain: Option<RawDomain>,
}

#[derive(Debug, Deserialize)]
struct RawProject {
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDomain {
    domain: String,
    #[serde(default)]
    zone: Option<String>,
    #[serde(default = "default_environment")]
    environment: String,
    #[serde(default = "default_visibility")]
    visibility: String,
    #[serde(default)]
    target: RawTarget,
    #[serde(default = "default_ttl")]
    ttl: u32,
    #[serde(default = "default_tls")]
    tls: String,
    #[serde(default)]
    allow_wildcard: bool,
    #[serde(default)]
    dns: BTreeMap<String, Vec<RawExtraRecord>>,
    #[serde(default)]
    environments: BTreeMap<String, RawDomainOverride>,
}

#[derive(Debug, Default, Deserialize)]
struct RawDomainOverride {
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    zone: Option<String>,
    #[serde(default)]
    visibility: Option<String>,
    #[serde(default)]
    target: Option<RawTarget>,
    #[serde(default)]
    ttl: Option<u32>,
    #[serde(default)]
    tls: Option<String>,
    #[serde(default)]
    allow_wildcard: Option<bool>,
    #[serde(default)]
    dns: Option<BTreeMap<String, Vec<RawExtraRecord>>>,
}

#[derive(Debug, Deserialize)]
struct RawTarget {
    #[serde(default = "default_target_kind")]
    kind: String,
    #[serde(rename = "ref", default = "default_target_ref")]
    ref_value: String,
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
struct RawExtraRecord {
    name: String,
    value: String,
}

fn default_environment() -> String {
    "production".to_owned()
}

fn default_visibility() -> String {
    "private".to_owned()
}

fn default_ttl() -> u32 {
    DEFAULT_TTL
}

fn default_tls() -> String {
    "managed_http01".to_owned()
}

fn default_target_kind() -> String {
    "service".to_owned()
}

fn default_target_ref() -> String {
    "default".to_owned()
}

/// Resolved domain binding: the only input to plan generation.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Binding {
    host: String,
    zone: String,
    environment: Environment,
    visibility: Visibility,
    target_ref: String,
    ttl: u32,
    tls: TlsMode,
    allow_wildcard: bool,
    dns: BTreeMap<String, Vec<RawExtraRecord>>,
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
    fn canonical_stream(&self) -> String {
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

fn push_field(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "{key}:{}:{value}", value.len());
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

// ============================================================
// Manifest loading and validation
// ============================================================

fn parse_record_type(value: &str) -> Result<RecordType, String> {
    match value.trim().to_ascii_uppercase().as_str() {
        "A" => Ok(RecordType::A),
        "AAAA" => Ok(RecordType::Aaaa),
        "CNAME" => Ok(RecordType::Cname),
        "TXT" => Ok(RecordType::Txt),
        "MX" => Ok(RecordType::Mx),
        other => Err(format!(
            "unsupported record type '{other}'; expected A, AAAA, CNAME, TXT or MX"
        )),
    }
}

fn validate_record_name(name: &str, allow_wildcard: bool) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("record name is empty".to_owned());
    }
    if name.chars().any(char::is_whitespace) {
        return Err(format!("record name '{name}' contains whitespace"));
    }
    if name.contains("://") || name.contains('/') || name.contains('?') || name.contains('#') {
        return Err(format!("record name '{name}' must be a bare DNS name"));
    }
    if name.contains('*') {
        if !allow_wildcard {
            return Err(format!(
                "record name '{name}' uses a wildcard; set domain.allow_wildcard = true with \
                 tls = \"managed_dns01\" to opt in"
            ));
        }
        let rest = name.strip_prefix("*.").ok_or_else(|| {
            format!("record name '{name}' has an invalid wildcard; only a leading '*.' is allowed")
        })?;
        if rest.contains('*') || rest.is_empty() {
            return Err(format!("record name '{name}' has an invalid wildcard"));
        }
    }
    Ok(())
}

fn validate_record_value(record_type: &RecordType, value: &str) -> Result<(), String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("record value is empty".to_owned());
    }
    if value.chars().any(char::is_control) {
        return Err("record value contains a control character".to_owned());
    }
    match record_type {
        RecordType::A | RecordType::Aaaa => {
            let ip: IpAddr = value.parse().map_err(|_| {
                format!("record value '{value}' is not a valid IP address for {record_type}")
            })?;
            let family_mismatch = matches!(record_type, RecordType::A) && ip.is_ipv6()
                || matches!(record_type, RecordType::Aaaa) && ip.is_ipv4();
            if family_mismatch {
                return Err(format!(
                    "record value '{value}' does not match record type {record_type}"
                ));
            }
        }
        RecordType::Cname | RecordType::Mx => {
            if value.chars().any(char::is_whitespace) {
                return Err(format!("record value '{value}' is not a valid host target"));
            }
        }
        RecordType::Txt => {}
    }
    Ok(())
}

/// Expands a record name relative to the registrable zone, as defined by
/// RFC-0033 section 4.1. `@` denotes the zone apex.
fn expand_record_name(name: &str, zone: &str) -> String {
    let trimmed = name.trim().trim_end_matches('.');
    if trimmed == "@" {
        zone.to_owned()
    } else if trimmed.contains('.') {
        trimmed.to_owned()
    } else {
        format!("{trimmed}.{zone}")
    }
}

fn resolve_binding(raw: &RawDomain, environment: &str) -> Result<Binding, String> {
    let environment = Environment::parse(environment)?;
    if environment == Environment::Local {
        return Ok(Binding {
            host: String::new(),
            zone: String::new(),
            environment,
            visibility: Visibility::Private,
            target_ref: "default".to_owned(),
            ttl: DEFAULT_TTL,
            tls: TlsMode::Disabled,
            allow_wildcard: false,
            dns: BTreeMap::new(),
        });
    }

    let override_values = raw.environments.get(environment.as_str());
    let host_value = override_values
        .and_then(|values| values.domain.as_deref())
        .unwrap_or(raw.domain.as_str());
    let hostname = Hostname::parse(host_value).map_err(|e| e.to_string())?;
    let zone = override_values
        .and_then(|values| values.zone.as_deref())
        .or(raw.zone.as_deref())
        .unwrap_or_else(|| hostname.registered_domain())
        .trim()
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if zone.is_empty() {
        return Err("domain.zone must not be empty".to_owned());
    }

    let visibility = match override_values.and_then(|values| values.visibility.as_deref()) {
        Some(value) => Visibility::parse(value)?,
        None => Visibility::parse(&raw.visibility)?,
    };
    let ttl = override_values
        .and_then(|values| values.ttl)
        .unwrap_or(raw.ttl);
    if !(MIN_TTL..=MAX_TTL).contains(&ttl) {
        return Err(format!(
            "domain.ttl must be between {MIN_TTL} and {MAX_TTL} seconds, got {ttl}"
        ));
    }
    let tls = match override_values.and_then(|values| values.tls.as_deref()) {
        Some(value) => TlsMode::parse(value)?,
        None => TlsMode::parse(&raw.tls)?,
    };
    let allow_wildcard = override_values
        .and_then(|values| values.allow_wildcard)
        .unwrap_or(raw.allow_wildcard);
    let is_wildcard = hostname.kind() == HostnameKind::Wildcard;
    if is_wildcard && !(allow_wildcard && tls == TlsMode::ManagedDns01) {
        return Err(format!(
            "domain '{}' is a wildcard: it requires allow_wildcard = true and \
             tls = \"managed_dns01\"",
            hostname.ascii()
        ));
    }
    if visibility == Visibility::Private && tls == TlsMode::ManagedHttp01 {
        // Private hosts cannot complete a public HTTP-01 challenge.
        return Err(
            "private visibility requires tls = \"disabled\" or \"managed_dns01\"".to_owned(),
        );
    }

    let target = override_values
        .and_then(|values| values.target.as_ref())
        .unwrap_or(&raw.target);
    let target_ref = target.ref_value.trim().to_owned();
    if target_ref.is_empty() {
        return Err("domain.target.ref must not be empty".to_owned());
    }
    match target.kind.trim().to_ascii_lowercase().as_str() {
        "service" => {}
        "link" => {
            return Err(
                "target.kind = \"link\" is specified by RFC-0033 but not implemented in this \
                 build; use \"service\" (no external URL is fetched)"
                    .to_owned(),
            )
        }
        other => {
            return Err(format!(
                "unknown target.kind '{other}'; expected service (link is not implemented)"
            ))
        }
    }

    let dns = override_values
        .and_then(|values| values.dns.as_ref())
        .unwrap_or(&raw.dns)
        .clone();

    Ok(Binding {
        host: hostname.ascii().to_owned(),
        zone,
        environment,
        visibility,
        target_ref,
        ttl,
        tls,
        allow_wildcard,
        dns,
    })
}

/// Loads and validates the `\[domain\]` binding. Missing required fields are
/// clear errors; no default host, zone or record set is ever fabricated.
pub(crate) fn load_plan(
    manifest_path: &Path,
    env_override: Option<&str>,
) -> Result<DnsPlan, String> {
    let text = fs::read_to_string(manifest_path).map_err(|e| {
        format!(
            "could not read project manifest '{}': {e}. Add a [domain] table as documented in \
             docs/dns/DOMAIN_LOOP.md",
            manifest_path.display()
        )
    })?;
    let manifest: AgManifest = toml::from_str(&text).map_err(|e| {
        format!(
            "invalid project manifest '{}': {e}",
            manifest_path.display()
        )
    })?;
    let domain = manifest.domain.ok_or_else(|| {
        "manifest is missing the [domain] table; refusing to guess a binding".to_owned()
    })?;
    let project = manifest
        .project
        .and_then(|project| project.name)
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| {
            "manifest is missing [project].name (project identifier); refusing to guess".to_owned()
        })?;
    let environment = env_override
        .map(str::to_owned)
        .unwrap_or_else(|| domain.environment.clone());
    let binding = resolve_binding(&domain, &environment)?;

    let mut records = Vec::new();
    for (type_name, entries) in &binding.dns {
        let record_type = parse_record_type(type_name).map_err(|e| format!("domain.dns: {e}"))?;
        for entry in entries {
            validate_record_name(&entry.name, binding.allow_wildcard)
                .map_err(|e| format!("domain.dns.{type_name}: {e}"))?;
            validate_record_value(&record_type, &entry.value)
                .map_err(|e| format!("domain.dns.{type_name}: {e}"))?;
            records.push(PlannedRecord {
                record_type: record_type.clone(),
                name: expand_record_name(&entry.name, &binding.zone),
                value: entry.value.trim().to_owned(),
                ttl: binding.ttl,
                is_ownership: false,
            });
        }
    }

    // Fail closed: a private binding never emits public routing records.
    if binding.visibility == Visibility::Private {
        records.retain(|record| {
            !matches!(
                record.record_type,
                RecordType::A | RecordType::Aaaa | RecordType::Cname
            )
        });
    }
    records.sort_by(|a, b| {
        (a.record_type.to_string(), &a.name, &a.value, a.ttl).cmp(&(
            b.record_type.to_string(),
            &b.name,
            &b.value,
            b.ttl,
        ))
    });
    records.dedup();

    let mut notes = Vec::new();
    if binding.visibility == Visibility::Private {
        notes.push(
            "Private binding: no public A/AAAA/CNAME record is planned. Use a private link or a \
             local hosts entry."
                .to_owned(),
        );
    }
    if binding.environment == Environment::Local {
        notes.push(
            "Local environment: loopback only; the domain plan is intentionally empty.".to_owned(),
        );
    }

    let mut plan = DnsPlan {
        plan_id: String::new(),
        domain: binding.host.clone(),
        zone: binding.zone.clone(),
        environment: binding.environment,
        visibility: binding.visibility,
        project,
        service: binding.target_ref.clone(),
        target_kind: "service".to_owned(),
        tls: binding.tls,
        records,
        notes,
    };

    // Deterministic ownership token derived from the pre-ownership plan. It is
    // an identity marker for the plan, not a secret and not a real proof.
    if !binding.host.is_empty() {
        let pre_id = fnv1a64(plan.canonical_stream().as_bytes());
        plan.records.push(PlannedRecord {
            record_type: RecordType::Txt,
            name: format!("_ag-domain.{}", binding.host),
            value: format!("ag-verification={pre_id:016x}"),
            ttl: binding.ttl,
            is_ownership: true,
        });
        plan.records.sort_by(|a, b| {
            (a.record_type.to_string(), &a.name, &a.value, a.ttl).cmp(&(
                b.record_type.to_string(),
                &b.name,
                &b.value,
                b.ttl,
            ))
        });
    }
    plan.plan_id = format!(
        "dns-{}-{:016x}",
        plan.environment.as_str(),
        fnv1a64(plan.canonical_stream().as_bytes())
    );
    Ok(plan)
}

// ============================================================
// Resolved local configuration
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AppliedPlan {
    plan_id: String,
    domain: String,
    zone: String,
    environment: Environment,
    visibility: Visibility,
    project: String,
    service: String,
    target_kind: String,
    tls: TlsMode,
    records: Vec<PlannedRecord>,
}

impl From<&DnsPlan> for AppliedPlan {
    fn from(plan: &DnsPlan) -> Self {
        Self {
            plan_id: plan.plan_id.clone(),
            domain: plan.domain.clone(),
            zone: plan.zone.clone(),
            environment: plan.environment,
            visibility: plan.visibility,
            project: plan.project.clone(),
            service: plan.service.clone(),
            target_kind: plan.target_kind.clone(),
            tls: plan.tls,
            records: plan.records.clone(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct DnsState {
    version: u32,
    #[serde(default)]
    applied: BTreeMap<String, AppliedPlan>,
}

fn load_state(path: &Path) -> Result<DnsState, String> {
    if !path.exists() {
        return Ok(DnsState {
            version: STATE_VERSION,
            applied: BTreeMap::new(),
        });
    }
    let text = fs::read_to_string(path)
        .map_err(|e| format!("could not read state '{}': {e}", path.display()))?;
    let state: DnsState = serde_json::from_str(&text)
        .map_err(|e| format!("invalid state file '{}': {e}", path.display()))?;
    if state.version != STATE_VERSION {
        return Err(format!(
            "unsupported state version {} in '{}'; expected {}",
            state.version,
            path.display(),
            STATE_VERSION
        ));
    }
    Ok(state)
}

fn save_state(path: &Path, state: &DnsState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("could not create '{}': {e}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    let temp = path.with_extension("json.tmp");
    fs::write(&temp, format!("{text}\n"))
        .map_err(|e| format!("could not write '{}': {e}", temp.display()))?;
    fs::rename(&temp, path).map_err(|e| format!("could not commit '{}': {e}", path.display()))?;
    Ok(())
}

// ============================================================
// Provider adapter (explicit opt-in only)
// ============================================================

/// Local-only summary of the optional adapter step. Never contains a
/// credential, a provider zone id or any adapter response body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProviderSummary {
    pub(crate) created: usize,
    pub(crate) updated: usize,
    pub(crate) unchanged: usize,
}

async fn apply_provider(
    plan: &DnsPlan,
    provider: &str,
    zone_id: Option<&str>,
) -> Result<Option<ProviderSummary>, String> {
    if provider == "none" {
        return Ok(None);
    }
    if provider != "cloudflare" {
        return Err(format!(
            "provider '{provider}' is not available in this build; supported opt-in providers: \
             none, cloudflare"
        ));
    }

    let token = std::env::var("AG_CLOUDFLARE_TOKEN").map_err(|_| {
        "provider 'cloudflare' requires AG_CLOUDFLARE_TOKEN in the environment; no credential is \
         read from the manifest"
            .to_owned()
    })?;
    let zone_id = zone_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            "provider 'cloudflare' requires --zone-id or AG_DNS_ZONE_ID; no provider zone id is \
             written to any generated file"
                .to_owned()
        })?;
    if token.is_empty() {
        return Err("AG_CLOUDFLARE_TOKEN is empty".to_owned());
    }

    let adapter = ag_domains::provider::cloudflare::CloudflareProvider::new(token);
    let existing = adapter
        .list_records(zone_id)
        .await
        .map_err(|e| format!("provider list_records failed: {e}"))?;

    let mut summary = ProviderSummary {
        created: 0,
        updated: 0,
        unchanged: 0,
    };
    for record in &plan.records {
        let spec = DnsRecordSpec {
            name: record.name.clone(),
            record_type: record.record_type.clone(),
            content: record.value.clone(),
            ttl: record.ttl,
            proxied: false,
        };
        let unchanged = existing.iter().any(|current| {
            current.name == spec.name
                && current.record_type == spec.record_type
                && current.content == spec.content
                && current.ttl == spec.ttl
        });
        if unchanged {
            summary.unchanged += 1;
            continue;
        }
        let changed = existing
            .iter()
            .find(|current| current.name == spec.name && current.record_type == spec.record_type);
        match changed {
            Some(current) => {
                adapter
                    .update_record(zone_id, &current.id, &spec)
                    .await
                    .map_err(|e| format!("provider update_record failed: {e}"))?;
                summary.updated += 1;
            }
            None => {
                adapter
                    .create_record(zone_id, &spec)
                    .await
                    .map_err(|e| format!("provider create_record failed: {e}"))?;
                summary.created += 1;
            }
        }
    }
    Ok(Some(summary))
}

// ============================================================
// Command implementations
// ============================================================

pub(crate) fn print_plan(plan: &DnsPlan) {
    println!("Plan:        {}", plan.plan_id);
    println!("Domain:      {}", plan.domain);
    println!("Zone:        {}", plan.zone);
    println!("Environment: {}", plan.environment.as_str());
    println!("Visibility:  {}", plan.visibility.as_str());
    println!("Project:     {}", plan.project);
    println!("Service:     {}", plan.service);
    println!("Target:      {}", plan.target_kind);
    println!("TLS:         {}", plan.tls.as_str());
    println!("\nRecords ({}):", plan.records.len());
    println!("Type   Name                              Value                                 TTL  Ownership");
    for record in &plan.records {
        println!(
            "{:<6} {:<33} {:<36} {:<4} {}",
            record.record_type.to_string(),
            record.name,
            record.value,
            record.ttl,
            if record.is_ownership { "yes" } else { "no" }
        );
    }
    if plan.records.is_empty() {
        println!("(none declared)");
    }
    for note in &plan.notes {
        println!("\nNote: {note}");
    }
}

pub(crate) fn cmd_dns_plan(manifest: &Path, env: Option<&str>) -> Result<DnsPlan, String> {
    let plan = load_plan(manifest, env)?;
    print_plan(&plan);
    Ok(plan)
}

/// Applies a previously reviewed plan. The id is mandatory and is checked
/// against a fresh computation before anything is written or transmitted.
pub(crate) async fn cmd_dns_apply(
    manifest: &Path,
    env: Option<&str>,
    state: &Path,
    plan_id: &str,
    provider: &str,
    zone_id: Option<&str>,
) -> Result<(DnsPlan, Option<ProviderSummary>), String> {
    let plan = load_plan(manifest, env)?;
    if plan_id.trim().is_empty() {
        return Err("--plan is required; run 'ag dns plan' first".to_owned());
    }
    if plan.plan_id != plan_id.trim() {
        return Err(format!(
            "plan drift: requested '{}' but the manifest now resolves to '{}'; re-run \
             'ag dns plan' and review the result",
            plan_id.trim(),
            plan.plan_id
        ));
    }

    // Provider configuration is checked before any local write so a failed
    // opt-in cannot leave a state file that claims a successful apply.
    let summary = apply_provider(&plan, provider, zone_id).await?;

    let mut state_doc = load_state(state)?;
    state_doc.applied.insert(
        plan.environment.as_str().to_owned(),
        AppliedPlan::from(&plan),
    );
    save_state(state, &state_doc)?;

    println!("Applied plan {} for '{}'.", plan.plan_id, plan.domain);
    println!("Environment: {}", plan.environment.as_str());
    println!("Visibility:  {}", plan.visibility.as_str());
    println!("Records:     {}", plan.records.len());
    match summary {
        Some(summary) => println!(
            "Provider:    {} (created {}, updated {}, unchanged {})",
            provider, summary.created, summary.updated, summary.unchanged
        ),
        None => println!("Provider:    none (local configuration only; no network access)"),
    }
    println!("State:       {}", state.display());
    Ok((plan, summary))
}

/// Result of `dns status`, shared by prose and the versioned JSON shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DnsStatus {
    pub(crate) environment: Environment,
    pub(crate) applied: bool,
    pub(crate) plan_id: Option<String>,
    pub(crate) domain: Option<String>,
    pub(crate) visibility: Option<Visibility>,
    pub(crate) provider: Option<String>,
    pub(crate) record_count: usize,
    pub(crate) in_sync: bool,
}

pub(crate) fn resolve_status(
    manifest: &Path,
    env: Option<&str>,
    state: &Path,
) -> Result<DnsStatus, String> {
    let environment = match env {
        Some(value) => Environment::parse(value)?,
        None if manifest.exists() => {
            let text = fs::read_to_string(manifest)
                .map_err(|e| format!("could not read '{}': {e}", manifest.display()))?;
            let parsed: AgManifest = toml::from_str(&text)
                .map_err(|e| format!("invalid manifest '{}': {e}", manifest.display()))?;
            match parsed.domain {
                Some(domain) => Environment::parse(&domain.environment)?,
                None => Environment::Production,
            }
        }
        None => Environment::Production,
    };
    let state_doc = load_state(state)?;
    let applied = state_doc.applied.get(environment.as_str());
    let current_id = if manifest.exists() {
        load_plan(manifest, env).ok().map(|plan| plan.plan_id)
    } else {
        None
    };

    Ok(DnsStatus {
        environment,
        applied: applied.is_some(),
        plan_id: applied.map(|entry| entry.plan_id.clone()),
        domain: applied.map(|entry| entry.domain.clone()),
        visibility: applied.map(|entry| entry.visibility),
        provider: applied.map(|_| "none".to_owned()),
        record_count: applied.map_or(0, |entry| entry.records.len()),
        in_sync: match (applied, &current_id) {
            (Some(entry), Some(id)) => entry.plan_id == *id,
            _ => false,
        },
    })
}

pub(crate) fn cmd_dns_status(
    manifest: &Path,
    env: Option<&str>,
    state: &Path,
) -> Result<DnsStatus, String> {
    let status = resolve_status(manifest, env, state)?;
    println!("Environment: {}", status.environment.as_str());
    if status.applied {
        println!("Plan:        {}", status.plan_id.as_deref().unwrap_or(""));
        println!("Domain:      {}", status.domain.as_deref().unwrap_or(""));
        println!(
            "Visibility:  {}",
            status.visibility.map_or("", Visibility::as_str)
        );
        println!("Provider:    {}", status.provider.as_deref().unwrap_or(""));
        println!("Records:     {}", status.record_count);
        println!("In sync:     {}", if status.in_sync { "yes" } else { "no" });
    } else {
        println!(
            "Not applied for '{}'. Run 'ag dns plan' then 'ag dns apply --plan ID'.",
            status.environment.as_str()
        );
    }
    Ok(status)
}

/// Host and environment from a manifest, used by `ag domains attach`.
pub(crate) fn manifest_attachment_hints(
    manifest: &Path,
    env: Option<&str>,
) -> Result<(String, String, String, String), String> {
    let plan = load_plan(manifest, env)?;
    Ok((
        plan.domain,
        plan.environment.as_str().to_owned(),
        plan.service,
        plan.project,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn manifest(binding: &str) -> String {
        format!(
            "[project]\nname = \"demo\"\n\n[domain]\ndomain = \"api.example.com\"\nenvironment = \
             \"staging\"\nvisibility = \"private\"\ntls = \"disabled\"\ntarget = {{ kind = \
             \"service\", ref = \"api\" }}\n\n{binding}"
        )
    }

    fn write_manifest(dir: &Path, text: &str) -> PathBuf {
        let path = dir.join("ag.toml");
        fs::write(&path, text).expect("write manifest");
        path
    }

    fn scratch(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ag-dns-unit-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("scratch");
        dir
    }

    fn record_set(entries: &str) -> String {
        format!("[domain.dns]\n{entries}")
    }

    #[test]
    fn environment_aliases_are_canonicalized() {
        assert_eq!(Environment::parse("dev").unwrap(), Environment::Dev);
        assert_eq!(Environment::parse("DEVELOPMENT").unwrap(), Environment::Dev);
        assert_eq!(Environment::parse("stage").unwrap(), Environment::Staging);
        assert_eq!(Environment::parse("prod").unwrap(), Environment::Production);
        assert_eq!(
            Environment::parse("production").unwrap(),
            Environment::Production
        );
        assert_eq!(Environment::parse("local").unwrap(), Environment::Local);
        assert!(Environment::parse("qa").is_err());
    }

    #[test]
    fn visibility_defaults_to_private_and_parses_explicitly() {
        assert_eq!(Visibility::parse("private").unwrap(), Visibility::Private);
        assert_eq!(Visibility::parse("PUBLIC").unwrap(), Visibility::Public);
        assert!(Visibility::parse("internal").is_err());
        assert_eq!(default_visibility(), "private");
    }

    #[test]
    fn plan_id_is_deterministic() {
        let dir = scratch("deterministic");
        let path = write_manifest(
            &dir,
            &manifest(&record_set(
                "TXT = [{ name = \"_app\", value = \"v=1\" }]\n",
            )),
        );
        let first = load_plan(&path, None).expect("first plan");
        let second = load_plan(&path, None).expect("second plan");
        assert_eq!(first.plan_id, second.plan_id);
        assert!(first.plan_id.starts_with("dns-staging-"));
        assert_eq!(first.plan_id.len(), "dns-staging-".len() + 16);
    }

    #[test]
    fn ownership_record_is_deterministic_and_not_a_secret() {
        let dir = scratch("ownership");
        let path = write_manifest(&dir, &manifest(""));
        let plan = load_plan(&path, None).expect("plan");
        let ownership: Vec<_> = plan
            .records
            .iter()
            .filter(|record| record.is_ownership)
            .collect();
        assert_eq!(ownership.len(), 1);
        assert_eq!(ownership[0].name, "_ag-domain.api.example.com");
        assert!(ownership[0].value.starts_with("ag-verification="));
    }

    #[test]
    fn records_are_sorted_and_deduplicated() {
        let dir = scratch("sorting");
        let path = write_manifest(
            &dir,
            &manifest(&record_set(
                "TXT = [{ name = \"a\", value = \"z\" }, { name = \"a\", value = \"z\" }]\n",
            )),
        );
        let plan = load_plan(&path, None).expect("plan");
        let explicit: Vec<_> = plan.records.iter().filter(|r| !r.is_ownership).collect();
        assert_eq!(explicit.len(), 1);
        assert_eq!(explicit[0].name, "a.example.com");
    }

    #[test]
    fn wildcard_records_are_rejected_by_default() {
        let dir = scratch("wildcard");
        let path = write_manifest(
            &dir,
            &manifest(&record_set(
                "CNAME = [{ name = \"*.example.com\", value = \"edge.example.net\" }]\n",
            )),
        );
        let error = load_plan(&path, None).expect_err("wildcard must be rejected");
        assert!(error.contains("allow_wildcard"), "{error}");
    }

    #[test]
    fn wildcard_domain_requires_dns01_opt_in() {
        let dir = scratch("wildcard-domain");
        let base = "[project]\nname = \"demo\"\n\n[domain]\ndomain = \"*.example.com\"\n\
                    environment = \"prod\"\nallow_wildcard = true\n";
        let path = write_manifest(&dir, base);
        let error = load_plan(&path, None).expect_err("DNS-01 required");
        assert!(error.contains("managed_dns01"), "{error}");
        fs::write(&path, format!("{base}tls = \"managed_dns01\"\n")).expect("rewrite");
        assert!(load_plan(&path, None).is_ok());
    }

    #[test]
    fn private_binding_drops_public_routing_records() {
        let dir = scratch("visibility");
        let path = write_manifest(
            &dir,
            &manifest(&record_set(
                "A = [{ name = \"@\", value = \"192.0.2.10\" }]\nTXT = [{ name = \"note\", \
                 value = \"v=ag-private\" }]\n",
            )),
        );
        let plan = load_plan(&path, None).expect("plan");
        assert!(!plan
            .records
            .iter()
            .any(|record| record.record_type == RecordType::A));
        assert!(plan
            .records
            .iter()
            .any(|record| record.record_type == RecordType::Txt && !record.is_ownership));
        assert!(plan.notes.iter().any(|note| note.contains("Private")));
    }

    #[test]
    fn public_binding_keeps_routing_records() {
        let dir = scratch("public");
        let text = manifest(&record_set(
            "A = [{ name = \"@\", value = \"192.0.2.10\" }]\n",
        ))
        .replace("visibility = \"private\"", "visibility = \"public\"");
        let path = write_manifest(&dir, &text);
        let plan = load_plan(&path, None).expect("plan");
        assert!(plan
            .records
            .iter()
            .any(|record| record.record_type == RecordType::A));
    }

    #[test]
    fn environment_override_is_applied() {
        let dir = scratch("override");
        let text = format!(
            "{}\n[domain.environments.production]\nvisibility = \"public\"\ndomain = \
             \"app.example.com\"\n",
            manifest("")
        );
        let path = write_manifest(&dir, &text);
        let staging = load_plan(&path, None).expect("staging");
        assert_eq!(staging.domain, "api.example.com");
        let production = load_plan(&path, Some("prod")).expect("production");
        assert_eq!(production.domain, "app.example.com");
        assert_eq!(production.environment, Environment::Production);
        assert_eq!(production.visibility, Visibility::Public);
    }

    #[test]
    fn local_environment_produces_an_empty_domain_plan() {
        let dir = scratch("local");
        let path = write_manifest(&dir, &manifest(""));
        let plan = load_plan(&path, Some("local")).expect("local plan");
        assert!(plan.domain.is_empty());
        assert!(plan.records.is_empty());
        assert!(plan.notes.iter().any(|note| note.contains("Local")));
    }

    #[test]
    fn ttl_bounds_are_enforced() {
        let dir = scratch("ttl");
        let text = manifest("").replace("tls = \"disabled\"", "tls = \"disabled\"\nttl = 30");
        let path = write_manifest(&dir, &text);
        assert!(load_plan(&path, None)
            .expect_err("ttl too low")
            .contains("between 60 and 86400"));
    }

    #[test]
    fn invalid_ips_types_and_unknown_fields_fail_clearly() {
        let dir = scratch("invalid");
        let path = write_manifest(
            &dir,
            &manifest(&record_set(
                "A = [{ name = \"@\", value = \"not-an-ip\" }]\n",
            )),
        );
        assert!(load_plan(&path, None)
            .expect_err("bad IP")
            .contains("not a valid IP address"));

        let path = write_manifest(
            &dir,
            &manifest(&record_set("NS = [{ name = \"@\", value = \"x\" }]\n")),
        );
        assert!(load_plan(&path, None)
            .expect_err("bad type")
            .contains("unsupported record type"));

        let path = write_manifest(
            &dir,
            "[project]\nname = \"demo\"\n\n[domain]\ndomain = \"example.com\"\nzone_id = \
             \"CANARY-ZONE\"\ntoken = \"CANARY-TOKEN\"\n",
        );
        assert!(load_plan(&path, None)
            .expect_err("unknown fields")
            .contains("unknown field"));
    }

    #[test]
    fn missing_project_or_domain_is_an_error_not_a_guess() {
        let dir = scratch("missing");
        let path = write_manifest(&dir, "[domain]\ndomain = \"example.com\"\n");
        assert!(load_plan(&path, None)
            .expect_err("no project")
            .contains("missing [project].name"));
        let path = write_manifest(&dir, "[project]\nname = \"demo\"\n");
        assert!(load_plan(&path, None)
            .expect_err("no domain table")
            .contains("missing the [domain] table"));
    }
}
