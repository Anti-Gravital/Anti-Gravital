//! Read-only `ag deploy status` reporting and drift detection.

use crate::output::DeployStatusData;
use std::fs;
use std::io;
use std::path::Path;

use super::model::{DeployError, DeployState, DeployTarget, GENERATED_DIR, STATE_FILE};
use super::plan::build_plan;

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
