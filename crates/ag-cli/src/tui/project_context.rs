//! Typed project context for the home screen (RFC-0018 extension, ADR-0014).
//!
//! All disk I/O happens here, at the boundary, before the alternate screen is
//! opened. The view (`home.rs`) only formats the resolved fields and never
//! reads the file system. A missing, stale or malformed state is `Unknown`,
//! never an assumed `running` (K-10).

use std::time::SystemTime;

use crate::project_state::{self, DevState};

/// Provenance of the run status. Only two honest values exist: a verified
/// fresh state (`Real`) or the absence of one (`Unknown`). There is no
/// third "supposed" value (context design §2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StatusSource {
    Real,
    Unknown,
}

/// Snapshot of the current directory resolved before the screen opens.
///
/// Invariants (context design §2.2):
/// - INV-1: `running => status_source == Real && pid.is_some() && bind.is_some()
///   && detected_at.is_some()`.
/// - INV-2: `status_source == Unknown => !running`.
/// - INV-3: `path` is a file name only, never a full path.
/// - INV-5: absence of a detector result never produces `running == true`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProjectContext {
    pub(crate) exists: bool,
    pub(crate) path: Option<String>,
    pub(crate) template: Option<String>,
    pub(crate) running: bool,
    pub(crate) pid: Option<u32>,
    pub(crate) bind: Option<String>,
    pub(crate) status_source: StatusSource,
    pub(crate) detected_at: Option<SystemTime>,
}

impl ProjectContext {
    /// Context for the compatibility shim and any caller that did not run
    /// state detection: markers are known, execution status is not.
    pub(crate) fn detected(exists: bool) -> Self {
        Self {
            exists,
            path: None,
            template: None,
            running: false,
            pid: None,
            bind: None,
            status_source: StatusSource::Unknown,
            detected_at: None,
        }
    }

    /// Builds the run-status fields from a detector result. `state = Some` is
    /// the only path that can ever set `running = true` (INV-1/INV-2/INV-5).
    pub(crate) fn from_detection(exists: bool, state: Option<DevState>) -> Self {
        let (running, pid, bind, status_source) = match state {
            Some(dev) => (true, Some(dev.pid), Some(dev.bind), StatusSource::Real),
            None => (false, None, None, StatusSource::Unknown),
        };
        Self {
            exists,
            path: base_name(),
            template: read_layout(),
            running,
            pid,
            bind,
            status_source,
            detected_at: Some(SystemTime::now()),
        }
    }
}

/// Resolves the context at the I/O boundary (called only by `run_home` on the
/// full-screen path). Outside a project the detector is not even invoked, so
/// headless and out-of-project paths never touch `.ag/dev.json`.
pub(crate) fn detect_context() -> ProjectContext {
    detect_with(super::in_project(), || {
        project_state::read_fresh(project_state::now_secs())
    })
}

/// Testable core. The detector is injected so tests never touch a real state
/// file; when `exists` is false it is deliberately never called.
pub(crate) fn detect_with(exists: bool, detect: impl Fn() -> Option<DevState>) -> ProjectContext {
    let state = if exists { detect() } else { None };
    ProjectContext::from_detection(exists, state)
}

/// Base name of the current directory only — never a full path (INV-3, S-3).
fn base_name() -> Option<String> {
    std::env::current_dir()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .filter(|name| !name.is_empty())
}

/// `[project].layout` from `ag.toml`; `None` when missing or unreadable.
/// Never inferred from the directory name or `Cargo.toml` (context design E10).
fn read_layout() -> Option<String> {
    let raw = std::fs::read_to_string("ag.toml").ok()?;
    layout_from_str(&raw)
}

/// Pure parser for the manifest layout, exercised by tests without disk I/O.
fn layout_from_str(raw: &str) -> Option<String> {
    let doc: toml::Value = toml::from_str(raw).ok()?;
    doc.get("project")?
        .get("layout")?
        .as_str()
        .map(str::to_owned)
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dev(pid: u32, bind: &str) -> DevState {
        DevState {
            schema: project_state::SCHEMA,
            pid,
            bind: bind.to_owned(),
            started_at: 100,
            heartbeat_at: 105,
        }
    }

    #[test]
    fn context_unknown_when_state_missing() {
        let ctx = detect_with(true, || None);
        assert!(!ctx.running);
        assert_eq!(ctx.status_source, StatusSource::Unknown);
        assert_eq!(ctx.pid, None);
        assert_eq!(ctx.bind, None);
        assert!(ctx.detected_at.is_some());
    }

    #[test]
    fn context_real_when_detector_returns_fresh_state() {
        let ctx = detect_with(true, || Some(dev(4242, "127.0.0.1:8080")));
        // INV-1
        assert!(ctx.running);
        assert_eq!(ctx.status_source, StatusSource::Real);
        assert_eq!(ctx.pid, Some(4242));
        assert_eq!(ctx.bind.as_deref(), Some("127.0.0.1:8080"));
        assert!(ctx.detected_at.is_some());
    }

    #[test]
    fn context_unknown_when_outside_project_does_not_call_detector() {
        // The detector would panic if reached: out-of-project must not read state.
        let ctx = detect_with(false, || -> Option<DevState> {
            panic!("detector must not run outside a project")
        });
        assert!(!ctx.exists);
        assert!(!ctx.running);
        assert_eq!(ctx.status_source, StatusSource::Unknown);
    }

    #[test]
    fn detected_shim_is_always_unknown() {
        let ctx = ProjectContext::detected(true);
        assert!(ctx.exists);
        assert!(!ctx.running);
        assert_eq!(ctx.status_source, StatusSource::Unknown);
        assert_eq!(ctx.pid, None);
        assert_eq!(ctx.bind, None);
        // INV-2
        assert!(ctx.status_source == StatusSource::Unknown || ctx.running);
    }

    #[test]
    fn base_name_never_leaks_absolute_path() {
        let name = base_name().expect("cwd has a base name");
        assert!(!name.contains('/'));
        assert!(!name.contains('\\'));
    }

    #[test]
    fn template_comes_only_from_project_layout() {
        let manifest = "[project]\nname = \"demo-api\"\nlayout = \"layered\"\n\n[modules]\n";
        assert_eq!(layout_from_str(manifest).as_deref(), Some("layered"));
        // A manifest without [project].layout is unknown, never inferred.
        assert_eq!(layout_from_str("[modules]\ndata = true\n"), None);
        assert_eq!(layout_from_str("not valid toml"), None);
    }
}
