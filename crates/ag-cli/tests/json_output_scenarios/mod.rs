//! Scenario fixtures used only by the `json_output` integration test binary.
//!
//! These helpers were historically part of `json_output_support`, which both
//! JSON test binaries included. Because `json_output_config` never called them,
//! compiling that binary flagged each one as dead code and required a crate-wide
//! dead-code lint suppression. Keeping them in a module included only by
//! `json_output` removes the need for that suppression without weakening any
//! test: both binaries still compile with `-D warnings` and the full suite is
//! unchanged.

use crate::json_output_support::{scratch_dir, write};
use std::path::PathBuf;

// Relative to CARGO_MANIFEST_DIR (crates/ag-cli).
pub(crate) const SNAPSHOT_DIR: &str = "../../docs/mcp/snapshots";

pub(crate) fn clean_schema() -> &'static str {
    "model User {\n  id UUID @primary\n  name String\n}\n"
}

pub(crate) fn warning_schema() -> &'static str {
    "model Tag {\n  name String\n}\n"
}

pub(crate) fn invalid_schema() -> &'static str {
    "model { broken\n"
}

pub(crate) fn reference_schema() -> &'static str {
    "model Old {\n  id UUID @primary\n  name String\n}\n"
}

pub(crate) fn current_schema() -> &'static str {
    "model Old {\n  id UUID @primary\n  title String\n}\nmodel New {\n  id UUID @primary\n}\n"
}

pub(crate) fn dns_manifest() -> &'static str {
    "[project]\nname = \"demo\"\n\n[domain]\ndomain = \"api.example.com\"\nenvironment = \
     \"production\"\nvisibility = \"public\"\ntls = \"managed_http01\"\ntarget = { kind = \
     \"service\", ref = \"api\" }\n\n[domain.dns]\nCNAME = [{ name = \"api.example.com\", \
     value = \"edge.example-cloud.net\" }]\n"
}

pub(crate) fn workers_schema() -> &'static str {
    "worker SendWelcomeEmail {\n  queue \"mail\"\n  mode static\n  concurrency 4\n  retry {\n    max_attempts 5\n  }\n  input {\n    user_id UUID\n    email String @email\n  }\n}\n"
}

pub(crate) fn docker_project() -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT_DOCKER: AtomicUsize = AtomicUsize::new(0);
    let dir = scratch_dir(&format!(
        "deploy-json-{}",
        NEXT_DOCKER.fetch_add(1, Ordering::Relaxed)
    ));
    write(&dir, "Cargo.toml", "[package]\nname = \"demo\"\n");
    let docker = dir.join("deploy/docker");
    std::fs::create_dir_all(&docker).expect("docker source directory");
    std::fs::write(docker.join("Dockerfile"), "FROM scratch\n").expect("Dockerfile");
    std::fs::write(docker.join("compose.yml"), "services: {}\n").expect("compose file");
    dir
}
