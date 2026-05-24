# ag-mail

Correo transaccional outbound para Anti-Gravital.

Estado: **Fase 4.5 — skeleton (Etapa 2-1)**. No implementado todavia.
Decision: `docs/adr/0007-ag-mail-ag-domains.md`. Plan tecnico:
`docs/rfc/RFC-0006-ag-mail-alcance.md`. Ficha de modulo:
`docs/modules/ag-mail/README.md`.

## Alcance v1

- Sender SMTP nativo (lettre + rustls) y adapters Resend/SES/Postmark.
- Templates HTML/plaintext con askama validados en compile-time contra
  `schema.ag`.
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
- `templates` (default): renderizado askama.
- `metrics` (default): metricas hacia ag-observe.
- `resend`, `ses`, `postmark`: adapters de proveedor (opcionales).
- `queue-persistent`: backend persistente de cola via ag-data.
