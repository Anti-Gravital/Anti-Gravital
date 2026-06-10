//! Declarative domain and TLS management for the Anti-Gravital ecosystem.
//!
//! `ag-domains` orchestrates:
//! - DNS via the `DnsProvider` trait with an initial Cloudflare adapter.
//! - TLS via an ACME client against Let's Encrypt.
//! - Cooperation with `ag-mail` for SPF/DKIM/DMARC.
//! - Propagation verification against multiple public resolvers.
//!
//! # Status
//!
//! Phase 4.5 library — implemented: `DnsProvider` trait with a Cloudflare
//! adapter, the declarative A/AAAA/CNAME/TXT/MX record model, SPF/DKIM/DMARC
//! generation for `ag-mail`, an ACME client (Let's Encrypt) for
//! issuance/renewal, and DNS propagation checks against public resolvers.
//!
//! Control plane (ADR-0012 / RFC-0011, phase A) — implemented: hostname
//! normalization/classification ([`hostname`]), the attachment state machine
//! ([`attachment`]), native persistence ([`store`]), TXT ownership proof
//! ([`ownership`]), the DNS instruction engine with BIND export
//! ([`instructions`]), CAA preflight ([`caa`]) and diagnostics
//! ([`diagnostics`]). Later phases (live edge listeners, REST API, SQL store,
//! provider automation) are tracked in `RFC-0011` and `docs/DEBT.md`.
//!
//! # Scope
//!
//! It is not a domain registrar. It does not replace Terraform/Pulumi, and it
//! is not an arbitrary multi-tenant DNS hosting panel. It manages domains
//! declared in `schema.ag` and domains attached to a project. See
//! `docs/architecture/08-modulos-batteries-included.md` section 8.9 and
//! section 10.6 (integration with `ag-cloud`).

#![forbid(unsafe_code)]

pub mod error;
pub mod provider;
pub mod record;

#[cfg(feature = "acme")]
pub mod acme;

#[cfg(feature = "propagation")]
pub mod propagation;

pub mod mail_records;
pub mod metrics;

// Control-plane layer (ADR-0012 / RFC-0011). Additive: the declarative DNS+TLS
// library above is unchanged. These modules add domain attachment lifecycle,
// ownership proof, DNS instruction generation, CAA preflight, diagnostics and
// native persistence.
pub mod attachment;
pub mod caa;
pub mod diagnostics;
pub mod events;
pub mod hostname;
pub mod instructions;
pub mod issuance;
pub mod ownership;
pub mod store;

#[cfg(feature = "api")]
pub mod api;

#[cfg(feature = "sql-store")]
pub mod sql_store;

pub use attachment::DomainAttachment;
pub use error::AgDomainsError;
pub use hostname::{Hostname, HostnameKind};
pub use provider::sdk::{
    diff as zone_diff, ChangeRef, ProviderAdapter, VerifyOutcome, ZoneAdapter, ZoneChange,
    ZonePlan, ZoneRef,
};

#[cfg(feature = "acme")]
pub use acme::wildcard::{issue_dns01_with_adapter, Dns01Desired, Dns01OrderDriver};
