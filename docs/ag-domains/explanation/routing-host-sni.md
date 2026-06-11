# Explanation — routing by Host and SNI

The edge has to answer one question for every request: which tenant does this
hostname belong to? It uses two signals — the HTTP `Host` (or HTTP/2
`:authority`) header and the TLS SNI — and a deliberately conservative
resolution order.

## Two layers, one identity

- **SNI** is sent during the TLS handshake, before any HTTP bytes. The edge uses
  it to pick the right certificate (`ag_edge::tls::SniCertStore`).
- **Host / :authority** is the HTTP-layer hostname. The edge uses it to resolve
  the route (`ag_edge::resolve_hostname`).

Both must agree on the same attached hostname for a request to be served as that
tenant.

## Resolution precedence

`resolve_hostname` tries, in order (RFC-0011 §14.2):

1. an exact custom hostname binding,
2. a wildcard custom hostname binding,
3. the caller-supplied legacy default route,
4. the existing global fallback.

Exact wins over wildcard, so `api.example.com` and `*.example.com` can coexist
and `api` goes to its exact target. A single-label rule applies to wildcards:
`*.example.com` matches `a.example.com` but not `a.b.example.com`, matching TLS
wildcard semantics.

## Fail closed

If a hostname matches no binding and the legacy resolver returns nothing, the
result is the global fallback — never another tenant's binding. A malformed host
is `Unknown`. This is the security property that prevents one tenant's request
from being served as another's (`reference/security-model.md`).

## Observability

Every resolution records its latency and whether it was a cache hit (a custom
binding matched) or a miss (`reference/events-and-metrics.md`), so the route
cache hit-ratio is visible in dashboards.
