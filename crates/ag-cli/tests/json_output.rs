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
const SNAPSHOT_DIR: &str = "../../docs/mcp/snapshots";

fn snapshot(name: &str) -> &'static str {
    match name {
        "schema.lint.clean" => include_str!("../../../docs/mcp/snapshots/schema.lint.clean.json"),
        "schema.lint.warnings" => {
            include_str!("../../../docs/mcp/snapshots/schema.lint.warnings.json")
        }
        "schema.lint.errors" => {
            include_str!("../../../docs/mcp/snapshots/schema.lint.errors.json")
        }
        "schema.lint.read_failed" => {
            include_str!("../../../docs/mcp/snapshots/schema.lint.read_failed.json")
        }
        "schema.diff" => include_str!("../../../docs/mcp/snapshots/schema.diff.json"),
        "workers.list" => include_str!("../../../docs/mcp/snapshots/workers.list.json"),
        "unsupported_command" => {
            include_str!("../../../docs/mcp/snapshots/unsupported_command.json")
        }
        other => panic!("unknown snapshot: {other}"),
    }
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ag-json-test-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch directory");
    dir
}

fn write(dir: &Path, name: &str, contents: &str) {
    std::fs::write(dir.join(name), contents).expect("write fixture");
}

fn ag(args: &[&str], dir: &Path, env: &[(&str, &str)]) -> Output {
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

fn stdout(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).expect("stdout is UTF-8")
}

fn assert_snapshot(output: &Output, name: &str) {
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

fn clean_schema() -> &'static str {
    "model User {\n  id UUID @primary\n  name String\n}\n"
}

fn warning_schema() -> &'static str {
    "model Tag {\n  name String\n}\n"
}

fn invalid_schema() -> &'static str {
    "model { broken\n"
}

fn reference_schema() -> &'static str {
    "model Old {\n  id UUID @primary\n  name String\n}\n"
}

fn current_schema() -> &'static str {
    "model Old {\n  id UUID @primary\n  title String\n}\nmodel New {\n  id UUID @primary\n}\n"
}

fn workers_schema() -> &'static str {
    "worker SendWelcomeEmail {\n  queue \"mail\"\n  mode static\n  concurrency 4\n  retry {\n    max_attempts 5\n  }\n  input {\n    user_id UUID\n    email String @email\n  }\n}\n"
}

#[test]
fn schema_lint_clean_matches_snapshot() {
    let dir = scratch_dir("lint-clean");
    write(&dir, "schema.ag", clean_schema());
    let output = ag(&["--output", "json", "schema", "lint"], &dir, &[]);
    assert!(output.status.success(), "{output:?}");
    assert_snapshot(&output, "schema.lint.clean");
}

#[test]
fn schema_lint_warnings_matches_snapshot_with_flag_after_subcommand() {
    let dir = scratch_dir("lint-warn");
    write(&dir, "warn.ag", warning_schema());
    let output = ag(
        &["schema", "lint", "--schema", "warn.ag", "--output", "json"],
        &dir,
        &[],
    );
    assert!(output.status.success(), "{output:?}");
    assert_snapshot(&output, "schema.lint.warnings");
}

#[test]
fn schema_lint_errors_are_data_and_exit_one() {
    let dir = scratch_dir("lint-errors");
    write(&dir, "invalid.ag", invalid_schema());
    let output = ag(
        &[
            "--output",
            "json",
            "schema",
            "lint",
            "--schema",
            "invalid.ag",
        ],
        &dir,
        &[],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_snapshot(&output, "schema.lint.errors");
}

#[test]
fn schema_lint_missing_file_is_a_versioned_read_error() {
    let dir = scratch_dir("lint-missing");
    let output = ag(
        &[
            "--output",
            "json",
            "schema",
            "lint",
            "--schema",
            "missing.ag",
        ],
        &dir,
        &[],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_snapshot(&output, "schema.lint.read_failed");
}

#[test]
fn schema_diff_matches_snapshot() {
    let dir = scratch_dir("diff");
    write(&dir, "reference.ag", reference_schema());
    write(&dir, "schema_diff.ag", current_schema());
    let output = ag(
        &[
            "--output",
            "json",
            "schema",
            "diff",
            "reference.ag",
            "--schema",
            "schema_diff.ag",
        ],
        &dir,
        &[],
    );
    assert!(output.status.success(), "{output:?}");
    assert_snapshot(&output, "schema.diff");
}

#[test]
fn workers_list_matches_snapshot() {
    let dir = scratch_dir("workers");
    write(&dir, "workers.ag", workers_schema());
    let output = ag(
        &[
            "--output",
            "json",
            "workers",
            "list",
            "--schema",
            "workers.ag",
        ],
        &dir,
        &[],
    );
    assert!(output.status.success(), "{output:?}");
    assert_snapshot(&output, "workers.list");
}

#[test]
fn mutating_commands_refuse_json_before_doing_work() {
    let dir = scratch_dir("refuse-mutating");
    write(&dir, "schema.ag", clean_schema());

    let new = ag(&["--output", "json", "new", "demo"], &dir, &[]);
    assert_eq!(new.status.code(), Some(2));
    assert_snapshot(&new, "unsupported_command");
    assert!(
        !dir.join("demo").exists(),
        "a refused command created a project"
    );

    let generate = ag(
        &[
            "--output",
            "json",
            "generate",
            "--schema",
            "schema.ag",
            "--output",
            "generated",
        ],
        &dir,
        &[],
    );
    assert_eq!(generate.status.code(), Some(2));
    assert_snapshot(&generate, "unsupported_command");
    assert!(
        !dir.join("generated").exists(),
        "a refused command wrote artifacts"
    );
}

#[test]
fn read_commands_without_a_shape_are_refused() {
    let dir = scratch_dir("refuse-read");
    for args in [
        vec!["--output", "json", "domains", "list"],
        vec!["--output", "json", "workers", "doctor"],
    ] {
        let output = ag(&args, &dir, &[]);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert_snapshot(&output, "unsupported_command");
    }
}

#[test]
fn prose_output_is_unchanged_without_the_flag() {
    let dir = scratch_dir("prose");
    write(&dir, "schema.ag", clean_schema());
    let output = ag(&["schema", "lint", "--schema", "schema.ag"], &dir, &[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "'schema.ag': no issues found.\n");
    assert!(!stdout(&output).contains("contract_version"));
}

#[test]
fn canary_secrets_are_never_echoed_by_a_refusal() {
    let dir = scratch_dir("canary");
    const CANARY: &str = "CANARY-SECRET-9f3a";
    for args in [
        vec!["--output", "json", "mail", "test", "--to", CANARY],
        vec![
            "--output",
            "json",
            "domains",
            "sync",
            "--schema",
            "schema.ag",
            "--zone-id",
            CANARY,
            "--token",
            CANARY,
        ],
    ] {
        let output = ag(
            &args,
            &dir,
            &[
                ("AG_SMTP_PASS", CANARY),
                ("AG_CLOUDFLARE_TOKEN", CANARY),
                ("DATABASE_URL", CANARY),
            ],
        );
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(!stdout(&output).contains(CANARY));
        assert!(
            !String::from_utf8_lossy(&output.stderr).contains(CANARY),
            "secret leaked to stderr: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn every_supported_document_carries_the_contract_version() {
    let dir = scratch_dir("version");
    write(&dir, "schema.ag", clean_schema());
    write(&dir, "workers.ag", workers_schema());
    for args in [
        vec!["--output", "json", "schema", "lint"],
        vec![
            "--output",
            "json",
            "workers",
            "list",
            "--schema",
            "workers.ag",
        ],
    ] {
        let output = ag(&args, &dir, &[]);
        assert!(output.status.success(), "{args:?}: {output:?}");
        let value: serde_json::Value =
            serde_json::from_str(stdout(&output).trim()).expect("valid JSON");
        assert_eq!(value["contract_version"], "0.1.0");
        assert_eq!(value["status"], "ok");
        assert!(value.get("data").is_some());
        assert!(value.get("error").is_none());
    }
}

#[test]
fn snapshots_are_valid_json_with_a_matching_version() {
    for name in [
        "schema.lint.clean",
        "schema.lint.warnings",
        "schema.lint.errors",
        "schema.lint.read_failed",
        "schema.diff",
        "workers.list",
        "unsupported_command",
    ] {
        let value: serde_json::Value =
            serde_json::from_str(snapshot(name)).expect("snapshot is valid JSON");
        assert_eq!(value["contract_version"], "0.1.0", "{name}");
    }
    // Keep the directory honest if a file is moved.
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SNAPSHOT_DIR);
    assert!(
        dir.is_dir(),
        "missing snapshot directory: {}",
        dir.display()
    );
}
