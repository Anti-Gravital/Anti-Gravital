//! Manifest loading, validation and deterministic plan generation.

use std::collections::BTreeMap;
use std::fs;
use std::net::IpAddr;
use std::path::Path;

use ag_domains::hostname::{Hostname, HostnameKind};
use ag_domains::record::RecordType;

use super::model::*;

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
