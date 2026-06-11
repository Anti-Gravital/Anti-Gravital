# ag-domains

Gestion declarativa de dominios y TLS para proyectos Anti-Gravital.

Estado: **Fase 4.5 — implementado (Etapas 2-2 a 2-4)**.
Decision: `docs/adr/0007-ag-mail-ag-domains.md`. Plan tecnico:
`docs/rfc/RFC-0007-ag-domains-alcance.md`. Ficha de modulo:
`docs/modules/ag-domains/README.md`.

## Alcance

- Trait `DnsProvider` con adapter Cloudflare inicial.
- Modelo declarativo de registros A/AAAA/CNAME/TXT/MX.
- Cliente ACME para emision y renovacion de certificados (Let's Encrypt).
- Generacion de SPF/DKIM/DMARC para `ag-mail` (cooperacion).
- Verificacion de propagacion DNS contra resolvers publicos.

## Fuera de alcance

- **NO es un registrador de dominios.** El dominio se compra externamente.
- **NO reemplaza Terraform/Pulumi.** Solo orquesta dominios declarados en
  `schema.ag`.

## Direccion de dependencia

`ag-cloud` consume `ag-domains` durante `ag deploy` sin dependencia
rigida en todos los targets (septima regla de dependencias en
`docs/architecture/05-ecosistema-modulos.md`, proveniente de `ADR-0007`).

## Features de Cargo

- `acme` (default): cliente ACME contra Let's Encrypt.
- `propagation` (default): verificacion via resolvers DNS publicos.
- `psl` (RFC-0016): derivacion de eTLD+1 via Public Suffix List, correcta para
  sufijos multi-etiqueta (`co.uk`, `com.br`). OFF por defecto; el heuristico de
  dos etiquetas mantiene el build nativo offline (ADR-0009).
- `cloudflare`: adapter Cloudflare (DnsProvider). Otros adapters se añaden
  en iteraciones futuras detras de su propia feature.
- `route53`: adapter Amazon Route 53 (AWS SigV4 + REST/XML). El firmador SigV4 se
  verifica contra el vector `get-vanilla` publicado por AWS; la ruta real va con
  test `#[ignore]` de credencial.

## Control plane (ADR-0012 / RFC-0011, phase A)

In addition to the declarative library above, `ag-domains` provides a native
domain attachment and serving control plane:

- `hostname` — normalization, IDN/Punycode identity, apex/subdomain/wildcard
  classification and validation.
- `attachment` — the attachment state machine (ownership/DNS/TLS/routing
  readiness dimensions + derived lifecycle).
- `store` — `AttachmentStore` trait with native `InMemoryStore` and
  `JsonFileStore` (a SQL store is a later, feature-gated phase).
- `ownership` — TXT ownership token generation and verification.
- `instructions` — the DNS instruction engine (apex/subdomain/wildcard) and
  BIND zone-file export.
- `caa` — CAA preflight before ACME issuance.
- `diagnostics` — expected vs observed record comparison.

This is additive: the declarative DNS+TLS library is unchanged. It remains not a
registrar, not a Terraform replacement, and not an arbitrary multi-tenant DNS
hosting panel (RFC-0011 §3.2). Deferred phases (provider automation, registrar
module) are tracked as GitHub Issues (label `tech-debt`; historical DEBT-025).

The data-plane logic (hostname routing, SNI certificate selection,
canonical/redirect policy) and the runnable HTTP/HTTPS edge listeners live in
the `ag-edge` crate.

### `api` feature (RFC-0011 phase C)

Enables `ag_domains::api`, an `axum` router (`build_router` / `serve`) exposing
`/v1/domains/attachments` (create/list/get/instructions/status/detach) backed by
any `AttachmentStore`. The native store keeps it self-hostable; no database is
required. Covered by `tests/api_rest.rs` (real HTTP). See
`openapi/ag-domains.v1.yaml`.

### `sql-store` feature (RFC-0011 phase D)

Enables `ag_domains::sql_store::SqlAttachmentStore`, a Postgres-backed
`AttachmentStore` (sqlx) with an embedded schema/migration. Optional accelerator
for multi-node operation; the native in-memory/JSON store stays the default
(ADR-0009). Integration tests are `#[ignore]` and require `DATABASE_URL`.

## Tech Debt

- `notAfter` parsing for date-based certificate renewal (currently renews every
  cycle). Tracked as a GitHub Issue (label `tech-debt`).
- Additional DNS provider adapters — optional, not required. Tracked as a GitHub
  Issue (label `tech-debt`).
