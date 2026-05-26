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
//! Phase 4.5 skeleton (Stage 2-1). The trait and types are completed
//! in Stage 2-2. The Cloudflare adapter in 2-3. ACME and propagation
//! in 2-4. The governing decision is `ADR-0007` and the technical plan
//! lives in `RFC-0007`.
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
