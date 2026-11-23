//! Command-line definitions of the `ag` binary (clap).

use crate::output::OutputFormat;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(
    name = "ag",
    version = VERSION,
    about = "Anti-Gravital CLI - create and manage AG projects",
    long_about = None,
)]
pub(crate) struct Cli {
    /// Never prompt or open the interactive interface. Missing required
    /// input becomes an error. Implied when the session is not a terminal,
    /// when `CI` or `AG_NO_TUI` is set, or when `TERM` is `dumb`.
    #[arg(long, global = true)]
    pub(crate) no_interactive: bool,

    /// Selects the output format. `json` is available only for read-only
    /// commands with a versioned shape documented under `docs/mcp/`.
    /// Mutating commands refuse it before doing any work.
    #[arg(long, global = true, value_enum, default_value = "prose")]
    pub(crate) output: OutputFormat,

    /// Omitted in an interactive terminal: opens the home screen (stage C).
    /// In every other case it prints help, as in earlier releases.
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
}

impl Commands {
    /// Resolves the format selector, accounting for `ag generate`, whose
    /// directory flag owns the `--output` long name in that subcommand.
    pub(crate) fn effective_output_format(&self, global: OutputFormat) -> OutputFormat {
        match self {
            // `--check` has no versioned JSON shape and must exit on drift, not
            // on the format selector. Its report is always prose (RFC-0019 PR 4).
            Commands::Generate { check: true, .. } => OutputFormat::Prose,
            Commands::Generate { output_format, .. } if global == OutputFormat::Prose => {
                *output_format
            }
            _ => global,
        }
    }

    /// Versioned JSON command name, or `None` when this command has no
    /// versioned shape yet. Only read-only commands may return `Some`.
    pub(crate) fn json_command_name(&self) -> Option<&'static str> {
        match self {
            Commands::Config {
                command: Some(ConfigCommands::Get { .. }),
            } => Some("config.get"),
            Commands::Schema {
                command: SchemaCommands::Lint { .. },
            } => Some("schema.lint"),
            Commands::Schema {
                command: SchemaCommands::Diff { .. },
            } => Some("schema.diff"),
            Commands::Workers {
                command: WorkersCommands::List { .. },
            } => Some("workers.list"),
            Commands::Deploy {
                command: DeployCommands::Plan { .. },
            } => Some("deploy.plan"),
            Commands::Deploy {
                command: DeployCommands::Status,
            } => Some("deploy.status"),
            Commands::Dns {
                command: DnsCommands::Plan { .. },
            } => Some("dns.plan"),
            Commands::Dns {
                command: DnsCommands::Status { .. },
            } => Some("dns.status"),
            Commands::Env {
                command: EnvCommands::List { .. },
            } => Some("env.list"),
            Commands::Env {
                command: EnvCommands::Check { .. },
            } => Some("env.check"),
            Commands::Describe => Some("project.describe"),
            _ => None,
        }
    }
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Creates a new Anti-Gravital project.
    ///
    /// Generates the file structure from the chosen template
    /// and substitutes the project name across all files.
    New {
        /// Name of the new project (also used as the directory name). When
        /// omitted in an interactive terminal, a guided wizard asks for it.
        name: Option<String>,

        /// Starting template. When omitted in an interactive terminal, the
        /// selector asks for it. In non-interactive sessions it defaults to
        /// "rest". Every name in templates/CATALOG.md is accepted; unknown
        /// names are a usage error.
        #[arg(long, short = 't', value_parser = crate::catalog::TEMPLATE_NAMES)]
        template: Option<String>,

        /// Enables an additional Anti-Gravital module in the generated
        /// ag.toml \[modules\] table. Repeatable. Requires a template that
        /// generates a manifest (for example `--template layered`); the
        /// default scaffold is unchanged when this flag is absent.
        #[arg(long, value_parser = crate::request::MODULE_NAMES)]
        with: Vec<String>,
    },

    /// Starts the server in development mode with hot reload.
    ///
    /// Requires `cargo-watch` to be installed (`cargo install cargo-watch`).
    /// If it is not available, runs `cargo run` without automatic reloading.
    Dev {
        /// Server listen address. Defaults to loopback; pass
        /// `--bind 0.0.0.0:8080` explicitly to listen on all interfaces.
        #[arg(long, default_value = "127.0.0.1:8080")]
        bind: String,
    },

    /// Compiles the project in release mode.
    Build {
        /// Cross-compilation triple (e.g.: `x86_64-unknown-linux-musl`).
        #[arg(long)]
        target: Option<String>,
    },

    /// Views, edits and validates the project's config.toml (RFC-0018).
    ///
    /// With no subcommand it opens a full-screen form in an interactive
    /// terminal. `get`, `set` and `validate` are the non-interactive parity.
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },

    /// Reads and validates the project's declared environment (RFC-0020).
    ///
    /// Read-only: the declarations come from the `[env.*]` tables of ag.toml.
    /// No encrypted file is read, no value is ever printed, and nothing is
    /// written. Stage B resolves the process environment and declared defaults
    /// only; environment-specific files are a later stage.
    Env {
        #[command(subcommand)]
        command: EnvCommands,
    },

    /// Generates artifacts (Rust, SQL, TypeScript, OpenAPI) from schema.ag.
    ///
    /// Reads the given DSL file (by default `schema.ag` in the current
    /// directory), compiles the schema and writes the generated artifacts to
    /// the output directory.
    Generate {
        /// Input DSL schema file.
        #[arg(long, default_value = "schema.ag")]
        schema: PathBuf,

        /// Directory where the generated artifacts are written.
        ///
        /// The argument id is `generate_output` so it does not collide with
        /// the global `--output` format selector (RFC-0021 section 4.3).
        #[arg(long, id = "generate_output", default_value = "generated")]
        output: PathBuf,

        /// Format selector for this subcommand. `ag generate --output DIR`
        /// predates the global flag and owns the `--output` long name here, so
        /// this variant exposes the same selector as `--output-format`. The
        /// `output` id prevents clap from propagating the global flag into
        /// this subcommand, which would create a duplicate long name.
        #[arg(
            long = "output-format",
            id = "output",
            value_enum,
            default_value = "prose"
        )]
        output_format: OutputFormat,

        /// Read-only verification (RFC-0019 PR 4). Compiles `schema.ag` and
        /// compares every generated artifact against the file on disk at
        /// `--output` without writing anything. Exits non-zero when a file is
        /// missing, differs byte-for-byte, or is unexpected; exits zero when
        /// every artifact matches. `--output-format` is kept for selector
        /// parity but the check report itself is always prose.
        #[arg(long)]
        check: bool,
    },

    /// Operations on the DSL schema.
    Schema {
        #[command(subcommand)]
        command: SchemaCommands,
    },

    /// Local, plan-first DNS configuration from the `[domain]` table of
    /// `ag.toml` (RFC-0033). No network or provider is used by default.
    Dns {
        #[command(subcommand)]
        command: DnsCommands,
    },

    /// Operations on DNS domains.
    Domains {
        #[command(subcommand)]
        command: DomainsCommands,
    },

    /// Operations on transactional mail.
    Mail {
        #[command(subcommand)]
        command: MailCommands,
    },

    /// Operations on background workers (ag-workers).
    Workers {
        #[command(subcommand)]
        command: WorkersCommands,
    },

    /// Renders deployment artifacts locally from authored Docker or Kubernetes
    /// files (RFC-0034). Plan first, apply with the returned plan id. Never
    /// contacts a cluster, container runtime, registry or network service;
    /// this is not the Phase 5 cloud deploy.
    Deploy {
        #[command(subcommand)]
        command: DeployCommands,
    },

    /// Describes the current project without changing it (RFC-0024).
    ///
    /// Read-only summary of recognized layout, artifact destinations, source
    /// paths, sizes and generated-vs-authoured state. No secret is emitted.
    Describe,
}

mod subcommands;
pub(crate) use subcommands::*;
#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn dev_bind_defaults_to_loopback() {
        let cli = Cli::parse_from(["ag", "dev"]);
        match cli.command {
            Some(Commands::Dev { bind }) => assert_eq!(bind, "127.0.0.1:8080"),
            _ => panic!("expected the dev command"),
        }
    }

    #[test]
    fn dev_bind_can_opt_in_to_all_interfaces() {
        let cli = Cli::parse_from(["ag", "dev", "--bind", "0.0.0.0:8080"]);
        match cli.command {
            Some(Commands::Dev { bind }) => assert_eq!(bind, "0.0.0.0:8080"),
            _ => panic!("expected the dev command"),
        }
    }

    // ---- `--output json` (RFC-0021 section 4.3) ----------------------------

    #[test]
    fn output_defaults_to_prose() {
        let cli = Cli::try_parse_from(["ag", "schema", "lint"]).expect("parse");
        assert_eq!(cli.output, OutputFormat::Prose);
    }

    #[test]
    fn output_json_is_accepted_before_and_after_the_subcommand() {
        for args in [
            ["ag", "--output", "json", "schema", "lint"],
            ["ag", "schema", "lint", "--output", "json"],
        ] {
            let cli = Cli::try_parse_from(args).expect("parse");
            assert_eq!(cli.output, OutputFormat::Json);
            assert_eq!(
                cli.command.as_ref().and_then(|c| c.json_command_name()),
                Some("schema.lint")
            );
        }
    }

    #[test]
    fn output_rejects_unknown_formats() {
        assert!(Cli::try_parse_from(["ag", "--output", "yaml", "workers", "list"]).is_err());
    }

    #[test]
    fn only_read_commands_have_a_versioned_json_shape() {
        let cases = [
            (vec!["ag", "config", "get", "bind"], Some("config.get")),
            (vec!["ag", "config", "set", "bind", "127.0.0.1:7000"], None),
            (vec!["ag", "config", "validate"], None),
            (vec!["ag", "config"], None),
            (vec!["ag", "schema", "lint"], Some("schema.lint")),
            (vec!["ag", "schema", "diff", "ref.ag"], Some("schema.diff")),
            (vec!["ag", "workers", "list"], Some("workers.list")),
            (
                vec!["ag", "deploy", "plan", "--target", "docker"],
                Some("deploy.plan"),
            ),
            (vec!["ag", "deploy", "status"], Some("deploy.status")),
            (
                vec![
                    "ag",
                    "deploy",
                    "apply",
                    "--plan",
                    "dpl-docker-0123456789abcdef",
                ],
                None,
            ),
            (vec!["ag", "env", "list"], Some("env.list")),
            (vec!["ag", "env", "check"], Some("env.check")),
            (vec!["ag", "describe"], Some("project.describe")),
            (vec!["ag", "new", "demo"], None),
            (vec!["ag", "domains", "list"], None),
            (vec!["ag", "mail", "test", "--to", "a@example.com"], None),
        ];
        for (args, expected) in cases {
            let cli = Cli::try_parse_from(args.clone()).expect("parse");
            assert_eq!(
                cli.command.as_ref().and_then(|c| c.json_command_name()),
                expected,
                "{args:?}"
            );
        }
    }

    #[test]
    fn generate_keeps_its_legacy_output_directory_flag() {
        let cli = Cli::try_parse_from(["ag", "generate", "--output", "out"]).expect("parse");
        match &cli.command {
            Some(Commands::Generate { output, .. }) => assert_eq!(output, &PathBuf::from("out")),
            _ => panic!("expected the generate command"),
        }
        assert_eq!(cli.output, OutputFormat::Prose);
    }

    #[test]
    fn generate_exposes_the_format_selector_as_output_format() {
        let cli =
            Cli::try_parse_from(["ag", "generate", "--output-format", "json"]).expect("parse");
        assert_eq!(
            cli.command
                .as_ref()
                .expect("command")
                .effective_output_format(cli.output),
            OutputFormat::Json
        );
        assert_eq!(
            cli.command.as_ref().and_then(|c| c.json_command_name()),
            None
        );
    }

    #[test]
    fn generate_accepts_the_global_selector_before_the_subcommand() {
        let cli = Cli::try_parse_from(["ag", "--output", "json", "generate"]).expect("parse");
        assert_eq!(
            cli.command
                .as_ref()
                .expect("command")
                .effective_output_format(cli.output),
            OutputFormat::Json
        );
    }

    #[test]
    fn generate_check_forces_the_prose_report_even_with_a_json_selector() {
        for args in [
            vec!["ag", "generate", "--check", "--output-format", "json"],
            vec!["ag", "--output", "json", "generate", "--check"],
        ] {
            let cli = Cli::try_parse_from(args.clone()).expect("parse");
            assert_eq!(
                cli.command
                    .as_ref()
                    .expect("command")
                    .effective_output_format(cli.output),
                OutputFormat::Prose,
                "{args:?}"
            );
        }
    }

    #[test]
    fn generate_check_is_absent_by_default() {
        let cli = Cli::try_parse_from(["ag", "generate"]).expect("parse");
        match cli.command {
            Some(Commands::Generate { check, .. }) => assert!(!check),
            _ => panic!("expected the generate command"),
        }
    }

    #[test]
    fn generate_check_is_accepted_before_and_after_other_flags() {
        for args in [
            vec!["ag", "generate", "--check"],
            vec!["ag", "generate", "--check", "--output", "out"],
            vec!["ag", "generate", "--output", "out", "--check"],
            vec![
                "ag",
                "generate",
                "--schema",
                "schema.ag",
                "--check",
                "--output-format",
                "prose",
            ],
        ] {
            let cli =
                Cli::try_parse_from(args.clone()).unwrap_or_else(|e| panic!("parse {args:?}: {e}"));
            match cli.command {
                Some(Commands::Generate { check, .. }) => assert!(check, "{args:?}"),
                _ => panic!("expected the generate command for {args:?}"),
            }
        }
    }

    #[test]
    fn bare_ag_parses_without_a_command() {
        let cli = Cli::try_parse_from(["ag"]).expect("parse");
        assert!(cli.command.is_none());
    }

    #[test]
    fn deploy_subcommands_parse_and_apply_requires_a_plan() {
        for args in [
            vec!["ag", "deploy", "plan", "--target", "docker"],
            vec!["ag", "deploy", "plan", "--target", "k8s"],
            vec!["ag", "deploy", "status"],
            vec![
                "ag",
                "deploy",
                "apply",
                "--plan",
                "dpl-docker-0123456789abcdef",
            ],
        ] {
            let cli = Cli::try_parse_from(args.clone()).expect("parse");
            assert!(
                matches!(cli.command, Some(Commands::Deploy { .. })),
                "{args:?}"
            );
        }
        assert!(Cli::try_parse_from(["ag", "deploy", "apply"]).is_err());
        assert!(Cli::try_parse_from(["ag", "deploy", "plan", "--target", "nomad"]).is_err());
    }

    #[test]
    fn config_subcommands_parse() {
        for args in [
            vec!["ag", "config"],
            vec!["ag", "config", "get", "bind"],
            vec!["ag", "config", "set", "bind", "127.0.0.1:7000"],
            vec!["ag", "config", "validate"],
        ] {
            let cli = Cli::try_parse_from(args.clone()).expect("parse");
            assert!(
                matches!(cli.command, Some(Commands::Config { .. })),
                "{args:?}"
            );
        }
    }

    // ---- template catalog parity -----------------------------------------

    #[test]
    fn every_catalog_name_is_accepted_by_the_value_parser() {
        for name in crate::catalog::TEMPLATE_NAMES {
            let cli = Cli::try_parse_from(["ag", "new", "demo", "--template", name])
                .unwrap_or_else(|error| panic!("catalog name '{name}' was rejected: {error}"));
            match cli.command {
                Some(Commands::New { template, .. }) => {
                    assert_eq!(template.as_deref(), Some(name), "name '{name}'");
                }
                _ => panic!("expected the new command for '{name}'"),
            }
        }
    }

    #[test]
    fn an_unknown_template_name_is_a_usage_error() {
        let error = Cli::try_parse_from(["ag", "new", "demo", "--template", "not-a-template"])
            .err()
            .expect("unknown template must fail");
        assert_eq!(error.kind(), clap::error::ErrorKind::InvalidValue);
    }

    // ---- `ag new --with MODULE` (RFC-0019 stage A) ------------------------

    #[test]
    fn every_declared_module_is_accepted_and_unknown_names_are_rejected() {
        for name in crate::request::MODULE_NAMES {
            let cli = Cli::try_parse_from(["ag", "new", "demo", "--with", name])
                .unwrap_or_else(|error| panic!("module '{name}' was rejected: {error}"));
            match cli.command {
                Some(Commands::New { with, .. }) => assert_eq!(with, vec![name.to_owned()]),
                _ => panic!("expected the new command for '{name}'"),
            }
        }
        let error = Cli::try_parse_from(["ag", "new", "demo", "--with", "not-a-module"])
            .err()
            .expect("unknown module must fail");
        assert_eq!(error.kind(), clap::error::ErrorKind::InvalidValue);
    }

    #[test]
    fn with_is_repeatable_and_absent_by_default() {
        let cli = Cli::try_parse_from([
            "ag",
            "new",
            "demo",
            "--template",
            "layered",
            "--with",
            "auth",
            "--with",
            "cache",
        ])
        .expect("parse");
        match cli.command {
            Some(Commands::New { with, .. }) => {
                assert_eq!(with, vec!["auth".to_owned(), "cache".to_owned()])
            }
            _ => panic!("expected the new command"),
        }
        let cli = Cli::try_parse_from(["ag", "new", "demo"]).expect("parse");
        match cli.command {
            Some(Commands::New { with, .. }) => assert!(with.is_empty()),
            _ => panic!("expected the new command"),
        }
    }

    // ---- `ag env list|check` (RFC-0020 stage B) ---------------------------

    #[test]
    fn env_subcommands_parse_with_manifest_and_environment() {
        for args in [
            vec!["ag", "env", "list"],
            vec!["ag", "env", "list", "--env", "prod"],
            vec!["ag", "env", "list", "--manifest", "custom.toml"],
            vec!["ag", "env", "check"],
            vec!["ag", "env", "check", "--env", "prod"],
            vec!["ag", "env", "check", "--manifest", "custom.toml"],
        ] {
            let cli = Cli::try_parse_from(args.clone()).expect("parse");
            assert!(
                matches!(cli.command, Some(Commands::Env { .. })),
                "{args:?}"
            );
        }
    }

    #[test]
    fn env_requires_a_subcommand() {
        assert!(Cli::try_parse_from(["ag", "env"]).is_err());
    }
}
