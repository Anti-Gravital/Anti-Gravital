//! Trait `MailSender` y sus implementaciones.
//!
//! El trait define la interfaz que cumplen el sender SMTP nativo y los
//! adapters de proveedor (Resend, SES, Postmark). Cada adapter vive en un
//! submodulo activado por feature de Cargo.
//!
//! Skeleton de la Fase 4.5. El trait completo se implementa en la Etapa
//! 2-5 (RFC-0006 § 4.2).

#[cfg(feature = "smtp")]
pub mod smtp;

#[cfg(feature = "resend")]
pub mod resend;

#[cfg(feature = "ses")]
pub mod ses;

#[cfg(feature = "postmark")]
pub mod postmark;
