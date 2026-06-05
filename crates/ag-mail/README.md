# ag-mail

Correo transaccional outbound para Anti-Gravital.

Status: **Phase 4.5 — implemented**, plus the opt-in native MTA core of
**Phase 4.6-A**. Native SMTP relay, an opt-in native MTA, in-memory
and persistent retry queues, custom SMTP headers, string templating and
`ag-observe` metrics are functional. The `mta` feature (off by default) adds a
native outbound MTA: MX resolution, ESMTP+STARTTLS direct delivery, Ed25519 and
RSA-SHA256 DKIM signing, a bounce classifier and MTA metrics. Remaining tech
debt (external template engines; native MTA durable queue/shaping/DSN-FBL/REST
in later phases) is tracked in `docs/DEBT.md`. Decisions:
`docs/adr/0007-ag-mail-ag-domains.md`, `docs/adr/0010-ag-mail-native-mta-pivot.md`.
Technical plans: `docs/rfc/RFC-0006-ag-mail-alcance.md`,
`docs/rfc/RFC-0009-ag-mail-native-mta.md`. Module sheet:
`docs/modules/ag-mail/README.md`.

## Alcance v1

- Relay SMTP nativo (lettre + rustls) y MTA outbound nativo (opt-in).
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
- Ed25519 and RSA-SHA256 DKIM signing applied last (`mail-auth`); key material
  supplied by the caller / `ag-domains`.
- Pure SMTP bounce classifier (transient vs permanent, RFC 3463).
- `ag-observe` metrics on the send path; `mail-mta` CI job.

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

- `smtp` (default): relay SMTP nativo (apuntable a cualquier proveedor externo).
- `templates` (default): built-in StringTemplate rendering.
- `metrics` (default): metricas hacia ag-observe.
- `queue-persistent`: PostgreSQL-backed persistent queue via ag-data (`PersistentQueue`,
  migration in `migrations/0001_mail_queue.sql`).
- `mta` (opt-in): native outbound MTA core (MX resolution, ESMTP+STARTTLS
  delivery, Ed25519/RSA DKIM signing, bounce classification, metrics). Pulls
  `mail-send`, `mail-auth`, `hickory-resolver`, `rustls-pki-types`; not in the
  default build.
