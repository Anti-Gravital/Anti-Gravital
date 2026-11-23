//! Read-only `ag describe` (RFC-0024 section 4.2 `project.describe`).
//!
//! Describes the current project without mutating anything: which recognized
//! markers are present, which artifacts each source set would render (path,
//! source, size, fingerprint) and whether generated output is in sync with the
//! authored sources. Every path is relative; no secret, file content or
//! absolute path is emitted (S-1/S-8).
//!
//! One builder (`describe_report`) feeds both the prose and the JSON path, so
//! the two outputs cannot drift. An absent or incomplete source set is data,
//! not an error: `describe` must work on an empty directory, like
//! `deploy.status` does before the first apply.

use crate::commands::deploy::{build_plan, project_label, status_report, DeployTarget};
use crate::output::{DescribeArtifactView, DescribeData};
use std::path::Path;

/// Recognized project markers, in a fixed order. Only these fixed strings are
/// emitted: an arbitrary file name from the directory can never reach JSON.
const LAYOUT_MARKERS: [&str; 8] = [
    "Cargo.toml",
    "ag.toml",
    "schema.ag",
    "config.toml",
    "src",
    "crates",
    "deploy",
    "templates",
];

/// Both local artifact families, in a fixed order.
const TARGETS: [DeployTarget; 2] = [DeployTarget::Docker, DeployTarget::K8s];

/// Builds the versioned `project.describe` view from the current directory.
pub(crate) fn describe_report(root: &Path) -> Result<DescribeData, String> {
    // A malformed recorded state is a real read failure and is propagated.
    // A missing state file is already handled by `status_report` as neutral.
    let status = status_report(root)?;

    let layout = LAYOUT_MARKERS
        .iter()
        .filter(|marker| root.join(marker).exists())
        .map(|marker| (*marker).to_owned())
        .collect();

    let mut artifacts = Vec::new();
    for target in TARGETS {
        // An absent or incomplete source set is data, not a failure: describe
        // reports what is available and fabricates nothing.
        if let Ok(plan) = build_plan(root, target) {
            for artifact in plan.artifacts {
                let generated = root.join(&artifact.destination).is_file();
                artifacts.push(DescribeArtifactView {
                    target: target.as_str(),
                    path: artifact.destination,
                    source: artifact.source,
                    bytes: artifact.bytes,
                    fingerprint: artifact.fingerprint,
                    generated,
                });
            }
        }
    }
    artifacts.sort_by(|left, right| {
        left.target
            .cmp(right.target)
            .then_with(|| left.path.cmp(&right.path))
    });

    let generated = artifacts.iter().any(|artifact| artifact.generated);
    Ok(DescribeData {
        project: project_label(root),
        layout,
        artifacts,
        generated,
        in_sync: status.in_sync,
    })
}

/// `ag describe` (prose). Renders the same builder as the JSON path.
pub(crate) fn cmd_describe(root: &Path) -> Result<(), String> {
    let data = describe_report(root)?;
    println!("Project description");
    println!("Project: {}", data.project);
    println!(
        "Layout: {}",
        if data.layout.is_empty() {
            "none recognized".to_owned()
        } else {
            data.layout.join(", ")
        }
    );
    println!("Artifacts ({}):", data.artifacts.len());
    for artifact in &data.artifacts {
        println!(
            "  {} {} <- {} ({} bytes, {}, generated: {})",
            artifact.target,
            artifact.path,
            artifact.source,
            artifact.bytes,
            artifact.fingerprint,
            if artifact.generated { "yes" } else { "no" }
        );
    }
    println!(
        "Generated artifacts present: {}",
        if data.generated { "yes" } else { "no" }
    );
    println!(
        "In sync with current sources: {}",
        if data.in_sync { "yes" } else { "no" }
    );
    println!("Read-only: no files were written.");
    Ok(())
}
