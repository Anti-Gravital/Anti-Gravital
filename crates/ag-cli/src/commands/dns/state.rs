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

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_state() -> DnsState {
        let mut state = DnsState::default();
        state.version = STATE_VERSION;
        state.applied.insert(
            "production".to_owned(),
            AppliedPlan {
                plan_id: "dns-production-abc123".to_owned(),
                domain: "api.example.test".to_owned(),
                zone: "example.test".to_owned(),
                environment: Environment::Production,
                visibility: Visibility::Private,
                project: "demo".to_owned(),
                service: "api".to_owned(),
                target_kind: "service".to_owned(),
                tls: TlsMode::Disabled,
                records: Vec::new(),
            },
        );
        state
    }

    #[test]
    fn missing_state_file_loads_as_an_empty_default() {
        let dir = std::env::temp_dir().join(format!("ag-dns-state-default-{}", std::process::id()));
        let path = dir.join("state.json");
        let _ = std::fs::remove_dir_all(&dir);
        let state = load_state(&path).expect("missing state is not an error");
        assert_eq!(state.version, STATE_VERSION);
        assert!(state.applied.is_empty());
    }

    #[test]
    fn save_then_load_preserves_the_applied_plan() {
        let dir =
            std::env::temp_dir().join(format!("ag-dns-state-roundtrip-{}", std::process::id()));
        let path = dir.join("nested/state.json");
        let _ = std::fs::remove_dir_all(&dir);
        let original = sample_state();
        save_state(&path, &original).expect("state written");
        let loaded = load_state(&path).expect("state read");
        assert_eq!(loaded, original);
        assert!(path.exists());
        assert!(!path.with_extension("json.tmp").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn malformed_and_unsupported_state_files_fail_closed() {
        let dir = std::env::temp_dir().join(format!("ag-dns-state-bad-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("fixture dir");
        let bad = dir.join("bad.json");
        std::fs::write(&bad, "not json").expect("fixture");
        assert!(load_state(&bad).is_err());

        let future = dir.join("future.json");
        std::fs::write(&future, r#"{"version":99,"applied":{}}"#).expect("fixture");
        let error = load_state(&future).expect_err("unsupported version");
        assert!(error.contains("unsupported state version"), "{error}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
