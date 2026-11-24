//! Snapshot tests for `ag --output json` (RFC-0021 section 4.3).
//!
//! Every supported read command has a checked-in golden document under
//! `docs/mcp/snapshots/`. The comparison is byte-exact: field order, spacing
//! and version are part of the contract. A change in emitted JSON must change
//! the snapshot and `contract_version` in the same commit.

mod json_output_scenarios;
mod json_output_support;
use json_output_scenarios::*;
use json_output_support::*;
use std::path::PathBuf;

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
fn dns_plan_matches_snapshot_without_network_or_writes() {
    let dir = scratch_dir("dns-plan");
    write(&dir, "ag.toml", dns_manifest());
    let output = ag(
        &["--output", "json", "dns", "plan"],
        &dir,
        &[
            ("AG_CLOUDFLARE_TOKEN", "CANARY-NETWORK-TOKEN"),
            ("AG_DNS_ZONE_ID", "CANARY-ZONE"),
        ],
    );
    assert!(output.status.success(), "{output:?}");
    assert_snapshot(&output, "dns.plan");
    let value: serde_json::Value = serde_json::from_str(stdout(&output).trim()).expect("JSON");
    assert!(value["data"]["plan_id"]
        .as_str()
        .unwrap()
        .starts_with("dns-production-"));
    assert_eq!(value["data"]["records"].as_array().unwrap().len(), 2);
    assert!(!dir.join(".ag/dns/state.json").exists());
}

#[test]
fn dns_status_matches_snapshot_when_not_applied() {
    let dir = scratch_dir("dns-status");
    write(&dir, "ag.toml", dns_manifest());
    let output = ag(&["--output", "json", "dns", "status"], &dir, &[]);
    assert!(output.status.success(), "{output:?}");
    assert_snapshot(&output, "dns.status");
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
        vec![
            "--output",
            "json",
            "dns",
            "apply",
            "--plan",
            "dns-prod-canary",
        ],
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
fn dns_apply_refuses_json_without_creating_a_state_file() {
    let dir = scratch_dir("dns-refuse");
    write(&dir, "ag.toml", dns_manifest());
    let output = ag(
        &[
            "--output",
            "json",
            "dns",
            "apply",
            "--plan",
            "CANARY-PLAN-ID",
        ],
        &dir,
        &[("AG_CLOUDFLARE_TOKEN", "CANARY-TOKEN")],
    );
    assert_eq!(output.status.code(), Some(2));
    assert_snapshot(&output, "unsupported_command");
    assert!(!dir.join(".ag/dns/state.json").exists());
    assert!(!stdout(&output).contains("CANARY"));
}

#[test]
fn every_supported_document_carries_the_contract_version() {
    let dir = scratch_dir("version");
    write(&dir, "schema.ag", clean_schema());
    write(&dir, "workers.ag", workers_schema());
    write(&dir, "config.toml", config_toml());
    for args in [
        vec!["--output", "json", "schema", "lint"],
        vec!["--output", "json", "config", "get", "bind"],
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
        assert_eq!(value["contract_version"], "0.2.0");
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
        "dns.plan",
        "dns.status",
        "config.get",
        "env.list",
        "env.check",
        "project.describe",
        "unsupported_command",
    ] {
        let value: serde_json::Value =
            serde_json::from_str(snapshot(name)).expect("snapshot is valid JSON");
        assert_eq!(value["contract_version"], "0.2.0", "{name}");
    }
    // Keep the directory honest if a file is moved.
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SNAPSHOT_DIR);
    assert!(
        dir.is_dir(),
        "missing snapshot directory: {}",
        dir.display()
    );
}

#[test]
fn mutating_commands_refuse_json_before_doing_work_deploy() {
    let dir = scratch_dir("refuse-mutating-deploy");
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
fn read_commands_without_a_shape_are_refused_deploy() {
    let dir = scratch_dir("refuse-read-deploy");
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
fn every_supported_document_carries_the_contract_version_deploy() {
    let dir = scratch_dir("version-deploy");
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
        assert_eq!(value["contract_version"], "0.2.0");
        assert_eq!(value["status"], "ok");
        assert!(value.get("data").is_some());
        assert!(value.get("error").is_none());
    }
}

#[test]
fn snapshots_are_valid_json_with_a_matching_version_deploy() {
    for name in [
        "schema.lint.clean",
        "schema.lint.warnings",
        "schema.lint.errors",
        "schema.lint.read_failed",
        "schema.diff",
        "workers.list",
        "deploy.plan",
        "deploy.status",
        "config.get",
        "env.list",
        "env.check",
        "project.describe",
        "unsupported_command",
    ] {
        let value: serde_json::Value =
            serde_json::from_str(snapshot(name)).expect("snapshot is valid JSON");
        assert_eq!(value["contract_version"], "0.2.0", "{name}");
    }
    // Keep the directory honest if a file is moved.
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SNAPSHOT_DIR);
    assert!(
        dir.is_dir(),
        "missing snapshot directory: {}",
        dir.display()
    );
}

#[test]
fn deploy_plan_and_status_match_snapshots() {
    let dir = docker_project();
    let plan = ag(
        &["--output", "json", "deploy", "plan", "--target", "docker"],
        &dir,
        &[],
    );
    assert!(plan.status.success(), "{plan:?}");
    assert_snapshot(&plan, "deploy.plan");

    let value: serde_json::Value =
        serde_json::from_str(stdout(&plan).trim()).expect("valid plan JSON");
    let plan_id = value["data"]["plan_id"]
        .as_str()
        .expect("plan id")
        .to_owned();

    let applied = ag(&["deploy", "apply", "--plan", &plan_id], &dir, &[]);
    assert!(applied.status.success(), "{applied:?}");
    assert_eq!(
        std::fs::read(dir.join("deploy/generated/docker/Dockerfile")).expect("rendered file"),
        b"FROM scratch\n"
    );

    let status = ag(&["--output", "json", "deploy", "status"], &dir, &[]);
    assert!(status.status.success(), "{status:?}");
    assert_snapshot(&status, "deploy.status");
}

#[test]
fn deploy_apply_refuses_json_without_writing() {
    let dir = docker_project();
    let output = ag(
        &[
            "--output",
            "json",
            "deploy",
            "apply",
            "--plan",
            "dpl-docker-0123456789abcdef",
        ],
        &dir,
        &[],
    );
    assert_eq!(output.status.code(), Some(2));
    assert_snapshot(&output, "unsupported_command");
    assert!(
        !dir.join("deploy/generated").exists(),
        "a refused apply wrote artifacts"
    );
}
