//! Versioned JSON dispatch for read-only `ag` commands (RFC-0021).
//!
//! Moved from `main.rs` without changing any emitted document or exit code.

use crate::cli::*;
use crate::commands::config::get_data;
use crate::commands::deploy::{plan_report, status_report, DeployTarget};
use crate::commands::dns::{load_plan, resolve_status};
use crate::commands::env::{check_data, list_data, EnvRequest};
use crate::commands::schema::{diff_report, lint_report};
use crate::commands::workers::workers_report;
use crate::output;
use crate::output::CONTRACT_VERSION;
use crate::project_root;
use std::path::Path;

/// Builds `dns.plan` data without printing prose. The loader is the single
/// source of truth for both output paths, so they cannot drift apart.
fn cmd_dns_json_plan(manifest: &Path, env: Option<&str>) -> Result<output::DnsPlanData, String> {
    let plan = load_plan(manifest, env)?;
    Ok(output::DnsPlanData {
        plan_id: plan.plan_id,
        domain: plan.domain,
        zone: plan.zone,
        environment: plan.environment.as_str().to_owned(),
        visibility: plan.visibility.as_str().to_owned(),
        project: plan.project,
        service: plan.service,
        target_kind: plan.target_kind.to_owned(),
        tls: plan.tls.as_str().to_owned(),
        records: plan
            .records
            .into_iter()
            .map(|record| output::DnsRecordView {
                record_type: record.record_type.to_string(),
                name: record.name,
                value: record.value,
                ttl: record.ttl,
                is_ownership: record.is_ownership,
            })
            .collect(),
        notes: plan.notes,
    })
}

/// Builds `dns.status` data without printing prose.
fn cmd_dns_json_status(
    manifest: &Path,
    env: Option<&str>,
    state: &Path,
) -> Result<output::DnsStatusData, String> {
    let status = resolve_status(manifest, env, state)?;
    Ok(output::DnsStatusData {
        environment: status.environment.as_str().to_owned(),
        applied: status.applied,
        plan_id: status.plan_id,
        domain: status.domain,
        visibility: status.visibility.map(|value| value.as_str().to_owned()),
        provider: status.provider,
        record_count: status.record_count,
        in_sync: status.in_sync,
    })
}

/// Runs a read-only command in JSON mode and returns its exit code.
///
/// Commands without a versioned shape are refused *before* dispatch, so no
/// mutating executor is ever reached. The refusal itself is a versioned JSON
/// error; no secret, argument value or filesystem content is echoed.
pub(crate) fn run_json_command(command: &Commands) -> i32 {
    let Some(name) = command.json_command_name() else {
        output::emit_error(
            "unknown",
            "unsupported_command",
            format!(
                "--output json is not supported by this command in contract version \
                 {CONTRACT_VERSION}; supported commands: config.get, schema.lint, \
                 schema.diff, workers.list, deploy.plan, deploy.status, dns.plan, \
                 dns.status, env.list, env.check"
            ),
        );
        return 2;
    };

    match command {
        Commands::Config {
            command: Some(ConfigCommands::Get { key }),
        } => match get_data(key) {
            Ok(data) => {
                output::emit(name, data);
                0
            }
            Err(message) => {
                output::emit_error(name, "read_failed", message);
                1
            }
        },
        Commands::Schema {
            command: SchemaCommands::Lint { schema },
        } => match lint_report(schema) {
            Ok(data) => {
                let exit_code = if data.result == "errors" { 1 } else { 0 };
                output::emit(name, data);
                exit_code
            }
            Err(message) => {
                output::emit_error(name, "read_failed", message);
                1
            }
        },
        Commands::Schema {
            command: SchemaCommands::Diff { reference, schema },
        } => match diff_report(schema, reference) {
            Ok(data) => {
                output::emit(name, data);
                0
            }
            Err(message) => {
                output::emit_error(name, "read_failed", message);
                1
            }
        },
        Commands::Workers {
            command: WorkersCommands::List { schema },
        } => match workers_report(schema) {
            Ok(data) => {
                output::emit(name, data);
                0
            }
            Err(message) => {
                output::emit_error(name, "read_failed", message);
                1
            }
        },
        Commands::Dns {
            command: DnsCommands::Plan { manifest, env },
        } => match cmd_dns_json_plan(manifest, env.as_deref()) {
            Ok(data) => {
                output::emit(name, data);
                0
            }
            Err(message) => {
                output::emit_error(name, "read_failed", message);
                1
            }
        },
        Commands::Dns {
            command:
                DnsCommands::Status {
                    manifest,
                    state,
                    env,
                },
        } => match cmd_dns_json_status(manifest, env.as_deref(), state) {
            Ok(data) => {
                output::emit(name, data);
                0
            }
            Err(message) => {
                output::emit_error(name, "read_failed", message);
                1
            }
        },
        Commands::Env {
            command: EnvCommands::List { manifest, env },
        } => {
            let request = EnvRequest::List {
                manifest: manifest.clone(),
                environment: env.clone(),
            };
            match list_data(&request) {
                Ok(data) => {
                    output::emit(name, data);
                    0
                }
                Err(error) => {
                    output::emit_error(name, "read_failed", error.into_message());
                    1
                }
            }
        }
        Commands::Env {
            command: EnvCommands::Check { manifest, env },
        } => {
            let request = EnvRequest::Check {
                manifest: manifest.clone(),
                environment: env.clone(),
            };
            match check_data(&request) {
                Ok(data) => {
                    let exit_code = if data.ok { 0 } else { 1 };
                    output::emit(name, data);
                    exit_code
                }
                Err(error) => {
                    output::emit_error(name, "read_failed", error.into_message());
                    1
                }
            }
        }
        Commands::Deploy {
            command: DeployCommands::Plan { target },
        } => {
            let Some(target) = DeployTarget::parse(target) else {
                output::emit_error(name, "read_failed", "invalid deployment target");
                return 1;
            };
            match plan_report(&project_root(), target) {
                Ok(data) => {
                    output::emit(name, data);
                    0
                }
                Err(message) => {
                    output::emit_error(name, "read_failed", message);
                    1
                }
            }
        }
        Commands::Deploy {
            command: DeployCommands::Status,
        } => match status_report(&project_root()) {
            Ok(data) => {
                output::emit(name, data);
                0
            }
            Err(message) => {
                output::emit_error(name, "read_failed", message);
                1
            }
        },
        // Unreachable: `json_command_name` returns None for every other arm.
        _ => {
            output::emit_error(name, "unsupported_command", "command has no JSON shape");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(dir: &std::path::Path) -> std::path::PathBuf {
        let path = dir.join("ag.toml");
        std::fs::write(
            &path,
            "[project]\nname = \"demo\"\n\n[domain]\ndomain = \"api.example.com\"\n\
             environment = \"production\"\nvisibility = \"private\"\ntls = \"disabled\"\n\
             target = { kind = \"service\", ref = \"api\" }\n",
        )
        .expect("manifest fixture");
        path
    }

    #[test]
    fn json_mode_refuses_commands_without_a_shape_before_dispatch() {
        let rc = run_json_command(&Commands::Build { target: None });
        assert_eq!(rc, 2, "mutating command must be refused in JSON mode");

        let rc = run_json_command(&Commands::Config {
            command: Some(ConfigCommands::Set {
                key: "bind".to_owned(),
                value: "127.0.0.1:9000".to_owned(),
            }),
        });
        assert_eq!(rc, 2, "config set must be refused in JSON mode");

        let rc = run_json_command(&Commands::Config {
            command: Some(ConfigCommands::Validate),
        });
        assert_eq!(rc, 2, "a command without a published shape must be refused");
    }

    #[test]
    fn dns_plan_uses_the_same_loader_as_prose_output() {
        let dir = std::env::temp_dir().join(format!("ag-json-mode-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("fixture dir");
        let path = manifest(&dir);
        let data = cmd_dns_json_plan(&path, Some("production")).expect("plan data");
        assert!(!data.plan_id.is_empty());
        assert_eq!(data.visibility, "private");
        assert_eq!(data.environment, "production");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
