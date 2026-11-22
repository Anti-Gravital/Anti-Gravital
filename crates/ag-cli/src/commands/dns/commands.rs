//! `ag dns` command implementations: plan, apply, status and attach hints.

use std::fs;
use std::path::Path;

use super::manifest::load_plan;
use super::model::*;
use super::provider::{apply_provider, ProviderSummary};
use super::state::{load_state, save_state, AppliedPlan};

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
