//! Cliente ACME para emision y renovacion de certificados TLS.
//!
//! Basado en `instant-acme` contra Let's Encrypt. Soporta challenge
//! DNS-01 (preferido, usa el `DnsProvider` para crear el TXT).
//!
//! Flujo: crear cuenta -> crear orden -> resolver DNS-01 -> esperar
//! validacion -> generar CSR con rcgen -> finalizar -> descargar PEM.

pub mod challenge;
pub mod renewal;
