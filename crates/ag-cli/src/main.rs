//! Unified `ag` binary of the Anti-Gravital ecosystem.
//!
//! Phase 2 commands:
//! - `ag new <name> [--template <name>]` (catalog: `templates/CATALOG.md`)
//! - `ag dev [--bind host:port]`
//! - `ag build [--target triple]`
//!
//! Phase 3 commands (DSL):
//! - `ag generate [--schema schema.ag] [--output ./generated]`
//! - `ag schema lint [--schema schema.ag]`
//! - `ag schema diff <ref> [--schema schema.ag]`
//!
//! Phase 4.5 commands (mail and domains):
//! - `ag domains check --domain example.com [--expected value] [--min-confirmed N]`
//! - `ag domains sync --schema schema.ag --zone-id ZONE --token TOKEN`
//! - `ag mail test --to dest@email.com [--from from@email.com] [--smtp-host ...]`
//!
//! Phase 4.7 commands (RFC-0018):
//! - `ag config [get KEY | set KEY VALUE | validate]`
//! - bare `ag` opens the home screen in an interactive terminal.

mod catalog;
mod cli;
mod commands;
mod config;
mod interactive;
mod output;
mod request;
mod scaffold;
#[cfg(feature = "tui")]
mod tui;

use clap::{CommandFactory, Parser};
use cli::*;
use commands::config::*;
use commands::domains::*;
use commands::mail::*;
use commands::project::*;
use commands::schema::*;
use commands::workers::*;
use output::{OutputFormat, CONTRACT_VERSION};
use std::process;

/// Runs a read-only command in JSON mode and returns its exit code.
///
/// Commands without a versioned shape are refused *before* dispatch, so no
/// mutating executor is ever reached. The refusal itself is a versioned JSON
/// error; no secret, argument value or filesystem content is echoed.
fn run_json_command(command: &Commands) -> i32 {
    let Some(name) = command.json_command_name() else {
        output::emit_error(
            "unknown",
            "unsupported_command",
            format!(
                "--output json is not supported by this command in contract version \
                 {CONTRACT_VERSION}; supported commands: schema.lint, schema.diff, workers.list"
            ),
        );
        return 2;
    };

    match command {
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
        // Unreachable: `json_command_name` returns None for every other arm.
        _ => {
            output::emit_error(name, "unsupported_command", "command has no JSON shape");
            2
        }
    }
}

#[cfg(feature = "tui")]
use interactive::Session;

/// Exit classification of a command. Usage errors use exit code 2, every
/// other failure uses exit code 1 (as in earlier releases).
enum CliError {
    Usage(String),
    Failed(String),
}

fn main() {
    let cli = Cli::parse();

    // RFC-0021 section 4.3: JSON mode is a separate output path for read-only
    // commands. The prose path below is left byte-for-byte unchanged.
    match cli.command.as_ref() {
        Some(command) if command.effective_output_format(cli.output) == OutputFormat::Json => {
            let exit_code = run_json_command(command);
            if exit_code != 0 {
                process::exit(exit_code);
            }
            return;
        }
        None if cli.output == OutputFormat::Json => {
            output::emit_error(
                "unknown",
                "unsupported_command",
                "--output json requires a command with a versioned shape",
            );
            process::exit(2);
        }
        _ => {}
    }

    let result = match cli.command {
        Some(command) => dispatch(command, cli.no_interactive),
        None => {
            #[cfg(feature = "tui")]
            {
                let session = Session::from_environment(cli.no_interactive);
                if session.is_interactive() {
                    if let Some(command_line) = tui::run_home(&session) {
                        println!("{command_line}");
                        launch(&command_line, cli.no_interactive);
                        return;
                    }
                }
            }
            Cli::command().print_help().expect("write help");
            println!();
            Ok(())
        }
    };
    handle(result);
}

/// Runs a command line selected on the home screen. The screen only returns
/// text; re-parsing through the real CLI guarantees it runs the same command
/// the user could have typed (RFC-0018 principle 1 and section 4.5).
#[cfg(feature = "tui")]
fn launch(command_line: &str, no_interactive: bool) {
    let cli = match Cli::try_parse_from(command_line.split_whitespace()) {
        Ok(cli) => cli,
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            print!("{error}");
            return;
        }
        Err(error) => {
            handle(Err(CliError::Failed(error.to_string())));
            return;
        }
    };
    let result = match cli.command {
        Some(command) => dispatch(command, no_interactive || cli.no_interactive),
        None => Ok(()),
    };
    handle(result);
}

/// Prints an error and exits with the documented code.
fn handle(result: Result<(), CliError>) {
    if let Err(error) = result {
        match error {
            CliError::Usage(message) => {
                eprintln!("error: {message}");
                process::exit(2);
            }
            CliError::Failed(message) => {
                eprintln!("error: {message}");
                process::exit(1);
            }
        }
    }
}

/// Single dispatch table for the flag path and for commands launched by the
/// home screen. Every branch converts a command error into an exit class.
fn dispatch(command: Commands, no_interactive: bool) -> Result<(), CliError> {
    match command {
        Commands::New { name, template } => cmd_new(name, template.as_deref(), no_interactive)
            .map_err(|error| match error {
                NewError::Failed(message) => CliError::Failed(message),
                NewError::Usage(message) => CliError::Usage(message),
            }),
        Commands::Config { command } => {
            cmd_config(command, no_interactive).map_err(|error| match error {
                ConfigCommandError::Failed(message) => CliError::Failed(message),
                ConfigCommandError::Usage(message) => CliError::Usage(message),
            })
        }
        Commands::Dev { bind } => cmd_dev(&bind).map_err(CliError::Failed),
        Commands::Build { target } => cmd_build(target.as_deref()).map_err(CliError::Failed),
        Commands::Generate { schema, output, .. } => {
            cmd_generate(&schema, &output).map_err(CliError::Failed)
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
                    edge_host,
                    ips,
                    state,
                } => rt.block_on(cmd_domains_attach(
                    &domain, &project, &env, &service, &edge_host, &ips, &state,
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, Parser};

    #[test]
    fn command_tree_matches_the_documented_public_surface() {
        let command = Cli::command();
        let root_commands: Vec<_> = command
            .get_subcommands()
            .map(|subcommand| subcommand.get_name())
            .collect();
        assert_eq!(
            root_commands,
            [
                "new", "dev", "build", "config", "generate", "schema", "domains", "mail",
                "workers",
            ]
        );
        assert!(!root_commands.contains(&"deploy"));
        assert!(!root_commands.contains(&"migrate"));
        assert!(!root_commands.contains(&"plugin"));

        let domains = command
            .get_subcommands()
            .find(|subcommand| subcommand.get_name() == "domains")
            .expect("domains command must exist");
        let domain_commands: Vec<_> = domains
            .get_subcommands()
            .map(|subcommand| subcommand.get_name())
            .collect();
        assert_eq!(
            domain_commands,
            [
                "check",
                "sync",
                "attach",
                "instructions",
                "export-zone",
                "status",
                "list",
                "verify",
                "detach",
                "diagnose",
            ]
        );

        let config = command
            .get_subcommands()
            .find(|subcommand| subcommand.get_name() == "config")
            .expect("config command must exist");
        let config_commands: Vec<_> = config
            .get_subcommands()
            .map(|subcommand| subcommand.get_name())
            .collect();
        assert_eq!(config_commands, ["get", "set", "validate"]);
    }

    #[test]
    fn validate_name_accepts_valid_names() {
        assert!(validate_project_name("my-api").is_ok());
        assert!(validate_project_name("my_service").is_ok());
        assert!(validate_project_name("todoapi123").is_ok());
    }

    #[test]
    fn validate_name_rejects_empty() {
        assert!(validate_project_name("").is_err());
    }

    #[test]
    fn validate_name_rejects_spaces() {
        assert!(validate_project_name("my project").is_err());
    }

    #[test]
    fn apply_template_substitutes_name() {
        let tmpl = "name = \"{{name}}\"";
        let result = apply_template(tmpl, "my-app");
        assert_eq!(result, "name = \"my-app\"");
    }

    #[test]
    fn apply_template_replaces_all_occurrences() {
        let tmpl = "{{name}} and {{name}}";
        let result = apply_template(tmpl, "alpha");
        assert_eq!(result, "alpha and alpha");
    }

    #[test]
    fn lint_fn_surfaces_warnings_for_model_without_primary() {
        let src = "model Tag { name String }";
        let diags = ag_dsl::lint(src);
        assert!(
            !diags.is_empty(),
            "a model without @primary must produce warnings"
        );
    }

    // ---- `ag new` input resolution (RFC-0018) ------------------------------

    use crate::interactive::Session;
    use crate::request::{NewProjectRequest, Template};

    fn scripted_session() -> Session {
        Session::default()
    }

    fn terminal_session() -> Session {
        Session {
            stdin_is_terminal: true,
            stdout_is_terminal: true,
            ..Session::default()
        }
    }

    fn must_not_ask(
        _: Option<String>,
        _: Option<Template>,
    ) -> Result<Option<NewProjectRequest>, String> {
        panic!("a prompt was opened in a session that must never prompt");
    }

    fn catalog_template(name: &str) -> Template {
        Template::parse(name).expect("cataloged template")
    }

    #[test]
    fn complete_flags_never_prompt_even_in_a_terminal() {
        let resolved = resolve_new_request(
            Some("api".into()),
            Some(catalog_template("realtime")),
            &terminal_session(),
            must_not_ask,
        );
        assert_eq!(
            resolved,
            Ok(Resolved::FromFlags(NewProjectRequest {
                name: "api".into(),
                template: catalog_template("realtime"),
            }))
        );
    }

    #[test]
    fn non_interactive_session_never_prompts_and_defaults_the_template() {
        let resolved =
            resolve_new_request(Some("api".into()), None, &scripted_session(), must_not_ask);
        assert_eq!(
            resolved,
            Ok(Resolved::FromFlags(NewProjectRequest {
                name: "api".into(),
                template: catalog_template("rest"),
            }))
        );
    }

    #[test]
    fn non_interactive_session_without_a_name_is_a_usage_error() {
        let resolved = resolve_new_request(None, None, &scripted_session(), must_not_ask);
        assert!(matches!(resolved, Err(NewError::Usage(_))));
    }

    #[test]
    fn no_interactive_flag_wins_over_a_terminal() {
        let session = Session {
            no_interactive_flag: true,
            ..terminal_session()
        };
        let resolved = resolve_new_request(None, None, &session, must_not_ask);
        assert!(matches!(resolved, Err(NewError::Usage(_))));
    }

    #[cfg(feature = "tui")]
    #[test]
    fn missing_input_in_a_terminal_is_asked_with_the_given_values_prefilled() {
        let resolved = resolve_new_request(
            Some("api".into()),
            None,
            &terminal_session(),
            |name, template| {
                assert_eq!(name.as_deref(), Some("api"));
                assert_eq!(template, None);
                Ok(Some(NewProjectRequest {
                    name: "api".into(),
                    template: catalog_template("fullstack"),
                }))
            },
        );
        assert_eq!(
            resolved,
            Ok(Resolved::Asked(NewProjectRequest {
                name: "api".into(),
                template: catalog_template("fullstack"),
            }))
        );
    }

    #[cfg(feature = "tui")]
    #[test]
    fn cancelling_the_interactive_flow_creates_nothing() {
        let resolved = resolve_new_request(None, None, &terminal_session(), |_, _| Ok(None));
        assert_eq!(resolved, Ok(Resolved::Cancelled));
    }

    /// Parity (RFC-0018, principle 1): the command printed after an
    /// interactive flow, parsed by the real argument parser in a session that
    /// cannot prompt, yields the same request the interactive flow built.
    #[test]
    fn printed_equivalent_command_reproduces_the_interactive_request() {
        for &template in Template::ALL {
            let asked = NewProjectRequest {
                name: "my-api_2".into(),
                template,
            };
            let command = asked.equivalent_command();
            let cli = Cli::try_parse_from(command.split_whitespace()).expect("parse");
            let Some(Commands::New { name, template }) = cli.command else {
                panic!("the equivalent command is not `ag new`: {command}");
            };
            let template = template.as_deref().and_then(Template::parse);
            let resolved = resolve_new_request(name, template, &scripted_session(), must_not_ask);
            assert_eq!(resolved, Ok(Resolved::FromFlags(asked)));
        }
    }

    #[test]
    fn no_interactive_is_accepted_before_and_after_the_subcommand() {
        for args in [
            ["ag", "--no-interactive", "new"],
            ["ag", "new", "--no-interactive"],
        ] {
            let cli = Cli::try_parse_from(args).expect("parse");
            assert!(cli.no_interactive);
        }
    }
}
