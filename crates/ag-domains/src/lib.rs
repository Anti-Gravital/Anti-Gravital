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
//! Phase 4.5 — implemented. `DnsProvider` trait with a Cloudflare adapter, the
//! declarative A/AAAA/CNAME/TXT/MX record model, SPF/DKIM/DMARC generation for
//! `ag-mail`, an ACME client (Let's Encrypt) for issuance/renewal, and DNS
//! propagation checks against public resolvers. Outstanding tech debt
//! (`notAfter` parsing for scheduled renewal, extra DNS adapters) is tracked in
//! `docs/DEBT.md`. Governing decision: `ADR-0007`; technical plan: `RFC-0007`.
//!
//! # Scope
//!
//! It is not a domain registrar. It does not replace Terraform/Pulumi. It only
//! orchestrates domains declared in `schema.ag`. See
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

pub use error::AgDomainsError;
