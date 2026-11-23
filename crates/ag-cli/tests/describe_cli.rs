//! Contract and behaviour tests for `ag describe` (RFC-0024 section 4.2).
//!
//! Kept in a sibling test binary so neither this file nor `json_output.rs`
//! approaches the 600-line guardrail (RFC-0022 section 4.3). Every assertion
//! uses the real binary through `CARGO_BIN_EXE_ag`; no helper mutates the repo.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const SNAPSHOT: &str = include_str!("../../../docs/mcp/snapshots/project.describe.json");
const CANARY: &str = "CANARY-SECRET-9f3a";
const LAYOUT_SET: &[&str] = &[
    "Cargo.toml",
    "ag.toml",
    "schema.ag",
    "config.toml",
    "src",
    "crates",
    "deploy",
    "templates",
];

fn ag(args: &[&str], dir: &Path, envs: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ag"));
    command
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .env_remove("CI")
        .env_remove("AG_NO_TUI");
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("run ag")
}

fn stdout(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).expect("stdout is UTF-8")
}

fn write(dir: &Path, name: &str, contents: &str) {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("fixture parent");
    }
    std::fs::write(path, contents).expect("fixture write");
}

/// Deterministic project fixture whose JSON is the checked-in snapshot.
fn fixture(label: &str) -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "ag-describe-test-{label}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("fixture dir");
    write(&dir, "Cargo.toml", "[package]\nname = \"demo\"\n");
    write(
        &dir,
        "schema.ag",
        "model User {\n  id UUID @primary\n  name String\n}\n",
    );
    write(&dir, "deploy/docker/Dockerfile", "FROM scratch\n");
    write(&dir, "deploy/k8s/kustomization.yaml", "resources: []\n");
    dir
}

/// Recursive (path, bytes) view of a directory, sorted for stable comparison.
fn tree(dir: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in std::fs::read_dir(dir).expect("read_dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            let name = path
                .strip_prefix(base)
                .expect("inside base")
                .to_string_lossy()
                .into_owned();
            if path.is_dir() {
                walk(base, &path, out);
            } else {
                out.push((name, std::fs::read(&path).expect("read file")));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort_by(|left, right| left.0.cmp(&right.0));
    out
}

fn value(output: &Output) -> serde_json::Value {
    serde_json::from_str(stdout(output).trim()).expect("one compact JSON line")
}

#[test]
fn describe_json_matches_the_checked_in_snapshot() {
    let dir = fixture("snapshot");
    let output = ag(&["--output", "json", "describe"], &dir, &[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output).trim_end(), SNAPSHOT.trim_end());
    assert!(
        output.stderr.is_empty(),
        "JSON mode must not write to stderr"
    );
}

#[test]
fn describe_document_carries_the_contract_version() {
    let dir = fixture("version");
    let output = ag(&["--output", "json", "describe"], &dir, &[]);
    assert!(output.status.success(), "{output:?}");
    let doc = value(&output);
    assert_eq!(doc["contract_version"], "0.2.0");
    assert_eq!(doc["command"], "project.describe");
    assert_eq!(doc["status"], "ok");
    assert!(doc.get("data").is_some());
    assert!(doc.get("error").is_none());
}

#[test]
fn describe_data_has_a_closed_field_set() {
    let dir = fixture("closed");
    let output = ag(&["--output", "json", "describe"], &dir, &[]);
    assert!(output.status.success(), "{output:?}");
    let data = value(&output);
    let data = data["data"].as_object().expect("data object");

    let mut fields: Vec<&str> = data.keys().map(String::as_str).collect();
    fields.sort_unstable();
    assert_eq!(
        fields,
        ["artifacts", "generated", "in_sync", "layout", "project"]
    );

    let layout = data["layout"].as_array().expect("layout array");
    for marker in layout {
        let marker = marker.as_str().expect("layout string");
        assert!(
            LAYOUT_SET.contains(&marker),
            "unknown layout marker: {marker}"
        );
    }

    let artifacts = data["artifacts"].as_array().expect("artifacts array");
    for artifact in artifacts {
        let artifact = artifact.as_object().expect("artifact object");
        let mut names: Vec<&str> = artifact.keys().map(String::as_str).collect();
        names.sort_unstable();
        assert_eq!(
            names,
            [
                "bytes",
                "fingerprint",
                "generated",
                "path",
                "source",
                "target"
            ]
        );
        let target = artifact["target"].as_str().expect("target string");
        assert!(
            matches!(target, "docker" | "k8s"),
            "unknown target: {target}"
        );
        let fingerprint = artifact["fingerprint"].as_str().expect("fingerprint");
        assert_eq!(fingerprint.len(), 16);
        assert!(fingerprint
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
        let path = artifact["path"].as_str().expect("path");
        assert!(
            !path.starts_with('/'),
            "no absolute destination path: {path}"
        );
    }
}

#[test]
fn describe_never_leaks_secrets_or_absolute_paths() {
    let dir = fixture("canary");
    // The canary is in file *content* and in the process environment: a
    // fingerprint is a hash, so neither path can carry it into the document.
    write(
        &dir,
        "deploy/docker/Dockerfile",
        "FROM scratch\n# CANARY-SECRET-9f3a\n",
    );
    let output = ag(
        &["--output", "json", "describe"],
        &dir,
        &[("AG_SMTP_PASS", CANARY), ("DATABASE_URL", CANARY)],
    );
    assert!(output.status.success(), "{output:?}");
    let text = stdout(&output);
    assert!(!text.contains(CANARY), "secret leaked to stdout: {text}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains(CANARY),
        "secret leaked to stderr: {stderr}"
    );
    let absolute = dir.to_string_lossy();
    assert!(
        !text.contains(absolute.as_ref()),
        "absolute path leaked: {text}"
    );
}

#[test]
fn describe_prose_is_human_readable_and_stable() {
    let dir = fixture("prose");
    let first = ag(&["describe"], &dir, &[]);
    let second = ag(&["describe"], &dir, &[]);
    assert!(first.status.success(), "{first:?}");
    assert_eq!(
        stdout(&first),
        stdout(&second),
        "prose must be deterministic"
    );
    let prose = stdout(&first);
    for marker in [
        "Project description",
        "Project: demo",
        "Layout: Cargo.toml, schema.ag, deploy",
        "Artifacts (2):",
        "deploy/generated/docker/Dockerfile",
        "deploy/docker/Dockerfile",
        "13 bytes",
        "In sync with current sources: no",
        "Read-only: no files were written.",
    ] {
        assert!(prose.contains(marker), "missing prose marker: {marker}");
    }
    assert!(!prose.contains("contract_version"), "prose is not JSON");
    assert!(!prose.contains("{"), "prose is not JSON");
}

#[test]
fn describe_is_read_only_in_both_output_modes() {
    let dir = fixture("readonly");
    let before = tree(&dir);
    assert!(ag(&["describe"], &dir, &[]).status.success());
    assert!(ag(&["--output", "json", "describe"], &dir, &[])
        .status
        .success());
    assert_eq!(tree(&dir), before, "describe must not change any file");
}

#[test]
fn describe_reports_generated_versus_authored_state() {
    let dir = fixture("generated");
    let plan = ag(
        &["--output", "json", "deploy", "plan", "--target", "docker"],
        &dir,
        &[],
    );
    assert!(plan.status.success(), "{plan:?}");
    let plan_id = value(&plan)["data"]["plan_id"]
        .as_str()
        .expect("plan id")
        .to_owned();

    let applied = ag(&["deploy", "apply", "--plan", &plan_id], &dir, &[]);
    assert!(applied.status.success(), "{applied:?}");

    let output = ag(&["--output", "json", "describe"], &dir, &[]);
    assert!(output.status.success(), "{output:?}");
    let doc = value(&output);
    let data = doc["data"].as_object().expect("data object");
    assert_eq!(data["generated"], true, "a destination now exists");
    assert_eq!(data["in_sync"], true, "recorded state matches sources");
    let artifacts = data["artifacts"].as_array().expect("artifacts array");
    let docker = artifacts
        .iter()
        .find(|item| item["target"] == "docker")
        .expect("docker artifact");
    let k8s = artifacts
        .iter()
        .find(|item| item["target"] == "k8s")
        .expect("k8s artifact");
    assert_eq!(docker["generated"], true, "docker destination was rendered");
    assert_eq!(k8s["generated"], false, "k8s destination was not rendered");
}

#[test]
fn describe_on_an_empty_directory_is_a_neutral_document() {
    let dir = std::env::temp_dir().join(format!(
        "ag-describe-empty-{}-{}",
        std::process::id(),
        std::process::id() % 997
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("empty dir");
    let output = ag(&["--output", "json", "describe"], &dir, &[]);
    assert!(output.status.success(), "{output:?}");
    let doc = value(&output);
    assert_eq!(doc["command"], "project.describe");
    assert_eq!(doc["data"]["layout"], serde_json::json!([]));
    assert_eq!(doc["data"]["artifacts"], serde_json::json!([]));
    assert_eq!(doc["data"]["generated"], false);
    assert_eq!(doc["data"]["in_sync"], false);
}

#[test]
fn mutating_commands_still_refuse_json_and_name_the_new_shape() {
    let dir = fixture("refusal");
    let cases: [&[&str]; 3] = [
        &["--output", "json", "new", "demo"],
        &[
            "--output",
            "json",
            "deploy",
            "apply",
            "--plan",
            "CANARY-PLAN",
        ],
        &["--output", "json", "generate"],
    ];
    for args in cases {
        let output = ag(args, &dir, &[]);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        let doc = value(&output);
        assert_eq!(doc["status"], "error");
        assert_eq!(doc["error"]["code"], "unsupported_command");
        let message = doc["error"]["message"].as_str().expect("message");
        assert!(message.contains("project.describe"), "{message}");
        assert!(!message.contains("CANARY"), "argument leaked: {message}");
    }
    let before = tree(&dir);
    assert_eq!(tree(&dir), before, "a refusal must not write anything");
}
