//! Cliente ACME para emision y renovacion de certificados TLS.
//!
//! Basado en `instant-acme` contra Let's Encrypt. Soporta challenge
//! DNS-01 (preferido, usa el `DnsProvider` para crear el TXT) y HTTP-01
//! (alternativo).
//!
//! Skeleton de la Fase 4.5. Implementacion en la Etapa 2-4.

pub mod challenge;
pub mod renewal;
