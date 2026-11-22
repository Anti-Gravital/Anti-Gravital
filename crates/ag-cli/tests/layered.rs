//! Regression suite for RFC-0019 stage A: the opt-in `layered` template.
//!
//! Runs the real `ag` binary against a scratch directory and checks that the
//! generated Cargo workspace encodes the inward-only dependency rule. The
//! three default templates are asserted unchanged so `ag new` and `ag dev`
//! keep their existing behaviour.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static SCRATCH_SEQ: AtomicUsize = AtomicUsize::new(0);

fn scratch_dir(label: &str) -> PathBuf {
    let seq = SCRATCH_SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "ag-cli-layered-{label}-{}-{seq}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch directory");
    dir
}

fn ag(args: &[&str], dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ag"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .expect("run ag")
}

fn read(dir: &Path, relative: &str) -> String {
    std::fs::read_to_string(dir.join(relative)).unwrap_or_else(|e| panic!("read {relative}: {e}"))
}

/// Dependency keys of the `[dependencies]` table. Layer crates alias their
/// siblings (`domain = { package = "<name>-domain", path = ... }`), so the
/// key is the stable identifier used for direction checks.
fn dependency_keys(manifest: &str) -> Vec<String> {
    let mut in_dependencies = false;
    let mut keys = Vec::new();
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_dependencies = trimmed == "[dependencies]";
            continue;
        }
        if !in_dependencies || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, _)) = trimmed.split_once('=') {
            keys.push(key.trim().trim_matches('"').to_owned());
        }
    }
    keys
}

fn assert_no(keys: &[String], forbidden: &[&str], layer: &str) {
    for name in forbidden {
        assert!(
            !keys.iter().any(|k| k == name),
            "{layer} must not depend on {name}; got {keys:?}"
        );
    }
}

#[test]
fn layered_template_creates_the_workspace_and_manifest() {
    let dir = scratch_dir("files");
    let output = ag(&["new", "shop", "--template", "layered"], &dir);
    assert!(output.status.success(), "{output:?}");

    let project = dir.join("shop");
    for file in [
        "ag.toml",
        "Cargo.toml",
        "config.toml",
        "src/main.rs",
        "crates/domain/Cargo.toml",
        "crates/domain/src/lib.rs",
        "crates/app/Cargo.toml",
        "crates/app/src/lib.rs",
        "crates/infra/Cargo.toml",
        "crates/infra/src/lib.rs",
        "crates/infra/migrations/0001_init.sql",
        "crates/api/Cargo.toml",
        "crates/api/src/lib.rs",
    ] {
        assert!(project.join(file).is_file(), "missing {file}");
    }

    let manifest = read(&project, "ag.toml");
    assert!(manifest.contains("name = \"shop\""));
    assert!(manifest.contains("layout = \"layered\""));
    assert!(manifest.contains("[modules]"));
    assert!(manifest.contains("[generate]"));

    let workspace = read(&project, "Cargo.toml");
    assert!(workspace.contains("[workspace]"));
    for member in ["crates/domain", "crates/app", "crates/infra", "crates/api"] {
        assert!(workspace.contains(member), "workspace misses {member}");
    }

    // Each layer file lands in the crate that owns it.
    assert!(read(&project, "crates/domain/src/lib.rs").contains("pub struct Item"));
    assert!(read(&project, "crates/app/src/lib.rs").contains("pub trait ItemRepository"));
    assert!(read(&project, "crates/infra/src/lib.rs").contains("impl ItemRepository"));
    assert!(read(&project, "crates/api/src/lib.rs").contains("pub async fn serve"));
    assert!(read(&project, "src/main.rs").contains("api::serve"));
}

#[test]
fn layered_dependencies_point_inward() {
    let dir = scratch_dir("deps");
    assert!(ag(&["new", "svc", "--template", "layered"], &dir)
        .status
        .success());
    let project = dir.join("svc");

    let domain = dependency_keys(&read(&project, "crates/domain/Cargo.toml"));
    assert_no(
        &domain,
        &["app", "infra", "api", "sqlx", "axum", "tokio"],
        "domain",
    );
    assert!(
        !domain.iter().any(|k| k.starts_with("ag-")),
        "domain must not depend on any ag-* crate; got {domain:?}"
    );

    let app = dependency_keys(&read(&project, "crates/app/Cargo.toml"));
    assert!(app.iter().any(|k| k == "domain"), "app needs domain");
    assert_no(&app, &["infra", "api", "sqlx", "axum"], "app");
    assert!(
        !app.iter().any(|k| k.starts_with("ag-")),
        "app must not depend on any ag-* crate; got {app:?}"
    );

    let infra = dependency_keys(&read(&project, "crates/infra/Cargo.toml"));
    assert!(infra.iter().any(|k| k == "app"), "infra needs app");
    assert!(infra.iter().any(|k| k == "domain"), "infra needs domain");
    assert!(infra.iter().any(|k| k == "ag-data"), "infra owns ag-data");
    assert_no(&infra, &["api", "sqlx"], "infra");

    let api = dependency_keys(&read(&project, "crates/api/Cargo.toml"));
    assert!(api.iter().any(|k| k == "app"), "api needs app");
    assert!(api.iter().any(|k| k == "domain"), "api needs domain");
    assert!(api.iter().any(|k| k == "ag-core"), "api owns ag-core");
    assert_no(&api, &["infra", "sqlx"], "api");

    let root = dependency_keys(&read(&project, "Cargo.toml"));
    for layer in ["domain", "app", "infra", "api"] {
        assert!(root.iter().any(|k| k == layer), "root needs {layer}");
    }
}

#[test]
fn default_and_legacy_templates_are_unchanged() {
    let dir = scratch_dir("defaults");

    // No --template in a non-interactive session still means `rest`.
    assert!(ag(&["new", "api"], &dir).status.success());
    // The three named templates remain byte-compatible in shape.
    assert!(ag(&["new", "rest-api", "--template", "rest"], &dir)
        .status
        .success());
    assert!(ag(&["new", "ws", "--template", "realtime"], &dir)
        .status
        .success());
    assert!(ag(&["new", "shop", "--template", "fullstack"], &dir)
        .status
        .success());

    for project in ["api", "rest-api", "ws", "shop"] {
        let root = dir.join(project);
        assert!(root.join("Cargo.toml").is_file(), "{project}: Cargo.toml");
        assert!(root.join("src/main.rs").is_file(), "{project}: main.rs");
        assert!(root.join("config.toml").is_file(), "{project}: config.toml");
        assert!(
            !root.join("ag.toml").exists(),
            "{project}: ag.toml appeared"
        );
        assert!(!root.join("crates").exists(), "{project}: crates/ appeared");
    }
    assert!(dir.join("shop/migrations/0001_init.sql").is_file());
    assert!(!dir.join("api/migrations").exists());
}
