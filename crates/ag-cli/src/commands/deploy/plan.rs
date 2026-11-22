//! Read-only plan construction: source discovery, fingerprinting and display.

use crate::output::DeployPlanData;
use std::fs;
use std::path::{Path, PathBuf};

use super::model::{
    fnv1a, hex64, project_label, push_field, relative_slash, Artifact, DeployError, DeployPlan,
    DeployTarget, GENERATED_DIR, PLAN_PREFIX,
};

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
