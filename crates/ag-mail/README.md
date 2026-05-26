# ag-mail

Correo transaccional outbound para Anti-Gravital.

Status: **Phase 4.5 — implemented.** Native SMTP sender, Resend/SES/Postmark
adapters, in-memory retry queue, string templating and `ag-observe` metrics are
functional. Pending tech debt (persistent queue, custom SMTP headers, external
template engines) is tracked in `docs/DEBT.md`.
Decision: `docs/adr/0007-ag-mail-ag-domains.md`. Technical plan:
`docs/rfc/RFC-0006-ag-mail-alcance.md`. Module sheet: `docs/modules/ag-mail/README.md`.

## Alcance v1

- Sender SMTP nativo (lettre + rustls) y adapters Resend/SES/Postmark.
- HTML/plaintext templates via the built-in `StringTemplate` engine. Compile-time
  validation against `schema.ag` and external engines (askama/minijinja) are tracked
  as tech debt in `docs/DEBT.md`.
- Cola asincrona con reintentos y backoff exponencial.
- Metricas hacia ag-observe.

## Fuera de alcance (v1)

- Servidor MTA completo.
- Recepcion de correo (IMAP/POP).
- Antispam, reputacion de IP, gestion de bounces avanzada.

## Direccion de dependencia

`ag-auth` consume `ag-mail` para verificacion de correo, recuperacion de
contrasena y magic links. `ag-mail` NO depende de `ag-auth`. Esta regla
esta documentada como sexta regla de dependencias en
`docs/architecture/05-ecosistema-modulos.md` y proviene de `ADR-0007`.

## Features de Cargo

- `smtp` (default): sender SMTP nativo.
- `templates` (default): built-in StringTemplate rendering.
- `metrics` (default): metricas hacia ag-observe.
- `resend`, `ses`, `postmark`: adapters de proveedor (opcionales).
- `queue-persistent`: backend persistente de cola via ag-data.
