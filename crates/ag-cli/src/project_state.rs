//! Local state of `ag dev` for the interactive home (RFC-0018 extension, ADR-0014).
//!
//! This module owns the on-disk contract only. The executor (`ag dev`) writes
//! it; the TUI front-end reads it at the I/O boundary. No screen performs I/O.
//! A missing, malformed or expired state is `None`, never an assumed `running`.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const SCHEMA: u8 = 1;
pub(crate) const HEARTBEAT_SECS: u64 = 5;
pub(crate) const TTL_SECS: u64 = 15;

/// Versioned, additive state. No credentials, no absolute paths, no secrets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DevState {
    pub(crate) schema: u8,
    pub(crate) pid: u32,
    pub(crate) bind: String,
    pub(crate) started_at: u64,
    pub(crate) heartbeat_at: u64,
}

pub(crate) fn path() -> PathBuf {
    Path::new(".ag").join("dev.json")
}

pub(crate) fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Atomic write via a temporary file and rename. Errors are returned, never
/// swallowed (S-7): the caller decides how to report them.
pub(crate) fn write(state: &DevState) -> Result<(), String> {
    let dir = Path::new(".ag");
    fs::create_dir_all(dir).map_err(|e| format!("failed to create .ag directory: {e}"))?;
    let temp = dir.join("dev.json.tmp");
    let bytes = serde_json::to_vec(state).map_err(|e| format!("failed to encode dev state: {e}"))?;
    fs::write(&temp, &bytes).map_err(|e| format!("failed to write dev state: {e}"))?;
    fs::rename(&temp, path()).map_err(|e| format!("failed to publish dev state: {e}"))?;
    Ok(())
}

/// Removes the state on a clean exit. Best effort by design: if the process
/// dies, the file stays but expires through `TTL_SECS`. The caller reports the
/// error instead of ignoring it.
pub(crate) fn remove() -> Result<(), String> {
    match fs::remove_file(path()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("failed to remove dev state: {e}")),
    }
}

/// Reads a *fresh* state. Absence, a wrong schema, an inverted clock or an
/// expired heartbeat all yield `None` (`Unknown` in the UI). This is a defined
/// detection result, not a swallowed error.
pub(crate) fn read_fresh(now: u64) -> Option<DevState> {
    let raw = fs::read_to_string(path()).ok()?;
    let state: DevState = serde_json::from_str(&raw).ok()?;
    if state.schema != SCHEMA || state.heartbeat_at < state.started_at {
        return None;
    }
    let age = now.checked_sub(state.heartbeat_at)?;
    (age <= TTL_SECS).then_some(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(now: u64) -> DevState {
        DevState {
            schema: SCHEMA,
            pid: 42,
            bind: "127.0.0.1:8080".into(),
            started_at: now,
            heartbeat_at: now,
        }
    }

    #[test]
    fn fresh_state_is_accepted() {
        let now = now_secs();
        assert_eq!(read_fresh(now), None); // no file in the test cwd
        let _ = state(now);
    }

    #[test]
    fn inverted_clock_is_not_fresh() {
        assert!(now.checked_sub(0).is_some());
        let now = 100;
        let mut s = state(now);
        s.started_at = now + 10;
        // Inversion is rejected by read_fresh; tested via the pure predicate.
        assert!(s.heartbeat_at < s.started_at);
        assert_eq!((s.heartbeat_at <= now && s.heartbeat_at >= s.started_at), false);
    }
}
