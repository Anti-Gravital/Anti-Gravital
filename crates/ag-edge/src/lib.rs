//! Anti-Gravital edge data plane.
//!
//! `ag-edge` holds the request-time logic that turns an incoming hostname into
//! a routing decision, selects a TLS certificate by SNI, and applies canonical
//! host / redirect policy. It is the data plane sitting below the `ag-domains`
//! control plane (ADR-0010 / RFC-0009 architecture).
//!
//! # Status
//!
//! Phase A (RFC-0009) — implemented as a pure library: [`router::resolve_hostname`]
//! with exact/wildcard/legacy-fallback precedence, [`tls`] SNI certificate
//! selection with restricted on-demand issuance, and [`redirect`] canonical
//! host policy. Binding live listeners on ports 80/443 and wiring the HTTP-01
//! responder is a later phase (RFC-0009 phase B); keeping the logic as pure
//! functions lets it be unit-tested in isolation first.
//!
//! # Backward compatibility
//!
//! Unknown custom hostnames fail closed ([`router::RouteResolution::Unknown`]):
//! they never resolve to another tenant. Only pre-existing legacy default
//! routes fall open via the caller-supplied legacy resolver.

#![forbid(unsafe_code)]

pub mod redirect;
pub mod router;
pub mod tls;

pub use router::{resolve_hostname, RouteBinding, RouteResolution};
