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
- `cloudflare`: adapter Cloudflare (DnsProvider). Otros adapters se añaden
  en iteraciones futuras detras de su propia feature.

## Tech Debt

- `notAfter` parsing for date-based certificate renewal (currently renews every
  cycle). Tracked in `docs/DEBT.md`.
- Additional DNS provider adapters (Namecheap, Route 53, Google Domains) — optional,
  not required. Tracked in `docs/DEBT.md`.
