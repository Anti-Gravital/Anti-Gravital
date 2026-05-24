//! Gestion declarativa de dominios y TLS para el ecosistema Anti-Gravital.
//!
//! `ag-domains` orquesta:
//! - DNS via trait `DnsProvider` con adapter Cloudflare inicial.
//! - TLS via cliente ACME contra Let's Encrypt.
//! - Cooperacion con `ag-mail` para SPF/DKIM/DMARC.
//! - Verificacion de propagacion contra multiples resolvers publicos.
//!
//! # Estado
//!
//! Skeleton de la Fase 4.5 (Etapa 2-1). El trait y los tipos se completan
//! en la Etapa 2-2. El adapter Cloudflare en la 2-3. ACME y propagacion
//! en la 2-4. La decision gobernante es `ADR-0007` y el plan tecnico
//! vive en `RFC-0007`.
//!
//! # Alcance
//!
//! No es un registrador de dominios. No reemplaza Terraform/Pulumi. Solo
//! orquesta dominios declarados en `schema.ag`. Vease
//! `docs/architecture/08-modulos-batteries-included.md` seccion 8.9 y la
//! seccion 10.6 (integracion con `ag-cloud`).

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
