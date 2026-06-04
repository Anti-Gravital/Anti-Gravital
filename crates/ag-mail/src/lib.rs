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
//! Phase 4.6-A adds an opt-in native outbound MTA behind the `mta` feature:
//! [`sender::mta::MtaSender`] resolves the destination MX, opens an ESMTP
//! session with opportunistic STARTTLS, signs with DKIM (Ed25519), and
//! delivers directly to the recipient server, plus a pure bounce classifier
//! ([`sender::mta::bounce`]). This is additive: the default sender and the
//! provider adapters are unchanged, and the feature is off by default.
//! Governing decision: `ADR-0010`; technical plan: `RFC-0009`. Durable queues,
//! traffic shaping, the REST API, and DSN/FBL processing are later phases.
//!
//! # Scope
//!
//! Outbound only. Even with the `mta` feature it does **not** receive email
//! (beyond future DSN/FBL parsing for bounces), does **not** offer IMAP/POP,
//! and is **not** a complete mail server. See
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

pub mod error;
pub mod message;
pub mod metrics;
pub mod queue;
pub mod sender;
pub mod template;

pub use error::AgMailError;
