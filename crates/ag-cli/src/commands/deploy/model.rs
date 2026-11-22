//! Shared types and deterministic helpers for the local deployment renderer.
//!
//! Everything here is pure data or path/hash computation. No filesystem write
//! happens in this module; writes are confined to `apply.rs`.

use crate::output::{DeployFileView, DeployPlanData};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Component, Path};

/// Root of the authored Docker artifacts, relative to the project.
pub(crate) const DOCKER_DIR: &str = "deploy/docker";
/// Root of the authored Kubernetes artifacts, relative to the project.
pub(crate) const K8S_DIR: &str = "deploy/k8s";
/// Root of the rendered artifacts, relative to the project.
pub(crate) const GENERATED_DIR: &str = "deploy/generated";
/// Recorded state of the last apply, relative to the project.
pub(crate) const STATE_FILE: &str = "deploy/generated/state.json";
/// Append-only local audit record, relative to the project.
pub(crate) const AUDIT_FILE: &str = "deploy/generated/audit.log";
/// Plan identifier prefix.
pub(crate) const PLAN_PREFIX: &str = "dpl";

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

impl std::fmt::Display for DeployError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            DeployError::Usage(message) | DeployError::Failed(message) => message,
        };
        formatter.write_str(message)
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
    pub(crate) fn source_dir(self) -> &'static str {
        match self {
            DeployTarget::Docker => DOCKER_DIR,
            DeployTarget::K8s => K8S_DIR,
        }
    }
}

/// One artifact in a plan. `content` is kept for `apply` and never serialized.
#[derive(Debug, Clone)]
pub(crate) struct Artifact {
    pub(crate) source: String,
    pub(crate) destination: String,
    pub(crate) bytes: u64,
    pub(crate) fingerprint: String,
    pub(crate) content: Vec<u8>,
}

impl Artifact {
    pub(crate) fn view(&self) -> DeployFileView {
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
    pub(crate) target: DeployTarget,
    pub(crate) project: String,
    pub(crate) plan_id: String,
    pub(crate) artifacts: Vec<Artifact>,
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
pub(crate) struct DeployState {
    pub(crate) target: String,
    pub(crate) plan_id: String,
    pub(crate) project: String,
    pub(crate) files: Vec<String>,
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
pub(crate) fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// Lowercase, zero-padded hexadecimal rendering of a 64-bit fingerprint.
pub(crate) fn hex64(value: u64) -> String {
    format!("{value:016x}")
}

/// Length-prefixed field of the canonical plan stream. Prefixing avoids
/// ambiguity when a project name or path happens to contain separators.
pub(crate) fn push_field(out: &mut String, value: &str) {
    out.push_str(&value.len().to_string());
    out.push(':');
    out.push_str(value);
    out.push('\n');
}

/// Path as slash-separated `Normal` components, with no parent or root parts.
pub(crate) fn relative_slash(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Detects a project label from `Cargo.toml`, falling back to the directory.
pub(crate) fn project_label(root: &Path) -> String {
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
