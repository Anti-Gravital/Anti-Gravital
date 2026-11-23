//! Project commands: `ag new`, `ag dev`, `ag build`.

use std::fs;
use std::path::Path;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::interactive::Session;
use crate::project_state::{self, DevState, HEARTBEAT_SECS, SCHEMA};
use crate::request::{Module, NewProjectRequest, Template};

// Template files are discovered and embedded by `build.rs` (see
// crates/ag-cli/src/scaffold.rs). The catalogue of accepted names lives in
// crates/ag-cli/src/catalog.rs and is documented in templates/CATALOG.md.

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
    modules: Vec<Module>,
    session: &Session,
    ask: impl FnOnce(Option<String>, Option<Template>) -> Result<Option<NewProjectRequest>, String>,
) -> Result<Resolved, NewError> {
    match (name, template) {
        (Some(name), Some(template)) => Ok(Resolved::FromFlags(NewProjectRequest {
            name,
            template,
            modules,
        })),
        (name, template) if session.is_interactive() && cfg!(feature = "tui") => {
            match ask(name, template).map_err(NewError::Failed)? {
                // The interactive interface has no module step, but modules
                // supplied on the command line are still honoured and appear
                // in the printed equivalent command.
                Some(mut request) => {
                    if !modules.is_empty() {
                        request.modules = modules;
                    }
                    Ok(Resolved::Asked(request))
                }
                None => Ok(Resolved::Cancelled),
            }
        }
        (Some(name), None) => Ok(Resolved::FromFlags(NewProjectRequest {
            name,
            template: Template::DEFAULT,
            modules,
        })),
        (None, _) => Err(NewError::Usage(
            "the project name is required when the session is not interactive.\n\
             usage: ag new <NAME> [--template <name>] [--with <module>]\
             (see templates/CATALOG.md)"
                .to_owned(),
        )),
    }
}

/// D-1 (ADR-0014): mirrors `tui::in_project` — `config.toml` is a file and
/// `Cargo.toml` contains `ag-core`. The executor is the authority for both the
/// flags and the TUI paths, so the guard lives here before any scaffolding.
fn current_dir_has_project() -> bool {
    Path::new("config.toml").is_file()
        && fs::read_to_string("Cargo.toml").is_ok_and(|manifest| manifest.contains("ag-core"))
}

fn guard_create_in_project() -> Result<(), NewError> {
    if current_dir_has_project() {
        return Err(NewError::Usage(
            "a project already exists in the current directory; run 'cd ..' \
             and repeat 'ag new <NAME>' (ADR-0014 D-1)"
                .to_owned(),
        ));
    }
    Ok(())
}

/// Creates the project described by the request. The single executor behind
/// both the flags and the interactive interface.
pub(crate) fn execute_new(request: &NewProjectRequest) -> Result<(), NewError> {
    // Fail before creating anything: D-1 forbids nesting a project in a project.
    guard_create_in_project()?;

    let name = request.name.as_str();
    validate_project_name(name).map_err(NewError::Failed)?;

    // Fail before creating anything: --with is only meaningful for a template
    // that actually generates the ag.toml manifest (RFC-0019 section 4.3).
    if !request.modules.is_empty()
        && !crate::scaffold::files_for(request.template.as_str()).contains(&"ag.toml")
    {
        return Err(NewError::Failed(format!(
            "--with requires a template that generates ag.toml (for example \
             --template layered); template '{}' has no manifest",
            request.template
        )));
    }

    let project_dir = Path::new(name);
    if project_dir.exists() {
        return Err(NewError::Failed(format!(
            "directory '{name}' already exists"
        )));
    }

    println!(
        "Creating project '{name}' with template '{}'...",
        request.template
    );

    crate::scaffold::scaffold(name, request.template.as_str(), project_dir)
        .map_err(NewError::Failed)?;

    // The default scaffold is byte-for-byte unchanged: the manifest is only
    // rewritten when the caller explicitly enabled modules.
    if !request.modules.is_empty() {
        let manifest = project_dir.join("ag.toml");
        let original = fs::read_to_string(&manifest)
            .map_err(|e| NewError::Failed(format!("failed to read generated ag.toml: {e}")))?;
        let updated =
            enable_modules_in_manifest(&original, &request.modules).map_err(NewError::Failed)?;
        write_file(&manifest, &updated).map_err(NewError::Failed)?;
    }

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
    with: &[String],
    no_interactive: bool,
) -> Result<(), NewError> {
    let template = match template {
        Some(value) => Some(
            Template::parse(value)
                .ok_or_else(|| NewError::Usage(format!("unknown template: {value}")))?,
        ),
        None => None,
    };
    let mut modules = Vec::with_capacity(with.len());
    for value in with {
        let module = Module::parse(value)
            .ok_or_else(|| NewError::Usage(format!("unknown module: {value}")))?;
        if !modules.contains(&module) {
            modules.push(module);
        }
    }
    let session = Session::from_environment(no_interactive);
    match resolve_new_request(name, template, modules, &session, |name, template| {
        ask_new_project(name, template, &session)
    })? {
        Resolved::FromFlags(request) => execute_new(&request),
        Resolved::Asked(request) => {
            execute_new(&request)?;
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

/// Validates a `host:port` bind before it reaches the state file or the child
/// process. Rejects credentials, schemes, paths, queries, whitespace and
/// non-numeric ports so the state never carries a secret (S-1/S-3/S-4/S-8).
/// The offending value is deliberately not echoed in the error (S-8).
fn validate_bind(bind: &str) -> Result<(), String> {
    if bind.contains('@')
        || bind.contains('/')
        || bind.contains('?')
        || bind.contains(char::is_whitespace)
    {
        return Err(
            "invalid bind: expected host:port without credentials, path or whitespace".to_owned(),
        );
    }
    let (host, port) = bind
        .split_once(':')
        .ok_or_else(|| "invalid bind: expected host:port".to_owned())?;
    let host_valid =
        !host.is_empty() && (!host.contains(':') || (host.starts_with('[') && host.ends_with(']')));
    if !host_valid || port.is_empty() || port.parse::<u16>().is_err() {
        return Err("invalid bind: expected host:port with a numeric port".to_owned());
    }
    Ok(())
}

/// Stops and joins the heartbeat thread from `Drop`, so every return path
/// (including `?` propagation) is clean without a duplicated join call.
struct HeartbeatGuard {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Drop for HeartbeatGuard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            if handle.join().is_err() {
                eprintln!("warning: heartbeat thread panicked");
            }
        }
    }
}

fn run_dev(bind: &str) -> Result<(), String> {
    validate_bind(bind)?;
    println!("Starting development server (bind: {bind})...");

    // Publish the state before any cargo child is spawned (ADR-0014 §1). A
    // failure here is returned, never swallowed (S-7).
    let started_at = project_state::now_secs();
    let state = DevState {
        schema: SCHEMA,
        pid: process::id(),
        bind: bind.to_owned(),
        started_at,
        heartbeat_at: started_at,
    };
    project_state::write(&state).map_err(|e| format!("failed to record development state: {e}"))?;

    // Refresh the heartbeat while the child is alive. `Arc<AtomicBool>` is the
    // stop signal; `HeartbeatGuard` joins before this function returns.
    let stop = Arc::new(AtomicBool::new(false));
    let heartbeat_stop = Arc::clone(&stop);
    let heartbeat_bind = bind.to_owned();
    let handle = thread::spawn(move || {
        while !heartbeat_stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(HEARTBEAT_SECS));
            if heartbeat_stop.load(Ordering::Relaxed) {
                break;
            }
            let refreshed = DevState {
                schema: SCHEMA,
                pid: process::id(),
                bind: heartbeat_bind.clone(),
                started_at,
                heartbeat_at: project_state::now_secs(),
            };
            if let Err(e) = project_state::write(&refreshed) {
                eprintln!("warning: failed to refresh development state: {e}");
            }
        }
    });
    let _heartbeat = HeartbeatGuard {
        stop,
        handle: Some(handle),
    };

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

pub(crate) fn cmd_dev(bind: &str) -> Result<(), String> {
    let result = run_dev(bind);
    // Always clear the state on exit: success, error or `?` propagation (S-7).
    // Never leave a file claiming `running` after this function returns.
    if let Err(e) = project_state::remove() {
        eprintln!("warning: failed to clear development state: {e}");
    }
    result
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

/// Sets `name = true` for each module in the `\[modules\]` table of a generated
/// `ag.toml`. Other lines, tables and the trailing newline are preserved.
///
/// This is a targeted text edit rather than a TOML round-trip so the rest of
/// the manifest stays byte-identical and the default scaffold is untouched.
fn enable_modules_in_manifest(text: &str, modules: &[Module]) -> Result<String, String> {
    let trailing_newline = text.ends_with('\n');
    let mut lines: Vec<String> = text.lines().map(|line| line.to_owned()).collect();

    let Some(start) = lines.iter().position(|line| line.trim() == "[modules]") else {
        return Err("generated ag.toml has no [modules] table".to_owned());
    };
    let mut end = lines.len();
    for (index, line) in lines.iter().enumerate().skip(start + 1) {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            end = index;
            break;
        }
    }

    for module in modules {
        let key = module.as_str();
        let existing = lines
            .iter_mut()
            .take(end)
            .skip(start + 1)
            .find(|line| line.trim().split('=').next().map(str::trim) == Some(key));
        if let Some(line) = existing {
            *line = format!("{key} = true");
        } else {
            lines.insert(end, format!("{key} = true"));
        }
    }

    let mut result = lines.join("\n");
    if trailing_newline {
        result.push('\n');
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = "[project]\nname = \"shop\"\nlayout = \"layered\"\n\n\
                            [modules]\ndata = true\nauth = false\ncache = false\n\n\
                            [generate]\nrust = \"crates\"\n";

    #[test]
    fn enabling_modules_flips_existing_keys_and_preserves_other_lines() {
        let updated =
            enable_modules_in_manifest(MANIFEST, &[Module::parse("auth").expect("declared")])
                .expect("edit");
        assert!(updated.contains("data = true"));
        assert!(updated.contains("auth = true"));
        assert!(!updated.contains("auth = false"));
        assert!(updated.contains("cache = false"));
        assert!(updated.contains("[generate]\nrust = \"crates\"\n"));
        assert!(updated.ends_with('\n'));
    }

    #[test]
    fn enabling_an_undeclared_key_appends_it_inside_the_modules_table() {
        let updated =
            enable_modules_in_manifest(MANIFEST, &[Module::parse("mail").expect("declared")])
                .expect("edit");
        let modules = updated
            .split("[modules]")
            .nth(1)
            .expect("modules table")
            .split("[generate]")
            .next()
            .expect("generate table");
        assert!(modules.contains("mail = true"), "{modules}");
        assert!(updated.contains("[generate]\n"));
    }

    #[test]
    fn enabling_multiple_modules_is_ordered_and_deduplicated_by_the_caller() {
        let updated = enable_modules_in_manifest(
            MANIFEST,
            &[
                Module::parse("auth").expect("declared"),
                Module::parse("cache").expect("declared"),
            ],
        )
        .expect("edit");
        assert!(updated.contains("auth = true"));
        assert!(updated.contains("cache = true"));
        assert_eq!(updated.matches("auth = true").count(), 1);
    }

    #[test]
    fn a_manifest_without_a_modules_table_is_an_error() {
        let error = enable_modules_in_manifest(
            "[project]\nname = \"shop\"\n",
            &[Module::parse("auth").expect("declared")],
        )
        .expect_err("no modules table");
        assert!(error.contains("[modules]"), "{error}");
    }

    use std::env;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
    use std::sync::Mutex;

    // The tests below change the process-wide cwd; serialise them so they do
    // not interfere with each other or with other modules' tests.
    static CWD_LOCK: Mutex<()> = Mutex::new(());
    static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

    fn make_temp_dir(tag: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!(
            "ag-cli-{tag}-{}-{}",
            process::id(),
            NEXT_TEMP_DIR.fetch_add(1, AtomicOrdering::Relaxed)
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    struct CwdGuard(PathBuf);
    impl Drop for CwdGuard {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.0);
        }
    }

    fn enter_dir(dir: &Path) -> CwdGuard {
        let original = env::current_dir().expect("current dir");
        env::set_current_dir(dir).expect("enter temp dir");
        CwdGuard(original)
    }

    #[test]
    fn guard_rejects_new_project_inside_existing_project() {
        let _serial = CWD_LOCK.lock().expect("cwd lock");
        let dir = make_temp_dir("guard");
        {
            let _cwd = enter_dir(&dir);
            fs::write("config.toml", "").expect("config.toml marker");
            fs::write(
                "Cargo.toml",
                "[package]\nname = \"host\"\n[dependencies]\nag-core = \"1\"\n",
            )
            .expect("Cargo.toml marker");

            let request = NewProjectRequest {
                name: "nested".to_owned(),
                template: Template::DEFAULT,
                modules: vec![],
            };
            match execute_new(&request).expect_err("D-1 must reject") {
                NewError::Usage(message) => {
                    assert!(message.contains("cd .."), "{message}");
                }
                other => panic!("expected NewError::Usage, got {other:?}"),
            }
            assert!(!Path::new("nested").exists(), "nothing must be scaffolded");
        }
        fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }

    #[test]
    fn cmd_dev_does_not_write_state_when_state_write_fails() {
        let _serial = CWD_LOCK.lock().expect("cwd lock");
        let dir = make_temp_dir("dev-state-fail");
        {
            let _cwd = enter_dir(&dir);
            // A regular file at `.ag` makes project_state::write fail (S-7).
            fs::write(".ag", "not a directory").expect(".ag file");

            let error = cmd_dev("127.0.0.1:8080").expect_err("write must fail");
            assert!(
                error.contains("failed to record development state"),
                "{error}"
            );
            assert!(
                !Path::new(".ag/dev.json").exists(),
                "no state file must claim running"
            );
        }
        fs::remove_dir_all(&dir).expect("cleanup temp dir");
    }
}
