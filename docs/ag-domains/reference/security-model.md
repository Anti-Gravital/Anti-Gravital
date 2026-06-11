# Reference — security model

The security properties `ag-domains` and `ag-edge` enforce. This describes
implemented behavior.

## Ownership proof

A domain is not routed until ownership is proven by a `_ag-domain` TXT record
(`ownership::ownership_record_name` + a per-attachment token), checked across
public resolvers. Ownership cannot be granted without a real DNS check: the
default verifier never confirms (`api::NullVerifier`). See
`explanation/why-txt-ownership.md`.

## Fail-closed routing

Edge route resolution never serves an unknown custom hostname another tenant's
content. An unmatched hostname falls through to the legacy resolver or the global
fallback, and a malformed host resolves to `Unknown`
(`ag_edge::resolve_hostname`). Exact bindings win over wildcards. See
`explanation/routing-host-sni.md`.

## Tombstones (anti-takeover)

Detaching a hostname removes the live attachment and writes a tombstone for a
default window, blocking immediate re-attachment by another party
(`store::Tombstone`). This closes the post-detach takeover window.

## Dangling-DNS detection

The `dangling` worker detects hostnames that still point at the edge but are no
longer attached (including detached ones in their tombstone window) and emits
`domain.dangling_dns_detected` so operators are notified. The edge keeps failing
closed for these hostnames (blueprint §15.3).

## Abuse controls

Optional per-tenant attachment and issuance limits and a global ACME issuance
queue bound fan-out and protect the CA rate budget
(`reference/abuse-controls.md`, blueprint §15.6). The REST API returns HTTP 429
at the per-tenant attachment limit.

## CAA preflight

Issuance is gated by a CAA check (RFC 8659) so a certificate is not ordered for
a domain that has not authorized Let's Encrypt (`caa::evaluate`).

## On-demand certificate issuance is restricted

The edge only allows on-demand certificate issuance for hostnames with a
verified attachment (`ag_edge::tls`), so an attacker cannot trigger issuance for
arbitrary hostnames by sending SNI.

## Native-first, minimal trust

No external service is required to use the control plane (ADR-0009): the default
store is in-memory/JSON and provider adapters are feature-gated. Fewer moving
parts means a smaller attack surface.
