# Reference — TLS certificate lifecycle

How `ag-domains` issues and renews TLS certificates via ACME (Let's Encrypt).
This describes implemented behavior in `ag_domains::acme`.

## Challenge selection

| Hostname kind | TLS mode | ACME challenge |
|---|---|---|
| Exact (apex or subdomain) | `managed_http01` | HTTP-01 |
| Wildcard (`*.example.com`) | `managed_dns01` | DNS-01 (required) |
| Any | `disabled` | no managed certificate |

HTTP-01 proves control of a single hostname by serving a token at
`/.well-known/acme-challenge/...` (the `ag-edge` responder). DNS-01 proves
control by publishing an `_acme-challenge` TXT record and is the only challenge
that can cover a wildcard. See `explanation/http01-vs-dns01.md`.

## Issuance

- Single hostname: `acme::renewal::issue` (or `issue_with_credentials` to reuse
  an ACME account) drives account creation, the order, the DNS-01 challenge via
  a `DnsProvider`, CSR generation (`rcgen`), and certificate download.
- Wildcard / multi-SAN automation: `issue_dns01_with_adapter` publishes the
  `_acme-challenge` records through the provider adapter SDK, finalizes the
  order, and tears the challenge records down afterwards
  (`reference/provider-adapter-sdk.md`). Manual DNS-01 (publish the TXT yourself)
  stays supported.
- Rate protection: issuance is deduplicated by SAN set and bounded per
  registered domain (`issuance::IssuanceLimiter`), plus the optional abuse
  controls (`reference/abuse-controls.md`).

## Renewal scheduling

`acme::renewal::spawn_renewal_task` runs a background task that renews before
expiry. The sleep until the next renewal is decided by:

1. **ARI (RFC 9773)** when available: the CA's suggested renewal window is
   honored (`acme::ari`), scheduling at the window start. ARI also lets the CA
   force early renewal during incidents. Use `spawn_renewal_task_with_ari` to
   feed an ARI fetch hook.
2. **`notAfter` fallback** otherwise: renew `renew_before_days` before expiry
   (`seconds_until_renewal`).

On renewal error the task retries after 24h. After each renewal the
`ag_domains_cert_days_until_expiry` gauge is updated
(`reference/events-and-metrics.md`).

## Certificate states

The attachment's `tls_status` tracks the lifecycle: `disabled`, `pending`,
`active`, `renewal_due`, `failed`, `expired`, `retired` (set on detach). See
`reference/state-machine.md`.

## CAA preflight

Before issuance, `caa::evaluate` checks the domain's CAA records (RFC 8659) to
confirm Let's Encrypt is authorized to issue, surfacing a clear error instead of
a failed order when it is not.
