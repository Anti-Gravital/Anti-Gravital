//! Outbound transactional email for the Anti-Gravital ecosystem.
//!
//! `ag-mail` covers sending transactional email (account verification,
//! magic links, password recovery, alerts) with a native SMTP relay and an
//! opt-in native outbound MTA. To send through an external email provider,
//! point the native SMTP relay at that provider's SMTP endpoint — there are
//! no provider-brand adapters (see `ADR-0011`).
//!
//! # Status
//!
//! Phase 4.5 — implemented. Public API: [`sender::MailSender`] trait with the
//! native `SmtpSender` (default relay), the [`message::Email`]/`EmailBuilder`
//! model, an in-memory retry queue and `ag-observe` metrics. Outstanding tech
//! debt (persistent queue worker, custom SMTP headers, external template
//! engines) is tracked in `docs/DEBT.md`. Governing decisions: `ADR-0007`,
//! `ADR-0011`; technical plans: `RFC-0006`, `RFC-0010`.
//!
//! # Native MTA (opt-in, `mta` feature)
//!
//! The `mta` feature adds an opt-in native outbound MTA (`sender::mta`):
//! `MtaSender` resolves the destination MX, optionally picks an egress source
//! from a weighted pool (`sender::mta::egress`), opens an ESMTP session with
//! opportunistic STARTTLS, signs with DKIM (Ed25519 or RSA-SHA256,
//! `sender::mta::dkim`), and delivers directly, classifying SMTP replies. A
//! two-tier delivery queue (`sender::mta::queue`) adds retry/backoff,
//! per-`site_name` traffic shaping (`sender::mta::shaping`) and an automatic
//! suppression list (`sender::mta::suppress`) fed by synchronous failures and
//! by asynchronous DSN/ARF intake (`sender::mta::dsn`), with `ag-observe`
//! metrics throughout. The separate `api` feature adds HMAC-SHA256 signed
//! webhooks (`api::webhook`). All of this is additive: the default sender is
//! unchanged and these features are off by default. Governing decision:
//! `ADR-0010`; technical plan: `RFC-0009`. A durable queue spool and the REST
//! API routes are later phases (`docs/DEBT.md`).
//!
//! # Scope
//!
//! Outbound only. Even with the `mta` feature it does **not** offer IMAP/POP
//! and is **not** a complete mail server (inbound is limited to DSN/ARF parsing
//! for bounce and complaint processing). See
//! `docs/architecture/08-modulos-batteries-included.md` section 8.8 and the
//! ADRs `docs/adr/0007-ag-mail-ag-domains.md` and
//! `docs/adr/0010-ag-mail-native-mta-pivot.md`.
//!
//! # Dependency direction
//!
//! `ag-auth` consumes `ag-mail` through a small trait that `ag-auth`
//! defines. `ag-mail` does **NOT** depend on `ag-auth`. This rule (the sixth
//! of chapter 5 of the Technical Architecture) preserves composability and
//! avoids cycles verifiable in CI.

#![forbid(unsafe_code)]

#[cfg(feature = "api")]
pub mod api;
pub mod error;
pub mod message;
pub mod metrics;
pub mod queue;
pub mod sender;
pub mod template;

pub use error::AgMailError;
