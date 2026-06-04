# ag-mail

Correo transaccional outbound para Anti-Gravital.

Status: **Phase 4.5 — implemented**, plus the opt-in native MTA core of
**Phase 4.6-A**. Native SMTP sender, Resend/SES/Postmark adapters, in-memory
and persistent retry queues, custom SMTP headers, string templating and
`ag-observe` metrics are functional. The `mta` feature (off by default) adds a
native outbound MTA: MX resolution, ESMTP+STARTTLS direct delivery, Ed25519
DKIM signing and a bounce classifier. Remaining tech debt (external template
engines, RSA DKIM, MTA metrics) is tracked in `docs/DEBT.md`. Decisions:
`docs/adr/0007-ag-mail-ag-domains.md`, `docs/adr/0010-ag-mail-native-mta-pivot.md`.
Technical plans: `docs/rfc/RFC-0006-ag-mail-alcance.md`,
`docs/rfc/RFC-0009-ag-mail-native-mta.md`. Module sheet:
`docs/modules/ag-mail/README.md`.

## Alcance v1

- Sender SMTP nativo (lettre + rustls) y adapters Resend/SES/Postmark.
- HTML/plaintext templates via the built-in `StringTemplate` engine. Compile-time
  validation against `schema.ag` and external engines (askama/minijinja) are tracked
  as tech debt in `docs/DEBT.md`.
- Cola asincrona con reintentos y backoff exponencial.
- Metricas hacia ag-observe.

## Native MTA (feature `mta`, opt-in)

Additive native outbound MTA core (Phase 4.6-A, `ADR-0010` / `RFC-0009`):

- `MtaSender` — direct delivery to the destination MX over ESMTP+STARTTLS.
- MX resolution with preference ordering, `site_name` rollup and implicit-MX
  fallback (`hickory-resolver`).
- Ed25519 DKIM signing applied last (`mail-auth`); key material supplied by
  the caller / `ag-domains`.
- Pure SMTP bounce classifier (transient vs permanent, RFC 3463).

Off by default; the default sender and adapters are unchanged. Durable queues,
traffic shaping, REST API, webhooks and DSN/FBL processing are later phases.

## Fuera de alcance

- Servidor de correo completo / buzones / IMAP / POP / JMAP.
- Recepcion de correo general (inbound mas alla de DSN/FBL para bounces).
- Antispam como servicio.

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
- `queue-persistent`: PostgreSQL-backed persistent queue via ag-data (`PersistentQueue`,
  migration in `migrations/0001_mail_queue.sql`).
- `mta` (opt-in): native outbound MTA core (MX resolution, ESMTP+STARTTLS
  delivery, Ed25519 DKIM signing, bounce classification). Pulls `mail-send`,
  `mail-auth` and `hickory-resolver`; not in the default build.
