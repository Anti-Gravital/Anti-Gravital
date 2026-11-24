//! Shared test-only scaffolding for the `ag --output json` snapshot tests
//! (RFC-0021 section 4.3).
//!
//! This module holds only the helpers used by BOTH sibling test binaries
//! (`json_output` and `json_output_config`). Keeping it to the shared surface
//! means neither binary compiles dead code. Scenario fixtures used by a single
//! binary live in their own module (e.g. `json_output_scenarios`, included only
//! by `json_output`).
//!
//! The golden documents are checked in under `docs/mcp/snapshots/`. Comparison
//! is byte-exact: field order, spacing and version are part of the contract. A
//! change in emitted JSON must change the snapshot and `contract_version` in
//! the same commit.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

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
        "project.describe" => {
            include_str!("../../../../docs/mcp/snapshots/project.describe.json")
        }
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

pub(crate) fn config_toml() -> &'static str {
    "bind = \"127.0.0.1:8080\"\n"
}
