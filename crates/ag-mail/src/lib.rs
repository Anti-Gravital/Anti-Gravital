//! Outbound transactional email for the Anti-Gravital ecosystem.
//!
//! `ag-mail` covers sending transactional email (account verification,
//! magic links, password recovery, alerts) with a native SMTP sender and
//! first-class adapters for Resend, SES and Postmark.
//!
//! # Status
//!
//! Phase 4.5 skeleton (Stage 2-1). The public APIs are declared as empty
//! modules; the implementation starts in Stages 2-5 and later. The
//! governing decision is `ADR-0007` and the technical plan lives in
//! `RFC-0006`.
//!
//! # Scope
//!
//! Outbound only. It is **not** an MTA, it does **not** receive email, it
//! does **not** offer IMAP/POP or antispam. See
//! `docs/architecture/08-modulos-batteries-included.md` section 8.8 and the
//! ADR `docs/adr/0007-ag-mail-ag-domains.md`.
//!
//! # Dependency direction
//!
//! `ag-auth` consumes `ag-mail` through a small trait that `ag-auth`
//! defines. `ag-mail` does **NOT** depend on `ag-auth`. This rule (the sixth
//! of chapter 5 of the Technical Architecture) preserves composability and
//! avoids cycles verifiable in CI.

#![forbid(unsafe_code)]

pub mod error;
pub mod message;
pub mod metrics;
pub mod queue;
pub mod sender;
pub mod template;

pub use error::AgMailError;
