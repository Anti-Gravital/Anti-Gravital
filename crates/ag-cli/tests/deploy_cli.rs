//! End-to-end tests for the local `ag deploy` capability (RFC-0034).
//!
//! Every test runs the real `ag` binary in a scratch project. No Docker daemon,
//! Kubernetes cluster, registry or network service is needed or contacted.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn scratch(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ag-deploy-cli-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("parent directory");
    std::fs::write(path, content).expect("fixture file");
}

fn ag(args: &[&str], dir: &Path, path: Option<&Path>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ag"));
    command
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .env_remove("CI")
        .env_remove("AG_NO_TUI");
    if let Some(path) = path {
        command.env("PATH", path);
    }
    command.output().expect("run ag")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("UTF-8 stdout")
}

fn project(label: &str) -> PathBuf {
    let root = scratch(label);
    write(&root, "Cargo.toml", "[package]\nname = \"demo\"\n");
    root
}

fn docker_project(label: &str) -> PathBuf {
    let root = project(label);
    write(
        &root,
        "deploy/docker/Dockerfile",
        "FROM scratch\nCOPY app /app\n",
    );
    write(&root, "deploy/docker/compose.yml", "services: {}\n");
    root
}

fn k8s_project(label: &str) -> PathBuf {
    let root = project(label);
    write(
        &root,
        "deploy/k8s/kustomization.yaml",
        "apiVersion: kustomize.config.k8s.io/v1beta1\nkind: Kustomization\nresources:\n  - deployment.yaml\n",
    );
    write(&root, "deploy/k8s/deployment.yaml", "kind: Deployment\n");
    root
}

fn plan_id(output: &Output) -> String {
    stdout(output)
        .lines()
        .find_map(|line| line.strip_prefix("Plan ID: "))
        .expect("plan id line")
        .trim()
        .to_owned()
}

#[test]
fn plan_apply_status_parity_for_docker() {
    let root = docker_project("parity-docker");
    let plan = ag(&["deploy", "plan", "--target", "docker"], &root, None);
    assert!(plan.status.success(), "{plan:?}");
    let id = plan_id(&plan);
    assert!(id.starts_with("dpl-docker-"), "{id}");
    assert!(stdout(&plan).contains("No files were written"));
    assert!(!root.join("deploy/generated").exists());

    let applied = ag(&["deploy", "apply", "--plan", &id], &root, None);
    assert!(applied.status.success(), "{applied:?}");
    assert_eq!(
        std::fs::read(root.join("deploy/generated/docker/Dockerfile"))
            .expect("rendered Dockerfile"),
        b"FROM scratch\nCOPY app /app\n"
    );
    assert_eq!(
        std::fs::read(root.join("deploy/generated/docker/compose.yml")).expect("rendered compose"),
        b"services: {}\n"
    );

    let status = ag(&["deploy", "status"], &root, None);
    assert!(status.status.success(), "{status:?}");
    let text = stdout(&status);
    assert!(text.contains("Target: docker"), "{text}");
    assert!(text.contains("In sync with current sources: yes"), "{text}");
}

#[test]
fn plan_apply_parity_for_k8s() {
    let root = k8s_project("parity-k8s");
    let plan = ag(&["deploy", "plan", "--target", "k8s"], &root, None);
    assert!(plan.status.success(), "{plan:?}");
    let id = plan_id(&plan);
    assert!(id.starts_with("dpl-k8s-"), "{id}");

    let applied = ag(&["deploy", "apply", "--plan", &id], &root, None);
    assert!(applied.status.success(), "{applied:?}");
    assert_eq!(
        std::fs::read(root.join("deploy/generated/k8s/deployment.yaml"))
            .expect("rendered manifest"),
        b"kind: Deployment\n"
    );
}

#[test]
fn apply_without_a_plan_id_is_refused_before_writing() {
    let root = docker_project("no-plan");
    let output = ag(&["deploy", "apply"], &root, None);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(!root.join("deploy/generated").exists());
}

#[test]
fn apply_with_a_malformed_plan_id_is_refused_before_writing() {
    let root = docker_project("bad-plan");
    for id in ["dpl-docker-short", "dpl-nomad-0123456789abcdef", "raw-id"] {
        let output = ag(&["deploy", "apply", "--plan", id], &root, None);
        assert_eq!(output.status.code(), Some(2), "{id}: {output:?}");
        assert!(!root.join("deploy/generated").exists());
    }
}

#[test]
fn a_stale_plan_is_refused_before_writing() {
    let root = docker_project("stale-plan");
    let plan = ag(&["deploy", "plan", "--target", "docker"], &root, None);
    let id = plan_id(&plan);
    write(&root, "deploy/docker/Dockerfile", "FROM alpine\n");

    let output = ag(&["deploy", "apply", "--plan", &id], &root, None);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("stale"),
        "{output:?}"
    );
    assert!(!root.join("deploy/generated").exists());
}

#[test]
fn missing_sources_fail_without_fabricating_manifests() {
    let root = project("missing-sources");
    let output = ag(&["deploy", "plan", "--target", "k8s"], &root, None);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let message = String::from_utf8_lossy(&output.stderr);
    assert!(message.contains("kustomization"), "{message}");
    assert!(!root.join("deploy/generated").exists());
}

#[test]
fn implementation_has_no_network_or_subprocess_primitives() {
    let source = include_str!("../src/commands/deploy.rs");
    for forbidden in [
        "std::net",
        "TcpStream",
        "TcpListener",
        "UdpSocket",
        "tokio::net",
        "reqwest",
        "process::Command",
        "Command::new",
    ] {
        assert!(
            !source.contains(forbidden),
            "deploy renderer must not use {forbidden}"
        );
    }
}

#[cfg(unix)]
#[test]
fn no_container_or_cluster_command_is_invoked() {
    use std::os::unix::fs::PermissionsExt;

    let root = docker_project("sentinel");
    let marker = scratch("sentinel-marker").join("invoked");
    let bin = scratch("sentinel-bin");
    for tool in ["docker", "kubectl", "podman", "helm", "curl"] {
        let script = bin.join(tool);
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\necho {tool} >> \"{}\"\nexit 0\n",
                marker.display()
            ),
        )
        .expect("sentinel script");
        let mut permissions = std::fs::metadata(&script).expect("metadata").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script, permissions).expect("chmod");
    }

    let plan = ag(&["deploy", "plan", "--target", "docker"], &root, Some(&bin));
    assert!(plan.status.success(), "{plan:?}");
    let id = plan_id(&plan);
    let applied = ag(&["deploy", "apply", "--plan", &id], &root, Some(&bin));
    assert!(applied.status.success(), "{applied:?}");
    let status = ag(&["deploy", "status"], &root, Some(&bin));
    assert!(status.status.success(), "{status:?}");

    assert!(
        !marker.exists(),
        "a container, cluster or network command was invoked"
    );
}
