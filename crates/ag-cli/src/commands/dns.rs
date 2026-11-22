//! `ag dns` commands: local, plan-first domain configuration (RFC-0033).
//!
//! The canonical input is the `\[domain\]` table of the project's `ag.toml`.
//! `plan` is read-only and deterministic; `apply` is gated by the exact plan
//! id and writes only a local resolved configuration. A provider adapter is
//! contacted only when `--provider` is passed explicitly. No credential, zone
//! id from a provider or token is ever read from or written to a file.
//!
//! The capability is split into focused submodules; this root re-exports the
//! original `pub(crate)` API so callers keep using `commands::dns::*`.

mod commands;
mod manifest;
mod model;
mod provider;
mod state;
#[cfg(test)]
mod tests;

// These re-exports preserve the original crate-visible API of this module.
// Not every item is referenced by current callers, hence the lint allowance.
#[allow(unused_imports)]
pub(crate) use commands::{
    cmd_dns_apply, cmd_dns_plan, cmd_dns_status, manifest_attachment_hints, print_plan,
    resolve_status, DnsStatus,
};
#[allow(unused_imports)]
pub(crate) use manifest::load_plan;
#[allow(unused_imports)]
pub(crate) use model::{
    DnsPlan, Environment, PlannedRecord, TlsMode, Visibility, DEFAULT_MANIFEST, DEFAULT_STATE,
};
#[allow(unused_imports)]
pub(crate) use provider::ProviderSummary;
