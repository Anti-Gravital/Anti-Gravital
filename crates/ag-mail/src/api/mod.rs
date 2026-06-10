//! Managed email API surface for the native MTA (Phase 4.6-C, `api` feature).
//!
//! This is the start of the multi-tenant REST/event surface (RFC-0009). The
//! first piece is signed-webhook support ([`webhook`]); the HTTP routes, data
//! model and marketing objects are tracked in `docs/DEBT.md` (DEBT-021).

pub mod webhook;
