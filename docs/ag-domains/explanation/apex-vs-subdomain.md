# Explanation — apex vs subdomain (and wildcards)

## Apex (root / naked) domain

`example.com` with no label in front. DNS forbids a plain `CNAME` at the apex
because the apex also carries SOA/NS records. So the apex points at the edge
with `A` (IPv4) and `AAAA` (IPv6) records, or via a provider feature
(ALIAS/ANAME/CNAME flattening) that synthesizes them.

## Subdomain

`api.example.com`, `www.example.com`, etc. A subdomain can use a `CNAME` to the
edge host, which lets the edge IPs change without the operator updating records.

## Wildcard

`*.example.com` matches one label below the base (`a.example.com`, not
`a.b.example.com`), consistent with TLS wildcard certificate semantics. Wildcard
TLS requires a DNS-01 ACME challenge, so wildcards carry stricter validation.

## Why this matters

The instruction engine and the edge router both classify a hostname into one of
these three kinds. The classification decides which DNS records are generated
and how a request hostname is matched at the edge (exact before wildcard, so a
specific `api.example.com` binding always wins over `*.example.com`).

## Registrable domain

Ownership is proven once at the registrable domain (`example.com`) via a TXT
record under `_ag-domain`, so a single proof covers the apex and its
subdomains. Deriving the registrable domain correctly for multi-label public
suffixes (`co.uk`) needs the Public Suffix List; the current heuristic uses the
last two labels and is tracked as tech debt (DEBT-024).
