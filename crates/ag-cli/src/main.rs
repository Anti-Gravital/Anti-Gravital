//! Binario unificado `ag` del ecosistema Anti-Gravital.
//!
//! Comandos de Fase 2:
//! - `ag new <nombre> [--template rest|realtime|fullstack]`
//! - `ag dev [--bind host:port]`
//! - `ag build [--target triple]`
//!
//! Comandos de Fase 3 (DSL):
//! - `ag generate [--schema schema.ag] [--output ./generated]`
//! - `ag schema lint [--schema schema.ag]`
//! - `ag schema diff <ref> [--schema schema.ag]`

use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

// Templates embebidos en el binario al compilar. Las rutas son relativas
// al directorio raiz del workspace (crates/ag-cli/src/../../../templates/).

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
    /// Crea un nuevo proyecto Anti-Gravital.
    ///
    /// Genera la estructura de archivos a partir del template elegido
    /// y sustituye el nombre del proyecto en todos los archivos.
    New {
        /// Nombre del nuevo proyecto (también se usa como nombre de directorio).
        name: String,

        /// Template de partida.
        #[arg(long, short = 't', default_value = "rest",
              value_parser = ["rest", "realtime", "fullstack"])]
        template: String,
    },

    /// Arranca el servidor en modo desarrollo con hot reload.
    ///
    /// Requiere `cargo-watch` instalado (`cargo install cargo-watch`).
    /// Si no esta disponible, ejecuta `cargo run` sin recarga automatica.
    Dev {
        /// Direccion de escucha del servidor.
        #[arg(long, default_value = "0.0.0.0:8080")]
        bind: String,
    },

    /// Compila el proyecto en modo release.
    Build {
        /// Triple de compilacion cruzada (ej: `x86_64-unknown-linux-musl`).
        #[arg(long)]
        target: Option<String>,
    },

    /// Genera artefactos (Rust, SQL, TypeScript, OpenAPI) desde schema.ag.
    ///
    /// Lee el archivo DSL indicado (por defecto `schema.ag` en el directorio
    /// actual), compila el schema y escribe los artefactos generados en el
    /// directorio de salida.
    Generate {
        /// Archivo schema DSL de entrada.
        #[arg(long, default_value = "schema.ag")]
        schema: PathBuf,

        /// Directorio donde se escriben los artefactos generados.
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },

    /// Operaciones sobre el schema DSL.
    Schema {
        #[command(subcommand)]
        command: SchemaCommands,
    },
}

/// Sub-comandos de `ag schema`.
#[derive(Subcommand)]
enum SchemaCommands {
    /// Verifica el schema y reporta warnings de mejores practicas.
    Lint {
        /// Archivo schema DSL.
        #[arg(long, default_value = "schema.ag")]
        schema: PathBuf,
    },
    /// Muestra cambios breaking vs no-breaking respecto a un archivo de referencia.
    Diff {
        /// Archivo de referencia (otro schema.ag, snapshot, etc.).
        reference: PathBuf,
        /// Archivo schema actual.
        #[arg(long, default_value = "schema.ag")]
        schema: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::New { name, template } => cmd_new(&name, &template),
        Commands::Dev { bind } => cmd_dev(&bind),
        Commands::Build { target } => cmd_build(target.as_deref()),
        Commands::Generate { schema, output } => cmd_generate(&schema, &output),
        Commands::Schema { command } => match command {
            SchemaCommands::Lint { schema } => cmd_schema_lint(&schema),
            SchemaCommands::Diff { reference, schema } => cmd_schema_diff(&schema, &reference),
        },
    };
    if let Err(msg) = result {
        eprintln!("error: {msg}");
        process::exit(1);
    }
}

fn cmd_new(name: &str, template: &str) -> Result<(), String> {
    validate_project_name(name)?;

    let project_dir = Path::new(name);
    if project_dir.exists() {
        return Err(format!("el directorio '{name}' ya existe"));
    }

    println!("Creando proyecto '{name}' con template '{template}'...");

    match template {
        "rest" => scaffold_rest(name, project_dir),
        "realtime" => scaffold_realtime(name, project_dir),
        "fullstack" => scaffold_fullstack(name, project_dir),
        _ => Err(format!("template desconocido: {template}")),
    }?;

    println!();
    println!("Proyecto '{name}' creado.");
    println!();
    println!("Proximos pasos:");
    println!("  cd {name}");
    println!("  ag dev");

    Ok(())
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

// ---- Comandos DSL (Fase 3) ------------------------------------------

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

/// Compara dos schemas y retorna una lista de cambios legibles.
///
/// Detecta: modelos añadidos/eliminados, campos añadidos/eliminados/cambiados.
/// Los cambios breaking (eliminar modelo, eliminar campo, cambiar tipo) se marcan
/// con `[BREAKING]`. Los cambios no-breaking se marcan con `[additive]`.
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

    // Modelos eliminados (breaking)
    for name in old_models.keys() {
        if !new_models.contains_key(name) {
            changes.push(format!("[BREAKING]  modelo '{name}' eliminado"));
        }
    }

    // Modelos añadidos (no breaking)
    for name in new_models.keys() {
        if !old_models.contains_key(name) {
            changes.push(format!("[additive]  modelo '{name}' añadido"));
        }
    }

    // Campos en modelos comunes
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
        assert!(!diags.is_empty(), "debe haber warnings para modelo sin @primary");
    }
}
