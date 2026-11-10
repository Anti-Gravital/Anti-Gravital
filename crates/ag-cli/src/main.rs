//! Unified `ag` binary of the Anti-Gravital ecosystem.
//!
//! Phase 2 commands:
//! - `ag new <name> [--template rest|realtime|fullstack]`
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

mod cli;
mod commands;
mod interactive;
mod request;
#[cfg(feature = "tui")]
mod tui;

use clap::Parser;
use cli::*;
use commands::domains::*;
use commands::mail::*;
use commands::project::*;
use commands::schema::*;
use commands::workers::*;
use std::process;

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::New { name, template } => {
            match cmd_new(name, template.as_deref(), cli.no_interactive) {
                Ok(()) => Ok(()),
                Err(NewError::Failed(msg)) => Err(msg),
                Err(NewError::Usage(msg)) => {
                    eprintln!("error: {msg}");
                    process::exit(2);
                }
            }
        }
        Commands::Dev { bind } => cmd_dev(&bind),
        Commands::Build { target } => cmd_build(target.as_deref()),
        Commands::Generate { schema, output } => cmd_generate(&schema, &output),
        Commands::Workers { command } => match command {
            WorkersCommands::List { schema } => cmd_workers_list(&schema),
            #[cfg(feature = "workers-runtime")]
            WorkersCommands::Run => cmd_workers_run(),
            #[cfg(feature = "workers-runtime")]
            other => {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("failed to create the Tokio runtime");
                rt.block_on(cmd_workers_runtime(other))
            }
        },
        Commands::Schema { command } => match command {
            SchemaCommands::Lint { schema } => cmd_schema_lint(&schema),
            SchemaCommands::Diff { reference, schema } => cmd_schema_diff(&schema, &reference),
        },
        Commands::Domains { command } => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to create the Tokio runtime");
            match command {
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
            }
        }
        Commands::Mail { command } => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to create the Tokio runtime");
            match command {
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
            }
        }
    };
    if let Err(msg) = result {
        eprintln!("error: {msg}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn command_tree_matches_the_documented_public_surface() {
        let command = Cli::command();
        let root_commands: Vec<_> = command
            .get_subcommands()
            .map(|subcommand| subcommand.get_name())
            .collect();
        assert_eq!(
            root_commands,
            ["new", "dev", "build", "generate", "schema", "domains", "mail", "workers",]
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

    #[test]
    fn complete_flags_never_prompt_even_in_a_terminal() {
        let resolved = resolve_new_request(
            Some("api".into()),
            Some(Template::Realtime),
            &terminal_session(),
            must_not_ask,
        );
        assert_eq!(
            resolved,
            Ok(Resolved::FromFlags(NewProjectRequest {
                name: "api".into(),
                template: Template::Realtime,
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
                template: Template::Rest,
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
                    template: Template::Fullstack,
                }))
            },
        );
        assert_eq!(
            resolved,
            Ok(Resolved::Asked(NewProjectRequest {
                name: "api".into(),
                template: Template::Fullstack,
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
        for template in Template::ALL {
            let asked = NewProjectRequest {
                name: "my-api_2".into(),
                template,
            };
            let command = asked.equivalent_command();
            let cli = Cli::try_parse_from(command.split_whitespace()).expect("parse");
            let Commands::New { name, template } = cli.command else {
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
