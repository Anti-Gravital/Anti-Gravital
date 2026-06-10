//! ACME client for TLS certificate issuance and renewal.
//!
//! Based on `instant-acme` against Let's Encrypt. Supports the DNS-01
//! challenge (preferred, uses the `DnsProvider` to create the TXT record).
//!
//! Flow: create account -> create order -> resolve DNS-01 -> wait for
//! validation -> generate CSR with rcgen -> finalize -> download PEM.

pub mod challenge;
pub mod renewal;
pub mod wildcard;
