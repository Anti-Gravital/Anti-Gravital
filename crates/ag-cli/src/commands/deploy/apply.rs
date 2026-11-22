//! Mutating `ag deploy apply`: confined writes for artifacts, state and audit.

use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};
use std::io::Write;
use std::path::Path;

use super::model::{DeployError, DeployState, AUDIT_FILE, GENERATED_DIR, STATE_FILE};
use super::plan::{build_plan, parse_plan_id};

/// Opens `deploy/generated/` as a capability rooted at the project directory.
/// Every write in `apply` goes through this handle: cap-std walks each parent
/// without following symlinks outside the root (RFC-0013), so a hostile
/// project cannot redirect artifacts, state or audit lines elsewhere.
fn generated_capability(root: &Path) -> Result<Dir, DeployError> {
    let project = Dir::open_ambient_dir(root, ambient_authority())?;
    project.create_dir_all(GENERATED_DIR)?;
    Ok(project.open_dir(GENERATED_DIR)?)
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
