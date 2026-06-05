# ag-edge

Anti-Gravital edge data plane.

`ag-edge` is the request-time layer below the `ag-domains` control plane
(ADR-0010 / RFC-0009). Given an incoming hostname it produces a routing
decision, selects a TLS certificate by SNI, and applies canonical host /
redirect policy.

## Status

Phase A (RFC-0009) — implemented as a pure, unit-tested library:

- `router::resolve_hostname` — exact, then wildcard, then legacy-fallback
  precedence. Unknown custom hostnames fail closed and never resolve to another
  tenant.
- `tls::SniCertStore` — SNI certificate selection (exact + single-label
  wildcard) over opaque certificate handles (never key material).
- `tls::allow_on_demand` — restricted on-demand issuance, only for hostnames
  already verified in the control plane.
- `redirect::CanonicalPolicy` — www<->apex and forced-canonical redirects,
  preserving path/query, with a configurable permanent status.

Binding live listeners on ports 80/443 and wiring the ACME HTTP-01 responder is
a later phase (RFC-0009 phase B). Keeping the logic as pure functions lets it be
tested in isolation first and reused by whatever listener wires it.

## Scope

`ag-edge` does not own attachment state, ownership proof, or certificate
issuance: those belong to `ag-domains`. It is a data-plane consumer of bindings
and certificate references the control plane produces.
