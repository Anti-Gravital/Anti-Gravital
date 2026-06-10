# How-to — configure a wildcard domain

A wildcard (`*.example.com`) serves any single label below the base
(`a.example.com`, not `a.b.example.com`), matching TLS wildcard semantics.

## Attach

```
ag domains attach '*.example.com' --project <p> --edge-host <edge>
```

Quote the `*` so the shell does not expand it. The attachment is created with
`tls_mode = managed_dns01` automatically, because wildcard certificates can only
be issued via the ACME **DNS-01** challenge (HTTP-01 cannot prove a wildcard).

## DNS records

`ag domains instructions '*.example.com' --edge-host <edge>` prints:

- `CNAME * -> <edge-host>` (the wildcard route),
- `TXT _ag-domain.example.com -> ag-verification=...` (ownership).

During certificate issuance a `_acme-challenge.example.com` TXT record is also
required (DNS-01). With the manual flow you publish it when prompted; with a
`read_apply` provider adapter it is created and cleaned up automatically.

## Why stricter validation

A wildcard certificate covers every subdomain, so issuance is gated harder:
ownership must be verified at the registrable domain, and exact-hostname
attachments always win over the wildcard at the edge
(`reference/state-machine.md`, `explanation/apex-vs-subdomain.md`).

## Verify

```
ag domains verify '*.example.com'
ag domains status '*.example.com'
```
