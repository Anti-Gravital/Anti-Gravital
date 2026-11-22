//! Unit tests for the local deployment renderer.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::apply::cmd_deploy_apply;
use super::model::{DeployTarget, AUDIT_FILE, STATE_FILE};
use super::plan::{build_plan, parse_plan_id};
use super::status::status_report;

static NEXT_SCRATCH: AtomicUsize = AtomicUsize::new(0);

fn scratch(label: &str) -> PathBuf {
    let unique = NEXT_SCRATCH.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "ag-deploy-test-{label}-{}-{unique}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("parent")).expect("parent directory");
    fs::write(path, content).expect("fixture file");
}

fn docker_project() -> PathBuf {
    let root = scratch("docker");
    write(&root, "Cargo.toml", "[package]\nname = \"demo\"\n");
    write(
        &root,
        "deploy/docker/Dockerfile",
        "FROM scratch\nCOPY app /app\n",
    );
    write(&root, "deploy/docker/compose.yml", "services: {}\n");
    root
}

#[test]
fn plan_is_deterministic_and_target_scoped() {
    let root = docker_project();
    write(&root, "deploy/k8s/kustomization.yaml", "resources: []\n");

    let first = build_plan(&root, DeployTarget::Docker).expect("docker plan");
    let second = build_plan(&root, DeployTarget::Docker).expect("docker plan");
    assert_eq!(first.plan_id, second.plan_id);
    assert!(first.plan_id.starts_with("dpl-docker-"));
    assert_eq!(first.plan_id.len(), "dpl-docker-".len() + 16);

    let k8s = build_plan(&root, DeployTarget::K8s).expect("k8s plan");
    assert!(k8s.plan_id.starts_with("dpl-k8s-"));
    assert_ne!(first.plan_id, k8s.plan_id);
}

#[test]
fn plan_copies_verbatim_and_sorts_destinations() {
    let root = scratch("verbatim");
    write(&root, "Cargo.toml", "[package]\nname = \"demo\"\n");
    write(&root, "deploy/docker/z/Dockerfile", "FROM scratch\n");
    write(&root, "deploy/docker/a/compose.yml", "services: {}\n");

    let plan = build_plan(&root, DeployTarget::Docker).expect("plan");
    let paths: Vec<_> = plan
        .artifacts
        .iter()
        .map(|a| a.destination.clone())
        .collect();
    assert_eq!(
        paths,
        vec![
            "deploy/generated/docker/a/compose.yml",
            "deploy/generated/docker/z/Dockerfile",
        ]
    );
    assert_eq!(plan.artifacts[1].content, b"FROM scratch\n");
    assert_eq!(plan.artifacts[1].bytes, 13);
}

#[test]
fn renders_the_documented_wave7_docker_filenames() {
    let root = scratch("wave7-docker");
    write(&root, "Cargo.toml", "[package]\nname = \"demo\"\n");
    write(&root, "deploy/docker/Dockerfile.template", "FROM scratch\n");
    write(&root, "deploy/docker/compose/api.yml", "services: {}\n");
    write(&root, "deploy/docker/.env.example", "PORT=8080\n");
    write(&root, "deploy/docker/README.md", "not an artifact\n");

    let plan = build_plan(&root, DeployTarget::Docker).expect("plan");
    let sources: Vec<_> = plan.artifacts.iter().map(|a| a.source.clone()).collect();
    assert_eq!(
        sources,
        vec![
            "deploy/docker/.env.example",
            "deploy/docker/Dockerfile.template",
            "deploy/docker/compose/api.yml",
        ]
    );
}

#[test]
fn missing_source_set_fails_without_writing() {
    let root = scratch("missing");
    let error = build_plan(&root, DeployTarget::K8s).expect_err("missing sources");
    let message = error.to_string();
    assert!(message.contains("kustomization"), "{message}");
    assert!(!root.join("deploy/generated").exists());
}

#[test]
fn incomplete_source_set_does_not_fabricate_a_manifest() {
    let root = scratch("incomplete");
    write(&root, "deploy/k8s/deployment.yaml", "kind: Deployment\n");
    let error = build_plan(&root, DeployTarget::K8s).expect_err("no kustomization");
    assert!(error.to_string().contains("kustomization"));
    assert!(!root.join("deploy/generated").exists());
}

#[test]
fn plan_id_parsing_rejects_malformed_values() {
    assert_eq!(
        parse_plan_id("dpl-docker-0123456789abcdef").ok(),
        Some(DeployTarget::Docker)
    );
    assert!(parse_plan_id("dpl-k8s-0123456789abcdef").is_ok());
    assert!(parse_plan_id("0123456789abcdef").is_err());
    assert!(parse_plan_id("dpl-other-0123456789abcdef").is_err());
    assert!(parse_plan_id("dpl-docker-0123456789ABCDEf").is_err());
    assert!(parse_plan_id("dpl-docker-short").is_err());
}

#[test]
fn apply_requires_a_current_matching_plan_and_writes_verbatim() {
    let root = docker_project();
    let plan = build_plan(&root, DeployTarget::Docker).expect("plan");

    let stale = "dpl-docker-0123456789abcdef";
    let error = cmd_deploy_apply(&root, stale).expect_err("stale plan must be refused");
    assert!(error.to_string().contains("stale"));
    assert!(!root.join("deploy/generated").exists());

    cmd_deploy_apply(&root, &plan.plan_id).expect("apply");
    assert_eq!(
        fs::read(root.join("deploy/generated/docker/Dockerfile")).expect("rendered file"),
        b"FROM scratch\nCOPY app /app\n"
    );
    assert!(root.join(STATE_FILE).exists());
    assert!(root.join(AUDIT_FILE).exists());
}

#[cfg(unix)]
#[test]
fn apply_refuses_a_symlink_that_escapes_generated() {
    use std::os::unix::fs::symlink;

    let root = docker_project();
    let plan = build_plan(&root, DeployTarget::Docker).expect("plan");
    let outside = scratch("symlink-outside");
    write(&outside, "target.txt", "untouched\n");
    let rendered = root.join("deploy/generated/docker");
    fs::create_dir_all(&rendered).expect("rendered directory");
    symlink(outside.join("target.txt"), rendered.join("Dockerfile")).expect("malicious symlink");

    assert!(cmd_deploy_apply(&root, &plan.plan_id).is_err());
    assert_eq!(
        fs::read(outside.join("target.txt")).expect("outside file"),
        b"untouched\n"
    );
}

#[test]
fn status_detects_drift_after_sources_change() {
    let root = docker_project();
    let plan = build_plan(&root, DeployTarget::Docker).expect("plan");
    cmd_deploy_apply(&root, &plan.plan_id).expect("apply");

    let status = status_report(&root).expect("status");
    assert!(status.generated);
    assert!(status.in_sync);
    assert!(status.sources_available);
    assert_eq!(status.plan_id.as_deref(), Some(plan.plan_id.as_str()));
    assert_eq!(status.files.len(), 2);

    write(&root, "deploy/docker/Dockerfile", "FROM alpine\n");
    let changed = status_report(&root).expect("status after change");
    assert!(!changed.in_sync);
    assert_ne!(
        changed.current_plan_id.as_deref(),
        Some(plan.plan_id.as_str())
    );
}

#[test]
fn status_without_state_is_reported_as_not_generated() {
    let root = scratch("empty-status");
    let status = status_report(&root).expect("status");
    assert!(!status.generated);
    assert!(status.files.is_empty());
    assert!(!status.in_sync);
}
