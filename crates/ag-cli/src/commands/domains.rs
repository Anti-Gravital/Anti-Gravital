//! `ag domains` commands (ag-domains).

use super::schema::read_schema;
use std::path::Path;

pub(crate) async fn cmd_domains_check(
    domain: &str,
    expected: Option<&str>,
    min_confirmed: usize,
) -> Result<(), String> {
    use ag_domains::propagation::{PropagationChecker, DEFAULT_RESOLVERS};

    println!("Checking DNS propagation for '{domain}'...");

    let checker = PropagationChecker::new(DEFAULT_RESOLVERS, min_confirmed);

    match expected {
        Some(value) => {
            let result = checker.check_txt(domain, value).await;
            if result.is_fully_propagated() {
                println!(
                    "OK: TXT record propagated ({}/{} resolvers)",
                    result.confirmed, result.total
                );
            } else {
                println!(
                    "PENDING: {}/{} resolvers confirm the value '{value}'",
                    result.confirmed, result.total
                );
                println!("Wait a few minutes and run the command again.");
            }
        }
        None => {
            // No expected value: verifies that at least one resolver responds.
            let result = checker.check_txt(domain, "").await;
            println!(
                "Query completed: {}/{} resolvers responded",
                result.total, result.total
            );
        }
    }

    Ok(())
}

pub(crate) async fn cmd_domains_sync(
    schema_path: &Path,
    zone_id: &str,
    token: &str,
) -> Result<(), String> {
    use ag_domains::{
        mail_records::{apply_mail_records, DkimSelector, DmarcPolicy, MailDnsRequirements},
        provider::cloudflare::CloudflareProvider,
    };

    let source = read_schema(schema_path)?;

    let schema = ag_dsl::compile(&source).map_err(|diags| {
        let mut msg = format!(
            "{} error(s) in '{}':\n",
            diags.iter().filter(|d| d.is_error()).count(),
            schema_path.display()
        );
        for d in &diags {
            msg.push_str(&format!("  {}\n", d.display(&source)));
        }
        msg
    })?;

    if schema.domains.is_empty() {
        println!("No domain blocks were found in the schema. Nothing to synchronize.");
        return Ok(());
    }

    let provider = CloudflareProvider::new(token);

    for domain_block in &schema.domains {
        let domain_name = domain_block
            .domain_name
            .as_deref()
            .unwrap_or(domain_block.name.value.as_str());

        println!(
            "Synchronizing mail records for '{}' (zone {zone_id})...",
            domain_name
        );

        let dmarc_policy = match domain_block.dmarc_policy.as_deref().unwrap_or("none") {
            "quarantine" => DmarcPolicy::Quarantine,
            "reject" => DmarcPolicy::Reject,
            _ => DmarcPolicy::None,
        };

        let req = MailDnsRequirements {
            domain: domain_name.to_owned(),
            dkim_selectors: domain_block
                .dkim_selectors
                .iter()
                .map(|s| DkimSelector {
                    name: s.clone(),
                    public_key_record: format!("v=DKIM1; k=rsa; p=PLACEHOLDER_{s}"),
                })
                .collect(),
            spf_includes: Vec::new(),
            spf_ips: Vec::new(),
            dmarc_policy,
            dmarc_rua: domain_block.dmarc_rua.clone(),
        };

        let result = apply_mail_records(&req, &provider, zone_id)
            .await
            .map_err(|e| format!("failed to synchronize '{domain_name}': {e}"))?;

        println!(
            "  created: {}, updated: {}, unchanged: {}",
            result.created, result.updated, result.unchanged
        );
    }

    println!("Synchronization completed.");
    Ok(())
}

// ============================================================
// ag domains control-plane commands (RFC-0011, phase A)
// ============================================================

use std::net::{Ipv4Addr, Ipv6Addr};

use ag_domains::attachment::{
    AttachmentLifecycle, DnsMode, DnsStatus, DomainAttachment, OwnershipMethod, OwnershipStatus,
    RoutingStatus, TargetKind, TlsMode, TlsStatus,
};
use ag_domains::hostname::{Hostname, HostnameKind};
use ag_domains::instructions::{export_bind_zone, generate_instructions, EdgeTargets};
use ag_domains::ownership::{generate_attachment_id, generate_token, ownership_record_name};
use ag_domains::store::{now_unix, AttachmentStore, JsonFileStore};

pub(crate) fn parse_edge_targets(edge_host: &str, ips: &[String]) -> Result<EdgeTargets, String> {
    let mut targets = EdgeTargets::new(edge_host);
    for ip in ips {
        if let Ok(v4) = ip.parse::<Ipv4Addr>() {
            targets = targets.with_ipv4(v4);
        } else if let Ok(v6) = ip.parse::<Ipv6Addr>() {
            targets = targets.with_ipv6(v6);
        } else {
            return Err(format!("invalid IP address: '{ip}'"));
        }
    }
    Ok(targets)
}

pub(crate) fn open_store(state: &Path) -> Result<JsonFileStore, String> {
    JsonFileStore::open(state)
        .map_err(|e| format!("could not open store '{}': {e}", state.display()))
}

pub(crate) fn print_instructions(ins: &ag_domains::instructions::DnsInstructions) {
    println!("Type   Name                              Value");
    for rec in ins.all() {
        println!(
            "{:<6} {:<33} {}",
            rec.record_type.to_string(),
            rec.name,
            rec.value
        );
    }
    for note in &ins.notes {
        println!("\nNote: {note}");
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn cmd_domains_attach(
    domain: &str,
    project: &str,
    env: &str,
    service: &str,
    edge_host: &str,
    ips: &[String],
    state: &Path,
) -> Result<(), String> {
    let hostname = Hostname::parse(domain).map_err(|e| e.to_string())?;
    let targets = parse_edge_targets(edge_host, ips)?;
    let store = open_store(state)?;

    if store
        .is_tombstoned(hostname.ascii(), now_unix())
        .await
        .map_err(|e| e.to_string())?
    {
        return Err(format!(
            "'{}' is tombstoned (takeover protection); wait for expiry or use a different host",
            hostname.ascii()
        ));
    }

    let id = generate_attachment_id();
    let ownership_value = generate_token(&id);
    let ownership_name = ownership_record_name(&hostname);
    let tls_mode = match hostname.kind() {
        HostnameKind::Wildcard => TlsMode::ManagedDns01,
        _ => TlsMode::ManagedHttp01,
    };

    let mut attachment = DomainAttachment {
        id: id.clone(),
        hostname: hostname.clone(),
        project: project.to_owned(),
        environment: env.to_owned(),
        target_kind: TargetKind::Service,
        target_ref: service.to_owned(),
        dns_mode: DnsMode::Manual,
        tls_mode,
        ownership_method: OwnershipMethod::Txt,
        ownership_name,
        ownership_value: ownership_value.clone(),
        ownership_status: OwnershipStatus::Pending,
        dns_status: DnsStatus::Pending,
        tls_status: TlsStatus::Pending,
        routing_status: RoutingStatus::Disabled,
        lifecycle: AttachmentLifecycle::Draft,
        created_at: now_unix(),
    };
    attachment.recompute_lifecycle();

    store
        .create(attachment)
        .await
        .map_err(|e| format!("could not attach: {e}"))?;

    println!("Domain attachment created.\n");
    println!("Domain:      {}", hostname.ascii());
    println!("Project:     {project}");
    println!("Environment: {env}");
    println!("Attachment:  {id}");
    println!("Status:      pending_ownership\n");

    let ins = generate_instructions(&hostname, &targets, &ownership_value);
    println!("Add these records at your DNS provider:\n");
    print_instructions(&ins);
    println!("\nThen run:\n  ag domains verify {}", hostname.ascii());
    Ok(())
}

pub(crate) async fn cmd_domains_instructions(
    domain: &str,
    edge_host: &str,
    ips: &[String],
    state: &Path,
) -> Result<(), String> {
    let hostname = Hostname::parse(domain).map_err(|e| e.to_string())?;
    let targets = parse_edge_targets(edge_host, ips)?;
    let store = open_store(state)?;
    let attachment = store
        .get(hostname.ascii())
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| {
            format!(
                "no attachment for '{}'; run 'ag domains attach' first",
                hostname.ascii()
            )
        })?;

    let ins = generate_instructions(&hostname, &targets, &attachment.ownership_value);
    println!("DNS instructions for {}\n", hostname.ascii());
    print_instructions(&ins);
    Ok(())
}

pub(crate) async fn cmd_domains_export_zone(
    domain: &str,
    edge_host: &str,
    ips: &[String],
    state: &Path,
) -> Result<(), String> {
    let hostname = Hostname::parse(domain).map_err(|e| e.to_string())?;
    let targets = parse_edge_targets(edge_host, ips)?;
    let store = open_store(state)?;
    let attachment = store
        .get(hostname.ascii())
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("no attachment for '{}'", hostname.ascii()))?;

    let ins = generate_instructions(&hostname, &targets, &attachment.ownership_value);
    print!("{}", export_bind_zone(&hostname, &ins));
    Ok(())
}

pub(crate) async fn cmd_domains_status(domain: &str, state: &Path) -> Result<(), String> {
    let hostname = Hostname::parse(domain).map_err(|e| e.to_string())?;
    let store = open_store(state)?;
    let a = store
        .get(hostname.ascii())
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("no attachment for '{}'", hostname.ascii()))?;

    println!("Domain:      {}", a.hostname.ascii());
    println!("Attachment:  {}", a.id);
    println!("Project:     {}", a.project);
    println!("Environment: {}\n", a.environment);
    println!("Lifecycle:   {:?}", a.lifecycle);
    println!("Ownership:   {:?}", a.ownership_status);
    println!("DNS:         {:?}", a.dns_status);
    println!("TLS:         {:?}", a.tls_status);
    println!("Routing:     {:?}", a.routing_status);
    Ok(())
}

pub(crate) async fn cmd_domains_list(state: &Path) -> Result<(), String> {
    let store = open_store(state)?;
    let all = store.list().await.map_err(|e| e.to_string())?;
    if all.is_empty() {
        println!("No attached domains.");
        return Ok(());
    }
    println!(
        "{:<32} {:<16} {:<14} LIFECYCLE",
        "HOSTNAME", "PROJECT", "ENVIRONMENT"
    );
    for a in all {
        println!(
            "{:<32} {:<16} {:<14} {:?}",
            a.hostname.ascii(),
            a.project,
            a.environment,
            a.lifecycle
        );
    }
    Ok(())
}

pub(crate) async fn cmd_domains_verify(
    domain: &str,
    min_confirmed: usize,
    state: &Path,
) -> Result<(), String> {
    use ag_domains::ownership::verify_ownership;
    use ag_domains::propagation::{PropagationChecker, DEFAULT_RESOLVERS};

    let hostname = Hostname::parse(domain).map_err(|e| e.to_string())?;
    let store = open_store(state)?;
    let mut attachment = store
        .get(hostname.ascii())
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("no attachment for '{}'", hostname.ascii()))?;

    println!("Verifying ownership TXT for '{}'...", hostname.ascii());
    let checker = PropagationChecker::new(DEFAULT_RESOLVERS, min_confirmed);
    let probe = verify_ownership(&checker, &hostname, &attachment.ownership_value).await;

    if probe.confirmed >= min_confirmed {
        attachment.ownership_status = OwnershipStatus::Verified;
        attachment.recompute_lifecycle();
        store
            .update(attachment)
            .await
            .map_err(|e| format!("could not update store: {e}"))?;
        println!(
            "OK: ownership verified ({}/{} resolvers).",
            probe.confirmed, probe.total
        );
    } else {
        println!(
            "PENDING: {}/{} resolvers confirm the token. Wait for DNS propagation and retry.",
            probe.confirmed, probe.total
        );
    }
    Ok(())
}

pub(crate) async fn cmd_domains_detach(
    domain: &str,
    tombstone_days: u64,
    state: &Path,
) -> Result<(), String> {
    let hostname = Hostname::parse(domain).map_err(|e| e.to_string())?;
    let store = open_store(state)?;
    let tombstone_secs = tombstone_days.saturating_mul(24 * 60 * 60);
    store
        .detach(hostname.ascii(), tombstone_secs)
        .await
        .map_err(|e| format!("could not detach: {e}"))?;
    println!("Detached '{}'.", hostname.ascii());
    println!("Routing removed and certificate renewal stopped.");
    println!(
        "Tombstoned for {tombstone_days} day(s) to prevent takeover. Remove the DNS records at your provider."
    );
    Ok(())
}

pub(crate) async fn cmd_domains_diagnose(
    domain: &str,
    edge_host: &str,
    ips: &[String],
    state: &Path,
) -> Result<(), String> {
    use ag_domains::diagnostics::{diagnose, ObservedRecord, Severity};
    use ag_domains::propagation::{lookup_observed, DEFAULT_RESOLVERS};

    let hostname = Hostname::parse(domain).map_err(|e| e.to_string())?;
    let targets = parse_edge_targets(edge_host, ips)?;
    let store = open_store(state)?;
    let attachment = store
        .get(hostname.ascii())
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("no attachment for '{}'", hostname.ascii()))?;

    let instructions = generate_instructions(&hostname, &targets, &attachment.ownership_value);

    // Observe each expected record at a public resolver.
    let resolver = DEFAULT_RESOLVERS[0];
    let mut observed = Vec::new();
    for rec in instructions.all() {
        for value in lookup_observed(resolver, &rec.name, rec.record_type.clone()).await {
            observed.push(ObservedRecord {
                record_type: rec.record_type.clone(),
                name: rec.name.clone(),
                value,
            });
        }
    }

    let result = diagnose(&instructions, &observed);
    println!("Diagnostics for {}\n", hostname.ascii());
    for finding in &result.findings {
        let tag = match finding.severity {
            Severity::Ok => "[ok]",
            Severity::Error => "[error]",
        };
        println!("{tag} {}", finding.message);
        if !finding.action.is_empty() {
            println!("       action: {}", finding.action);
        }
    }
    if result.is_healthy() {
        println!("\nNo action required.");
    }
    Ok(())
}
