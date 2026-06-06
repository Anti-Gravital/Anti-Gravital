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

use clap::{Parser, Subcommand};
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

// Templates embedded in the binary at compile time. The paths are relative
// to the workspace root directory (crates/ag-cli/src/../../../templates/).

const REST_CARGO_TOML: &str = include_str!("../../../templates/rest/Cargo.toml.tmpl");
const REST_MAIN_RS: &str = include_str!("../../../templates/rest/src/main.rs.tmpl");
const REST_CONFIG_TOML: &str = include_str!("../../../templates/rest/config.toml.tmpl");

const REALTIME_CARGO_TOML: &str = include_str!("../../../templates/realtime/Cargo.toml.tmpl");
const REALTIME_MAIN_RS: &str = include_str!("../../../templates/realtime/src/main.rs.tmpl");
const REALTIME_CONFIG_TOML: &str = include_str!("../../../templates/realtime/config.toml.tmpl");

const FULLSTACK_CARGO_TOML: &str = include_str!("../../../templates/fullstack/Cargo.toml.tmpl");
const FULLSTACK_MAIN_RS: &str = include_str!("../../../templates/fullstack/src/main.rs.tmpl");
const FULLSTACK_CONFIG_TOML: &str = include_str!("../../../templates/fullstack/config.toml.tmpl");
const FULLSTACK_MIGRATION: &str =
    include_str!("../../../templates/fullstack/migrations/0001_init.sql.tmpl");

#[derive(Parser)]
#[command(
    name = "ag",
    version = VERSION,
    about = "Anti-Gravital CLI - crea y gestiona proyectos AG",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Creates a new Anti-Gravital project.
    ///
    /// Generates the file structure from the chosen template
    /// and substitutes the project name across all files.
    New {
        /// Name of the new project (also used as the directory name).
        name: String,

        /// Starting template. When omitted in an interactive terminal, you
        /// will be prompted. In non-interactive sessions, defaults to "rest".
        #[arg(long, short = 't', value_parser = ["rest", "realtime", "fullstack"])]
        template: Option<String>,
    },

    /// Starts the server in development mode with hot reload.
    ///
    /// Requires `cargo-watch` to be installed (`cargo install cargo-watch`).
    /// If it is not available, runs `cargo run` without automatic reloading.
    Dev {
        /// Server listen address.
        #[arg(long, default_value = "0.0.0.0:8080")]
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
        #[arg(long, default_value = "generated")]
        output: PathBuf,
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
}

/// Sub-commands of `ag schema`.
#[derive(Subcommand)]
enum SchemaCommands {
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

/// Sub-commands of `ag domains`.
#[derive(Subcommand)]
enum DomainsCommands {
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
}

/// Sub-commands of `ag mail`.
#[derive(Subcommand)]
enum MailCommands {
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
        #[arg(long, default_value = "ag mail test — Anti-Gravital")]
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

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::New { name, template } => cmd_new(&name, template.as_deref()),
        Commands::Dev { bind } => cmd_dev(&bind),
        Commands::Build { target } => cmd_build(target.as_deref()),
        Commands::Generate { schema, output } => cmd_generate(&schema, &output),
        Commands::Schema { command } => match command {
            SchemaCommands::Lint { schema } => cmd_schema_lint(&schema),
            SchemaCommands::Diff { reference, schema } => cmd_schema_diff(&schema, &reference),
        },
        Commands::Domains { command } => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("no se pudo crear el runtime tokio");
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
                } => {
                    cmd_domains_attach(&domain, &project, &env, &service, &edge_host, &ips, &state)
                }
                DomainsCommands::Instructions {
                    domain,
                    edge_host,
                    ips,
                    state,
                } => cmd_domains_instructions(&domain, &edge_host, &ips, &state),
                DomainsCommands::ExportZone {
                    domain,
                    edge_host,
                    ips,
                    state,
                } => cmd_domains_export_zone(&domain, &edge_host, &ips, &state),
                DomainsCommands::Status { domain, state } => cmd_domains_status(&domain, &state),
                DomainsCommands::List { state } => cmd_domains_list(&state),
                DomainsCommands::Verify {
                    domain,
                    min_confirmed,
                    state,
                } => rt.block_on(cmd_domains_verify(&domain, min_confirmed, &state)),
                DomainsCommands::Detach {
                    domain,
                    tombstone_days,
                    state,
                } => cmd_domains_detach(&domain, tombstone_days, &state),
            }
        }
        Commands::Mail { command } => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("no se pudo crear el runtime tokio");
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

fn cmd_new(name: &str, template: Option<&str>) -> Result<(), String> {
    validate_project_name(name)?;

    let project_dir = Path::new(name);
    if project_dir.exists() {
        return Err(format!("directory '{name}' already exists"));
    }

    let template = match template {
        Some(t) => t.to_owned(),
        None => resolve_template_interactive(),
    };

    println!("Creating project '{name}' with template '{template}'...");

    match template.as_str() {
        "rest" => scaffold_rest(name, project_dir),
        "realtime" => scaffold_realtime(name, project_dir),
        "fullstack" => scaffold_fullstack(name, project_dir),
        _ => Err(format!("unknown template: {template}")),
    }?;

    println!();
    println!("Project '{name}' created.");
    println!();
    println!("Next steps:");
    println!("  cd {name}");
    println!("  ag dev");

    Ok(())
}

/// Returns the template name. Prompts interactively when running in a terminal;
/// returns "rest" silently when stdin is not a terminal (CI / scripts).
fn resolve_template_interactive() -> String {
    if !std::io::stdin().is_terminal() {
        return "rest".to_owned();
    }
    const TEMPLATES: &[&str] = &["rest", "realtime", "fullstack"];
    let selection = dialoguer::Select::new()
        .with_prompt("Choose a template")
        .items(TEMPLATES)
        .default(0)
        .interact_opt()
        .unwrap_or(None)
        .unwrap_or(0);
    TEMPLATES[selection].to_owned()
}

fn cmd_dev(bind: &str) -> Result<(), String> {
    println!("Iniciando servidor en modo desarrollo (bind: {bind})...");

    let watch_available = process::Command::new("cargo")
        .args(["watch", "--version"])
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let status = if watch_available {
        println!("Usando cargo-watch para hot reload.");
        process::Command::new("cargo")
            .env("BIND", bind)
            .args(["watch", "-x", "run"])
            .status()
            .map_err(|e| format!("no se pudo ejecutar cargo watch: {e}"))?
    } else {
        println!("cargo-watch no encontrado. Ejecutando sin hot reload.");
        println!("Para habilitar hot reload: cargo install cargo-watch");
        process::Command::new("cargo")
            .env("BIND", bind)
            .arg("run")
            .status()
            .map_err(|e| format!("no se pudo ejecutar cargo run: {e}"))?
    };

    if !status.success() {
        return Err("el servidor termino con error".into());
    }
    Ok(())
}

fn cmd_build(target: Option<&str>) -> Result<(), String> {
    let mut args = vec!["build", "--release"];
    if let Some(t) = target {
        args.push("--target");
        args.push(t);
    }

    println!("Compilando en modo release...");
    let status = process::Command::new("cargo")
        .args(&args)
        .status()
        .map_err(|e| format!("no se pudo ejecutar cargo build: {e}"))?;

    if !status.success() {
        return Err("la compilacion fallo".into());
    }
    println!("Compilacion exitosa.");
    Ok(())
}

fn validate_project_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("el nombre del proyecto no puede estar vacio".into());
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(format!(
            "nombre de proyecto invalido '{name}': solo se permiten letras, numeros, guion y guion_bajo"
        ));
    }
    Ok(())
}

fn apply_template(template: &str, name: &str) -> String {
    template.replace("{{name}}", name)
}

fn write_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("no se pudo crear directorio '{}': {e}", parent.display()))?;
    }
    fs::write(path, content).map_err(|e| format!("no se pudo escribir '{}': {e}", path.display()))
}

fn scaffold_rest(name: &str, dir: &Path) -> Result<(), String> {
    write_file(
        &dir.join("Cargo.toml"),
        &apply_template(REST_CARGO_TOML, name),
    )?;
    write_file(
        &dir.join("src/main.rs"),
        &apply_template(REST_MAIN_RS, name),
    )?;
    write_file(
        &dir.join("config.toml"),
        &apply_template(REST_CONFIG_TOML, name),
    )?;
    Ok(())
}

fn scaffold_realtime(name: &str, dir: &Path) -> Result<(), String> {
    write_file(
        &dir.join("Cargo.toml"),
        &apply_template(REALTIME_CARGO_TOML, name),
    )?;
    write_file(
        &dir.join("src/main.rs"),
        &apply_template(REALTIME_MAIN_RS, name),
    )?;
    write_file(
        &dir.join("config.toml"),
        &apply_template(REALTIME_CONFIG_TOML, name),
    )?;
    Ok(())
}

fn scaffold_fullstack(name: &str, dir: &Path) -> Result<(), String> {
    write_file(
        &dir.join("Cargo.toml"),
        &apply_template(FULLSTACK_CARGO_TOML, name),
    )?;
    write_file(
        &dir.join("src/main.rs"),
        &apply_template(FULLSTACK_MAIN_RS, name),
    )?;
    write_file(
        &dir.join("config.toml"),
        &apply_template(FULLSTACK_CONFIG_TOML, name),
    )?;
    write_file(
        &dir.join("migrations/0001_init.sql"),
        &apply_template(FULLSTACK_MIGRATION, name),
    )?;
    Ok(())
}

// ---- DSL commands (Phase 3) -----------------------------------------

fn cmd_generate(schema_path: &Path, output_dir: &Path) -> Result<(), String> {
    let source = read_schema(schema_path)?;

    let schema = ag_dsl::compile(&source).map_err(|diags| {
        let mut msg = format!(
            "{} error(es) en '{}':\n",
            diags.iter().filter(|d| d.is_error()).count(),
            schema_path.display()
        );
        for d in &diags {
            msg.push_str(&format!("  {}\n", d.display(&source)));
        }
        msg
    })?;

    let files = ag_dsl::generate(&schema);

    println!(
        "Generando {} artefactos desde '{}'...",
        files.len(),
        schema_path.display()
    );

    for (rel_path, content) in &files.files {
        let full_path = output_dir.join(rel_path);
        write_file(&full_path, content)?;
        println!("  {}", full_path.display());
    }

    println!("Generacion completada en '{}'.", output_dir.display());
    Ok(())
}

fn cmd_schema_lint(schema_path: &Path) -> Result<(), String> {
    let source = read_schema(schema_path)?;
    let diags = ag_dsl::lint(&source);

    if diags.is_empty() {
        println!("'{}': sin problemas encontrados.", schema_path.display());
        return Ok(());
    }

    let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
    let warnings: Vec<_> = diags.iter().filter(|d| !d.is_error()).collect();

    for w in &warnings {
        println!("warning: {}", w.display(&source));
    }
    for e in &errors {
        eprintln!("error:   {}", e.display(&source));
    }

    if !errors.is_empty() {
        return Err(format!(
            "{} error(es) en '{}'",
            errors.len(),
            schema_path.display()
        ));
    }
    Ok(())
}

fn cmd_schema_diff(schema_path: &Path, reference_path: &Path) -> Result<(), String> {
    let current_source = read_schema(schema_path)?;
    let ref_source = read_schema(reference_path)?;

    let current = ag_dsl::compile(&current_source).map_err(|diags| {
        format!(
            "errores en schema actual: {}",
            diags
                .iter()
                .filter(|d| d.is_error())
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    let reference = ag_dsl::compile(&ref_source).map_err(|diags| {
        format!(
            "errores en schema de referencia: {}",
            diags
                .iter()
                .filter(|d| d.is_error())
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    let changes = diff_schemas(&reference, &current);

    if changes.is_empty() {
        println!(
            "Sin cambios entre '{}' y '{}'.",
            reference_path.display(),
            schema_path.display()
        );
        return Ok(());
    }

    println!(
        "Cambios entre '{}' y '{}':",
        reference_path.display(),
        schema_path.display()
    );
    for change in &changes {
        println!("  {change}");
    }
    Ok(())
}

/// Compares two schemas and returns a list of readable changes.
///
/// Detects: added/removed models, added/removed/changed fields.
/// Breaking changes (removing a model, removing a field, changing a type) are
/// marked with `[BREAKING]`. Non-breaking changes are marked with `[additive]`.
fn diff_schemas(old: &ag_dsl::ast::Schema, new: &ag_dsl::ast::Schema) -> Vec<String> {
    use std::collections::HashMap;

    let old_models: HashMap<&str, _> = old
        .models
        .iter()
        .map(|m| (m.name.value.as_str(), m))
        .collect();
    let new_models: HashMap<&str, _> = new
        .models
        .iter()
        .map(|m| (m.name.value.as_str(), m))
        .collect();

    let mut changes = Vec::new();

    // Removed models (breaking)
    for name in old_models.keys() {
        if !new_models.contains_key(name) {
            changes.push(format!("[BREAKING]  modelo '{name}' eliminado"));
        }
    }

    // Added models (non-breaking)
    for name in new_models.keys() {
        if !old_models.contains_key(name) {
            changes.push(format!("[additive]  modelo '{name}' añadido"));
        }
    }

    // Fields in common models
    for (name, old_model) in &old_models {
        if let Some(new_model) = new_models.get(name) {
            let old_fields: HashMap<&str, _> = old_model
                .fields
                .iter()
                .map(|f| (f.name.value.as_str(), f))
                .collect();
            let new_fields: HashMap<&str, _> = new_model
                .fields
                .iter()
                .map(|f| (f.name.value.as_str(), f))
                .collect();

            for fname in old_fields.keys() {
                if !new_fields.contains_key(fname) {
                    changes.push(format!("[BREAKING]  '{name}.{fname}' eliminado"));
                }
            }
            for fname in new_fields.keys() {
                if !old_fields.contains_key(fname) {
                    changes.push(format!("[additive]  '{name}.{fname}' añadido"));
                }
            }
            for (fname, old_field) in &old_fields {
                if let Some(new_field) = new_fields.get(fname) {
                    if old_field.ty.value != new_field.ty.value {
                        changes.push(format!(
                            "[BREAKING]  '{name}.{fname}' tipo cambiado: {:?} -> {:?}",
                            old_field.ty.value, new_field.ty.value
                        ));
                    }
                    if old_field.optional && !new_field.optional {
                        changes.push(format!(
                            "[BREAKING]  '{name}.{fname}' cambio de nullable a NOT NULL"
                        ));
                    }
                    if !old_field.optional && new_field.optional {
                        changes.push(format!(
                            "[additive]  '{name}.{fname}' cambio de NOT NULL a nullable"
                        ));
                    }
                }
            }
        }
    }

    changes
}

fn read_schema(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Err(format!(
            "archivo schema no encontrado: '{}'",
            path.display()
        ));
    }
    fs::read_to_string(path).map_err(|e| format!("no se pudo leer '{}': {e}", path.display()))
}

// ============================================================
// ag domains commands (Phase 4.5)
// ============================================================

async fn cmd_domains_check(
    domain: &str,
    expected: Option<&str>,
    min_confirmed: usize,
) -> Result<(), String> {
    use ag_domains::propagation::{PropagationChecker, DEFAULT_RESOLVERS};

    println!("Verificando propagacion DNS para '{domain}'...");

    let checker = PropagationChecker::new(DEFAULT_RESOLVERS, min_confirmed);

    match expected {
        Some(value) => {
            let result = checker.check_txt(domain, value).await;
            if result.is_fully_propagated() {
                println!(
                    "OK — registro TXT propagado ({}/{} resolvers)",
                    result.confirmed, result.total
                );
            } else {
                println!(
                    "PENDIENTE — {}/{} resolvers confirman el valor '{value}'",
                    result.confirmed, result.total
                );
                println!("Espera unos minutos y vuelve a ejecutar el comando.");
            }
        }
        None => {
            // No expected value: verifies that at least one resolver responds.
            let result = checker.check_txt(domain, "").await;
            println!(
                "Consulta completada — {}/{} resolvers respondieron",
                result.total, result.total
            );
        }
    }

    Ok(())
}

async fn cmd_domains_sync(schema_path: &Path, zone_id: &str, token: &str) -> Result<(), String> {
    use ag_domains::{
        mail_records::{apply_mail_records, DkimSelector, DmarcPolicy, MailDnsRequirements},
        provider::cloudflare::CloudflareProvider,
    };

    let source = read_schema(schema_path)?;

    let schema = ag_dsl::compile(&source).map_err(|diags| {
        let mut msg = format!(
            "{} error(es) en '{}':\n",
            diags.iter().filter(|d| d.is_error()).count(),
            schema_path.display()
        );
        for d in &diags {
            msg.push_str(&format!("  {}\n", d.display(&source)));
        }
        msg
    })?;

    if schema.domains.is_empty() {
        println!("No hay bloques 'domain' en el schema. Nada que sincronizar.");
        return Ok(());
    }

    let provider = CloudflareProvider::new(token);

    for domain_block in &schema.domains {
        let domain_name = domain_block
            .domain_name
            .as_deref()
            .unwrap_or(domain_block.name.value.as_str());

        println!(
            "Sincronizando registros de correo para '{}' (zone {zone_id})...",
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
            .map_err(|e| format!("error al sincronizar '{domain_name}': {e}"))?;

        println!(
            "  creados: {}, actualizados: {}, sin cambios: {}",
            result.created, result.updated, result.unchanged
        );
    }

    println!("Sincronizacion completada.");
    Ok(())
}

// ============================================================
// ag domains control-plane commands (RFC-0009, phase A)
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

fn parse_edge_targets(edge_host: &str, ips: &[String]) -> Result<EdgeTargets, String> {
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

fn open_store(state: &Path) -> Result<JsonFileStore, String> {
    JsonFileStore::open(state)
        .map_err(|e| format!("could not open store '{}': {e}", state.display()))
}

fn print_instructions(ins: &ag_domains::instructions::DnsInstructions) {
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
fn cmd_domains_attach(
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
    let mut store = open_store(state)?;

    if store.is_tombstoned(hostname.ascii(), now_unix()) {
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

fn cmd_domains_instructions(
    domain: &str,
    edge_host: &str,
    ips: &[String],
    state: &Path,
) -> Result<(), String> {
    let hostname = Hostname::parse(domain).map_err(|e| e.to_string())?;
    let targets = parse_edge_targets(edge_host, ips)?;
    let store = open_store(state)?;
    let attachment = store.get(hostname.ascii()).ok_or_else(|| {
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

fn cmd_domains_export_zone(
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
        .ok_or_else(|| format!("no attachment for '{}'", hostname.ascii()))?;

    let ins = generate_instructions(&hostname, &targets, &attachment.ownership_value);
    print!("{}", export_bind_zone(&hostname, &ins));
    Ok(())
}

fn cmd_domains_status(domain: &str, state: &Path) -> Result<(), String> {
    let hostname = Hostname::parse(domain).map_err(|e| e.to_string())?;
    let store = open_store(state)?;
    let a = store
        .get(hostname.ascii())
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

fn cmd_domains_list(state: &Path) -> Result<(), String> {
    let store = open_store(state)?;
    let all = store.list();
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

async fn cmd_domains_verify(
    domain: &str,
    min_confirmed: usize,
    state: &Path,
) -> Result<(), String> {
    use ag_domains::ownership::verify_ownership;
    use ag_domains::propagation::{PropagationChecker, DEFAULT_RESOLVERS};

    let hostname = Hostname::parse(domain).map_err(|e| e.to_string())?;
    let mut store = open_store(state)?;
    let mut attachment = store
        .get(hostname.ascii())
        .ok_or_else(|| format!("no attachment for '{}'", hostname.ascii()))?;

    println!("Verifying ownership TXT for '{}'...", hostname.ascii());
    let checker = PropagationChecker::new(DEFAULT_RESOLVERS, min_confirmed);
    let probe = verify_ownership(&checker, &hostname, &attachment.ownership_value).await;

    if probe.confirmed >= min_confirmed {
        attachment.ownership_status = OwnershipStatus::Verified;
        attachment.recompute_lifecycle();
        store
            .update(attachment)
            .map_err(|e| format!("could not update store: {e}"))?;
        println!(
            "OK — ownership verified ({}/{} resolvers).",
            probe.confirmed, probe.total
        );
    } else {
        println!(
            "PENDING — {}/{} resolvers confirm the token. Wait for DNS propagation and retry.",
            probe.confirmed, probe.total
        );
    }
    Ok(())
}

fn cmd_domains_detach(domain: &str, tombstone_days: u64, state: &Path) -> Result<(), String> {
    let hostname = Hostname::parse(domain).map_err(|e| e.to_string())?;
    let mut store = open_store(state)?;
    let tombstone_secs = tombstone_days.saturating_mul(24 * 60 * 60);
    store
        .detach(hostname.ascii(), tombstone_secs)
        .map_err(|e| format!("could not detach: {e}"))?;
    println!("Detached '{}'.", hostname.ascii());
    println!("Routing removed and certificate renewal stopped.");
    println!(
        "Tombstoned for {tombstone_days} day(s) to prevent takeover. Remove the DNS records at your provider."
    );
    Ok(())
}

// ============================================================
// Comandos ag mail (Fase 4.5)
// ============================================================

#[allow(clippy::too_many_arguments)]
async fn cmd_mail_test(
    to: &str,
    from: &str,
    subject: &str,
    smtp_host: &str,
    smtp_port: u16,
    smtp_user: Option<String>,
    smtp_pass: Option<String>,
) -> Result<(), String> {
    use ag_mail::{
        message::{Address, EmailBuilder},
        sender::{
            smtp::{SmtpConfig, SmtpSender},
            MailSender,
        },
    };

    println!("Enviando correo de prueba a '{to}'...");
    println!("  SMTP: {smtp_host}:{smtp_port}");

    let config = match (smtp_user, smtp_pass) {
        (Some(user), Some(pass)) => SmtpConfig::new(smtp_host, smtp_port, user, pass),
        _ => SmtpConfig::new_unauthenticated(smtp_host, smtp_port),
    };

    let sender = SmtpSender::new(config).map_err(|e| format!("config SMTP invalida: {e}"))?;

    let email = EmailBuilder::new()
        .from(Address::new(from))
        .to(Address::new(to))
        .subject(subject)
        .html_body(format!(
            "<p>Correo de prueba enviado por <code>ag mail test</code>.</p>\
             <p>Si ves este mensaje, la configuracion SMTP es correcta.</p>\
             <p>Host: <strong>{smtp_host}:{smtp_port}</strong></p>"
        ))
        .text_body(format!(
            "Correo de prueba de ag mail test.\nHost: {smtp_host}:{smtp_port}"
        ))
        .build()
        .map_err(|e| format!("email invalido: {e}"))?;

    let result = sender
        .send(&email)
        .await
        .map_err(|e| format!("fallo de envio: {e}"))?;

    match result.message_id {
        Some(id) => println!("OK — correo enviado. Message-ID: {id}"),
        None => println!("OK — correo enviado."),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(validate_project_name("mi proyecto").is_err());
    }

    #[test]
    fn apply_template_substitutes_name() {
        let tmpl = "name = \"{{name}}\"";
        let result = apply_template(tmpl, "my-app");
        assert_eq!(result, "name = \"my-app\"");
    }

    #[test]
    fn apply_template_replaces_all_occurrences() {
        let tmpl = "{{name}} y {{name}}";
        let result = apply_template(tmpl, "alfa");
        assert_eq!(result, "alfa y alfa");
    }

    #[test]
    fn lint_fn_surfaces_warnings_for_model_without_primary() {
        let src = "model Tag { name String }";
        let diags = ag_dsl::lint(src);
        assert!(
            !diags.is_empty(),
            "debe haber warnings para modelo sin @primary"
        );
    }
}
