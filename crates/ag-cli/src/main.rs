//! Binario unificado `ag` del ecosistema Anti-Gravital.
//!
//! Implementa los comandos de Fase 2:
//!
//! - `ag new <nombre> [--template rest|realtime|fullstack]`
//! - `ag dev [--bind host:port]`
//! - `ag build [--target triple]`

use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
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
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::New { name, template } => cmd_new(&name, &template),
        Commands::Dev { bind } => cmd_dev(&bind),
        Commands::Build { target } => cmd_build(target.as_deref()),
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
}
