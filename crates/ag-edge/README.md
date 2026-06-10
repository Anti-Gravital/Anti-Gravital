# ag-edge

Anti-Gravital edge data plane.

`ag-edge` is the request-time layer below the `ag-domains` control plane
(ADR-0012 / RFC-0011). Given an incoming hostname it produces a routing
decision, selects a TLS certificate by SNI, and applies canonical host /
redirect policy.

## Status

Pure logic (always available), unit-tested:

- `router::resolve_hostname` — exact, then wildcard, then legacy-fallback
  precedence. Unknown custom hostnames fail closed and never resolve to another
  tenant.
- `tls::SniCertStore` / `tls::allow_on_demand` — handle-level SNI selection and
  restricted on-demand issuance (only for hostnames verified in the control
  plane).
- `redirect::CanonicalPolicy` — www<->apex and forced-canonical redirects,
  preserving path/query, with a configurable permanent status.
- `challenge::Http01ChallengeStore` — ACME HTTP-01 responder state with
  path-traversal-safe token extraction.

Runnable listeners (RFC-0011 phases B), implemented and integration-tested:

- `server` feature: `server::serve_http` (axum) serves the ACME HTTP-01
  challenge, applies redirects and routes by `Host`/`:authority`. Real TCP
  integration tests in `tests/http_edge.rs`.
- `tls` feature: `cert::CertStore` holds real `rustls` certified keys (exact +
  wildcard) with a PEM bridge (`insert_pem`) from the `ag-domains` ACME issuance
  output; `cert::SniCertResolver` selects by SNI (RFC 6066); `server::serve_https`
  serves HTTPS. Real TLS-handshake integration test in `tests/tls_edge.rs`.

## Scope

`ag-edge` does not own attachment state, ownership proof, or certificate
issuance: those belong to `ag-domains`. It is a data-plane consumer of bindings
and certificate references the control plane produces.
