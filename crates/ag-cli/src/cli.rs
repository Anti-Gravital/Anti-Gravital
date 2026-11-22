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

    #[command(subcommand)]
    pub(crate) command: Commands,
}

impl Commands {
    /// Resolves the format selector, accounting for `ag generate`, whose
    /// directory flag owns the `--output` long name in that subcommand.
    pub(crate) fn effective_output_format(&self, global: OutputFormat) -> OutputFormat {
        match self {
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
            Commands::Schema {
                command: SchemaCommands::Lint { .. },
            } => Some("schema.lint"),
            Commands::Schema {
                command: SchemaCommands::Diff { .. },
            } => Some("schema.diff"),
            Commands::Workers {
                command: WorkersCommands::List { .. },
            } => Some("workers.list"),
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
        /// wizard asks for it. In non-interactive sessions it defaults to
        /// "rest".
        #[arg(long, short = 't', value_parser = ["rest", "realtime", "fullstack", "layered"])]
        template: Option<String>,
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
    },

    /// Operations on the DSL schema.
    Schema {
        #[command(subcommand)]
        command: SchemaCommands,
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
}

/// Sub-commands of `ag schema`.
#[derive(Subcommand)]
pub(crate) enum SchemaCommands {
    /// Verifies the schema and reports best-practice warnings.
    Lint {
        /// DSL schema file.
        #[arg(long, default_value = "schema.ag")]
        schema: PathBuf,
    },
    /// Shows breaking vs non-breaking changes against a reference file.
    Diff {
        /// Reference file (another schema.ag, snapshot, etc.).
        reference: PathBuf,
        /// Current schema file.
        #[arg(long, default_value = "schema.ag")]
        schema: PathBuf,
    },
}

/// Sub-commands of `ag workers`.
#[derive(Subcommand)]
pub(crate) enum WorkersCommands {
    /// Lists the background workers declared in a schema (no infrastructure needed).
    List {
        /// DSL schema file.
        #[arg(long, default_value = "schema.ag")]
        schema: PathBuf,
    },

    /// Explains where worker execution lives (hosted by your application binary).
    #[cfg(feature = "workers-runtime")]
    Run,

    /// Enqueues a job onto a durable queue (requires DATABASE_URL).
    ///
    /// The payload file must contain the already-encoded payload bytes the target
    /// handler expects; the CLI stores them verbatim.
    #[cfg(feature = "workers-runtime")]
    Enqueue {
        /// Job kind (must match a handler registered by the worker process).
        kind: String,
        /// Queue to enqueue onto.
        #[arg(long, default_value = "default")]
        queue: String,
        /// File with the encoded payload bytes.
        #[arg(long)]
        payload: PathBuf,
        /// Payload schema version recorded on the job.
        #[arg(long, default_value = "1")]
        payload_version: u16,
    },

    /// Lists queues seen on the durable backend with their depths (requires DATABASE_URL).
    #[cfg(feature = "workers-runtime")]
    Queues,

    /// Dead-letter queue inspection and management (requires DATABASE_URL).
    #[cfg(feature = "workers-runtime")]
    Dlq {
        #[command(subcommand)]
        command: DlqCommands,
    },

    /// Checks configuration and durable-backend connectivity (requires DATABASE_URL).
    #[cfg(feature = "workers-runtime")]
    Doctor,
}

/// Sub-commands of `ag workers dlq`.
#[cfg(feature = "workers-runtime")]
#[derive(Subcommand)]
pub(crate) enum DlqCommands {
    /// Lists dead-letter entries, newest first.
    List {
        /// Restrict to a single queue.
        #[arg(long)]
        queue: Option<String>,
        /// Maximum number of entries to show.
        #[arg(long, default_value = "50")]
        limit: i64,
    },
    /// Shows a single dead-letter entry by job id.
    Inspect {
        /// Job id (UUID).
        job_id: String,
    },
    /// Re-enqueues dead-letters onto their queue (resets attempts).
    ///
    /// Single mode: pass a JOB_ID. Bulk mode: pass `--queue` (with optional
    /// `--kind`/`--limit`) to re-drive many entries of one queue at once.
    Retry {
        /// Job id (UUID) for a single re-drive. Omit when using `--queue`.
        job_id: Option<String>,
        /// Bulk re-drive: re-enqueue dead-letters of this queue.
        #[arg(long)]
        queue: Option<String>,
        /// Narrow a bulk re-drive to a single job kind.
        #[arg(long)]
        kind: Option<String>,
        /// Maximum entries a bulk re-drive affects.
        #[arg(long, default_value = "100")]
        limit: i64,
        /// Preview what a bulk re-drive would affect without changing anything.
        #[arg(long)]
        dry_run: bool,
    },
    /// Deletes dead-letters by age and/or filter.
    ///
    /// Age mode: `--older-than` purges across all queues. Filter mode: add
    /// `--queue` (with optional `--kind`) to purge one queue; age composes.
    Purge {
        /// Age threshold; entries older than this are removed.
        #[arg(long, default_value = "30d")]
        older_than: String,
        /// Bulk purge restricted to this queue (composes with `--older-than`).
        #[arg(long)]
        queue: Option<String>,
        /// Narrow a bulk purge to a single job kind.
        #[arg(long)]
        kind: Option<String>,
        /// Maximum entries a bulk purge affects.
        #[arg(long, default_value = "1000")]
        limit: i64,
        /// Preview what a bulk purge would affect without changing anything.
        #[arg(long)]
        dry_run: bool,
    },
}

/// Sub-commands of `ag domains`.
#[derive(Subcommand)]
pub(crate) enum DomainsCommands {
    /// Verifies the propagation of a TXT record across public resolvers.
    ///
    /// Does not require DNS credentials. Useful for verifying that the
    /// SPF/DKIM/DMARC records are visible from the Internet before enabling DMARC.
    ///
    /// Reads the AG_CLOUDFLARE_TOKEN environment variable if --sync is used.
    Check {
        /// Domain to verify (e.g., example.com).
        #[arg(long)]
        domain: String,
        /// Expected TXT value (if omitted, only shows the current records).
        #[arg(long)]
        expected: Option<String>,
        /// Minimum number of resolvers that must confirm (default 2).
        #[arg(long, default_value = "2")]
        min_confirmed: usize,
    },
    /// Applies the SPF/DKIM/DMARC records via the DNS provider.
    ///
    /// Reads the configuration from the project's schema.ag and applies the
    /// records with an idempotent upsert. Requires AG_CLOUDFLARE_TOKEN.
    Sync {
        /// DSL schema file.
        #[arg(long, default_value = "schema.ag")]
        schema: PathBuf,
        /// Zone ID of the domain in the DNS provider.
        #[arg(long, env = "AG_DNS_ZONE_ID")]
        zone_id: String,
        /// DNS provider token (defaults to reading AG_CLOUDFLARE_TOKEN).
        #[arg(long, env = "AG_CLOUDFLARE_TOKEN")]
        token: String,
    },
    /// Attaches an external domain to a project and prints DNS instructions.
    ///
    /// Native and manual: writes a local attachment store and produces the
    /// exact records to paste at your DNS provider. No credentials required.
    Attach {
        /// Hostname to attach (e.g. example.com, api.example.com, *.example.com).
        domain: String,
        /// Project identifier.
        #[arg(long)]
        project: String,
        /// Environment identifier.
        #[arg(long, default_value = "production")]
        env: String,
        /// Target service reference.
        #[arg(long, default_value = "default")]
        service: String,
        /// Edge CNAME target host for subdomains/wildcards.
        #[arg(long, env = "AG_EDGE_HOST")]
        edge_host: String,
        /// Apex IP address (repeatable; A/AAAA records).
        #[arg(long = "ip")]
        ips: Vec<String>,
        /// Path to the local attachment store.
        #[arg(long, default_value = ".ag/domains.json")]
        state: PathBuf,
    },
    /// Reprints DNS instructions for an attached domain.
    Instructions {
        /// Attached hostname.
        domain: String,
        /// Edge CNAME target host.
        #[arg(long, env = "AG_EDGE_HOST")]
        edge_host: String,
        /// Apex IP address (repeatable).
        #[arg(long = "ip")]
        ips: Vec<String>,
        /// Path to the local attachment store.
        #[arg(long, default_value = ".ag/domains.json")]
        state: PathBuf,
    },
    /// Exports a BIND zone-file fragment for an attached domain.
    ExportZone {
        /// Attached hostname.
        domain: String,
        /// Edge CNAME target host.
        #[arg(long, env = "AG_EDGE_HOST")]
        edge_host: String,
        /// Apex IP address (repeatable).
        #[arg(long = "ip")]
        ips: Vec<String>,
        /// Path to the local attachment store.
        #[arg(long, default_value = ".ag/domains.json")]
        state: PathBuf,
    },
    /// Shows the status of an attached domain.
    Status {
        /// Attached hostname.
        domain: String,
        /// Path to the local attachment store.
        #[arg(long, default_value = ".ag/domains.json")]
        state: PathBuf,
    },
    /// Lists all attached domains in the local store.
    List {
        /// Path to the local attachment store.
        #[arg(long, default_value = ".ag/domains.json")]
        state: PathBuf,
    },
    /// Verifies the TXT ownership record across public resolvers.
    Verify {
        /// Attached hostname.
        domain: String,
        /// Minimum number of resolvers that must confirm.
        #[arg(long, default_value = "1")]
        min_confirmed: usize,
        /// Path to the local attachment store.
        #[arg(long, default_value = ".ag/domains.json")]
        state: PathBuf,
    },
    /// Detaches a domain, stops renewal and writes a takeover tombstone.
    Detach {
        /// Attached hostname.
        domain: String,
        /// Tombstone duration in days (subdomain-takeover prevention).
        #[arg(long, default_value = "30")]
        tombstone_days: u64,
        /// Path to the local attachment store.
        #[arg(long, default_value = ".ag/domains.json")]
        state: PathBuf,
    },
    /// Compares expected vs observed DNS records and reports findings.
    ///
    /// Queries public resolvers for the records this attachment expects and
    /// reports what is missing, wrong, or conflicting. Read-only; no credentials.
    Diagnose {
        /// Attached hostname.
        domain: String,
        /// Edge CNAME target host.
        #[arg(long, env = "AG_EDGE_HOST")]
        edge_host: String,
        /// Apex IP address (repeatable).
        #[arg(long = "ip")]
        ips: Vec<String>,
        /// Path to the local attachment store.
        #[arg(long, default_value = ".ag/domains.json")]
        state: PathBuf,
    },
}

/// Sub-commands of `ag mail`.
#[derive(Subcommand)]
pub(crate) enum MailCommands {
    /// Sends a test email to verify the SMTP configuration.
    ///
    /// Reads the configuration from environment variables:
    /// AG_SMTP_HOST, AG_SMTP_PORT, AG_SMTP_USER, AG_SMTP_PASS.
    Test {
        /// Recipient of the test email.
        #[arg(long)]
        to: String,
        /// Sender.
        #[arg(long, env = "AG_MAIL_FROM", default_value = "test@localhost")]
        from: String,
        /// Subject of the test email.
        #[arg(long, default_value = "ag mail test: Anti-Gravital")]
        subject: String,
        /// SMTP host.
        #[arg(long, env = "AG_SMTP_HOST", default_value = "localhost")]
        smtp_host: String,
        /// SMTP port.
        #[arg(long, env = "AG_SMTP_PORT", default_value = "587")]
        smtp_port: u16,
        /// SMTP user (optional).
        #[arg(long, env = "AG_SMTP_USER")]
        smtp_user: Option<String>,
        /// SMTP password (optional).
        #[arg(long, env = "AG_SMTP_PASS")]
        smtp_pass: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn dev_bind_defaults_to_loopback() {
        let cli = Cli::parse_from(["ag", "dev"]);
        match cli.command {
            Commands::Dev { bind } => assert_eq!(bind, "127.0.0.1:8080"),
            _ => panic!("expected the dev command"),
        }
    }

    #[test]
    fn dev_bind_can_opt_in_to_all_interfaces() {
        let cli = Cli::parse_from(["ag", "dev", "--bind", "0.0.0.0:8080"]);
        match cli.command {
            Commands::Dev { bind } => assert_eq!(bind, "0.0.0.0:8080"),
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
            assert_eq!(cli.command.json_command_name(), Some("schema.lint"));
        }
    }

    #[test]
    fn output_rejects_unknown_formats() {
        assert!(Cli::try_parse_from(["ag", "--output", "yaml", "workers", "list"]).is_err());
    }

    #[test]
    fn only_read_commands_have_a_versioned_json_shape() {
        let cases = [
            (vec!["ag", "schema", "lint"], Some("schema.lint")),
            (vec!["ag", "schema", "diff", "ref.ag"], Some("schema.diff")),
            (vec!["ag", "workers", "list"], Some("workers.list")),
            (vec!["ag", "new", "demo"], None),
            (vec!["ag", "domains", "list"], None),
            (vec!["ag", "mail", "test", "--to", "a@example.com"], None),
        ];
        for (args, expected) in cases {
            let cli = Cli::try_parse_from(args.clone()).expect("parse");
            assert_eq!(cli.command.json_command_name(), expected, "{args:?}");
        }
    }

    #[test]
    fn generate_keeps_its_legacy_output_directory_flag() {
        let cli = Cli::try_parse_from(["ag", "generate", "--output", "out"]).expect("parse");
        match &cli.command {
            Commands::Generate { output, .. } => assert_eq!(output, &PathBuf::from("out")),
            _ => panic!("expected the generate command"),
        }
        assert_eq!(cli.output, OutputFormat::Prose);
    }

    #[test]
    fn generate_exposes_the_format_selector_as_output_format() {
        let cli =
            Cli::try_parse_from(["ag", "generate", "--output-format", "json"]).expect("parse");
        assert_eq!(
            cli.command.effective_output_format(cli.output),
            OutputFormat::Json
        );
        assert_eq!(cli.command.json_command_name(), None);
    }

    #[test]
    fn generate_accepts_the_global_selector_before_the_subcommand() {
        let cli = Cli::try_parse_from(["ag", "--output", "json", "generate"]).expect("parse");
        assert_eq!(
            cli.command.effective_output_format(cli.output),
            OutputFormat::Json
        );
    }
}
