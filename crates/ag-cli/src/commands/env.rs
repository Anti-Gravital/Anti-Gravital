//! `ag env list|check`: a read-only view of the environment declared in a
//! project's `ag.toml` (RFC-0020 section 4.1 and 4.6, stage B).
//!
//! The declarations are the single source of truth. They are turned into
//! `ag_core::EnvDeclaration` values and resolved through the core loader, so
//! `check` validates exactly the way startup will. No encrypted file exists
//! yet, nothing is written, and no value (including a non-secret default) is
//! ever placed in a view, a prose line or a JSON document.

use std::fs;
use std::path::{Path, PathBuf};

use ag_core::env::EnvErrorReason;
use ag_core::{EnvDeclaration, EnvLoader, EnvType};
use serde::Serialize;

/// Default project manifest read by the `ag env` commands.
pub(crate) const DEFAULT_MANIFEST: &str = "ag.toml";

/// Why an `ag env` command could not complete.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EnvCommandError {
    /// Invalid command line (exit code 2).
    #[allow(dead_code)]
    Usage(String),
    /// Reading or parsing the manifest failed (exit code 1).
    Failed(String),
}

impl EnvCommandError {
    /// The message rendered by the caller on stderr.
    pub(crate) fn into_message(self) -> String {
        match self {
            Self::Usage(message) | Self::Failed(message) => message,
        }
    }
}

/// Which read-only view was requested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EnvRequest {
    List {
        manifest: PathBuf,
        environment: Option<String>,
    },
    Check {
        manifest: PathBuf,
        environment: Option<String>,
    },
}

impl EnvRequest {
    fn manifest(&self) -> &Path {
        match self {
            Self::List { manifest, .. } | Self::Check { manifest, .. } => manifest,
        }
    }

    fn environment(&self) -> Option<&str> {
        match self {
            Self::List { environment, .. } | Self::Check { environment, .. } => {
                environment.as_deref()
            }
        }
    }
}

/// One declared variable in the `env.list` view. Contains no value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EnvVariableView {
    pub(crate) name: String,
    /// One of `string`, `int`, `bool`, `url`, `duration`, `path`.
    pub(crate) ty: &'static str,
    pub(crate) required: bool,
    pub(crate) secret: bool,
    /// Whether a declared default exists. The default itself is never shown.
    pub(crate) has_default: bool,
    /// `process`, `default` or `none`.
    pub(crate) source: &'static str,
    pub(crate) present: bool,
}

/// Data of `env.list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EnvListData {
    pub(crate) manifest: String,
    pub(crate) environment: Option<String>,
    pub(crate) variables: Vec<EnvVariableView>,
}

/// One problem in the `env.check` view. Contains no value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EnvProblemView {
    pub(crate) name: String,
    pub(crate) ty: &'static str,
    /// `missing`, `invalid_syntax` or `not_unicode`.
    pub(crate) reason: &'static str,
}

impl EnvProblemView {
    fn human_reason(&self) -> &'static str {
        match self.reason {
            "missing" => "missing (checked process environment and declared default)",
            "invalid_syntax" => "value failed type validation (value withheld)",
            "not_unicode" => "value is not valid Unicode (value withheld)",
            _ => "unknown problem",
        }
    }
}

/// Data of `env.check`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EnvCheckData {
    pub(crate) manifest: String,
    pub(crate) environment: Option<String>,
    pub(crate) ok: bool,
    /// Resolved variables on success. The loader does not expose a partial
    /// result on failure, so this is zero when `ok` is false.
    pub(crate) resolved_count: usize,
    pub(crate) problems: Vec<EnvProblemView>,
}

/// Stable name of an `EnvType` (identical to its `Display` output).
fn type_name(ty: EnvType) -> &'static str {
    match ty {
        EnvType::String => "string",
        EnvType::Int => "int",
        EnvType::Bool => "bool",
        EnvType::Url => "url",
        EnvType::Duration => "duration",
        EnvType::Path => "path",
    }
}

fn parse_type(value: &str) -> Option<EnvType> {
    match value {
        "string" => Some(EnvType::String),
        "int" => Some(EnvType::Int),
        "bool" => Some(EnvType::Bool),
        "url" => Some(EnvType::Url),
        "duration" => Some(EnvType::Duration),
        "path" => Some(EnvType::Path),
        _ => None,
    }
}

fn reason_code(reason: &EnvErrorReason) -> &'static str {
    match reason {
        EnvErrorReason::Missing => "missing",
        EnvErrorReason::InvalidSyntax => "invalid_syntax",
        EnvErrorReason::NotUnicode => "not_unicode",
    }
}

/// Renders a TOML scalar as the string the loader would validate. Booleans
/// and numbers are accepted so `default = 8080` works as in RFC-0020 section 4.1.
fn default_as_string(value: &toml::Value) -> Option<String> {
    match value {
        toml::Value::String(text) => Some(text.clone()),
        toml::Value::Integer(number) => Some(number.to_string()),
        toml::Value::Boolean(flag) => Some(flag.to_string()),
        toml::Value::Float(number) => Some(number.to_string()),
        _ => None,
    }
}

/// Builds the core loader from the `[env.*]` tables of a project manifest.
///
/// Unknown types and malformed declarations are hard errors: a read-only view
/// must never guess what a declaration means.
fn loader_from_manifest(manifest: &Path) -> Result<EnvLoader, EnvCommandError> {
    let text = fs::read_to_string(manifest).map_err(|e| {
        EnvCommandError::Failed(format!(
            "cannot read {}: {e}. Declare [env.*] variables in ag.toml (RFC-0020 section 4.1)",
            manifest.display()
        ))
    })?;
    let document: toml::Value = toml::from_str(&text)
        .map_err(|e| EnvCommandError::Failed(format!("invalid {}: {e}", manifest.display())))?;

    let mut declarations = Vec::new();
    if let Some(table) = document.get("env").and_then(toml::Value::as_table) {
        for (name, spec) in table {
            let spec = spec.as_table().ok_or_else(|| {
                EnvCommandError::Failed(format!(
                    "{}: [env.{name}] must be a table",
                    manifest.display()
                ))
            })?;
            let raw_type = spec
                .get("type")
                .and_then(toml::Value::as_str)
                .unwrap_or("string");
            let ty = parse_type(raw_type).ok_or_else(|| {
                EnvCommandError::Failed(format!(
                    "{}: [env.{name}] has unknown type '{raw_type}' \
                     (expected string|int|bool|url|duration|path)",
                    manifest.display()
                ))
            })?;

            let mut declaration = EnvDeclaration::new(name.clone(), ty);
            if let Some(required) = spec.get("required").and_then(toml::Value::as_bool) {
                declaration = declaration.required(required);
            }
            if let Some(secret) = spec.get("secret").and_then(toml::Value::as_bool) {
                declaration = declaration.secret(secret);
            }
            if let Some(default) = spec.get("default") {
                let default = default_as_string(default).ok_or_else(|| {
                    EnvCommandError::Failed(format!(
                        "{}: [env.{name}] default must be a string, number or boolean",
                        manifest.display()
                    ))
                })?;
                declaration = declaration.with_default(default);
            }
            if let Some(doc) = spec.get("description").and_then(toml::Value::as_str) {
                declaration = declaration.description_text(doc);
            }
            declarations.push(declaration);
        }
    }
    // TOML tables are already ordered by the `toml` crate, but sort explicitly
    // so the view stays deterministic if that ever changes.
    declarations.sort_by(|left, right| left.name().cmp(right.name()));
    Ok(EnvLoader::new(declarations))
}

/// Builds the `env.list` data without printing anything.
pub(crate) fn list_data(request: &EnvRequest) -> Result<EnvListData, EnvCommandError> {
    let loader = loader_from_manifest(request.manifest())?;
    let variables = loader
        .declarations()
        .iter()
        .map(|declaration| {
            let source = if std::env::var_os(declaration.name()).is_some() {
                "process"
            } else if declaration.default().is_some() {
                "default"
            } else {
                "none"
            };
            EnvVariableView {
                name: declaration.name().to_owned(),
                ty: type_name(declaration.ty()),
                required: declaration.is_required(),
                secret: declaration.is_secret(),
                has_default: declaration.default().is_some(),
                source,
                present: source != "none",
            }
        })
        .collect();
    Ok(EnvListData {
        manifest: request.manifest().display().to_string(),
        environment: request.environment().map(str::to_owned),
        variables,
    })
}

/// Builds the `env.check` data without printing anything. Uses the same loader
/// the composition root will use, so validation is identical to startup.
pub(crate) fn check_data(request: &EnvRequest) -> Result<EnvCheckData, EnvCommandError> {
    let loader = loader_from_manifest(request.manifest())?;
    let shared = || EnvCheckData {
        manifest: request.manifest().display().to_string(),
        environment: request.environment().map(str::to_owned),
        ok: true,
        resolved_count: 0,
        problems: Vec::new(),
    };
    match loader.load_process() {
        Ok(loaded) => {
            let mut data = shared();
            data.resolved_count = loaded.len();
            Ok(data)
        }
        Err(error) => {
            let mut data = shared();
            data.ok = false;
            data.problems = error
                .problems()
                .iter()
                .map(|problem| EnvProblemView {
                    name: problem.name().to_owned(),
                    ty: type_name(problem.expected()),
                    reason: reason_code(problem.reason()),
                })
                .collect();
            Ok(data)
        }
    }
}

/// Prints the prose `env.list` view. No value is included.
pub(crate) fn print_list(data: &EnvListData) {
    println!("Declared environment in {}:", data.manifest);
    print_environment_note(data.environment.as_deref());
    if data.variables.is_empty() {
        println!("  no [env.*] declarations found");
        return;
    }
    println!(
        "{:<24} {:<9} {:<10} {:<8} {:<9} PRESENT",
        "NAME", "TYPE", "REQUIRED", "SECRET", "SOURCE"
    );
    for variable in &data.variables {
        println!(
            "{:<24} {:<9} {:<10} {:<8} {:<9} {}",
            variable.name,
            variable.ty,
            variable.required,
            variable.secret,
            variable.source,
            variable.present
        );
    }
    println!(
        "{} variable(s) declared. Values are never printed.",
        data.variables.len()
    );
}

/// Prints a successful prose `env.check`. Failures are returned as errors so
/// the caller renders them once and exits non-zero.
pub(crate) fn check_report(data: &EnvCheckData) -> Result<(), String> {
    print_environment_note(data.environment.as_deref());
    if data.ok {
        println!(
            "Environment OK: {} variable(s) resolved from the process environment \
             and declared defaults.",
            data.resolved_count
        );
        return Ok(());
    }
    let mut message = format!(
        "environment validation failed ({} problem(s); values are never printed):",
        data.problems.len()
    );
    for problem in &data.problems {
        message.push_str(&format!(
            "\n  - '{}': expected {}; {}",
            problem.name,
            problem.ty,
            problem.human_reason()
        ));
    }
    Err(message)
}

fn print_environment_note(environment: Option<&str>) {
    if let Some(name) = environment {
        println!(
            "Note: environment '{name}' is a label only in stage B; \
             config/<environment>.toml and encrypted secrets are not read."
        );
    }
}

/// Entry point of `ag env` (prose path).
pub(crate) fn cmd_env(request: &EnvRequest) -> Result<(), EnvCommandError> {
    match request {
        EnvRequest::List { .. } => {
            print_list(&list_data(request)?);
            Ok(())
        }
        EnvRequest::Check { .. } => {
            let data = check_data(request)?;
            check_report(&data).map_err(EnvCommandError::Failed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> &'static str {
        "[project]\nname = \"shop\"\n\n\
         [env.DATABASE_URL]\ntype = \"url\"\nrequired = true\nsecret = true\n\
         description = \"PostgreSQL connection string\"\n\n\
         [env.PORT]\ntype = \"int\"\ndefault = 8080\n\n\
         [env.API_TOKEN]\ntype = \"string\"\nsecret = true\nrequired = false\n"
    }

    fn write_manifest(dir: &Path, text: &str) -> PathBuf {
        let path = dir.join("ag.toml");
        fs::write(&path, text).expect("write manifest");
        path
    }

    fn scratch(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ag-env-test-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create dir");
        dir
    }

    #[test]
    fn list_shows_metadata_and_presence_without_any_value() {
        let dir = scratch("list");
        let path = write_manifest(&dir, manifest());
        std::env::set_var("AG_ENV_TEST_DATABASE_URL", "postgres://localhost/shop");
        let data = list_data(&EnvRequest::List {
            manifest: path.clone(),
            environment: Some("dev".to_owned()),
        })
        .expect("list data");
        let db = data
            .variables
            .iter()
            .find(|variable| variable.name == "DATABASE_URL")
            .expect("declaration");
        assert_eq!(db.ty, "url");
        assert!(db.secret && db.required);
        assert_eq!(db.source, "none");
        assert!(!db.present);
        let port = data
            .variables
            .iter()
            .find(|variable| variable.name == "PORT")
            .expect("declaration");
        assert_eq!(port.ty, "int");
        assert!(port.has_default);
        assert_eq!(port.source, "default");
        assert!(port.present);
        assert_eq!(data.environment.as_deref(), Some("dev"));
        // No view field ever contains a value or a default value.
        let json = serde_json::to_string(&data).expect("serialize");
        assert!(!json.contains("postgres://localhost/shop"));
        assert!(!json.contains("8080"));
        std::env::remove_var("AG_ENV_TEST_DATABASE_URL");
    }

    #[test]
    fn check_collects_all_problems_without_values() {
        let dir = scratch("check");
        let path = write_manifest(
            &dir,
            "[env.A]\ntype = \"url\"\n\n[env.B]\ntype = \"int\"\n\n[env.C]\ntype = \"path\"\n",
        );
        let data = check_data(&EnvRequest::Check {
            manifest: path,
            environment: None,
        })
        .expect("check data");
        assert!(!data.ok);
        assert_eq!(data.problems.len(), 3);
        assert_eq!(data.problems[0].reason, "missing");
        assert!(!serde_json::to_string(&data)
            .expect("serialize")
            .contains("8080"));
    }

    #[test]
    fn an_unknown_type_is_a_hard_error() {
        let dir = scratch("unknown-type");
        let path = write_manifest(&dir, "[env.X]\ntype = \"mystery\"\n");
        let error = list_data(&EnvRequest::List {
            manifest: path,
            environment: None,
        })
        .expect_err("unknown type must fail");
        assert!(error.into_message().contains("unknown type"));
    }

    #[test]
    fn a_missing_manifest_is_a_clear_error() {
        let dir = scratch("missing");
        let error = list_data(&EnvRequest::List {
            manifest: dir.join("absent.toml"),
            environment: None,
        })
        .expect_err("missing manifest must fail");
        assert!(error.into_message().contains("Declare [env.*]"));
    }
}
