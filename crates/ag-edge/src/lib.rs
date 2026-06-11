//! Anti-Gravital edge data plane.
//!
//! `ag-edge` holds the request-time logic that turns an incoming hostname into
//! a routing decision, selects a TLS certificate by SNI, serves ACME HTTP-01
//! challenges, and applies canonical host / redirect policy. It is the data
//! plane sitting below the `ag-domains` control plane (ADR-0012 / RFC-0011).
//!
//! # Status
//!
//! - Pure logic (always available): [`router::resolve_hostname`] (exact /
//!   wildcard / legacy-fallback precedence, fail-closed for unknown hosts),
//!   [`redirect::CanonicalPolicy`], [`tls`] SNI handle selection and restricted
//!   on-demand issuance, and [`challenge`] HTTP-01 responder state.
//! - `server` feature: a runnable HTTP listener (`server::serve_http`) that
//!   serves challenges, applies redirects and routes by `Host`/`:authority`.
//! - `tls` feature: a real `rustls` certificate store with PEM bridge
//!   (`cert`) and an HTTPS listener (`server::serve_https`) that selects
//!   certificates by SNI.
//!
//! # Backward compatibility
//!
//! Unknown custom hostnames fail closed ([`router::RouteResolution::Unknown`]
//! and `ExistingFallback`): they never resolve to another tenant. Only
//! pre-existing legacy default routes fall open via the caller-supplied legacy
//! resolver.

#![forbid(unsafe_code)]

pub mod challenge;
pub mod metrics;
pub mod redirect;
pub mod router;
pub mod tls;

#[cfg(feature = "tls")]
pub mod cert;

#[cfg(feature = "server")]
pub mod server;

pub use router::{resolve_hostname, RouteBinding, RouteResolution};
