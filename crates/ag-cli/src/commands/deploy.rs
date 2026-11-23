//! Local deployment artifact rendering (`ag deploy`).
//!
//! Capability: copy already-authored Docker and Kubernetes artifacts from a
//! project into `deploy/generated/`. This module deliberately has no network,
//! container-runtime or cluster dependency: it never starts a subprocess and
//! never opens a socket (RFC-0034). Sources are discovered without following
//! symlinks; writes go through a `cap_std::fs::Dir` rooted at
//! `deploy/generated/`, so neither artifacts, state nor audit lines can be
//! redirected outside the project by a hostile symlink (RFC-0013).
//!
//! The capability is split into focused submodules; this root re-exports the
//! original `pub(crate)` API so callers keep using `commands::deploy::*`.

mod apply;
mod model;
mod plan;
mod status;
#[cfg(test)]
mod tests;

// These re-exports preserve the original crate-visible API of this module.
// Not every item is referenced by current callers, hence the lint allowance.
#[allow(unused_imports)]
pub(crate) use apply::cmd_deploy_apply;
#[allow(unused_imports)]
pub(crate) use model::{project_label, DeployError, DeployPlan, DeployTarget};
#[allow(unused_imports)]
pub(crate) use plan::{build_plan, cmd_deploy_plan, parse_plan_id, plan_report};
#[allow(unused_imports)]
pub(crate) use status::{cmd_deploy_status, status_report};
