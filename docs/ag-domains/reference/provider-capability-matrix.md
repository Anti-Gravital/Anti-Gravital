# Reference — provider capability matrix

Backed by `ag_domains::provider::capabilities` and served at
`GET /v1/domains/provider-capabilities`. Two axes are tracked separately:

- **Adapter** — what Anti-Gravital automates today: `manual` (paste records) or
  `read_apply` (a built-in adapter reads and applies records).
- **Provider features** — facts the DNS instruction engine uses.

A provider not listed here is handled by the manual flow. This is a conservative,
living list (RFC-0011 phase E flips the adapter column as adapters land); it never
claims automation that does not exist.

| Provider | Adapter | Apex alias/ANAME | CNAME flattening | DNS-01 auto | BIND export | Domain purchase |
|---|---|---|---|---|---|---|
| manual | manual | no | no | no | yes | no |
| cloudflare | read_apply | yes | yes | yes | yes | no |
| route53 | read_apply | yes | no | yes | yes | no |
| google-cloud-dns | read_apply | no | no | yes | yes | no |
| azure-dns | read_apply | yes | no | yes | yes | no |
| namecheap | manual | no | no | no | yes | no |
| hostinger | manual | no | no | no | yes | no |
| squarespace | manual | no | no | no | yes | no |

Notes:

- `Adapter = read_apply` is the only column that means Anti-Gravital changes your
  DNS for you. Everything else is the manual flow: `ag domains instructions` /
  `export-zone` produce records you paste.
- `Domain purchase` is `no` for every provider in v1: Anti-Gravital is not a
  registrar (a future `ag-registrars`).
- `DNS-01 auto` requires a `read_apply` adapter; with the manual flow you publish
  the `_acme-challenge` TXT record yourself.
- Apex/flattening facts inform whether the apex can target the edge host via a
  provider feature instead of fixed A/AAAA records.
