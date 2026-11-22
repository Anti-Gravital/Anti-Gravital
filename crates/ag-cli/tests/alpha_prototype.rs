//! Alpha prototype offline-path regression suite.
//!
//! Exercises the two promises of the alpha: a scaffolded project can run a
//! local API, and a DSL schema can produce a generated SDK. Every assertion is
//! a local filesystem or process check: no DATABASE_URL, NATS, Redis, SMTP or
//! network service is started. This keeps the suite deterministic in CI.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn scratch_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ag-alpha-test-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch directory");
    dir
}

fn ag(args: &[&str], dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ag"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .env_remove("CI")
        .env_remove("AG_NO_TUI")
        .env("TERM", "xterm-256color")
        .output()
        .expect("run ag")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn rest_scaffold_runs_without_external_services() {
    let dir = scratch_dir("rest-offline");
    let output = ag(
        &["new", "demo", "--template", "rest", "--no-interactive"],
        &dir,
    );
    assert!(output.status.success(), "{output:?}");

    let project = dir.join("demo");
    for file in ["Cargo.toml", "config.toml", "src/main.rs"] {
        assert!(project.join(file).is_file(), "missing {file}");
    }
    assert!(
        !project.join("migrations").exists(),
        "rest needs no migrations"
    );

    let cargo = read(&project.join("Cargo.toml"));
    assert!(cargo.contains("ag-core"), "rest must use ag-core");
    assert!(cargo.contains("axum"), "rest must use axum");
    for banned in [
        "ag-data", "ag-mail", "ag-auth", "sqlx", "redis", "nats", "reqwest",
    ] {
        assert!(
            !cargo.contains(banned),
            "rest scaffold pulled in external service dependency '{banned}': {cargo}"
        );
    }

    // The generated server binds loopback by default and honours BIND (issue #191).
    let main = read(&project.join("src/main.rs"));
    assert!(main.contains("BIND"), "listener must honour BIND: {main}");
    assert!(main.contains("127.0.0.1:8080"), "loopback default: {main}");
    assert!(
        !main.contains("0.0.0.0"),
        "must not bind all interfaces: {main}"
    );

    // Both the listener and the generated config default to loopback (issue #191).
    let config = read(&project.join("config.toml"));
    assert!(
        config.contains("127.0.0.1"),
        "config loopback default: {config}"
    );
    assert!(
        !config.contains("0.0.0.0"),
        "config must not suggest all interfaces: {config}"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ag dev"),
        "next steps should mention ag dev"
    );
}

#[test]
fn layered_scaffold_also_binds_loopback() {
    let dir = scratch_dir("layered-offline");
    let output = ag(
        &["new", "svc", "--template", "layered", "--no-interactive"],
        &dir,
    );
    assert!(output.status.success(), "{output:?}");
    let main = read(&dir.join("svc/src/main.rs"));
    assert!(
        main.contains("BIND"),
        "layered listener must honour BIND: {main}"
    );
    assert!(main.contains("127.0.0.1:8080"), "loopback default: {main}");
    assert!(
        !main.contains("0.0.0.0"),
        "must not bind all interfaces: {main}"
    );
}

#[test]
fn fullstack_scaffold_declares_its_database_requirement() {
    let dir = scratch_dir("fullstack-boundary");
    let output = ag(
        &["new", "shop", "--template", "fullstack", "--no-interactive"],
        &dir,
    );
    assert!(output.status.success(), "{output:?}");
    let project = dir.join("shop");
    let cargo = read(&project.join("Cargo.toml"));
    assert!(cargo.contains("ag-data"), "fullstack must use ag-data");
    assert!(cargo.contains("sqlx"), "fullstack must use sqlx");
    let main = read(&project.join("src/main.rs"));
    assert!(
        main.contains("DATABASE_URL"),
        "fullstack needs DATABASE_URL"
    );
    assert!(project.join("migrations/0001_init.sql").is_file());
}

#[test]
fn generate_produces_sdk_and_service_artifacts_offline() {
    let dir = scratch_dir("generate");
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let schema = workspace.join("examples/ecommerce-api/schema.ag");
    assert!(
        schema.is_file(),
        "missing source schema: {}",
        schema.display()
    );
    std::fs::copy(&schema, dir.join("schema.ag")).expect("copy schema");

    let output = ag(
        &["generate", "--schema", "schema.ag", "--output", "generated"],
        &dir,
    );
    assert!(output.status.success(), "{output:?}");

    let generated = dir.join("generated");
    for artifact in [
        "src/mod.rs",
        "src/models.rs",
        "src/router.rs",
        "migrations/0001_initial.sql",
        "openapi.json",
        "openapi.yaml",
        "clients/typescript/client.ts",
        "clients/typescript/types.ts",
    ] {
        assert!(
            generated.join(artifact).is_file(),
            "missing generated {artifact}"
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("10 artifacts"),
        "unexpected generate summary: {stdout}"
    );
}
