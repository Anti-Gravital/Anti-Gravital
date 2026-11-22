//! Local resolved configuration: versioned, atomically written state file.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::model::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AppliedPlan {
    pub(crate) plan_id: String,
    pub(crate) domain: String,
    pub(crate) zone: String,
    pub(crate) environment: Environment,
    pub(crate) visibility: Visibility,
    pub(crate) project: String,
    pub(crate) service: String,
    pub(crate) target_kind: String,
    pub(crate) tls: TlsMode,
    pub(crate) records: Vec<PlannedRecord>,
}

impl From<&DnsPlan> for AppliedPlan {
    fn from(plan: &DnsPlan) -> Self {
        Self {
            plan_id: plan.plan_id.clone(),
            domain: plan.domain.clone(),
            zone: plan.zone.clone(),
            environment: plan.environment,
            visibility: plan.visibility,
            project: plan.project.clone(),
            service: plan.service.clone(),
            target_kind: plan.target_kind.clone(),
            tls: plan.tls,
            records: plan.records.clone(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct DnsState {
    pub(crate) version: u32,
    #[serde(default)]
    pub(crate) applied: BTreeMap<String, AppliedPlan>,
}

pub(crate) fn load_state(path: &Path) -> Result<DnsState, String> {
    if !path.exists() {
        return Ok(DnsState {
            version: STATE_VERSION,
            applied: BTreeMap::new(),
        });
    }
    let text = fs::read_to_string(path)
        .map_err(|e| format!("could not read state '{}': {e}", path.display()))?;
    let state: DnsState = serde_json::from_str(&text)
        .map_err(|e| format!("invalid state file '{}': {e}", path.display()))?;
    if state.version != STATE_VERSION {
        return Err(format!(
            "unsupported state version {} in '{}'; expected {}",
            state.version,
            path.display(),
            STATE_VERSION
        ));
    }
    Ok(state)
}

pub(crate) fn save_state(path: &Path, state: &DnsState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("could not create '{}': {e}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    let temp = path.with_extension("json.tmp");
    fs::write(&temp, format!("{text}\n"))
        .map_err(|e| format!("could not write '{}': {e}", temp.display()))?;
    fs::rename(&temp, path).map_err(|e| format!("could not commit '{}': {e}", path.display()))?;
    Ok(())
}
