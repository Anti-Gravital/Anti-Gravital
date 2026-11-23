//! Single dispatch table for the flag path and home-screen commands.
//!
//! Moved from `main.rs` without changing any command routing or exit class.

use crate::cli::*;
use crate::commands::config::*;
use crate::commands::deploy::*;
use crate::commands::describe::*;
use crate::commands::dns::*;
use crate::commands::domains::*;
use crate::commands::env::*;
use crate::commands::mail::*;
use crate::commands::project::*;
use crate::commands::schema::*;
use crate::commands::workers::*;
use crate::{project_root, CliError};

/// Single dispatch table for the flag path and for commands launched by the
/// home screen. Every branch converts a command error into an exit class.
pub(crate) fn dispatch(command: Commands, no_interactive: bool) -> Result<(), CliError> {
    match command {
        Commands::New {
            name,
            template,
            with,
        } => {
            cmd_new(name, template.as_deref(), &with, no_interactive).map_err(|error| match error {
                NewError::Failed(message) => CliError::Failed(message),
                NewError::Usage(message) => CliError::Usage(message),
            })
        }
        Commands::Env { command } => {
            let request = match command {
                EnvCommands::List { manifest, env } => EnvRequest::List {
                    manifest,
                    environment: env,
                },
                EnvCommands::Check { manifest, env } => EnvRequest::Check {
                    manifest,
                    environment: env,
                },
            };
            cmd_env(&request).map_err(|error| match error {
                EnvCommandError::Failed(message) => CliError::Failed(message),
                EnvCommandError::Usage(message) => CliError::Usage(message),
            })
        }
        Commands::Config { command } => {
            cmd_config(command, no_interactive).map_err(|error| match error {
                ConfigCommandError::Failed(message) => CliError::Failed(message),
                ConfigCommandError::Usage(message) => CliError::Usage(message),
            })
        }
        Commands::Dev { bind } => cmd_dev(&bind).map_err(CliError::Failed),
        Commands::Build { target } => cmd_build(target.as_deref()).map_err(CliError::Failed),
        Commands::Generate {
            schema,
            output,
            check,
            ..
        } => {
            if check {
                cmd_generate_check(&schema, &output).map_err(CliError::Failed)
            } else {
                cmd_generate(&schema, &output).map_err(CliError::Failed)
            }
        }
        Commands::Workers { command } => match command {
            WorkersCommands::List { schema } => cmd_workers_list(&schema).map_err(CliError::Failed),
            #[cfg(feature = "workers-runtime")]
            WorkersCommands::Run => cmd_workers_run().map_err(CliError::Failed),
            #[cfg(feature = "workers-runtime")]
            other => {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("failed to create the Tokio runtime");
                rt.block_on(cmd_workers_runtime(other))
                    .map_err(CliError::Failed)
            }
        },
        Commands::Schema { command } => match command {
            SchemaCommands::Lint { schema } => cmd_schema_lint(&schema).map_err(CliError::Failed),
            SchemaCommands::Diff { reference, schema } => {
                cmd_schema_diff(&schema, &reference).map_err(CliError::Failed)
            }
        },
        Commands::Dns { command } => match command {
            DnsCommands::Plan { manifest, env } => {
                cmd_dns_plan(&manifest, env.as_deref()).map(|_| ())
            }
            DnsCommands::Apply {
                manifest,
                env,
                plan,
                state,
                provider,
                zone_id,
            } => {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("failed to create the Tokio runtime");
                rt.block_on(cmd_dns_apply(
                    &manifest,
                    env.as_deref(),
                    &state,
                    &plan,
                    &provider,
                    zone_id.as_deref(),
                ))
                .map(|_| ())
            }
            DnsCommands::Status {
                manifest,
                state,
                env,
            } => cmd_dns_status(&manifest, env.as_deref(), &state).map(|_| ()),
        }
        .map_err(CliError::Failed),
        Commands::Domains { command } => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to create the Tokio runtime");
            let result = match command {
                DomainsCommands::Check {
                    domain,
                    expected,
                    min_confirmed,
                } => rt.block_on(cmd_domains_check(
                    &domain,
                    expected.as_deref(),
                    min_confirmed,
                )),
                DomainsCommands::Sync {
                    schema,
                    zone_id,
                    token,
                } => rt.block_on(cmd_domains_sync(&schema, &zone_id, &token)),
                DomainsCommands::Attach {
                    domain,
                    project,
                    env,
                    service,
                    manifest,
                    allow_wildcard,
                    edge_host,
                    ips,
                    state,
                } => rt.block_on(cmd_domains_attach(
                    domain.as_deref(),
                    project.as_deref(),
                    env.as_deref(),
                    service.as_deref(),
                    manifest.as_deref(),
                    allow_wildcard,
                    edge_host.as_deref(),
                    &ips,
                    &state,
                )),
                DomainsCommands::Instructions {
                    domain,
                    edge_host,
                    ips,
                    state,
                } => rt.block_on(cmd_domains_instructions(&domain, &edge_host, &ips, &state)),
                DomainsCommands::ExportZone {
                    domain,
                    edge_host,
                    ips,
                    state,
                } => rt.block_on(cmd_domains_export_zone(&domain, &edge_host, &ips, &state)),
                DomainsCommands::Status { domain, state } => {
                    rt.block_on(cmd_domains_status(&domain, &state))
                }
                DomainsCommands::List { state } => rt.block_on(cmd_domains_list(&state)),
                DomainsCommands::Verify {
                    domain,
                    min_confirmed,
                    state,
                } => rt.block_on(cmd_domains_verify(&domain, min_confirmed, &state)),
                DomainsCommands::Detach {
                    domain,
                    tombstone_days,
                    state,
                } => rt.block_on(cmd_domains_detach(&domain, tombstone_days, &state)),
                DomainsCommands::Diagnose {
                    domain,
                    edge_host,
                    ips,
                    state,
                } => rt.block_on(cmd_domains_diagnose(&domain, &edge_host, &ips, &state)),
            };
            result.map_err(CliError::Failed)
        }
        Commands::Mail { command } => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to create the Tokio runtime");
            let result = match command {
                MailCommands::Test {
                    to,
                    from,
                    subject,
                    smtp_host,
                    smtp_port,
                    smtp_user,
                    smtp_pass,
                } => rt.block_on(cmd_mail_test(
                    &to, &from, &subject, &smtp_host, smtp_port, smtp_user, smtp_pass,
                )),
            };
            result.map_err(CliError::Failed)
        }
        Commands::Deploy { command } => match command {
            DeployCommands::Plan { target } => {
                let Some(target) = DeployTarget::parse(&target) else {
                    return Err(CliError::Usage("invalid deployment target".to_owned()));
                };
                cmd_deploy_plan(&project_root(), target)
            }
            DeployCommands::Apply { plan } => cmd_deploy_apply(&project_root(), &plan),
            DeployCommands::Status => cmd_deploy_status(&project_root()),
        }
        .map_err(|error| match error {
            DeployError::Usage(message) => CliError::Usage(message),
            DeployError::Failed(message) => CliError::Failed(message),
        }),
        Commands::Describe => cmd_describe(&project_root()).map_err(CliError::Failed),
    }
}
