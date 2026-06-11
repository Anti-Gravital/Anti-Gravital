# Explanation — HTTP-01 vs DNS-01

ACME offers more than one way to prove you control a domain. `ag-domains` uses
two: HTTP-01 and DNS-01. Understanding the difference explains why wildcards are
treated specially.

## HTTP-01

The CA gives you a token; you serve it at
`http://<host>/.well-known/acme-challenge/<token>`. The CA fetches that URL over
HTTP. If it gets the expected content, control of *that exact hostname* is
proven.

- Pros: nothing to change in DNS; the `ag-edge` HTTP-01 responder serves it
  automatically.
- Limits: proves only the exact hostname requested. It **cannot** prove a
  wildcard, because there is no single URL that represents `*.example.com`.

Used for exact apex and subdomain hostnames (`tls_mode = managed_http01`).

## DNS-01

The CA gives you a key authorization; you publish it as a TXT record at
`_acme-challenge.<domain>`. The CA queries DNS for that record.

- Pros: proves control of the whole domain, including wildcards. Works even when
  no HTTP server is reachable yet.
- Cost: requires writing a DNS record. You can do it manually, or let a provider
  adapter publish and clean it up automatically
  (`reference/provider-adapter-sdk.md`).

Used for wildcards (`tls_mode = managed_dns01`), where it is mandatory, and
available for any hostname.

## Why wildcards force DNS-01

A wildcard certificate covers every single-label subdomain. The CA will only
issue it when control of the parent domain is proven, and the only ACME challenge
that proves the parent (not one specific host) is DNS-01. That is why attaching
`*.example.com` sets `managed_dns01`.

See `reference/tls-lifecycle.md` for how each challenge fits into issuance and
renewal.
