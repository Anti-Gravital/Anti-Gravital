//! Project commands: `ag new`, `ag dev`, `ag build`.

use std::fs;
use std::path::Path;
use std::process;

use crate::interactive::Session;
use crate::request::{NewProjectRequest, Template};

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

/// Why `ag new` could not produce a project.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum NewError {
    /// The command line is incomplete and the session cannot ask (exit code 2).
    Usage(String),
    /// Creating the project failed (exit code 1).
    Failed(String),
}

/// How the inputs of `ag new` were obtained.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Resolved {
    /// Every input came from the command line or from documented defaults.
    FromFlags(NewProjectRequest),
    /// At least one input was asked interactively.
    Asked(NewProjectRequest),
    /// The user cancelled the interactive flow. Nothing was written.
    Cancelled,
}

/// True when a file or directory named `name` exists in the current directory.
#[cfg_attr(not(feature = "tui"), allow(dead_code))]
pub(crate) fn path_exists(name: &str) -> bool {
    Path::new(name).exists()
}

/// Turns the command-line inputs of `ag new` into a request.
///
/// Complete input never prompts. Incomplete input prompts only in an
/// interactive session; otherwise a missing name is a usage error and a
/// missing template takes the documented default. `ask` is the interactive
/// front-end, injected so the decision logic is testable without a terminal.
pub(crate) fn resolve_new_request(
    name: Option<String>,
    template: Option<Template>,
    session: &Session,
    ask: impl FnOnce(Option<String>, Option<Template>) -> Result<Option<NewProjectRequest>, String>,
) -> Result<Resolved, NewError> {
    match (name, template) {
        (Some(name), Some(template)) => {
            Ok(Resolved::FromFlags(NewProjectRequest { name, template }))
        }
        (name, template) if session.is_interactive() && cfg!(feature = "tui") => {
            match ask(name, template).map_err(NewError::Failed)? {
                Some(request) => Ok(Resolved::Asked(request)),
                None => Ok(Resolved::Cancelled),
            }
        }
        (Some(name), None) => Ok(Resolved::FromFlags(NewProjectRequest {
            name,
            template: Template::Rest,
        })),
        (None, _) => Err(NewError::Usage(
            "the project name is required when the session is not interactive.\n\
             usage: ag new <NAME> [--template rest|realtime|fullstack]"
                .to_owned(),
        )),
    }
}

/// Creates the project described by the request. The single executor behind
/// both the flags and the interactive interface.
pub(crate) fn execute_new(request: &NewProjectRequest) -> Result<(), String> {
    let name = request.name.as_str();
    validate_project_name(name)?;

    let project_dir = Path::new(name);
    if project_dir.exists() {
        return Err(format!("directory '{name}' already exists"));
    }

    println!(
        "Creating project '{name}' with template '{}'...",
        request.template
    );

    match request.template {
        Template::Rest => scaffold_rest(name, project_dir),
        Template::Realtime => scaffold_realtime(name, project_dir),
        Template::Fullstack => scaffold_fullstack(name, project_dir),
    }?;

    println!();
    println!("Project '{name}' created.");
    println!();
    println!("Next steps:");
    println!("  cd {name}");
    println!("  ag dev");

    Ok(())
}

/// Entry point of `ag new`.
pub(crate) fn cmd_new(
    name: Option<String>,
    template: Option<&str>,
    no_interactive: bool,
) -> Result<(), NewError> {
    let template = match template {
        Some(value) => Some(
            Template::parse(value)
                .ok_or_else(|| NewError::Usage(format!("unknown template: {value}")))?,
        ),
        None => None,
    };
    let session = Session::from_environment(no_interactive);
    match resolve_new_request(name, template, &session, |name, template| {
        ask_new_project(name, template, &session)
    })? {
        Resolved::FromFlags(request) => execute_new(&request).map_err(NewError::Failed),
        Resolved::Asked(request) => {
            execute_new(&request).map_err(NewError::Failed)?;
            println!();
            println!("{EQUIVALENT_COMMAND_LABEL}");
            println!("  {}", request.equivalent_command());
            Ok(())
        }
        Resolved::Cancelled => {
            println!("{CANCELLED_LABEL}");
            Ok(())
        }
    }
}

#[cfg(feature = "tui")]
use crate::tui::text::{
    ABORTED as CANCELLED_LABEL, EQUIVALENT_COMMAND as EQUIVALENT_COMMAND_LABEL,
};
#[cfg(not(feature = "tui"))]
const CANCELLED_LABEL: &str = "Cancelled. Nothing was written.";
#[cfg(not(feature = "tui"))]
const EQUIVALENT_COMMAND_LABEL: &str = "Equivalent non-interactive command:";

#[cfg(feature = "tui")]
fn ask_new_project(
    name: Option<String>,
    template: Option<Template>,
    session: &Session,
) -> Result<Option<NewProjectRequest>, String> {
    crate::tui::ask_new_project(name, template, session, path_exists)
}

#[cfg(not(feature = "tui"))]
fn ask_new_project(
    _name: Option<String>,
    _template: Option<Template>,
    _session: &Session,
) -> Result<Option<NewProjectRequest>, String> {
    Err("this build of ag has no interactive interface".to_owned())
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
