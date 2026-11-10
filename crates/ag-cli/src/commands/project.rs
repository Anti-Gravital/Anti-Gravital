//! Project commands: `ag new`, `ag dev`, `ag build`.

use std::fs;
use std::io::IsTerminal;
use std::path::Path;
use std::process;

// Templates embedded in the binary at compile time. The paths are relative
// to the workspace root directory (crates/ag-cli/src/commands/../../../../templates/).

const REST_CARGO_TOML: &str = include_str!("../../../../templates/rest/Cargo.toml.tmpl");
const REST_MAIN_RS: &str = include_str!("../../../../templates/rest/src/main.rs.tmpl");
const REST_CONFIG_TOML: &str = include_str!("../../../../templates/rest/config.toml.tmpl");

const REALTIME_CARGO_TOML: &str = include_str!("../../../../templates/realtime/Cargo.toml.tmpl");
const REALTIME_MAIN_RS: &str = include_str!("../../../../templates/realtime/src/main.rs.tmpl");
const REALTIME_CONFIG_TOML: &str = include_str!("../../../../templates/realtime/config.toml.tmpl");

const FULLSTACK_CARGO_TOML: &str = include_str!("../../../../templates/fullstack/Cargo.toml.tmpl");
const FULLSTACK_MAIN_RS: &str = include_str!("../../../../templates/fullstack/src/main.rs.tmpl");
const FULLSTACK_CONFIG_TOML: &str =
    include_str!("../../../../templates/fullstack/config.toml.tmpl");
const FULLSTACK_MIGRATION: &str =
    include_str!("../../../../templates/fullstack/migrations/0001_init.sql.tmpl");

pub(crate) fn cmd_new(name: &str, template: Option<&str>) -> Result<(), String> {
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
pub(crate) fn resolve_template_interactive() -> String {
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

pub(crate) fn cmd_dev(bind: &str) -> Result<(), String> {
    println!("Starting development server (bind: {bind})...");

    let watch_available = process::Command::new("cargo")
        .args(["watch", "--version"])
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let status = if watch_available {
        println!("Using cargo-watch for hot reload.");
        process::Command::new("cargo")
            .env("BIND", bind)
            .args(["watch", "-x", "run"])
            .status()
            .map_err(|e| format!("failed to run cargo watch: {e}"))?
    } else {
        println!("cargo-watch not found. Running without hot reload.");
        println!("To enable hot reload: cargo install cargo-watch");
        process::Command::new("cargo")
            .env("BIND", bind)
            .arg("run")
            .status()
            .map_err(|e| format!("failed to run cargo run: {e}"))?
    };

    if !status.success() {
        return Err("the server exited with an error".into());
    }
    Ok(())
}

pub(crate) fn cmd_build(target: Option<&str>) -> Result<(), String> {
    let mut args = vec!["build", "--release"];
    if let Some(t) = target {
        args.push("--target");
        args.push(t);
    }

    println!("Building in release mode...");
    let status = process::Command::new("cargo")
        .args(&args)
        .status()
        .map_err(|e| format!("failed to run cargo build: {e}"))?;

    if !status.success() {
        return Err("the build failed".into());
    }
    println!("Build completed successfully.");
    Ok(())
}

pub(crate) fn validate_project_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("project name cannot be empty".into());
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(format!(
            "invalid project name '{name}': only letters, numbers, hyphens, and underscores are allowed"
        ));
    }
    Ok(())
}

pub(crate) fn apply_template(template: &str, name: &str) -> String {
    template.replace("{{name}}", name)
}

pub(crate) fn write_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create directory '{}': {e}", parent.display()))?;
    }
    fs::write(path, content).map_err(|e| format!("failed to write '{}': {e}", path.display()))
}

pub(crate) fn scaffold_rest(name: &str, dir: &Path) -> Result<(), String> {
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

pub(crate) fn scaffold_realtime(name: &str, dir: &Path) -> Result<(), String> {
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

pub(crate) fn scaffold_fullstack(name: &str, dir: &Path) -> Result<(), String> {
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
