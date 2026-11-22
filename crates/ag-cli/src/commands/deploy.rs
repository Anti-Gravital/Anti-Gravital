//! Local deployment artifact rendering (`ag deploy`).
//!
//! Capability: copy already-authored Docker and Kubernetes artifacts from a
//! project into `deploy/generated/`. This module deliberately has no network,
//! container-runtime or cluster dependency: it never starts a subprocess and
//! never opens a socket (RFC-0034). Sources are discovered without following
//! symlinks; writes go through a `cap_std::fs::Dir` rooted at
//! `deploy/generated/`, so neither artifacts, state nor audit lines can be
//! redirected outside the project by a hostile symlink (RFC-0013).

use crate::output::{DeployFileView, DeployPlanData, DeployStatusData};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

/// Root of the authored Docker artifacts, relative to the project.
const DOCKER_DIR: &str = "deploy/docker";
/// Root of the authored Kubernetes artifacts, relative to the project.
const K8S_DIR: &str = "deploy/k8s";
/// Root of the rendered artifacts, relative to the project.
const GENERATED_DIR: &str = "deploy/generated";
/// Recorded state of the last apply, relative to the project.
const STATE_FILE: &str = "deploy/generated/state.json";
/// Append-only local audit record, relative to the project.
const AUDIT_FILE: &str = "deploy/generated/audit.log";
/// Plan identifier prefix.
const PLAN_PREFIX: &str = "dpl";

/// Failure classes of the deploy capability.
#[derive(Debug)]
pub(crate) enum DeployError {
    /// Bad invocation: invalid target or malformed/missing plan identifier.
    Usage(String),
    /// Precondition, drift or I/O failure.
    Failed(String),
}

impl From<io::Error> for DeployError {
    fn from(error: io::Error) -> Self {
        DeployError::Failed(error.to_string())
    }
}

/// The two supported local targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeployTarget {
    /// Authored Dockerfile and compose set.
    Docker,
    /// Authored Kubernetes manifests including a kustomization root.
    K8s,
}

impl DeployTarget {
    /// Stable token used in the CLI, plan id and state file.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            DeployTarget::Docker => "docker",
            DeployTarget::K8s => "k8s",
        }
    }

    /// Parses a target token from clap or from a recorded state file.
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "docker" => Some(DeployTarget::Docker),
            "k8s" => Some(DeployTarget::K8s),
            _ => None,
        }
    }

    /// Source directory inspected for this target.
    fn source_dir(self) -> &'static str {
        match self {
            DeployTarget::Docker => DOCKER_DIR,
            DeployTarget::K8s => K8S_DIR,
        }
    }
}

/// One artifact in a plan. `content` is kept for `apply` and never serialized.
#[derive(Debug, Clone)]
struct Artifact {
    source: String,
    destination: String,
    bytes: u64,
    fingerprint: String,
    content: Vec<u8>,
}

impl Artifact {
    fn view(&self) -> DeployFileView {
        DeployFileView {
            path: self.destination.clone(),
            source: self.source.clone(),
            bytes: self.bytes,
            fingerprint: self.fingerprint.clone(),
        }
    }
}

/// A deterministic, read-only rendering plan.
#[derive(Debug, Clone)]
pub(crate) struct DeployPlan {
    target: DeployTarget,
    project: String,
    plan_id: String,
    artifacts: Vec<Artifact>,
}

impl DeployPlan {
    /// Machine-readable view used by the `deploy.plan` JSON shape.
    pub(crate) fn view(&self) -> DeployPlanData {
        DeployPlanData {
            target: self.target.as_str(),
            project: self.project.clone(),
            plan_id: self.plan_id.clone(),
            files: self.artifacts.iter().map(Artifact::view).collect(),
        }
    }
}

/// State written by `apply` and read by `status`.
#[derive(Debug, Serialize, Deserialize)]
struct DeployState {
    target: String,
    plan_id: String,
    project: String,
    files: Vec<String>,
}

/// Minimal view of `Cargo.toml` used only to label a plan.
#[derive(Deserialize)]
struct CargoName {
    package: Option<CargoPackage>,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: Option<String>,
}

/// FNV-1a 64-bit. Used as a deterministic identity and drift detector only
/// (RFC-0034 section 5.4); it is not a cryptographic signature.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// Lowercase, zero-padded hexadecimal rendering of a 64-bit fingerprint.
fn hex64(value: u64) -> String {
    format!("{value:016x}")
}

/// Length-prefixed field of the canonical plan stream. Prefixing avoids
/// ambiguity when a project name or path happens to contain separators.
fn push_field(out: &mut String, value: &str) {
    out.push_str(&value.len().to_string());
    out.push(':');
    out.push_str(value);
    out.push('\n');
}

/// Path as slash-separated `Normal` components, with no parent or root parts.
fn relative_slash(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Whether a file name belongs to the documented Docker source set.
fn is_docker_source(name: &str) -> bool {
    name == "Dockerfile"
        || name.starts_with("Dockerfile.")
        || name.ends_with(".Dockerfile")
        || name == ".dockerignore"
        || name == ".env.example"
        || matches!(
            name,
            "docker-compose.yml" | "docker-compose.yaml" | "compose.yml" | "compose.yaml"
        )
}

/// Whether a relative path is one of the compose files under `compose/`.
fn is_compose_file(relative: &str) -> bool {
    relative.starts_with("compose/") && (relative.ends_with(".yml") || relative.ends_with(".yaml"))
}

/// Whether a file name belongs to the documented Kubernetes source set.
fn is_k8s_source(name: &str) -> bool {
    name == "kustomization.yaml"
        || name == "kustomization.yml"
        || name.ends_with(".yaml")
        || name.ends_with(".yml")
}

/// The required marker that proves the source set is complete enough to render.
fn has_marker(paths: &[PathBuf], target: DeployTarget) -> bool {
    paths.iter().any(|path| {
        let name = path
            .file_name()
            .and_then(|part| part.to_str())
            .unwrap_or("");
        match target {
            DeployTarget::Docker => {
                name == "Dockerfile"
                    || name.starts_with("Dockerfile.")
                    || name.ends_with(".Dockerfile")
            }
            DeployTarget::K8s => name == "kustomization.yaml" || name == "kustomization.yml",
        }
    })
}

/// Walks a source tree without following symlinks and returns accepted files.
fn collect_sources(root: &Path, target: DeployTarget) -> Result<Vec<PathBuf>, DeployError> {
    let source_root = root.join(target.source_dir());
    if !source_root.is_dir() {
        let expected = match target {
            DeployTarget::Docker => {
                "a Dockerfile* (plus optional .dockerignore, .env.example and compose/*.yml)"
            }
            DeployTarget::K8s => "a kustomization.yaml or kustomization.yml root (plus manifests)",
        };
        return Err(DeployError::Failed(format!(
            "deployment source set for '{}' is not available: '{}' is missing; expected {expected} there (no file was written)",
            target.as_str(),
            source_root.display()
        )));
    }

    let mut found = Vec::new();
    walk(&source_root, &source_root, target, &mut found)?;
    found.sort();

    if found.is_empty() || !has_marker(&found, target) {
        let expected = match target {
            DeployTarget::Docker => {
                "a Dockerfile* (plus optional .dockerignore, .env.example and compose/*.yml)"
            }
            DeployTarget::K8s => "a kustomization.yaml or kustomization.yml root (plus manifests)",
        };
        return Err(DeployError::Failed(format!(
            "deployment source set for '{}' is incomplete: expected {expected} under '{}'; no manifest was fabricated and no file was written",
            target.as_str(),
            source_root.display()
        )));
    }
    Ok(found)
}

/// Recursive directory walk. `file_type` does not follow symlinks, so a link
/// inside the authored tree can never redirect the renderer outside the root.
fn walk(
    source_root: &Path,
    directory: &Path,
    target: DeployTarget,
    found: &mut Vec<PathBuf>,
) -> Result<(), DeployError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();
        if file_type.is_dir() {
            walk(source_root, &path, target, found)?;
        } else if file_type.is_file() {
            let name = path
                .file_name()
                .and_then(|part| part.to_str())
                .unwrap_or("");
            let relative = relative_slash(
                path.strip_prefix(source_root)
                    .expect("walked path is inside its source root"),
            );
            let accepted = match target {
                DeployTarget::Docker => is_docker_source(name) || is_compose_file(&relative),
                DeployTarget::K8s => is_k8s_source(name),
            };
            if accepted && path.starts_with(source_root) {
                found.push(path);
            }
        }
    }
    Ok(())
}

/// Opens `deploy/generated/` as a capability rooted at the project directory.
/// Every write in `apply` goes through this handle: cap-std walks each parent
/// without following symlinks outside the root (RFC-0013), so a hostile
/// project cannot redirect artifacts, state or audit lines elsewhere.
fn generated_capability(root: &Path) -> Result<Dir, DeployError> {
    let project = Dir::open_ambient_dir(root, ambient_authority())?;
    project.create_dir_all(GENERATED_DIR)?;
    Ok(project.open_dir(GENERATED_DIR)?)
}

/// Detects a project label from `Cargo.toml`, falling back to the directory.
fn project_label(root: &Path) -> String {
    if let Ok(text) = fs::read_to_string(root.join("Cargo.toml")) {
        if let Ok(manifest) = toml::from_str::<CargoName>(&text) {
            if let Some(name) = manifest.package.and_then(|package| package.name) {
                if !name.is_empty() {
                    return name;
                }
            }
        }
    }
    root.file_name()
        .and_then(|part| part.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("project")
        .to_owned()
}

/// Builds the read-only plan for one target from the current project state.
pub(crate) fn build_plan(root: &Path, target: DeployTarget) -> Result<DeployPlan, DeployError> {
    let sources = collect_sources(root, target)?;
    let project = project_label(root);
    let mut artifacts = Vec::with_capacity(sources.len());

    for source in sources {
        let relative = relative_slash(
            source
                .strip_prefix(root.join(target.source_dir()))
                .map_err(|_| DeployError::Failed("source escaped its root".to_owned()))?,
        );
        let content = fs::read(&source)?;
        let fingerprint = hex64(fnv1a(&content));
        let destination = Path::new(GENERATED_DIR)
            .join(target.as_str())
            .join(&relative);
        artifacts.push(Artifact {
            source: format!("{}/{relative}", target.source_dir()),
            destination: relative_slash(&destination),
            bytes: content.len() as u64,
            fingerprint,
            content,
        });
    }
    artifacts.sort_by(|left, right| left.destination.cmp(&right.destination));

    let mut canonical = String::from("ag-deploy-plan-v1\n");
    push_field(&mut canonical, target.as_str());
    push_field(&mut canonical, &project);
    for artifact in &artifacts {
        push_field(&mut canonical, &artifact.source);
        push_field(&mut canonical, &artifact.bytes.to_string());
        push_field(&mut canonical, &artifact.fingerprint);
    }
    let plan_id = format!(
        "{PLAN_PREFIX}-{}-{}",
        target.as_str(),
        hex64(fnv1a(canonical.as_bytes()))
    );

    Ok(DeployPlan {
        target,
        project,
        plan_id,
        artifacts,
    })
}

/// Extracts the target from a plan id and validates the whole identifier.
pub(crate) fn parse_plan_id(plan_id: &str) -> Result<DeployTarget, DeployError> {
    let body = plan_id
        .strip_prefix(&format!("{PLAN_PREFIX}-"))
        .ok_or_else(|| DeployError::Usage("plan id must start with 'dpl-'".to_owned()))?;
    let (target_token, hash) = body
        .split_once('-')
        .ok_or_else(|| DeployError::Usage("malformed plan id".to_owned()))?;
    let target = DeployTarget::parse(target_token)
        .ok_or_else(|| DeployError::Usage("plan id has an unknown target".to_owned()))?;
    if hash.len() != 16
        || !hash
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(DeployError::Usage(
            "plan id must end with 16 lowercase hexadecimal characters".to_owned(),
        ));
    }
    Ok(target)
}

/// `ag deploy plan` (prose). Never writes to the filesystem.
pub(crate) fn cmd_deploy_plan(root: &Path, target: DeployTarget) -> Result<(), DeployError> {
    let plan = build_plan(root, target)?;
    println!("Deployment plan ({})", target.as_str());
    println!("Project: {}", plan.project);
    println!("Plan ID: {}", plan.plan_id);
    println!("Files ({}):", plan.artifacts.len());
    for artifact in &plan.artifacts {
        println!(
            "  {} <- {} ({} bytes, {})",
            artifact.destination, artifact.source, artifact.bytes, artifact.fingerprint
        );
    }
    println!(
        "No files were written. Review, then: ag deploy apply --plan {}",
        plan.plan_id
    );
    Ok(())
}

/// Read-only data builder for the `deploy.plan` JSON shape.
pub(crate) fn plan_report(root: &Path, target: DeployTarget) -> Result<DeployPlanData, String> {
    build_plan(root, target)
        .map(|plan| plan.view())
        .map_err(|error| error.to_string())
}

/// `ag deploy apply --plan ID` (mutating). Requires a current, matching plan.
pub(crate) fn cmd_deploy_apply(root: &Path, plan_id: &str) -> Result<(), DeployError> {
    let target = parse_plan_id(plan_id)?;
    let plan = build_plan(root, target)?;
    if plan.plan_id != plan_id {
        return Err(DeployError::Failed(format!(
            "plan '{plan_id}' is stale: current plan is '{}'; re-run ag deploy plan and review it before applying",
            plan.plan_id
        )));
    }

    let generated = generated_capability(root)?;
    for artifact in &plan.artifacts {
        let relative = Path::new(&artifact.destination)
            .strip_prefix(GENERATED_DIR)
            .expect("destination is under deploy/generated");
        if let Some(parent) = relative.parent() {
            if !parent.as_os_str().is_empty() {
                generated.create_dir_all(parent)?;
            }
        }
        generated.write(relative, &artifact.content)?;
    }

    let state = DeployState {
        target: target.as_str().to_owned(),
        plan_id: plan.plan_id.clone(),
        project: plan.project.clone(),
        files: plan
            .artifacts
            .iter()
            .map(|artifact| artifact.destination.clone())
            .collect(),
    };
    generated.write(
        "state.json",
        serde_json::to_string(&state).expect("state is serializable"),
    )?;

    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let mut audit = generated.open_with(
        "audit.log",
        CapOpenOptions::new().create(true).append(true).write(true),
    )?;
    writeln!(
        audit,
        "{seconds} apply {} files={}",
        plan.plan_id,
        plan.artifacts.len()
    )?;

    println!(
        "Applied plan {} ({} files) under {}/{}.",
        plan.plan_id,
        plan.artifacts.len(),
        GENERATED_DIR,
        target.as_str()
    );
    println!("State: {STATE_FILE}");
    println!("Audit: {AUDIT_FILE}");
    Ok(())
}

/// Builds the read-only `deploy.status` view from recorded local state.
pub(crate) fn status_report(root: &Path) -> Result<DeployStatusData, String> {
    let text = match fs::read_to_string(root.join(STATE_FILE)) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(DeployStatusData::empty())
        }
        Err(error) => return Err(format!("cannot read deployment state: {error}")),
    };
    let state: DeployState = serde_json::from_str(&text)
        .map_err(|error| format!("invalid deployment state file: {error}"))?;
    let target = DeployTarget::parse(&state.target);

    let mut current_plan_id = None;
    let mut sources_available = false;
    let mut in_sync = false;
    if let Some(target) = target {
        if let Ok(plan) = build_plan(root, target) {
            current_plan_id = Some(plan.plan_id.clone());
            sources_available = true;
            in_sync = plan.plan_id == state.plan_id;
        }
    }

    Ok(DeployStatusData {
        generated: true,
        target: Some(state.target),
        plan_id: Some(state.plan_id),
        project: Some(state.project),
        files: state.files,
        current_plan_id,
        in_sync,
        sources_available,
    })
}

/// `ag deploy status` (prose). Never writes to the filesystem.
pub(crate) fn cmd_deploy_status(root: &Path) -> Result<(), DeployError> {
    let status = status_report(root).map_err(DeployError::Failed)?;
    if !status.generated {
        println!("No deployment artifacts generated under {GENERATED_DIR}/.");
        return Ok(());
    }
    println!("Deployment status");
    println!("Target: {}", status.target.as_deref().unwrap_or("unknown"));
    println!(
        "Plan ID: {}",
        status.plan_id.as_deref().unwrap_or("unknown")
    );
    println!(
        "Project: {}",
        status.project.as_deref().unwrap_or("unknown")
    );
    println!("Generated files ({}):", status.files.len());
    for path in &status.files {
        println!("  {path}");
    }
    println!(
        "Sources available: {}",
        if status.sources_available {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "In sync with current sources: {}",
        if status.in_sync { "yes" } else { "no" }
    );
    Ok(())
}

impl std::fmt::Display for DeployError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            DeployError::Usage(message) | DeployError::Failed(message) => message,
        };
        formatter.write_str(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

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
        symlink(outside.join("target.txt"), rendered.join("Dockerfile"))
            .expect("malicious symlink");

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
}
