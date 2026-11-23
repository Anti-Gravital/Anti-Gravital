//! Snapshot tests for `ag --output json` (RFC-0021 section 4.3).
//!
//! Every supported read command has a checked-in golden document under
//! `docs/mcp/snapshots/`. The comparison is byte-exact: field order, spacing
//! and version are part of the contract. A change in emitted JSON must change
//! the snapshot and `contract_version` in the same commit.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

// Relative to CARGO_MANIFEST_DIR (crates/ag-cli); include_str! paths below are
// relative to this source file, hence one extra `..` there.
pub(crate) const SNAPSHOT_DIR: &str = "../../docs/mcp/snapshots";

pub(crate) fn snapshot(name: &str) -> &'static str {
    match name {
        "schema.lint.clean" => {
            include_str!("../../../../docs/mcp/snapshots/schema.lint.clean.json")
        }
        "schema.lint.warnings" => {
            include_str!("../../../../docs/mcp/snapshots/schema.lint.warnings.json")
        }
        "schema.lint.errors" => {
            include_str!("../../../../docs/mcp/snapshots/schema.lint.errors.json")
        }
        "schema.lint.read_failed" => {
            include_str!("../../../../docs/mcp/snapshots/schema.lint.read_failed.json")
        }
        "schema.diff" => include_str!("../../../../docs/mcp/snapshots/schema.diff.json"),
        "workers.list" => include_str!("../../../../docs/mcp/snapshots/workers.list.json"),
        "dns.plan" => include_str!("../../../../docs/mcp/snapshots/dns.plan.json"),
        "dns.status" => include_str!("../../../../docs/mcp/snapshots/dns.status.json"),
        "deploy.plan" => include_str!("../../../../docs/mcp/snapshots/deploy.plan.json"),
        "deploy.status" => include_str!("../../../../docs/mcp/snapshots/deploy.status.json"),
        "config.get" => include_str!("../../../../docs/mcp/snapshots/config.get.json"),
        "env.list" => include_str!("../../../../docs/mcp/snapshots/env.list.json"),
        "env.check" => include_str!("../../../../docs/mcp/snapshots/env.check.json"),
        "unsupported_command" => {
            include_str!("../../../../docs/mcp/snapshots/unsupported_command.json")
        }
        other => panic!("unknown snapshot: {other}"),
    }
}

pub(crate) fn scratch_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ag-json-test-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch directory");
    dir
}

pub(crate) fn write(dir: &Path, name: &str, contents: &str) {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create fixture parent");
    }
    std::fs::write(path, contents).expect("write fixture");
}

pub(crate) fn ag(args: &[&str], dir: &Path, env: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ag"));
    command
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .env_remove("CI")
        .env_remove("AG_NO_TUI");
    for (key, value) in env {
        command.env(key, value);
    }
    command.output().expect("run ag")
}

pub(crate) fn stdout(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).expect("stdout is UTF-8")
}

pub(crate) fn assert_snapshot(output: &Output, name: &str) {
    assert_eq!(
        stdout(output).trim_end(),
        snapshot(name).trim_end(),
        "snapshot mismatch for {name}"
    );
    assert!(
        output.stderr.is_empty(),
        "JSON mode must not write prose to stderr: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
}

pub(crate) fn clean_schema() -> &'static str {
    "model User {\n  id UUID @primary\n  name String\n}\n"
}

pub(crate) fn warning_schema() -> &'static str {
    "model Tag {\n  name String\n}\n"
}

pub(crate) fn invalid_schema() -> &'static str {
    "model { broken\n"
}

pub(crate) fn reference_schema() -> &'static str {
    "model Old {\n  id UUID @primary\n  name String\n}\n"
}

pub(crate) fn current_schema() -> &'static str {
    "model Old {\n  id UUID @primary\n  title String\n}\nmodel New {\n  id UUID @primary\n}\n"
}

pub(crate) fn dns_manifest() -> &'static str {
    "[project]\nname = \"demo\"\n\n[domain]\ndomain = \"api.example.com\"\nenvironment = \
     \"production\"\nvisibility = \"public\"\ntls = \"managed_http01\"\ntarget = { kind = \
     \"service\", ref = \"api\" }\n\n[domain.dns]\nCNAME = [{ name = \"api.example.com\", \
     value = \"edge.example-cloud.net\" }]\n"
}

pub(crate) fn config_toml() -> &'static str {
    "bind = \"127.0.0.1:8080\"\n"
}

pub(crate) fn workers_schema() -> &'static str {
    "worker SendWelcomeEmail {\n  queue \"mail\"\n  mode static\n  concurrency 4\n  retry {\n    max_attempts 5\n  }\n  input {\n    user_id UUID\n    email String @email\n  }\n}\n"
}

pub(crate) fn docker_project() -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT_DOCKER: AtomicUsize = AtomicUsize::new(0);
    let dir = scratch_dir(&format!(
        "deploy-json-{}",
        NEXT_DOCKER.fetch_add(1, Ordering::Relaxed)
    ));
    write(&dir, "Cargo.toml", "[package]\nname = \"demo\"\n");
    let docker = dir.join("deploy/docker");
    std::fs::create_dir_all(&docker).expect("docker source directory");
    std::fs::write(docker.join("Dockerfile"), "FROM scratch\n").expect("Dockerfile");
    std::fs::write(docker.join("compose.yml"), "services: {}\n").expect("compose file");
    dir
}
