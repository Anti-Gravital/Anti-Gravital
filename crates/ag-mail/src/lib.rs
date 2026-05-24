//! Correo transaccional outbound para el ecosistema Anti-Gravital.
//!
//! `ag-mail` cubre el envio de correo transaccional (verificacion de
//! cuentas, magic links, recuperacion de contrasena, alertas) con un
//! sender SMTP nativo y adapters de primera clase para Resend, SES y
//! Postmark.
//!
//! # Estado
//!
//! Skeleton de la Fase 4.5 (Etapa 2-1). Las APIs publicas estan declaradas
//! como modulos vacios; la implementacion arranca en las Etapas 2-5 y
//! posteriores. La decision gobernante es `ADR-0007` y el plan tecnico
//! vive en `RFC-0006`.
//!
//! # Alcance
//!
//! Solo outbound. **No** es un MTA, **no** recibe correo, **no** ofrece
//! IMAP/POP ni antispam. Vease `docs/architecture/08-modulos-batteries-included.md`
//! seccion 8.8 y el ADR `docs/adr/0007-ag-mail-ag-domains.md`.
//!
//! # Direccion de dependencia
//!
//! `ag-auth` consume `ag-mail` mediante un trait pequeno que `ag-auth`
//! define. `ag-mail` **NO** depende de `ag-auth`. Esta regla (sexta del
//! capitulo 5 de la Arquitectura Tecnica) preserva la composabilidad y
//! evita ciclos verificables en CI.

#![forbid(unsafe_code)]

pub mod error;
pub mod message;
pub mod sender;
pub mod template;
pub mod queue;
pub mod metrics;

pub use error::AgMailError;
